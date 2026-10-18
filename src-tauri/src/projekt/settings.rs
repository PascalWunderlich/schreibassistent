//! App-Einstellungen als Klartext-JSON im Konfigurationsordner der App.

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Settings {
    /// Zuletzt geöffneter Arbeits-Vault.
    pub last_vault: Option<PathBuf>,
    /// Anbieter und Modell je KI-Aufgabe (ohne Schlüssel).
    #[serde(default)]
    pub ki: crate::ki::einstellungen::KiSettings,
    /// Der Ersteinstieg (Arbeit, Quellen, KI) wurde durchlaufen oder übersprungen.
    #[serde(default)]
    pub ersteinstieg_fertig: bool,
}

impl Settings {
    /// Ersteinstieg zeigen? Nur beim allerersten Start: wer schon eine Arbeit geöffnet hatte, kennt die App.
    pub fn ersteinstieg_offen(&self) -> bool {
        !self.ersteinstieg_fertig && self.last_vault.is_none()
    }
}

pub fn load(file: &Path) -> Settings {
    std::fs::read_to_string(file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save(file: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    super::files::atomic_write(file, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_missing_file() {
        let file = std::env::temp_dir().join(format!("sa-settings-{}/settings.json", std::process::id()));
        assert_eq!(load(&file), Settings::default());
        let settings = Settings { last_vault: Some("/Users/x/Arbeiten/Testarbeit".into()), ..Default::default() };
        save(&file, &settings).unwrap();
        assert_eq!(load(&file), settings);
        std::fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }

    #[test]
    fn first_run_shows_onboarding_once() {
        let mut settings = Settings::default();
        assert!(settings.ersteinstieg_offen());
        settings.ersteinstieg_fertig = true;
        assert!(!settings.ersteinstieg_offen());
    }

    #[test]
    fn existing_users_and_old_files_skip_onboarding() {
        // Datei aus einer früheren Version: kein Feld `ersteinstieg_fertig`, aber eine zuletzt geöffnete Arbeit.
        let old: Settings = serde_json::from_str(r#"{"last_vault": "/x/Arbeit"}"#).unwrap();
        assert!(!old.ersteinstieg_offen());
        let empty: Settings = serde_json::from_str("{}").unwrap();
        assert!(empty.ersteinstieg_offen());
    }
}
