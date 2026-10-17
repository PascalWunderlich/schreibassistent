//! Tinymist starten, LSP zwischen CodeMirror und Tinymist.
//!
//! P2 (Prototyp B): Tinymist als Sprachserver mit Live-Vorschau. Änderungen gehen als
//! `didChange` an Tinymist (nichts wird gespeichert); die Vorschau läuft lokal unter
//! 127.0.0.1 und meldet Klicks als `tinymist/preview/scrollSource` zurück.

pub mod client;
pub mod commands;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Value, json};

use client::LspClient;

pub const NAME: &str = "lsp";

const PREVIEW_TASK: &str = "schreibassistent";

pub struct TinymistSession {
    client: LspClient,
    root: PathBuf,
    versions: Mutex<HashMap<String, i64>>,
    pub preview_port: u16,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceFile {
    /// Pfad relativ zum Arbeits-Vault.
    pub path: String,
    pub text: String,
}

impl TinymistSession {
    /// Startet Tinymist für den Arbeits-Vault `root`, öffnet alle `.typ`-Dateien unter
    /// `Arbeit/` und startet die Vorschau für `main` (relativ zu `root`).
    pub fn start(
        program: &Path,
        root: &Path,
        main: &str,
        on_notification: impl Fn(&str, Value) + Send + 'static,
    ) -> Result<(Self, Vec<SourceFile>), String> {
        let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
        let client = LspClient::spawn(program, on_notification)?;
        client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": uri(&root)?,
                "capabilities": {},
                "initializationOptions": { "compileStatus": "enable" },
            }),
        )?;
        client.notify("initialized", json!({}))?;

        let mut files = Vec::new();
        collect_typ_files(&root, &root.join("Arbeit"), &mut files)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        for file in &files {
            client.notify(
                "textDocument/didOpen",
                json!({ "textDocument": {
                    "uri": uri(&root.join(&file.path))?,
                    "languageId": "typst",
                    "version": 1,
                    "text": file.text,
                }}),
            )?;
        }

        let result = client.request(
            "workspace/executeCommand",
            json!({
                "command": "tinymist.doStartPreview",
                "arguments": [[
                    "--task-id", PREVIEW_TASK,
                    "--data-plane-host=127.0.0.1:0",
                    "--no-open",
                    root.join(main).to_string_lossy(),
                ]],
            }),
        )?;
        let preview_port = result
            .get("staticServerPort")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("Vorschau ohne Port: {result}"))? as u16;

        let versions = files.iter().map(|f| (f.path.clone(), 1)).collect();
        Ok((Self { client, root, versions: Mutex::new(versions), preview_port }, files))
    }

    /// Neuer Inhalt einer Datei (nur im Speicher von Tinymist).
    pub fn change(&self, path: &str, text: &str) -> Result<(), String> {
        let version = {
            let mut versions = self.versions.lock().unwrap();
            let version = versions.entry(path.to_owned()).or_insert(1);
            *version += 1;
            *version
        };
        self.client.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri(&self.root.join(path))?, "version": version },
                "contentChanges": [{ "text": text }],
            }),
        )
    }

    /// Vorschau zur Stelle `line`/`character` (0-basiert, UTF-16) in `path` bewegen.
    pub fn scroll_preview(&self, path: &str, line: u32, character: u32) -> Result<(), String> {
        self.client
            .request(
                "workspace/executeCommand",
                json!({
                    "command": "tinymist.scrollPreview",
                    "arguments": [PREVIEW_TASK, {
                        "event": "panelScrollTo",
                        "filepath": self.root.join(path).to_string_lossy(),
                        "line": line,
                        "character": character,
                    }],
                }),
            )
            .map(|_| ())
    }

    /// Absoluten Pfad aus einer Tinymist-Meldung in einen Pfad relativ zum Vault umwandeln.
    pub fn relative(&self, absolute: &str) -> Option<String> {
        Path::new(absolute)
            .strip_prefix(&self.root)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    }
}

fn uri(path: &Path) -> Result<String, String> {
    url::Url::from_file_path(path)
        .map(String::from)
        .map_err(|_| format!("Kein absoluter Pfad: {}", path.display()))
}

fn collect_typ_files(root: &Path, dir: &Path, out: &mut Vec<SourceFile>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_typ_files(root, &path, out)?;
        } else if path.extension().is_some_and(|e| e == "typ") {
            out.push(SourceFile {
                path: path.strip_prefix(root).unwrap().to_string_lossy().into_owned(),
                text: std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?,
            });
        }
    }
    Ok(())
}

/// Tinymist im PATH oder unter /opt/homebrew/bin (P3: als Sidecar mitliefern).
pub fn find_tinymist() -> Option<PathBuf> {
    let from_path = std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|dir| dir.join("tinymist"));
    from_path
        .chain([PathBuf::from("/opt/homebrew/bin/tinymist")])
        .find(|p| p.is_file())
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

    /// Braucht ein installiertes Tinymist; sonst wird der Test übersprungen.
    #[test]
    fn starts_preview_and_recompiles_on_change() {
        let Some(program) = find_tinymist() else {
            eprintln!("tinymist nicht gefunden, Test übersprungen");
            return;
        };
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
        let (tx, rx) = channel();
        let (session, files) = TinymistSession::start(&program, root, "Arbeit/main.typ", move |method, params| {
            if method == "tinymist/compileStatus" {
                let _ = tx.send(params);
            }
        })
        .expect("Tinymist startet");
        assert!(session.preview_port > 0);
        assert!(files.iter().any(|f| f.path == "Arbeit/kapitel/01-einleitung.typ"));

        while rx.recv_timeout(Duration::from_millis(500)).is_ok() {}
        let file = files.iter().find(|f| f.path.ends_with("01-einleitung.typ")).unwrap();
        session.change(&file.path, &format!("{}\nNeuer Satz.\n", file.text)).unwrap();
        let status = rx.recv_timeout(Duration::from_secs(10)).expect("compileStatus nach Änderung");
        assert!(status.get("status").is_some(), "{status}");
        session.scroll_preview(&file.path, 2, 0).unwrap();
        assert_eq!(session.relative(&root.canonicalize().unwrap().join(&file.path).to_string_lossy()).unwrap(), file.path);
    }
}
