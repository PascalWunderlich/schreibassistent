//! Fremdprogramme finden (Tinymist, Vale, LTeX+, uv, typst2vast): zuerst die mit der App
//! gelieferten Werkzeuge (`<Ressourcen>/werkzeuge/`), dann der `PATH`, im Entwicklungsbetrieb
//! zusätzlich bekannte Orte. Nur dieses Modul sucht Programme.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Ressourcenordner der App merken (beim Start, `app.path().resource_dir()`).
pub fn init(resource_dir: PathBuf) {
    let _ = RESOURCE_DIR.set(resource_dir);
}

/// Ordner `werkzeuge/` mit den gelieferten Programmen, falls vorhanden.
pub fn bundle_dir() -> Option<PathBuf> {
    let from_resources = RESOURCE_DIR.get().map(|r| r.join("werkzeuge"));
    let from_build = cfg!(debug_assertions).then(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../build/werkzeuge"));
    from_resources.into_iter().chain(from_build).find(|d| d.is_dir())
}

/// Pfad eines Programms (`bin/` der Lieferung, dann `PATH`, dann bekannte Orte).
pub fn find(name: &str) -> Option<PathBuf> {
    find_in(name, bundle_dir().as_deref(), &path_dirs(), &known_dirs())
}

/// Ordner, die Fremdprogramme als `PATH` bekommen (z. B. Vale braucht `typst2vast`).
pub fn search_path() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(bundle) = bundle_dir() {
        dirs.push(bundle.join("bin"));
    }
    dirs.extend(path_dirs());
    dirs.extend(known_dirs());
    dirs
}

/// Konfiguration von Vale: aus der Lieferung, sonst aus dem Repo (Entwicklung).
pub fn vale_config() -> PathBuf {
    let bundled = bundle_dir().map(|d| d.join("vale/vale.ini")).filter(|p| p.is_file());
    bundled.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../sidecars/vale/.vale.ini"))
}

/// Gelieferte Vorlage `fhdw-arbeit` (Ordner mit `typst.toml`).
pub fn template_dir() -> Option<PathBuf> {
    let bundled = bundle_dir().map(|d| d.join("typst/fhdw-arbeit"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../typst/fhdw-arbeit");
    bundled.into_iter().chain([repo]).find(|d| d.join("typst.toml").is_file())
}

fn path_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default()
}

/// Orte, an denen Entwickler Programme ablegen (nur Rückfall hinter `PATH`).
fn known_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "macos")]
    dirs.push(PathBuf::from("/opt/homebrew/bin"));
    #[cfg(unix)]
    dirs.push(PathBuf::from("/usr/local/bin"));
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" });
    if let Some(home) = home {
        let home = PathBuf::from(home);
        dirs.push(home.join(".cargo/bin"));
        if !cfg!(windows) {
            dirs.push(home.join(".local/bin"));
        }
    }
    dirs
}

/// Dateinamen eines Programms auf diesem System.
fn file_names(name: &str) -> Vec<String> {
    if cfg!(windows) {
        ["exe", "bat", "cmd"].iter().map(|e| format!("{name}.{e}")).collect()
    } else {
        vec![name.to_owned()]
    }
}

fn find_in(name: &str, bundle: Option<&Path>, path: &[PathBuf], known: &[PathBuf]) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(bundle) = bundle {
        dirs.push(bundle.join("bin"));
        dirs.push(bundle.join(name).join("bin"));
    }
    dirs.extend(path.iter().cloned());
    dirs.extend(known.iter().cloned());
    let names = file_names(name);
    dirs.iter().flat_map(|d| names.iter().map(move |n| d.join(n))).find(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sa-werkzeuge-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn exe(name: &str) -> String {
        file_names(name).remove(0)
    }

    #[test]
    fn bundled_program_beats_path() {
        let dir = temp("order");
        let bundle = dir.join("werkzeuge");
        let path_dir = dir.join("path");
        touch(&bundle.join("bin").join(exe("vale")));
        touch(&path_dir.join(exe("vale")));
        let found = find_in("vale", Some(&bundle), &[path_dir], &[]).unwrap();
        assert!(found.starts_with(&bundle));
    }

    #[test]
    fn path_beats_known_places() {
        let dir = temp("path");
        let path_dir = dir.join("path");
        let known = dir.join("known");
        touch(&path_dir.join(exe("uv")));
        touch(&known.join(exe("uv")));
        assert!(find_in("uv", None, &[path_dir.clone()], &[known]).unwrap().starts_with(&path_dir));
    }

    #[test]
    fn known_place_is_the_fallback() {
        let dir = temp("known");
        let known = dir.join("known");
        touch(&known.join(exe("tinymist")));
        assert!(find_in("tinymist", None, &[dir.join("leer")], &[known.clone()]).unwrap().starts_with(&known));
    }

    #[test]
    fn program_in_its_own_folder_is_found() {
        let dir = temp("own");
        let bundle = dir.join("werkzeuge");
        touch(&bundle.join("ltex-ls-plus").join("bin").join(exe("ltex-ls-plus")));
        assert!(find_in("ltex-ls-plus", Some(&bundle), &[], &[]).is_some());
    }

    #[test]
    fn missing_program_is_none() {
        let dir = temp("none");
        assert_eq!(find_in("gibt-es-nicht", Some(&dir), &[dir.clone()], &[dir.clone()]), None);
    }

    #[test]
    fn folder_with_program_name_is_not_a_program() {
        let dir = temp("folder");
        std::fs::create_dir_all(dir.join(exe("vale"))).unwrap();
        assert_eq!(find_in("vale", None, &[dir], &[]), None);
    }
}
