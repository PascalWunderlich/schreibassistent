//! Schreibrechte der KI (Leitplanke 1): `ki_schreibe` ist die einzige Funktion, mit der
//! KI-Ergebnisse auf die Platte kommen. Erlaubt sind nur Markdown-Dateien unter `Wissen/`;
//! das Protokoll schreibt `protokoll::append` (nur anhängen). Jeder andere Pfad wird
//! abgelehnt und der Versuch protokolliert.
//!
//! Geprüft wird zweimal: am Pfadtext (kein `..`, kein absoluter Pfad, erstes Glied genau
//! `Wissen`, nur `.md`) und an der Platte (der echte Ordner nach Auflösen aller Symlinks
//! liegt in `<Vault>/Wissen`, die Zieldatei ist kein Symlink).

use std::path::{Component, Path, PathBuf};

use crate::protokoll::{self, Kontext};

pub const KNOWLEDGE_DIR: &str = "Wissen";

/// Grund der Ablehnung, oder `Ok` mit dem Zielpfad.
fn check_text(relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() {
        return Err("leerer Pfad".into());
    }
    if relative.contains('\\') || relative.contains('\0') {
        return Err("ungültige Zeichen im Pfad".into());
    }
    let path = Path::new(relative);
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_str().ok_or("Pfad ist kein gültiges UTF-8")?;
                if part.starts_with('.') {
                    return Err(format!("versteckte Datei oder Ordner „{part}“"));
                }
                parts.push(part);
            }
            Component::ParentDir => return Err("„..“ im Pfad".into()),
            Component::CurDir => return Err("„.“ im Pfad".into()),
            Component::RootDir | Component::Prefix(_) => return Err("absoluter Pfad".into()),
        }
    }
    // Genau „Wissen“ (Byte für Byte): macOS unterscheidet Groß/klein nicht, „arbeit/“ wäre `Arbeit/`.
    if parts.first() != Some(&KNOWLEDGE_DIR) || parts.len() < 2 {
        return Err(format!("nur Dateien in {KNOWLEDGE_DIR}/ erlaubt"));
    }
    if !relative.ends_with(".md") {
        return Err("nur Markdown-Dateien (.md) erlaubt".into());
    }
    Ok(parts.iter().collect())
}

/// Ordner anlegen und prüfen, dass der aufgelöste Ordner im echten `Wissen/` liegt.
fn check_disk(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| format!("Vault nicht gefunden: {e}"))?;
    let allowed = root.join(KNOWLEDGE_DIR);
    // `Wissen` selbst darf kein Symlink sein (sonst zeigte er z. B. auf `Arbeit/`).
    match std::fs::symlink_metadata(&allowed) {
        Ok(meta) if meta.file_type().is_symlink() => return Err(format!("{KNOWLEDGE_DIR}/ ist ein Symlink")),
        Ok(meta) if !meta.is_dir() => return Err(format!("{KNOWLEDGE_DIR} ist kein Ordner")),
        Ok(_) => {}
        Err(_) => std::fs::create_dir(&allowed).map_err(|e| e.to_string())?,
    }
    let target = root.join(relative);
    let parent = target.parent().ok_or("kein Ordner")?;
    // Ordner Stück für Stück anlegen, jeder davon muss nach dem Auflösen in `Wissen/` liegen.
    let mut dir = allowed.clone();
    for part in relative.parent().into_iter().flat_map(|p| p.components()).skip(1) {
        dir.push(part);
        if !dir.exists() {
            std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
        }
        let real = dir.canonicalize().map_err(|e| e.to_string())?;
        if !real.starts_with(&allowed) {
            return Err(format!("Ordner zeigt aus {KNOWLEDGE_DIR}/ heraus (Symlink)"));
        }
    }
    let real_parent = parent.canonicalize().map_err(|e| e.to_string())?;
    if !real_parent.starts_with(&allowed) {
        return Err(format!("Ordner zeigt aus {KNOWLEDGE_DIR}/ heraus (Symlink)"));
    }
    let real_target = real_parent.join(target.file_name().ok_or("kein Dateiname")?);
    if std::fs::symlink_metadata(&real_target).is_ok_and(|m| m.file_type().is_symlink() || !m.is_file()) {
        return Err("Ziel ist ein Symlink oder keine Datei".into());
    }
    Ok(real_target)
}

/// Einzige Schreibfunktion der KI. Abgelehnte Versuche landen im Protokoll
/// (Aufgabe `schreiben`, Ergebnis `abgelehnt`) und als Fehler beim Aufrufer.
pub fn ki_schreibe(root: &Path, relative: &str, content: &str, aufgabe: &str) -> Result<(), String> {
    let result = check_text(relative).and_then(|p| check_disk(root, &p));
    match result {
        Ok(target) => crate::projekt::files::atomic_write(&target, content),
        Err(reason) => {
            let mut eintrag = protokoll::entry("", "", "schreiben", Kontext { datei: Some(relative.to_owned()), absatz: None }, relative);
            eintrag.antwort = format!("abgelehnt ({aufgabe}): {reason}");
            eintrag.ergebnis = "abgelehnt".into();
            protokoll::append(root, eintrag).map_err(|e| format!("Schreiben abgelehnt ({reason}); Protokoll: {e}"))?;
            Err(format!("Schreiben abgelehnt: {reason}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("sa-rechte-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for dir in ["Arbeit/kapitel", "Notizen", "System", "Wissen"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        std::fs::write(root.join("Arbeit/main.typ"), "= Original\n").unwrap();
        std::fs::write(root.join("Arbeit/kapitel/01.typ"), "Original\n").unwrap();
        std::fs::write(root.join("Notizen/idee.md"), "Original\n").unwrap();
        root
    }

    /// Nach jedem Angriff: `Arbeit/` und `Notizen/` unverändert, nichts Neues dort.
    fn untouched(root: &Path) {
        assert_eq!(std::fs::read_to_string(root.join("Arbeit/main.typ")).unwrap(), "= Original\n");
        assert_eq!(std::fs::read_to_string(root.join("Arbeit/kapitel/01.typ")).unwrap(), "Original\n");
        assert_eq!(std::fs::read_to_string(root.join("Notizen/idee.md")).unwrap(), "Original\n");
        let count = |d: &str| walk(&root.join(d)).len();
        assert_eq!(count("Arbeit"), 2);
        assert_eq!(count("Notizen"), 1);
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() { out.extend(walk(&path)) } else { out.push(path) }
        }
        out
    }

    fn denied_entries(root: &Path) -> usize {
        protokoll::read_all(root).unwrap().iter().filter(|e| e.ergebnis == "abgelehnt").count()
    }

    #[test]
    fn writes_markdown_in_knowledge() {
        let root = vault("erlaubt");
        ki_schreibe(&root, "Wissen/Quellen/sutton2018.md", "# sutton2018\n", "ingest").unwrap();
        assert_eq!(std::fs::read_to_string(root.join("Wissen/Quellen/sutton2018.md")).unwrap(), "# sutton2018\n");
        ki_schreibe(&root, "Wissen/Konzepte/Ümlaut Begriff.md", "x", "ingest").unwrap();
        assert_eq!(denied_entries(&root), 0);
        untouched(&root);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_everything_outside_knowledge() {
        let root = vault("abgelehnt");
        let attempts = [
            "Arbeit/main.typ",
            "Arbeit/kapitel/01.typ",
            "Arbeit/kapitel/neu.md",
            "Notizen/idee.md",
            "Notizen/neu.md",
            // Groß/klein: auf macOS dieselben Ordner.
            "arbeit/main.typ",
            "ARBEIT/kapitel/01.typ",
            "notizen/idee.md",
            "wissen/../Arbeit/main.typ",
            "WISSEN/x.md",
            // Aus Wissen/ heraus.
            "Wissen/../Arbeit/main.typ",
            "Wissen/../Notizen/idee.md",
            "Wissen/../../etc/x.md",
            "Wissen/./../Arbeit/x.md",
            "./Wissen/x.md",
            // Absolut.
            "/tmp/x.md",
            &format!("{}/Arbeit/main.typ", root.display()),
            &format!("{}/Wissen/x.md", root.display()),
            // System außer Protokoll, versteckte Dateien, andere Endungen.
            "System/ki-protokoll.jsonl",
            "System/regelprofil.yaml",
            "Wissen/.git/config.md",
            "Wissen/x.typ",
            "Wissen/x.md.typ",
            "Wissen",
            "Wissen/",
            "",
            // Unicode-Doppelgänger von „Wissen“ (kyrillisches „і“, NFD-Varianten u. Ä.).
            "Wіssen/x.md",
            "Wissen\u{200B}/x.md",
            "Arbeit\u{0301}/x.md",
            "Wissen\\..\\Arbeit\\main.typ",
        ];
        for (i, path) in attempts.iter().enumerate() {
            let result = ki_schreibe(&root, path, "KI-TEXT", "test");
            assert!(result.is_err(), "nicht abgelehnt: {path:?}");
            assert_eq!(denied_entries(&root), i + 1, "nicht protokolliert: {path:?}");
        }
        untouched(&root);
        assert!(walk(&root.join("Wissen")).is_empty());
        assert!(crate::protokoll::verify(&root).unwrap().fehler.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuses_symlinks_out_of_knowledge() {
        use std::os::unix::fs::symlink;
        let root = vault("symlink");
        // Ordner-Symlink in Wissen/ auf Arbeit/ und auf Notizen/.
        symlink(root.join("Arbeit"), root.join("Wissen/arbeit-link")).unwrap();
        symlink("../Notizen", root.join("Wissen/notizen-link")).unwrap();
        // Datei-Symlink in Wissen/ auf eine Kapiteldatei.
        symlink(root.join("Arbeit/kapitel/01.typ"), root.join("Wissen/kapitel.md")).unwrap();
        symlink(root.join("Notizen/idee.md"), root.join("Wissen/idee.md")).unwrap();
        for path in [
            "Wissen/arbeit-link/main.typ",
            "Wissen/arbeit-link/kapitel/x.md",
            "Wissen/notizen-link/idee.md",
            "Wissen/notizen-link/neu.md",
            "Wissen/kapitel.md",
            "Wissen/idee.md",
        ] {
            assert!(ki_schreibe(&root, path, "KI-TEXT", "test").is_err(), "nicht abgelehnt: {path}");
        }
        assert_eq!(denied_entries(&root), 6);
        untouched(&root);

        // Wissen/ selbst als Symlink auf Notizen/.
        std::fs::remove_dir_all(root.join("Wissen")).unwrap();
        symlink(root.join("Notizen"), root.join("Wissen")).unwrap();
        assert!(ki_schreibe(&root, "Wissen/neu.md", "KI-TEXT", "test").is_err());
        assert!(ki_schreibe(&root, "Wissen/idee.md", "KI-TEXT", "test").is_err());
        untouched(&root);
        std::fs::remove_dir_all(root).unwrap();
    }
}
