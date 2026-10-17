//! Tauri-Befehle des Moduls `lsp` (Prototyp B).

use std::sync::Mutex;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, State};

use super::{SourceFile, TinymistSession, find_tinymist};

#[derive(Default)]
pub struct LspState(pub Mutex<Option<TinymistSession>>);

#[derive(serde::Serialize)]
pub struct PreviewSession {
    pub preview_url: String,
    pub files: Vec<SourceFile>,
}

/// Startet Tinymist mit Vorschau für `beispiel-arbeit/`. Meldungen der Vorschau gehen als
/// Ereignis `lsp_notification` ({method, params}) an die Oberfläche.
#[tauri::command]
pub async fn lsp_open_example(app: AppHandle, state: State<'_, LspState>) -> Result<PreviewSession, String> {
    let program = find_tinymist().ok_or("Tinymist nicht gefunden (brew install tinymist)")?;
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
    // Alte Sitzung zuerst beenden (Drop beendet den Prozess).
    state.0.lock().unwrap().take();
    let (session, files) = TinymistSession::start(&program, root, "Arbeit/main.typ", move |method, params| {
        // Klick in der Vorschau kommt als `window/showDocument` (Datei-URI + Auswahl).
        if matches!(method, "window/showDocument" | "tinymist/preview/scrollSource" | "tinymist/compileStatus") {
            let _ = app.emit("lsp_notification", json!({ "method": method, "params": params }));
        }
    })?;
    let preview_url = format!("http://127.0.0.1:{}", session.preview_port);
    *state.0.lock().unwrap() = Some(session);
    Ok(PreviewSession { preview_url, files })
}

#[tauri::command]
pub async fn lsp_change(state: State<'_, LspState>, path: String, text: String) -> Result<(), String> {
    with_session(&state, |s| s.change(&path, &text))
}

#[tauri::command]
pub async fn lsp_scroll_preview(
    state: State<'_, LspState>,
    path: String,
    line: u32,
    character: u32,
) -> Result<(), String> {
    with_session(&state, |s| s.scroll_preview(&path, line, character))
}

/// Absoluter Pfad oder `file://`-URI aus einer Tinymist-Meldung → Pfad relativ zum Vault.
#[tauri::command]
pub async fn lsp_relative_path(state: State<'_, LspState>, absolute: String) -> Result<Value, String> {
    let path = url::Url::parse(&absolute)
        .ok()
        .and_then(|u| u.to_file_path().ok())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(absolute);
    with_session(&state, |s| Ok(json!(s.relative(&path))))
}

fn with_session<T>(
    state: &State<'_, LspState>,
    f: impl FnOnce(&TinymistSession) -> Result<T, String>,
) -> Result<T, String> {
    let guard = state.0.lock().unwrap();
    f(guard.as_ref().ok_or("Tinymist läuft nicht")?)
}
