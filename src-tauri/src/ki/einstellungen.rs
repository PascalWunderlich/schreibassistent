//! KI-Einstellungen: Anbieter und Modell je Aufgabe. Liegen in den App-Einstellungen
//! (nicht in der Arbeit); API-Schlüssel nie hier, sondern im Schlüsselbund (`schluessel`).

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aufgabe {
    Frage,
    Belegpruefung,
    Feedback,
    Ingest,
    Regelprofil,
}

impl Aufgabe {
    pub const ALL: [Aufgabe; 5] = [Aufgabe::Frage, Aufgabe::Belegpruefung, Aufgabe::Feedback, Aufgabe::Ingest, Aufgabe::Regelprofil];

    pub fn id(self) -> &'static str {
        match self {
            Aufgabe::Frage => "frage",
            Aufgabe::Belegpruefung => "belegpruefung",
            Aufgabe::Feedback => "feedback",
            Aufgabe::Ingest => "ingest",
            Aufgabe::Regelprofil => "regelprofil",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Anbieter {
    Ollama,
    Openrouter,
    Deepseek,
    Anthropic,
    /// Beliebiger OpenAI-kompatibler Dienst (Adresse in `openai_url`).
    Openai,
}

impl Anbieter {
    pub const ALL: [Anbieter; 5] = [Anbieter::Ollama, Anbieter::Openrouter, Anbieter::Deepseek, Anbieter::Anthropic, Anbieter::Openai];

    pub fn id(self) -> &'static str {
        match self {
            Anbieter::Ollama => "ollama",
            Anbieter::Openrouter => "openrouter",
            Anbieter::Deepseek => "deepseek",
            Anbieter::Anthropic => "anthropic",
            Anbieter::Openai => "openai",
        }
    }

    /// Braucht einen API-Schlüssel? (Ollama läuft lokal ohne.)
    pub fn needs_key(self) -> bool {
        !matches!(self, Anbieter::Ollama)
    }

    /// Vorschlag, wenn man den Anbieter wählt.
    pub fn default_model(self) -> &'static str {
        match self {
            Anbieter::Ollama => "qwen3:8b",
            Anbieter::Openrouter => "anthropic/claude-sonnet-5.5",
            // Ohne Nachdenken: schnell und günstig.
            Anbieter::Deepseek => "deepseek-chat",
            Anbieter::Anthropic => "claude-sonnet-5-5",
            Anbieter::Openai => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Wahl {
    pub anbieter: Anbieter,
    pub modell: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct KiSettings {
    pub aufgaben: BTreeMap<Aufgabe, Wahl>,
    /// Adresse des OpenAI-kompatiblen Dienstes, z. B. `https://api.example.com/v1/`.
    pub openai_url: String,
    pub ollama_url: String,
}

pub const OLLAMA_URL: &str = "http://localhost:11434/";

impl Default for KiSettings {
    /// Ollama für alles: läuft lokal ohne Schlüssel und Konto.
    fn default() -> Self {
        let wahl = Wahl { anbieter: Anbieter::Ollama, modell: Anbieter::Ollama.default_model().to_owned() };
        Self {
            aufgaben: Aufgabe::ALL.iter().map(|a| (*a, wahl.clone())).collect(),
            openai_url: String::new(),
            ollama_url: OLLAMA_URL.to_owned(),
        }
    }
}

impl KiSettings {
    pub fn wahl(&self, aufgabe: Aufgabe) -> Wahl {
        self.aufgaben.get(&aufgabe).cloned().unwrap_or_else(|| Self::default().aufgaben[&aufgabe].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_without_ki_get_defaults() {
        let settings: crate::projekt::settings::Settings = serde_json::from_str(r#"{"last_vault": "/x"}"#).unwrap();
        assert_eq!(settings.ki, KiSettings::default());
        assert_eq!(settings.ki.wahl(Aufgabe::Frage).modell, "qwen3:8b");
    }

    #[test]
    fn partial_settings_fall_back_per_task() {
        let ki: KiSettings = serde_json::from_str(r#"{"aufgaben": {"frage": {"anbieter": "openrouter", "modell": "anthropic/claude-sonnet-5.5"}}}"#).unwrap();
        assert_eq!(ki.wahl(Aufgabe::Frage).anbieter, Anbieter::Openrouter);
        assert_eq!(ki.wahl(Aufgabe::Ingest).anbieter, Anbieter::Ollama);
        assert_eq!(ki.ollama_url, OLLAMA_URL);
    }
}
