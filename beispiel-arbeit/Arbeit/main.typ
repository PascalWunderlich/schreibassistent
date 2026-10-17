#import "@local/fhdw-arbeit:0.1.0": *
#import "metadaten.typ": arbeit
#import "glossar.typ": glossar

#show: fhdw-arbeit.with(
  ..arbeit,
  zusammenfassung: [#lorem(120)],
  glossar: glossar,
  quellen: bibliography("../System/quellen.bib"),
  zitierstil: "alphanumerisch",
  abbildungsverzeichnis: true,
  tabellenverzeichnis: true,
  anhang: include "anhang.typ",
)

#include "kapitel/01-einleitung.typ"
#include "kapitel/02-grundlagen.typ"
#include "kapitel/03-umsetzung.typ"
