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
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypFile {
    pub headings: Vec<Heading>,
    pub references: Vec<Reference>,
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
    walk(&LinkedNode::new(&root), text, &mut out);
    out
}

fn walk(node: &LinkedNode, text: &str, out: &mut TypFile) {
    match node.kind() {
        SyntaxKind::Heading => {
            let level = node.children().find(|c| c.kind() == SyntaxKind::HeadingMarker).map_or(1, |m| m.range().len());
            let body = node.children().find(|c| c.kind() == SyntaxKind::Markup).map(|m| text[m.range()].trim().to_owned());
            out.headings.push(Heading { level, text: body.unwrap_or_default(), offset: node.offset() });
        }
        SyntaxKind::Ref => {
            let marker = node.children().find(|c| c.kind() == SyntaxKind::RefMarker);
            if let Some(marker) = marker {
                let supplement = node.children().find(|c| c.kind() == SyntaxKind::ContentBlock).map(|c| block_text(&c, text));
                out.references.push(Reference { key: text[marker.range()][1..].to_owned(), supplement, offset: node.offset() });
            }
        }
        SyntaxKind::FuncCall => {
            let callee = node.children().next();
            if callee.as_ref().is_some_and(|c| c.kind() == SyntaxKind::Ident && &text[c.range()] == "cite")
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
                    out.references.push(Reference { key, supplement, offset: node.offset() });
                }
            }
        }
        _ => {}
    }
    for child in node.children() {
        walk(&child, text, out);
    }
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
