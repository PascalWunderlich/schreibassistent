//! Tauri-Commands für KI und Protokoll.

use std::collections::BTreeMap;

use tauri::{AppHandle, State};

use super::einstellungen::{Anbieter, Aufgabe, KiSettings};
use super::{FrageErgebnis, KontextWahl, anbieter, schluessel};
use crate::projekt::commands::{AppState, settings_file, with_project};
use crate::projekt::settings;
use crate::protokoll;

#[derive(serde::Serialize)]
pub struct KiInfo {
    pub settings: KiSettings,
    /// Je Anbieter: Schlüssel im Schlüsselbund gesetzt? (Der Schlüssel selbst nie.)
    pub schluessel: BTreeMap<&'static str, bool>,
    pub ollama_modelle: Vec<String>,
}

#[tauri::command]
pub async fn ki_einstellungen(app: AppHandle) -> Result<KiInfo, String> {
    let settings = settings::load(&settings_file(&app)?).ki;
    let url = settings.ollama_url.clone();
    let (schluessel, ollama_modelle) = tauri::async_runtime::spawn_blocking(move || {
        let keys = Anbieter::ALL.iter().filter(|a| a.needs_key()).map(|a| (a.id(), schluessel::is_set(*a))).collect();
        (keys, anbieter::ollama_models(&url))
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(KiInfo { settings, schluessel, ollama_modelle })
}

#[tauri::command]
pub async fn ki_einstellungen_speichern(app: AppHandle, ki: KiSettings) -> Result<(), String> {
    let file = settings_file(&app)?;
    let mut all = settings::load(&file);
    all.ki = ki;
    settings::save(&file, &all)
}

#[tauri::command]
pub async fn ki_schluessel_setzen(anbieter: Anbieter, schluessel: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || schluessel::set(anbieter, &schluessel)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ki_schluessel_loeschen(anbieter: Anbieter) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || schluessel::delete(anbieter)).await.map_err(|e| e.to_string())?
}

/// Frage im Assistenten. Die Antwort kommt erst, wenn sie im Protokoll steht.
#[tauri::command]
pub async fn ki_frage(app: AppHandle, state: State<'_, AppState>, frage: String, kontext: KontextWahl) -> Result<FrageErgebnis, String> {
    let root = with_project(&state, |p| Ok(p.root.clone()))?;
    let settings = settings::load(&settings_file(&app)?).ki;
    super::frage(&root, &settings, Aufgabe::Frage, &frage, &kontext).await
}

#[tauri::command]
pub async fn protokoll_pruefen(state: State<'_, AppState>) -> Result<protokoll::ChainCheck, String> {
    let root = with_project(&state, |p| Ok(p.root.clone()))?;
    protokoll::verify(&root)
}

/// Export nach `Arbeit/anhang/ki-nutzung.typ` – schreibt die App, nicht die KI, nur auf Klick.
#[tauri::command]
pub async fn protokoll_export(state: State<'_, AppState>) -> Result<String, String> {
    with_project(&state, |p| {
        let check = protokoll::verify(&p.root)?;
        if let Some((line, reason)) = check.fehler {
            return Err(format!("Protokoll fehlerhaft (Zeile {line}: {reason}) – kein Export."));
        }
        let text = protokoll::export_typst(&protokoll::read_all(&p.root)?);
        let dir = p.root.join("Arbeit/anhang");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        p.write(protokoll::EXPORT_FILE, &text)?;
        Ok(protokoll::EXPORT_FILE.to_owned())
    })
}
