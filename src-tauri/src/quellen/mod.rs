//! quellen.bib lesen, Zotero Local API (localhost:23119).
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "quellen";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "quellen");
    }
}
