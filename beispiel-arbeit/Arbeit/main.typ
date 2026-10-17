#import "@local/fhdw-arbeit:0.1.0": *
#import "metadaten.typ": thesis
#import "glossar.typ": glossary

#show: fhdw-thesis.with(
  ..thesis,
  summary: [#lorem(120)],
  glossary: glossary,
  bibliography: bibliography("../System/quellen.bib"),
  citation-style: "alphanumeric",
  list-of-figures: true,
  list-of-tables: true,
  appendix: include "anhang.typ",
)

#include "kapitel/01-einleitung.typ"
#include "kapitel/02-grundlagen.typ"
#include "kapitel/03-umsetzung.typ"
