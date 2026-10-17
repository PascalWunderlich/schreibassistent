// Erwartung: Abbruch mit Hinweis auf `second-examiner` (FHDW-Hinweise S. 4).
#import "@local/fhdw-arbeit:0.1.0": *
#show: fhdw-thesis.with(
  title: "Testtitel", thesis-type: "Bachelorarbeit", study-program: "Informatik",
  author: "Erika Muster", examiner: "Prof. A",
)
= Einleitung
