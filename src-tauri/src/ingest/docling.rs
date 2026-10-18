//! Docling-Erstinstallation: `uv` (liegt in der App) lädt Python, Docling und die Modelle in einen
//! festen Ordner der App-Daten. Das braucht Netz und rund 2 GB und geschieht nur nach Bestätigung
//! (Leitplanke 4: einmalige Erstinstallation). Danach läuft das Einlesen lokal.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Grob geschätzte Größe von Python, Docling, PyTorch und Layout-Modell (für die Rückfrage).
pub const GROESSE_MB: u32 = 2000;

const MARKER: &str = "bereit.txt";

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Ordner für Docling merken (beim Start: App-Daten). Ohne `init` (Tests, Beispielprogramme)
/// gilt die Umgebung des Rechners unverändert.
pub fn init(dir: PathBuf) {
    let _ = DIR.set(dir);
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct WerkzeugStatus {
    /// Docling ist eingerichtet (oder die App läuft ohne eigenen Ordner, z. B. im Test).
    pub bereit: bool,
    /// `uv` wurde gefunden.
    pub uv: bool,
    pub groesse_mb: u32,
    pub ordner: Option<String>,
}

pub fn status() -> WerkzeugStatus {
    status_in(DIR.get().map(PathBuf::as_path), super::extrakt::find_uv().is_some())
}

fn status_in(dir: Option<&Path>, uv: bool) -> WerkzeugStatus {
    WerkzeugStatus {
        bereit: dir.is_none_or(|d| d.join(MARKER).is_file()),
        uv,
        groesse_mb: GROESSE_MB,
        ordner: dir.map(|d| d.to_string_lossy().into_owned()),
    }
}

/// Meldung, wenn Docling noch nicht eingerichtet ist (für das Einlesen).
pub fn ensure_ready() -> Result<(), String> {
    if status().bereit {
        Ok(())
    } else {
        Err("Docling ist noch nicht eingerichtet. Es wird einmalig geladen (ca. 2 GB).".into())
    }
}

/// `uv` auf den festen Ordner einstellen: Cache, Python und Modelle liegen in den App-Daten.
pub fn apply_env(command: &mut Command) {
    let Some(dir) = DIR.get() else { return };
    command
        .env("UV_CACHE_DIR", dir.join("uv-cache"))
        .env("UV_PYTHON_INSTALL_DIR", dir.join("python"))
        .env("HF_HOME", dir.join("huggingface"))
        .env("DOCLING_CACHE_DIR", dir.join("docling"));
}

/// Docling einrichten. `progress` bekommt die Meldungen von `uv` zeilenweise.
pub fn install(mut progress: impl FnMut(&str)) -> Result<(), String> {
    let dir = DIR.get().ok_or("Kein Ordner für Docling festgelegt")?;
    let uv = super::extrakt::find_uv().ok_or("uv nicht gefunden (Installation der App unvollständig)")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let _ = std::fs::remove_file(dir.join(MARKER));

    let mut command = Command::new(uv);
    command.args(["run"]).arg(super::extrakt::script()).arg("--vorbereiten");
    apply_env(&mut command);
    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("uv: {e}"))?;

    // stdout ungelesen zu lassen könnte den Prozess blockieren; beide Ströme sammeln.
    let stdout = child.stdout.take().map(|s| std::thread::spawn(move || read_all(s)));
    let mut tail: Vec<String> = Vec::new();
    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr).split(b'\n').filter_map(Result::ok) {
            // uv überschreibt seine Fortschrittszeile mit \r; auch daran trennen.
            for part in String::from_utf8_lossy(&line).split('\r') {
                let part = part.trim();
                if !part.is_empty() {
                    progress(part);
                    tail.push(part.to_owned());
                    if tail.len() > 20 {
                        tail.remove(0);
                    }
                }
            }
        }
    }
    let status = child.wait().map_err(|e| format!("uv: {e}"))?;
    if let Some(handle) = stdout {
        let _ = handle.join();
    }
    if !status.success() {
        return Err(failure_message(&tail));
    }
    std::fs::write(dir.join(MARKER), "Docling eingerichtet\n").map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(())
}

fn read_all(mut r: impl std::io::Read) -> String {
    let mut s = String::new();
    let _ = r.read_to_string(&mut s);
    s
}

/// Verständliche Meldung aus den letzten Zeilen von `uv`.
fn failure_message(tail: &[String]) -> String {
    let text = tail.join("\n").to_lowercase();
    let offline = ["error sending request", "dns error", "failed to lookup", "could not resolve", "network is unreachable", "connection refused", "timed out", "connect error"];
    if offline.iter().any(|m| text.contains(m)) {
        return "Keine Internetverbindung. Bitte mit dem Netz verbinden und erneut versuchen.".into();
    }
    if text.contains("no space left") || text.contains("not enough space") || text.contains("disk full") {
        return "Nicht genug Speicherplatz (Docling braucht rund 2 GB).".into();
    }
    let last = tail.iter().rev().find(|l| !l.trim().is_empty()).map_or("unbekannter Fehler", String::as_str);
    format!("Docling konnte nicht eingerichtet werden: {last}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_folder_docling_counts_as_ready() {
        assert!(status_in(None, true).bereit);
    }

    #[test]
    fn ready_only_with_marker() {
        let dir = std::env::temp_dir().join(format!("sa-docling-status-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!status_in(Some(&dir), true).bereit);
        std::fs::write(dir.join(MARKER), "").unwrap();
        let status = status_in(Some(&dir), false);
        assert!(status.bereit && !status.uv);
        assert_eq!(status.groesse_mb, GROESSE_MB);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn offline_failure_gets_a_clear_message() {
        let tail = vec!["error: Failed to fetch: `https://pypi.org/simple/docling/`".to_owned(), "Caused by: error sending request for url".to_owned()];
        assert!(failure_message(&tail).starts_with("Keine Internetverbindung"));
    }

    #[test]
    fn other_failures_show_the_last_line() {
        let tail = vec!["Resolved 3 packages".to_owned(), "error: No solution found".to_owned()];
        assert_eq!(failure_message(&tail), "Docling konnte nicht eingerichtet werden: error: No solution found");
        assert!(failure_message(&["write: No space left on device".to_owned()]).contains("Speicherplatz"));
    }
}
