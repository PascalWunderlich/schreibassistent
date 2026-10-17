// Sperrvermerk / Geheimhaltungserklärung (optional, FHDW-Hinweise S. 4, Punkt 3)
//
// OFFEN: Der Wortlaut steht nicht im FHDW-PDF, sondern in der LaTeX-Vorlage
// (misc/datarestriction.sty, Overleaf/FHDW-Git). Bis er vorliegt, erzeugt diese Seite
// einen deutlich sichtbaren Platzhalter. Keinen eigenen Text erfinden.

#let sperrvermerk() = {
  heading(level: 1, numbering: none, outlined: false)[Sperrvermerk]
  block(
    width: 100%,
    inset: 1em,
    stroke: 1pt + red,
    text(fill: red)[
      *Wortlaut fehlt.* Den Text des Sperrvermerks aus der FHDW-LaTeX-Vorlage
      (`misc/datarestriction.sty`) in `typst/fhdw-arbeit/sperrvermerk.typ` übernehmen.
    ],
  )
}
