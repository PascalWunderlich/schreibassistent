//! Tauri-Befehle des Moduls `projekt` und gemeinsamer App-Zustand.

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};

use super::files::{self, TreeNode};
use super::settings::{self, Settings};
use super::{Project, ProjectInfo, SaveInfo, git};

/// Das geöffnete Projekt (höchstens eins).
#[derive(Default)]
pub struct AppState(pub Mutex<Option<Project>>);

pub fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("settings.json"))
}

pub fn with_project<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&mut Project) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = state.0.lock().unwrap();
    f(guard.as_mut().ok_or("Kein Arbeits-Vault geöffnet")?)
}

/// Zuletzt geöffneter Arbeits-Vault, falls er noch existiert.
#[tauri::command]
pub async fn projekt_last(app: AppHandle) -> Result<Option<String>, String> {
    let last = settings::load(&settings_file(&app)?).last_vault;
    Ok(last.filter(|p| p.join(super::MAIN_FILE).is_file()).map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn projekt_open(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<ProjectInfo, Vec<String>> {
    let emitter = app.clone();
    let (project, info) = Project::open(std::path::Path::new(&path), move |file| {
        let _ = emitter.emit("lsp_diagnostics", file.to_owned());
    })?;
    let mut project = project;
    // Änderungen von außen: Satz neu laden und die Oberfläche benachrichtigen.
    let watch_app = app.clone();
    let watch_root = project.root.clone();
    project.watcher = super::watch::start(&project.root, move |paths| {
        let Some(state) = watch_app.try_state::<AppState>() else { return };
        let changed: Vec<String> = {
            let mut guard = state.0.lock().unwrap();
            let Some(project) = guard.as_mut().filter(|p| p.root == watch_root) else { return };
            paths.into_iter().filter(|p| project.external_change(p).unwrap_or(true)).collect()
        };
        if !changed.is_empty() {
            let _ = watch_app.emit("index_aktualisiert", ());
            let _ = watch_app.emit("datei_geaendert", changed);
        }
    })
    .ok();
    // Vorheriges Projekt beim Wechsel sichern (Regel: höchstens alle 5 Minuten).
    if let Some(mut previous) = state.0.lock().unwrap().replace(project) {
        previous.autosave(false);
    }
    crate::pruefung::commands::warm_up(app.clone(), PathBuf::from(&info.root));
    let file = settings_file(&app).map_err(|e| vec![e])?;
    let settings = Settings { last_vault: Some(PathBuf::from(&info.root)), ..settings::load(&file) };
    settings::save(&file, &settings).map_err(|e| vec![e])?;
    Ok(info)
}

#[tauri::command]
pub async fn projekt_git_init(state: State<'_, AppState>) -> Result<(), String> {
    with_project(&state, |p| git::init(&p.root))
}

#[tauri::command]
pub async fn projekt_tree(state: State<'_, AppState>) -> Result<Vec<TreeNode>, String> {
    with_project(&state, |p| Ok(files::tree(&p.root)))
}

#[tauri::command]
pub async fn projekt_read(state: State<'_, AppState>, path: String) -> Result<String, String> {
    with_project(&state, |p| {
        let file = files::resolve(&p.root, &path)?;
        std::fs::read_to_string(&file).map_err(|e| format!("{path}: {e}"))
    })
}

/// Ordner für PDF-Exporte (`Export/`, nicht in der Git-Sicherung); wird bei Bedarf angelegt.
#[tauri::command]
pub async fn projekt_export_dir(state: State<'_, AppState>) -> Result<String, String> {
    with_project(&state, |p| {
        let dir = p.root.join("Export");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir.to_string_lossy().into_owned())
    })
}

/// Neues Kapitel anlegen und einbinden; danach setzt die Oberfläche neu.
#[tauri::command]
pub async fn projekt_new_chapter(state: State<'_, AppState>, title: String) -> Result<String, String> {
    with_project(&state, |p| p.new_chapter(&title))
}

/// Neue Notiz `Notizen/<name>.md` mit Frontmatter `author: ich` (docs/DATA-FORMATS.md).
#[tauri::command]
pub async fn projekt_new_note(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let clean: String = name.trim().chars().filter(|c| !"/\\:*?\"<>|".contains(*c)).collect();
    if clean.is_empty() || clean.starts_with('.') {
        return Err("Ungültiger Name".to_owned());
    }
    let path = format!("Notizen/{clean}.md");
    with_project(&state, |p| {
        if files::resolve(&p.root, &path)?.exists() {
            return Err(format!("{path} gibt es schon"));
        }
        let today = crate::projekt::today();
        p.write(&path, &format!("---\nauthor: ich\ncreated: {today}\n---\n\n# {clean}\n\n"))?;
        Ok(path)
    })
}

/// Datei schreiben (Notizen, Quelltextmodus). `.typ`-Dateien gehen auch in den Satz.
#[tauri::command]
pub async fn projekt_write(state: State<'_, AppState>, path: String, text: String) -> Result<SaveInfo, String> {
    with_project(&state, |p| {
        let info = p.write(&path, &text)?;
        if path.ends_with(".typ") {
            p.satz.world().set_source_text(&path, &text)?;
        }
        Ok(info)
    })
}

/// Beim Schließen der App: sichern, falls der letzte Autosave mindestens 5 Minuten her ist.
/// Nicht erzwungen, sonst entstünden bei schnellen Neustarts viele Commits.
pub fn on_exit(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>()
        && let Some(project) = state.0.lock().unwrap().as_mut()
    {
        project.autosave(false);
    }
}

/// Ereignisse der Oberfläche protokollieren: `<Vault>/.schreibassistent/app.log` (nicht im Git).
#[tauri::command]
pub async fn projekt_log(state: State<'_, AppState>, message: String) -> Result<(), String> {
    use std::io::Write;
    with_project(&state, |p| {
        let dir = p.root.join(".schreibassistent");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("app.log"))
            .map_err(|e| e.to_string())?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        writeln!(file, "{:.3} {message}", now.as_secs_f64()).map_err(|e| e.to_string())
    })
}

/// Neue Arbeit im Ordner `path` anlegen; die Oberfläche öffnet sie danach mit `projekt_open`.
#[tauri::command]
pub async fn projekt_create(path: String, input: super::neu::NeueArbeit) -> Result<String, String> {
    super::neu::create(std::path::Path::new(&path), &input)?;
    Ok(path)
}

/// „Speichern unter“: die geöffnete Arbeit nach `path` kopieren (ohne Cache).
#[tauri::command]
pub async fn projekt_copy_to(state: State<'_, AppState>, path: String) -> Result<String, String> {
    with_project(&state, |p| {
        p.autosave(true);
        super::neu::copy_to(&p.root, std::path::Path::new(&path))
    })?;
    Ok(path)
}

/// „Speichern“: Git-Sicherung jetzt, nicht erst nach 5 Minuten. Die Dateien selbst liegen schon auf der Platte.
#[tauri::command]
pub async fn projekt_sichern(state: State<'_, AppState>) -> Result<SaveInfo, String> {
    with_project(&state, |p| Ok(p.autosave(true)))
}
