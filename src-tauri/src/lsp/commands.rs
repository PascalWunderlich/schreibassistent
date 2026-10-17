//! Tauri-Befehle des Moduls `lsp`: Tinymist im Feld des Seiteneditors.
//!
//! Vor Anfragen wird das Feld wie beim Speichern in die Datei geschrieben, damit Tinymist
//! den aktuellen Stand kennt (siehe Moduldoku).

use std::time::Duration;

use tauri::State;

use super::{Completion, Definition, Field, FieldDiagnostic, TinymistSession};
use crate::projekt::Project;
use crate::projekt::commands::{AppState, with_project};

/// Zeit, die Tinymist braucht, um eine geänderte Datei neu zu lesen (gemessen ~50 ms).
const RELOAD_DELAY: Duration = Duration::from_millis(100);

/// Feld schreiben und die Anfrage an Tinymist stellen.
fn ask<T>(
    state: &State<'_, AppState>,
    field: &Field,
    f: impl FnOnce(&TinymistSession, &str) -> Result<T, String>,
) -> Result<T, String> {
    let full = with_project(state, |p| write_field(p, field))?;
    std::thread::sleep(RELOAD_DELAY);
    with_project(state, |p| f(p.lsp.as_ref().ok_or("Tinymist läuft nicht")?, &full))
}

fn write_field(project: &mut Project, field: &Field) -> Result<String, String> {
    let full = field.splice(&project.satz.world().source_text(&field.file)?)?;
    let on_disk = std::fs::read_to_string(project.root.join(&field.file)).unwrap_or_default();
    if on_disk != full {
        project.write(&field.file, &full)?;
    }
    Ok(full)
}

#[tauri::command]
pub async fn lsp_completion(state: State<'_, AppState>, field: Field, cursor: usize) -> Result<Vec<Completion>, String> {
    ask(&state, &field, |lsp, full| lsp.completion(full, &field, cursor))
}

#[tauri::command]
pub async fn lsp_hover(state: State<'_, AppState>, field: Field, cursor: usize) -> Result<Option<String>, String> {
    ask(&state, &field, |lsp, full| lsp.hover(full, &field, cursor))
}

#[tauri::command]
pub async fn lsp_definition(state: State<'_, AppState>, field: Field, cursor: usize) -> Result<Option<Definition>, String> {
    ask(&state, &field, |lsp, full| lsp.definition(full, &field, cursor))
}

/// Fehler im Feld (nach `lsp_diagnostics`-Ereignis abfragen). Schreibt nichts.
#[tauri::command]
pub async fn lsp_field_diagnostics(state: State<'_, AppState>, field: Field) -> Result<Vec<FieldDiagnostic>, String> {
    with_project(&state, |p| {
        let Some(lsp) = &p.lsp else { return Ok(Vec::new()) };
        let full = field.splice(&p.satz.world().source_text(&field.file)?)?;
        lsp.field_diagnostics(&full, &field)
    })
}

#[tauri::command]
pub async fn lsp_error_count(state: State<'_, AppState>) -> Result<usize, String> {
    with_project(&state, |p| Ok(p.lsp.as_ref().map_or(0, TinymistSession::error_count)))
}
