//! Cache des Index in `.schreibassistent/cache.sqlite`: je Datei Prüfsumme und Ergebnis
//! als JSON. Nur zur Beschleunigung, jederzeit löschbar (Leitplanke 6).

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};

/// Bei Änderungen am Format der gespeicherten Ergebnisse erhöhen: alter Cache wird verworfen.
const VERSION: i64 = 2;

pub struct Cache {
    db: Mutex<Connection>,
}

fn hash(text: &str) -> String {
    format!("{:032x}", typst_utils::hash128(text.as_bytes()))
}

impl Cache {
    pub fn open(file: &Path) -> Result<Self, String> {
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let db = Connection::open(file).map_err(|e| format!("Cache {}: {e}", file.display()))?;
        let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(|e| e.to_string())?;
        if version != VERSION {
            db.execute_batch(&format!("DROP TABLE IF EXISTS files; PRAGMA user_version = {VERSION};"))
                .map_err(|e| e.to_string())?;
        }
        db.execute_batch("CREATE TABLE IF NOT EXISTS files (path TEXT PRIMARY KEY, hash TEXT NOT NULL, data TEXT NOT NULL)")
            .map_err(|e| e.to_string())?;
        Ok(Self { db: Mutex::new(db) })
    }

    /// Gespeichertes Ergebnis, falls die Datei seitdem unverändert ist.
    pub fn get<T: DeserializeOwned>(&self, path: &str, text: &str) -> Option<T> {
        let db = self.db.lock().unwrap();
        let row: Option<(String, String)> = db
            .query_row("SELECT hash, data FROM files WHERE path = ?1", params![path], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()
            .ok()?;
        let (stored, data) = row?;
        (stored == hash(text)).then(|| serde_json::from_str(&data).ok()).flatten()
    }

    pub fn put<T: Serialize>(&self, path: &str, text: &str, value: &T) -> Result<(), String> {
        let data = serde_json::to_string(value).map_err(|e| e.to_string())?;
        self.db
            .lock()
            .unwrap()
            .execute("INSERT OR REPLACE INTO files (path, hash, data) VALUES (?1, ?2, ?3)", params![path, hash(text), data])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Einträge gelöschter Dateien entfernen.
    pub fn retain(&self, paths: &[String]) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        let stored: Vec<String> = db
            .prepare("SELECT path FROM files")
            .and_then(|mut s| s.query_map([], |r| r.get(0))?.collect())
            .map_err(|e| e.to_string())?;
        for path in stored.iter().filter(|p| !paths.contains(p)) {
            db.execute("DELETE FROM files WHERE path = ?1", params![path]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn len(&self) -> Result<usize, String> {
        let db = self.db.lock().unwrap();
        db.query_row("SELECT COUNT(*) FROM files", [], |r| r.get::<_, i64>(0)).map(|n| n as usize).map_err(|e| e.to_string())
    }
}
