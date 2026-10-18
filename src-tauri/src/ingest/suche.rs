//! Volltextindex (SQLite FTS5) über alle Abschnitte aller eingelesenen Quellen:
//! `.schreibassistent/abschnitte.sqlite` (Cache, jederzeit löschbar, Leitplanke 6).

use std::path::Path;

use rusqlite::{Connection, params};

use super::extrakt::Extrakt;

pub const FILE: &str = ".schreibassistent/abschnitte.sqlite";

pub struct Suche {
    db: Connection,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Treffer {
    pub citekey: String,
    pub seite: u32,
    pub label: String,
    pub text: String,
    /// Index des Abschnitts im Extrakt.
    pub nr: usize,
}

/// Suchwort für FTS5 als Phrase (Sonderzeichen wie `-` oder `:` nicht als Operator).
fn phrase(word: &str) -> String {
    format!("\"{}\"", word.replace('"', ""))
}

impl Suche {
    pub fn open(root: &Path) -> Result<Self, String> {
        let file = root.join(FILE);
        std::fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
        Self::with(Connection::open(file).map_err(|e| e.to_string())?)
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self, String> {
        Self::with(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }

    fn with(db: Connection) -> Result<Self, String> {
        db.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS abschnitte USING fts5(
                citekey UNINDEXED, seite UNINDEXED, label UNINDEXED, nr UNINDEXED, ueberschrift, text,
                tokenize = 'unicode61 remove_diacritics 2')",
        )
        .map_err(|e| format!("Volltextindex: {e}"))?;
        Ok(Self { db })
    }

    /// Abschnitte einer Quelle ersetzen.
    pub fn replace(&mut self, citekey: &str, extrakt: &Extrakt) -> Result<(), String> {
        let tx = self.db.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM abschnitte WHERE citekey = ?1", params![citekey]).map_err(|e| e.to_string())?;
        {
            let mut insert = tx
                .prepare("INSERT INTO abschnitte (citekey, seite, label, nr, ueberschrift, text) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
                .map_err(|e| e.to_string())?;
            for (nr, a) in extrakt.abschnitte.iter().enumerate() {
                insert.execute(params![citekey, a.seite, a.label, nr as i64, a.ueberschrift, a.text]).map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }

    /// Beste Treffer (BM25) für irgendeines der Wörter, optional nur in einer Quelle.
    pub fn search(&self, citekey: Option<&str>, words: &[String], limit: usize) -> Result<Vec<Treffer>, String> {
        let query: Vec<String> = words.iter().filter(|w| !w.replace('"', "").trim().is_empty()).map(|w| phrase(w)).collect();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let query = query.join(" OR ");
        let mut statement = self
            .db
            .prepare(
                "SELECT citekey, seite, label, text, nr FROM abschnitte
                 WHERE abschnitte MATCH ?1 AND (?2 IS NULL OR citekey = ?2)
                 ORDER BY bm25(abschnitte) LIMIT ?3",
            )
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map(params![query, citekey, limit as i64], |r| {
                Ok(Treffer {
                    citekey: r.get(0)?,
                    seite: r.get(1)?,
                    label: r.get(2)?,
                    text: r.get(3)?,
                    nr: r.get::<_, i64>(4)? as usize,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn count(&self, citekey: &str) -> Result<usize, String> {
        self.db
            .query_row("SELECT COUNT(*) FROM abschnitte WHERE citekey = ?1", params![citekey], |r| r.get::<_, i64>(0))
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_sections_with_page_and_replaces_per_source() {
        let mut suche = Suche::in_memory().unwrap();
        let extrakt = super::super::extrakt::sample();
        suche.replace("fritzsch2019", &extrakt).unwrap();
        suche.replace("andere", &extrakt).unwrap();
        let hits = suche.search(Some("fritzsch2019"), &["decomposition".into(), "monolith".into()], 10).unwrap();
        assert_eq!(hits.len(), 3);
        assert!(hits.iter().all(|h| h.citekey == "fritzsch2019"));
        let data = hits.iter().find(|h| h.text.contains("data decomposition")).unwrap();
        assert_eq!((data.seite, data.label.as_str(), data.nr), (4, "3", 3));
        // Sonderzeichen in Suchwörtern sind keine FTS5-Operatoren.
        assert!(suche.search(None, &["Monolith-Ablösung:".into(), "\"".into()], 5).is_ok());
        suche.replace("fritzsch2019", &extrakt).unwrap();
        assert_eq!(suche.count("fritzsch2019").unwrap(), 4);
    }
}
