//! Kapitel ↔ Citekey ↔ Wissensseite ↔ Notiz, SQLite-Cache.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "index";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "index");
    }
}
