//! Typst-World, Kompilieren, SVG je Seite, PDF, Positions-Mapping.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "satz";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "satz");
    }
}
