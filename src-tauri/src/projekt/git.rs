//! Automatische Sicherung: Der Arbeits-Vault ist ein eigenes Git-Repo; beim Speichern
//! entsteht höchstens alle 5 Minuten ein lokaler Commit „autosave“. Kein Push.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use git2::{IndexAddOption, Repository, Signature, StatusOptions};

pub const AUTOSAVE_MESSAGE: &str = "autosave";
pub const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5 * 60);

fn err(e: git2::Error) -> String {
    format!("Git: {}", e.message())
}

/// Autor der Sicherung: die Git-Identität des Nutzers, sonst „Schreibassistent“.
fn signature(repo: &Repository) -> Result<Signature<'static>, String> {
    match repo.signature() {
        Ok(sig) => Ok(sig.to_owned()),
        Err(_) => Signature::now("Schreibassistent", "autosave@schreibassistent.local").map_err(err),
    }
}

/// Ist `root` selbst die Wurzel eines Git-Repos? (Nicht nur ein Unterordner eines anderen
/// Repos – sonst landeten Autosaves z. B. im Repo der App.) `Repository::open` sucht nicht
/// in übergeordneten Ordnern.
pub fn is_own_repo(root: &Path) -> bool {
    Repository::open(root).is_ok_and(|repo| !repo.is_bare())
}

/// Was nie in die Sicherung gehört: Cache der App, Dateien des Finders, PDF-Exporte.
const IGNORED: [&str; 3] = [".schreibassistent/", ".DS_Store", "Export/"];

/// Ergänzt fehlende Einträge in `.gitignore` (vorhandene bleiben unverändert).
pub fn ensure_gitignore(root: &Path) -> Result<(), String> {
    let ignore = root.join(".gitignore");
    let mut text = std::fs::read_to_string(&ignore).unwrap_or_default();
    let missing: Vec<_> = IGNORED.iter().filter(|e| !text.lines().any(|l| l.trim() == **e)).collect();
    if missing.is_empty() {
        return Ok(());
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    for entry in missing {
        text.push_str(entry);
        text.push('\n');
    }
    super::files::atomic_write(&ignore, &text)
}

/// Legt das Repo an: `.gitignore` (siehe `IGNORED`) und erster Commit.
pub fn init(root: &Path) -> Result<(), String> {
    let repo = Repository::init(root).map_err(err)?;
    ensure_gitignore(root)?;
    commit_all(&repo, "Arbeit angelegt")?;
    Ok(())
}

/// Alle Änderungen (neu, geändert, gelöscht; ohne Ignoriertes) als ein Commit auf `HEAD`.
fn commit_all(repo: &Repository, message: &str) -> Result<(), String> {
    let mut index = repo.index().map_err(err)?;
    index.add_all(["*"], IndexAddOption::DEFAULT, None).map_err(err)?;
    index.update_all(["*"], None).map_err(err)?;
    index.write().map_err(err)?;
    let tree = repo.find_tree(index.write_tree().map_err(err)?).map_err(err)?;
    let sig = signature(repo)?;
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents).map_err(err)?;
    Ok(())
}

/// Zeitpunkt des letzten Autosave-Commits (Sekunden seit 1970), falls vorhanden.
pub fn last_autosave(root: &Path) -> Option<u64> {
    let repo = Repository::open(root).ok()?;
    let mut walk = repo.revwalk().ok()?;
    walk.push_head().ok()?;
    walk.filter_map(Result::ok)
        .filter_map(|id| repo.find_commit(id).ok())
        .find(|c| c.message().is_some_and(|m| m.contains(AUTOSAVE_MESSAGE)))
        .map(|c| c.time().seconds().max(0) as u64)
}

/// Committet alle Änderungen als „autosave“, wenn der letzte Autosave mindestens
/// `AUTOSAVE_INTERVAL` her ist (oder `force`, nur für Tests) und es Änderungen gibt.
/// Gibt den Zeitpunkt des neuen Commits zurück.
pub fn autosave(root: &Path, force: bool) -> Result<Option<u64>, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    if !force && last_autosave(root).is_some_and(|last| now.saturating_sub(last) < AUTOSAVE_INTERVAL.as_secs()) {
        return Ok(None);
    }
    let repo = Repository::open(root).map_err(err)?;
    let mut options = StatusOptions::new();
    options.include_untracked(true).recurse_untracked_dirs(true);
    if repo.statuses(Some(&mut options)).map_err(err)?.is_empty() {
        return Ok(None);
    }
    commit_all(&repo, AUTOSAVE_MESSAGE)?;
    Ok(Some(now))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_vault(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("sa-git-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Arbeit")).unwrap();
        std::fs::write(dir.join("Arbeit/main.typ"), "= Test\n").unwrap();
        dir
    }

    fn commits(root: &Path) -> Vec<String> {
        let repo = Repository::open(root).unwrap();
        let mut walk = repo.revwalk().unwrap();
        walk.push_head().unwrap();
        walk.map(|id| repo.find_commit(id.unwrap()).unwrap().message().unwrap().trim().to_owned()).collect()
    }

    #[test]
    fn init_creates_own_repo_with_gitignore() {
        let root = temp_vault("init");
        assert!(!is_own_repo(&root));
        init(&root).unwrap();
        assert!(is_own_repo(&root));
        let ignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(ignore.contains(".schreibassistent/") && ignore.contains(".DS_Store"), "{ignore}");
        assert_eq!(commits(&root), ["Arbeit angelegt"]);
        let _ = std::fs::remove_dir_all(&root); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn subfolder_of_another_repo_is_not_its_own_repo() {
        // beispiel-arbeit liegt im Repo der App: dort darf nie ein Autosave landen.
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
        assert!(!is_own_repo(root));
    }

    #[test]
    fn autosave_at_most_every_five_minutes_and_only_with_changes() {
        let root = temp_vault("autosave");
        init(&root).unwrap();
        assert_eq!(autosave(&root, false).unwrap(), None, "keine Änderung");
        std::fs::write(root.join("Arbeit/main.typ"), "= Test\n\nText.\n").unwrap();
        assert!(autosave(&root, false).unwrap().is_some(), "erster Autosave");
        std::fs::write(root.join("Arbeit/main.typ"), "= Test\n\nMehr Text.\n").unwrap();
        assert_eq!(autosave(&root, false).unwrap(), None, "innerhalb von 5 Minuten");
        assert!(autosave(&root, true).unwrap().is_some(), "erzwungen, z. B. beim Schließen");
        assert_eq!(commits(&root), ["autosave", "autosave", "Arbeit angelegt"]);
        let _ = std::fs::remove_dir_all(&root); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn autosave_records_deletions_and_skips_ignored_files() {
        let root = temp_vault("delete");
        init(&root).unwrap();
        std::fs::remove_file(root.join("Arbeit/main.typ")).unwrap();
        std::fs::create_dir_all(root.join(".schreibassistent")).unwrap();
        std::fs::write(root.join(".schreibassistent/cache.db"), "x").unwrap();
        assert!(autosave(&root, true).unwrap().is_some());
        let repo = Repository::open(&root).unwrap();
        let tree = repo.head().unwrap().peel_to_tree().unwrap();
        assert!(tree.get_path(Path::new("Arbeit/main.typ")).is_err(), "gelöscht");
        assert!(tree.get_path(Path::new(".schreibassistent/cache.db")).is_err(), "ignoriert");
        assert!(last_autosave(&root).is_some());
        let _ = std::fs::remove_dir_all(&root); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn commit_works_without_git_identity() {
        let root = temp_vault("identity");
        init(&root).unwrap();
        let repo = Repository::open(&root).unwrap();
        let author = repo.head().unwrap().peel_to_commit().unwrap().author().name().map(str::to_owned);
        assert!(author.is_some_and(|n| !n.is_empty()));
        let _ = std::fs::remove_dir_all(&root); // Windows hält Dateien teils noch kurz offen
    }
}
