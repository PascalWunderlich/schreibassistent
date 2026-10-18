//! Tauri-Befehle des Moduls `quellen`.

use tauri::State;

use super::zotero::{self, ZoteroSource};
use crate::projekt::commands::AppState;

/// Zotero-Daten (Schlüssel, PDFs, Markierungen) zu einem Citekey, live abgefragt.
/// `None`, wenn Zotero nicht läuft oder die Quelle dort fehlt.
#[tauri::command]
pub async fn quellen_zotero(_state: State<'_, AppState>, citekey: String) -> Result<Option<ZoteroSource>, String> {
    // Außerhalb der Projektsperre: Die Abfrage kann bis zum Timeout dauern.
    let result = tauri::async_runtime::spawn_blocking(move || zotero::fetch().and_then(|mut all| all.remove(&citekey)))
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

/// Läuft Zotero mit eingeschalteter Local API? (Ersteinstieg „Zotero verbinden“)
#[tauri::command]
pub async fn quellen_zotero_laeuft() -> bool {
    tauri::async_runtime::spawn_blocking(|| zotero::fetch().is_some()).await.unwrap_or(false)
}

/// `.bib`-Datei als Quellen der geöffneten Arbeit übernehmen; gibt die Zahl der Quellen zurück.
#[tauri::command]
pub async fn quellen_bib_uebernehmen(state: State<'_, AppState>, path: String) -> Result<usize, String> {
    let root = crate::projekt::commands::with_project(&state, |p| Ok(p.root.clone()))?;
    super::adopt_bib(&root, std::path::Path::new(&path))
}
