// Erwartung: kompiliert. Bachelorarbeit mit allen optionalen Angaben.
#import "@local/fhdw-arbeit:0.1.0": *
#show: fhdw-arbeit.with(
  titel: "Testtitel", art: "Bachelorarbeit", studiengang: "Wirtschaftsinformatik",
  autor: "Erika Muster", matrikelnummer: "123456", firma: "Beispiel GmbH",
  pruefer: "Prof. A", zweitpruefer: "Prof. B",
  datum: datetime(year: 2027, month: 6, day: 1), sperrvermerk: true,
  zusammenfassung: [Kurz.], abstract: [Short.],
  quellen: bibliography("../../beispiel-arbeit/System/quellen.bib"),
  zitierstil: "numerisch",
)
= Einleitung
Text @sutton2018.
