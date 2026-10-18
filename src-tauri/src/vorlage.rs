//! Vorlage `@local/fhdw-arbeit` und die Typst-Pakete, die sie braucht, beim Start in den
//! Typst-Paketordner des Systems legen (die App lädt selbst keine Pakete nach).
//!
//! Quelle ist `<werkzeuge>/typst/`: `fhdw-arbeit/` (Vorlage) und `packages/<Namespace>/<Name>/<Version>/`.
//! Ziel ist der Paketordner von typst-kit (macOS `~/Library/Application Support/typst/packages`,
//! Windows `%APPDATA%\typst\packages`, Linux `~/.local/share/typst/packages`).

use std::path::{Path, PathBuf};

use typst_kit::packages::FsPackages;

/// Vorlage und Pakete aus der Lieferung installieren; gibt zurück, was neu geschrieben wurde.
pub fn install_bundled() -> Result<Vec<String>, String> {
    let source = crate::werkzeuge::bundle_dir().map(|d| d.join("typst")).filter(|d| d.is_dir());
    let source = source.ok_or("Vorlage nicht in der Installation gefunden")?;
    let data = FsPackages::system_data().ok_or("Typst-Paketordner des Systems nicht bestimmbar")?;
    install_into(data.path(), &source)
}

/// Wie `install_bundled`, mit festem Ziel- und Quellordner (Tests).
pub fn install_into(data: &Path, source: &Path) -> Result<Vec<String>, String> {
    let mut done = Vec::new();

    let template = source.join("fhdw-arbeit");
    if template.join("typst.toml").is_file() {
        let version = version_of(&template)?;
        let target = data.join("local/fhdw-arbeit").join(&version);
        // Ein Symlink stammt von `typst/installieren.sh` (Entwicklung): nicht anfassen.
        let is_link = std::fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink());
        if !is_link && !same_content(&template, &target) {
            replace_dir(&template, &target)?;
            done.push(format!("@local/fhdw-arbeit:{version}"));
        }
    }

    let packages = source.join("packages");
    for namespace in dirs_in(&packages) {
        for name in dirs_in(&namespace) {
            for version in dirs_in(&name) {
                let relative = version.strip_prefix(&packages).map_err(|e| e.to_string())?;
                let target = data.join(relative);
                if !target.exists() {
                    replace_dir(&version, &target)?;
                    let part = |p: &Path| p.file_name().unwrap_or_default().to_string_lossy().into_owned();
                    done.push(format!("@{}/{}:{}", part(&namespace), part(&name), part(&version)));
                }
            }
        }
    }
    Ok(done)
}

fn version_of(template: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(template.join("typst.toml")).map_err(|e| format!("typst.toml: {e}"))?;
    text.lines()
        .find_map(|l| l.trim().strip_prefix("version"))
        .and_then(|rest| rest.split('"').nth(1))
        .map(str::to_owned)
        .ok_or_else(|| "typst.toml: keine Version".to_owned())
}

fn dirs_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<PathBuf> = entries.filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_dir()).collect();
    out.sort();
    out
}

fn files_in(dir: &Path) -> Vec<(PathBuf, PathBuf)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(PathBuf, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else if let Ok(relative) = path.strip_prefix(base) {
                out.push((relative.to_path_buf(), path));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// Gleiche Dateien mit gleichem Inhalt?
fn same_content(a: &Path, b: &Path) -> bool {
    let (fa, fb) = (files_in(a), files_in(b));
    fa.len() == fb.len()
        && !fa.is_empty()
        && fa.iter().zip(&fb).all(|((ra, pa), (rb, pb))| ra == rb && std::fs::read(pa).ok() == std::fs::read(pb).ok())
}

/// `from` nach `to` kopieren: erst in einen Nachbarordner, dann umbenennen (kein halber Stand).
fn replace_dir(from: &Path, to: &Path) -> Result<(), String> {
    let parent = to.parent().ok_or("Zielordner ohne Elternordner")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let tmp = parent.join(format!(".{}.neu", to.file_name().unwrap_or_default().to_string_lossy()));
    let _ = std::fs::remove_dir_all(&tmp);
    for (relative, path) in files_in(from) {
        let dest = tmp.join(&relative);
        std::fs::create_dir_all(dest.parent().unwrap_or(&tmp)).map_err(|e| e.to_string())?;
        std::fs::copy(&path, &dest).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if to.exists() {
        std::fs::remove_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    }
    std::fs::rename(&tmp, to).map_err(|e| format!("{}: {e}", to.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sa-vorlage-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// Lieferung mit Vorlage 0.1.0 und dem Paket preview/glossarium 0.5.10.
    fn bundle(dir: &Path) -> PathBuf {
        let source = dir.join("typst");
        write(&source.join("fhdw-arbeit/typst.toml"), "[package]\nname = \"fhdw-arbeit\"\nversion = \"0.1.0\"\n");
        write(&source.join("fhdw-arbeit/lib.typ"), "// Vorlage");
        write(&source.join("fhdw-arbeit/unter/ordner.typ"), "// tief");
        write(&source.join("packages/preview/glossarium/0.5.10/typst.toml"), "[package]\nname = \"glossarium\"\n");
        source
    }

    #[test]
    fn empty_data_dir_gets_template_and_packages_and_second_run_changes_nothing() {
        let dir = temp("leer");
        let source = bundle(&dir);
        let data = dir.join("data");
        let first = install_into(&data, &source).unwrap();
        assert_eq!(first, ["@local/fhdw-arbeit:0.1.0", "@preview/glossarium:0.5.10"]);
        assert!(data.join("local/fhdw-arbeit/0.1.0/lib.typ").is_file());
        assert!(data.join("local/fhdw-arbeit/0.1.0/unter/ordner.typ").is_file());
        assert!(data.join("preview/glossarium/0.5.10/typst.toml").is_file());
        assert!(install_into(&data, &source).unwrap().is_empty(), "zweiter Start ändert nichts");
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn changed_template_is_replaced_but_existing_package_is_kept() {
        let dir = temp("geaendert");
        let source = bundle(&dir);
        let data = dir.join("data");
        install_into(&data, &source).unwrap();
        write(&source.join("fhdw-arbeit/lib.typ"), "// neuer Stand");
        write(&data.join("preview/glossarium/0.5.10/typst.toml"), "// vom Nutzer");
        assert_eq!(install_into(&data, &source).unwrap(), ["@local/fhdw-arbeit:0.1.0"]);
        assert_eq!(std::fs::read_to_string(data.join("local/fhdw-arbeit/0.1.0/lib.typ")).unwrap(), "// neuer Stand");
        assert_eq!(std::fs::read_to_string(data.join("preview/glossarium/0.5.10/typst.toml")).unwrap(), "// vom Nutzer");
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn new_version_is_installed_next_to_the_old_one() {
        let dir = temp("version");
        let source = bundle(&dir);
        let data = dir.join("data");
        install_into(&data, &source).unwrap();
        write(&source.join("fhdw-arbeit/typst.toml"), "[package]\nname = \"fhdw-arbeit\"\nversion = \"0.2.0\"\n");
        assert_eq!(install_into(&data, &source).unwrap(), ["@local/fhdw-arbeit:0.2.0"]);
        assert!(data.join("local/fhdw-arbeit/0.1.0").is_dir() && data.join("local/fhdw-arbeit/0.2.0").is_dir());
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }

    #[cfg(unix)]
    #[test]
    fn developer_symlink_is_left_alone() {
        let dir = temp("link");
        let source = bundle(&dir);
        let data = dir.join("data");
        let repo = dir.join("repo");
        write(&repo.join("lib.typ"), "// Entwicklung");
        std::fs::create_dir_all(data.join("local/fhdw-arbeit")).unwrap();
        std::os::unix::fs::symlink(&repo, data.join("local/fhdw-arbeit/0.1.0")).unwrap();
        assert_eq!(install_into(&data, &source).unwrap(), ["@preview/glossarium:0.5.10"]);
        assert_eq!(std::fs::read_to_string(repo.join("lib.typ")).unwrap(), "// Entwicklung");
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn missing_source_installs_nothing() {
        let dir = temp("quelle");
        assert!(install_into(&dir.join("data"), &dir.join("gibt-es-nicht")).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }
}
