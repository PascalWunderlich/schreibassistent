//! Anbieter (genai), Rechte, Aufgaben, Kontextbau.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "ki";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "ki");
    }
}
