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
