//! Arbeits-Vault öffnen, Dateien beobachten, Git-Sicherung.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "projekt";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "projekt");
    }
}
