//! Tauri-Befehle des Moduls `pruefung`: feste Prüfungen und Vale sofort, Rechtschreibung
//! (LTeX+) getrennt, weil sie beim ersten Mal mehrere Sekunden braucht.

use std::sync::Arc;
use std::time::{Duration, Instant};

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};

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
pub async fn pruefung_rechtschreibung(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<HinweisMitStelle>, String> {
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
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || session_for(&app, &root))
                .await
                .map_err(|e| e.to_string())??
        }
    };
    let hinweise = tauri::async_runtime::spawn_blocking(move || session.check(&files, Duration::from_secs(60)))
        .await
        .map_err(|e| e.to_string())??;
    with_project(&state, |p| Ok(with_stellen(p, hinweise)))
}

/// Nur eine LTeX+-Sitzung gleichzeitig starten (Vorwärmen und erste Prüfung treffen sich oft).
static START: Mutex<()> = Mutex::new(());

/// Vorhandenes Ergebnis oder genau einmal starten: Wer gleichzeitig kommt, wartet auf den ersten.
fn ensure_once<T: Clone>(
    existing: impl Fn() -> Option<T>,
    start: impl FnOnce() -> Result<T, String>,
    store: impl FnOnce(&T),
) -> Result<T, String> {
    let _only_one = START.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(value) = existing() {
        return Ok(value);
    }
    let value = start()?;
    store(&value);
    Ok(value)
}

/// LTeX+-Sitzung des geöffneten Projekts: die vorhandene oder eine neu gestartete (blockiert dann einige Sekunden).
fn session_for(app: &AppHandle, root: &Path) -> Result<Arc<LtexSession>, String> {
    let state = app.state::<AppState>();
    ensure_once(
        || state.0.lock().unwrap().as_ref().filter(|p| p.root == root).and_then(|p| p.ltex.clone()),
        || {
            let program = find_ltex().ok_or("LTeX+ nicht gefunden (Installation der App unvollständig)")?;
            LtexSession::start(&program, root).map(Arc::new)
        },
        |session| {
            if let Some(project) = state.0.lock().unwrap().as_mut().filter(|p| p.root == root) {
                project.ltex = Some(session.clone());
            }
        },
    )
}

/// LTeX+ beim Öffnen einer Arbeit im Hintergrund starten (Java braucht kalt rund 15 s), damit die
/// erste Rechtschreibprüfung nicht warten muss. Fehler zeigt erst die Prüfung selbst.
pub fn warm_up(app: AppHandle, root: PathBuf) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = session_for(&app, &root) {
            eprintln!("LTeX+ vorwärmen: {e}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn concurrent_callers_start_once() {
        let started = AtomicUsize::new(0);
        let slot: Mutex<Option<usize>> = Mutex::new(None);
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    let value = ensure_once(
                        || *slot.lock().unwrap(),
                        || {
                            std::thread::sleep(Duration::from_millis(50));
                            Ok(started.fetch_add(1, Ordering::SeqCst) + 1)
                        },
                        |v| *slot.lock().unwrap() = Some(*v),
                    );
                    assert_eq!(value, Ok(1));
                });
            }
        });
        assert_eq!(started.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failed_start_is_not_stored_and_can_be_retried() {
        let slot: Mutex<Option<u8>> = Mutex::new(None);
        let failed = ensure_once(|| *slot.lock().unwrap(), || Err("kein Java".to_owned()), |v| *slot.lock().unwrap() = Some(*v));
        assert_eq!(failed, Err("kein Java".to_owned()));
        assert_eq!(*slot.lock().unwrap(), None);
        assert_eq!(ensure_once(|| *slot.lock().unwrap(), || Ok(7), |v| *slot.lock().unwrap() = Some(*v)), Ok(7));
    }
}
