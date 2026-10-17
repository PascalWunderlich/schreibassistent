//! Arbeits-Vault öffnen, Dateien beobachten, Git-Sicherung.
//!
//! Ein offenes Projekt hält den Satz (`satz`) und schreibt jede Änderung sofort und
//! atomar auf die Platte; Git-Sicherung höchstens alle 5 Minuten (`git`).

pub mod commands;
pub mod files;
pub mod git;
pub mod settings;

use std::path::{Path, PathBuf};

use crate::satz::{CompileReport, Satz};

pub const NAME: &str = "projekt";

/// Hauptdatei einer Arbeit (docs/DATA-FORMATS.md).
pub const MAIN_FILE: &str = "Arbeit/main.typ";

pub struct Project {
    pub root: PathBuf,
    pub satz: Satz,
    /// Zeitpunkt des letzten Autosave-Commits (Sekunden seit 1970).
    pub last_autosave: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectInfo {
    pub root: String,
    pub name: String,
    /// Eigenes Git-Repo vorhanden? Sonst bietet die Oberfläche `git init` an.
    pub git: bool,
    pub last_autosave: Option<u64>,
    pub report: CompileReport,
}

/// Ergebnis eines Speichervorgangs.
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct SaveInfo {
    /// Zeitpunkt eines neuen Autosave-Commits, falls einer entstand.
    pub autosaved: Option<u64>,
    /// Fehler der Git-Sicherung (Speichern selbst hat geklappt).
    pub git_error: Option<String>,
}

impl Project {
    pub fn open(root: &Path) -> Result<(Self, ProjectInfo), Vec<String>> {
        let root = root.canonicalize().map_err(|e| vec![format!("{}: {e}", root.display())])?;
        if !root.join(MAIN_FILE).is_file() {
            return Err(vec![format!("{} ist kein Arbeits-Vault: {MAIN_FILE} fehlt.", root.display())]);
        }
        let mut satz = Satz::open(&root, MAIN_FILE).map_err(|e| vec![e])?;
        let report = satz.compile()?;
        let git = git::is_own_repo(&root);
        let last_autosave = if git { git::last_autosave(&root) } else { None };
        let info = ProjectInfo {
            root: root.to_string_lossy().into_owned(),
            name: root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            git,
            last_autosave,
            report,
        };
        Ok((Self { root, satz, last_autosave }, info))
    }

    /// Schreibt eine Datei (nur `Arbeit/` und `Notizen/`) und sichert ggf. per Git.
    pub fn write(&mut self, relative: &str, text: &str) -> Result<SaveInfo, String> {
        if !files::is_writable(relative) {
            return Err(format!("{relative} ist nur lesbar."));
        }
        files::atomic_write(&files::resolve(&self.root, relative)?, text)?;
        Ok(self.autosave(false))
    }

    /// Git-Sicherung, höchstens alle 5 Minuten (oder erzwungen, z. B. beim Schließen).
    pub fn autosave(&mut self, force: bool) -> SaveInfo {
        if !git::is_own_repo(&self.root) {
            return SaveInfo::default();
        }
        match git::autosave(&self.root, force) {
            Ok(Some(time)) => {
                self.last_autosave = Some(time);
                SaveInfo { autosaved: Some(time), git_error: None }
            }
            Ok(None) => SaveInfo::default(),
            Err(e) => SaveInfo { autosaved: None, git_error: Some(e) },
        }
    }

    /// Aktuellen Satz-Quelltext einer `.typ`-Datei auf die Platte schreiben.
    pub fn write_source(&mut self, relative: &str) -> Result<SaveInfo, String> {
        let text = self.satz.world().source_text(relative)?;
        self.write(relative, &text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "projekt");
    }

    fn copy_example(name: &str) -> PathBuf {
        let target = std::env::temp_dir().join(format!("sa-projekt-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&target);
        let source = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
        let status = std::process::Command::new("cp").arg("-R").arg(source).arg(&target).status().unwrap();
        assert!(status.success());
        let _ = std::fs::remove_dir_all(target.join(".schreibassistent"));
        target
    }

    #[test]
    fn rejects_folder_without_main_file() {
        let errors = Project::open(&std::env::temp_dir()).err().unwrap();
        assert!(errors[0].contains("kein Arbeits-Vault"));
    }

    #[test]
    fn edit_is_written_to_disk_and_autosaved() {
        let root = copy_example("edit");
        let (mut project, info) = Project::open(&root).unwrap();
        assert!(!info.git);
        git::init(&project.root).unwrap();

        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let text = project.satz.world().source_text(path).unwrap();
        let start = text.find("Die Modellierung").unwrap();
        project.satz.replace(path, start, start, "Neu: ").unwrap();
        let save = project.write_source(path).unwrap();
        assert!(save.autosaved.is_some(), "{save:?}");
        assert!(std::fs::read_to_string(root.join(path)).unwrap().contains("Neu: Die Modellierung"));

        assert!(project.write("Wissen/Quellen/x.md", "nein").is_err(), "Wissen/ ist nur lesbar");
        assert!(project.write("Notizen/idee.md", "# Idee\n").is_ok());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
