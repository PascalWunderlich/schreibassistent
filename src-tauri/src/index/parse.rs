//! Dateien auswerten: Überschriften und Zitate aus `.typ` (Typst-Parser), Frontmatter
//! und `[[Links]]` aus `.md`.

use typst::syntax::{LinkedNode, SyntaxKind};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Heading {
    pub level: usize,
    pub text: String,
    /// Byte-Position im Quelltext.
    pub offset: usize,
}

/// `@key`, `@key[…]` oder `#cite(<key>, supplement: […])`. Ob es eine Quelle oder ein
/// Label ist, entscheidet erst der Abgleich mit der `.bib`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Reference {
    pub key: String,
    /// Ergänzung wie „S. 14“.
    pub supplement: Option<String>,
    pub offset: usize,
    /// Steht in einer Fußnote (FHDW S. 9: keine Fußnoten für Zitate).
    #[serde(default)]
    pub in_footnote: bool,
}

/// `#figure(…)` im Quelltext.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Figure {
    pub offset: usize,
    /// "abbildung" (`image`), "tabelle" (`table`), "listing" (Code-Block) oder `None`.
    pub kind: Option<String>,
    /// Byte-Bereiche von Inhalt und Beschriftung im Quelltext.
    pub body: (usize, usize),
    pub caption: Option<(usize, usize)>,
    /// Label direkt nach der Abbildung, z. B. `abb:architektur`.
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypFile {
    pub headings: Vec<Heading>,
    pub references: Vec<Reference>,
    /// Im Text definierte Labels (`<name>`).
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub figures: Vec<Figure>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MdFile {
    pub title: Option<String>,
    pub author: Option<String>,
    /// Frontmatter `kapitel: [02-grundlagen]` (Dateiname ohne `.typ`).
    pub kapitel: Vec<String>,
    /// Frontmatter `citekeys: […]` und `citekey: …`.
    pub citekeys: Vec<String>,
    /// Ziele von `[[Links]]`.
    pub links: Vec<String>,
}

pub fn parse_typ(text: &str) -> TypFile {
    let root = typst::syntax::parse(text);
    let mut out = TypFile::default();
    walk(&LinkedNode::new(&root), text, false, &mut out);
    out
}

/// Name der aufgerufenen Funktion, z. B. `figure` bei `#figure(…)`.
fn callee<'a>(node: &LinkedNode, text: &'a str) -> Option<&'a str> {
    let first = node.children().next()?;
    (first.kind() == SyntaxKind::Ident).then(|| &text[first.range()])
}

fn walk(node: &LinkedNode, text: &str, in_footnote: bool, out: &mut TypFile) {
    let mut in_footnote = in_footnote;
    match node.kind() {
        SyntaxKind::Label => {
            out.labels.push(text[node.range()].trim_start_matches('<').trim_end_matches('>').to_owned());
        }
        SyntaxKind::Heading => {
            let level = node.children().find(|c| c.kind() == SyntaxKind::HeadingMarker).map_or(1, |m| m.range().len());
            let body = node.children().find(|c| c.kind() == SyntaxKind::Markup).map(|m| text[m.range()].trim().to_owned());
            out.headings.push(Heading { level, text: body.unwrap_or_default(), offset: node.offset() });
        }
        SyntaxKind::Ref => {
            let marker = node.children().find(|c| c.kind() == SyntaxKind::RefMarker);
            if let Some(marker) = marker {
                let supplement = node.children().find(|c| c.kind() == SyntaxKind::ContentBlock).map(|c| block_text(&c, text));
                out.references.push(Reference {
                    key: text[marker.range()][1..].to_owned(),
                    supplement,
                    offset: node.offset(),
                    in_footnote,
                });
            }
        }
        SyntaxKind::FuncCall if callee(node, text) == Some("footnote") => in_footnote = true,
        SyntaxKind::FuncCall if callee(node, text) == Some("figure") => {
            if let Some(figure) = parse_figure(node, text) {
                out.figures.push(figure);
            }
        }
        SyntaxKind::FuncCall => {
            if callee(node, text) == Some("cite")
                && let Some(args) = node.children().find(|c| c.kind() == SyntaxKind::Args)
            {
                let label = args.children().find(|c| c.kind() == SyntaxKind::Label);
                let supplement = args
                    .children()
                    .filter(|c| c.kind() == SyntaxKind::Named)
                    .find(|n| n.children().next().is_some_and(|i| &text[i.range()] == "supplement"))
                    .and_then(|n| n.children().find(|c| c.kind() == SyntaxKind::ContentBlock))
                    .map(|c| block_text(&c, text));
                if let Some(label) = label {
                    let key = text[label.range()].trim_start_matches('<').trim_end_matches('>').to_owned();
                    out.references.push(Reference { key, supplement, offset: node.offset(), in_footnote });
                }
                // Das Label in `cite(<key>)` ist ein Verweis, keine Definition.
                return;
            }
        }
        _ => {}
    }
    for child in node.children() {
        walk(&child, text, in_footnote, out);
    }
}

fn parse_figure(node: &LinkedNode, text: &str) -> Option<Figure> {
    let args = node.children().find(|c| c.kind() == SyntaxKind::Args)?;
    let mut body = None;
    let mut caption = None;
    for arg in args.children() {
        match arg.kind() {
            SyntaxKind::Named => {
                let name = arg.children().next().map(|n| &text[n.range()]);
                if name == Some("caption") {
                    let value = arg.children().filter(|c| !matches!(c.kind(), SyntaxKind::Ident | SyntaxKind::Colon | SyntaxKind::Space)).last()?;
                    caption = Some((value.range().start, value.range().end));
                }
            }
            SyntaxKind::LeftParen | SyntaxKind::RightParen | SyntaxKind::Comma | SyntaxKind::Space => {}
            _ if body.is_none() => body = Some(arg),
            _ => {}
        }
    }
    let body = body?;
    let kind = match body.kind() {
        SyntaxKind::FuncCall => match callee(&body, text) {
            Some("image") => Some("abbildung"),
            Some("table") => Some("tabelle"),
            _ => None,
        },
        SyntaxKind::Raw => Some("listing"),
        SyntaxKind::ContentBlock if text[body.range()].contains("```") => Some("listing"),
        _ => None,
    };
    // Label direkt dahinter (auf derselben Markup-Ebene, nach Leerraum).
    let mut figure_node = node.clone();
    while let Some(parent) = figure_node.parent() {
        if parent.kind() == SyntaxKind::Markup {
            break;
        }
        figure_node = parent.clone();
    }
    let mut label = None;
    let mut next = figure_node.next_sibling_with_trivia();
    while let Some(sibling) = next {
        match sibling.kind() {
            SyntaxKind::Space => next = sibling.next_sibling_with_trivia(),
            SyntaxKind::Label => {
                label = Some(text[sibling.range()].trim_start_matches('<').trim_end_matches('>').to_owned());
                break;
            }
            _ => break,
        }
    }
    Some(Figure {
        offset: node.offset(),
        kind: kind.map(str::to_owned),
        body: (body.range().start, body.range().end),
        caption,
        label,
    })
}

/// Text eines Inhaltsblocks `[…]` ohne Klammern.
fn block_text(node: &LinkedNode, text: &str) -> String {
    text[node.range()].trim().trim_start_matches('[').trim_end_matches(']').trim().to_owned()
}

pub fn parse_md(text: &str) -> MdFile {
    let mut out = MdFile::default();
    let mut body = text;
    if let Some(rest) = text.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        frontmatter(&rest[..end], &mut out);
        body = &rest[end + 4..];
    }
    out.title = body.lines().find_map(|l| l.strip_prefix("# ")).map(|t| t.trim().to_owned());
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        let target = after[..end].split(['|', '#']).next().unwrap_or_default().trim();
        if !target.is_empty() && !out.links.iter().any(|l| l == target) {
            out.links.push(target.to_owned());
        }
        rest = &after[end + 2..];
    }
    out
}

/// Kleiner YAML-Ausschnitt: `key: wert`, `key: [a, b]` und Listen mit `- a`.
fn frontmatter(yaml: &str, out: &mut MdFile) {
    let mut current: Option<String> = None;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if let Some(item) = trimmed.strip_prefix("- ") {
            if let Some(key) = &current {
                assign(out, key, vec![unquote(item)]);
            }
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else { continue };
        let (key, value) = (key.trim().to_owned(), value.trim());
        current = Some(key.clone());
        if value.is_empty() {
            continue;
        }
        let values = match value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            Some(list) => list.split(',').map(unquote).filter(|v| !v.is_empty()).collect(),
            None => vec![unquote(value)],
        };
        assign(out, &key, values);
    }
}

fn assign(out: &mut MdFile, key: &str, values: Vec<String>) {
    match key {
        "author" => out.author = values.into_iter().next(),
        "kapitel" => out.kapitel.extend(values),
        "citekeys" | "citekey" => out.citekeys.extend(values),
        _ => {}
    }
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches(['"', '\'']).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typst_headings_references_and_cite_calls() {
        let text = "= Grundlagen <kap:g>\n\n== Teil\n\nSiehe @sutton2018 und @greenyer2013[S. 1], vgl. @kap:g.\n#cite(<koenig2023>, supplement: [S. 7]) und #cite(<omg2017>).\n";
        let file = parse_typ(text);
        assert_eq!(file.headings.iter().map(|h| (h.level, h.text.as_str())).collect::<Vec<_>>(), [(1, "Grundlagen"), (2, "Teil")]);
        let refs: Vec<_> = file.references.iter().map(|r| (r.key.as_str(), r.supplement.as_deref())).collect();
        assert_eq!(
            refs,
            [("sutton2018", None), ("greenyer2013", Some("S. 1")), ("kap:g", None), ("koenig2023", Some("S. 7")), ("omg2017", None)]
        );
    }

    #[test]
    fn labels_footnotes_and_figures() {
        let text = "= A <kap:a>\n\nText#footnote[Siehe @sutton2018.] und @kap:a.\n\n#figure(\n  image(\"x.svg\"),\n  caption: [Bild],\n) <abb:x>\n\n#figure(table(columns: 1, [a]))\n";
        let file = parse_typ(text);
        assert_eq!(file.labels, ["kap:a", "abb:x"]);
        let refs: Vec<_> = file.references.iter().map(|r| (r.key.as_str(), r.in_footnote)).collect();
        assert_eq!(refs, [("sutton2018", true), ("kap:a", false)]);
        assert_eq!(file.figures.len(), 2);
        let (a, b) = (&file.figures[0], &file.figures[1]);
        assert_eq!((a.kind.as_deref(), a.label.as_deref()), (Some("abbildung"), Some("abb:x")));
        assert_eq!(&text[a.caption.unwrap().0..a.caption.unwrap().1], "[Bild]");
        assert_eq!((b.kind.as_deref(), b.caption, b.label.as_deref()), (Some("tabelle"), None, None));
    }

    #[test]
    fn markdown_frontmatter_and_links() {
        let text = "---\nauthor: ich\nkapitel: [02-grundlagen, \"05-zusammenfassung\"]\ncitekeys:\n  - sutton2018\n  - gebhard2021\n---\n\n# Titel\n\nSiehe [[greenyer2013|Greenyer]] und [[Konzepte/x#Abschnitt]], nochmal [[greenyer2013]].\n";
        let md = parse_md(text);
        assert_eq!(md.author.as_deref(), Some("ich"));
        assert_eq!(md.kapitel, ["02-grundlagen", "05-zusammenfassung"]);
        assert_eq!(md.citekeys, ["sutton2018", "gebhard2021"]);
        assert_eq!(md.title.as_deref(), Some("Titel"));
        assert_eq!(md.links, ["greenyer2013", "Konzepte/x"]);
    }

    #[test]
    fn markdown_without_frontmatter() {
        let md = parse_md("Nur Text mit [[a]].\n");
        assert_eq!((md.author, md.title, md.links), (None, None, vec!["a".to_owned()]));
    }
}
