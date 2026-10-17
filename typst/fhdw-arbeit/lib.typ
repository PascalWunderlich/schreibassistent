// fhdw-arbeit – Vorlage für Praxis- und Bachelorarbeiten an der FHDW Hannover
// (technische Studiengänge). Abgeleitet von clean-dhbw 0.6.0 (MIT), siehe LICENSE.
//
// Seitenangaben „S. x“ beziehen sich auf die gedruckten Seitenzahlen in
// „FHDW Hinweise wissenschaftliches Arbeiten (Stand 2026-09)“; Übersicht in
// docs/fhdw-checkliste.md.

#import "@preview/glossarium:0.5.10": make-glossary, register-glossary, print-glossary, gls, glspl
#import "datum.typ": datum-lang
#import "titelblatt.typ": titelblatt
#import "erklaerung.typ": erklaerung
#import "sperrvermerk.typ": sperrvermerk as sperrvermerk-seite
#import "ki-nutzung.typ": ki-nutzung

#let arten = ("Praxisarbeit", "Bachelorarbeit")
#let zitierstile = (
  alphanumerisch: "fhdw-alphanumerisch.csl", // [SB18], wie im FHDW-Dokument (S. 8)
  numerisch: "fhdw-numerisch.csl",         // [1]
)

// ---------- Grundstil (Schrift, Absatz, Seite) ---------------------------------------
// S. 5: Schriftgröße 11pt, gängige Schrift, keine Narrow-Schrift, Blocksatz.
// S. 6: A4, einseitig, Ränder links 3–4 cm, rechts 4–5 cm, oben/unten 4–5 cm
//       → jeweils der kleinste erlaubte Wert (Entscheidung 2026-10-17).
// S. 3: Silbentrennung für die richtige Sprache.
// S. 5: Absatzbeginn eingerückt, nach Überschriften nicht.

#let grundstil(body) = {
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

#let kapitel-ueberschrift(it) = {
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

#let ist-kapitelanfang() = {
  let seite = here().page()
  query(heading.where(level: 1)).any(h => h.location().page() == seite)
}

#let kopfzeile = context {
  if here().page-numbering() != "1" or ist-kapitelanfang() { return }
  let vorher = query(heading.where(level: 1).before(here()))
  if vorher.len() == 0 { return }
  let kapitel = vorher.last()
  let nummer = if kapitel.numbering != none {
    numbering(kapitel.numbering, ..counter(heading).at(kapitel.location())) + h(0.6em)
  }
  set text(size: 10pt)
  grid(
    columns: (1fr, auto),
    column-gutter: 1em,
    smallcaps(nummer + kapitel.body), counter(page).display(),
  )
  v(-0.6em)
  line(length: 100%, stroke: 0.4pt)
}

#let fusszeile = context {
  let nummerierung = here().page-numbering()
  if nummerierung == none { return }
  if nummerierung == "i" or ist-kapitelanfang() {
    align(center, text(size: 10pt, counter(page).display()))
  }
}

// ---------- Vorlage ---------------------------------------

#let fhdw-arbeit(
  titel: none,               // exakt wie im FHDW-Portal angemeldet (S. 4, S. 25)
  art: "Praxisarbeit",       // "Praxisarbeit" | "Bachelorarbeit"
  studiengang: none,
  autor: none,
  matrikelnummer: none,      // optional, keine FHDW-Vorgabe
  firma: none,               // optional, keine FHDW-Vorgabe
  pruefer: none,
  zweitpruefer: none,        // Pflicht bei Bachelorarbeit (S. 4)
  ort: "Hannover",
  datum: datetime.today(),   // Abgabedatum
  sperrvermerk: false,
  zusammenfassung: none,     // optional, max. eine Seite (S. 4)
  abstract: none,            // optional, englische Zusammenfassung (S. 4)
  glossar: (),               // glossarium-Einträge; mit `long` → Abkürzungsverzeichnis, sonst Glossar
  quellen: none,             // bibliography("…/quellen.bib")
  zitierstil: "alphanumerisch",
  abbildungsverzeichnis: false,
  tabellenverzeichnis: false,
  anhang: none,              // eigene Überschriften: = Titel → „Anhang A“
  body,
) = {
  // ---------- Pflichtangaben prüfen ----------
  for (name, wert) in (titel: titel, studiengang: studiengang, autor: autor, pruefer: pruefer) {
    if wert == none or wert == "" { panic("fhdw-arbeit: `" + name + "` fehlt.") }
  }
  if art not in arten { panic("fhdw-arbeit: `art` muss \"Praxisarbeit\" oder \"Bachelorarbeit\" sein.") }
  if art == "Bachelorarbeit" and zweitpruefer == none {
    panic("fhdw-arbeit: Bei einer Bachelorarbeit ist `zweitpruefer` Pflicht (FHDW-Hinweise S. 4).")
  }
  if zitierstil not in zitierstile { panic("fhdw-arbeit: `zitierstil` muss \"alphanumerisch\" oder \"numerisch\" sein.") }

  set document(title: titel, author: autor)
  show: grundstil
  set page(header: kopfzeile, footer: fusszeile)

  // ---------- Abbildungen, Tabellen, Listings ----------
  // S. 5: Nummerierung im Dezimalsystem mit Kapitelnummer (1.1);
  //       Abbildungen haben Unterschriften, Tabellen und Listings Überschriften.
  // Kapitelnummer + laufende Nummer. Kein `context` in der Funktion: Typst wertet sie
  // dann an der Stelle der Abbildung aus, auch für Verweise und Verzeichnisse.
  let nummer-mit-kapitel(stil) = n => numbering(stil, counter(heading).get().first(), n)
  set figure(numbering: nummer-mit-kapitel("1.1"))
  show figure.where(kind: table): set figure.caption(position: top)
  show figure.where(kind: raw): set figure.caption(position: top)
  show figure.caption: set text(size: 10pt)
  set table(stroke: (x: none, y: 0.5pt), inset: (x: 0.5em, y: 0.45em), align: left)
  show table: set par(justify: false)
  show table.cell.where(y: 0): set text(weight: "bold")

  show heading.where(level: 1): it => {
    for art in (image, table, raw) {
      counter(figure.where(kind: art)).update(0)
    }
    kapitel-ueberschrift(it)
  }
  show heading.where(level: 2): set text(size: 14pt)
  show heading.where(level: 3): set text(size: 12pt)
  show heading: set block(above: 1.6em, below: 1em)
  show heading: set par(justify: false)

  show: make-glossary
  register-glossary(glossar)
  let abkuerzungen = glossar.filter(e => "long" in e)
  let begriffe = glossar.filter(e => "long" not in e)

  show link: set text(fill: blue.darken(40%))

  // ========== 1. Titelblatt (S. 4, Punkt 1) ==========
  {
    set page(numbering: none)
    titelblatt(
      titel: titel, art: art, studiengang: studiengang, autor: autor,
      matrikelnummer: matrikelnummer, firma: firma,
      pruefer: pruefer, zweitpruefer: zweitpruefer, ort: ort, datum: datum,
    )
  }

  // ========== Vorspann, römisch nummeriert (S. 4) ==========
  {
    set page(numbering: "i")
    counter(page).update(1)

    // 2. Ehrenwörtliche Erklärung im exakten Wortlaut (S. 4, Punkt 2; S. i)
    erklaerung(art: art, autor: autor, ort: ort, datum: datum)

    // 3. Optional: Sperrvermerk (S. 4, Punkt 3)
    if sperrvermerk { sperrvermerk-seite() }

    // 4. Optional: Zusammenfassung, max. eine Seite, optional Abstract (S. 4, Punkt 4)
    if zusammenfassung != none {
      heading(level: 1, numbering: none, outlined: false)[Zusammenfassung]
      zusammenfassung
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
    if abkuerzungen.len() > 0 {
      heading(level: 1, numbering: none, outlined: false)[Abkürzungsverzeichnis]
      print-glossary(abkuerzungen, disable-back-references: true)
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
    if quellen != none {
      set bibliography(title: [Quellverzeichnis], style: zitierstile.at(zitierstil))
      show bibliography: set heading(numbering: none)
      quellen
    }

    // 9. Optional: Glossar (S. 4, Punkt 9)
    if begriffe.len() > 0 {
      heading(level: 1, numbering: none)[Glossar]
      print-glossary(begriffe, disable-back-references: true)
    }

    // 10. Optional: Abbildungs- und/oder Tabellenverzeichnis (S. 4, Punkt 10)
    if abbildungsverzeichnis {
      heading(level: 1, numbering: none)[Abbildungsverzeichnis]
      outline(title: none, target: figure.where(kind: image))
    }
    if tabellenverzeichnis {
      heading(level: 1, numbering: none)[Tabellenverzeichnis]
      outline(title: none, target: figure.where(kind: table))
    }

    // 11. Optional: Anhang (S. 4, Punkt 11; Anhang A der FHDW-Hinweise)
    if anhang != none {
      counter(heading).update(0)
      set figure(numbering: nummer-mit-kapitel("A.1"))
      set heading(numbering: "A.1", supplement: [Anhang])
      show heading.where(level: 1): set heading(supplement: [Anhang])
      anhang
    }
  }
}

// ---------- Erklärung als eigenes PDF ---------------------------------------
// S. 25: Die Erklärung wird zusätzlich eigenhändig unterschrieben und eingescannt
// als separate PDF-Datei hochgeladen. Nimmt dieselben Angaben wie `fhdw-arbeit`.

#let erklaerung-dokument(art: "Praxisarbeit", autor: none, ort: "Hannover", datum: datetime.today(), ..rest) = {
  set document(title: "Erklärung der Selbstständigkeit", author: autor)
  show: grundstil
  set page(numbering: none)
  show heading.where(level: 1): kapitel-ueberschrift
  erklaerung(art: art, autor: autor, ort: ort, datum: datum)
}
