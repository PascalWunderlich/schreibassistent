//! Dateien beobachten: Änderungen von außen (Zotero-Export nach `System/quellen.bib`,
//! Bearbeiten in VS Code …) melden, gesammelt über kurze Zeit.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

/// Ereignisse so lange sammeln, bis so lange Ruhe ist.
const QUIET: Duration = Duration::from_millis(200);

/// Pfad relativ zum Vault, falls er beobachtet werden soll: keine versteckten Dateien
/// oder Ordner (`.git/`, `.schreibassistent/`, temporäre Dateien des atomaren Schreibens).
pub fn relevant(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    if relative.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.')) {
        return None;
    }
    // Immer mit „/“, auch unter Windows (die Oberfläche und der Index rechnen so).
    Some(super::files::rel_string(relative))
}

/// Startet die Beobachtung; `on_change` erhält die geänderten Pfade (relativ, sortiert).
/// Die Beobachtung endet, wenn der zurückgegebene Watcher verworfen wird.
pub fn start(root: &Path, on_change: impl Fn(Vec<String>) + Send + 'static) -> Result<RecommendedWatcher, String> {
    let root: PathBuf = root.to_path_buf();
    let (tx, rx) = channel::<PathBuf>();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event {
            for path in event.paths {
                let _ = tx.send(path);
            }
        }
    })
    .map_err(|e| format!("Dateibeobachtung nicht möglich: {e}"))?;
    watcher.watch(&root, RecursiveMode::Recursive).map_err(|e| format!("Dateibeobachtung nicht möglich: {e}"))?;

    std::thread::spawn(move || {
        let root = crate::projekt::files::canonical(&root).unwrap_or(root);
        let mut batch = BTreeSet::new();
        loop {
            match rx.recv_timeout(QUIET) {
                Ok(path) => {
                    let path = crate::projekt::files::canonical(&path).unwrap_or(path);
                    // Ordner melden ihre Änderung mit (Linux/inotify); gemeint sind die Dateien darin.
                    if path.is_dir() {
                        continue;
                    }
                    if let Some(relative) = relevant(&root, &path) {
                        batch.insert(relative);
                    }
                }
                Err(RecvTimeoutError::Timeout) if !batch.is_empty() => on_change(std::mem::take(&mut batch).into_iter().collect()),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn ignores_hidden_and_temporary_files() {
        let root = Path::new("/vault");
        assert_eq!(relevant(root, Path::new("/vault/System/quellen.bib")).as_deref(), Some("System/quellen.bib"));
        assert_eq!(relevant(root, Path::new("/vault/.git/index")), None);
        assert_eq!(relevant(root, Path::new("/vault/.schreibassistent/app.log")), None);
        assert_eq!(relevant(root, Path::new("/vault/Arbeit/.main.typ.123.tmp")), None);
        assert_eq!(relevant(Path::new("/anders"), Path::new("/vault/x")), None);
    }

    #[test]
    fn reports_changed_files_once_per_batch() {
        let root = std::env::temp_dir().join(format!("sa-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("System")).unwrap();
        let (tx, rx) = channel();
        let _watcher = start(&root, move |paths| {
            let _ = tx.send(paths);
        })
        .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(root.join("System/quellen.bib"), "@book{a,}").unwrap();
        std::fs::write(root.join("System/quellen.bib"), "@book{a, title={T}}").unwrap();
        let paths = rx.recv_timeout(Duration::from_secs(5)).expect("Änderung gemeldet");
        assert_eq!(paths, ["System/quellen.bib"]);
        let _ = std::fs::remove_dir_all(&root); // Windows hält Dateien teils noch kurz offen
    }
}
