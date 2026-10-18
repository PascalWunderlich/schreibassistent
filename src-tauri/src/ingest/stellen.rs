//! Prüfung der KI-Antwort für eine Quellenseite: Jede „S. n“ muss es im PDF geben, und das
//! Zitat muss (unscharf) im Text dieser Seite stehen. Bestätigte Stellen bekommt die App
//! als Link zur PDF-Seite in Zotero; Fehlschläge werden sichtbar markiert, nicht verworfen.

use std::sync::LazyLock;

use regex::Regex;

use super::extrakt::Extrakt;

/// Höchstens so viele Wörter je Zitat (Systemprompt „Quellenseite“).
pub const MAX_WORDS: usize = 25;
/// Anteil der Zitatwörter, die in gleicher Reihenfolge auf der Seite stehen müssen.
const MIN_MATCH: f64 = 0.85;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Stelle {
    pub label: String,
    pub zitat: String,
    pub warum: String,
    /// PDF-Seite, auf der das Zitat steht (bestätigt), sonst `None`.
    pub pdf_seite: Option<u32>,
    /// Grund, wenn nicht bestätigt.
    pub problem: Option<String>,
}

static LINE: LazyLock<Regex> = LazyLock::new(|| {
    // - S. 14: „Zitat“ – warum     (auch "…", “…”, »…«; Gedankenstrich oder Bindestrich)
    Regex::new(r#"^\s*[-*]\s*(?:\[)?S\.\s*([^\s:\]]+)(?:\][^:]*)?\s*:\s*[„"“»«](.+?)[“"”«»]\s*(?:[–—-]\s*(.*))?$"#).unwrap()
});

/// Text vergleichbar machen: Trennstriche am Zeilenende zusammenziehen, nur Buchstaben und Ziffern.
pub fn normalize(text: &str) -> Vec<String> {
    static BREAK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\p{L})[-\u{00AD}]\s+(\p{Ll})").unwrap());
    let joined = BREAK.replace_all(text, "$1$2");
    joined
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// Anteil der Zitatwörter, die in einem Fenster des Seitentexts in gleicher Reihenfolge stehen (LCS).
fn match_ratio(quote: &[String], page: &[String]) -> f64 {
    if quote.is_empty() {
        return 0.0;
    }
    let window = quote.len() + quote.len() / 4 + 2;
    let mut best = 0usize;
    let starts: Vec<usize> = page.iter().enumerate().filter(|(_, w)| quote.contains(w)).map(|(i, _)| i).collect();
    for &start in &starts {
        let slice = &page[start..(start + window).min(page.len())];
        // LCS-Länge zwischen Zitat und Fenster.
        let mut row = vec![0usize; slice.len() + 1];
        for q in quote {
            let mut prev = 0;
            for (j, w) in slice.iter().enumerate() {
                let tmp = row[j + 1];
                row[j + 1] = if q == w { prev + 1 } else { row[j + 1].max(row[j]) };
                prev = tmp;
            }
        }
        best = best.max(row[slice.len()]);
        if best == quote.len() {
            break;
        }
    }
    best as f64 / quote.len() as f64
}

/// Text einer PDF-Seite (alle Abschnitte), dazu der Anfang der nächsten (Zitat über Seitenumbruch).
fn page_words(extrakt: &Extrakt, pdf_page: u32, with_next: bool) -> Vec<String> {
    let text_of = |p: u32| extrakt.abschnitte.iter().filter(|a| a.seite == p).map(|a| a.text.as_str()).collect::<Vec<_>>().join(" ");
    let mut words = normalize(&text_of(pdf_page));
    if with_next {
        words.extend(normalize(&text_of(pdf_page + 1)).into_iter().take(60));
    }
    words
}

/// Eine Stelle prüfen: Gibt es die Seite, und steht das Zitat dort?
pub fn check(extrakt: &Extrakt, label: &str, zitat: &str, warum: &str) -> Stelle {
    let mut stelle = Stelle { label: label.to_owned(), zitat: zitat.to_owned(), warum: warum.to_owned(), pdf_seite: None, problem: None };
    let pages: Vec<u32> =
        extrakt.labels.iter().enumerate().filter(|(_, l)| l.as_str() == label).map(|(i, _)| i as u32 + 1).collect();
    if pages.is_empty() {
        stelle.problem = Some(format!("Seite {label} gibt es in diesem PDF nicht"));
        return stelle;
    }
    let quote = normalize(zitat);
    if quote.len() > MAX_WORDS + 5 {
        stelle.problem = Some(format!("Zitat zu lang ({} Wörter)", quote.len()));
        return stelle;
    }
    let best = pages.iter().map(|&p| (p, match_ratio(&quote, &page_words(extrakt, p, true)))).max_by(|a, b| a.1.total_cmp(&b.1));
    match best {
        Some((page, ratio)) if ratio >= MIN_MATCH => stelle.pdf_seite = Some(page),
        _ => {
            // Steht es auf einer anderen Seite? Dann diese nennen (hilft beim Nachsehen).
            let elsewhere = (1..=extrakt.seiten)
                .filter(|p| !pages.contains(p))
                .find(|&p| match_ratio(&quote, &page_words(extrakt, p, false)) >= MIN_MATCH)
                .and_then(|p| extrakt.labels.get(p as usize - 1).cloned());
            stelle.problem = Some(match elsewhere {
                Some(other) => format!("Zitat steht nicht auf S. {label}, sondern auf S. {other}"),
                None => format!("Zitat auf S. {label} nicht gefunden"),
            });
        }
    }
    stelle
}

/// Abschnitt „Interessante Stellen“ der KI-Antwort prüfen und neu schreiben.
/// Gibt den neuen Markdown-Text und die geprüften Stellen zurück.
pub fn verify_answer(answer: &str, extrakt: &Extrakt, attachment_key: &str) -> (String, Vec<Stelle>) {
    let mut out = Vec::new();
    let mut stellen = Vec::new();
    let mut in_section = false;
    for line in answer.lines() {
        if line.starts_with("## ") {
            in_section = line.trim_start_matches('#').trim().to_lowercase().starts_with("interessante stellen");
            out.push(line.to_owned());
            continue;
        }
        if !in_section || !line.trim_start().starts_with(['-', '*']) {
            out.push(line.to_owned());
            continue;
        }
        let Some(caps) = LINE.captures(line) else {
            out.push(format!("- ⚠ {} *(Format unklar, nicht geprüft)*", line.trim_start_matches([' ', '-', '*']).trim()));
            continue;
        };
        let stelle = check(extrakt, &caps[1], caps[2].trim(), caps.get(3).map_or("", |m| m.as_str().trim()));
        out.push(render(&stelle, attachment_key));
        stellen.push(stelle);
    }
    (out.join("\n"), stellen)
}

pub fn render(stelle: &Stelle, attachment_key: &str) -> String {
    let warum = if stelle.warum.is_empty() { String::new() } else { format!(" – {}", stelle.warum) };
    match (stelle.pdf_seite, &stelle.problem) {
        (Some(page), _) => format!(
            "- [S. {}](zotero://open-pdf/library/items/{attachment_key}?page={page}): „{}“{warum}",
            stelle.label, stelle.zitat
        ),
        (None, problem) => format!("- ⚠ S. {}: „{}“{warum} *({})*", stelle.label, stelle.zitat, problem.as_deref().unwrap_or("nicht geprüft")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::extrakt::sample;

    #[test]
    fn exact_fuzzy_and_hyphenated_quotes_are_confirmed() {
        let e = sample();
        // wörtlich
        assert_eq!(check(&e, "3", "data decomposition was the hardest part", "").pdf_seite, Some(4));
        // Trennstrich aus dem PDF („capabili- ties“), andere Anführungszeichen/Satzzeichen
        assert_eq!(check(&e, "1", "split into services along business capabilities", "").pdf_seite, Some(2));
        // ein Wort anders (unscharf, ≥ 85 %)
        assert_eq!(
            check(&e, "2", "Most companies migrate step by step and keep the monolith alive during the transition", "").pdf_seite,
            Some(3)
        );
        // römische Seite
        assert_eq!(check(&e, "i", "small autonomous services", "").pdf_seite, Some(1));
    }

    #[test]
    fn wrong_page_missing_page_and_invented_quote_are_flagged() {
        let e = sample();
        let s = check(&e, "1", "data decomposition was the hardest part", "");
        assert_eq!(s.pdf_seite, None);
        assert_eq!(s.problem.as_deref(), Some("Zitat steht nicht auf S. 1, sondern auf S. 3"));
        assert_eq!(check(&e, "99", "anything", "").problem.as_deref(), Some("Seite 99 gibt es in diesem PDF nicht"));
        assert_eq!(check(&e, "2", "Microservices always reduce costs", "").problem.as_deref(), Some("Zitat auf S. 2 nicht gefunden"));
    }

    #[test]
    fn answer_is_rewritten_with_links_and_warnings() {
        let answer = "## Worum es geht\nText.\n\n## Interessante Stellen\n\
            - S. 3: „data decomposition was the hardest part“ – Kernproblem der Migration\n\
            - S. 2: \"Microservices always reduce costs\" – erfunden\n\
            - irgendwas ohne Seite\n\n## Bezüge\n- [[Microservices]]\n";
        let (text, stellen) = verify_answer(answer, &sample(), "ABCD1234");
        assert_eq!(stellen.len(), 2);
        assert!(text.contains("- [S. 3](zotero://open-pdf/library/items/ABCD1234?page=4): „data decomposition was the hardest part“ – Kernproblem der Migration"), "{text}");
        assert!(text.contains("- ⚠ S. 2: „Microservices always reduce costs“ – erfunden *(Zitat auf S. 2 nicht gefunden)*"), "{text}");
        assert!(text.contains("- ⚠ irgendwas ohne Seite *(Format unklar, nicht geprüft)*"));
        assert!(text.contains("## Bezüge\n- [[Microservices]]"), "andere Abschnitte bleiben");
    }
}
