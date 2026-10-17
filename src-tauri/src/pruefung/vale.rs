//! Stilregeln mit Vale (`typst2vast` liest Typst ohne zu kompilieren). Nur Regeln, die im
//! Profil als `art: stilregel` mit `vale: <Stil.Regel>` stehen, werden zu Hinweisen.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::engine::Hinweis;
use super::profil::{Art, Profil};

/// Konfiguration und Stil FHDW (im Repo unter `sidecars/vale/`; später als App-Ressource).
pub fn config() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../sidecars/vale/.vale.ini"))
}

/// Vale im PATH oder unter /opt/homebrew/bin; `typst2vast` muss für Vale auffindbar sein.
pub fn find_vale() -> Option<PathBuf> {
    search_path().into_iter().map(|d| d.join("vale")).find(|p| p.is_file())
}

fn search_path() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    dirs.push("/opt/homebrew/bin".into());
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(Path::new(&home).join(".cargo/bin"));
    }
    dirs
}

#[derive(Debug, Deserialize)]
struct Alert {
    #[serde(rename = "Check")]
    check: String,
    #[serde(rename = "Line")]
    line: usize,
    #[serde(rename = "Span")]
    span: (usize, usize),
    #[serde(rename = "Message")]
    message: String,
}

/// Vale auf `files` (relativ zu `root`) laufen lassen. Ohne Vale: keine Hinweise und `Err`.
pub fn check(profil: &Profil, root: &Path, files: &[String]) -> Result<Vec<Hinweis>, String> {
    let rules: Vec<_> = profil.regeln.iter().filter(|r| r.art == Some(Art::Stilregel) && r.vale.is_some()).collect();
    if rules.is_empty() || files.is_empty() {
        return Ok(Vec::new());
    }
    let vale = find_vale().ok_or("Vale nicht gefunden (brew install vale)")?;
    let path = std::env::join_paths(search_path()).map_err(|e| e.to_string())?;
    let output = Command::new(vale)
        .current_dir(root)
        .env("PATH", path)
        .arg("--config")
        .arg(config())
        .args(["--output", "JSON", "--no-exit"])
        .args(files)
        .output()
        .map_err(|e| format!("Vale: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let alerts: std::collections::BTreeMap<String, Vec<Alert>> = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("Vale-Ausgabe unlesbar: {e}: {}", String::from_utf8_lossy(&output.stderr)))?;

    let mut out = Vec::new();
    for (file, alerts) in alerts {
        let text = std::fs::read_to_string(root.join(&file)).unwrap_or_default();
        for alert in alerts {
            let Some(regel) = rules.iter().find(|r| r.vale.as_deref() == Some(alert.check.as_str())) else { continue };
            out.push(Hinweis {
                regel_id: regel.id.clone(),
                offset: offset_of(&text, alert.line, alert.span.0),
                zeile: Some(alert.line),
                datei: Some(file.clone()),
                text: alert.message,
                schwere: regel.schwere,
                fundstelle: regel.fundstelle.clone(),
                quelle: "vale".to_owned(),
            });
        }
    }
    Ok(out)
}

/// Byte-Position aus Zeile (ab 1) und Spalte (Zeichen, ab 1).
fn offset_of(text: &str, line: usize, column: usize) -> Option<usize> {
    let start: usize = text.split_inclusive('\n').take(line.saturating_sub(1)).map(str::len).sum();
    let line_text = text.get(start..)?.lines().next()?;
    Some(start + line_text.char_indices().nth(column.saturating_sub(1)).map_or(line_text.len(), |(i, _)| i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_from_line_and_column_with_umlauts() {
        let text = "Erste Zeile\nÜber man reden\n";
        let offset = offset_of(text, 2, 6).unwrap();
        assert!(text[offset..].starts_with("man"));
    }
}
