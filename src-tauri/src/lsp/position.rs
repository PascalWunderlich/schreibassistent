//! Umrechnung von Positionen: Byte (Rust/Typst) ↔ UTF-16 (CodeMirror) ↔ LSP (Zeile, UTF-16-Spalte).

/// Byte-Position im Text → (Zeile, UTF-16-Spalte), beide 0-basiert.
pub fn byte_to_lsp(text: &str, byte: usize) -> (u32, u32) {
    let byte = byte.min(text.len());
    let before = &text[..floor_char_boundary(text, byte)];
    let line = before.matches('\n').count() as u32;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let character = before[line_start..].encode_utf16().count() as u32;
    (line, character)
}

/// (Zeile, UTF-16-Spalte) → Byte-Position; über das Zeilenende hinaus wird begrenzt.
pub fn lsp_to_byte(text: &str, (line, character): (u32, u32)) -> usize {
    let mut offset = 0;
    for _ in 0..line {
        match text[offset..].find('\n') {
            Some(i) => offset += i + 1,
            None => return text.len(),
        }
    }
    let line_text = &text[offset..text[offset..].find('\n').map_or(text.len(), |i| offset + i)];
    let mut units = 0;
    for (i, c) in line_text.char_indices() {
        if units >= character as usize {
            return offset + i;
        }
        units += c.len_utf16();
    }
    offset + line_text.len()
}

/// UTF-16-Index im Feld → Byte-Index im Feld.
pub fn field_byte(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (i, c) in text.char_indices() {
        if units >= utf16 {
            return i;
        }
        units += c.len_utf16();
    }
    text.len()
}

/// Byte-Index im Feld → UTF-16-Index im Feld.
pub fn field_utf16(text: &str, byte: usize) -> usize {
    text[..floor_char_boundary(text, byte.min(text.len()))].encode_utf16().count()
}

fn floor_char_boundary(text: &str, mut byte: usize) -> usize {
    while !text.is_char_boundary(byte) {
        byte -= 1;
    }
    byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_with_umlauts_and_emoji() {
        let text = "Größe\nÄpfel 🍎 und Birnen\n";
        let birnen = text.find("Birnen").unwrap();
        // 🍎 zählt in UTF-16 zwei Einheiten.
        assert_eq!(byte_to_lsp(text, birnen), (1, 13));
        assert_eq!(lsp_to_byte(text, (1, 13)), birnen);
        assert_eq!(lsp_to_byte(text, (0, 99)), text.find('\n').unwrap());
        assert_eq!(lsp_to_byte(text, (9, 0)), text.len());
        let field = "Ä🍎b";
        assert_eq!(field_byte(field, 3), "Ä🍎".len());
        assert_eq!(field_utf16(field, "Ä🍎".len()), 3);
    }
}
