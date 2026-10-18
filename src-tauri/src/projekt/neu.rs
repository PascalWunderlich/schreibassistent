//! Neue Arbeit anlegen und Arbeit kopieren („Speichern unter“). Beides schreibt nur die App auf Klick des Menschen.

use std::path::Path;

const REGELPROFIL: &str = include_str!("../../../typst/fhdw-arbeit/regelprofil-fhdw.yaml");
const SCHEMA: &str = include_str!("../../../typst/fhdw-arbeit/regelprofil.schema.json");

/// Eingaben für eine neue Arbeit.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NeueArbeit {
    pub title: String,
    pub author: String,
    /// „Praxisarbeit“ oder „Bachelorarbeit“.
    pub thesis_type: String,
}

/// Text für ein Typst-Zeichenkettenliteral: Anführungszeichen und Rückwärtsschrägstriche maskieren.
fn typst_string(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ")
}

fn is_empty_dir(dir: &Path) -> bool {
    std::fs::read_dir(dir).map(|mut d| d.next().is_none()).unwrap_or(false)
}

fn write(dir: &Path, relative: &str, text: &str) -> Result<(), String> {
    let path = dir.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| format!("{relative}: {e}"))
}

/// Legt das Gerüst einer Arbeit an. Der Ordner darf nicht existieren oder muss leer sein.
pub fn create(dir: &Path, input: &NeueArbeit) -> Result<(), String> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err("Bitte einen Titel angeben.".to_owned());
    }
    if input.thesis_type != "Praxisarbeit" && input.thesis_type != "Bachelorarbeit" {
        return Err("Art der Arbeit: Praxisarbeit oder Bachelorarbeit.".to_owned());
    }
    if dir.exists() && !is_empty_dir(dir) {
        return Err(format!("{} ist nicht leer. Bitte einen neuen oder leeren Ordner wählen.", dir.display()));
    }
    let second = if input.thesis_type == "Bachelorarbeit" { "\"Name des Zweitprüfers\"" } else { "none" };
    let metadata = format!(
        "// Angaben zur Arbeit – gemeinsam genutzt von main.typ und erklaerung-einzeln.typ.\n#let thesis = (\n  title: \"{}\",\n  thesis-type: \"{}\",\n  study-program: \"Informatik\",\n  author: \"{}\",\n  student-id: none,\n  company: none,\n  examiner: \"Name des Prüfers\",\n  second-examiner: {second},\n  city: \"Hannover\",\n  date: datetime(year: {year}, month: 1, day: 1),\n  confidentiality-notice: false,\n)\n",
        typst_string(title),
        input.thesis_type,
        typst_string(input.author.trim()),
        year = super::today().get(..4).and_then(|y| y.parse::<i32>().ok()).unwrap_or(2026),
    );
    let main = "#import \"@local/fhdw-arbeit:0.1.0\": *\n#import \"metadaten.typ\": thesis\n#import \"glossar.typ\": glossary\n\n#show: fhdw-thesis.with(\n  ..thesis,\n  summary: [Kurzfassung hier schreiben.],\n  glossary: glossary,\n  bibliography: bibliography(\"../System/quellen.bib\"),\n  citation-style: \"alphanumeric\",\n  list-of-figures: true,\n  list-of-tables: true,\n  appendix: include \"anhang.typ\",\n)\n\n#include \"kapitel/01-einleitung.typ\"\n";
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    write(dir, "Arbeit/main.typ", main)?;
    write(dir, "Arbeit/metadaten.typ", &metadata)?;
    write(dir, "Arbeit/glossar.typ", "// Abkürzungen (mit `long`) und Begriffe (mit `description`); im Text: #gls(\"schluessel\").\n#let glossary = ()\n")?;
    write(dir, "Arbeit/anhang.typ", "= Weitere Unterlagen\n\nKeine.\n")?;
    write(dir, "Arbeit/kapitel/01-einleitung.typ", "= Einleitung <kap:einleitung>\n\nHier beginnt die Arbeit.\n")?;
    std::fs::create_dir_all(dir.join("Arbeit/abbildungen")).map_err(|e| e.to_string())?;
    write(dir, "System/quellen.bib", "% Quellen: Better-BibTeX-Auto-Export aus Zotero hierher legen.\n")?;
    write(dir, "System/regelprofil.yaml", REGELPROFIL)?;
    write(dir, "System/regelprofil.schema.json", SCHEMA)?;
    write(dir, "System/woerterbuch.txt", "")?;
    std::fs::create_dir_all(dir.join("Notizen")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir.join("Wissen")).map_err(|e| e.to_string())?;
    Ok(())
}

/// Kopiert die Arbeit samt Notizen und Wissen nach `target` (ohne den Cache `.schreibassistent`).
pub fn copy_to(source: &Path, target: &Path) -> Result<(), String> {
    if target.exists() && !is_empty_dir(target) {
        return Err(format!("{} ist nicht leer. Bitte einen neuen oder leeren Ordner wählen.", target.display()));
    }
    if target.starts_with(source) {
        return Err("Das Ziel liegt in der Arbeit selbst.".to_owned());
    }
    fn walk(from: &Path, to: &Path) -> Result<(), String> {
        std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(from).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            if name == ".schreibassistent" || name == ".DS_Store" {
                continue;
            }
            let (src, dst) = (entry.path(), to.join(&name));
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                walk(&src, &dst)?;
            } else {
                std::fs::copy(&src, &dst).map_err(|e| format!("{}: {e}", src.display()))?;
            }
        }
        Ok(())
    }
    walk(source, target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("sa-neu-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn input(title: &str) -> NeueArbeit {
        NeueArbeit { title: title.to_owned(), author: "Max \"M\" Muster".to_owned(), thesis_type: "Praxisarbeit".to_owned() }
    }

    #[test]
    fn creates_skeleton_with_escaped_title() {
        let dir = temp("create");
        create(&dir, &input("Titel mit \"Zitat\"")).unwrap();
        for f in ["Arbeit/main.typ", "Arbeit/metadaten.typ", "Arbeit/kapitel/01-einleitung.typ", "System/quellen.bib", "System/regelprofil.yaml"] {
            assert!(dir.join(f).is_file(), "{f} fehlt");
        }
        assert!(dir.join("Notizen").is_dir() && dir.join("Wissen").is_dir());
        let meta = std::fs::read_to_string(dir.join("Arbeit/metadaten.typ")).unwrap();
        assert!(meta.contains("title: \"Titel mit \\\"Zitat\\\"\""));
        assert!(meta.contains("second-examiner: none"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn bachelor_needs_second_examiner() {
        let dir = temp("bachelor");
        create(&dir, &NeueArbeit { thesis_type: "Bachelorarbeit".to_owned(), ..input("T") }).unwrap();
        let meta = std::fs::read_to_string(dir.join("Arbeit/metadaten.typ")).unwrap();
        assert!(meta.contains("second-examiner: \"Name des Zweitprüfers\""));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn refuses_nonempty_folder_and_bad_input() {
        let dir = temp("nonempty");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("x.txt"), "x").unwrap();
        assert!(create(&dir, &input("T")).unwrap_err().contains("nicht leer"));
        assert!(create(&temp("notitle"), &input("  ")).is_err());
        assert!(create(&temp("badtype"), &NeueArbeit { thesis_type: "Diplom".to_owned(), ..input("T") }).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn copies_without_cache() {
        let (src, dst) = (temp("copy-src"), temp("copy-dst"));
        create(&src, &input("T")).unwrap();
        std::fs::create_dir_all(src.join(".schreibassistent")).unwrap();
        std::fs::write(src.join(".schreibassistent/cache.db"), "x").unwrap();
        copy_to(&src, &dst).unwrap();
        assert!(dst.join("Arbeit/main.typ").is_file());
        assert!(!dst.join(".schreibassistent").exists());
        assert!(copy_to(&src, &src.join("innen")).is_err());
        std::fs::remove_dir_all(&src).unwrap();
        std::fs::remove_dir_all(&dst).unwrap();
    }
}

