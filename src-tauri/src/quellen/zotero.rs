//! Zotero Local API (http://localhost:23119/api/, ab Zotero 7): Zotero-Schlüssel, PDF-Anhänge
//! und Markierungen je Citekey. Nur lokal, nur lesend (Leitplanke 4). Läuft Zotero nicht oder
//! ist die Local API aus, gibt es still „nichts“ – die App nutzt dann nur die `.bib`.
//!
//! Voraussetzung in Zotero: Einstellungen → Erweitert → „Anderen Anwendungen auf diesem
//! Computer erlauben, mit Zotero zu kommunizieren“. Ergebnisse: docs/zotero-local-api.md.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

const API: &str = "http://localhost:23119/api/users/0";
/// Kurz halten: Ohne laufendes Zotero soll die Seitenleiste nicht warten.
const TIMEOUT: Duration = Duration::from_millis(800);
const PAGE: usize = 100;

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct ZoteroSource {
    /// Schlüssel des Eintrags in Zotero.
    pub item_key: String,
    pub pdfs: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Attachment {
    /// Schlüssel des Anhangs (für `zotero://open-pdf/library/items/<key>?page=<n>`).
    pub key: String,
    pub title: String,
    /// Dateiname in `~/Zotero/storage/<key>/` (nur bei gespeicherten Dateien, nicht bei Links).
    pub filename: Option<String>,
    pub annotations: Vec<Annotation>,
}

impl Attachment {
    /// Pfad der PDF-Datei im Zotero-Datenordner (Standardort `~/Zotero`), falls vorhanden.
    pub fn path(&self) -> Option<std::path::PathBuf> {
        let home = std::env::var_os("HOME")?;
        let path = std::path::Path::new(&home).join("Zotero/storage").join(&self.key).join(self.filename.as_deref()?);
        path.is_file().then_some(path)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Annotation {
    /// Gedruckte Seitenzahl, wie Zotero sie anzeigt (`annotationPageLabel`).
    pub page_label: Option<String>,
    /// PDF-Seite ab 1 (aus `annotationPosition.pageIndex`), für das Öffnen an der Stelle.
    pub page: Option<u32>,
    pub kind: String,
    pub text: Option<String>,
    pub comment: Option<String>,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(TIMEOUT)).build().into()
}

/// Alle Einträge einer Abfrage, seitenweise.
fn all(agent: &ureq::Agent, query: &str) -> Option<Vec<Value>> {
    let mut out = Vec::new();
    loop {
        let url = format!("{API}/{query}&format=json&limit={PAGE}&start={}", out.len());
        let page: Vec<Value> = agent.get(&url).call().ok()?.body_mut().read_json().ok()?;
        let done = page.len() < PAGE;
        out.extend(page);
        if done {
            return Some(out);
        }
    }
}

/// Läuft Zotero mit eingeschalteter Local API?
pub fn available() -> bool {
    agent().get(format!("{API}/items?limit=1")).call().is_ok()
}

/// Zotero-Daten je Citekey; `None`, wenn Zotero nicht erreichbar ist.
pub fn fetch() -> Option<HashMap<String, ZoteroSource>> {
    let agent = agent();
    let items = all(&agent, "items/top?")?;
    let attachments = all(&agent, "items?itemType=attachment")?;
    let annotations = all(&agent, "items?itemType=annotation")?;
    Some(assemble(&items, &attachments, &annotations))
}

fn text(value: &Value, field: &str) -> Option<String> {
    value["data"][field].as_str().map(str::to_owned).filter(|s| !s.is_empty())
}

/// Einträge, Anhänge und Markierungen über `parentItem` zusammensetzen.
pub fn assemble(items: &[Value], attachments: &[Value], annotations: &[Value]) -> HashMap<String, ZoteroSource> {
    let mut by_parent: HashMap<String, Vec<Annotation>> = HashMap::new();
    for annotation in annotations {
        let Some(parent) = text(annotation, "parentItem") else { continue };
        // annotationPosition ist ein JSON-Text: {"pageIndex": 3, "rects": [...]}.
        let page = text(annotation, "annotationPosition")
            .and_then(|p| serde_json::from_str::<Value>(&p).ok())
            .and_then(|p| p["pageIndex"].as_u64())
            .map(|i| i as u32 + 1);
        by_parent.entry(parent).or_default().push(Annotation {
            page_label: text(annotation, "annotationPageLabel"),
            page,
            kind: text(annotation, "annotationType").unwrap_or_default(),
            text: text(annotation, "annotationText"),
            comment: text(annotation, "annotationComment"),
        });
    }
    for list in by_parent.values_mut() {
        list.sort_by_key(|a| a.page);
    }

    let mut pdfs_by_item: HashMap<String, Vec<Attachment>> = HashMap::new();
    for attachment in attachments {
        if text(attachment, "contentType").as_deref() != Some("application/pdf") {
            continue;
        }
        let (Some(parent), Some(key)) = (text(attachment, "parentItem"), text(attachment, "key")) else { continue };
        pdfs_by_item.entry(parent).or_default().push(Attachment {
            title: text(attachment, "title").unwrap_or_else(|| "PDF".to_owned()),
            filename: text(attachment, "filename"),
            annotations: by_parent.remove(&key).unwrap_or_default(),
            key,
        });
    }

    items
        .iter()
        .filter_map(|item| {
            let citekey = text(item, "citationKey")?;
            let item_key = text(item, "key")?;
            Some((citekey, ZoteroSource { pdfs: pdfs_by_item.remove(&item_key).unwrap_or_default(), item_key }))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn assembles_items_pdfs_and_annotations_by_parent() {
        let items = [json!({"data": {"key": "ITEM1", "citationKey": "sutton2018"}}), json!({"data": {"key": "ITEM2"}})];
        let attachments = [
            json!({"data": {"key": "PDF1", "parentItem": "ITEM1", "contentType": "application/pdf", "title": "Volltext"}}),
            json!({"data": {"key": "WEB1", "parentItem": "ITEM1", "contentType": "text/html", "title": "Snapshot"}}),
        ];
        let annotations = [
            json!({"data": {"parentItem": "PDF1", "annotationType": "highlight", "annotationText": "später",
                "annotationPageLabel": "14", "annotationPosition": "{\"pageIndex\": 20, \"rects\": []}"}}),
            json!({"data": {"parentItem": "PDF1", "annotationType": "note", "annotationComment": "Idee",
                "annotationPageLabel": "3", "annotationPosition": "{\"pageIndex\": 9}"}}),
        ];
        let result = assemble(&items, &attachments, &annotations);
        assert_eq!(result.len(), 1, "nur Einträge mit Citekey");
        let source = &result["sutton2018"];
        assert_eq!(source.item_key, "ITEM1");
        assert_eq!(source.pdfs.len(), 1, "nur PDFs");
        let notes = &source.pdfs[0].annotations;
        assert_eq!(notes.iter().map(|a| (a.page, a.page_label.as_deref())).collect::<Vec<_>>(), [(Some(10), Some("3")), (Some(21), Some("14"))]);
        assert_eq!(notes[1].text.as_deref(), Some("später"));
    }

    /// Ohne laufendes Zotero: still `None`, ohne lange zu warten.
    #[test]
    fn unavailable_zotero_is_silent() {
        if available() {
            eprintln!("Zotero läuft – Test des Ausfalls übersprungen");
            return;
        }
        let start = std::time::Instant::now();
        assert!(fetch().is_none());
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
