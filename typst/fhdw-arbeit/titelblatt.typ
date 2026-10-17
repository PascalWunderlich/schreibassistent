// Titelblatt nach FHDW-Hinweisen S. 4, Punkt 1 (Aufbau wie Muster S. 1):
// Titel (exakt wie im FHDW-Portal), Art der Arbeit, Studiengang, Name, Prüfer
// (bei Bachelorarbeit Erst- und Zweitprüfer), Abgabedatum.
// Matrikelnummer (`student-id`) und Partnerunternehmen (`company`) sind keine FHDW-Vorgabe
// und erscheinen nur, wenn gesetzt.

#import "datum.typ": long-date-de

#let title-page(
  title: none,
  thesis-type: none,
  study-program: none,
  author: none,
  student-id: none,
  company: none,
  examiner: none,
  second-examiner: none,
  city: none,
  date: none,
) = {
  set align(center)
  set par(first-line-indent: 0pt, justify: false)
  set text(hyphenate: false)

  v(2cm)
  text(size: 20pt, weight: "bold", title)
  v(1.5cm)
  text(size: 14pt, thesis-type)
  linebreak()
  [im Studiengang #study-program]
  v(1.5cm)
  [von]
  linebreak()
  v(0.3em)
  text(size: 14pt, author)
  if student-id != none {
    linebreak()
    [Matrikelnummer: #student-id]
  }
  if company != none {
    linebreak()
    [Partnerunternehmen: #company]
  }

  v(1fr)
  if second-examiner != none {
    [Prüfer: #examiner]
    linebreak()
    [Zweitprüfer: #second-examiner]
  } else {
    [Prüfer: #examiner]
  }
  v(1.5cm)
  [#city, #long-date-de(date)]
}
