// Erwartung: Abbruch mit Hinweis auf `zweitpruefer` (FHDW-Hinweise S. 4).
#import "@local/fhdw-arbeit:0.1.0": *
#show: fhdw-arbeit.with(
  titel: "Testtitel", art: "Bachelorarbeit", studiengang: "Informatik",
  autor: "Erika Muster", pruefer: "Prof. A",
)
= Einleitung
