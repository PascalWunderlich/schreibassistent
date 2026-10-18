//! App-Einstellungen als Klartext-JSON im Konfigurationsordner der App.

use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Settings {
    /// Zuletzt geöffneter Arbeits-Vault.
    pub last_vault: Option<PathBuf>,
    /// Anbieter und Modell je KI-Aufgabe (ohne Schlüssel).
    #[serde(default)]
    pub ki: crate::ki::einstellungen::KiSettings,
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
}
