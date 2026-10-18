//! Rechtschreibung und Grammatik mit LTeX+ (LanguageTool, de-DE) als zweitem Sprachserver.
//! LTeX+ ist MPL-2.0 und bringt LanguageTool (LGPL-2.1) und Java mit; wir starten es
//! unverändert als eigenes Programm (keine Bündelung in P5, Lizenzprüfung siehe DECISIONS).
//!
//! Eigene Wörter (Namen, Fachbegriffe): `System/woerterbuch.txt`, ein Wort je Zeile.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde_json::json;

use super::engine::Hinweis;
use super::profil::Schwere;
use crate::lsp::client::LspClient;
use crate::lsp::position::lsp_to_byte;

pub const DICTIONARY_FILE: &str = "System/woerterbuch.txt";
pub const REGEL_ID: &str = "rechtschreibung";
const FUNDSTELLE: &str = "keine FHDW-Vorgabe; Rechtschreibung und Grammatik (LanguageTool)";

#[derive(Debug, Clone)]
struct Diagnostic {
    line: u32,
    character: u32,
    message: String,
    rule: String,
}

#[derive(Default)]
struct Received {
    /// Je Pfad: Anzahl empfangener Meldungsrunden und die letzte Runde.
    by_path: HashMap<String, (usize, Vec<Diagnostic>)>,
}

pub struct LtexSession {
    client: LspClient,
    root: PathBuf,
    received: Arc<(Mutex<Received>, Condvar)>,
    versions: Mutex<HashMap<String, i64>>,
}

/// Liegt `offset` in Fließtext (Markup-Text, auch in Inhaltsblöcken wie `caption: […]`)?
fn in_prose(root: &typst::syntax::SyntaxNode, offset: usize) -> bool {
    use typst::syntax::{LinkedNode, Side, SyntaxKind};
    let Some(leaf) = LinkedNode::new(root).leaf_at(offset, Side::After) else { return false };
    if leaf.kind() != SyntaxKind::Text {
        return false;
    }
    // Text in Code (z. B. Strings) ist kein Markup; der nächste Markup- oder Code-Elternknoten entscheidet.
    let mut node = leaf.parent().cloned();
    while let Some(n) = node {
        match n.kind() {
            SyntaxKind::Markup => return true,
            SyntaxKind::Code | SyntaxKind::Str | SyntaxKind::Raw => return false,
            _ => node = n.parent().cloned(),
        }
    }
    false
}

/// In einer Tabellenzelle (`table`/`grid`), in einem Inhaltsblock als Argument oder im ersten
/// Wort nach einem Code-Ausdruck?
fn structural(root: &typst::syntax::SyntaxNode, text: &str, offset: usize) -> bool {
    use typst::syntax::{LinkedNode, Side, SyntaxKind};
    let Some(leaf) = LinkedNode::new(root).leaf_at(offset, Side::After) else { return false };
    let mut node = leaf.parent().cloned();
    while let Some(n) = node {
        if n.kind() == SyntaxKind::FuncCall {
            let callee = n.children().next().map(|c| &text[c.range()]);
            if matches!(callee, Some("table" | "grid")) {
                return true;
            }
        }
        // Inhaltsblock als Argument (`caption: […]`, `description: […]`): eigener Textschnipsel.
        if n.kind() == SyntaxKind::ContentBlock && n.parent().is_some_and(|p| matches!(p.kind(), SyntaxKind::Named | SyntaxKind::Args)) {
            return true;
        }
        node = n.parent().cloned();
    }
    // Erstes Wort nach Code auf derselben Zeile: davor (über Leerraum) ein Code-Ausdruck.
    let mut prev = leaf.prev_sibling_with_trivia();
    while let Some(p) = prev {
        match p.kind() {
            SyntaxKind::Space if !text[p.range()].contains('\n') => prev = p.prev_sibling_with_trivia(),
            SyntaxKind::FuncCall | SyntaxKind::FieldAccess | SyntaxKind::Ident => return true,
            _ => return false,
        }
    }
    false
}

/// LTeX+ ersetzt Typst-Code (z. B. `#footnote[…]`) durch Platzhalter „DummyN“. Klebt einer an einem
/// Wort („beschriebenDummy3“), kann der Rechtschreibprüfer ihn als Tippfehler melden – das ist kein
/// Fehler im Text des Menschen (je nach Version von LanguageTool/LTeX+).
fn is_placeholder_typo(message: &str) -> bool {
    let word = message.strip_prefix('\'').and_then(|m| m.split('\'').next()).unwrap_or_default();
    match word.find("Dummy") {
        Some(i) => {
            let digits = &word[i + "Dummy".len()..];
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

pub fn find_ltex() -> Option<PathBuf> {
    crate::werkzeuge::find("ltex-ls-plus")
}

/// Wörter aus `System/woerterbuch.txt` (leere Zeilen und `#`-Kommentare ignoriert).
pub fn dictionary(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join(DICTIONARY_FILE))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

impl LtexSession {
    pub fn start(program: &Path, root: &Path) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let settings = json!({
            "language": "de-DE",
            "enabled": ["typst"],
            "dictionary": { "de-DE": dictionary(&root) },
            "checkFrequency": "edit",
            "additionalRules": { "motherTongue": "de-DE" },
        });
        let received: Arc<(Mutex<Received>, Condvar)> = Arc::default();
        let store = received.clone();
        let notify_root = root.clone();
        let client = LspClient::spawn_with(program, &[], settings, move |method, params| {
            if std::env::var_os("LTEX_DEBUG").is_some() {
                eprintln!("ltex <- {method} {}", params.to_string().chars().take(1500).collect::<String>());
            }
            if method != "textDocument/publishDiagnostics" {
                return;
            }
            let Some(path) = params["uri"]
                .as_str()
                .and_then(|u| url::Url::parse(u).ok())
                .and_then(|u| u.to_file_path().ok())
                .and_then(|p| p.strip_prefix(&notify_root).ok().map(|p| p.to_string_lossy().into_owned()))
            else {
                return;
            };
            let list = params["diagnostics"].as_array().cloned().unwrap_or_default();
            let parsed = list
                .iter()
                .filter(|d| !is_placeholder_typo(d["message"].as_str().unwrap_or_default()))
                .map(|d| Diagnostic {
                    line: d["range"]["start"]["line"].as_u64().unwrap_or(0) as u32,
                    character: d["range"]["start"]["character"].as_u64().unwrap_or(0) as u32,
                    message: d["message"].as_str().unwrap_or_default().to_owned(),
                    rule: d["code"].as_str().map(str::to_owned).unwrap_or_default(),
                })
                .collect();
            let (lock, cvar) = &*store;
            let mut received = lock.lock().unwrap();
            let entry = received.by_path.entry(path).or_default();
            entry.0 += 1;
            entry.1 = parsed;
            cvar.notify_all();
        })?;
        client.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": url::Url::from_file_path(&root).map(String::from).unwrap_or_default(),
                "capabilities": { "workspace": { "configuration": true } },
                "initializationOptions": {},
            }),
        )?;
        client.notify("initialized", json!({}))?;
        Ok(Self { client, root, received, versions: Mutex::new(HashMap::new()) })
    }

    /// Dateien (Pfad, Text) prüfen und auf die Meldungen warten (höchstens `timeout`).
    pub fn check(&self, files: &[(String, String)], timeout: Duration) -> Result<Vec<Hinweis>, String> {
        let before: HashMap<String, usize> = {
            let received = self.received.0.lock().unwrap();
            files.iter().map(|(p, _)| (p.clone(), received.by_path.get(p).map_or(0, |e| e.0))).collect()
        };
        for (path, text) in files {
            let uri = url::Url::from_file_path(self.root.join(path)).map(String::from).map_err(|_| "Pfad")?;
            let mut versions = self.versions.lock().unwrap();
            let version = versions.entry(path.clone()).or_insert(0);
            *version += 1;
            if *version == 1 {
                self.client.notify(
                    "textDocument/didOpen",
                    json!({ "textDocument": { "uri": uri, "languageId": "typst", "version": 1, "text": text } }),
                )?;
            } else {
                self.client.notify(
                    "textDocument/didChange",
                    json!({ "textDocument": { "uri": uri, "version": *version }, "contentChanges": [{ "text": text }] }),
                )?;
            }
        }
        // Warten, bis zu jeder Datei eine neue Meldungsrunde da ist.
        let deadline = Instant::now() + timeout;
        let (lock, cvar) = &*self.received;
        let mut received = lock.lock().unwrap();
        loop {
            let done = files.iter().all(|(p, _)| received.by_path.get(p).map_or(0, |e| e.0) > before[p]);
            let now = Instant::now();
            if done || now >= deadline {
                break;
            }
            received = cvar.wait_timeout(received, deadline - now).unwrap().0;
        }
        let mut out = Vec::new();
        for (path, text) in files {
            let syntax = typst::syntax::parse(text);
            for d in received.by_path.get(path).map(|e| e.1.clone()).unwrap_or_default() {
                let offset = lsp_to_byte(text, (d.line, d.character));
                // LTeX+ liest Typst nur grob und meldet auch Code, Labels, Zeichenketten und
                // Funktionsnamen. Nur Treffer im Fließtext zählen.
                if !in_prose(&syntax, offset) {
                    continue;
                }
                // Grammatik (nicht Rechtschreibung) in Tabellenzellen und direkt nach Code:
                // LTeX+ fügt dort Text falsch zusammen (Zellen zu einem Satz, Code fällt weg).
                if d.rule != "GERMAN_SPELLER_RULE" && structural(&syntax, text, offset) {
                    continue;
                }
                out.push(Hinweis {
                    regel_id: REGEL_ID.to_owned(),
                    datei: Some(path.clone()),
                    zeile: Some(d.line as usize + 1),
                    offset: Some(offset),
                    text: if d.rule.is_empty() { d.message } else { format!("{} ({})", d.message, d.rule) },
                    schwere: Schwere::Hinweis,
                    fundstelle: FUNDSTELLE.to_owned(),
                    quelle: "ltex".to_owned(),
                });
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_typos_are_dropped() {
        assert!(is_placeholder_typo("'beschriebenDummy3': Möglicher Tippfehler gefunden. (GERMAN_SPELLER_RULE)"));
        assert!(!is_placeholder_typo("'Tippfeler': Möglicher Tippfehler gefunden. (GERMAN_SPELLER_RULE)"));
        assert!(!is_placeholder_typo("'Dummy': Möglicher Tippfehler gefunden."));
        assert!(!is_placeholder_typo("Satzanfang klein schreiben"));
    }
}
