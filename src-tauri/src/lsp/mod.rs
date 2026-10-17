//! Tinymist starten, LSP zwischen CodeMirror und Tinymist.
//!
//! Tinymist läuft als Sprachserver für den Arbeits-Vault. Im Seiteneditor bearbeitet man
//! nur einen Absatz („Feld“); er wird in den Quelltext der Datei eingesetzt und die
//! Positionen werden zwischen Feld (UTF-16, wie CodeMirror) und Datei (LSP: Zeile/
//! UTF-16-Spalte) umgerechnet.
//!
//! Tinymist 0.15.10 übernimmt in unserer Konstellation Texte aus `didOpen`/`didChange`
//! nicht zuverlässig, liest aber geänderte Dateien nach ~50 ms von der Platte. Deshalb
//! schreibt das Projekt den Stand (wie beim Speichern) und meldet ihn mit
//! `workspace/didChangeWatchedFiles` (`file_changed`).

pub mod client;
pub mod commands;
pub mod position;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use client::LspClient;
use position::{byte_to_lsp, field_byte, field_utf16, lsp_to_byte};

pub const NAME: &str = "lsp";

/// Absatz, der gerade im Feld bearbeitet wird: `start..end` im Quelltext der Datei
/// (Stand des Satzes) wird durch `text` ersetzt.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Field {
    pub file: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// Quelltextmodus: `text` ist die ganze Datei (`start`/`end` werden ignoriert).
    #[serde(default)]
    pub whole_file: bool,
}

impl Field {
    /// Beginn des Feldes in der Datei (Byte).
    pub fn offset(&self) -> usize {
        if self.whole_file { 0 } else { self.start }
    }

    /// Quelltext der Datei mit eingesetztem Feld.
    pub fn splice(&self, base: &str) -> Result<String, String> {
        if self.whole_file {
            return Ok(self.text.clone());
        }
        if self.start > self.end || self.end > base.len() || !base.is_char_boundary(self.start) || !base.is_char_boundary(self.end) {
            return Err(format!("Ungültiger Bereich {}..{} in {}", self.start, self.end, self.file));
        }
        Ok(format!("{}{}{}", &base[..self.start], self.text, &base[self.end..]))
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Completion {
    pub label: String,
    pub detail: Option<String>,
    /// LSP-Art (1 Text, 3 Funktion, 6 Variable, 18 Verweis …).
    pub kind: Option<u64>,
    /// Zu ersetzender Bereich im Feld (UTF-16) und Ersatztext.
    pub from: usize,
    pub to: usize,
    pub insert: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct FieldDiagnostic {
    /// Bereich im Feld (UTF-16).
    pub from: usize,
    pub to: usize,
    /// "error" | "warning" | "info"
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Definition {
    /// Pfad relativ zum Vault, falls das Ziel im Vault liegt.
    pub file: Option<String>,
    /// Absoluter Pfad (z. B. für Ziele in Paketen).
    pub absolute: String,
    /// Byte-Position im Ziel und Zeile (0-basiert).
    pub offset: usize,
    pub line: u32,
}

#[derive(Debug, Clone)]
struct RawDiagnostic {
    start: (u32, u32),
    end: (u32, u32),
    severity: u64,
    message: String,
}

type Diagnostics = Arc<Mutex<HashMap<String, Vec<RawDiagnostic>>>>;

pub struct TinymistSession {
    client: LspClient,
    root: PathBuf,
    diagnostics: Diagnostics,
}

impl TinymistSession {
    /// Startet Tinymist für den Arbeits-Vault `root` mit der Hauptdatei `main`.
    /// `on_diagnostics` wird mit dem Pfad (relativ zum Vault) aufgerufen, wenn neue
    /// Fehlermeldungen für eine Datei vorliegen.
    pub fn start(
        program: &Path,
        root: &Path,
        main: &str,
        on_diagnostics: impl Fn(&str) + Send + 'static,
    ) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
        let diagnostics: Diagnostics = Arc::default();
        let store = diagnostics.clone();
        let notify_root = root.clone();
        let client = LspClient::spawn(program, move |method, params| {
            if method != "textDocument/publishDiagnostics" {
                return;
            }
            let Some(path) = params["uri"].as_str().and_then(|u| relative_from_uri(&notify_root, u)) else { return };
            let list = params["diagnostics"].as_array().cloned().unwrap_or_default();
            let parsed = list
                .iter()
                .map(|d| RawDiagnostic {
                    start: lsp_pos(&d["range"]["start"]),
                    end: lsp_pos(&d["range"]["end"]),
                    severity: d["severity"].as_u64().unwrap_or(1),
                    message: d["message"].as_str().unwrap_or_default().to_owned(),
                })
                .collect();
            store.lock().unwrap().insert(path.clone(), parsed);
            on_diagnostics(&path);
        })?;
        client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": uri(&root)?,
                "capabilities": {
                    "textDocument": {
                        "completion": { "completionItem": { "snippetSupport": false } },
                        "hover": { "contentFormat": ["plaintext", "markdown"] },
                    }
                },
                "initializationOptions": {},
            }),
        )?;
        client.notify("initialized", json!({}))?;
        // Kapiteldateien im Zusammenhang der ganzen Arbeit verstehen (Literatur, Labels).
        client.request(
            "workspace/executeCommand",
            json!({ "command": "tinymist.pinMain", "arguments": [root.join(main).to_string_lossy()] }),
        )?;
        Ok(Self { client, root, diagnostics })
    }

    /// Tinymist mitteilen, dass `path` (relativ zum Vault) auf der Platte geändert wurde.
    pub fn file_changed(&self, path: &str) -> Result<(), String> {
        self.client.notify(
            "workspace/didChangeWatchedFiles",
            json!({ "changes": [{ "uri": uri(&self.root.join(path))?, "type": 2 }] }),
        )
    }

    /// LSP-Position der Cursorposition `cursor` (UTF-16 im Feld); `full` ist der Text der
    /// Datei mit eingesetztem Feld, wie er auf der Platte steht.
    fn position(&self, full: &str, field: &Field, cursor: usize) -> Result<Value, String> {
        let byte = field.offset() + field_byte(&field.text, cursor);
        let (line, character) = byte_to_lsp(full, byte);
        Ok(json!({
            "textDocument": { "uri": uri(&self.root.join(&field.file))? },
            "position": { "line": line, "character": character },
        }))
    }

    /// Vervollständigung an der Cursorposition `cursor` (UTF-16 im Feld).
    pub fn completion(&self, full: &str, field: &Field, cursor: usize) -> Result<Vec<Completion>, String> {
        let params = self.position(full, field, cursor)?;
        let result = self.client.request("textDocument/completion", params)?;
        let items = result.get("items").or(Some(&result)).and_then(Value::as_array).cloned().unwrap_or_default();
        let field_start = field.offset();
        let field_end = field_start + field.text.len();
        Ok(items
            .iter()
            .filter_map(|item| {
                let label = item["label"].as_str()?.to_owned();
                let edit = item.get("textEdit");
                let (from, to, insert) = match edit {
                    Some(edit) => {
                        let range = edit.get("range").or_else(|| edit.get("replace"))?;
                        let a = lsp_to_byte(full, lsp_pos(&range["start"]));
                        let b = lsp_to_byte(full, lsp_pos(&range["end"]));
                        if a < field_start || b > field_end {
                            return None;
                        }
                        let from = field_utf16(&field.text, a - field_start);
                        let to = field_utf16(&field.text, b - field_start);
                        (from, to, edit["newText"].as_str()?.to_owned())
                    }
                    None => (cursor, cursor, item["insertText"].as_str().unwrap_or(&label).to_owned()),
                };
                Some(Completion {
                    detail: item["detail"].as_str().map(str::to_owned),
                    kind: item["kind"].as_u64(),
                    insert: strip_snippet(&insert),
                    label,
                    from,
                    to,
                })
            })
            .collect())
    }

    /// Hover-Text an der Cursorposition (Klartext oder Markdown).
    pub fn hover(&self, full: &str, field: &Field, cursor: usize) -> Result<Option<String>, String> {
        let params = self.position(full, field, cursor)?;
        let result = self.client.request("textDocument/hover", params)?;
        let contents = &result["contents"];
        let text = match contents {
            Value::String(s) => Some(s.clone()),
            Value::Object(o) => o.get("value").and_then(Value::as_str).map(str::to_owned),
            Value::Array(a) => Some(
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned).or_else(|| v["value"].as_str().map(str::to_owned)))
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            ),
            _ => None,
        };
        Ok(text.filter(|t| !t.trim().is_empty()))
    }

    /// Ziel der Definition an der Cursorposition.
    pub fn definition(&self, full: &str, field: &Field, cursor: usize) -> Result<Option<Definition>, String> {
        let params = self.position(full, field, cursor)?;
        let result = self.client.request("textDocument/definition", params)?;
        let location = match &result {
            Value::Array(a) => a.first().cloned(),
            Value::Null => None,
            other => Some(other.clone()),
        };
        let Some(location) = location else { return Ok(None) };
        let target_uri = location.get("uri").or_else(|| location.get("targetUri")).and_then(Value::as_str).unwrap_or_default();
        let range = location.get("range").or_else(|| location.get("targetSelectionRange")).cloned().unwrap_or_default();
        let Some(absolute) = url::Url::parse(target_uri).ok().and_then(|u| u.to_file_path().ok()) else { return Ok(None) };
        let (line, character) = lsp_pos(&range["start"]);
        let text = std::fs::read_to_string(&absolute).unwrap_or_default();
        Ok(Some(Definition {
            file: absolute.strip_prefix(&self.root).ok().map(|p| p.to_string_lossy().into_owned()),
            absolute: absolute.to_string_lossy().into_owned(),
            offset: lsp_to_byte(&text, (line, character)),
            line,
        }))
    }

    /// Fehlermeldungen von Tinymist, die im Feld liegen (Positionen im Feld, UTF-16).
    pub fn field_diagnostics(&self, full: &str, field: &Field) -> Result<Vec<FieldDiagnostic>, String> {
        let field_start = field.offset();
        let field_end = field_start + field.text.len();
        let all = self.diagnostics.lock().unwrap().get(&field.file).cloned().unwrap_or_default();
        Ok(all
            .iter()
            .filter_map(|d| {
                let a = lsp_to_byte(full, d.start).max(field_start);
                let b = lsp_to_byte(full, d.end).min(field_end);
                if lsp_to_byte(full, d.start) > field_end || lsp_to_byte(full, d.end) < field_start {
                    return None;
                }
                Some(FieldDiagnostic {
                    from: field_utf16(&field.text, a - field_start),
                    to: field_utf16(&field.text, b.max(a) - field_start),
                    severity: match d.severity {
                        1 => "error",
                        2 => "warning",
                        _ => "info",
                    }
                    .to_owned(),
                    message: d.message.clone(),
                })
            })
            .collect())
    }

    /// Anzahl der Fehler (Schwere „error“) über alle Dateien.
    pub fn error_count(&self) -> usize {
        self.diagnostics.lock().unwrap().values().flatten().filter(|d| d.severity == 1).count()
    }
}

fn lsp_pos(value: &Value) -> (u32, u32) {
    (value["line"].as_u64().unwrap_or(0) as u32, value["character"].as_u64().unwrap_or(0) as u32)
}

/// Platzhalter aus Snippets entfernen: `${1:text}` → `text`, `$1`/`$0` → nichts.
fn strip_snippet(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '$' if chars.peek() == Some(&'{') => {
                chars.next();
                while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    chars.next();
                }
                if chars.peek() == Some(&':') {
                    chars.next();
                }
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    out.push(c);
                }
            }
            '$' if chars.peek().is_some_and(|c| c.is_ascii_digit()) => {
                while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    chars.next();
                }
            }
            c => out.push(c),
        }
    }
    out
}

fn uri(path: &Path) -> Result<String, String> {
    url::Url::from_file_path(path).map(String::from).map_err(|_| format!("Kein absoluter Pfad: {}", path.display()))
}

fn relative_from_uri(root: &Path, file_uri: &str) -> Option<String> {
    let path = url::Url::parse(file_uri).ok()?.to_file_path().ok()?;
    path.strip_prefix(root).ok().map(|p| p.to_string_lossy().into_owned())
}

/// Tinymist im PATH oder unter /opt/homebrew/bin (später als Sidecar, siehe ROADMAP).
pub fn find_tinymist() -> Option<PathBuf> {
    let from_path = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|dir| dir.join("tinymist"));
    from_path.chain([PathBuf::from("/opt/homebrew/bin/tinymist")]).find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "lsp");
    }

    #[test]
    fn strips_snippet_placeholders() {
        assert_eq!(strip_snippet("lorem(${1:words})$0"), "lorem(words)");
        assert_eq!(strip_snippet("@sutton2018"), "@sutton2018");
        assert_eq!(strip_snippet("a\\$b"), "a$b");
    }

    /// Kopie der Beispielarbeit: Der Test schreibt Dateien.
    fn example_copy() -> PathBuf {
        let target = std::env::temp_dir().join(format!("sa-lsp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&target);
        let source = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
        assert!(std::process::Command::new("cp").arg("-R").arg(source).arg(&target).status().unwrap().success());
        target.canonicalize().unwrap()
    }

    /// Feld einsetzen, schreiben, Tinymist melden (wie das Projekt es tut).
    fn write_field(session: &TinymistSession, root: &Path, base: &str, field: &Field) -> String {
        let full = field.splice(base).unwrap();
        std::fs::write(root.join(&field.file), &full).unwrap();
        session.file_changed(&field.file).unwrap();
        std::thread::sleep(Duration::from_millis(150));
        full
    }

    /// Braucht ein installiertes Tinymist; sonst wird der Test übersprungen.
    #[test]
    fn completion_hover_definition_and_diagnostics_in_a_field() {
        let Some(program) = find_tinymist() else {
            eprintln!("tinymist nicht gefunden, Test übersprungen");
            return;
        };
        let root = example_copy();
        let (tx, rx) = channel();
        let session = TinymistSession::start(&program, &root, "Arbeit/main.typ", move |path| {
            let _ = tx.send(path.to_owned());
        })
        .expect("Tinymist startet");

        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let base = std::fs::read_to_string(root.join(path)).unwrap();
        let start = base.find("Die Modellierung erfolgt").unwrap();
        let end = start + base[start..].find('\n').unwrap();

        // Hover auf einem Citekey, der nur im Feld steht.
        let field = Field { file: path.into(), start, end, whole_file: false, text: "Siehe @sutton2018 dazu.".into() };
        let mut hover = None;
        for _ in 0..30 {
            let full = write_field(&session, &root, &base, &field);
            hover = session.hover(&full, &field, 9).unwrap();
            if hover.as_ref().is_some_and(|h| h.contains("sutton2018")) {
                break;
            }
        }
        assert!(hover.is_some_and(|h| h.to_lowercase().contains("reinforcement")), "Hover zeigt die Quelle");

        // Vervollständigung nach `@sut`: Citekey mit Bereich im Feld.
        let field = Field { file: path.into(), start, end, whole_file: false, text: "Wie gezeigt @sut".into() };
        let full = write_field(&session, &root, &base, &field);
        let items = session.completion(&full, &field, field.text.len()).unwrap();
        let item = items
            .iter()
            .find(|i| i.label.contains("sutton2018"))
            .unwrap_or_else(|| panic!("Citekey vorgeschlagen: {items:?}"));
        assert!(item.to <= field.text.len() && item.from <= item.to, "{item:?}");

        // Sprung zur Definition eines Labels in einer anderen Datei.
        let field = Field { file: path.into(), start, end, whole_file: false, text: "Wie in @kap:grundlagen.".into() };
        let full = write_field(&session, &root, &base, &field);
        let target = session.definition(&full, &field, 10).unwrap().expect("Ziel");
        assert_eq!(target.file.as_deref(), Some("Arbeit/kapitel/02-grundlagen.typ"));

        // Fehler im Feld: unbekannte Variable wird gemeldet, Position im Feld.
        let field = Field { file: path.into(), start, end, whole_file: false, text: "Text #gibtsnicht hier.".into() };
        let full = write_field(&session, &root, &base, &field);
        let mut found = Vec::new();
        for _ in 0..20 {
            let _ = rx.recv_timeout(Duration::from_millis(500));
            found = session.field_diagnostics(&full, &field).unwrap();
            if !found.is_empty() {
                break;
            }
        }
        let diagnostic = found.first().expect("Fehler im Feld");
        assert_eq!(&field.text[diagnostic.from..diagnostic.to], "gibtsnicht", "{diagnostic:?}");
        assert_eq!(diagnostic.severity, "error");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
