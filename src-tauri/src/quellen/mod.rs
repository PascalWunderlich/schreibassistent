//! quellen.bib lesen, Zotero Local API (localhost:23119).
//!
//! Bibliografische Daten stehen nur in Zotero; die App liest den Better-BibTeX-Export
//! `System/quellen.bib` (docs/DATA-FORMATS.md, Wissensmodell im Vault).

pub mod commands;
pub mod zotero;

use std::path::Path;

use biblatex::{Bibliography, ChunksExt};

pub const NAME: &str = "quellen";

/// Pfad des Exports im Arbeits-Vault.
pub const BIB_FILE: &str = "System/quellen.bib";

/// Eine Quelle aus der `.bib`, aufbereitet für die Anzeige.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Source {
    pub citekey: String,
    /// BibTeX-Typ (`book`, `article`, `inproceedings`, `misc` …).
    pub kind: String,
    /// Autoren bzw. Herausgeber, je „Nachname, Vorname“.
    pub authors: Vec<String>,
    pub title: Option<String>,
    pub year: Option<String>,
    /// Zeitschrift, Tagungsband oder Erscheinungsort im Netz.
    pub container: Option<String>,
    pub publisher: Option<String>,
    pub url: Option<String>,
    pub doi: Option<String>,
    /// Dateipfad des PDFs, falls Better BibTeX ihn exportiert (`file`).
    pub file: Option<String>,
    pub volume: Option<String>,
    pub pages: Option<String>,
    /// Zugriffsdatum (Internetquellen).
    pub urldate: Option<String>,
    pub editors: Vec<String>,
    /// Zeile des Eintrags in der `.bib` (ab 1).
    pub line: usize,
}

/// Eine `.bib`-Datei als `System/quellen.bib` übernehmen (Ersteinstieg „.bib wählen“). Die Datei muss
/// lesbar sein; vorhandene Quellen werden nie überschrieben.
pub fn adopt_bib(root: &Path, from: &Path) -> Result<usize, String> {
    let text = std::fs::read_to_string(from).map_err(|e| format!("{}: {e}", from.display()))?;
    let sources = parse(&text).map_err(|e| format!("Keine lesbare .bib-Datei: {e}"))?;
    if sources.is_empty() {
        return Err("In der Datei stehen keine Quellen.".into());
    }
    if !load(root)?.is_empty() {
        return Err("Die Arbeit hat schon Quellen (System/quellen.bib). Sie werden nicht überschrieben.".into());
    }
    crate::projekt::files::atomic_write(&root.join(BIB_FILE), &text)?;
    Ok(sources.len())
}

/// Liest `System/quellen.bib`. Fehlt die Datei, gibt es keine Quellen (kein Fehler).
pub fn load(root: &Path) -> Result<Vec<Source>, String> {
    let path = root.join(BIB_FILE);
    let Ok(text) = std::fs::read_to_string(&path) else { return Ok(Vec::new()) };
    parse(&text)
}

pub fn parse(text: &str) -> Result<Vec<Source>, String> {
    let bibliography = Bibliography::parse(text).map_err(|e| format!("{BIB_FILE}: {e}"))?;
    let field = |entry: &biblatex::Entry, name: &str| {
        entry.fields.get(name).map(|c| c.format_verbatim()).map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
    };
    let mut sources: Vec<Source> = bibliography
        .iter()
        .map(|entry| {
            let people = field(entry, "author").or_else(|| field(entry, "editor")).unwrap_or_default();
            let split = |s: Option<String>| -> Vec<String> {
                s.unwrap_or_default().split(" and ").map(|p| p.trim().to_owned()).filter(|p| !p.is_empty()).collect()
            };
            let marker = text.find(&format!("{{{},", entry.key)).unwrap_or(0);
            Source {
                volume: field(entry, "volume"),
                pages: field(entry, "pages"),
                urldate: field(entry, "urldate"),
                editors: split(field(entry, "editor")),
                line: text[..marker].matches('\n').count() + 1,
                citekey: entry.key.clone(),
                kind: entry.entry_type.to_string().to_lowercase(),
                authors: people.split(" and ").map(|p| p.trim().to_owned()).filter(|p| !p.is_empty()).collect(),
                title: field(entry, "title"),
                year: field(entry, "year").or_else(|| field(entry, "date").map(|d| d.chars().take(4).collect())),
                container: field(entry, "journal")
                    .or_else(|| field(entry, "journaltitle"))
                    .or_else(|| field(entry, "booktitle"))
                    .or_else(|| field(entry, "howpublished")),
                publisher: field(entry, "publisher"),
                url: field(entry, "url"),
                doi: field(entry, "doi"),
                file: field(entry, "file"),
            }
        })
        .collect();
    sources.sort_by(|a, b| a.citekey.cmp(&b.citekey));
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "quellen");
    }

    #[test]
    fn reads_example_bibliography() {
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
        let sources = load(root).unwrap();
        let keys: Vec<_> = sources.iter().map(|s| s.citekey.as_str()).collect();
        assert_eq!(keys, ["gebhard2021", "greenyer2013", "koenig2023", "omg2017", "sutton2018", "weber2022"]);
        let sutton = &sources[4];
        assert_eq!(sutton.kind, "book");
        assert_eq!(sutton.authors, ["Sutton, Richard S.", "Barto, Andrew G."]);
        assert_eq!(sutton.year.as_deref(), Some("2018"));
        assert_eq!(sources[1].container.as_deref(), Some("Requirements Engineering"));
        assert_eq!(sources[0].container.as_deref(), Some("Heise Developer (Online)"));
    }

    #[test]
    fn missing_file_means_no_sources_and_broken_file_is_an_error() {
        assert_eq!(load(&std::env::temp_dir().join("gibt-es-nicht")).unwrap(), Vec::new());
        assert!(parse("@book{kaputt, title = {").is_err());
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("sa-bib-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("System")).unwrap();
        root
    }

    const BIB: &str = "@book{mueller2024, author = {Müller, Eva}, title = {Ein Buch}, year = {2024}, publisher = {Verlag}}\n";

    #[test]
    fn adopt_bib_copies_into_empty_work_but_never_overwrites() {
        let root = temp_root("adopt");
        std::fs::write(root.join(BIB_FILE), "% Platzhalter\n").unwrap();
        let from = root.join("extern.bib");
        std::fs::write(&from, BIB).unwrap();
        assert_eq!(adopt_bib(&root, &from), Ok(1));
        assert_eq!(load(&root).unwrap()[0].citekey, "mueller2024");
        let err = adopt_bib(&root, &from).unwrap_err();
        assert!(err.contains("schon Quellen"), "{err}");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn adopt_bib_rejects_empty_and_missing_files() {
        let root = temp_root("reject");
        let empty = root.join("leer.bib");
        std::fs::write(&empty, "% nur Kommentar\n").unwrap();
        assert!(adopt_bib(&root, &empty).unwrap_err().contains("keine Quellen"));
        assert!(adopt_bib(&root, &root.join("gibt-es-nicht.bib")).is_err());
        assert!(!root.join(BIB_FILE).exists());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
