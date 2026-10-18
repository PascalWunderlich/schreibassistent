//! PDF → Abschnitte mit Seite über Docling (`sidecars/docling/extract.py`, per `uv`).
//! Ergebnis liegt als Cache in `.schreibassistent/chunks/<citekey>.json` (neu erzeugbar,
//! kostet nur Rechenzeit) zusammen mit dem Hash des PDFs.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub const CHUNK_DIR: &str = ".schreibassistent/chunks";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Abschnitt {
    /// PDF-Seite ab 1 (zum Öffnen).
    pub seite: u32,
    /// Gedruckte Seitenzahl (zum Zitieren), z. B. „14“, „iv“ oder „PDF 3“.
    pub label: String,
    pub ueberschrift: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Extrakt {
    pub seiten: u32,
    pub labels: Vec<String>,
    /// `gedruckt` (Kopf-/Fußzeilen), `pdf` (/PageLabels) oder `pdf-seite` (keine Angabe).
    pub labels_quelle: String,
    pub abschnitte: Vec<Abschnitt>,
    pub docling: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Cached {
    quelle_hash: String,
    extrakt: Extrakt,
}

impl Extrakt {
    /// Grobe Token-Schätzung: etwa 4 Zeichen je Token.
    pub fn tokens(&self) -> usize {
        self.abschnitte.iter().map(|a| a.text.len() / 4 + 4).sum()
    }

    /// Erste PDF-Seite mit dieser gedruckten Seitenzahl.
    pub fn pdf_page(&self, label: &str) -> Option<u32> {
        self.labels.iter().position(|l| l == label).map(|i| i as u32 + 1)
    }
}

pub fn file_hash(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}

pub(super) fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../sidecars/docling/extract.py")
}

pub fn find_uv() -> Option<PathBuf> {
    crate::werkzeuge::find("uv")
}

fn cache_file(root: &Path, citekey: &str) -> PathBuf {
    root.join(CHUNK_DIR).join(format!("{citekey}.json"))
}

/// Gespeichertes Ergebnis, wenn das PDF seitdem gleich ist.
pub fn cached(root: &Path, citekey: &str, quelle_hash: &str) -> Option<Extrakt> {
    let text = std::fs::read_to_string(cache_file(root, citekey)).ok()?;
    let cached: Cached = serde_json::from_str(&text).ok()?;
    (cached.quelle_hash == quelle_hash).then_some(cached.extrakt)
}

/// Gespeichertes Ergebnis ohne Abgleich mit dem PDF (wenn Zotero nicht läuft, P8).
pub fn cached_any(root: &Path, citekey: &str) -> Option<Extrakt> {
    let text = std::fs::read_to_string(cache_file(root, citekey)).ok()?;
    serde_json::from_str::<Cached>(&text).ok().map(|c| c.extrakt)
}

/// Docling ausführen (dauert je nach Umfang 10–60 s) und das Ergebnis speichern.
pub fn extract(root: &Path, citekey: &str, pdf: &Path, quelle_hash: &str) -> Result<Extrakt, String> {
    let uv = find_uv().ok_or("uv nicht gefunden (Installation der App unvollständig)")?;
    super::docling::ensure_ready()?;
    let out = std::env::temp_dir().join(format!("sa-docling-{citekey}-{}.json", std::process::id()));
    let mut command = std::process::Command::new(uv);
    command.args(["run", "-q"]).arg(script()).arg(pdf).arg(&out);
    super::docling::apply_env(&mut command);
    let output = command.output().map_err(|e| format!("Docling: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let last = stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("unbekannter Fehler");
        return Err(format!("Docling: {last}"));
    }
    let text = std::fs::read_to_string(&out).map_err(|e| format!("Docling-Ausgabe: {e}"))?;
    let _ = std::fs::remove_file(&out);
    let extrakt: Extrakt = serde_json::from_str(&text).map_err(|e| format!("Docling-Ausgabe: {e}"))?;
    if extrakt.abschnitte.is_empty() {
        return Err("Kein Text im PDF gefunden (gescanntes PDF? Texterkennung ist aus)".into());
    }
    let file = cache_file(root, citekey);
    std::fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
    let cached = Cached { quelle_hash: quelle_hash.to_owned(), extrakt: extrakt.clone() };
    crate::projekt::files::atomic_write(&file, &serde_json::to_string(&cached).map_err(|e| e.to_string())?)?;
    Ok(extrakt)
}

#[cfg(test)]
pub fn sample() -> Extrakt {
    let a = |seite: u32, label: &str, ueberschrift: &str, text: &str| Abschnitt {
        seite,
        label: label.into(),
        ueberschrift: ueberschrift.into(),
        text: text.into(),
    };
    Extrakt {
        seiten: 4,
        labels: vec!["i".into(), "1".into(), "2".into(), "3".into()],
        labels_quelle: "gedruckt".into(),
        abschnitte: vec![
            a(1, "i", "Abstract", "Microservices are small autonomous services that work together."),
            a(2, "1", "1 Introduction", "The monolith is split into services along business capabili- ties to allow independent deployment."),
            a(3, "2", "2 Migration", "Most companies migrate step by step and keep the monolith running during the transition."),
            a(4, "3", "3 Challenges", "Teams reported that data decomposition was the hardest part of the migration."),
        ],
        docling: "test".into(),
    }
}
