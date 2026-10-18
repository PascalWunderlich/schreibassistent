//! Ingest (P7): PDF aus Zotero → Abschnitte mit Seite (Docling) → Volltextindex →
//! Wegweiser-Seite `Wissen/Quellen/<citekey>.md` per KI, geprüft gegen das PDF.
//!
//! Kosten entstehen nur bei `create_page` und `create_concepts`, und nur auf Klick. Eine
//! Quellenseite gilt als aktuell, solange PDF (`quelle_hash`) und Zotero-Markierungen
//! (`markierungen_hash`) gleich sind; dann wird nichts neu erzeugt.

pub mod commands;
pub mod docling;
pub mod extrakt;
pub mod stellen;
pub mod suche;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::ki::einstellungen::{Aufgabe, KiSettings};
use crate::ki::rechte::ki_schreibe;
use crate::protokoll::Kontext;
use crate::quellen::zotero::{self, Annotation};
use extrakt::Extrakt;

pub const NAME: &str = "ingest";
pub const SOURCE_DIR: &str = "Wissen/Quellen";
pub const CONCEPT_DIR: &str = "Wissen/Konzepte";
pub const INDEX_FILE: &str = "Wissen/index.md";
pub const LOG_FILE: &str = "Wissen/log.md";
/// Höchstens so viele Tokens Quelltext je Quelle an die KI (Freigabe Pascal, P7).
pub const BUDGET_TOKENS: usize = 30_000;
/// Erwartete Länge der Antwort (für die Schätzung).
const ANSWER_TOKENS: usize = 1_500;

/// Systemprompt „Quellenseite“ (Vault: „Systemprompts in der App“), dazu das genaue Format.
const QUELLENSEITE_PROMPT: &str = "Erzeuge eine Wegweiser-Seite für {citekey} aus den folgenden Abschnitten (mit Seite).
Format genau nach Vorlage: Worum es geht (2–3 Sätze) · Interessante Stellen
(„S. n: „Zitat max. 25 Wörter“ – warum interessant“) · Bezüge · Widersprüche.
Zitate wörtlich aus den Abschnitten, keine Paraphrase als Zitat.
Nur Stellen, die zu diesen Themen der Arbeit passen: {gliederung}.

Antworte nur mit diesen vier Abschnitten in Markdown, ohne Einleitung:
## Worum es geht
2–3 Sätze.
## Interessante Stellen
- S. <Seite genau wie im Abschnitt [S. …]>: „<wörtliches Zitat, höchstens 25 Wörter, in der Sprache der Quelle>“ – <warum interessant für die Arbeit>
## Bezüge
- [[<Konzept>]] oder Kapitel der Arbeit, je eine Zeile
## Widersprüche
- zu anderen Citekeys mit Seite, oder „keine erkennbar“";

/// Eigener Systemprompt für Konzeptseiten (im Vault nicht vorgegeben, siehe DECISIONS).
const KONZEPT_PROMPT: &str = "Fasse aus den folgenden Wegweiser-Seiten 3 bis 8 Konzepte zusammen, die für die Arbeit
wichtig sind und möglichst mehrere Quellen verbinden. Nutze nur, was auf den Seiten steht.
Format je Konzept genau so:
=== Konzept: <Name, 1–4 Wörter> ===
## Überblick
2–3 Sätze, nur aus den Seiten.
## Quellen
- [[<citekey>]] S. <Seite, wie auf der Wegweiser-Seite>: <worum es dort geht>
## Widersprüche
- zwischen Quellen mit Citekey und Seite, oder „keine erkennbar“";

#[derive(Debug, Clone)]
pub struct Pdf {
    pub key: String,
    pub title: String,
    pub path: PathBuf,
    pub annotations: Vec<Annotation>,
}

fn sha(text: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(text.as_bytes()))
}

/// Fingerabdruck der Zotero-Markierungen; ändert er sich, ist die Quellenseite veraltet.
pub fn markierungen_hash(annotations: &[Annotation]) -> String {
    if annotations.is_empty() {
        return "keine".into();
    }
    sha(&serde_json::to_string(annotations).unwrap_or_default())
}

fn check_citekey(citekey: &str) -> Result<(), String> {
    if citekey.is_empty() || !citekey.chars().all(|c| c.is_ascii_alphanumeric() || "_-:.".contains(c)) || citekey.starts_with('.') {
        return Err(format!("Citekey „{citekey}“ eignet sich nicht als Dateiname"));
    }
    Ok(())
}

pub fn page_path(citekey: &str) -> String {
    format!("{SOURCE_DIR}/{citekey}.md")
}

/// PDF einer Quelle aus Zotero (erstes PDF mit Datei). Fehler, wenn Zotero nicht läuft.
pub fn find_pdf(citekey: &str) -> Result<Option<Pdf>, String> {
    let all = zotero::fetch().ok_or("Zotero läuft nicht (oder die Local API ist aus)")?;
    let Some(source) = all.get(citekey) else { return Ok(None) };
    Ok(source.pdfs.iter().find_map(|a| {
        a.path().map(|path| Pdf { key: a.key.clone(), title: a.title.clone(), path, annotations: a.annotations.clone() })
    }))
}

/// Felder aus dem Frontmatter einer Markdown-Datei (nur `schlüssel: wert`-Zeilen).
fn frontmatter(text: &str, key: &str) -> Option<String> {
    let body = text.strip_prefix("---\n")?;
    let end = body.find("\n---")?;
    body[..end].lines().find_map(|l| l.strip_prefix(&format!("{key}:")).map(|v| v.trim().to_owned()))
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Status {
    /// Titel des PDF-Anhangs in Zotero.
    pub pdf: Option<String>,
    pub zotero: bool,
    /// Abschnitte im Cache (Docling schon gelaufen).
    pub eingelesen: bool,
    /// Pfad der Quellenseite, falls vorhanden.
    pub seite: Option<String>,
    /// Quellenseite passt zu PDF und Markierungen.
    pub aktuell: bool,
    /// Warum nicht aktuell (geändertes PDF, neue Markierungen).
    pub grund: Option<String>,
    pub stellen: Option<String>,
}

pub fn status(root: &Path, citekey: &str) -> Result<Status, String> {
    check_citekey(citekey)?;
    let page_file = root.join(page_path(citekey));
    let page = std::fs::read_to_string(&page_file).ok();
    let mut status = Status {
        seite: page.as_ref().map(|_| page_path(citekey)),
        stellen: page.as_deref().and_then(|p| frontmatter(p, "stellen_bestaetigt")),
        ..Default::default()
    };
    let pdf = match find_pdf(citekey) {
        Ok(pdf) => {
            status.zotero = true;
            pdf
        }
        Err(_) => None,
    };
    let Some(pdf) = pdf else {
        // Ohne Zotero: vorhandene Seite gilt als aktuell (nichts zu vergleichen).
        status.aktuell = page.is_some();
        return Ok(status);
    };
    status.pdf = Some(pdf.title.clone());
    let quelle_hash = extrakt::file_hash(&pdf.path)?;
    status.eingelesen = extrakt::cached(root, citekey, &quelle_hash).is_some();
    if let Some(page) = &page {
        let same_pdf = frontmatter(page, "quelle_hash").as_deref() == Some(quelle_hash.as_str());
        let same_marks = frontmatter(page, "markierungen_hash").as_deref() == Some(markierungen_hash(&pdf.annotations).as_str());
        status.aktuell = same_pdf && same_marks;
        status.grund = match (same_pdf, same_marks) {
            (false, _) => Some("PDF hat sich geändert".into()),
            (true, false) => Some("neue oder geänderte Markierungen in Zotero".into()),
            _ => None,
        };
    }
    Ok(status)
}

/// Suchwörter für die Auswahl: Gliederung, Titel der Quelle, markierte Stellen.
fn terms(outline: &[String], title: &str, annotations: &[Annotation]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let texts = outline
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(title))
        .chain(annotations.iter().filter_map(|a| a.text.as_deref()));
    let mut out = Vec::new();
    for text in texts {
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            let word = word.to_lowercase();
            if word.chars().count() >= 4 && seen.insert(word.clone()) {
                out.push(word);
            }
        }
    }
    out.truncate(80);
    out
}

/// Abschnitte für die KI: alles, wenn es ins Budget passt; sonst zuerst die besten Treffer
/// der Volltextsuche (bis 60 % des Budgets), dann je Überschrift der erste Abschnitt, damit
/// die ganze Quelle vertreten ist (englische Quellen treffen deutsche Gliederungswörter selten).
fn select(extrakt: &Extrakt, ranked: &[usize], budget: usize) -> Vec<usize> {
    let cost = |i: usize| extrakt.abschnitte[i].text.len() / 4 + 4;
    if extrakt.tokens() <= budget {
        return (0..extrakt.abschnitte.len()).collect();
    }
    let mut chosen = BTreeSet::new();
    let mut used = 0;
    for &i in ranked {
        if i < extrakt.abschnitte.len() && used + cost(i) <= budget * 6 / 10 && chosen.insert(i) {
            used += cost(i);
        }
    }
    let mut last_heading = None;
    for (i, a) in extrakt.abschnitte.iter().enumerate() {
        if last_heading == Some(&a.ueberschrift) {
            continue;
        }
        last_heading = Some(&a.ueberschrift);
        if used + cost(i) <= budget && chosen.insert(i) {
            used += cost(i);
        }
    }
    for (i, _) in extrakt.abschnitte.iter().enumerate() {
        if used + cost(i) > budget {
            break;
        }
        if chosen.insert(i) {
            used += cost(i);
        }
    }
    chosen.into_iter().collect()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Schaetzung {
    pub citekey: String,
    pub seiten: u32,
    pub abschnitte: usize,
    pub abschnitte_gewaehlt: usize,
    pub tokens_ein: usize,
    pub tokens_aus: usize,
    pub anbieter: String,
    pub modell: String,
    /// `gedruckt`, `pdf` oder `pdf-seite`.
    pub seitenzahlen: String,
    pub markierungen: usize,
}

/// Alles für eine Quellenseite außer dem KI-Aufruf (kostenlos; Docling nur beim ersten Mal).
pub struct Prepared {
    pub citekey: String,
    pub pdf: Pdf,
    pub quelle_hash: String,
    pub extrakt: Extrakt,
    pub system: String,
    pub message: String,
    pub schaetzung: Schaetzung,
}

fn other_summaries(root: &Path, citekey: &str) -> Vec<(String, String)> {
    let Ok(dir) = std::fs::read_dir(root.join(SOURCE_DIR)) else { return Vec::new() };
    let mut out: Vec<(String, String)> = dir
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().strip_suffix(".md")?.to_owned();
            if name == citekey {
                return None;
            }
            let text = std::fs::read_to_string(e.path()).ok()?;
            Some((name, section(&text, "Worum es geht")?.replace('\n', " ")))
        })
        .collect();
    out.sort();
    out
}

/// Inhalt eines `## Abschnitts` (bis zum nächsten `## `).
fn section(text: &str, heading: &str) -> Option<String> {
    let start = text.find(&format!("## {heading}"))?;
    let rest = &text[start..];
    let body = rest.split_once('\n').map_or("", |x| x.1);
    let end = body.find("\n## ").unwrap_or(body.len());
    Some(body[..end].trim().to_owned()).filter(|s| !s.is_empty())
}

pub fn prepare(root: &Path, citekey: &str, title: &str, outline: &[String], settings: &KiSettings, budget: usize) -> Result<Prepared, String> {
    check_citekey(citekey)?;
    let pdf = find_pdf(citekey)?.ok_or_else(|| format!("Kein PDF zu {citekey} in Zotero"))?;
    let quelle_hash = extrakt::file_hash(&pdf.path)?;
    let extrakt = match extrakt::cached(root, citekey, &quelle_hash) {
        Some(e) => e,
        None => extrakt::extract(root, citekey, &pdf.path, &quelle_hash)?,
    };
    let mut suche = suche::Suche::open(root)?;
    if suche.count(citekey)? != extrakt.abschnitte.len() {
        suche.replace(citekey, &extrakt)?;
    }
    let ranked: Vec<usize> =
        suche.search(Some(citekey), &terms(outline, title, &pdf.annotations), 2_000)?.into_iter().map(|t| t.nr).collect();
    let chosen = select(&extrakt, &ranked, budget);

    let gliederung = outline.join(" · ");
    let system = format!(
        "{}\n\n{}",
        crate::ki::BASIS_PROMPT,
        QUELLENSEITE_PROMPT.replace("{citekey}", citekey).replace("{gliederung}", &gliederung)
    );
    let mut message = format!("Gliederung der Arbeit:\n{}\n", outline.iter().map(|h| format!("- {h}")).collect::<Vec<_>>().join("\n"));
    if !pdf.annotations.is_empty() {
        message.push_str("\nMarkierungen des Studenten in Zotero (besonders wichtig):\n");
        for a in &pdf.annotations {
            let page = a.page_label.clone().or(a.page.map(|p| p.to_string())).unwrap_or_else(|| "?".into());
            let text = a.text.as_deref().unwrap_or("");
            let comment = a.comment.as_deref().map(|c| format!(" (Kommentar: {c})")).unwrap_or_default();
            message.push_str(&format!("- S. {page}: „{text}“{comment}\n"));
        }
    }
    let others = other_summaries(root, citekey);
    if !others.is_empty() {
        message.push_str("\nAndere Quellen der Arbeit (für Widersprüche):\n");
        for (key, summary) in &others {
            message.push_str(&format!("- {key}: {summary}\n"));
        }
    }
    let auswahl = if chosen.len() == extrakt.abschnitte.len() {
        "vollständig".to_owned()
    } else {
        format!("Auswahl: {} von {} Abschnitten", chosen.len(), extrakt.abschnitte.len())
    };
    message.push_str(&format!("\nAbschnitte aus {citekey} „{title}“ ({auswahl}):\n"));
    for &i in &chosen {
        let a = &extrakt.abschnitte[i];
        message.push_str(&format!("[S. {}] {}\n", a.label, a.text));
    }

    let wahl = settings.wahl(Aufgabe::Ingest);
    let schaetzung = Schaetzung {
        citekey: citekey.to_owned(),
        seiten: extrakt.seiten,
        abschnitte: extrakt.abschnitte.len(),
        abschnitte_gewaehlt: chosen.len(),
        tokens_ein: (system.len() + message.len()) / 4,
        tokens_aus: ANSWER_TOKENS,
        anbieter: wahl.anbieter.id().to_owned(),
        modell: wahl.modell,
        seitenzahlen: extrakt.labels_quelle.clone(),
        markierungen: pdf.annotations.len(),
    };
    Ok(Prepared { citekey: citekey.to_owned(), pdf, quelle_hash, extrakt, system, message, schaetzung })
}

/// KI-Antwort aufräumen: Codeblock, eigenes Frontmatter und `# Titel` entfernen.
fn clean_answer(answer: &str) -> String {
    let mut text = answer.trim();
    if let Some(inner) = text.strip_prefix("```markdown").or_else(|| text.strip_prefix("```md")).or_else(|| text.strip_prefix("```")) {
        text = inner.trim_end().strip_suffix("```").unwrap_or(inner).trim();
    }
    if let Some(rest) = text.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        text = rest[end + 4..].trim();
    }
    let lines: Vec<&str> = text.lines().skip_while(|l| l.starts_with("# ") || l.trim().is_empty()).collect();
    lines.join("\n")
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Ergebnis {
    pub seite: String,
    pub bestaetigt: usize,
    pub gesamt: usize,
    pub modell: String,
    pub tokens: crate::protokoll::Tokens,
}

fn now_label() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

/// Quellenseite per KI erzeugen (kostet Tokens), prüfen und schreiben.
pub async fn create_page(root: &Path, settings: &KiSettings, prepared: Prepared) -> Result<Ergebnis, String> {
    let Prepared { citekey, pdf, quelle_hash, extrakt, system, message, schaetzung } = prepared;
    let path = page_path(&citekey);
    let kontext = Kontext { datei: Some(path.clone()), absatz: None };
    let answer = crate::ki::anfrage(root, settings, Aufgabe::Ingest, &system, &message, kontext).await?;
    let (body, stellen) = stellen::verify_answer(&clean_answer(&answer.text), &extrakt, &pdf.key);
    let confirmed = stellen.iter().filter(|s| s.pdf_seite.is_some()).count();
    let seitenhinweis = match extrakt.labels_quelle.as_str() {
        "gedruckt" => "",
        "pdf" => "\n> Seitenzahlen aus den Seitenbeschriftungen des PDFs (nicht aus dem Druckbild).",
        _ => "\n> Seitenzahlen = PDF-Seiten: keine gedruckten Seitenzahlen erkannt. Vor dem Zitieren im Original prüfen.",
    };
    let auswahl = if schaetzung.abschnitte_gewaehlt == schaetzung.abschnitte {
        format!("allen {} Abschnitten", schaetzung.abschnitte)
    } else {
        format!("{} von {} Abschnitten", schaetzung.abschnitte_gewaehlt, schaetzung.abschnitte)
    };
    let today = crate::projekt::today();
    let page = format!(
        "---\ntype: quelle\nauthor: ai\ncitekey: {citekey}\nerzeugt: {today}\nmodell: {}/{}\nquelle_hash: {quelle_hash}\n\
         markierungen_hash: {}\nseitenzahlen: {}\nstellen_bestaetigt: {confirmed} von {}\n---\n\
         # {citekey}\n> Wegweiser, kein Beleg. Zitiert wird aus dem Original.\n\
         > Erzeugt aus {auswahl}. Links öffnen das PDF in Zotero; ⚠ = Stelle nicht im PDF bestätigt.{seitenhinweis}\n\n{body}\n",
        answer.anbieter,
        answer.modell,
        markierungen_hash(&pdf.annotations),
        extrakt.labels_quelle,
        stellen.len(),
    );
    ki_schreibe(root, &path, &page, "ingest")?;
    append_log(root, &format!("Quellenseite [[{citekey}]] erzeugt ({}, {confirmed} von {} Stellen bestätigt)", answer.modell, stellen.len()))?;
    write_index(root)?;
    Ok(Ergebnis { seite: path, bestaetigt: confirmed, gesamt: stellen.len(), modell: answer.modell, tokens: answer.tokens })
}

fn append_log(root: &Path, line: &str) -> Result<(), String> {
    let existing = std::fs::read_to_string(root.join(LOG_FILE))
        .unwrap_or_else(|_| "# Log\n> Von der App gepflegt: was im Wissen entstanden ist.\n\n".into());
    let text = format!("{}{}- {} · {line}\n", existing, if existing.ends_with('\n') { "" } else { "\n" }, now_label());
    ki_schreibe(root, LOG_FILE, &text, "ingest")
}

fn md_files(dir: &Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().strip_suffix(".md")?.to_owned();
            Some((name, std::fs::read_to_string(e.path()).ok()?))
        })
        .collect();
    out.sort();
    out
}

/// `Wissen/index.md` aus allen Quellen- und Konzeptseiten (von der App, nicht der KI).
pub fn write_index(root: &Path) -> Result<(), String> {
    let mut text = String::from("# Wissen\n> Von der App gepflegt. Wegweiser, kein Beleg – zitiert wird aus dem Original.\n\n## Quellen\n");
    let sources = md_files(&root.join(SOURCE_DIR));
    if sources.is_empty() {
        text.push_str("- noch keine\n");
    }
    for (name, page) in &sources {
        let summary = section(page, "Worum es geht").map(|s| s.replace('\n', " ")).unwrap_or_default();
        let first = summary.split_inclusive(". ").next().unwrap_or("").trim();
        let checked = frontmatter(page, "stellen_bestaetigt").map(|s| format!(" ({s} Stellen bestätigt)")).unwrap_or_default();
        text.push_str(&format!("- [[{name}]]{checked} – {first}\n"));
    }
    text.push_str("\n## Konzepte\n");
    let concepts = md_files(&root.join(CONCEPT_DIR));
    if concepts.is_empty() {
        text.push_str("- noch keine (Knopf „Konzepte erzeugen“)\n");
    }
    for (name, _) in &concepts {
        text.push_str(&format!("- [[{name}]]\n"));
    }
    ki_schreibe(root, INDEX_FILE, &text, "ingest")
}

fn concept_file_name(name: &str) -> Option<String> {
    let clean: String = name.chars().filter(|c| c.is_alphanumeric() || " -".contains(*c)).collect();
    let clean = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    let clean: String = clean.chars().take(60).collect();
    (!clean.is_empty()).then_some(clean)
}

/// Bestätigte Seiten einer Quellenseite („S. n“, die als Link dastehen).
fn confirmed_labels(page: &str) -> BTreeSet<String> {
    static LINK: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| regex::Regex::new(r"\[S\. ([^\]]+)\]\(zotero://").unwrap());
    LINK.captures_iter(page).map(|c| c[1].to_owned()).collect()
}

/// Verweise `[[citekey]] S. n` prüfen: Quelle und bestätigte Stelle müssen existieren.
fn verify_concept(body: &str, pages: &[(String, String)]) -> (String, usize) {
    static REF: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"^(\s*[-*]\s*)\[\[([^\]]+)\]\]\s*S\.\s*([^\s:]+)(.*)$").unwrap());
    let mut problems = 0;
    let lines = body.lines().map(|line| {
        let Some(c) = REF.captures(line) else { return line.to_owned() };
        let (key, label) = (&c[2], &c[3]);
        match pages.iter().find(|(n, _)| n == key) {
            None => {
                problems += 1;
                format!("{}⚠ [[{key}]] S. {label}{} *(keine Quellenseite zu {key})*", &c[1], &c[4])
            }
            Some((_, page)) if !confirmed_labels(page).contains(label) => {
                problems += 1;
                format!("{}⚠ [[{key}]] S. {label}{} *(Seite nicht unter den bestätigten Stellen)*", &c[1], &c[4])
            }
            Some(_) => line.to_owned(),
        }
    });
    let text = lines.collect::<Vec<_>>().join("\n");
    (text, problems)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct KonzeptErgebnis {
    pub seiten: Vec<String>,
    pub warnungen: usize,
    pub modell: String,
}

/// Konzeptseiten aus allen Quellenseiten (auf Knopfdruck, kostet Tokens).
pub async fn create_concepts(root: &Path, settings: &KiSettings, outline: &[String]) -> Result<KonzeptErgebnis, String> {
    let pages = md_files(&root.join(SOURCE_DIR));
    if pages.len() < 2 {
        return Err("Erst mindestens zwei Quellenseiten erzeugen".into());
    }
    let system = format!("{}\n\n{KONZEPT_PROMPT}", crate::ki::BASIS_PROMPT);
    let mut message = format!("Gliederung der Arbeit:\n{}\n\nWegweiser-Seiten:\n", outline.iter().map(|h| format!("- {h}")).collect::<Vec<_>>().join("\n"));
    for (name, page) in &pages {
        let body = page.split_once("\n---\n").map_or(page.as_str(), |x| x.1);
        message.push_str(&format!("\n=== Quelle {name} ===\n{body}\n"));
    }
    let kontext = Kontext { datei: Some(CONCEPT_DIR.to_owned()), absatz: None };
    let answer = crate::ki::anfrage(root, settings, Aufgabe::Ingest, &system, &message, kontext).await?;
    let today = crate::projekt::today();
    let mut written = Vec::new();
    let mut warnings = 0;
    for part in answer.text.split("=== Konzept:").skip(1) {
        let Some((name, body)) = part.split_once("===") else { continue };
        let Some(file) = concept_file_name(name.trim()) else { continue };
        let (body, problems) = verify_concept(body.trim(), &pages);
        warnings += problems;
        let quellen: BTreeSet<&str> = pages.iter().map(|(n, _)| n.as_str()).filter(|n| body.contains(&format!("[[{n}]]"))).collect();
        let page = format!(
            "---\ntype: konzept\nauthor: ai\nerzeugt: {today}\nmodell: {}/{}\ncitekeys: [{}]\n---\n# {file}\n\
             > Grober Überblick aus den Wegweiser-Seiten, kein Beleg. ⚠ = Verweis nicht bestätigt.\n\n{body}\n",
            answer.anbieter,
            answer.modell,
            quellen.into_iter().collect::<Vec<_>>().join(", "),
        );
        let path = format!("{CONCEPT_DIR}/{file}.md");
        ki_schreibe(root, &path, &page, "ingest")?;
        written.push(path);
    }
    if written.is_empty() {
        return Err("Die KI-Antwort enthielt keine Konzepte im erwarteten Format (steht im Protokoll)".into());
    }
    append_log(root, &format!("{} Konzeptseiten erzeugt ({}, {warnings} Verweise nicht bestätigt)", written.len(), answer.modell))?;
    write_index(root)?;
    Ok(KonzeptErgebnis { seiten: written, warnungen: warnings, modell: answer.modell })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_takes_all_when_small_and_respects_budget_otherwise() {
        let e = extrakt::sample();
        assert_eq!(select(&e, &[], BUDGET_TOKENS), vec![0, 1, 2, 3]);
        let mut big = e.clone();
        for i in 0..400 {
            big.abschnitte.push(extrakt::Abschnitt { seite: 4, label: "3".into(), ueberschrift: format!("H{}", i / 10), text: "x".repeat(800) });
        }
        let chosen = select(&big, &[3, 200, 201], 10_000);
        let used: usize = chosen.iter().map(|&i| big.abschnitte[i].text.len() / 4 + 4).sum();
        assert!(used <= 10_000, "{used}");
        assert!(chosen.contains(&3) && chosen.contains(&200), "beste Treffer zuerst");
        assert!(chosen.contains(&44), "erste Abschnitte späterer Überschriften");
        assert!(chosen.windows(2).all(|w| w[0] < w[1]), "in Reihenfolge der Quelle");
    }

    #[test]
    fn answer_cleanup() {
        assert_eq!(clean_answer("```markdown\n# x\n\n## Worum es geht\nA\n```"), "## Worum es geht\nA");
        assert_eq!(clean_answer("---\na: b\n---\n## Worum es geht\nA"), "## Worum es geht\nA");
    }

    #[test]
    fn frontmatter_and_sections() {
        let page = "---\ntype: quelle\nquelle_hash: sha256:ab\n---\n# k\n## Worum es geht\nEins. Zwei.\n## Interessante Stellen\n- x\n";
        assert_eq!(frontmatter(page, "quelle_hash").as_deref(), Some("sha256:ab"));
        assert_eq!(section(page, "Worum es geht").as_deref(), Some("Eins. Zwei."));
        assert_eq!(markierungen_hash(&[]), "keine");
    }

    #[test]
    fn concept_references_are_checked_against_confirmed_pages() {
        let pages = vec![("fritzsch2019".to_owned(), "- [S. 3](zotero://open-pdf/library/items/K?page=4): „x“".to_owned())];
        let body = "## Quellen\n- [[fritzsch2019]] S. 3: Daten\n- [[fritzsch2019]] S. 9: erfunden\n- [[gibtsnicht]] S. 1: y";
        let (text, problems) = verify_concept(body, &pages);
        assert_eq!(problems, 2);
        assert!(text.contains("- [[fritzsch2019]] S. 3: Daten"));
        assert!(text.contains("⚠ [[fritzsch2019]] S. 9: erfunden *(Seite nicht unter den bestätigten Stellen)*"));
        assert!(text.contains("⚠ [[gibtsnicht]] S. 1: y *(keine Quellenseite zu gibtsnicht)*"));
        assert_eq!(concept_file_name(" Daten/Zerlegung: ../x ").as_deref(), Some("DatenZerlegung x"));
    }

    #[test]
    fn citekeys_must_be_safe_file_names() {
        assert!(check_citekey("fritzsch2019").is_ok());
        assert!(check_citekey("../Arbeit/main").is_err());
        assert!(check_citekey(".x").is_err());
    }
}
