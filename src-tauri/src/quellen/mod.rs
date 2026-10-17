//! quellen.bib lesen, Zotero Local API (localhost:23119).
//!
//! Bibliografische Daten stehen nur in Zotero; die App liest den Better-BibTeX-Export
//! `System/quellen.bib` (docs/DATA-FORMATS.md, Wissensmodell im Vault).

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
            Source {
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
}
