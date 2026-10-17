//! PDF → Abschnitte mit Seite → FTS5, Quellenseite.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "ingest";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "ingest");
    }
}
