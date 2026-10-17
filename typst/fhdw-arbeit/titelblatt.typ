// Titelblatt nach FHDW-Hinweisen S. 4, Punkt 1 (Aufbau wie Muster S. 1):
// Titel (exakt wie im FHDW-Portal), Art der Arbeit, Studiengang, Name, Prüfer
// (bei Bachelorarbeit Erst- und Zweitprüfer), Abgabedatum.
// Matrikelnummer und Partnerunternehmen sind keine FHDW-Vorgabe und erscheinen nur, wenn gesetzt.

#import "datum.typ": datum-lang

#let titelblatt(
  titel: none,
  art: none,
  studiengang: none,
  autor: none,
  matrikelnummer: none,
  firma: none,
  pruefer: none,
  zweitpruefer: none,
  ort: none,
  datum: none,
) = {
  set align(center)
  set par(first-line-indent: 0pt, justify: false)
  set text(hyphenate: false)

  v(2cm)
  text(size: 20pt, weight: "bold", titel)
  v(1.5cm)
  text(size: 14pt, art)
  linebreak()
  [im Studiengang #studiengang]
  v(1.5cm)
  [von]
  linebreak()
  v(0.3em)
  text(size: 14pt, autor)
  if matrikelnummer != none {
    linebreak()
    [Matrikelnummer: #matrikelnummer]
  }
  if firma != none {
    linebreak()
    [Partnerunternehmen: #firma]
  }

  v(1fr)
  if zweitpruefer != none {
    [Prüfer: #pruefer]
    linebreak()
    [Zweitprüfer: #zweitpruefer]
  } else {
    [Prüfer: #pruefer]
  }
  v(1.5cm)
  [#ort, #datum-lang(datum)]
}
