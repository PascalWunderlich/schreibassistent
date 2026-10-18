//! KI-Anbieter über die genai-Crate: Ollama (lokal), OpenRouter, Anthropic und beliebige
//! OpenAI-kompatible Dienste. Netzwerk nur zum gewählten Anbieter (Leitplanke 4).

use std::time::Duration;

use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};

use super::einstellungen::{Anbieter, KiSettings, Wahl};

/// Lokale Modelle brauchen beim ersten Laden länger.
const TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Antwort {
    pub text: String,
    pub tokens_ein: Option<u32>,
    pub tokens_aus: Option<u32>,
}

fn with_slash(url: &str) -> String {
    if url.ends_with('/') { url.to_owned() } else { format!("{url}/") }
}

fn target(wahl: &Wahl, settings: &KiSettings, key: Option<String>) -> Result<(AdapterKind, Endpoint, AuthData), String> {
    let auth = match (wahl.anbieter.needs_key(), key) {
        // Ollama prüft keinen Schlüssel, die OpenAI-Schnittstelle will aber einen.
        (false, _) => AuthData::from_single("ollama"),
        (true, Some(key)) => AuthData::from_single(key),
        (true, None) => return Err(format!("Kein API-Schlüssel für {} (KI-Einstellungen)", wahl.anbieter.id())),
    };
    let (kind, url) = match wahl.anbieter {
        // Über Ollamas OpenAI-kompatible Schnittstelle: nur dort lässt sich das Nachdenken
        // abschalten (`reasoning_effort`), sonst braucht qwen3 für eine Frage ~100 s statt ~10 s.
        Anbieter::Ollama => (AdapterKind::OpenAI, format!("{}v1/", with_slash(&settings.ollama_url))),
        Anbieter::Openrouter => (AdapterKind::OpenRouter, "https://openrouter.ai/api/v1/".to_owned()),
        Anbieter::Deepseek => (AdapterKind::DeepSeek, "https://api.deepseek.com/v1/".to_owned()),
        Anbieter::Anthropic => (AdapterKind::Anthropic, "https://api.anthropic.com/v1/".to_owned()),
        Anbieter::Openai => {
            if settings.openai_url.trim().is_empty() {
                return Err("Keine Adresse für den OpenAI-kompatiblen Dienst (KI-Einstellungen)".into());
            }
            (AdapterKind::OpenAI, with_slash(settings.openai_url.trim()))
        }
    };
    Ok((kind, Endpoint::from_owned(url), auth))
}

/// Eine Anfrage: Systemprompt + Nachricht, Antwort als Text (Denkanteile wie `<think>` entfernt).
pub async fn ask(wahl: &Wahl, settings: &KiSettings, key: Option<String>, system: &str, message: &str) -> Result<Antwort, String> {
    if wahl.modell.trim().is_empty() {
        return Err(format!("Kein Modell für {} gewählt (KI-Einstellungen)", wahl.anbieter.id()));
    }
    let (kind, endpoint, auth) = target(wahl, settings, key)?;
    let model = wahl.modell.trim().to_owned();
    let resolver = ServiceTargetResolver::from_resolver_fn(move |_: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
        Ok(ServiceTarget { endpoint: endpoint.clone(), auth: auth.clone(), model: ModelIden::new(kind, model.clone()) })
    });
    let client = Client::builder()
        .with_service_target_resolver(resolver)
        .with_web_config(genai::WebConfig::default().with_timeout(TIMEOUT))
        .build();
    let request = ChatRequest::new(vec![ChatMessage::user(message)]).with_system(system);
    let mut options = ChatOptions::default().with_normalize_reasoning_content(true).with_capture_usage(true);
    if wahl.anbieter == Anbieter::Ollama {
        options = options.with_extra_body(serde_json::json!({ "reasoning_effort": "none" }));
    }
    let response = client
        .exec_chat(wahl.modell.trim(), request, Some(&options))
        .await
        .map_err(|e| format!("{}: {e}", wahl.anbieter.id()))?;
    let text = response.first_text().unwrap_or_default().trim().to_owned();
    let text = strip_think(&text);
    if text.is_empty() {
        return Err(format!("{}: leere Antwort", wahl.anbieter.id()));
    }
    Ok(Antwort {
        text,
        tokens_ein: response.usage.prompt_tokens.map(|t| t as u32),
        tokens_aus: response.usage.completion_tokens.map(|t| t as u32),
    })
}

/// Falls ein Modell seinen Denkteil doch im Text liefert: `<think>…</think>` entfernen.
fn strip_think(text: &str) -> String {
    match (text.find("<think>"), text.find("</think>")) {
        (Some(start), Some(end)) if start < end => format!("{}{}", &text[..start], &text[end + "</think>".len()..]).trim().to_owned(),
        _ => text.to_owned(),
    }
}

/// Installierte Ollama-Modelle (leer, wenn Ollama nicht läuft).
pub fn ollama_models(url: &str) -> Vec<String> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_millis(800))).build().into();
    let Ok(mut response) = agent.get(format!("{}api/tags", with_slash(url))).call() else { return Vec::new() };
    let Ok(body) = response.body_mut().read_json::<serde_json::Value>() else { return Vec::new() };
    let mut names: Vec<String> =
        body["models"].as_array().into_iter().flatten().filter_map(|m| m["name"].as_str().map(str::to_owned)).collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn think_block_is_removed() {
        assert_eq!(strip_think("<think>\nhm\n</think>\n\n- Hinweis"), "- Hinweis");
        assert_eq!(strip_think("- Hinweis"), "- Hinweis");
    }

    #[test]
    fn missing_key_or_model_is_reported_without_network() {
        let settings = KiSettings::default();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let wahl = Wahl { anbieter: Anbieter::Openrouter, modell: "anthropic/claude-sonnet-5.5".into() };
        let err = rt.block_on(ask(&wahl, &settings, None, "s", "m")).unwrap_err();
        assert!(err.contains("Kein API-Schlüssel"), "{err}");
        let wahl = Wahl { anbieter: Anbieter::Deepseek, modell: Anbieter::Deepseek.default_model().into() };
        assert!(rt.block_on(ask(&wahl, &settings, None, "s", "m")).unwrap_err().contains("Kein API-Schlüssel für deepseek"));
        let wahl = Wahl { anbieter: Anbieter::Ollama, modell: " ".into() };
        assert!(rt.block_on(ask(&wahl, &settings, None, "s", "m")).unwrap_err().contains("Kein Modell"));
        let wahl = Wahl { anbieter: Anbieter::Openai, modell: "x".into() };
        assert!(rt.block_on(ask(&wahl, &settings, Some("k".into()), "s", "m")).unwrap_err().contains("Adresse"));
    }

    #[test]
    fn ollama_absent_gives_no_models() {
        assert!(ollama_models("http://127.0.0.1:9/").is_empty());
    }
}
