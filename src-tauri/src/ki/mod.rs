//! KI-Grundlage (P6): Anbieter (genai), Schlüssel im Schlüsselbund, Schreibrechte und
//! Fragen mit Kontext. Jede Anfrage wird protokolliert, bevor die Antwort zurückgeht
//! (Leitplanke 3); die KI schreibt nur über `rechte::ki_schreibe` (Leitplanke 1).

pub mod anbieter;
pub mod commands;
pub mod einstellungen;
pub mod rechte;
pub mod schluessel;

use std::path::Path;

use crate::protokoll::{self, Kontext, Tokens};
use einstellungen::{Aufgabe, KiSettings};

pub const NAME: &str = "ki";

/// Basis-Systemprompt vor jeder Aufgabe (Vault: „Systemprompts in der App“).
pub const BASIS_PROMPT: &str = "Du bist wissenschaftlicher Hilfsassistent für eine studentische Arbeit.
Du schreibst KEINEN Text für die Arbeit: keine Formulierungen, keine
Umformulierungen, keine Beispielsätze. Du gibst Hinweise, Fragen und Fundstellen.
Jede Aussage über eine Quelle nennt Citekey und Seite und stützt sich nur auf die
mitgegebenen Originalabschnitte. Wenn die Abschnitte nicht reichen, sag das.
Antworte auf Deutsch, knapp, als Liste.";

/// Welcher Teil der Arbeit mitgeschickt wird.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "art", rename_all = "lowercase")]
pub enum KontextWahl {
    Keiner,
    /// Absatz um `offset` (Byte) in `datei`.
    Absatz { datei: String, offset: usize },
    Kapitel { datei: String },
}

/// Absatz (durch Leerzeilen getrennt) um `offset`: Nummer ab 1 und Text. Liegt `offset`
/// zwischen Absätzen, gilt der Absatz davor.
fn paragraph_around(text: &str, offset: usize) -> (usize, &str) {
    // Blöcke aus nicht leeren Zeilen: (Anfang, Ende) in Bytes.
    let mut blocks: Vec<(usize, usize)> = Vec::new();
    let mut pos = 0;
    let mut open: Option<usize> = None;
    for line in text.split_inclusive('\n') {
        if line.trim().is_empty() {
            if let Some(start) = open.take() {
                blocks.push((start, pos));
            }
        } else if open.is_none() {
            open = Some(pos);
        }
        pos += line.len();
    }
    if let Some(start) = open {
        blocks.push((start, pos));
    }
    let index = blocks.iter().rposition(|(start, _)| *start <= offset).unwrap_or(0);
    match blocks.get(index) {
        Some((start, end)) => (index + 1, text[*start..*end].trim_end()),
        None => (1, ""),
    }
}

/// Kontext für das Protokoll und Text für die Anfrage.
fn build_context(root: &Path, wahl: &KontextWahl) -> Result<(Kontext, String), String> {
    let read = |datei: &str| {
        let path = crate::projekt::files::resolve(root, datei)?;
        std::fs::read_to_string(path).map_err(|e| format!("{datei}: {e}"))
    };
    Ok(match wahl {
        KontextWahl::Keiner => (Kontext::default(), String::new()),
        KontextWahl::Absatz { datei, offset } => {
            let text = read(datei)?;
            let (number, paragraph) = paragraph_around(&text, *offset);
            (
                Kontext { datei: Some(datei.clone()), absatz: Some(number) },
                format!("Absatz {number} aus {datei} (Typst-Quelltext):\n\n{paragraph}"),
            )
        }
        KontextWahl::Kapitel { datei } => {
            let text = read(datei)?;
            (Kontext { datei: Some(datei.clone()), absatz: None }, format!("Kapitel {datei} (Typst-Quelltext):\n\n{text}"))
        }
    })
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FrageErgebnis {
    pub text: String,
    pub anbieter: String,
    pub modell: String,
    pub tokens: Tokens,
    /// Zeile im Protokoll (ID des Eintrags).
    pub protokoll_id: String,
}

/// Frage an die KI. Erst wenn der Protokolleintrag auf der Platte ist, kommt die Antwort
/// zurück; scheitert das Protokoll, gibt es keine Antwort.
pub async fn frage(root: &Path, settings: &KiSettings, aufgabe: Aufgabe, question: &str, kontext: &KontextWahl) -> Result<FrageErgebnis, String> {
    let question = question.trim();
    if question.is_empty() {
        return Err("Leere Frage".into());
    }
    let (kontext, context_text) = build_context(root, kontext)?;
    let message = if context_text.is_empty() { question.to_owned() } else { format!("{context_text}\n\n---\n\nFrage: {question}") };
    let wahl = settings.wahl(aufgabe);
    let key = if wahl.anbieter.needs_key() { schluessel::get(wahl.anbieter)? } else { None };
    let result = anbieter::ask(&wahl, settings, key, BASIS_PROMPT, &message).await;

    let mut eintrag = protokoll::entry(wahl.anbieter.id(), wahl.modell.trim(), aufgabe.id(), kontext, &message);
    match &result {
        Ok(antwort) => {
            eintrag.antwort = antwort.text.clone();
            eintrag.tokens = Tokens { ein: antwort.tokens_ein, aus: antwort.tokens_aus };
            eintrag.ergebnis = "ok".into();
        }
        Err(e) => {
            eintrag.antwort = e.clone();
            eintrag.ergebnis = "fehler".into();
        }
    }
    let eintrag = protokoll::append(root, eintrag).map_err(|e| format!("Antwort nicht angezeigt, weil das Protokoll fehlt: {e}"))?;
    let antwort = result?;
    Ok(FrageErgebnis {
        text: antwort.text,
        anbieter: eintrag.anbieter,
        modell: eintrag.modell,
        tokens: eintrag.tokens,
        protokoll_id: eintrag.id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_around_offset() {
        let text = "= Titel\n\nErster Absatz\nnoch erster.\n\nZweiter Absatz.\n\n\nDritter.\n";
        assert_eq!(paragraph_around(text, 0), (1, "= Titel"));
        let second = text.find("noch").unwrap();
        assert_eq!(paragraph_around(text, second), (2, "Erster Absatz\nnoch erster."));
        assert_eq!(paragraph_around(text, text.find("Zweiter").unwrap()), (3, "Zweiter Absatz."));
        assert_eq!(paragraph_around(text, text.find("Dritter").unwrap()), (4, "Dritter."));
        assert_eq!(paragraph_around(text, 10_000), (4, "Dritter."));
    }

    fn vault(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("sa-ki-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("Arbeit/kapitel")).unwrap();
        std::fs::create_dir_all(root.join("System")).unwrap();
        std::fs::write(root.join("Arbeit/kapitel/01.typ"), "= Einleitung\n\nDas ist der Absatz.\n").unwrap();
        root
    }

    /// Auch eine gescheiterte Anfrage steht im Protokoll (ohne Netz: Ollama auf totem Port).
    #[test]
    fn failed_request_is_logged_before_error_is_returned() {
        let root = vault("fehler");
        let mut settings = KiSettings::default();
        settings.ollama_url = "http://127.0.0.1:9/".into();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let kontext = KontextWahl::Absatz { datei: "Arbeit/kapitel/01.typ".into(), offset: 15 };
        let err = rt.block_on(frage(&root, &settings, Aufgabe::Frage, "Ist das klar?", &kontext)).unwrap_err();
        assert!(err.starts_with("ollama"), "{err}");
        let entries = protokoll::read_all(&root).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].ergebnis, "fehler");
        assert_eq!(entries[0].kontext, Kontext { datei: Some("Arbeit/kapitel/01.typ".into()), absatz: Some(2) });
        assert!(entries[0].anfrage.contains("Das ist der Absatz.") && entries[0].anfrage.contains("Ist das klar?"));
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Ist das Protokoll nicht schreibbar, gibt es keine Antwort.
    #[test]
    fn no_answer_without_log() {
        let root = vault("ohne-protokoll");
        std::fs::remove_dir_all(root.join("System")).unwrap();
        std::fs::write(root.join("System"), "Datei statt Ordner").unwrap();
        let mut settings = KiSettings::default();
        settings.ollama_url = "http://127.0.0.1:9/".into();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let err = rt.block_on(frage(&root, &settings, Aufgabe::Frage, "x", &KontextWahl::Keiner)).unwrap_err();
        assert!(err.contains("Protokoll"), "{err}");
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Echte Frage an Ollama, falls es läuft (sonst übersprungen).
    #[test]
    fn real_question_to_ollama() {
        let settings = KiSettings::default();
        if !anbieter::ollama_models(&settings.ollama_url).contains(&settings.wahl(Aufgabe::Frage).modell) {
            eprintln!("Ollama mit {} läuft nicht – übersprungen", settings.wahl(Aufgabe::Frage).modell);
            return;
        }
        let root = vault("ollama");
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let kontext = KontextWahl::Kapitel { datei: "Arbeit/kapitel/01.typ".into() };
        let result = rt.block_on(frage(&root, &settings, Aufgabe::Frage, "Nenne eine Frage, die der Absatz offenlässt.", &kontext)).unwrap();
        assert!(!result.text.is_empty() && !result.text.contains("<think>"));
        let entries = protokoll::read_all(&root).unwrap();
        assert_eq!((entries.len(), entries[0].ergebnis.as_str(), entries[0].antwort.as_str()), (1, "ok", result.text.as_str()));
        assert!(protokoll::verify(&root).unwrap().fehler.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
