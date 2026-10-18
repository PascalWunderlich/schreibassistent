//! KI-Prüfungen auf Nachfrage (P8): Belegbedarf, Quelle ↔ Aussage, Roter Faden,
//! „Was habe ich hierzu?“, Urteilsregeln und Regelprofil aus einem Vorgaben-PDF.
//!
//! Jede Aufgabe hat ihren eigenen Systemprompt (Vault: „Systemprompts in der App“), läuft
//! über `ki::anfrage` (protokolliert) und schickt wenig Kontext: einen Absatz bzw. ein
//! Kapitel und höchstens 5 Originalabschnitte. Jede Antwort geht durch
//! `ohne_formulierungen`: Text, der weder in der Arbeit noch in den Quellen steht und als
//! Zitat oder Vorschlag daherkommt, wird entfernt (Leitplanke 1, Abnahme P8).

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use super::einstellungen::{Aufgabe, KiSettings};
use crate::ingest::extrakt::{self, Extrakt};
use crate::ingest::stellen::{match_ratio, normalize};
use crate::ingest::suche::Suche;
use crate::protokoll::Kontext;

const BELEGBEDARF_PROMPT: &str = "Prüfe den Absatz. Liste Sätze, die eine Tatsache, Zahl, Definition oder fremde
These enthalten und keinen Beleg (@citekey) haben. Pro Satz: Satznummer, Grund
(1 Zeile). Keine Formulierungsvorschläge.

Antworte nur mit Zeilen der Form:
- Satz <Nummer>: <Grund>
Wenn kein Satz einen Beleg braucht: - keine";

const QUELLE_AUSSAGE_PROMPT: &str = "Aussage aus der Arbeit: {satz} (zitiert {citekey}).
Originalabschnitte: {abschnitte mit Seite}.
Gib an: stützt / stützt teilweise / nicht gefunden, mit Seite und kurzer
Begründung. Bei Unsicherheit „unsicher“ sagen. Keine Umformulierung des Satzes.

Antworte genau in zwei Zeilen:
Einschätzung: <stützt | stützt teilweise | nicht gefunden | unsicher>, S. <Seite oder –>
Begründung: <1–2 Sätze, sachlich, als Hinweis, nicht als Urteil>";

const ROTER_FADEN_PROMPT: &str = "Gliederung: {gliederung}. Einleitung verspricht: {forschungsfrage/ziel}.
Kapitel: {kapiteltext}. Nenne bis zu 5 Stellen, an denen das Kapitel vom
Versprochenen abweicht, etwas fehlt oder sich wiederholt, mit Absatznummer.
Formuliere Fragen an den Autor, keine Lösungen.

Die Platzhalter stehen in der Nachricht. Antworte nur mit Zeilen der Form:
- Absatz <Nummer>: <Frage an den Autor>";

/// Eigener Prompt (im Vault keiner), siehe DECISIONS.
const HIERZU_PROMPT: &str = "Hier ist ein Absatz aus der Arbeit und Material, das der Autor dazu hat
(Wegweiser-Seiten, Notizen, Fundstellen mit Seite). Ordne ein, welches Material zum
Absatz passt und wofür (Beleg, Gegenposition, Vertiefung), und welche Frage offen bleibt.
Nenne immer Citekey und Seite. Höchstens 6 Zeilen als Liste. Keine Formulierungen.";

/// Eigener Prompt für Regeln der Klasse „urteil“ (im Vault keiner).
const URTEIL_PROMPT: &str = "Prüfe das Kapitel gegen die folgenden Vorgaben. Nenne nur Stellen, an denen eine
Vorgabe möglicherweise nicht erfüllt ist, als Frage an den Autor, mit Absatznummer.
Kein Urteil über die Note, keine Formulierungen. Antworte nur mit Zeilen der Form:
- [<regel-id>] Absatz <Nummer>: <Frage>
Wenn nichts auffällt: - keine";

const REGELPROFIL_PROMPT: &str = "Extrahiere alle Form- und Inhaltsvorgaben aus dem Dokument als Regeln im
JSON-Schema {schema}. Pro Regel: klasse, art (nur aus dem Vokabular, sonst
klasse=urteil), Werte, fundstelle (Seite + Abschnitt), wörtliches Zitat der Vorgabe.
Erfinde keine Werte. Unklare Vorgaben als klasse=urteil mit Zitat.

Antworte nur mit einem JSON-Array von Regel-Objekten (ohne Codeblock). Jedes Objekt hat
die Felder des Schemas für eine Regel und zusätzlich \"zitat\" (wörtlich, höchstens 25
Wörter, aus dem Abschnitt der genannten Seite). fundstelle beginnt mit \"S. <Seite>\".";

// ---------- Text vorbereiten ----------

/// Zeile ohne Typst-Kommentar (`// …`, nicht in `https://`).
fn without_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut i = 0;
    while let Some(pos) = line[i..].find("//") {
        let at = i + pos;
        if at == 0 || bytes[at - 1] != b':' {
            return &line[..at];
        }
        i = at + 2;
    }
    line
}

/// Text ohne Kommentarzeilen und Zeilenkommentare (geht so an die KI).
pub fn strip_comments(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .map(|l| without_comment(l).trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Absatz aus Fließtext? (nicht Überschrift, Code, Kommentar)
fn is_prose(block: &str) -> bool {
    let first = block.trim_start();
    !(first.starts_with('=') || first.starts_with('#') || first.starts_with("//"))
}

static CITE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@([A-Za-z][\w:.\-]*[\w])(\[[^\]]*\])?|#cite\(<([^>]+)>").unwrap());

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Satz {
    /// Nummer im Absatz bzw. Kapitel, ab 1.
    pub nr: usize,
    /// Byte-Position im Quelltext.
    pub start: usize,
    pub text: String,
    /// Zitierte Quellen (nur Citekeys aus der `.bib`) mit Ergänzung („S. 14“).
    pub belege: Vec<(String, Option<String>)>,
}

const ABBREVIATIONS: [&str; 16] = ["z", "b", "d", "h", "s", "vgl", "bzw", "u", "a", "abb", "kap", "tab", "et", "al", "ca", "nr"];

/// Sätze eines Bereichs (nur Fließtext-Absätze, ohne Kommentare), mit Position und Belegen.
pub fn sentences(text: &str, range: (usize, usize), bib: &BTreeSet<String>) -> Vec<Satz> {
    let mut out = Vec::new();
    for (start, end) in super::blocks(text) {
        if end <= range.0 || start >= range.1 || !is_prose(&text[start..end]) {
            continue;
        }
        // Zeichen mit Position, Kommentare weg, Zeilenumbrüche als Leerzeichen.
        let mut chars: Vec<(usize, char)> = Vec::new();
        let mut pos = start;
        for line in text[start..end].split_inclusive('\n') {
            if !line.trim_start().starts_with("//") {
                let kept = without_comment(line.trim_end_matches('\n'));
                chars.extend(kept.char_indices().map(|(i, c)| (pos + i, c)));
                chars.push((pos + kept.len(), ' '));
            }
            pos += line.len();
        }
        let mut sentence_start = 0;
        let mut depth = 0i32;
        let flush = |from: usize, to: usize, out: &mut Vec<Satz>| {
            let s: String = chars[from..to].iter().map(|(_, c)| *c).collect();
            let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
            if s.chars().filter(|c| c.is_alphabetic()).count() < 3 {
                return;
            }
            let belege = CITE
                .captures_iter(&s)
                .filter_map(|c| {
                    let key = c.get(1).or(c.get(3))?.as_str().to_owned();
                    let supplement = c.get(2).map(|m| m.as_str().trim_matches(['[', ']']).to_owned());
                    bib.contains(&key).then_some((key, supplement))
                })
                .collect();
            let first = (from..to).find(|&i| !chars[i].1.is_whitespace()).unwrap_or(from);
            out.push(Satz { nr: 0, start: chars[first].0, text: s, belege });
        };
        for i in 0..chars.len() {
            let c = chars[i].1;
            match c {
                '[' | '(' => depth += 1,
                ']' | ')' => depth -= 1,
                '.' | '!' | '?' if depth <= 0 => {
                    let next = chars[i + 1..].iter().map(|(_, c)| *c).find(|c| !c.is_whitespace());
                    let followed_by_space = chars.get(i + 1).is_some_and(|(_, c)| c.is_whitespace());
                    let word: String =
                        chars[sentence_start..i].iter().rev().map(|(_, c)| *c).take_while(|c| c.is_alphanumeric()).collect::<String>().chars().rev().collect();
                    let abbreviation = c == '.' && (ABBREVIATIONS.contains(&word.to_lowercase().as_str()) || word.chars().count() == 1);
                    let starts_new = next.is_none_or(|n| n.is_uppercase() || "„\"@#".contains(n));
                    if followed_by_space && !abbreviation && starts_new {
                        flush(sentence_start, i + 1, &mut out);
                        sentence_start = i + 1;
                    }
                }
                _ => {}
            }
        }
        if sentence_start < chars.len() {
            flush(sentence_start, chars.len(), &mut out);
        }
    }
    for (i, s) in out.iter_mut().enumerate() {
        s.nr = i + 1;
    }
    out
}

/// Bereich des Absatzes um `offset` bzw. die ganze Datei.
fn range_of(text: &str, offset: Option<usize>) -> (usize, usize) {
    match offset {
        None => (0, text.len()),
        Some(offset) => {
            let blocks = super::blocks(text);
            let i = blocks.iter().rposition(|(s, _)| *s <= offset).unwrap_or(0);
            blocks.get(i).copied().unwrap_or((0, text.len()))
        }
    }
}

fn read(root: &Path, datei: &str) -> Result<String, String> {
    std::fs::read_to_string(crate::projekt::files::resolve(root, datei)?).map_err(|e| format!("{datei}: {e}"))
}

fn absatz_nr(text: &str, offset: usize) -> usize {
    super::blocks(text).iter().rposition(|(s, _)| *s <= offset).map_or(1, |i| i + 1)
}

// ---------- Schutz gegen Formulierungen ----------

static QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"[„"“»]([^„"“”»«\n]{12,}?)[“"”«]"#).unwrap());
static SUGGESTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*[-*]?\s*(formulierungsvorschlag|vorschlag|besser|umformulierung|alternative formulierung|neue fassung|mögliche formulierung|beispielsatz|z\.\s*b\.\s*so)\s*:").unwrap()
});

/// Zitate, die weder in der Arbeit noch in den Quellen stehen, und als Vorschlag markierte
/// Zeilen entfernen. Gibt den bereinigten Text und die Zahl der Entfernungen zurück.
pub fn ohne_formulierungen(answer: &str, erlaubt: &[&str]) -> (String, usize) {
    let allowed: Vec<Vec<String>> = erlaubt.iter().map(|t| normalize(t)).collect();
    let mut removed = 0;
    let mut lines = Vec::new();
    for line in answer.lines() {
        if SUGGESTION.is_match(line) {
            removed += 1;
            lines.push("- [Formulierungsvorschlag entfernt]".to_owned());
            continue;
        }
        let replaced = QUOTE.replace_all(line, |c: &regex::Captures| {
            let words = normalize(&c[1]);
            if words.len() < 4 || allowed.iter().any(|a| match_ratio(&words, a) >= 0.85) {
                c[0].to_owned()
            } else {
                removed += 1;
                "[Formulierung entfernt]".to_owned()
            }
        });
        lines.push(replaced.into_owned());
    }
    (lines.join("\n"), removed)
}

// ---------- Belegbedarf ----------

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct BelegHinweis {
    pub nr: usize,
    pub start: usize,
    pub satz: String,
    pub grund: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Ergebnis<T> {
    pub hinweise: Vec<T>,
    /// Entfernte Formulierungen (sollte 0 sein).
    pub entfernt: usize,
    pub modell: String,
    pub datei: String,
    pub sonstiges: Vec<String>,
}

static NUMBERED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*[-*]?\s*(?:satz|absatz|\[a?)?\s*(\d+)\]?\s*[:.)–-]?\s*(.+)$").unwrap());

pub async fn belegbedarf(root: &Path, settings: &KiSettings, datei: &str, offset: Option<usize>, bib: &BTreeSet<String>) -> Result<Ergebnis<BelegHinweis>, String> {
    let text = read(root, datei)?;
    let saetze = sentences(&text, range_of(&text, offset), bib);
    if saetze.is_empty() {
        return Err("Kein Fließtext im gewählten Bereich".into());
    }
    let mut message = format!("{} aus {datei}, Sätze nummeriert:\n", if offset.is_some() { "Absatz" } else { "Kapitel" });
    for s in &saetze {
        let beleg = if s.belege.is_empty() { "kein Beleg".to_owned() } else { format!("Beleg: {}", s.belege.iter().map(|b| format!("@{}", b.0)).collect::<Vec<_>>().join(", ")) };
        message.push_str(&format!("[{}] {} ({beleg})\n", s.nr, s.text));
    }
    let kontext = Kontext { datei: Some(datei.to_owned()), absatz: offset.map(|o| absatz_nr(&text, o)) };
    let system = format!("{}\n\n{BELEGBEDARF_PROMPT}", super::BASIS_PROMPT);
    let answer = super::anfrage(root, settings, Aufgabe::Belegpruefung, &system, &message, kontext).await?;
    let all_text: Vec<&str> = saetze.iter().map(|s| s.text.as_str()).collect();
    let (clean, entfernt) = ohne_formulierungen(&answer.text, &all_text);
    let mut hinweise: Vec<BelegHinweis> = Vec::new();
    let mut sonstiges = Vec::new();
    for line in clean.lines().filter(|l| !l.trim().is_empty()) {
        match NUMBERED.captures(line).and_then(|c| Some((c[1].parse::<usize>().ok()?, c[2].trim().to_owned()))) {
            Some((nr, grund)) => {
                // Sätze mit Beleg und doppelte Nummern ignorieren (Belegbedarf heißt: ohne Beleg).
                if let Some(s) = saetze.iter().find(|s| s.nr == nr && s.belege.is_empty())
                    && !hinweise.iter().any(|h| h.nr == nr)
                {
                    hinweise.push(BelegHinweis { nr, start: s.start, satz: s.text.clone(), grund });
                }
            }
            None if !line.to_lowercase().contains("keine") => sonstiges.push(line.trim().to_owned()),
            None => {}
        }
    }
    Ok(Ergebnis { hinweise, entfernt, modell: answer.modell, datei: datei.to_owned(), sonstiges })
}

// ---------- Quelle ↔ Aussage ----------

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Fund {
    pub citekey: String,
    pub label: String,
    pub pdf_seite: u32,
    pub text: String,
    /// `zotero://open-pdf/…`, wenn der Anhang bekannt ist.
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AussageHinweis {
    pub start: usize,
    pub satz: String,
    pub citekey: String,
    pub ergaenzung: Option<String>,
    pub abschnitte: Vec<Fund>,
    /// stützt, stützt teilweise, nicht gefunden, unsicher – oder `None` ohne KI-Anfrage.
    pub einschaetzung: Option<String>,
    pub seite: Option<String>,
    pub begruendung: String,
    pub problem: Option<String>,
}

/// Anhang-Schlüssel aus der Wegweiser-Seite (Links `zotero://open-pdf/library/items/<key>`).
fn attachment_key(root: &Path, citekey: &str) -> Option<String> {
    static KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"zotero://open-pdf/library/items/([A-Z0-9]+)\?").unwrap());
    let page = std::fs::read_to_string(root.join(crate::ingest::page_path(citekey))).ok()?;
    KEY.captures(&page).map(|c| c[1].to_owned())
}

/// „S. 14“, „S. 14–15“, „14 f.“ → „14“.
fn page_from(supplement: &str) -> Option<String> {
    let rest = supplement.trim().trim_start_matches("S.").trim_start_matches("p.").trim();
    let page: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
    (!page.is_empty()).then_some(page)
}

/// Bis zu `n` Abschnitte: zuerst von der zitierten Seite, sonst Volltextsuche nach den Satzwörtern.
fn candidates(root: &Path, extrakt: &Extrakt, citekey: &str, satz: &str, page: Option<&str>, n: usize) -> Vec<usize> {
    let words: Vec<String> = normalize(satz).into_iter().filter(|w| w.chars().count() >= 4).collect();
    let ranked: Vec<usize> = Suche::open(root)
        .and_then(|s| s.search(Some(citekey), &words, 50))
        .map(|hits| hits.into_iter().map(|h| h.nr).collect())
        .unwrap_or_default();
    let mut out: Vec<usize> = Vec::new();
    if let Some(page) = page {
        let on_page: Vec<usize> = extrakt.abschnitte.iter().enumerate().filter(|(_, a)| a.label == page).map(|(i, _)| i).collect();
        // Auf der Seite zuerst die Treffer der Suche, dann der Reihe nach; kurze Schnipsel zuletzt.
        let mut sorted: Vec<usize> = ranked.iter().copied().filter(|i| on_page.contains(i)).collect();
        let long: Vec<usize> = on_page.iter().copied().filter(|i| extrakt.abschnitte[*i].text.len() > 80 && !sorted.contains(i)).collect();
        sorted.extend(long);
        let rest: Vec<usize> = on_page.iter().copied().filter(|i| !sorted.contains(i)).collect();
        sorted.extend(rest);
        out.extend(sorted.into_iter().take(n));
    }
    for i in ranked {
        if out.len() >= n {
            break;
        }
        if !out.contains(&i) {
            out.push(i);
        }
    }
    out
}

static VERDICT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(stützt teilweise|stuetzt teilweise|nicht gefunden|unsicher|stützt|stuetzt)").unwrap());
static PAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"S\.\s*([0-9ivxlcIVXLC]+)").unwrap());

pub async fn quelle_aussage(root: &Path, settings: &KiSettings, datei: &str, offset: usize, bib: &BTreeSet<String>) -> Result<Ergebnis<AussageHinweis>, String> {
    let text = read(root, datei)?;
    let saetze: Vec<Satz> = sentences(&text, range_of(&text, Some(offset)), bib).into_iter().filter(|s| !s.belege.is_empty()).collect();
    if saetze.is_empty() {
        return Err("Im Absatz wird keine Quelle zitiert".into());
    }
    let mut hinweise = Vec::new();
    let mut entfernt_total = 0;
    let mut modell = String::new();
    for satz in &saetze {
        for (citekey, ergaenzung) in &satz.belege {
            let mut hinweis = AussageHinweis {
                start: satz.start,
                satz: satz.text.clone(),
                citekey: citekey.clone(),
                ergaenzung: ergaenzung.clone(),
                abschnitte: Vec::new(),
                einschaetzung: None,
                seite: None,
                begruendung: String::new(),
                problem: None,
            };
            let Some(extrakt) = extrakt::cached_any(root, citekey) else {
                hinweis.problem = Some("Quelle nicht eingelesen (Wegweiser → Einlesen)".into());
                hinweise.push(hinweis);
                continue;
            };
            let page = ergaenzung.as_deref().and_then(page_from);
            let key = attachment_key(root, citekey);
            hinweis.abschnitte = candidates(root, &extrakt, citekey, &satz.text, page.as_deref(), 3)
                .into_iter()
                .map(|i| {
                    let a = &extrakt.abschnitte[i];
                    Fund {
                        citekey: citekey.clone(),
                        label: a.label.clone(),
                        pdf_seite: a.seite,
                        text: a.text.clone(),
                        link: key.as_ref().map(|k| format!("zotero://open-pdf/library/items/{k}?page={}", a.seite)),
                    }
                })
                .collect();
            if hinweis.abschnitte.is_empty() {
                hinweis.problem = Some("keine passenden Abschnitte gefunden".into());
                hinweise.push(hinweis);
                continue;
            }
            let abschnitte = hinweis.abschnitte.iter().map(|f| format!("[S. {}] {}", f.label, f.text)).collect::<Vec<_>>().join("\n");
            let system = format!(
                "{}\n\n{}",
                super::BASIS_PROMPT,
                QUELLE_AUSSAGE_PROMPT
                    .replace("{satz}", "(siehe Nachricht)")
                    .replace("{citekey}", citekey)
                    .replace("{abschnitte mit Seite}", "(siehe Nachricht)")
            );
            let message = format!(
                "Aussage aus der Arbeit: {} (zitiert {citekey}{}).\n\nOriginalabschnitte:\n{abschnitte}",
                satz.text,
                ergaenzung.as_ref().map(|e| format!(", {e}")).unwrap_or_default()
            );
            let kontext = Kontext { datei: Some(datei.to_owned()), absatz: Some(absatz_nr(&text, offset)) };
            let answer = super::anfrage(root, settings, Aufgabe::Belegpruefung, &system, &message, kontext).await?;
            modell = answer.modell.clone();
            let allowed: Vec<&str> = std::iter::once(satz.text.as_str()).chain(hinweis.abschnitte.iter().map(|f| f.text.as_str())).collect();
            let (clean, entfernt) = ohne_formulierungen(&answer.text, &allowed);
            entfernt_total += entfernt;
            let first = clean.lines().find(|l| VERDICT.is_match(l)).unwrap_or("");
            hinweis.einschaetzung = VERDICT.find(first).map(|m| m.as_str().to_lowercase().replace("stuetzt", "stützt"));
            hinweis.seite = PAGE.captures(first).map(|c| c[1].to_owned());
            hinweis.begruendung = clean
                .lines()
                .find_map(|l| l.trim().strip_prefix("Begründung:").map(|b| b.trim().to_owned()))
                .unwrap_or_else(|| clean.lines().filter(|l| !l.contains("Einschätzung")).collect::<Vec<_>>().join(" ").trim().to_owned());
            if let Some(seite) = &hinweis.seite
                && !hinweis.abschnitte.iter().any(|f| &f.label == seite)
            {
                hinweis.problem = Some(format!("KI nennt S. {seite}, die nicht unter den gezeigten Abschnitten ist"));
            }
            if hinweis.einschaetzung.is_none() {
                hinweis.problem = Some("Antwort ohne Einschätzung (steht im Protokoll)".into());
            }
            hinweise.push(hinweis);
        }
    }
    Ok(Ergebnis { hinweise, entfernt: entfernt_total, modell, datei: datei.to_owned(), sonstiges: Vec::new() })
}

// ---------- Roter Faden und Urteilsregeln ----------

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AbsatzHinweis {
    pub absatz: Option<usize>,
    pub start: Option<usize>,
    pub regel: Option<String>,
    pub text: String,
}

/// Kapitel mit nummerierten Absätzen „[A n] …“ (Nummern wie im Protokoll).
fn numbered_chapter(text: &str) -> String {
    super::blocks(text)
        .iter()
        .enumerate()
        .filter_map(|(i, (s, e))| {
            let block = strip_comments(&text[*s..*e]);
            let block = block.trim();
            (!block.is_empty()).then(|| format!("[A{}] {}", i + 1, block))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn absatz_hinweise(clean: &str, text: &str, with_rule: bool) -> Vec<AbsatzHinweis> {
    static RULE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*[-*]?\s*\[([\w-]+)\]\s*(.*)$").unwrap());
    let blocks = super::blocks(text);
    clean
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().trim_start_matches(['-', '*']).trim().eq_ignore_ascii_case("keine"))
        .map(|line| {
            let (regel, rest) = match with_rule.then(|| RULE.captures(line)).flatten() {
                Some(c) => (Some(c[1].to_owned()), c[2].to_owned()),
                None => (None, line.to_owned()),
            };
            match NUMBERED.captures(&rest).and_then(|c| Some((c[1].parse::<usize>().ok()?, c[2].trim().to_owned()))) {
                Some((nr, frage)) if nr >= 1 && nr <= blocks.len() => {
                    AbsatzHinweis { absatz: Some(nr), start: Some(blocks[nr - 1].0), regel, text: frage }
                }
                _ => AbsatzHinweis { absatz: None, start: None, regel, text: rest.trim().trim_start_matches(['-', '*']).trim().to_owned() },
            }
        })
        .collect()
}

pub async fn roter_faden(root: &Path, settings: &KiSettings, datei: &str, outline: &[String], einleitung: Option<&str>) -> Result<Ergebnis<AbsatzHinweis>, String> {
    let text = read(root, datei)?;
    let intro = match einleitung.filter(|e| *e != datei) {
        Some(file) => strip_comments(&read(root, file)?),
        None => "(keine eigene Einleitung gefunden)".into(),
    };
    let intro: String = intro.split_whitespace().take(1200).collect::<Vec<_>>().join(" ");
    let message = format!(
        "Gliederung:\n{}\n\nEinleitung verspricht (Text der Einleitung):\n{intro}\n\nKapitel {datei}, Absätze nummeriert:\n{}",
        outline.iter().map(|h| format!("- {h}")).collect::<Vec<_>>().join("\n"),
        numbered_chapter(&text)
    );
    let system = format!("{}\n\n{ROTER_FADEN_PROMPT}", super::BASIS_PROMPT);
    let kontext = Kontext { datei: Some(datei.to_owned()), absatz: None };
    let answer = super::anfrage(root, settings, Aufgabe::Feedback, &system, &message, kontext).await?;
    let (clean, entfernt) = ohne_formulierungen(&answer.text, &[&text, &intro]);
    Ok(Ergebnis { hinweise: absatz_hinweise(&clean, &text, false), entfernt, modell: answer.modell, datei: datei.to_owned(), sonstiges: Vec::new() })
}

pub async fn urteilsregeln(root: &Path, settings: &KiSettings, datei: &str) -> Result<Ergebnis<AbsatzHinweis>, String> {
    let profil = crate::pruefung::profil::load(root)?.ok_or("Kein Regelprofil (System/regelprofil.yaml)")?;
    let regeln: Vec<_> = profil.regeln.iter().filter(|r| r.klasse == crate::pruefung::profil::Klasse::Urteil).collect();
    if regeln.is_empty() {
        return Err("Das Regelprofil hat keine Regeln der Klasse „urteil“".into());
    }
    let text = read(root, datei)?;
    let message = format!(
        "Vorgaben:\n{}\n\nKapitel {datei}, Absätze nummeriert:\n{}",
        regeln.iter().map(|r| format!("- [{}] {} ({})", r.id, r.text.as_deref().unwrap_or(""), r.fundstelle)).collect::<Vec<_>>().join("\n"),
        numbered_chapter(&text)
    );
    let system = format!("{}\n\n{URTEIL_PROMPT}", super::BASIS_PROMPT);
    let kontext = Kontext { datei: Some(datei.to_owned()), absatz: None };
    let answer = super::anfrage(root, settings, Aufgabe::Feedback, &system, &message, kontext).await?;
    let (clean, entfernt) = ohne_formulierungen(&answer.text, &[&text]);
    let known: BTreeSet<&str> = regeln.iter().map(|r| r.id.as_str()).collect();
    let mut hinweise = absatz_hinweise(&clean, &text, true);
    for h in &mut hinweise {
        if h.regel.as_deref().is_some_and(|r| !known.contains(r)) {
            h.text = format!("{} (unbekannte Regel)", h.text);
        }
    }
    Ok(Ergebnis { hinweise, entfernt, modell: answer.modell, datei: datei.to_owned(), sonstiges: Vec::new() })
}

// ---------- Was habe ich hierzu? ----------

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Hierzu {
    /// Zitierte Quellen des Absatzes mit Wegweiser-Seite (falls vorhanden).
    pub quellen: Vec<(String, Option<String>)>,
    /// Notizen und Wissensseiten zu diesen Quellen oder diesem Kapitel.
    pub seiten: Vec<String>,
    /// Passende Abschnitte aus allen eingelesenen Quellen (Volltextsuche).
    pub fundstellen: Vec<Fund>,
    pub absatz: String,
}

/// Ohne KI: Index und Volltextsuche.
pub fn hierzu(root: &Path, index: &crate::index::Index, bib: &BTreeSet<String>, datei: &str, offset: usize) -> Result<Hierzu, String> {
    let text = read(root, datei)?;
    let (s, e) = range_of(&text, Some(offset));
    let absatz = strip_comments(&text[s..e]).trim().to_owned();
    let saetze = sentences(&text, (s, e), bib);
    let keys: BTreeSet<String> = saetze.iter().flat_map(|s| s.belege.iter().map(|b| b.0.clone())).collect();
    let quellen = keys
        .iter()
        .map(|k| {
            let page = crate::ingest::page_path(k);
            (k.clone(), root.join(&page).is_file().then_some(page))
        })
        .collect();
    let stem = Path::new(datei).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let seiten = index
        .md
        .iter()
        .filter(|(path, md)| {
            !path.starts_with(crate::ingest::SOURCE_DIR)
                && (md.kapitel.contains(&stem) || md.citekeys.iter().chain(md.links.iter()).any(|k| keys.contains(k)))
        })
        .map(|(path, _)| path.clone())
        .collect();
    let words: Vec<String> = normalize(&absatz).into_iter().filter(|w| w.chars().count() >= 5).collect();
    let fundstellen = match Suche::open(root).and_then(|s| s.search(None, &words, 5)) {
        Ok(hits) => hits
            .into_iter()
            .map(|h| {
                let pdf_seite = h.seite;
                let link = attachment_key(root, &h.citekey).map(|k| format!("zotero://open-pdf/library/items/{k}?page={pdf_seite}"));
                Fund { citekey: h.citekey, label: h.label, pdf_seite, text: h.text, link }
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    Ok(Hierzu { quellen, seiten, fundstellen, absatz })
}

/// Mit KI: das gefundene Material zum Absatz einordnen.
pub async fn hierzu_einordnen(root: &Path, settings: &KiSettings, datei: &str, offset: usize, material: &Hierzu) -> Result<(String, usize, String), String> {
    let mut message = format!("Absatz aus {datei}:\n{}\n\nMaterial:\n", material.absatz);
    for (key, page) in &material.quellen {
        if let Some(page) = page
            && let Ok(text) = std::fs::read_to_string(root.join(page))
        {
            let body = text.split_once("\n---\n").map_or(text.as_str(), |x| x.1);
            let short: String = body.chars().take(2500).collect();
            message.push_str(&format!("\n=== Wegweiser {key} ===\n{short}\n"));
        }
    }
    for path in material.seiten.iter().take(3) {
        if let Ok(text) = std::fs::read_to_string(root.join(path)) {
            let short: String = text.chars().take(1500).collect();
            message.push_str(&format!("\n=== Notiz {path} ===\n{short}\n"));
        }
    }
    for f in material.fundstellen.iter().take(5) {
        message.push_str(&format!("\n[{} S. {}] {}\n", f.citekey, f.label, f.text));
    }
    let system = format!("{}\n\n{HIERZU_PROMPT}", super::BASIS_PROMPT);
    let text = read(root, datei)?;
    let kontext = Kontext { datei: Some(datei.to_owned()), absatz: Some(absatz_nr(&text, offset)) };
    let answer = super::anfrage(root, settings, Aufgabe::Feedback, &system, &message, kontext).await?;
    let (clean, entfernt) = ohne_formulierungen(&answer.text, &[&message]);
    Ok((clean, entfernt, answer.modell))
}

// ---------- Regelprofil aus PDF ----------

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Vorschlag {
    /// Regel ohne `zitat`, wie sie ins Profil käme.
    pub regel: serde_json::Value,
    pub yaml: String,
    pub zitat: String,
    /// Zitat auf der genannten Seite gefunden.
    pub zitat_ok: bool,
    pub problem: Option<String>,
}

fn schema_text() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../typst/fhdw-arbeit/regelprofil.schema.json")).unwrap_or_default()
}

/// Einzelne Regel gegen das Schema prüfen (über einen Profil-Rahmen mit genau dieser Regel).
fn check_rule(regel: &serde_json::Value) -> Result<String, String> {
    let yaml = serde_norway::to_string(regel).map_err(|e| e.to_string())?;
    let framed = serde_json::json!({ "profil": "x", "quelle": "x", "bestaetigt": false, "regeln": [regel] });
    let framed = serde_norway::to_string(&framed).map_err(|e| e.to_string())?;
    crate::pruefung::profil::parse(&framed).map(|_| yaml).map_err(|e| e.replace(&format!("{}: ", crate::pruefung::profil::PROFILE_FILE), ""))
}

fn json_array(answer: &str) -> Result<Vec<serde_json::Value>, String> {
    let start = answer.find('[').ok_or("Antwort enthält kein JSON-Array")?;
    let end = answer.rfind(']').ok_or("Antwort enthält kein JSON-Array")?;
    serde_json::from_str(&answer[start..=end]).map_err(|e| format!("JSON der KI nicht lesbar: {e}"))
}

pub async fn regelprofil_aus_pdf(root: &Path, settings: &KiSettings, pdf: &Path) -> Result<(Vec<Vorschlag>, String), String> {
    let hash = extrakt::file_hash(pdf)?;
    let key = format!("vorgaben-{}", &hash[7..19]);
    let extrakt = match extrakt::cached(root, &key, &hash) {
        Some(e) => e,
        None => extrakt::extract(root, &key, pdf, &hash)?,
    };
    let mut message = String::from("Dokument mit Vorgaben, Abschnitte mit gedruckter Seite:\n");
    let mut used = 0;
    for a in &extrakt.abschnitte {
        used += a.text.len();
        if used > 160_000 {
            message.push_str("[… gekürzt]\n");
            break;
        }
        message.push_str(&format!("[S. {}] {}\n", a.label, a.text));
    }
    let system = format!("{}\n\n{}", super::BASIS_PROMPT, REGELPROFIL_PROMPT.replace("{schema}", &schema_text()));
    let kontext = Kontext { datei: Some(pdf.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()), absatz: None };
    let answer = super::anfrage(root, settings, Aufgabe::Regelprofil, &system, &message, kontext).await?;
    let mut out = Vec::new();
    let mut ids = BTreeSet::new();
    for mut value in json_array(&answer.text)? {
        let zitat = value.get("zitat").and_then(|z| z.as_str()).unwrap_or_default().to_owned();
        if let Some(obj) = value.as_object_mut() {
            obj.remove("zitat");
        }
        let id = value.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_owned();
        let fundstelle = value.get("fundstelle").and_then(|f| f.as_str()).unwrap_or_default().to_owned();
        let (yaml, mut problem) = match check_rule(&value) {
            Ok(yaml) => (yaml, None),
            Err(e) => (serde_norway::to_string(&value).unwrap_or_default(), Some(e)),
        };
        if !ids.insert(id.clone()) {
            problem = Some(format!("id „{id}“ doppelt"));
        }
        let page = PAGE.captures(&fundstelle).map(|c| c[1].to_owned());
        let zitat_check = match (&page, zitat.is_empty()) {
            (_, true) => Err("kein Zitat".to_owned()),
            (None, _) => Err("Fundstelle ohne Seite".to_owned()),
            (Some(page), _) => {
                let stelle = crate::ingest::stellen::check(&extrakt, page, &zitat, "");
                stelle.pdf_seite.map(|_| ()).ok_or(stelle.problem.unwrap_or_default())
            }
        };
        let zitat_ok = zitat_check.is_ok();
        if let Err(e) = zitat_check
            && problem.is_none()
        {
            problem = Some(format!("Zitat: {e}"));
        }
        out.push(Vorschlag { regel: value, yaml, zitat, zitat_ok, problem });
    }
    Ok((out, answer.modell))
}

/// Bestätigte Regeln als Profil speichern (schreibt die App, nicht die KI; nur auf Klick).
pub fn regelprofil_speichern(root: &Path, name: &str, quelle: &str, regeln: Vec<serde_json::Value>, ziel: &str) -> Result<String, String> {
    if !matches!(ziel, "System/regelprofil-aus-pdf.yaml" | "System/regelprofil.yaml") {
        return Err("Ziel nicht erlaubt".into());
    }
    let profil = serde_json::json!({ "profil": name, "quelle": quelle, "bestaetigt": true, "regeln": regeln });
    let yaml = format!(
        "# Aus einem Vorgaben-PDF per KI erzeugt und Regel für Regel bestätigt ({}).\n{}",
        crate::projekt::today(),
        serde_norway::to_string(&profil).map_err(|e| e.to_string())?
    );
    crate::pruefung::profil::parse(&yaml)?;
    let target = root.join(ziel);
    if ziel == "System/regelprofil.yaml" && target.is_file() {
        std::fs::copy(&target, root.join("System/regelprofil.vorher.yaml")).map_err(|e| e.to_string())?;
    }
    crate::projekt::files::atomic_write(&target, &yaml)?;
    Ok(ziel.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bib() -> BTreeSet<String> {
        ["dragoni2016", "fritzsch2019", "mnih2013"].into_iter().map(String::from).collect()
    }

    const TEST_CHAPTER: &str = include_str!("../../../tests/ki-pruefung/07-ki-test.typ");

    #[test]
    fn sentences_with_citations_and_without_comments() {
        let saetze = sentences(TEST_CHAPTER, (0, TEST_CHAPTER.len()), &bib());
        let texts: Vec<&str> = saetze.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(saetze.len(), 10, "{texts:#?}");
        assert!(texts.iter().all(|t| !t.contains("TESTFALL")), "Kommentare gehen nicht an die KI");
        let monolith = saetze.iter().find(|s| s.text.starts_with("Ein Monolith")).unwrap();
        assert_eq!(monolith.belege, [("dragoni2016".to_owned(), Some("S. 1".to_owned()))]);
        let u1 = saetze.iter().find(|s| s.text.contains("63 %")).unwrap();
        assert!(u1.belege.is_empty());
        assert_eq!(&TEST_CHAPTER[u1.start..u1.start + 5], "Laut ");
        // „z. B.“ beendet keinen Satz; Labels wie @kap:x sind keine Belege.
        let s = sentences("Das gilt z. B. hier. Siehe @kap:x und @mnih2013.\n", (0, 50), &bib());
        assert_eq!(s.len(), 2);
        assert_eq!(s[1].belege, [("mnih2013".to_owned(), None)]);
    }

    #[test]
    fn suggestions_and_invented_quotes_are_removed() {
        let satz = "Die meisten Unternehmen zerlegen dabei ihre bestehende Codebasis schrittweise.";
        let answer = "- Satz 3: Zahl ohne Beleg\n\
            - Vorschlag: Schreibe lieber etwas anderes.\n\
            - Der Satz „Die meisten Unternehmen zerlegen dabei ihre bestehende Codebasis“ braucht eine Quelle.\n\
            - Besser wäre „Viele Firmen schreiben ihre Systeme komplett neu und nutzen moderne Technik“.";
        let (clean, removed) = ohne_formulierungen(answer, &[satz]);
        assert_eq!(removed, 2, "{clean}");
        assert!(clean.contains("[Formulierungsvorschlag entfernt]"));
        assert!(clean.contains("„Die meisten Unternehmen zerlegen dabei ihre bestehende Codebasis“"), "Zitat aus der Arbeit bleibt");
        assert!(clean.contains("Besser wäre [Formulierung entfernt]"));
        assert!(clean.starts_with("- Satz 3: Zahl ohne Beleg"));
    }

    #[test]
    fn numbered_answers_are_parsed() {
        let text = "= T\n\nA eins.\n\nB zwei.\n";
        let hints = absatz_hinweise("- Absatz 2: Was ist gemeint?\n- [A3] Fehlt hier etwas?\n- allgemein\n- keine", text, false);
        assert_eq!(hints.len(), 3);
        assert_eq!((hints[0].absatz, hints[0].start), (Some(2), Some(5)));
        assert_eq!(hints[1].absatz, Some(3));
        assert_eq!(hints[2].absatz, None);
        let rules = absatz_hinweise("- [abbildung-beschrieben] Absatz 2: Wird die Abbildung erklärt?", text, true);
        assert_eq!(rules[0].regel.as_deref(), Some("abbildung-beschrieben"));
        assert_eq!(rules[0].text, "Wird die Abbildung erklärt?");
    }

    #[test]
    fn page_from_supplement() {
        assert_eq!(page_from("S. 14").as_deref(), Some("14"));
        assert_eq!(page_from("S. 14–15").as_deref(), Some("14"));
        assert_eq!(page_from("S. iv").as_deref(), Some("iv"));
    }

    #[test]
    fn rules_are_checked_against_the_schema() {
        let ok = serde_json::json!({"id": "umfang", "klasse": "maschinell", "art": "seitenumfang", "bereich": "inhaltsteil", "min": 20, "max": 22, "fundstelle": "S. 3"});
        assert!(check_rule(&ok).is_ok(), "{:?}", check_rule(&ok));
        let bad = serde_json::json!({"id": "x", "klasse": "maschinell", "art": "schoenheit", "fundstelle": "S. 1"});
        assert!(check_rule(&bad).is_err());
        assert_eq!(json_array("Hier:\n```json\n[{\"a\": 1}]\n```").unwrap().len(), 1);
    }

    #[test]
    fn saving_validates_and_backs_up() {
        let root = std::env::temp_dir().join(format!("sa-regelprofil-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("System")).unwrap();
        std::fs::write(root.join("System/regelprofil.yaml"), "alt").unwrap();
        let regel = serde_json::json!({"id": "firmenwebsites", "klasse": "urteil", "text": "Firmenwebsites meiden", "fundstelle": "S. 11"});
        let path = regelprofil_speichern(&root, "FHDW", "PDF", vec![regel.clone()], "System/regelprofil.yaml").unwrap();
        assert_eq!(path, "System/regelprofil.yaml");
        assert_eq!(std::fs::read_to_string(root.join("System/regelprofil.vorher.yaml")).unwrap(), "alt");
        assert!(crate::pruefung::profil::load(&root).unwrap().unwrap().bestaetigt);
        assert!(regelprofil_speichern(&root, "x", "y", vec![regel], "Arbeit/main.typ").is_err());
        let invalid = serde_json::json!({"id": "x", "klasse": "maschinell", "fundstelle": "S. 1"});
        assert!(regelprofil_speichern(&root, "x", "y", vec![invalid], "System/regelprofil-aus-pdf.yaml").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
