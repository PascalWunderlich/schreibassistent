//! Kapitel ↔ Citekey ↔ Wissensseite ↔ Notiz, SQLite-Cache.
//!
//! Der Index entsteht vollständig aus den Dateien des Arbeits-Vaults. Der Cache in
//! `.schreibassistent/cache.sqlite` speichert nur das Ergebnis je Datei (mit Prüfsumme)
//! und ist jederzeit löschbar (Leitplanke 6).

pub mod cache;
pub mod commands;
pub mod parse;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use parse::{MdFile, TypFile};

pub const NAME: &str = "index";

/// Ausgewertete Dateien, Schlüssel = Pfad relativ zum Vault.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Index {
    pub typ: BTreeMap<String, TypFile>,
    pub md: BTreeMap<String, MdFile>,
}

/// Was zu einem Kapitel gehört (Seitenleiste „Dieses Kapitel“).
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct ChapterView {
    pub file: String,
    pub title: Option<String>,
    /// Zitierte Quellen mit ihren Ergänzungen (z. B. „S. 14“), in Reihenfolge der Quelle.
    pub quellen: BTreeMap<String, Vec<String>>,
    /// Notizen mit `kapitel: [<Kapitel>]` im Frontmatter.
    pub notizen_direkt: Vec<String>,
    /// Weitere Notizen, die eine hier zitierte Quelle nennen, mit diesen Quellen.
    pub notizen_ueber: BTreeMap<String, Vec<String>>,
    pub wissen_direkt: Vec<String>,
    pub wissen_ueber: BTreeMap<String, Vec<String>>,
}

/// Eine Quelle mit den Kapiteln, die sie zitieren (Seitenleiste „Alle Quellen“).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SourceUse {
    pub citekey: String,
    pub chapters: Vec<String>,
}

/// Ordner, die der Index liest.
const TYP_ROOT: &str = "Arbeit";
const MD_ROOTS: [&str; 2] = ["Notizen", "Wissen"];
/// Kapitel im Sinne der Seitenleiste (docs/DATA-FORMATS.md: `kapitel/NN-name.typ`).
const CHAPTER_DIR: &str = "Arbeit/kapitel/";

impl Index {
    /// Index aufbauen; mit `cache` werden unveränderte Dateien nicht neu gelesen.
    pub fn build(root: &Path, cache: Option<&cache::Cache>) -> Result<Self, String> {
        let mut index = Index::default();
        let mut seen = Vec::new();
        for path in files(root, TYP_ROOT, "typ") {
            let text = std::fs::read_to_string(root.join(&path)).map_err(|e| format!("{path}: {e}"))?;
            let file = match cache.and_then(|c| c.get::<TypFile>(&path, &text)) {
                Some(file) => file,
                None => {
                    let file = parse::parse_typ(&text);
                    if let Some(c) = cache {
                        c.put(&path, &text, &file)?;
                    }
                    file
                }
            };
            seen.push(path.clone());
            index.typ.insert(path, file);
        }
        for dir in MD_ROOTS {
            for path in files(root, dir, "md") {
                let text = std::fs::read_to_string(root.join(&path)).map_err(|e| format!("{path}: {e}"))?;
                let file = match cache.and_then(|c| c.get::<MdFile>(&path, &text)) {
                    Some(file) => file,
                    None => {
                        let file = parse::parse_md(&text);
                        if let Some(c) = cache {
                            c.put(&path, &text, &file)?;
                        }
                        file
                    }
                };
                seen.push(path.clone());
                index.md.insert(path, file);
            }
        }
        if let Some(c) = cache {
            c.retain(&seen)?;
        }
        Ok(index)
    }

    /// Kapiteldateien in Reihenfolge des Dateinamens.
    pub fn chapters(&self) -> Vec<&str> {
        self.typ.keys().filter(|p| p.starts_with(CHAPTER_DIR)).map(String::as_str).collect()
    }

    /// Quellen, die eine Notiz/Wissensseite nennt: Frontmatter und `[[Links]]` auf Citekeys.
    fn md_citekeys(md: &MdFile, bib: &BTreeSet<String>) -> BTreeSet<String> {
        md.citekeys.iter().chain(md.links.iter().filter(|l| bib.contains(*l))).cloned().collect()
    }

    pub fn chapter_view(&self, file: &str, bib: &BTreeSet<String>) -> Option<ChapterView> {
        let typ = self.typ.get(file)?;
        let stem = Path::new(file).file_stem()?.to_string_lossy().into_owned();
        let mut view = ChapterView {
            file: file.to_owned(),
            title: typ.headings.iter().find(|h| h.level == 1).map(|h| h.text.clone()),
            ..Default::default()
        };
        for reference in typ.references.iter().filter(|r| bib.contains(&r.key)) {
            let pages = view.quellen.entry(reference.key.clone()).or_default();
            if let Some(s) = &reference.supplement
                && !pages.contains(s)
            {
                pages.push(s.clone());
            }
        }
        let cited: BTreeSet<String> = view.quellen.keys().cloned().collect();
        for (path, md) in &self.md {
            let (direct, via) = if path.starts_with("Notizen/") {
                (&mut view.notizen_direkt, &mut view.notizen_ueber)
            } else {
                (&mut view.wissen_direkt, &mut view.wissen_ueber)
            };
            if md.kapitel.iter().any(|k| *k == stem) {
                direct.push(path.clone());
                continue;
            }
            let shared: Vec<String> = Self::md_citekeys(md, bib).intersection(&cited).cloned().collect();
            if !shared.is_empty() {
                via.insert(path.clone(), shared);
            }
        }
        Some(view)
    }

    /// Alle Quellen der `.bib` mit den Kapiteln, die sie zitieren (leer = ungenutzt).
    pub fn source_uses(&self, bib: &BTreeSet<String>) -> Vec<SourceUse> {
        bib.iter()
            .map(|key| SourceUse {
                citekey: key.clone(),
                chapters: self
                    .chapters()
                    .into_iter()
                    .filter(|c| self.typ[*c].references.iter().any(|r| &r.key == key))
                    .map(str::to_owned)
                    .collect(),
            })
            .collect()
    }
}

/// Dateien mit Endung `ext` unter `root/dir`, relativ zu `root`, sortiert; ohne versteckte.
fn files(root: &Path, dir: &str, ext: &str) -> Vec<String> {
    let mut out = Vec::new();
    collect(root, &root.join(dir), ext, &mut out);
    out.sort();
    out
}

fn collect(root: &Path, dir: &Path, ext: &str, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext)
            && let Ok(relative) = path.strip_prefix(root)
        {
            out.push(crate::projekt::files::rel_string(relative));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "index");
    }

    fn example() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"))
    }

    fn bib(root: &Path) -> BTreeSet<String> {
        crate::quellen::load(root).unwrap().into_iter().map(|s| s.citekey).collect()
    }

    /// Abnahme P4: Zuordnungen in beispiel-arbeit stimmen zu 100 % mit tests/index/erwartet.json.
    #[test]
    fn example_assignments_match_expected_exactly() {
        let root = example();
        let expected: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("../tests/index/erwartet.json")).unwrap(),
        )
        .unwrap();
        let bib = bib(root);
        let index = Index::build(root, None).unwrap();

        let chapters = expected["kapitel"].as_object().unwrap();
        assert_eq!(index.chapters(), chapters.keys().map(String::as_str).collect::<Vec<_>>());
        for (file, want) in chapters {
            let view = index.chapter_view(file, &bib).unwrap();
            let got = json!({
                "titel": view.title,
                "quellen": view.quellen,
                "notizen_direkt": view.notizen_direkt,
                "notizen_ueber": view.notizen_ueber,
                "wissen_direkt": view.wissen_direkt,
                "wissen_ueber": view.wissen_ueber,
            });
            assert_eq!(&got, want, "Zuordnung für {file}");
        }
        let uses: BTreeMap<String, Vec<String>> =
            index.source_uses(&bib).into_iter().map(|u| (u.citekey, u.chapters)).collect();
        assert_eq!(json!(uses), expected["quellen"]);
    }

    #[test]
    fn labels_are_not_sources() {
        let root = example();
        let view = Index::build(root, None).unwrap().chapter_view("Arbeit/kapitel/03-umsetzung.typ", &bib(root)).unwrap();
        assert!(!view.quellen.contains_key("kap:grundlagen") && !view.quellen.contains_key("tab:anforderungen"));
    }

    /// Abnahme P4: Index nach Löschen des Caches identisch; zweiter Aufbau aus dem Cache gleich.
    #[test]
    fn cache_is_transparent_and_rebuildable() {
        let root = example();
        let dir = std::env::temp_dir().join(format!("sa-index-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let without = Index::build(root, None).unwrap();

        let cache = cache::Cache::open(&dir.join("cache.sqlite")).unwrap();
        let first = Index::build(root, Some(&cache)).unwrap();
        assert_eq!(cache.len().unwrap(), first.typ.len() + first.md.len());
        let from_cache = Index::build(root, Some(&cache)).unwrap();
        drop(cache);
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
        let cache = cache::Cache::open(&dir.join("cache.sqlite")).unwrap();
        let rebuilt = Index::build(root, Some(&cache)).unwrap();

        assert_eq!(first, without);
        assert_eq!(from_cache, without);
        assert_eq!(rebuilt, without);
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }
}
