//! Typst-World für einen Arbeits-Vault.
//!
//! Bleibt zwischen Kompilierungen erhalten, damit Typst (comemo) Zwischenergebnisse
//! wiederverwendet. Quelltexte werden im Speicher gehalten und können ohne Speichern
//! geändert werden (`set_source_text`).
//!
//! Pakete kommen nur aus den lokalen Verzeichnissen (`@local` und Cache), nie aus dem
//! Netz (Leitplanke 4 in CLAUDE.md).

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::{Library, LibraryExt, World};
use typst_kit::datetime::Time;
use typst_kit::downloader::Downloader;
use typst_kit::files::FsRoot;
use typst_kit::fonts::FontStore;
use typst_kit::packages::{FsPackages, SystemPackages, UniversePackages};
use typst_utils::LazyHash;

/// Lehnt jeden Download ab: fehlende Pakete führen zu einem Fehler statt zu Netzverkehr.
struct OfflineDownloader;

impl Downloader for OfflineDownloader {
    fn stream(
        &self,
        _key: &dyn std::any::Any,
        url: &str,
    ) -> io::Result<(Option<usize>, Box<dyn io::Read>)> {
        Err(io::Error::other(format!(
            "Paket nicht lokal vorhanden, Download deaktiviert: {url}"
        )))
    }
}

pub struct SatzWorld {
    root: FsRoot,
    main: FileId,
    library: LazyHash<Library>,
    fonts: FontStore,
    packages: SystemPackages,
    time: Time,
    sources: Mutex<HashMap<FileId, Source>>,
    files: Mutex<HashMap<FileId, Bytes>>,
}

impl SatzWorld {
    /// `root`: Arbeits-Vault (entspricht `typst compile --root`),
    /// `main`: Pfad der Hauptdatei relativ zu `root`, z. B. `Arbeit/main.typ`.
    pub fn new(root: impl Into<PathBuf>, main: &str) -> Result<Self, String> {
        // Wie `typst compile`: Systemschriften und die mit Typst gelieferten Schriften.
        // Sonst fehlen z. B. Helvetica/Arial in SVG-Abbildungen und das PDF weicht ab.
        let mut fonts = FontStore::new();
        fonts.extend(typst_kit::fonts::system());
        fonts.extend(typst_kit::fonts::embedded());
        let packages = SystemPackages::from_parts(
            FsPackages::system_data(),
            FsPackages::system_cache(),
            UniversePackages::new(OfflineDownloader),
        );
        Ok(Self {
            root: FsRoot::new(root.into()),
            main: project_file(main)?,
            library: LazyHash::new(Library::builder().build()),
            fonts,
            packages,
            time: Time::system(),
            sources: Mutex::new(HashMap::new()),
            files: Mutex::new(HashMap::new()),
        })
    }

    /// Ersetzt den Quelltext einer Projektdatei im Speicher (inkrementelles Neuparsen).
    pub fn set_source_text(&self, path: &str, text: &str) -> Result<(), String> {
        let id = project_file(path)?;
        let mut sources = self.sources.lock().unwrap();
        match sources.get_mut(&id) {
            Some(source) => {
                source.replace(text);
            }
            None => {
                sources.insert(id, Source::new(id, text.to_owned()));
            }
        }
        Ok(())
    }

    /// Nach einer Änderung von außen: Datei beim nächsten Setzen neu von der Platte lesen.
    /// Quelltexte werden inkrementell ersetzt (schnelles Neusetzen), andere Dateien
    /// (z. B. `quellen.bib`, Bilder) aus dem Zwischenspeicher entfernt.
    pub fn reload_from_disk(&self, path: &str) -> Result<(), String> {
        let id = project_file(path)?;
        self.files.lock().unwrap().remove(&id);
        let mut sources = self.sources.lock().unwrap();
        if let Some(source) = sources.get_mut(&id) {
            match self.root.load(id.vpath()) {
                Ok(bytes) => {
                    let text = std::str::from_utf8(&bytes).map_err(|_| format!("{path}: kein UTF-8"))?;
                    source.replace(text);
                }
                // Gelöscht: beim nächsten Zugriff meldet Typst den Fehler.
                Err(_) => {
                    sources.remove(&id);
                }
            }
        }
        Ok(())
    }

    /// Aktueller Quelltext einer Projektdatei (aus dem Speicher oder von der Platte).
    pub fn source_text(&self, path: &str) -> Result<String, String> {
        let id = project_file(path)?;
        self.source(id)
            .map(|s| s.text().to_owned())
            .map_err(|e| format!("{e:?}"))
    }

    /// Setzt die Datumsquelle zurück; vor jeder Kompilierung aufrufen.
    pub fn reset_time(&mut self) {
        self.time.reset();
    }

    fn load(&self, id: FileId) -> FileResult<Bytes> {
        match id.root() {
            VirtualRoot::Project => self.root.load(id.vpath()),
            VirtualRoot::Package(spec) => self.packages.obtain(spec)?.load(id.vpath()),
        }
    }
}

pub fn project_file(path: &str) -> Result<FileId, String> {
    let vpath = VirtualPath::new(path).map_err(|e| format!("Ungültiger Pfad {path}: {e:?}"))?;
    Ok(FileId::new(RootedPath::new(VirtualRoot::Project, vpath)))
}

impl World for SatzWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let mut sources = self.sources.lock().unwrap();
        if let Some(source) = sources.get(&id) {
            return Ok(source.clone());
        }
        let bytes = self.load(id)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| FileError::InvalidUtf8)?;
        let source = Source::new(id, text.to_owned());
        sources.insert(id, source.clone());
        Ok(source)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let mut files = self.files.lock().unwrap();
        if let Some(bytes) = files.get(&id) {
            return Ok(bytes.clone());
        }
        let bytes = self.load(id)?;
        files.insert(id, bytes.clone());
        Ok(bytes)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}
