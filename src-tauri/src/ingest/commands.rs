//! Tauri-Commands für den Ingest. Lange Schritte laufen außerhalb der Projektsperre.

use tauri::{AppHandle, Emitter, State};

use super::{Ergebnis, KonzeptErgebnis, Schaetzung, Status};
use crate::projekt::commands::{AppState, settings_file, with_project};
use crate::projekt::settings;

/// Gliederung (Überschriften Ebene 1–2 aller Kapitel) und Titel der Quelle.
fn context(state: &State<'_, AppState>, citekey: &str) -> Result<(std::path::PathBuf, Vec<String>, String), String> {
    with_project(state, |p| {
        let outline = p
            .index
            .chapters()
            .into_iter()
            .filter_map(|file| p.index.typ.get(file))
            .flat_map(|f| f.headings.iter().filter(|h| h.level <= 2).map(|h| h.text.clone()))
            .collect();
        let title = p.sources.iter().find(|s| s.citekey == citekey).and_then(|s| s.title.clone()).unwrap_or_default();
        Ok((p.root.clone(), outline, title))
    })
}

fn progress(app: &AppHandle, citekey: &str, schritt: &str) {
    let _ = app.emit("ingest_fortschritt", (citekey, schritt));
}

#[tauri::command]
pub async fn ingest_status(state: State<'_, AppState>, citekey: String) -> Result<Status, String> {
    let root = with_project(&state, |p| Ok(p.root.clone()))?;
    tauri::async_runtime::spawn_blocking(move || super::status(&root, &citekey)).await.map_err(|e| e.to_string())?
}

/// PDF einlesen (Docling, kostenlos) und schätzen, was die Quellenseite kostet.
#[tauri::command]
pub async fn ingest_schaetzen(app: AppHandle, state: State<'_, AppState>, citekey: String) -> Result<Schaetzung, String> {
    let (root, outline, title) = context(&state, &citekey)?;
    let settings = settings::load(&settings_file(&app)?).ki;
    progress(&app, &citekey, "liest das PDF (Docling) …");
    let prepared =
        tauri::async_runtime::spawn_blocking(move || super::prepare(&root, &citekey, &title, &outline, &settings, super::BUDGET_TOKENS)).await.map_err(|e| e.to_string())??;
    Ok(prepared.schaetzung)
}

/// Quellenseite per KI erzeugen. Ist sie aktuell, nur mit `neu = true` (Knopf „Neu erzeugen“).
#[tauri::command]
pub async fn ingest_erzeugen(app: AppHandle, state: State<'_, AppState>, citekey: String, neu: bool) -> Result<Ergebnis, String> {
    let (root, outline, title) = context(&state, &citekey)?;
    let settings = settings::load(&settings_file(&app)?).ki;
    let (status_root, key) = (root.clone(), citekey.clone());
    let status = tauri::async_runtime::spawn_blocking(move || super::status(&status_root, &key)).await.map_err(|e| e.to_string())??;
    if status.aktuell && status.seite.is_some() && !neu {
        return Err("Die Quellenseite ist aktuell; neu erzeugen nur über „Neu erzeugen“.".into());
    }
    progress(&app, &citekey, "bereitet vor …");
    let (prep_root, key, prep_settings) = (root.clone(), citekey.clone(), settings.clone());
    let prepared = tauri::async_runtime::spawn_blocking(move || super::prepare(&prep_root, &key, &title, &outline, &prep_settings, super::BUDGET_TOKENS))
        .await
        .map_err(|e| e.to_string())??;
    progress(&app, &citekey, &format!("fragt {} …", prepared.schaetzung.modell));
    let result = super::create_page(&root, &settings, prepared).await;
    progress(&app, &citekey, "");
    result
}

#[tauri::command]
pub async fn ingest_konzepte(app: AppHandle, state: State<'_, AppState>) -> Result<KonzeptErgebnis, String> {
    let (root, outline, _) = context(&state, "")?;
    let settings = settings::load(&settings_file(&app)?).ki;
    super::create_concepts(&root, &settings, &outline).await
}
