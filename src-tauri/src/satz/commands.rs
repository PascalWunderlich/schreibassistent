//! Tauri-Befehle des Moduls `satz` (Benennung `satz_<aktion>`, siehe docs/ARCHITECTURE.md).

use std::sync::Mutex;

use tauri::State;

use super::{CompileReport, ParagraphHit, Satz};

#[derive(Default)]
pub struct SatzState(pub Mutex<Option<Satz>>);

/// P2-Prototyp: öffnet `beispiel-arbeit/` aus dem Repo und kompiliert alle Seiten.
#[tauri::command]
pub async fn satz_open_example(state: State<'_, SatzState>) -> Result<CompileReport, Vec<String>> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit");
    let mut satz = Satz::open(root, "Arbeit/main.typ").map_err(|e| vec![e])?;
    let report = satz.compile()?;
    *state.0.lock().unwrap() = Some(satz);
    Ok(report)
}

#[tauri::command]
pub async fn satz_paragraph_at(
    state: State<'_, SatzState>,
    page: usize,
    x: f64,
    y: f64,
) -> Result<Option<ParagraphHit>, Vec<String>> {
    let guard = state.0.lock().unwrap();
    let satz = guard.as_ref().ok_or_else(|| vec!["Keine Arbeit geöffnet".to_owned()])?;
    Ok(satz.paragraph_at(page, x, y))
}

#[tauri::command]
pub async fn satz_replace(
    state: State<'_, SatzState>,
    file: String,
    start: usize,
    end: usize,
    text: String,
) -> Result<CompileReport, Vec<String>> {
    let mut guard = state.0.lock().unwrap();
    let satz = guard.as_mut().ok_or_else(|| vec!["Keine Arbeit geöffnet".to_owned()])?;
    satz.replace(&file, start, end, &text)
}
