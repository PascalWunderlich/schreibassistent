//! Regelprofil laden, feste Prüfungen, Vale/LTeX+ einbinden.

pub mod commands;
pub mod engine;
pub mod ltex;
pub mod profil;
pub mod vale;

pub const NAME: &str = "pruefung";

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::engine::{self, Hinweis};
    use super::profil;
    use crate::index::Index;
    use crate::quellen;
    use crate::satz::Satz;

    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "pruefung");
    }

    /// Ein `FEHLERFALL`-Marker: Regel, Datei und die Zeilen, für die er gilt (bis zur Leerzeile).
    #[derive(Debug)]
    struct Marker {
        regel: String,
        datei: String,
        zeilen: std::ops::RangeInclusive<usize>,
        ganze_arbeit: bool,
    }

    fn markers(root: &Path) -> Vec<Marker> {
        let mut files = vec![quellen::BIB_FILE.to_owned()];
        let mut stack = vec![root.join("Arbeit")];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "typ") {
                    files.push(path.strip_prefix(root).unwrap().to_string_lossy().into_owned());
                }
            }
        }
        let mut out = Vec::new();
        for file in files {
            let Ok(text) = std::fs::read_to_string(root.join(&file)) else { continue };
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let Some(rest) = line.trim().strip_prefix("// FEHLERFALL ").or_else(|| line.trim().strip_prefix("% FEHLERFALL ")) else {
                    continue;
                };
                let regel = rest.split_whitespace().next().unwrap().to_owned();
                let end = (i + 1..lines.len()).find(|&j| lines[j].trim().is_empty()).unwrap_or(lines.len());
                out.push(Marker { regel, datei: file.clone(), zeilen: (i + 1)..=end, ganze_arbeit: rest.contains("(ganze Arbeit") });
            }
        }
        out
    }

    fn hinweise(root: &Path) -> Vec<Hinweis> {
        let mut satz = Satz::open(root, "Arbeit/main.typ").unwrap();
        let _ = satz.compile();
        let profil = profil::load(root).unwrap().unwrap();
        let sources = quellen::load(root).unwrap();
        let index = Index::build(root, None).unwrap();
        let mut out = engine::check(&profil, &engine::Kontext { satz: &satz, index: &index, sources: &sources });
        let files: Vec<String> = index.typ.keys().cloned().collect();
        match super::vale::check(&profil, root, &files) {
            Ok(hinweise) => out.extend(hinweise),
            Err(e) => eprintln!("Vale übersprungen: {e}"),
        }
        // Rechtschreibung mit LTeX+, falls installiert (Kaltstart Java: einige Sekunden).
        if let Some(program) = super::ltex::find_ltex() {
            // Ein Java nach dem anderen: mehrere Kaltstarts gleichzeitig überschreiten auf CI-Rechnern die Fristen.
            static ONE_LTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let _one = ONE_LTEX.lock().unwrap_or_else(|e| e.into_inner());
            let session = super::ltex::LtexSession::start(&program, root).unwrap();
            let texts: Vec<(String, String)> =
                files.iter().map(|f| (f.clone(), std::fs::read_to_string(root.join(f)).unwrap())).collect();
            out.extend(session.check(&texts, std::time::Duration::from_secs(300)).unwrap());
        } else {
            eprintln!("LTeX+ übersprungen: nicht installiert");
        }
        out
    }

    fn covers(m: &Marker, h: &Hinweis) -> bool {
        m.regel == h.regel_id
            && match (&h.datei, h.zeile) {
                (None, _) => m.ganze_arbeit,
                (Some(d), Some(z)) => *d == m.datei && m.zeilen.contains(&z),
                (Some(_), None) => false,
            }
    }

    /// Abnahme P5: Jeder FEHLERFALL wird gemeldet, ohne FEHLERFALL keine Meldung.
    fn assert_exactly_error_cases(root: &Path) {
        let markers = markers(root);
        let hinweise = hinweise(root);
        assert!(!markers.is_empty(), "keine Marker in {}", root.display());
        let unreported: Vec<_> = markers.iter().filter(|m| !hinweise.iter().any(|h| covers(m, h))).collect();
        let false_alarms: Vec<_> = hinweise.iter().filter(|h| !markers.iter().any(|m| covers(m, h))).collect();
        assert!(unreported.is_empty(), "nicht gemeldet: {unreported:#?}");
        assert!(false_alarms.is_empty(), "Meldungen ohne FEHLERFALL: {false_alarms:#?}");
    }

    fn vault(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
    }

    #[test]
    fn example_reports_exactly_its_error_cases() {
        assert_exactly_error_cases(&vault("beispiel-arbeit"));
    }

    #[test]
    fn missing_citekey_is_reported_although_it_does_not_compile() {
        assert_exactly_error_cases(&vault("tests/pruefung/zitiert-fehlt"));
    }

    #[test]
    fn changed_declaration_is_reported() {
        assert_exactly_error_cases(&vault("tests/pruefung/erklaerung-geaendert"));
    }
}
