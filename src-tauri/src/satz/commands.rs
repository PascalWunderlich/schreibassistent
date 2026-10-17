//! Tauri-Befehle des Moduls `satz` (Benennung `satz_<aktion>`, siehe docs/ARCHITECTURE.md).
//! Arbeiten auf dem geöffneten Projekt (`projekt::commands::AppState`).

use tauri::State;

use super::{CompileReport, ParagraphHit};
use crate::projekt::SaveInfo;
use crate::projekt::commands::{AppState, with_project};

#[derive(serde::Serialize)]
pub struct ReplaceResult {
    pub report: CompileReport,
    pub save: SaveInfo,
}

#[tauri::command]
pub async fn satz_paragraph_at(
    state: State<'_, AppState>,
    page: usize,
    x: f64,
    y: f64,
) -> Result<Option<ParagraphHit>, String> {
    with_project(&state, |p| Ok(p.satz.paragraph_at(page, x, y)))
}

#[tauri::command]
pub async fn satz_paragraph_at_offset(
    state: State<'_, AppState>,
    file: String,
    offset: usize,
) -> Result<Option<ParagraphHit>, String> {
    with_project(&state, |p| Ok(p.satz.paragraph_at_offset(&file, offset)))
}

/// Absatz ersetzen, neu setzen und sofort speichern. Bei Fehlern im Satz bleibt die
/// Datei unverändert (Satz rollt zurück), die Meldungen gehen an die Oberfläche.
#[tauri::command]
pub async fn satz_replace(
    state: State<'_, AppState>,
    file: String,
    start: usize,
    end: usize,
    text: String,
) -> Result<ReplaceResult, Vec<String>> {
    let mut guard = state.0.lock().unwrap();
    let project = guard.as_mut().ok_or_else(|| vec!["Kein Arbeits-Vault geöffnet".to_owned()])?;
    let report = project.satz.replace(&file, start, end, &text)?;
    let save = project.write_source(&file).map_err(|e| vec![e])?;
    Ok(ReplaceResult { report, save })
}

/// Entwurf während des Tippens: Absatz in den Quelltext einsetzen und auf die Platte
/// schreiben, ohne neu zu setzen. Der Satz behält den Stand bis zur Übernahme (Enter),
/// damit `start..end` gültig bleiben.
#[tauri::command]
pub async fn satz_draft(
    state: State<'_, AppState>,
    file: String,
    start: usize,
    end: usize,
    text: String,
) -> Result<SaveInfo, String> {
    with_project(&state, |p| {
        let mut source = p.satz.world().source_text(&file)?;
        if end > source.len() || start > end || !source.is_char_boundary(start) || !source.is_char_boundary(end) {
            return Err(format!("Ungültiger Bereich {start}..{end} in {file}"));
        }
        source.replace_range(start..end, &text);
        p.write(&file, &source)
    })
}

/// Satz neu, z. B. nach Änderungen im Quelltextmodus (`projekt_write`).
#[tauri::command]
pub async fn satz_compile(state: State<'_, AppState>) -> Result<CompileReport, Vec<String>> {
    let mut guard = state.0.lock().unwrap();
    let project = guard.as_mut().ok_or_else(|| vec!["Kein Arbeits-Vault geöffnet".to_owned()])?;
    project.satz.compile()
}

/// Alle Seiten neu als SVG (nach dem Öffnen in der Oberfläche).
#[tauri::command]
pub async fn satz_all_pages(state: State<'_, AppState>) -> Result<CompileReport, Vec<String>> {
    let mut guard = state.0.lock().unwrap();
    let project = guard.as_mut().ok_or_else(|| vec!["Kein Arbeits-Vault geöffnet".to_owned()])?;
    project.satz.forget_pages();
    project.satz.compile()
}

/// Schriftdatei für das Editierfeld (roh, ohne JSON-Umweg).
#[tauri::command]
pub async fn satz_font(state: State<'_, AppState>, family: String, bold: bool) -> Result<tauri::ipc::Response, String> {
    with_project(&state, |p| {
        let data = p.satz.font_data(&family, bold).ok_or_else(|| format!("Schrift {family} fehlt"))?;
        Ok(tauri::ipc::Response::new(data))
    })
}

#[tauri::command]
pub async fn satz_first_page(state: State<'_, AppState>, file: String) -> Result<Option<usize>, String> {
    with_project(&state, |p| Ok(p.satz.first_page_of(&file)))
}
