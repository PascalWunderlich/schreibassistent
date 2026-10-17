//! Automatische Sicherung: Der Arbeits-Vault ist ein eigenes Git-Repo; beim Speichern
//! entsteht höchstens alle 5 Minuten ein lokaler Commit „autosave“. Kein Push.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const AUTOSAVE_MESSAGE: &str = "autosave";
pub const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5 * 60);

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    // Ohne eigene Git-Identität (user.name/email) trotzdem committen können.
    let mut command = Command::new("git");
    command.arg("-C").arg(root);
    if config(root, "user.name").is_none() {
        command.args(["-c", "user.name=Schreibassistent"]);
    }
    if config(root, "user.email").is_none() {
        command.args(["-c", "user.email=autosave@schreibassistent.local"]);
    }
    let output = command.args(args).output().map_err(|e| format!("git nicht ausführbar: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim()))
    }
}

fn config(root: &Path, key: &str) -> Option<String> {
    let output = Command::new("git").arg("-C").arg(root).args(["config", key]).output().ok()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (output.status.success() && !value.is_empty()).then_some(value)
}

/// Ist `root` selbst die Wurzel eines Git-Repos? (Nicht nur ein Unterordner eines anderen
/// Repos – sonst landeten Autosaves z. B. im Repo der App.)
pub fn is_own_repo(root: &Path) -> bool {
    let Ok(top) = git(root, &["rev-parse", "--show-toplevel"]) else { return false };
    let (Ok(top), Ok(root)) = (Path::new(top.trim()).canonicalize(), root.canonicalize()) else { return false };
    top == root
}

/// Was nie in die Sicherung gehört: Cache der App und Dateien des Finders.
const IGNORED: [&str; 2] = [".schreibassistent/", ".DS_Store"];

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
    git(root, &["init", "-q"])?;
    ensure_gitignore(root)?;
    git(root, &["add", "-A"])?;
    git(root, &["commit", "-q", "-m", "Arbeit angelegt"])?;
    Ok(())
}

/// Zeitpunkt des letzten Autosave-Commits (Sekunden seit 1970), falls vorhanden.
pub fn last_autosave(root: &Path) -> Option<u64> {
    let out = git(root, &["log", "-1", "--format=%ct", "--fixed-strings", "--grep", AUTOSAVE_MESSAGE]).ok()?;
    out.trim().parse().ok()
}

/// Committet alle Änderungen als „autosave“, wenn der letzte Autosave mindestens
/// `AUTOSAVE_INTERVAL` her ist (oder `force`, nur für Tests) und es Änderungen gibt.
/// Gibt den Zeitpunkt des neuen Commits zurück.
pub fn autosave(root: &Path, force: bool) -> Result<Option<u64>, String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    if !force && last_autosave(root).is_some_and(|last| now.saturating_sub(last) < AUTOSAVE_INTERVAL.as_secs()) {
        return Ok(None);
    }
    if git(root, &["status", "--porcelain"])?.trim().is_empty() {
        return Ok(None);
    }
    git(root, &["add", "-A"])?;
    git(root, &["commit", "-q", "-m", AUTOSAVE_MESSAGE])?;
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
        git(root, &["log", "--format=%s"]).unwrap().lines().map(str::to_owned).collect()
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
        std::fs::remove_dir_all(&root).unwrap();
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
        std::fs::remove_dir_all(&root).unwrap();
    }
}
