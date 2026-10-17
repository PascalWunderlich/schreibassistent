//! Tauri-Befehle des Moduls `index` (Seitenleiste: „Dieses Kapitel“, „Alle Quellen“).

use tauri::State;

use super::ChapterView;
use crate::projekt::commands::{AppState, with_project};
use crate::quellen::Source;

#[derive(serde::Serialize)]
pub struct ChapterInfo {
    pub file: String,
    pub title: Option<String>,
    /// Erste Seite (Index ab 0) im Satz, falls gesetzt.
    pub first_page: Option<usize>,
}

#[derive(serde::Serialize)]
pub struct SourceInfo {
    #[serde(flatten)]
    pub source: Source,
    /// Kapitel, die die Quelle zitieren (leer = ungenutzt).
    pub chapters: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct SourcesOverview {
    pub sources: Vec<SourceInfo>,
    /// Fehler beim Lesen der `.bib`.
    pub error: Option<String>,
}

/// Kapitel mit Titel und erster Seite (für „aktuelles Kapitel“ beim Scrollen).
#[tauri::command]
pub async fn index_chapters(state: State<'_, AppState>) -> Result<Vec<ChapterInfo>, String> {
    with_project(&state, |p| {
        Ok(p.index
            .chapters()
            .into_iter()
            .map(|file| ChapterInfo {
                file: file.to_owned(),
                title: p.index.typ[file].headings.iter().find(|h| h.level == 1).map(|h| h.text.clone()),
                first_page: p.satz.first_page_of(file),
            })
            .collect())
    })
}

#[tauri::command]
pub async fn index_chapter(state: State<'_, AppState>, file: String) -> Result<Option<ChapterView>, String> {
    with_project(&state, |p| Ok(p.index.chapter_view(&file, &p.citekeys())))
}

#[tauri::command]
pub async fn index_sources(state: State<'_, AppState>) -> Result<SourcesOverview, String> {
    with_project(&state, |p| {
        let uses = p.index.source_uses(&p.citekeys());
        let sources = p
            .sources
            .iter()
            .map(|source| SourceInfo {
                source: source.clone(),
                chapters: uses.iter().find(|u| u.citekey == source.citekey).map(|u| u.chapters.clone()).unwrap_or_default(),
            })
            .collect();
        Ok(SourcesOverview { sources, error: p.sources_error.clone() })
    })
}
