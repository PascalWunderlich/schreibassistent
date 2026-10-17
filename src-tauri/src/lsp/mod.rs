//! Tinymist starten, LSP zwischen CodeMirror und Tinymist.
//!
//! Noch leer (P0); siehe docs/ARCHITECTURE.md.

pub const NAME: &str = "lsp";

#[cfg(test)]
mod tests {
    #[test]
    fn module_is_registered() {
        assert_eq!(super::NAME, "lsp");
    }
}
