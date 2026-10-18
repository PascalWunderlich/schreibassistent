//! API-Schlüssel im macOS-Schlüsselbund (Leitplanke 7): Dienst „schreibassistent“,
//! Konto = Anbieter. Die Oberfläche kann Schlüssel setzen und löschen und erfährt nur,
//! ob einer gesetzt ist – lesen kann ihn nur das Backend für die Anfrage.

use super::einstellungen::Anbieter;

const SERVICE: &str = "schreibassistent";

fn entry(service: &str, anbieter: Anbieter) -> Result<keyring::Entry, String> {
    keyring::Entry::new(service, anbieter.id()).map_err(|e| format!("Schlüsselbund: {e}"))
}

fn get_in(service: &str, anbieter: Anbieter) -> Result<Option<String>, String> {
    match entry(service, anbieter)?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Schlüsselbund: {e}")),
    }
}

fn set_in(service: &str, anbieter: Anbieter, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Leerer Schlüssel".into());
    }
    entry(service, anbieter)?.set_password(key).map_err(|e| format!("Schlüsselbund: {e}"))
}

fn delete_in(service: &str, anbieter: Anbieter) -> Result<(), String> {
    match entry(service, anbieter)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Schlüsselbund: {e}")),
    }
}

pub fn get(anbieter: Anbieter) -> Result<Option<String>, String> {
    get_in(SERVICE, anbieter)
}

pub fn set(anbieter: Anbieter, key: &str) -> Result<(), String> {
    set_in(SERVICE, anbieter, key)
}

pub fn delete(anbieter: Anbieter) -> Result<(), String> {
    delete_in(SERVICE, anbieter)
}

pub fn is_set(anbieter: Anbieter) -> bool {
    get(anbieter).ok().flatten().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Echter Schlüsselbund, eigener Testdienst. Kann beim ersten Lauf nach Erlaubnis fragen.
    #[test]
    fn set_get_delete_in_keychain() {
        let service = "schreibassistent-test";
        let result = set_in(service, Anbieter::Openai, "  sk-test-123  ");
        if let Err(e) = &result {
            eprintln!("Schlüsselbund nicht verfügbar, übersprungen: {e}");
            return;
        }
        assert_eq!(get_in(service, Anbieter::Openai).unwrap().as_deref(), Some("sk-test-123"));
        delete_in(service, Anbieter::Openai).unwrap();
        assert_eq!(get_in(service, Anbieter::Openai).unwrap(), None);
        delete_in(service, Anbieter::Openai).unwrap();
        assert!(set_in(service, Anbieter::Openai, " ").is_err());
    }
}
