// fhdw-arbeit – Vorlage für Praxis- und Bachelorarbeiten an der FHDW Hannover
// (technische Studiengänge). Abgeleitet von clean-dhbw 0.6.0 (MIT), siehe LICENSE.
//
// Seitenangaben „S. x“ beziehen sich auf die gedruckten Seitenzahlen in
// „FHDW Hinweise wissenschaftliches Arbeiten (Stand 2026-09)“; Übersicht in
// docs/fhdw-checkliste.md.

#import "@preview/glossarium:0.5.10": make-glossary, register-glossary, print-glossary, gls, glspl
#import "titelblatt.typ": title-page
#import "erklaerung.typ": declaration
#import "sperrvermerk.typ": confidentiality-notice-page
#import "ki-nutzung.typ": ai-usage

// Der Parameter `bibliography` von `fhdw-thesis` überdeckt die eingebaute Funktion.
#let std-bibliography = bibliography

#let thesis-types = ("Praxisarbeit", "Bachelorarbeit")
#let citation-styles = (
  alphanumeric: "fhdw-alphanumerisch.csl", // [SB18], wie im FHDW-Dokument (S. 8)
  numeric: "fhdw-numerisch.csl",           // [1]
)

// ---------- Grundstil (Schrift, Absatz, Seite) ---------------------------------------
// S. 5: Schriftgröße 11pt, gängige Schrift, keine Narrow-Schrift, Blocksatz.
// S. 6: A4, einseitig, Ränder links 3–4 cm, rechts 4–5 cm, oben/unten 4–5 cm
//       → jeweils der kleinste erlaubte Wert (Entscheidung 2026-10-17).
// S. 3: Silbentrennung für die richtige Sprache.
// S. 5: Absatzbeginn eingerückt, nach Überschriften nicht.

#let base-style(body) = {
  set page(
    paper: "a4",
    margin: (left: 3cm, right: 4cm, top: 4cm, bottom: 4cm),
  )
  set text(
    font: "New Computer Modern",
    size: 11pt,
    lang: "de",
    region: "DE",
    hyphenate: true,
  )
  set par(
    justify: true,
    leading: 0.65em,
    spacing: 0.65em,
    first-line-indent: 1.5em,
  )
  body
}

// ---------- Überschriften ---------------------------------------
// S. 4: Dezimalgliederung 1, 1.1, 1.1.1. Kapitel beginnen auf neuer Seite
// (wie in der FHDW-Vorlage, „Kapitel 1“ über dem Titel).

#let chapter-heading(it) = {
  pagebreak(weak: true)
  set par(first-line-indent: 0pt, justify: false)
  v(1.5cm)
  if it.numbering != none {
    block(text(size: 14pt, weight: "bold", it.supplement + [ ] + counter(heading).display(it.numbering)))
    v(0.3em)
  }
  block(text(size: 20pt, weight: "bold", it.body))
  v(1.2em)
}

// ---------- Kopf- und Fußzeile ---------------------------------------
// S. 4: Vorspann römisch (i, ii, …), Inhaltsteil arabisch ab 1 bis einschließlich Anhang.
// Kopfzeile mit Kapitel wie in der FHDW-Vorlage; auf Kapitelanfängen Seitenzahl unten.

#let is-chapter-start() = {
  let current = here().page()
  query(heading.where(level: 1)).any(h => h.location().page() == current)
}

#let page-header = context {
  if here().page-numbering() != "1" or is-chapter-start() { return }
  let previous = query(heading.where(level: 1).before(here()))
  if previous.len() == 0 { return }
  let chapter = previous.last()
  let number = if chapter.numbering != none {
    numbering(chapter.numbering, ..counter(heading).at(chapter.location())) + h(0.6em)
  }
  set text(size: 10pt)
  grid(
    columns: (1fr, auto),
    column-gutter: 1em,
    smallcaps(number + chapter.body), counter(page).display(),
  )
  v(-0.6em)
  line(length: 100%, stroke: 0.4pt)
}

#let page-footer = context {
  let page-numbering = here().page-numbering()
  if page-numbering == none { return }
  if page-numbering == "i" or is-chapter-start() {
    align(center, text(size: 10pt, counter(page).display()))
  }
}

// ---------- Vorlage ---------------------------------------

#let fhdw-thesis(
  title: none,                   // exakt wie im FHDW-Portal angemeldet (S. 4, S. 25)
  thesis-type: "Praxisarbeit",   // "Praxisarbeit" | "Bachelorarbeit"
  study-program: none,           // Studiengang
  author: none,
  student-id: none,              // Matrikelnummer, optional, keine FHDW-Vorgabe
  company: none,                 // Partnerunternehmen, optional, keine FHDW-Vorgabe
  examiner: none,                // Prüfer
  second-examiner: none,         // Zweitprüfer, Pflicht bei Bachelorarbeit (S. 4)
  city: "Hannover",
  date: datetime.today(),        // Abgabedatum
  confidentiality-notice: false, // Sperrvermerk
  summary: none,                 // Zusammenfassung, optional, max. eine Seite (S. 4)
  abstract: none,                // optional, englische Zusammenfassung (S. 4)
  glossary: (),                  // glossarium-Einträge; mit `long` → Abkürzungsverzeichnis, sonst Glossar
  bibliography: none,            // bibliography("…/quellen.bib")
  citation-style: "alphanumeric", // "alphanumeric" [SB18] | "numeric" [1]
  list-of-figures: false,        // Abbildungsverzeichnis
  list-of-tables: false,         // Tabellenverzeichnis
  appendix: none,                // eigene Überschriften: = Titel → „Anhang A“
  body,
) = {
  // ---------- Pflichtangaben prüfen ----------
  for (name, value) in (title: title, study-program: study-program, author: author, examiner: examiner) {
    if value == none or value == "" { panic("fhdw-arbeit: `" + name + "` fehlt.") }
  }
  if thesis-type not in thesis-types {
    panic("fhdw-arbeit: `thesis-type` muss \"Praxisarbeit\" oder \"Bachelorarbeit\" sein.")
  }
  if thesis-type == "Bachelorarbeit" and second-examiner == none {
    panic("fhdw-arbeit: Bei einer Bachelorarbeit ist `second-examiner` Pflicht (FHDW-Hinweise S. 4).")
  }
  if citation-style not in citation-styles {
    panic("fhdw-arbeit: `citation-style` muss \"alphanumeric\" oder \"numeric\" sein.")
  }

  set document(title: title, author: author)
  show: base-style
  set page(header: page-header, footer: page-footer)

  // ---------- Abbildungen, Tabellen, Listings ----------
  // S. 5: Nummerierung im Dezimalsystem mit Kapitelnummer (1.1);
  //       Abbildungen haben Unterschriften, Tabellen und Listings Überschriften.
  // Kapitelnummer + laufende Nummer. Kein `context` in der Funktion: Typst wertet sie
  // dann an der Stelle der Abbildung aus, auch für Verweise und Verzeichnisse.
  let number-with-chapter(style) = n => numbering(style, counter(heading).get().first(), n)
  set figure(numbering: number-with-chapter("1.1"))
  show figure.where(kind: table): set figure.caption(position: top)
  show figure.where(kind: raw): set figure.caption(position: top)
  show figure.caption: set text(size: 10pt)
  set table(stroke: (x: none, y: 0.5pt), inset: (x: 0.5em, y: 0.45em), align: left)
  show table: set par(justify: false)
  show table.cell.where(y: 0): set text(weight: "bold")

  show heading.where(level: 1): it => {
    for kind in (image, table, raw) {
      counter(figure.where(kind: kind)).update(0)
    }
    chapter-heading(it)
  }
  show heading.where(level: 2): set text(size: 14pt)
  show heading.where(level: 3): set text(size: 12pt)
  show heading: set block(above: 1.6em, below: 1em)
  show heading: set par(justify: false)

  show: make-glossary
  register-glossary(glossary)
  let acronyms = glossary.filter(e => "long" in e)
  let terms = glossary.filter(e => "long" not in e)

  show link: set text(fill: blue.darken(40%))

  // ========== 1. Titelblatt (S. 4, Punkt 1) ==========
  {
    set page(numbering: none)
    title-page(
      title: title, thesis-type: thesis-type, study-program: study-program, author: author,
      student-id: student-id, company: company,
      examiner: examiner, second-examiner: second-examiner, city: city, date: date,
    )
  }

  // ========== Vorspann, römisch nummeriert (S. 4) ==========
  {
    set page(numbering: "i")
    counter(page).update(1)

    // 2. Ehrenwörtliche Erklärung im exakten Wortlaut (S. 4, Punkt 2; S. i)
    declaration(thesis-type: thesis-type, author: author, city: city, date: date)

    // 3. Optional: Sperrvermerk (S. 4, Punkt 3)
    if confidentiality-notice { confidentiality-notice-page() }

    // 4. Optional: Zusammenfassung, max. eine Seite, optional Abstract (S. 4, Punkt 4)
    if summary != none {
      heading(level: 1, numbering: none, outlined: false)[Zusammenfassung]
      summary
    }
    if abstract != none {
      heading(level: 1, numbering: none, outlined: false)[Abstract]
      text(lang: "en", abstract)
    }

    // 5. Inhaltsverzeichnis mit Seitenangaben (S. 4, Punkt 5)
    {
      show outline.entry.where(level: 1): set block(above: 1.1em)
      show outline.entry.where(level: 1): set text(weight: "bold")
      outline(title: [Inhaltsverzeichnis], depth: 3, indent: auto)
    }

    // 6. Optional: Abkürzungsverzeichnis (S. 4, Punkt 6)
    if acronyms.len() > 0 {
      heading(level: 1, numbering: none, outlined: false)[Abkürzungsverzeichnis]
      print-glossary(acronyms, disable-back-references: true)
    }
  }

  // ========== Inhaltsteil und Nachspann, arabisch ab 1 (S. 4) ==========
  {
    set page(numbering: "1")
    counter(page).update(1)
    counter(footnote).update(0)

    // 7. Inhaltsteil
    {
      set heading(numbering: "1.1.1", supplement: [Abschnitt])
      show heading.where(level: 1): set heading(supplement: [Kapitel])
      body
    }

    // 8. Quellverzeichnis (S. 4, Punkt 8; S. 8–9: nur zitierte Quellen, keine Fußnoten)
    if bibliography != none {
      set std-bibliography(title: [Quellverzeichnis], style: citation-styles.at(citation-style))
      show std-bibliography: set heading(numbering: none)
      bibliography
    }

    // 9. Optional: Glossar (S. 4, Punkt 9)
    if terms.len() > 0 {
      heading(level: 1, numbering: none)[Glossar]
      print-glossary(terms, disable-back-references: true)
    }

    // 10. Optional: Abbildungs- und/oder Tabellenverzeichnis (S. 4, Punkt 10)
    if list-of-figures {
      heading(level: 1, numbering: none)[Abbildungsverzeichnis]
      outline(title: none, target: figure.where(kind: image))
    }
    if list-of-tables {
      heading(level: 1, numbering: none)[Tabellenverzeichnis]
      outline(title: none, target: figure.where(kind: table))
    }

    // 11. Optional: Anhang (S. 4, Punkt 11; Anhang A der FHDW-Hinweise)
    if appendix != none {
      counter(heading).update(0)
      set figure(numbering: number-with-chapter("A.1"))
      set heading(numbering: "A.1", supplement: [Anhang])
      show heading.where(level: 1): set heading(supplement: [Anhang])
      appendix
    }
  }
}

// ---------- Erklärung als eigenes PDF ---------------------------------------
// S. 25: Die Erklärung wird zusätzlich eigenhändig unterschrieben und eingescannt
// als separate PDF-Datei hochgeladen. Nimmt dieselben Angaben wie `fhdw-thesis`.

#let declaration-document(
  thesis-type: "Praxisarbeit",
  author: none,
  city: "Hannover",
  date: datetime.today(),
  ..rest,
) = {
  set document(title: "Erklärung der Selbstständigkeit", author: author)
  show: base-style
  set page(numbering: none)
  show heading.where(level: 1): chapter-heading
  declaration(thesis-type: thesis-type, author: author, city: city, date: date)
}
