//! Typst-World, Kompilieren, SVG je Seite, PDF, Positions-Mapping.
//!
//! P2: World, Seiten-SVG mit Erkennung geänderter Seiten, Klick → Absatz.

pub mod commands;
pub mod mapping;
mod world;

use std::time::Instant;

use typst_layout::PagedDocument;

pub use world::{SatzWorld, project_file};

use mapping::Rect;

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
    /// Seitengrößen in pt (Breite, Höhe), alle Seiten.
    pub page_sizes: Vec<(f64, f64)>,
    /// Gedruckte Seitenzahl je Seite („“, „i“, „1“ …), alle Seiten.
    pub page_labels: Vec<String>,
}

/// Absatz unter einem Klickpunkt: Quelltext und Lage auf den Seiten.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParagraphHit {
    /// Pfad relativ zum Arbeits-Vault, z. B. `Arbeit/kapitel/01-einleitung.typ`.
    pub file: String,
    /// Byte-Bereich im Quelltext.
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// Je Seite (Index ab 0) das umschließende Rechteck in pt.
    pub rects: Vec<(usize, Rect)>,
    /// Schriftgröße und Zeilenabstand (Grundlinie zu Grundlinie) im Satz, in pt.
    pub font_size: f64,
    pub line_height: f64,
    /// Textspalte der Arbeit: linke Kante und Breite in pt.
    pub column_x: f64,
    pub column_width: f64,
    /// Einzug der ersten Zeile in pt.
    pub first_line_indent: f64,
}

/// Hält World und letztes Dokument eines Arbeits-Vaults.
pub struct Satz {
    world: SatzWorld,
    page_hashes: Vec<u128>,
    document: Option<PagedDocument>,
    /// Textspalte (linke Kante, Breite) in pt; einmal ermittelt, die Ränder ändern sich nicht.
    column: Option<(f64, f64)>,
}

impl Satz {
    pub fn open(root: impl Into<std::path::PathBuf>, main: &str) -> Result<Self, String> {
        Ok(Self { world: SatzWorld::new(root, main)?, page_hashes: Vec::new(), document: None, column: None })
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
        let page_sizes = document.pages().iter().map(|p| mapping::page_size(&p.frame)).collect();
        let page_labels = document.pages().iter().map(page_label).collect();
        if self.column.is_none() {
            let boxes: Vec<_> = document
                .pages()
                .iter()
                .enumerate()
                .map(|(i, p)| (i, mapping::text_boxes(&self.world, &p.frame)))
                .collect();
            self.column = mapping::column(&boxes);
        }
        self.document = Some(document);
        Ok(CompileReport { page_count, changed_pages, svgs, compile_ms, svg_ms, warnings, page_sizes, page_labels })
    }

    /// Absatz an (x, y) in pt auf Seite `page` (ab 0). `None`, wenn dort kein Text
    /// aus einer Projektdatei steht (z. B. Kopfzeile aus der Vorlage).
    pub fn paragraph_at(&self, page: usize, x: f64, y: f64) -> Option<ParagraphHit> {
        let document = self.document.as_ref()?;
        let pages = document.pages();
        let boxes = mapping::text_boxes(&self.world, &pages.get(page)?.frame);
        let hit = mapping::box_at(&boxes, x, y, 6.0)?;
        let source = typst::World::source(&self.world, hit.file).ok()?;
        let range = mapping::paragraph_range(&source, hit.range.start)?;

        // Ein Absatz kann über einen Seitenumbruch laufen: Nachbarseiten mit ansehen.
        let neighbours = page.saturating_sub(1)..(page + 2).min(pages.len());
        let page_boxes: Vec<_> = neighbours
            .map(|p| (p, if p == page { boxes.clone() } else { mapping::text_boxes(&self.world, &pages[p].frame) }))
            .collect();
        let rects = mapping::rects_for_range(&page_boxes, hit.file, &range);
        let (font_size, line_height) = mapping::typography(&page_boxes, hit.file, &range);
        let (column_x, column_width) = self.column.unwrap_or((rects[0].1.x, rects[0].1.width));
        let first_line_indent = mapping::first_line_indent(&page_boxes, hit.file, &range, column_x);

        Some(ParagraphHit {
            file: hit.file.vpath().get_without_slash().to_string(),
            text: source.text()[range.clone()].to_owned(),
            start: range.start,
            end: range.end,
            rects,
            font_size,
            line_height,
            column_x,
            column_width,
            first_line_indent,
        })
    }

    /// Absatz, der bei Byte `offset` in `file` beginnt oder ihn enthält, mit seiner Lage
    /// auf den Seiten. Liegt `offset` zwischen zwei Absätzen (nach Enter), ist der Absatz
    /// leer: `start == end == offset`, das Rechteck liegt unter dem Text davor.
    pub fn paragraph_at_offset(&self, file: &str, offset: usize) -> Option<ParagraphHit> {
        let id = project_file(file).ok()?;
        let source = typst::World::source(&self.world, id).ok()?;
        let text = source.text();
        let rest = text.get(offset..)?;
        // Leer, wenn danach nur Zeilenende, Dateiende oder das Ende eines Inhaltsblocks `]` folgt.
        let next = rest.trim_start_matches([' ', '\t']);
        let blank = next.is_empty() || next.starts_with('\n') || next.starts_with(']');
        let range = if blank { offset..offset } else { mapping::paragraph_range(&source, offset)? };

        let pages = self.document.as_ref()?.pages();
        let page_boxes: Vec<_> =
            pages.iter().enumerate().map(|(i, p)| (i, mapping::text_boxes(&self.world, &p.frame))).collect();
        let rects = if range.is_empty() {
            mapping::rect_after(&page_boxes, id, offset).into_iter().collect()
        } else {
            mapping::rects_for_range(&page_boxes, id, &range)
        };
        if rects.is_empty() {
            return None;
        }
        // Leerer Absatz: Typografie vom Text davor übernehmen.
        let probe = if range.is_empty() { offset.saturating_sub(1)..offset } else { range.clone() };
        let (font_size, line_height) = mapping::typography(&page_boxes, id, &probe);
        let (column_x, column_width) = self.column.unwrap_or((rects[0].1.x, rects[0].1.width));
        // Neuer Absatz: im Fließtext eingerückt wie die Vorlage (1,5 em), bestehender wie gesetzt.
        let first_line_indent = if range.is_empty() {
            1.5 * font_size
        } else {
            mapping::first_line_indent(&page_boxes, id, &range, column_x)
        };
        Some(ParagraphHit {
            file: file.to_owned(),
            text: text[range.clone()].to_owned(),
            start: range.start,
            end: range.end,
            rects,
            font_size,
            line_height,
            column_x,
            column_width,
            first_line_indent,
        })
    }

    /// Schriftdatei, mit der Typst eine Familie setzt (für das Editierfeld in der Oberfläche).
    pub fn font_data(&self, family: &str, bold: bool) -> Option<Vec<u8>> {
        use typst::text::{FontVariant, FontWeight};
        let mut variant = FontVariant::default();
        if bold {
            variant.weight = FontWeight::BOLD;
        }
        let index = typst::World::book(&self.world).select(&family.to_lowercase(), variant)?;
        typst::World::font(&self.world, index).map(|f| f.data().to_vec())
    }

    /// Ersetzt `start..end` in `file` durch `text` (nur im Speicher) und kompiliert neu.
    pub fn replace(&mut self, file: &str, start: usize, end: usize, text: &str) -> Result<CompileReport, Vec<String>> {
        let mut source = self.world.source_text(file).map_err(|e| vec![e])?;
        if start > end || end > source.len() || !source.is_char_boundary(start) || !source.is_char_boundary(end) {
            return Err(vec![format!("Ungültiger Bereich {start}..{end} in {file}")]);
        }
        source.replace_range(start..end, text);
        self.world.set_source_text(file, &source).map_err(|e| vec![e])?;
        self.compile()
    }
}

fn page_label(page: &typst_layout::Page) -> String {
    match &page.numbering {
        Some(typst::model::Numbering::Pattern(pattern)) => {
            pattern.apply(None, &[page.number]).map(|s| s.to_string()).unwrap_or_default()
        }
        _ => String::new(),
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
        assert_eq!(&report.page_labels[..6], ["", "i", "ii", "iii", "iv", "1"]);
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
    fn click_on_paragraph_finds_its_source_and_replace_recompiles() {
        let mut satz = beispiel();
        let report = satz.compile().unwrap();
        // Seite mit „Kapitel 3“ suchen: dort steht der Absatz mit „Die Modellierung erfolgt“.
        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let text = satz.world().source_text(path).unwrap();
        let offset = text.find("Die Modellierung erfolgt").unwrap();
        let file = project_file(path).unwrap();
        let (page, rect) = (0..report.page_count)
            .find_map(|p| {
                let frame = &satz.document().unwrap().pages()[p].frame;
                mapping::text_boxes(satz.world(), frame)
                    .into_iter()
                    .find(|b| b.file == file && b.range.contains(&offset))
                    .map(|b| (p, b.rect))
            })
            .expect("Absatz ist gesetzt");

        let hit = satz.paragraph_at(page, rect.x + 1.0, rect.y + rect.height / 2.0).expect("Treffer");
        assert_eq!(hit.file, path);
        assert!(hit.text.starts_with("#lorem(60) Die Modellierung erfolgt"), "{}", hit.text);
        assert!(hit.rects.iter().any(|(p, _)| *p == page));
        // Fließtext: 11 pt; Textspalte links 3 cm, rechts 4 cm (FHDW-Ränder der Vorlage).
        assert!((hit.font_size - 11.0).abs() < 0.01, "{}", hit.font_size);
        assert!(hit.line_height > 12.0 && hit.line_height < 18.0, "{}", hit.line_height);
        assert!((hit.column_x - 85.04).abs() < 1.0, "{}", hit.column_x);
        assert!((hit.column_width - (595.28 - 85.04 - 113.39)).abs() < 2.0, "{}", hit.column_width);
        assert!(satz.font_data("New Computer Modern", false).is_some_and(|d| d.len() > 10_000));
        // Direkt nach einer Überschrift: kein Einzug (FHDW-Hinweise S. 5, Vorlage).
        assert_eq!(hit.first_line_indent, 0.0);
        // Absatz nach einem Absatz: Einzug 1,5 em = 16,5 pt.
        let intro = "Arbeit/kapitel/01-einleitung.typ";
        let intro_text = satz.world().source_text(intro).unwrap();
        let second = intro_text.find("ausführlich.\n\n#lorem(50)").unwrap() + "ausführlich.\n\n".len();
        let indented = satz.paragraph_at_offset(intro, second).unwrap();
        assert_eq!(indented.text, "#lorem(50)");
        assert!((indented.first_line_indent - 16.5).abs() < 0.5, "{}", indented.first_line_indent);

        let new_text = hit.text.replace("Die Modellierung", "Die neue Modellierung");
        let report = satz.replace(&hit.file, hit.start, hit.end, &new_text).unwrap();
        assert!(!report.changed_pages.is_empty());
        assert!(satz.world().source_text(path).unwrap().contains("Die neue Modellierung"));
    }

    #[test]
    fn click_on_template_header_is_not_editable() {
        let mut satz = beispiel();
        let report = satz.compile().unwrap();
        // Kopfzeile auf der zweiten Inhaltsseite (Seitenindex 6) stammt aus dem Paket.
        assert!(report.page_count > 7);
        assert!(satz.paragraph_at(6, 100.0, 95.0).is_none());
    }

    #[test]
    fn long_paragraph_spans_two_pages() {
        let mut satz = beispiel();
        let report = satz.compile().unwrap();
        let path = "Arbeit/kapitel/04-loesungskonzept.typ";
        let text = satz.world().source_text(path).unwrap();
        let offset = text.find("#lorem(420)").unwrap();
        let file = project_file(path).unwrap();
        let (page, rect) = (0..report.page_count)
            .find_map(|p| {
                let frame = &satz.document().unwrap().pages()[p].frame;
                mapping::text_boxes(satz.world(), frame)
                    .into_iter()
                    .find(|b| b.file == file && b.range.contains(&(offset + 1)))
                    .map(|b| (p, b.rect))
            })
            .expect("langer Absatz ist gesetzt");
        let hit = satz.paragraph_at(page, rect.x + 1.0, rect.y + rect.height / 2.0).unwrap();
        assert_eq!(hit.text, "#lorem(420)");
        assert_eq!(hit.rects.len(), 2, "{:?}", hit.rects);
    }

    #[test]
    fn enter_creates_empty_paragraph_below_and_splits_in_the_middle() {
        let mut satz = beispiel();
        satz.compile().unwrap();
        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let text = satz.world().source_text(path).unwrap();
        let start = text.find("#lorem(60) Die Modellierung").unwrap();
        let end = start + text[start..].find('\n').unwrap();
        let paragraph = text[start..end].to_owned();

        // Enter am Ende: neuer leerer Absatz darunter.
        let new_text = format!("{paragraph}\n\n");
        satz.replace(path, start, end, &new_text).unwrap();
        let offset = start + new_text.len();
        let hit = satz.paragraph_at_offset(path, offset).expect("leerer Absatz");
        assert_eq!((hit.start, hit.end, hit.text.as_str()), (offset, offset, ""));
        assert_eq!(hit.rects.len(), 1);
        let report = satz.replace(path, offset, offset, "Ganz neuer Absatz.").unwrap();
        assert!(!report.changed_pages.is_empty());
        let after = satz.world().source_text(path).unwrap();
        assert!(after.contains("@omg2017.\n\nGanz neuer Absatz.\n"), "{}", &after[start..start + 200]);

        // Enter in der Mitte: zweiter Teil wird eigener Absatz.
        let split = after.find("Ganz neuer").unwrap() + "Ganz ".len();
        satz.replace(path, split, split, "\n\n").unwrap();
        let hit = satz.paragraph_at_offset(path, split + 2).expect("zweiter Teil");
        assert_eq!(hit.text, "neuer Absatz.");
    }

    #[test]
    fn enter_at_end_of_summary_stays_inside_content_block() {
        // Fehler aus dem Test am 2026-10-17: Enter am Ende von `summary: [#lorem(120)]`
        // wählte den ganzen `#show:`-Aufruf als neuen Absatz.
        let mut satz = beispiel();
        satz.compile().unwrap();
        let path = "Arbeit/main.typ";
        let text = satz.world().source_text(path).unwrap();
        let start = text.find("#lorem(120)").unwrap();
        let end = start + "#lorem(120)".len();
        satz.replace(path, start, end, "#lorem(120)\n\n").unwrap();
        let offset = end + 2;
        let hit = satz.paragraph_at_offset(path, offset).expect("leerer Absatz in der Zusammenfassung");
        assert_eq!((hit.start, hit.end), (offset, offset));
        satz.replace(path, offset, offset, "Zweiter Absatz der Zusammenfassung.").unwrap();
        let after = satz.world().source_text(path).unwrap();
        assert!(after.contains("summary: [#lorem(120)\n\nZweiter Absatz der Zusammenfassung.]"), "{after}");
    }

    #[test]
    fn clicks_anywhere_never_panic() {
        let mut satz = beispiel();
        let report = satz.compile().unwrap();
        let mut hits = 0;
        for page in 0..report.page_count {
            let (w, h) = report.page_sizes[page];
            for ix in 0..=20 {
                for iy in 0..=40 {
                    let (x, y) = (w * ix as f64 / 20.0, h * iy as f64 / 40.0);
                    if satz.paragraph_at(page, x, y).is_some() {
                        hits += 1;
                    }
                }
            }
        }
        assert!(hits > 100, "nur {hits} Treffer");
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
