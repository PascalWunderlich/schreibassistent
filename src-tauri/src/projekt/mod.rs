//! Arbeits-Vault öffnen, Dateien beobachten, Git-Sicherung.
//!
//! Ein offenes Projekt hält den Satz (`satz`) und schreibt jede Änderung sofort und
//! atomar auf die Platte; Git-Sicherung höchstens alle 5 Minuten (`git`).

pub mod neu;
pub mod commands;
pub mod files;
pub mod git;
pub mod settings;
pub mod watch;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::index::{Index, cache::Cache};
use crate::lsp::{TinymistSession, find_tinymist};
use crate::quellen::{self, Source};
use crate::satz::{CompileReport, Satz};

pub const NAME: &str = "projekt";

/// Hauptdatei einer Arbeit (docs/DATA-FORMATS.md).
pub const MAIN_FILE: &str = "Arbeit/main.typ";

pub struct Project {
    pub root: PathBuf,
    pub satz: Satz,
    /// Tinymist für Vervollständigung, Fehler, Hover, Definition (fehlt, wenn nicht installiert).
    pub lsp: Option<TinymistSession>,
    /// Zeitpunkt des letzten Autosave-Commits (Sekunden seit 1970).
    pub last_autosave: Option<u64>,
    /// Prüfsumme des zuletzt selbst geschriebenen Inhalts je Datei: eigene Schreibvorgänge
    /// sind keine „Änderung von außen“.
    written: HashMap<String, u128>,
    /// Dateibeobachtung (endet beim Schließen des Projekts).
    pub watcher: Option<notify::RecommendedWatcher>,
    /// Quellen aus `System/quellen.bib`.
    pub sources: Vec<Source>,
    /// Kapitel ↔ Quelle ↔ Notiz ↔ Wissensseite.
    pub index: Index,
    cache: Option<Cache>,
    /// Fehler beim Lesen der `.bib` (Quellen bleiben dann auf dem letzten Stand).
    pub sources_error: Option<String>,
    /// Rechtschreibung (LTeX+), beim ersten Bedarf gestartet.
    pub ltex: Option<std::sync::Arc<crate::pruefung::ltex::LtexSession>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectInfo {
    pub root: String,
    pub name: String,
    /// Eigenes Git-Repo vorhanden? Sonst bietet die Oberfläche `git init` an.
    pub git: bool,
    pub last_autosave: Option<u64>,
    pub report: CompileReport,
    /// Warum Tinymist nicht läuft (z. B. nicht installiert).
    pub lsp_error: Option<String>,
    /// Fehler beim Setzen (die Arbeit öffnet trotzdem, damit man sie beheben kann).
    pub compile_errors: Vec<String>,
    /// Ort des ersten Fehlers: Datei (relativ) und Byte-Position.
    pub error_location: Option<(String, usize)>,
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
    /// Öffnet den Vault; `on_diagnostics` erhält den Pfad, wenn Tinymist neue Fehler meldet.
    pub fn open(root: &Path, on_diagnostics: impl Fn(&str) + Send + 'static) -> Result<(Self, ProjectInfo), Vec<String>> {
        let root = root.canonicalize().map_err(|e| vec![format!("{}: {e}", root.display())])?;
        if !root.join(MAIN_FILE).is_file() {
            return Err(vec![format!("{} ist kein Arbeits-Vault: {MAIN_FILE} fehlt.", root.display())]);
        }
        let mut satz = Satz::open(&root, MAIN_FILE).map_err(|e| vec![e])?;
        // Auch eine Arbeit mit Fehlern öffnen: Sonst käme man an den Text nicht mehr heran
        // (z. B. nach einem Absturz mitten in einer Eingabe).
        let (report, compile_errors) = match satz.compile() {
            Ok(report) => (report, Vec::new()),
            Err(errors) => (CompileReport::default(), errors),
        };
        let git = git::is_own_repo(&root);
        if git {
            git::ensure_gitignore(&root).map_err(|e| vec![e])?;
        }
        let last_autosave = if git { git::last_autosave(&root) } else { None };
        let (lsp, lsp_error) = match find_tinymist() {
            Some(program) => match TinymistSession::start(&program, &root, MAIN_FILE, on_diagnostics) {
                Ok(session) => (Some(session), None),
                Err(e) => (None, Some(e)),
            },
            None => (None, Some("Tinymist nicht gefunden (brew install tinymist)".to_owned())),
        };
        let info = ProjectInfo {
            root: root.to_string_lossy().into_owned(),
            name: root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            git,
            last_autosave,
            report,
            lsp_error,
            compile_errors,
            error_location: satz.error_location(),
        };
        let cache = Cache::open(&root.join(".schreibassistent/cache.sqlite")).ok();
        let mut project = Self {
            root,
            satz,
            lsp,
            last_autosave,
            written: HashMap::new(),
            watcher: None,
            sources: Vec::new(),
            index: Index::default(),
            cache,
            sources_error: None,
            ltex: None,
        };
        project.reload_sources();
        project.reindex().map_err(|e| vec![e])?;
        Ok((project, info))
    }

    /// Schreibt eine Datei (nur `Arbeit/` und `Notizen/`) und sichert ggf. per Git.
    pub fn write(&mut self, relative: &str, text: &str) -> Result<SaveInfo, String> {
        if !files::is_writable(relative) {
            return Err(format!("{relative} ist nur lesbar."));
        }
        files::atomic_write(&files::resolve(&self.root, relative)?, text)?;
        self.written.insert(relative.to_owned(), typst_utils::hash128(text.as_bytes()));
        if relative.ends_with(".typ")
            && let Some(lsp) = &self.lsp
        {
            lsp.file_changed(relative)?;
        }
        if relative.ends_with(".typ") || relative.ends_with(".md") {
            self.reindex()?;
        }
        Ok(self.autosave(false))
    }

    /// `System/quellen.bib` neu lesen (z. B. nach dem Auto-Export von Better BibTeX).
    pub fn reload_sources(&mut self) {
        match quellen::load(&self.root) {
            Ok(sources) => {
                self.sources = sources;
                self.sources_error = None;
            }
            Err(e) => self.sources_error = Some(e),
        }
    }

    /// Index neu aufbauen; unveränderte Dateien kommen aus dem Cache.
    pub fn reindex(&mut self) -> Result<(), String> {
        self.index = Index::build(&self.root, self.cache.as_ref())?;
        Ok(())
    }

    /// Citekeys der `.bib`.
    pub fn citekeys(&self) -> std::collections::BTreeSet<String> {
        self.sources.iter().map(|s| s.citekey.clone()).collect()
    }

    /// Git-Sicherung, höchstens alle 5 Minuten. Erzwingen ist nicht nötig: Jede Änderung
    /// liegt sofort auf der Platte, der nächste Autosave nimmt sie mit.
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

    /// Änderung von außen übernehmen. `false`, wenn die Datei nur selbst geschrieben wurde.
    pub fn external_change(&mut self, relative: &str) -> Result<bool, String> {
        let path = files::resolve(&self.root, relative)?;
        let on_disk = std::fs::read(&path).ok();
        if on_disk.as_ref().map(|b| typst_utils::hash128(b.as_slice())) == self.written.get(relative).copied() {
            return Ok(false);
        }
        self.written.remove(relative);
        self.satz.world().reload_from_disk(relative)?;
        if relative == quellen::BIB_FILE {
            self.reload_sources();
        }
        if relative.ends_with(".typ") || relative.ends_with(".md") || relative == quellen::BIB_FILE {
            self.reindex()?;
        }
        Ok(true)
    }

    /// Neues Kapitel `Arbeit/kapitel/NN-name.typ` mit Überschrift anlegen und in `main.typ`
    /// hinter dem letzten Kapitel einbinden. Gibt den Pfad des Kapitels zurück.
    pub fn new_chapter(&mut self, title: &str) -> Result<String, String> {
        let title = title.trim();
        let slug = slug(title);
        if slug.is_empty() {
            return Err("Bitte einen Titel für das Kapitel angeben.".to_owned());
        }
        let dir = self.root.join("Arbeit/kapitel");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let next = std::fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .filter_map(|e| e.file_name().to_string_lossy().get(..2)?.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;
        let file = format!("kapitel/{next:02}-{slug}.typ");
        let path = format!("Arbeit/{file}");

        let main = self.satz.world().source_text(MAIN_FILE)?;
        let include = format!("#include \"{file}\"");
        let mut lines: Vec<&str> = main.lines().collect();
        match lines.iter().rposition(|l| l.trim_start().starts_with("#include \"kapitel/")) {
            Some(i) => lines.insert(i + 1, &include),
            None => lines.push(&include),
        }
        let main = format!("{}\n", lines.join("\n"));

        self.write(&path, &format!("= {title}\n\n"))?;
        self.write(MAIN_FILE, &main)?;
        self.satz.world().set_source_text(MAIN_FILE, &main)?;
        Ok(path)
    }

    /// Aktuellen Satz-Quelltext einer `.typ`-Datei auf die Platte schreiben.
    pub fn write_source(&mut self, relative: &str) -> Result<SaveInfo, String> {
        let text = self.satz.world().source_text(relative)?;
        self.write(relative, &text)
    }
}

/// Dateiname aus einem Titel: klein, Umlaute umschrieben, Wörter mit Bindestrich.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for c in title.to_lowercase().chars() {
        match c {
            'ä' => out.push_str("ae"),
            'ö' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            c if c.is_ascii_alphanumeric() => out.push(c),
            _ if !out.ends_with('-') && !out.is_empty() => out.push('-'),
            _ => {}
        }
    }
    out.trim_end_matches('-').to_owned()
}

/// Heutiges Datum (UTC) als `YYYY-MM-DD`, z. B. fürs Frontmatter neuer Notizen.
pub fn today() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    crate::satz::datetime_from_unix(now)
        .and_then(|d| Some(format!("{:04}-{:02}-{:02}", d.year()?, d.month()?, d.day()?)))
        .unwrap_or_default()
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
    fn slug_from_title() {
        assert_eq!(slug("Grundlagen & Begriffe"), "grundlagen-begriffe");
        assert_eq!(slug("Größe der Lösung"), "groesse-der-loesung");
        assert_eq!(slug("  !!  "), "");
    }

    #[test]
    fn new_chapter_is_created_included_and_set() {
        let root = copy_example("chapter");
        let (mut project, _) = Project::open(&root, |_| {}).unwrap();
        let before = project.satz.compile().unwrap().page_count;
        let path = project.new_chapter("Kritische Reflexion").unwrap();
        assert_eq!(path, "Arbeit/kapitel/06-kritische-reflexion.typ");
        assert_eq!(std::fs::read_to_string(root.join(&path)).unwrap(), "= Kritische Reflexion\n\n");
        let main = std::fs::read_to_string(root.join(MAIN_FILE)).unwrap();
        assert!(
            main.contains("#include \"kapitel/05-zusammenfassung.typ\"\n#include \"kapitel/06-kritische-reflexion.typ\""),
            "{main}"
        );
        let report = project.satz.compile().unwrap();
        assert!(report.page_count > before, "neues Kapitel beginnt auf einer neuen Seite");
        assert!(project.satz.first_page_of(&path).is_some());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn opens_vault_even_if_it_does_not_compile() {
        let root = copy_example("broken");
        let path = root.join("Arbeit/kapitel/05-zusammenfassung.typ");
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}\n@sutton2018 #gibtsnich\n")).unwrap();
        let (_, info) = Project::open(&root, |_| {}).expect("öffnet trotz Fehler");
        assert_eq!(info.report.page_count, 0);
        assert!(info.compile_errors[0].contains("unknown variable: gibtsnich"), "{:?}", info.compile_errors);
        assert_eq!(info.error_location.unwrap().0, "Arbeit/kapitel/05-zusammenfassung.typ");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rejects_folder_without_main_file() {
        let errors = Project::open(&std::env::temp_dir(), |_| {}).err().unwrap();
        assert!(errors[0].contains("kein Arbeits-Vault"));
    }

    #[test]
    fn edit_is_written_to_disk_and_autosaved() {
        let root = copy_example("edit");
        let (mut project, info) = Project::open(&root, |_| {}).unwrap();
        assert!(!info.git);
        git::init(&project.root).unwrap();

        let path = "Arbeit/kapitel/03-umsetzung.typ";
        let text = project.satz.world().source_text(path).unwrap();
        let start = text.find("Die Modellierung").unwrap();
        project.satz.replace(path, start, start, "Neu: ").unwrap();
        let save = project.write_source(path).unwrap();
        assert!(save.autosaved.is_some(), "{save:?}");
        assert!(std::fs::read_to_string(root.join(path)).unwrap().contains("Neu: Die Modellierung"));

        assert!(!project.external_change(path).unwrap(), "eigener Schreibvorgang ist keine Änderung von außen");
        std::fs::write(root.join(path), "= Von außen\n\nGeändert in VS Code.\n").unwrap();
        assert!(project.external_change(path).unwrap());
        let report = project.satz.compile().unwrap();
        assert!(!report.changed_pages.is_empty());
        assert!(project.satz.world().source_text(path).unwrap().contains("Geändert in VS Code"));

        assert!(project.write("Wissen/Quellen/x.md", "nein").is_err(), "Wissen/ ist nur lesbar");
        assert!(project.write("Notizen/idee.md", "# Idee\n").is_ok());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
