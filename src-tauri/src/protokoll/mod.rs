//! KI-Protokoll schreiben, Export als Typst-Tabelle.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "protokoll";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "protokoll");
    }
}
