//! Positions-Mapping: Stelle auf einer Seite ↔ Absatz im Quelltext.
//!
//! Jeder gesetzte Text trägt einen Typst-Span, der auf seinen Quelltext zeigt.
//! Vom Klickpunkt geht es über den Span zur Byte-Position und von dort im Syntaxbaum
//! zum umgebenden Absatz (Bereich zwischen zwei Leerzeilen).

use std::ops::Range;

use typst::WorldExt;
use typst::layout::{Frame, FrameItem, Point, Transform};
use typst::syntax::{FileId, LinkedNode, Side, Source, SyntaxKind, VirtualRoot};

use super::SatzWorld;

/// Rechteck in pt, Ursprung oben links auf der Seite.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn union(self, other: Rect) -> Rect {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.width).max(other.x + other.width);
        let y1 = (self.y + self.height).max(other.y + other.height);
        Rect { x: x0, y: y0, width: x1 - x0, height: y1 - y0 }
    }

    fn vertical_distance(&self, y: f64) -> f64 {
        if y < self.y { self.y - y } else if y > self.y + self.height { y - self.y - self.height } else { 0.0 }
    }
}

/// Ein gesetztes Textstück mit seiner Herkunft im Quelltext.
#[derive(Debug, Clone)]
pub struct TextBox {
    pub rect: Rect,
    pub file: FileId,
    pub range: Range<usize>,
}

/// Alle Textstücke einer Seite, die aus Projektdateien stammen (nicht aus Paketen).
pub fn text_boxes(world: &SatzWorld, frame: &Frame) -> Vec<TextBox> {
    let mut out = Vec::new();
    collect(world, frame, Transform::identity(), &mut out);
    out
}

fn collect(world: &SatzWorld, frame: &Frame, ts: Transform, out: &mut Vec<TextBox>) {
    for (pos, item) in frame.items() {
        match item {
            FrameItem::Group(group) => {
                let ts = ts.pre_concat(Transform::translate(pos.x, pos.y)).pre_concat(group.transform);
                collect(world, &group.frame, ts, out);
            }
            FrameItem::Text(text) => {
                let spans = text.glyphs.iter().map(|g| g.span.0).filter(|s| !s.is_detached());
                let mut file = None;
                let mut range: Option<Range<usize>> = None;
                for span in spans {
                    let Some(id) = span.id() else { continue };
                    if !matches!(id.root(), VirtualRoot::Project) || file.is_some_and(|f| f != id) {
                        continue;
                    }
                    let Some(r) = world.range(span) else { continue };
                    file = Some(id);
                    range = Some(match range {
                        Some(prev) => prev.start.min(r.start)..prev.end.max(r.end),
                        None => r,
                    });
                }
                let (Some(file), Some(range)) = (file, range) else { continue };
                let metrics = text.font.metrics();
                let top = pos.y - metrics.ascender.at(text.size);
                let bottom = pos.y - metrics.descender.at(text.size);
                let a = Point::new(pos.x, top).transform(ts);
                let b = Point::new(pos.x + text.width(), bottom).transform(ts);
                let rect = Rect {
                    x: a.x.to_pt().min(b.x.to_pt()),
                    y: a.y.to_pt().min(b.y.to_pt()),
                    width: (b.x - a.x).to_pt().abs(),
                    height: (b.y - a.y).to_pt().abs(),
                };
                out.push(TextBox { rect, file, range });
            }
            _ => {}
        }
    }
}

/// Textstück an (x, y) in pt; ohne Treffer das vertikal nächste innerhalb von `tolerance` pt.
pub fn box_at(boxes: &[TextBox], x: f64, y: f64, tolerance: f64) -> Option<&TextBox> {
    if let Some(hit) = boxes.iter().find(|b| b.rect.contains(x, y)) {
        return Some(hit);
    }
    boxes
        .iter()
        .map(|b| (b.rect.vertical_distance(y), b))
        .filter(|(d, _)| *d <= tolerance)
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, b)| b)
}

/// Absatz um `offset`: zusammenhängende Elemente der Markup-Ebene zwischen Leerzeilen.
/// Überschriften, Listenpunkte und Abbildungen gelten als eigener Absatz.
/// Führende und abschließende Leerzeichen und Kommentare gehören nicht dazu.
pub fn paragraph_range(source: &Source, offset: usize) -> Option<Range<usize>> {
    let root = LinkedNode::new(source.root());
    let leaf = root.leaf_at(offset, Side::After)?;

    // Bis zum Kind eines Markup-Knotens auf Absatzebene hochsteigen: Markup der Datei
    // oder eines Inhaltsblocks `[…]` (z. B. Abbildungsunterschrift), nicht das Innere
    // von `*fett*`, `_kursiv_` oder einer Überschrift.
    let mut node = leaf;
    while let Some(parent) = node.parent() {
        let block_level = parent.kind() == SyntaxKind::Markup
            && parent.parent().is_none_or(|p| p.kind() == SyntaxKind::ContentBlock);
        if block_level {
            break;
        }
        node = parent.clone();
    }
    if is_block(node.kind()) {
        return Some(node.range());
    }

    let mut first = node.clone();
    while let Some(prev) = first.prev_sibling_with_trivia() {
        if is_boundary(prev.kind()) {
            break;
        }
        first = prev;
    }
    let mut last = node;
    while let Some(next) = last.next_sibling_with_trivia() {
        if is_boundary(next.kind()) {
            break;
        }
        last = next;
    }
    while is_trivia(first.kind()) && first.range().start < last.range().start {
        first = first.next_sibling_with_trivia()?;
    }
    while is_trivia(last.kind()) && last.range().start > first.range().start {
        last = last.prev_sibling_with_trivia()?;
    }
    Some(first.range().start..last.range().end)
}

fn is_block(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Heading | SyntaxKind::ListItem | SyntaxKind::EnumItem | SyntaxKind::TermItem)
}

fn is_boundary(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::Parbreak || is_block(kind)
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Space | SyntaxKind::LineComment | SyntaxKind::BlockComment)
}

/// Umrandung eines Quelltextbereichs auf den Seiten: je Seite ein Rechteck.
pub fn rects_for_range(
    page_boxes: &[(usize, Vec<TextBox>)],
    file: FileId,
    range: &Range<usize>,
) -> Vec<(usize, Rect)> {
    let mut out: Vec<(usize, Rect)> = Vec::new();
    for (page, boxes) in page_boxes {
        let hits = boxes
            .iter()
            .filter(|b| b.file == file && b.range.start < range.end && b.range.end > range.start);
        if let Some(rect) = hits.map(|b| b.rect).reduce(Rect::union) {
            out.push((*page, rect));
        }
    }
    out
}

/// Größe einer Seite in pt.
pub fn page_size(frame: &Frame) -> (f64, f64) {
    let size = frame.size();
    (size.x.to_pt(), size.y.to_pt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::syntax::{FileId, RootedPath, VirtualPath};

    fn source(text: &str) -> Source {
        let id = FileId::new(RootedPath::new(VirtualRoot::Project, VirtualPath::new("t.typ").unwrap()));
        Source::new(id, text.to_owned())
    }

    fn para(text: &str, needle: &str) -> String {
        let src = source(text);
        let offset = text.find(needle).unwrap();
        let range = paragraph_range(&src, offset).unwrap();
        text[range].to_owned()
    }

    #[test]
    fn paragraph_between_blank_lines() {
        let text = "= Titel\n\nErster Absatz mit *fett* und @ref.\nzweite Zeile.\n\nZweiter Absatz.\n";
        assert_eq!(para(text, "fett"), "Erster Absatz mit *fett* und @ref.\nzweite Zeile.");
        assert_eq!(para(text, "Zweiter"), "Zweiter Absatz.");
    }

    #[test]
    fn heading_is_its_own_block_and_keeps_label_outside() {
        // Das Label bleibt außerhalb des Editierfelds und kann nicht versehentlich verloren gehen.
        let text = "= Titel <kap:a>\n\nText.\n";
        assert_eq!(para(text, "Titel"), "= Titel");
    }

    #[test]
    fn code_call_belongs_to_paragraph_and_comments_are_excluded() {
        let text = "Vorher.\n\n// Kommentar\n#lorem(20) und Text.\n\nNachher.\n";
        assert_eq!(para(text, "lorem"), "#lorem(20) und Text.");
    }
}
