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

// ---------- KI-Prüfungen (P8) ----------

use super::pruefen::{self, AbsatzHinweis, AussageHinweis, BelegHinweis, Ergebnis, Hierzu, Vorschlag};

/// Wurzel, Citekeys der `.bib`, Gliederung und Einleitung (erstes Kapitel).
fn project_context(state: &State<'_, AppState>) -> Result<(std::path::PathBuf, std::collections::BTreeSet<String>, Vec<String>, Option<String>), String> {
    with_project(state, |p| {
        let chapters = p.index.chapters();
        let outline = chapters
            .iter()
            .filter_map(|f| p.index.typ.get(*f))
            .flat_map(|f| f.headings.iter().filter(|h| h.level <= 2).map(|h| h.text.clone()))
            .collect();
        let intro = chapters.first().map(|s| s.to_string());
        Ok((p.root.clone(), p.citekeys(), outline, intro))
    })
}

fn ki_settings(app: &AppHandle) -> Result<KiSettings, String> {
    Ok(settings::load(&settings_file(app)?).ki)
}

#[tauri::command]
pub async fn ki_belegbedarf(app: AppHandle, state: State<'_, AppState>, datei: String, offset: Option<usize>) -> Result<Ergebnis<BelegHinweis>, String> {
    let (root, bib, _, _) = project_context(&state)?;
    pruefen::belegbedarf(&root, &ki_settings(&app)?, &datei, offset, &bib).await
}

#[tauri::command]
pub async fn ki_quelle_aussage(app: AppHandle, state: State<'_, AppState>, datei: String, offset: usize) -> Result<Ergebnis<AussageHinweis>, String> {
    let (root, bib, _, _) = project_context(&state)?;
    pruefen::quelle_aussage(&root, &ki_settings(&app)?, &datei, offset, &bib).await
}

#[tauri::command]
pub async fn ki_roter_faden(app: AppHandle, state: State<'_, AppState>, datei: String) -> Result<Ergebnis<AbsatzHinweis>, String> {
    let (root, _, outline, intro) = project_context(&state)?;
    pruefen::roter_faden(&root, &ki_settings(&app)?, &datei, &outline, intro.as_deref()).await
}

#[tauri::command]
pub async fn ki_urteilsregeln(app: AppHandle, state: State<'_, AppState>, datei: String) -> Result<Ergebnis<AbsatzHinweis>, String> {
    let (root, _, _, _) = project_context(&state)?;
    pruefen::urteilsregeln(&root, &ki_settings(&app)?, &datei).await
}

/// „Was habe ich hierzu?“ ohne KI.
#[tauri::command]
pub async fn ki_hierzu(state: State<'_, AppState>, datei: String, offset: usize) -> Result<Hierzu, String> {
    with_project(&state, |p| pruefen::hierzu(&p.root, &p.index, &p.citekeys(), &datei, offset))
}

#[derive(serde::Serialize)]
pub struct Einordnung {
    pub text: String,
    pub entfernt: usize,
    pub modell: String,
}

#[tauri::command]
pub async fn ki_hierzu_einordnen(app: AppHandle, state: State<'_, AppState>, datei: String, offset: usize) -> Result<Einordnung, String> {
    let material = with_project(&state, |p| pruefen::hierzu(&p.root, &p.index, &p.citekeys(), &datei, offset))?;
    let root = with_project(&state, |p| Ok(p.root.clone()))?;
    let (text, entfernt, modell) = pruefen::hierzu_einordnen(&root, &ki_settings(&app)?, &datei, offset, &material).await?;
    Ok(Einordnung { text, entfernt, modell })
}

#[derive(serde::Serialize)]
pub struct Vorschlaege {
    pub regeln: Vec<Vorschlag>,
    pub modell: String,
}

#[tauri::command]
pub async fn ki_regelprofil_aus_pdf(app: AppHandle, state: State<'_, AppState>, pdf: String) -> Result<Vorschlaege, String> {
    let root = with_project(&state, |p| Ok(p.root.clone()))?;
    let (regeln, modell) = pruefen::regelprofil_aus_pdf(&root, &ki_settings(&app)?, std::path::Path::new(&pdf)).await?;
    Ok(Vorschlaege { regeln, modell })
}

#[tauri::command]
pub async fn ki_regelprofil_speichern(
    state: State<'_, AppState>,
    name: String,
    quelle: String,
    regeln: Vec<serde_json::Value>,
    ziel: String,
) -> Result<String, String> {
    with_project(&state, |p| pruefen::regelprofil_speichern(&p.root, &name, &quelle, regeln, &ziel))
}
