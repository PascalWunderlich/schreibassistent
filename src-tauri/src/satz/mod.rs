//! Typst-World, Kompilieren, SVG je Seite, PDF, Positions-Mapping.
//!
//! P2: World und Seiten-SVG mit Erkennung geänderter Seiten; Mapping folgt.

mod world;

use std::time::Instant;

use typst_layout::PagedDocument;

pub use world::SatzWorld;

pub const NAME: &str = "satz";

/// Ergebnis einer Kompilierung: welche Seiten sich geändert haben und wie lange es dauerte.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompileReport {
    pub page_count: usize,
    /// Indizes (ab 0) der Seiten, deren Inhalt sich seit der letzten Kompilierung geändert hat.
    pub changed_pages: Vec<usize>,
    /// SVG der geänderten Seiten, gleiche Reihenfolge wie `changed_pages`.
    pub svgs: Vec<String>,
    pub compile_ms: f64,
    pub svg_ms: f64,
    pub warnings: Vec<String>,
}

/// Hält World und letztes Dokument eines Arbeits-Vaults.
pub struct Satz {
    world: SatzWorld,
    page_hashes: Vec<u128>,
    document: Option<PagedDocument>,
}

impl Satz {
    pub fn open(root: impl Into<std::path::PathBuf>, main: &str) -> Result<Self, String> {
        Ok(Self { world: SatzWorld::new(root, main)?, page_hashes: Vec::new(), document: None })
    }

    pub fn world(&self) -> &SatzWorld {
        &self.world
    }

    pub fn document(&self) -> Option<&PagedDocument> {
        self.document.as_ref()
    }

    /// Kompiliert und erzeugt SVG nur für Seiten, deren Frame sich geändert hat.
    pub fn compile(&mut self) -> Result<CompileReport, Vec<String>> {
        self.world.reset_time();
        let start = Instant::now();
        let warned = typst::compile::<PagedDocument>(&self.world);
        let compile_ms = start.elapsed().as_secs_f64() * 1000.0;
        // Speicher begrenzen; ältere Zwischenergebnisse verwerfen (wie typst-cli).
        comemo::evict(10);

        let warnings = warned.warnings.iter().map(|w| w.message.to_string()).collect();
        let document = warned
            .output
            .map_err(|errors| errors.iter().map(|e| e.message.to_string()).collect::<Vec<_>>())?;

        let start = Instant::now();
        let hashes: Vec<u128> =
            document.pages().iter().map(|page| typst_utils::hash128(&page.frame)).collect();
        let changed_pages: Vec<usize> = (0..hashes.len())
            .filter(|&i| self.page_hashes.get(i) != Some(&hashes[i]))
            .collect();
        let options = typst_svg::SvgOptions::default();
        let svgs = changed_pages
            .iter()
            .map(|&i| typst_svg::svg(&document.pages()[i], &options))
            .collect();
        let svg_ms = start.elapsed().as_secs_f64() * 1000.0;

        self.page_hashes = hashes;
        let page_count = document.pages().len();
        self.document = Some(document);
        Ok(CompileReport { page_count, changed_pages, svgs, compile_ms, svg_ms, warnings })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "satz");
    }

    fn beispiel() -> Satz {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit");
        Satz::open(root, "Arbeit/main.typ").expect("World")
    }

    #[test]
    fn compiles_example_and_renders_all_pages_first() {
        let mut satz = beispiel();
        let report = satz.compile().expect("Beispielarbeit kompiliert");
        assert!(report.page_count > 5);
        assert_eq!(report.changed_pages.len(), report.page_count);
        assert!(report.svgs[0].starts_with("<svg"));
    }

    #[test]
    fn unchanged_recompile_renders_nothing() {
        let mut satz = beispiel();
        satz.compile().unwrap();
        let report = satz.compile().unwrap();
        assert!(report.changed_pages.is_empty());
    }

    #[test]
    fn edit_rerenders_only_affected_pages() {
        let mut satz = beispiel();
        let first = satz.compile().unwrap();
        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let text = satz.world().source_text(path).unwrap();
        let edited = text.replacen("Die Modellierung erfolgt", "Die Modellierung erfolgt hier", 1);
        assert_ne!(text, edited);
        satz.world().set_source_text(path, &edited).unwrap();
        let report = satz.compile().unwrap();
        assert!(!report.changed_pages.is_empty());
        assert!(report.changed_pages.len() < first.page_count);
    }

    #[test]
    fn missing_package_fails_without_network() {
        let dir = std::env::temp_dir().join("schreibassistent-satz-offline");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("main.typ"), "#import \"@preview/gibt-es-nicht:9.9.9\": *\n").unwrap();
        let mut satz = Satz::open(&dir, "main.typ").unwrap();
        let errors = satz.compile().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("gibt-es-nicht")), "{errors:?}");
    }
}
