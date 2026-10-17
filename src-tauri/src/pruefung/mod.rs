//! Regelprofil laden, feste Prüfungen, Vale/LTeX+ einbinden.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "pruefung";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "pruefung");
    }
}
