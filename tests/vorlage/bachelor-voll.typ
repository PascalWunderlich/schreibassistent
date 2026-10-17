// Erwartung: kompiliert. Bachelorarbeit mit allen optionalen Angaben.
#import "@local/fhdw-arbeit:0.1.0": *
#show: fhdw-thesis.with(
  title: "Testtitel", thesis-type: "Bachelorarbeit", study-program: "Wirtschaftsinformatik",
  author: "Erika Muster", student-id: "123456", company: "Beispiel GmbH",
  examiner: "Prof. A", second-examiner: "Prof. B",
  date: datetime(year: 2027, month: 6, day: 1), confidentiality-notice: true,
  summary: [Kurz.], abstract: [Short.],
  bibliography: bibliography("../../beispiel-arbeit/System/quellen.bib"),
  citation-style: "numeric",
)
= Einleitung
Text @sutton2018.
