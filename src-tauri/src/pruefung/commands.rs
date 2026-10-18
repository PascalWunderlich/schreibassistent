//! Tauri-Befehle des Moduls `pruefung`: feste Prüfungen und Vale sofort, Rechtschreibung
//! (LTeX+) getrennt, weil sie beim ersten Mal mehrere Sekunden braucht.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::State;

use super::engine::{self, Hinweis};
use super::ltex::{LtexSession, find_ltex};
use super::{profil, vale};
use crate::projekt::Project;
use crate::projekt::commands::{AppState, with_project};
use crate::satz::mapping::Rect;

/// Hinweis mit seiner Lage auf der Seite (für die Markierung am Absatz).
#[derive(serde::Serialize)]
pub struct HinweisMitStelle {
    #[serde(flatten)]
    pub hinweis: Hinweis,
    /// Seite (ab 0) und Rechteck des betroffenen Absatzes, falls gesetzt.
    pub stelle: Option<(usize, Rect)>,
}

#[derive(serde::Serialize)]
pub struct ProfilInfo {
    pub profil: String,
    pub bestaetigt: bool,
}

#[derive(serde::Serialize)]
pub struct Pruefbericht {
    pub hinweise: Vec<HinweisMitStelle>,
    pub profil: Option<ProfilInfo>,
    /// Probleme der Prüfung selbst (Profil fehlerhaft, Vale fehlt …).
    pub fehler: Vec<String>,
    /// Seitenvorgabe für die Art dieser Arbeit (Statusleiste).
    pub umfang: Option<(u32, u32)>,
    pub ms: f64,
}

fn with_stellen(project: &Project, hinweise: Vec<Hinweis>) -> Vec<HinweisMitStelle> {
    hinweise
        .into_iter()
        .map(|hinweis| {
            let stelle = match (&hinweis.datei, hinweis.offset) {
                (Some(file), Some(offset)) if file.ends_with(".typ") => {
                    project.satz.paragraph_at_offset(file, offset).and_then(|hit| hit.rects.first().copied())
                }
                _ => None,
            };
            HinweisMitStelle { hinweis, stelle }
        })
        .collect()
}

/// Feste Prüfungen und Stilregeln (Vale) auf dem aktuellen Stand.
#[tauri::command]
pub async fn pruefung_run(state: State<'_, AppState>) -> Result<Pruefbericht, String> {
    with_project(&state, |p| {
        let start = Instant::now();
        let mut fehler = Vec::new();
        let profil = match profil::load(&p.root) {
            Ok(Some(profil)) => profil,
            Ok(None) => {
                return Ok(Pruefbericht { hinweise: Vec::new(), profil: None, fehler, umfang: None, ms: 0.0 });
            }
            Err(e) => {
                fehler.push(e);
                return Ok(Pruefbericht { hinweise: Vec::new(), profil: None, fehler, umfang: None, ms: 0.0 });
            }
        };
        let mut hinweise =
            engine::check(&profil, &engine::Kontext { satz: &p.satz, index: &p.index, sources: &p.sources });
        let files: Vec<String> = p.index.typ.keys().cloned().collect();
        match vale::check(&profil, &p.root, &files) {
            Ok(more) => hinweise.extend(more),
            Err(e) => fehler.push(e),
        }
        let thesis_type = p.satz.thesis_type();
        let umfang = profil
            .regeln
            .iter()
            .filter(|r| r.art == Some(profil::Art::Seitenumfang))
            .find(|r| r.gilt_fuer.is_none() || r.gilt_fuer == thesis_type)
            .and_then(|r| Some((r.min?, r.max?)));
        Ok(Pruefbericht {
            hinweise: with_stellen(p, hinweise),
            profil: Some(ProfilInfo { profil: profil.profil.clone(), bestaetigt: profil.bestaetigt }),
            fehler,
            umfang,
            ms: start.elapsed().as_secs_f64() * 1000.0,
        })
    })
}

/// Rechtschreibung und Grammatik (LTeX+). Startet LTeX+ beim ersten Aufruf (Java, einige Sekunden).
#[tauri::command]
pub async fn pruefung_rechtschreibung(state: State<'_, AppState>) -> Result<Vec<HinweisMitStelle>, String> {
    let (session, root, files) = with_project(&state, |p| {
        let files: Vec<(String, String)> = p
            .index
            .typ
            .keys()
            .filter_map(|f| p.satz.world().source_text(f).ok().map(|t| (f.clone(), t)))
            .collect();
        Ok((p.ltex.clone(), p.root.clone(), files))
    })?;
    let session = match session {
        Some(session) => session,
        None => {
            let program = find_ltex().ok_or("LTeX+ nicht gefunden (brew install ltex-ls-plus)")?;
            let session = tauri::async_runtime::spawn_blocking(move || LtexSession::start(&program, &root))
                .await
                .map_err(|e| e.to_string())??;
            let session = Arc::new(session);
            with_project(&state, |p| {
                p.ltex = Some(session.clone());
                Ok(())
            })?;
            session
        }
    };
    let hinweise = tauri::async_runtime::spawn_blocking(move || session.check(&files, Duration::from_secs(60)))
        .await
        .map_err(|e| e.to_string())??;
    with_project(&state, |p| Ok(with_stellen(p, hinweise)))
}
