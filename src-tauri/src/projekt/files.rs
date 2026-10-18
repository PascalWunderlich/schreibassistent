//! Dateien im Arbeits-Vault: Pfade prüfen, Dateibaum, sicheres Schreiben.

use std::io::Write;
use std::path::{Component, Path, PathBuf};

/// Ordner, die in der Seitenleiste erscheinen (docs/DATA-FORMATS.md).
pub const TREE_ROOTS: [&str; 3] = ["Arbeit", "Notizen", "Wissen"];

/// Der Mensch schreibt in `Arbeit/` und `Notizen/`; `Wissen/` pflegt später die KI
/// und ist in der Oberfläche nur lesbar.
pub fn is_writable(relative: &str) -> bool {
    relative.starts_with("Arbeit/") || relative.starts_with("Notizen/")
}

/// Pfad relativ zum Vault → absoluter Pfad. Lehnt absolute Pfade und `..` ab.
pub fn resolve(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if relative.is_empty() || path.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(format!("Ungültiger Pfad im Arbeits-Vault: {relative}"));
    }
    Ok(root.join(path))
}

/// Schreibt atomar: erst eine temporäre Datei im selben Ordner, dann umbenennen.
/// Ein Absturz hinterlässt so entweder die alte oder die neue Datei, nie eine halbe.
pub fn atomic_write(path: &Path, text: &str) -> Result<(), String> {
    let dir = path.parent().ok_or("Datei ohne Ordner")?;
    let name = path.file_name().ok_or("Datei ohne Namen")?.to_string_lossy();
    let tmp = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    result.map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{} lässt sich nicht speichern: {e}", path.display())
    })
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct TreeNode {
    pub name: String,
    /// Pfad relativ zum Vault.
    pub path: String,
    pub is_dir: bool,
    pub writable: bool,
    pub children: Vec<TreeNode>,
}

/// Dateibaum der Ordner aus `TREE_ROOTS`; versteckte Dateien und `.gitkeep` fehlen.
pub fn tree(root: &Path) -> Vec<TreeNode> {
    TREE_ROOTS
        .iter()
        .filter(|name| root.join(name).is_dir())
        .map(|name| node(root, name))
        .collect()
}

fn node(root: &Path, relative: &str) -> TreeNode {
    let path = root.join(relative);
    let mut children: Vec<TreeNode> = std::fs::read_dir(&path)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| node(root, &format!("{relative}/{}", e.file_name().to_string_lossy())))
        .collect();
    children.sort_by(|a, b| (!a.is_dir, &a.name).cmp(&(!b.is_dir, &b.name)));
    TreeNode {
        name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path: relative.to_owned(),
        is_dir: path.is_dir(),
        // Ordner und Dateien unter `Arbeit/` bzw. `Notizen/` sind beschreibbar.
        writable: is_writable(&format!("{relative}/")),
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_paths_leaving_the_vault() {
        let root = Path::new("/vault");
        assert!(resolve(root, "../x").is_err());
        assert!(resolve(root, "/etc/passwd").is_err());
        assert!(resolve(root, "Arbeit/../../x").is_err());
        assert_eq!(resolve(root, "Arbeit/main.typ").unwrap(), Path::new("/vault/Arbeit/main.typ"));
    }

    #[test]
    fn only_arbeit_and_notizen_are_writable() {
        assert!(is_writable("Arbeit/kapitel/01.typ"));
        assert!(is_writable("Notizen/idee.md"));
        assert!(!is_writable("Wissen/Quellen/mueller2024.md"));
        assert!(!is_writable("System/ki-protokoll.jsonl"));
    }

    #[test]
    fn atomic_write_replaces_content_and_leaves_no_temp_file() {
        let dir = std::env::temp_dir().join(format!("sa-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.typ");
        atomic_write(&file, "eins").unwrap();
        atomic_write(&file, "zwei").unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "zwei");
        let names: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names.len(), 1, "{names:?}");
        let _ = std::fs::remove_dir_all(&dir); // Windows hält Dateien teils noch kurz offen
    }

    #[test]
    fn tree_lists_vault_folders_and_marks_wissen_read_only() {
        let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../beispiel-arbeit"));
        let tree = tree(root);
        assert_eq!(tree.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(), ["Arbeit", "Notizen", "Wissen"]);
        let arbeit = &tree[0];
        let kapitel = arbeit.children.iter().find(|n| n.name == "kapitel").unwrap();
        assert!(kapitel.children.iter().any(|n| n.name == "01-einleitung.typ" && n.writable));
        let wissen = &tree[2];
        assert!(!wissen.writable);
        assert!(wissen.children.iter().all(|n| !n.writable));
    }
}
