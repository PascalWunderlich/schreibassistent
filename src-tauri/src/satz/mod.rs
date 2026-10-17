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
    /// Seiten des Inhaltsteils: von Seite „1“ bis vor das Quellverzeichnis
    /// (FHDW-Hinweise S. 3). `None`, wenn es kein Quellverzeichnis gibt.
    pub content_pages: Option<usize>,
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

    /// Beim nächsten Kompilieren alle Seiten als geändert melden (z. B. neue Ansicht).
    pub fn forget_pages(&mut self) {
        self.page_hashes.clear();
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
        let page_labels: Vec<String> = document.pages().iter().map(page_label).collect();
        if self.column.is_none() {
            let boxes: Vec<_> = document
                .pages()
                .iter()
                .enumerate()
                .map(|(i, p)| (i, mapping::text_boxes(&self.world, &p.frame)))
                .collect();
            self.column = mapping::column(&boxes);
        }
        let content_pages = content_pages(&document, &page_labels);
        self.document = Some(document);
        Ok(CompileReport {
            page_count,
            changed_pages,
            svgs,
            compile_ms,
            svg_ms,
            warnings,
            page_sizes,
            page_labels,
            content_pages,
        })
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

    /// Erste Seite (Index ab 0), auf der Text aus `file` steht, z. B. für „zu Kapitel springen“.
    pub fn first_page_of(&self, file: &str) -> Option<usize> {
        let id = project_file(file).ok()?;
        let pages = self.document.as_ref()?.pages();
        // Nur Seiten mit arabischer Zählung: Überschriften stehen auch im Inhaltsverzeichnis.
        (0..pages.len())
            .filter(|&i| page_label(&pages[i]).starts_with(|c: char| c.is_ascii_digit()))
            .find(|&i| mapping::text_boxes(&self.world, &pages[i].frame).iter().any(|b| b.file == id))
    }

    /// PDF des zuletzt gesetzten Dokuments, wie `typst compile` (gleiche Typst-Version).
    /// `timestamp`: Erstellungszeit (UNIX); ohne Angabe die aktuelle Zeit.
    pub fn export_pdf(&self, timestamp: Option<i64>) -> Result<Vec<u8>, Vec<String>> {
        let document = self.document.as_ref().ok_or_else(|| vec!["Noch nicht gesetzt".to_owned()])?;
        let seconds = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
        });
        let options = typst_pdf::PdfOptions {
            timestamp: datetime_from_unix(seconds).map(typst_pdf::Timestamp::new_utc),
            ..Default::default()
        };
        typst_pdf::pdf(document, &options).map_err(|errors| errors.iter().map(|e| e.message.to_string()).collect())
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
        let previous = source.clone();
        source.replace_range(start..end, text);
        self.world.set_source_text(file, &source).map_err(|e| vec![e])?;
        let result = self.compile();
        if result.is_err() {
            // Fehlerhafte Eingabe nicht im Quelltext lassen: Die Oberfläche hält den Text
            // offen und schickt ihn korrigiert mit demselben Bereich erneut.
            self.world.set_source_text(file, &previous).map_err(|e| vec![e])?;
        }
        result
    }
}

/// Seiten von der ersten mit Zahl „1“ bis vor die Seite, auf der das Quellverzeichnis beginnt.
fn content_pages(document: &PagedDocument, labels: &[String]) -> Option<usize> {
    use typst::foundations::{NativeElement, Selector};
    use typst::introspection::Introspector;
    use typst::model::BibliographyElem;
    let first = labels.iter().position(|l| l == "1")?;
    let introspector = document.introspector();
    let bibliography = introspector.query(&Selector::Elem(BibliographyElem::ELEM, None));
    let page = introspector.page(bibliography.first()?.location()?)?.get() - 1;
    Some(page.saturating_sub(first))
}

/// UNIX-Zeitstempel → Typst-Datum (UTC), wie `typst compile --creation-timestamp`.
fn datetime_from_unix(seconds: i64) -> Option<typst::foundations::Datetime> {
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    // Tage seit 1970 → Kalenderdatum (Algorithmus von H. Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    typst::foundations::Datetime::from_ymd_hms(
        year as i32,
        month as u8,
        day as u8,
        (rest / 3600) as u8,
        (rest % 3600 / 60) as u8,
        (rest % 60) as u8,
    )
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

    /// Absatz, dessen Quelltext mit `needle` beginnt, über einen Klick auf seinen Text öffnen.
    fn click_on(satz: &Satz, path: &str, needle: &str) -> ParagraphHit {
        let text = satz.world().source_text(path).unwrap();
        let offset = text.find(needle).unwrap_or_else(|| panic!("{needle} fehlt"));
        let file = project_file(path).unwrap();
        let pages = satz.document().unwrap().pages();
        let (page, rect) = (0..pages.len())
            .find_map(|p| {
                mapping::text_boxes(satz.world(), &pages[p].frame)
                    .into_iter()
                    .find(|b| b.file == file && b.range.contains(&offset))
                    .map(|b| (p, b.rect))
            })
            .unwrap_or_else(|| panic!("{needle} ist nicht gesetzt"));
        satz.paragraph_at(page, rect.x + 1.0, rect.y + rect.height / 2.0).expect("Treffer")
    }

    #[test]
    fn syntax_error_is_reported_and_rolled_back() {
        let mut satz = beispiel();
        satz.compile().unwrap();
        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let before = satz.world().source_text(path).unwrap();
        let hit = click_on(&satz, path, "Die Modellierung erfolgt");
        let errors = satz.replace(path, hit.start, hit.end, "#lorem(").unwrap_err();
        assert!(!errors.is_empty());
        assert_eq!(satz.world().source_text(path).unwrap(), before, "Quelltext zurückgerollt");
        // Korrigiert mit demselben Bereich erneut senden.
        satz.replace(path, hit.start, hit.end, "Korrigierter Absatz.").unwrap();
        let after = satz.world().source_text(path).unwrap();
        assert!(after.contains("= Umsetzung und Evaluierung <kap:umsetzung>\n\nKorrigierter Absatz.\n"), "{}", &after[..200]);
    }

    #[test]
    fn figure_caption_and_table_cell_are_their_own_paragraphs() {
        let mut satz = beispiel();
        satz.compile().unwrap();
        let caption = click_on(&satz, "Arbeit/kapitel/02-grundlagen.typ", "Aufbau des Beispielsystems");
        assert_eq!(caption.text, "Aufbau des Beispielsystems mit drei Komponenten");
        let cell = click_on(&satz, "Arbeit/kapitel/03-umsetzung.typ", "Das System lädt");
        assert_eq!(cell.text, "Das System lädt eine Konfiguration beim Start.");
        let heading = click_on(&satz, "Arbeit/kapitel/04-loesungskonzept.typ", "Datenabgleich");
        assert_eq!(heading.text, "=== Datenabgleich");
    }

    #[test]
    fn first_page_of_chapter_file() {
        let mut satz = beispiel();
        let report = satz.compile().unwrap();
        let page = satz.first_page_of("Arbeit/kapitel/01-einleitung.typ").unwrap();
        assert_eq!(report.page_labels[page], "1");
        assert!(satz.first_page_of("Arbeit/kapitel/02-grundlagen.typ").unwrap() > page);
        assert_eq!(satz.first_page_of("Arbeit/gibt-es-nicht.typ"), None);
    }

    #[test]
    fn content_pages_count_from_page_one_to_bibliography() {
        let mut satz = beispiel();
        let report = satz.compile().unwrap();
        let first = report.page_labels.iter().position(|l| l == "1").unwrap();
        let bibliography = (0..report.page_count)
            .find(|&p| {
                let frame = &satz.document().unwrap().pages()[p].frame;
                format!("{:?}", frame).contains("Quellverzeichnis") && p > first
            })
            .map(|p| p - first);
        let pages = report.content_pages.expect("Quellverzeichnis vorhanden");
        assert!(pages > 10 && pages < report.page_count, "{pages}");
        if let Some(expected) = bibliography {
            assert_eq!(pages, expected);
        }
    }

    #[test]
    fn pdf_is_identical_to_typst_cli() {
        let typst = ["/opt/homebrew/bin/typst", "/usr/local/bin/typst"].into_iter().find(|p| std::path::Path::new(p).is_file());
        let Some(typst) = typst else {
            eprintln!("typst-CLI nicht gefunden, Test übersprungen");
            return;
        };
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit");
        let out = std::env::temp_dir().join(format!("sa-cli-{}.pdf", std::process::id()));
        let timestamp = 1_792_000_000;
        let status = std::process::Command::new(typst)
            .args(["compile", "--root", root, "--creation-timestamp", &timestamp.to_string()])
            .arg(format!("{root}/Arbeit/main.typ"))
            .arg(&out)
            .status()
            .unwrap();
        assert!(status.success());
        let cli = std::fs::read(&out).unwrap();
        std::fs::remove_file(&out).unwrap();

        let mut satz = beispiel();
        satz.compile().unwrap();
        let ours = satz.export_pdf(Some(timestamp)).unwrap();
        // Byte-Gleichheit scheitert nur an der Deflate-Bibliothek, mit der die CLI gebaut
        // wurde. Verglichen wird deshalb der Inhalt: alle Datenströme entpackt, ohne den
        // Querverweis-Strom (Byte-Positionen) und ohne die XMP-InstanceID (Prüfsumme der
        // fertigen Bytes). Die DocumentID bleibt im Vergleich.
        let a = pdf_streams(&ours);
        let b = pdf_streams(&cli);
        assert_eq!(a.len(), b.len(), "gleich viele Datenströme");
        for (i, (x, y)) in a.iter().zip(&b).enumerate() {
            assert!(x == y, "Datenstrom {i} unterscheidet sich inhaltlich");
        }
    }

    /// Datenströme eines PDFs, Flate-Ströme entpackt, ohne Querverweis-Strom.
    fn pdf_streams(pdf: &[u8]) -> Vec<Vec<u8>> {
        let find = |hay: &[u8], needle: &[u8], from: usize| {
            hay[from..].windows(needle.len()).position(|w| w == needle).map(|i| i + from)
        };
        let mut out = Vec::new();
        let mut pos = 0;
        while let Some(start) = find(pdf, b"stream\n", pos) {
            let dict_start = pdf[..start].iter().rposition(|&c| c == b'<').unwrap_or(0).saturating_sub(200);
            let dict = &pdf[dict_start..start];
            let body = start + b"stream\n".len();
            let end = find(pdf, b"\nendstream", body).expect("endstream");
            pos = end + b"\nendstream".len();
            if find(dict, b"/XRef", 0).is_some() {
                continue;
            }
            let raw = &pdf[body..end];
            let mut data = miniz_oxide::inflate::decompress_to_vec_zlib(raw).unwrap_or_else(|_| raw.to_vec());
            if let (Some(a), Some(b)) = (find(&data, b"<xmpMM:InstanceID>", 0), find(&data, b"</xmpMM:InstanceID>", 0)) {
                data.drain(a..b);
            }
            out.push(data);
        }
        out
    }

    #[test]
    fn unix_timestamp_to_datetime() {
        let d = datetime_from_unix(1_792_000_000).unwrap();
        assert_eq!((d.year(), d.month(), d.day(), d.hour(), d.minute()), (Some(2026), Some(10), Some(14), Some(17), Some(46)));
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
