//! Feste Prüfungen ohne KI gegen die Regeln der Klassen „maschinell“ und „vorlage“.
//! Stilregeln (Vale) und Rechtschreibung (LTeX+) laufen getrennt (`vale`, `ltex`).

use std::collections::BTreeSet;

use super::profil::{Art, Element, Position, Profil, QuellenPruefung, Regel, Schwere};
use crate::index::Index;
use crate::quellen::{BIB_FILE, Source};
use crate::satz::Satz;

/// Ein Befund einer Prüfung.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Hinweis {
    pub regel_id: String,
    /// Datei relativ zum Vault; `None` für die ganze Arbeit (z. B. Seitenumfang).
    pub datei: Option<String>,
    /// Zeile (ab 1) und Byte-Position in der Datei.
    pub zeile: Option<usize>,
    pub offset: Option<usize>,
    pub text: String,
    pub schwere: Schwere,
    pub fundstelle: String,
    /// "regel", "vale" oder "ltex".
    pub quelle: String,
}

/// Alles, was die Prüfungen lesen.
pub struct Kontext<'a> {
    pub satz: &'a Satz,
    pub index: &'a Index,
    pub sources: &'a [Source],
}

impl Kontext<'_> {
    fn line(&self, file: &str, offset: usize) -> Option<usize> {
        let text = self.satz.world().source_text(file).ok()?;
        Some(text[..offset.min(text.len())].matches('\n').count() + 1)
    }

    fn hinweis(&self, regel: &Regel, file: Option<&str>, offset: Option<usize>, text: String) -> Hinweis {
        Hinweis {
            regel_id: regel.id.clone(),
            datei: file.map(str::to_owned),
            zeile: match (file, offset) {
                (Some(f), Some(o)) => self.line(f, o),
                _ => None,
            },
            offset,
            text,
            schwere: regel.schwere,
            fundstelle: regel.fundstelle.clone(),
            quelle: "regel".to_owned(),
        }
    }
}

/// Alle festen Prüfungen des Profils. Reihenfolge: wie die Regeln im Profil.
pub fn check(profil: &Profil, ctx: &Kontext) -> Vec<Hinweis> {
    let thesis_type = ctx.satz.thesis_type();
    let mut out = Vec::new();
    for regel in &profil.regeln {
        if regel.gilt_fuer.as_ref().is_some_and(|g| Some(g) != thesis_type.as_ref()) {
            continue;
        }
        match regel.art {
            Some(Art::Seitenumfang) => seitenumfang(regel, ctx, &mut out),
            Some(Art::Beschriftung) => beschriftung(regel, ctx, &mut out),
            Some(Art::Gliederungstiefe) => gliederung(regel, ctx, &mut out),
            Some(Art::Zitierstil) => zitierstil(regel, ctx, &mut out),
            Some(Art::QuellenVollstaendig) => quellen(regel, ctx, &mut out),
            Some(Art::Textbaustein) => textbaustein(regel, ctx, &mut out),
            // Stilregeln prüft Vale; Klasse „urteil“ ohne art prüft P8 auf Nachfrage.
            Some(Art::Stilregel) | None => {}
        }
    }
    out
}

fn seitenumfang(regel: &Regel, ctx: &Kontext, out: &mut Vec<Hinweis>) {
    let (Some(min), Some(max), Some(pages)) = (regel.min, regel.max, ctx.satz.content_pages()) else { return };
    let pages = pages as u32;
    if pages < min || pages > max {
        out.push(ctx.hinweis(regel, None, None, format!("Inhaltsteil hat {pages} Seiten, vorgegeben sind {min}–{max}.")));
    }
}

fn element_name(element: Element) -> (&'static str, &'static str) {
    match element {
        Element::Abbildung => ("abbildung", "Abbildung"),
        Element::Tabelle => ("tabelle", "Tabelle"),
        Element::Listing => ("listing", "Listing"),
    }
}

fn beschriftung(regel: &Regel, ctx: &Kontext, out: &mut Vec<Hinweis>) {
    let Some(element) = regel.element else { return };
    let (kind, name) = element_name(element);
    let referenced: BTreeSet<&str> =
        ctx.index.typ.values().flat_map(|f| f.references.iter().map(|r| r.key.as_str())).collect();
    for (file, typ) in &ctx.index.typ {
        for figure in typ.figures.iter().filter(|f| f.kind.as_deref() == Some(kind)) {
            let Some(caption) = figure.caption else {
                out.push(ctx.hinweis(regel, Some(file), Some(figure.offset), format!("{name} ohne Beschriftung.")));
                continue;
            };
            if let Some(want) = regel.position {
                let body = ctx.satz.rects_of(file, figure.body.0..figure.body.1);
                let cap = ctx.satz.rects_of(file, caption.0..caption.1);
                if let (Some((bp, b)), Some((cp, c))) = (body.first(), cap.first())
                    && bp == cp
                {
                    let above = c.y + c.height / 2.0 < b.y + b.height / 2.0;
                    let is = if above { Position::Oben } else { Position::Unten };
                    if is != want {
                        let (soll, ist) = match want {
                            Position::Oben => ("Überschrift (oben)", "unten"),
                            Position::Unten => ("Unterschrift (unten)", "oben"),
                        };
                        out.push(ctx.hinweis(
                            regel,
                            Some(file),
                            Some(figure.offset),
                            format!("{name}: Beschriftung steht {ist}, vorgegeben ist eine {soll}."),
                        ));
                    }
                }
            }
            if regel.referenziert == Some(true) {
                match &figure.label {
                    None => out.push(ctx.hinweis(
                        regel,
                        Some(file),
                        Some(figure.offset),
                        format!("{name} ohne Label – sie kann im Text nicht referenziert werden."),
                    )),
                    Some(label) if !referenced.contains(label.as_str()) => out.push(ctx.hinweis(
                        regel,
                        Some(file),
                        Some(figure.offset),
                        format!("{name} <{label}> wird im Text nicht referenziert."),
                    )),
                    Some(_) => {}
                }
            }
        }
    }
}

fn gliederung(regel: &Regel, ctx: &Kontext, out: &mut Vec<Hinweis>) {
    for file in ctx.index.chapters() {
        let headings = &ctx.index.typ[file].headings;
        for (i, heading) in headings.iter().enumerate() {
            if let Some(max) = regel.max
                && heading.level > max as usize
            {
                out.push(ctx.hinweis(
                    regel,
                    Some(file),
                    Some(heading.offset),
                    format!("Gliederungsebene {} („{}“), vorgesehen sind höchstens {max} Ebenen.", heading.level, heading.text),
                ));
            }
            if regel.unterabschnitt_allein == Some(true) {
                let children: Vec<_> = headings[i + 1..]
                    .iter()
                    .take_while(|h| h.level > heading.level)
                    .filter(|h| h.level == heading.level + 1)
                    .collect();
                if let [only] = children.as_slice() {
                    out.push(ctx.hinweis(
                        regel,
                        Some(file),
                        Some(only.offset),
                        format!("„{}“ ist der einzige Unterabschnitt von „{}“; es sollten mindestens zwei sein.", only.text, heading.text),
                    ));
                }
            }
        }
    }
}

fn zitierstil(regel: &Regel, ctx: &Kontext, out: &mut Vec<Hinweis>) {
    if regel.fussnoten.as_deref() != Some("verboten") {
        return;
    }
    let bib: BTreeSet<&str> = ctx.sources.iter().map(|s| s.citekey.as_str()).collect();
    for (file, typ) in &ctx.index.typ {
        for r in typ.references.iter().filter(|r| r.in_footnote && bib.contains(r.key.as_str())) {
            out.push(ctx.hinweis(regel, Some(file), Some(r.offset), format!("Zitat @{} in einer Fußnote; Quellen im Text zitieren.", r.key)));
        }
    }
}

fn quellen(regel: &Regel, ctx: &Kontext, out: &mut Vec<Hinweis>) {
    let pruefe = regel.pruefe.clone().unwrap_or_default();
    let bib: BTreeSet<&str> = ctx.sources.iter().map(|s| s.citekey.as_str()).collect();
    let labels: BTreeSet<&str> = ctx.index.typ.values().flat_map(|f| f.labels.iter().map(String::as_str)).collect();
    let cited: BTreeSet<&str> = ctx.index.typ.values().flat_map(|f| f.references.iter().map(|r| r.key.as_str())).collect();

    if pruefe.contains(&QuellenPruefung::ZitiertFehlt) {
        for (file, typ) in &ctx.index.typ {
            for r in typ.references.iter().filter(|r| !bib.contains(r.key.as_str()) && !labels.contains(r.key.as_str())) {
                out.push(ctx.hinweis(
                    regel,
                    Some(file),
                    Some(r.offset),
                    format!("@{} ist weder eine Quelle in {BIB_FILE} noch ein Label.", r.key),
                ));
            }
        }
    }
    let bib_hint = |source: &Source, text: String| {
        let mut h = ctx.hinweis(regel, Some(BIB_FILE), None, text);
        h.zeile = Some(source.line);
        h
    };
    for source in ctx.sources {
        if pruefe.contains(&QuellenPruefung::Ungenutzt) && !cited.contains(source.citekey.as_str()) {
            out.push(bib_hint(source, format!("Quelle {} wird nicht zitiert; ins Quellverzeichnis gehören nur zitierte Quellen.", source.citekey)));
        }
        if pruefe.contains(&QuellenPruefung::Pflichtfelder) {
            let missing = pflichtfelder(source);
            if !missing.is_empty() {
                out.push(bib_hint(source, format!("{}: Angaben fehlen: {}.", source.citekey, missing.join(", "))));
            }
        }
    }
}

/// Fehlende Angaben je Quellentyp (FHDW-Hinweise S. 9–10).
pub fn pflichtfelder(s: &Source) -> Vec<&'static str> {
    let mut missing = Vec::new();
    let mut need = |ok: bool, name: &'static str| {
        if !ok {
            missing.push(name);
        }
    };
    let people = !s.authors.is_empty() || !s.editors.is_empty();
    match s.kind.as_str() {
        "book" => {
            need(people, "Autor/Herausgeber");
            need(s.title.is_some(), "Titel");
            need(s.publisher.is_some(), "Verlag");
            need(s.year.is_some(), "Jahr");
        }
        "article" => {
            need(people, "Autor");
            need(s.title.is_some(), "Titel");
            need(s.container.is_some(), "Zeitschrift");
            need(s.volume.is_some(), "Band");
            need(s.pages.is_some(), "Seiten");
            need(s.year.is_some(), "Jahr");
        }
        "inproceedings" | "incollection" => {
            need(people, "Autor");
            need(s.title.is_some(), "Titel");
            need(s.container.is_some(), "Konferenzband");
            need(s.pages.is_some(), "Seiten");
            need(s.year.is_some(), "Jahr");
        }
        // Internetquellen (FHDW: @misc): Titel, Datum, URL, Zugriffsdatum.
        "misc" | "online" => {
            need(s.title.is_some(), "Titel");
            need(s.year.is_some(), "Datum");
            need(s.url.is_some(), "URL");
            need(s.urldate.is_some() || s.url.is_none(), "Zugriffsdatum");
        }
        _ => {
            need(s.title.is_some(), "Titel");
            need(s.year.is_some(), "Jahr");
        }
    }
    missing
}

fn textbaustein(regel: &Regel, ctx: &Kontext, out: &mut Vec<Hinweis>) {
    let Some(wortlaut) = &regel.wortlaut else { return };
    let pattern = wortlaut_regex(wortlaut);
    let texts = ctx.satz.page_texts();
    // Ohne gesetztes Dokument (Satzfehler) lässt sich der Wortlaut nicht beurteilen.
    if texts.is_empty() {
        return;
    }
    let Some(page) = texts.iter().find(|t| t.contains("Erklärung der Selbstständigkeit")) else {
        out.push(ctx.hinweis(regel, None, None, "Die Erklärung der Selbstständigkeit fehlt.".to_owned()));
        return;
    };
    let page = Some(page);
    // Hochgestellte Fußnotenzeichen liegen höher als ihre Zeile und landen als eigene Zeile
    // davor; solche Zeilen (nur Ziffern) vor dem Vergleich entfernen.
    let without_markers = |t: &str| {
        t.lines().filter(|l| !l.trim().chars().all(|c| c.is_ascii_digit())).collect::<Vec<_>>().join("\n")
    };
    let found = page.is_some_and(|t| pattern.is_match(&canonical(&without_markers(t))));
    if !found {
        out.push(ctx.hinweis(regel, None, None, "Die Erklärung entspricht nicht dem vorgegebenen Wortlaut.".to_owned()));
    }
}

/// Nur Buchstaben und Ziffern, klein: unempfindlich für Silbentrennung, Zeilenumbrüche und
/// Leerraum um Satzzeichen; Wörter und ihre Reihenfolge müssen exakt stimmen.
fn canonical(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Wortlaut → Muster auf dem kanonischen Text: `{art}` = Praxis-/Bachelorarbeit,
/// `{fn}` = Fußnotenzeichen (Ziffern).
fn wortlaut_regex(wortlaut: &str) -> regex::Regex {
    let mut pattern = String::new();
    for (i, part) in wortlaut.split("{art}").enumerate() {
        if i > 0 {
            pattern.push_str("(?:praxisarbeit|bachelorarbeit)");
        }
        for (j, piece) in part.split("{fn}").enumerate() {
            if j > 0 {
                pattern.push_str(r"\d*");
            }
            pattern.push_str(&regex::escape(&canonical(piece)));
        }
    }
    regex::Regex::new(&pattern).expect("Muster aus Wortlaut")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wortlaut_matches_with_placeholders_and_hyphenation() {
        let pattern = wortlaut_regex("Die vorliegende {art} mit KI-gestützte Werkzeuge im Anhang{fn}) dokumentiert.");
        assert!(pattern.is_match(&canonical("Die vorlie-\ngende Praxisarbeit mit KI-\ngestützte Werkzeuge im Anhang1) dokumentiert.")));
        assert!(!pattern.is_match(&canonical("Die vorliegende Arbeit mit KI-gestützte Werkzeuge im Anhang) dokumentiert.")));
        assert!(!pattern.is_match(&canonical("Die vorliegende Praxisarbeit mit KI-gestützten Werkzeugen im Anhang) dokumentiert.")));
    }

    #[test]
    fn mandatory_fields_per_type() {
        let base = Source {
            citekey: "x".into(),
            kind: "misc".into(),
            authors: vec![],
            title: Some("T".into()),
            year: Some("2021".into()),
            container: None,
            publisher: None,
            url: Some("https://x".into()),
            doi: None,
            file: None,
            volume: None,
            pages: None,
            urldate: None,
            editors: vec![],
            line: 1,
        };
        assert_eq!(pflichtfelder(&base), ["Zugriffsdatum"]);
        let book = Source { kind: "book".into(), url: None, ..base.clone() };
        assert_eq!(pflichtfelder(&book), ["Autor/Herausgeber", "Verlag"]);
    }
}
