//! KI-Protokoll `System/ki-protokoll.jsonl`: jede KI-Anfrage (und jeder abgelehnte
//! Schreibversuch) als eine JSON-Zeile, nur anhängen (Leitplanke 3). Jede Zeile trägt den
//! Hash der vorherigen (`vorheriger_hash`), damit Änderungen und Lücken auffallen.
//! Export als Typst-Tabelle für den Anhang (nur auf Klick, siehe `export_typst`).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};

pub const NAME: &str = "protokoll";
pub const FILE: &str = "System/ki-protokoll.jsonl";
/// Die einzige Datei in `Arbeit/`, die die App erzeugt – und nur auf Klick.
pub const EXPORT_FILE: &str = "Arbeit/anhang/ki-nutzung.typ";

/// Schreibt nacheinander: Protokolleinträge innerhalb der App nie gleichzeitig anhängen.
static APPEND: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kontext {
    pub datei: Option<String>,
    /// Absatz in der Datei, ab 1 (durch Leerzeilen getrennt).
    pub absatz: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tokens {
    pub ein: Option<u32>,
    pub aus: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eintrag {
    pub id: String,
    pub zeit: String,
    /// Leer bei Einträgen ohne KI-Aufruf (abgelehnter Schreibversuch).
    pub anbieter: String,
    pub modell: String,
    /// frage, belegpruefung, feedback, ingest, regelprofil oder `schreiben`.
    pub aufgabe: String,
    pub kontext: Kontext,
    pub anfrage: String,
    pub antwort: String,
    pub tokens: Tokens,
    /// `ok`, `fehler` (Anbieter nicht erreichbar o. Ä.) oder `abgelehnt` (Schreibversuch).
    pub ergebnis: String,
    pub vorheriger_hash: String,
    pub hash: String,
}

/// Neuer Eintrag (ohne id, zeit und Hashes – die setzt `append`).
pub fn entry(anbieter: &str, modell: &str, aufgabe: &str, kontext: Kontext, anfrage: &str) -> Eintrag {
    Eintrag {
        anbieter: anbieter.to_owned(),
        modell: modell.to_owned(),
        aufgabe: aufgabe.to_owned(),
        kontext,
        anfrage: anfrage.to_owned(),
        ..Default::default()
    }
}

/// SHA-256 über die Zeile ohne das Feld `hash` (feste Feldreihenfolge der Struktur).
fn hash_of(eintrag: &Eintrag) -> String {
    let mut copy = eintrag.clone();
    copy.hash.clear();
    let json = serde_json::to_string(&copy).expect("Eintrag ist serialisierbar");
    format!("sha256:{:x}", Sha256::digest(json.as_bytes()))
}

fn file(root: &Path) -> PathBuf {
    root.join(FILE)
}

/// Letzte nicht leere Zeile.
fn last_line(text: &str) -> Option<&str> {
    text.lines().rev().find(|l| !l.trim().is_empty())
}

/// Eintrag anhängen und auf die Platte bringen (fsync). Erst danach darf die Antwort
/// angezeigt werden. Gibt den geschriebenen Eintrag zurück.
pub fn append(root: &Path, mut eintrag: Eintrag) -> Result<Eintrag, String> {
    let _guard = APPEND.lock().unwrap_or_else(|e| e.into_inner());
    let path = file(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let existing = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("KI-Protokoll nicht lesbar: {e}")),
    };
    eintrag.vorheriger_hash = match last_line(&existing) {
        Some(line) => serde_json::from_str::<Eintrag>(line)
            .map_err(|e| format!("KI-Protokoll beschädigt (letzte Zeile): {e}"))?
            .hash,
        None => String::new(),
    };
    eintrag.zeit = now();
    eintrag.id = format!("{}-{}", eintrag.zeit, existing.lines().filter(|l| !l.trim().is_empty()).count() + 1);
    eintrag.hash = hash_of(&eintrag);
    let mut line = serde_json::to_string(&eintrag).map_err(|e| e.to_string())?;
    line.push('\n');
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("KI-Protokoll nicht schreibbar: {e}"))?;
    // Fehlte am Ende der Zeilenumbruch, nicht an die letzte Zeile ankleben.
    if !existing.is_empty() && !existing.ends_with('\n') {
        line.insert(0, '\n');
    }
    out.write_all(line.as_bytes()).and_then(|_| out.sync_all()).map_err(|e| format!("KI-Protokoll nicht schreibbar: {e}"))?;
    Ok(eintrag)
}

/// Ortszeit mit Versatz, z. B. `2026-11-02T10:15:00+01:00`.
fn now() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ChainCheck {
    pub eintraege: usize,
    /// Erste fehlerhafte Zeile (ab 1) mit Grund; `None` = Kette in Ordnung.
    pub fehler: Option<(usize, String)>,
}

/// Hash-Kette prüfen: jede Zeile lesbar, Hash stimmt, `vorheriger_hash` = Hash der Zeile davor.
pub fn verify(root: &Path) -> Result<ChainCheck, String> {
    let text = match std::fs::read_to_string(file(root)) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ChainCheck { eintraege: 0, fehler: None }),
        Err(e) => return Err(e.to_string()),
    };
    Ok(verify_text(&text))
}

fn verify_text(text: &str) -> ChainCheck {
    let mut previous = String::new();
    let mut count = 0;
    for (i, line) in text.lines().enumerate() {
        let nr = i + 1;
        if line.trim().is_empty() {
            return ChainCheck { eintraege: count, fehler: Some((nr, "leere Zeile".into())) };
        }
        let eintrag: Eintrag = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(e) => return ChainCheck { eintraege: count, fehler: Some((nr, format!("nicht lesbar: {e}"))) },
        };
        if eintrag.vorheriger_hash != previous {
            return ChainCheck { eintraege: count, fehler: Some((nr, "Kette unterbrochen (Eintrag davor fehlt oder wurde geändert)".into())) };
        }
        if hash_of(&eintrag) != eintrag.hash {
            return ChainCheck { eintraege: count, fehler: Some((nr, "Inhalt geändert (Hash stimmt nicht)".into())) };
        }
        previous = eintrag.hash;
        count += 1;
    }
    ChainCheck { eintraege: count, fehler: None }
}

pub fn read_all(root: &Path) -> Result<Vec<Eintrag>, String> {
    let text = match std::fs::read_to_string(file(root)) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, l)| serde_json::from_str(l).map_err(|e| format!("Zeile {}: {e}", i + 1)))
        .collect()
}

/// Zweck einer Aufgabe für den Anhang.
fn purpose(aufgabe: &str) -> &str {
    match aufgabe {
        "frage" => "Fragen zum eigenen Text",
        "belegpruefung" => "Prüfung von Belegen gegen die Quelle",
        "feedback" => "Rückmeldung zu Aufbau und rotem Faden",
        "ingest" => "Erschließung von Quellen (Wegweiser)",
        "regelprofil" => "Auslesen der Formvorgaben",
        other => other,
    }
}

fn typst_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Anzeigename des Anbieters im Anhang.
fn provider_name(anbieter: &str) -> &str {
    match anbieter {
        "ollama" => "Ollama, lokal",
        "openrouter" => "OpenRouter",
        "deepseek" => "DeepSeek",
        "anthropic" => "Anthropic",
        "openai" => "OpenAI-kompatibel",
        other => other,
    }
}

/// Typst-Datei mit `ai-usage` aus der Vorlage (Datum | Werkzeug/Modell | Zweck | Verwendung):
/// erfolgreiche KI-Anfragen, zusammengefasst je Tag, Modell und Zweck (mit Anzahl).
/// Einbinden im Anhang mit `#include "anhang/ki-nutzung.typ"`.
pub fn export_typst(eintraege: &[Eintrag]) -> String {
    let mut rows: Vec<(String, String, String, usize)> = Vec::new();
    for e in eintraege.iter().filter(|e| e.ergebnis == "ok" && !e.anbieter.is_empty()) {
        let datum = e.zeit.get(..10).unwrap_or(&e.zeit).to_owned();
        let werkzeug = format!("{} ({})", e.modell, provider_name(&e.anbieter));
        let zweck = purpose(&e.aufgabe).to_owned();
        match rows.iter_mut().find(|r| r.0 == datum && r.1 == werkzeug && r.2 == zweck) {
            Some(row) => row.3 += 1,
            None => rows.push((datum, werkzeug, zweck, 1)),
        }
    }
    let mut out = String::from(
        "// Erzeugt vom Schreibassistenten aus System/ki-protokoll.jsonl.\n\
         // Nicht von Hand ändern, sondern neu exportieren.\n\
         #import \"@local/fhdw-arbeit:0.1.0\": ai-usage\n\n",
    );
    if rows.is_empty() {
        out.push_str("Es wurden keine KI-Werkzeuge genutzt.\n");
        return out;
    }
    out.push_str("#ai-usage(\n");
    for (datum, werkzeug, zweck, n) in &rows {
        let anzahl = if *n == 1 { "1 Anfrage".to_owned() } else { format!("{n} Anfragen") };
        out.push_str(&format!(
            "  ({}, {}, {}, {}),\n",
            typst_string(datum),
            typst_string(werkzeug),
            typst_string(&format!("{zweck} ({anzahl})")),
            typst_string("Hinweise gelesen, kein Text übernommen"),
        ));
    }
    out.push_str(")\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_vault(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("sa-protokoll-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("System")).unwrap();
        root
    }

    fn frage(text: &str) -> Eintrag {
        let mut e = entry("ollama", "qwen3:8b", "frage", Kontext { datei: Some("Arbeit/kapitel/01.typ".into()), absatz: Some(2) }, text);
        e.antwort = "- Hinweis".into();
        e.tokens = Tokens { ein: Some(10), aus: Some(3) };
        e.ergebnis = "ok".into();
        e
    }

    #[test]
    fn chain_is_built_and_verified() {
        let root = temp_vault("kette");
        let a = append(&root, frage("Erste")).unwrap();
        let b = append(&root, frage("Zweite")).unwrap();
        assert_eq!(a.vorheriger_hash, "");
        assert_eq!(b.vorheriger_hash, a.hash);
        assert!(a.hash.starts_with("sha256:"));
        assert_eq!(verify(&root).unwrap(), ChainCheck { eintraege: 2, fehler: None });
        let _ = std::fs::remove_dir_all(root); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn tampering_is_detected() {
        let root = temp_vault("manipuliert");
        for i in 0..3 {
            append(&root, frage(&format!("Frage {i}"))).unwrap();
        }
        let path = root.join(FILE);
        let original = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = original.lines().collect();

        // Antwort nachträglich geändert.
        std::fs::write(&path, original.replacen("Frage 1", "Frage X", 1)).unwrap();
        assert_eq!(verify(&root).unwrap().fehler.map(|f| f.0), Some(2));

        // Mittlere Zeile gelöscht.
        std::fs::write(&path, format!("{}\n{}\n", lines[0], lines[2])).unwrap();
        assert_eq!(verify(&root).unwrap().fehler.map(|f| f.0), Some(2));

        // Zeile geändert und Hash neu berechnet: die nächste Zeile passt nicht mehr.
        let mut forged: Eintrag = serde_json::from_str(lines[1]).unwrap();
        forged.antwort = "anders".into();
        forged.hash = hash_of(&forged);
        let forged = serde_json::to_string(&forged).unwrap();
        std::fs::write(&path, format!("{}\n{forged}\n{}\n", lines[0], lines[2])).unwrap();
        assert_eq!(verify(&root).unwrap().fehler.map(|f| f.0), Some(3));

        // Unbekanntes Feld eingeschmuggelt.
        let extra = lines[0].replacen('{', "{\"x\":1,", 1);
        std::fs::write(&path, format!("{extra}\n")).unwrap();
        assert_eq!(verify(&root).unwrap().fehler.map(|f| f.0), Some(1));
        let _ = std::fs::remove_dir_all(root); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn export_groups_by_day_model_and_purpose() {
        let mut a = frage("a");
        a.zeit = "2026-11-02T10:00:00+01:00".into();
        let mut b = a.clone();
        b.zeit = "2026-11-02T11:00:00+01:00".into();
        let mut c = a.clone();
        c.anbieter = "openrouter".into();
        c.modell = "anthropic/claude-sonnet-5.5".into();
        let mut failed = a.clone();
        failed.ergebnis = "fehler".into();
        let mut denied = entry("", "", "schreiben", Kontext::default(), "Arbeit/main.typ");
        denied.ergebnis = "abgelehnt".into();
        let typ = export_typst(&[a, b, c, failed, denied]);
        assert!(typ.contains("(\"2026-11-02\", \"qwen3:8b (Ollama, lokal)\", \"Fragen zum eigenen Text (2 Anfragen)\""), "{typ}");
        assert!(typ.contains("\"anthropic/claude-sonnet-5.5 (OpenRouter)\""));
        assert!(export_typst(&[]).contains("keine KI-Werkzeuge"));
        assert_eq!(typ.matches("Hinweise gelesen").count(), 2, "fehlgeschlagene und abgelehnte zählen nicht");
    }

    #[test]
    fn export_compiles_in_the_template() {
        let mut a = frage("a");
        a.zeit = "2026-11-02T10:00:00+01:00".into();
        a.modell = "modell \"mit\" Zeichen \\ #".into();
        let root = temp_vault("export");
        std::fs::write(root.join("main.typ"), export_typst(&[a])).unwrap();
        let world = crate::satz::world::SatzWorld::new(&root, "main.typ").unwrap();
        let result = typst::compile::<typst_layout::PagedDocument>(&world);
        assert!(result.output.is_ok(), "{:?}", result.output.err());
        let _ = std::fs::remove_dir_all(root); // Windows hält Dateien teils noch kurz offen
    }
}
