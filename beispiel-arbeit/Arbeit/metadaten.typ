// Angaben zur Arbeit – gemeinsam genutzt von main.typ und erklaerung-einzeln.typ.
#let thesis = (
  title: "Beispieltitel exakt wie im FHDW-Portal angemeldet",
  thesis-type: "Praxisarbeit",   // oder "Bachelorarbeit"
  study-program: "Informatik",
  author: "Vorname Nachname",
  student-id: none,              // optional, z. B. "123456"
  company: none,                 // optional, z. B. "Beispiel GmbH"
  examiner: "Name des Prüfers",
  second-examiner: none,         // Pflicht bei "Bachelorarbeit"
  city: "Hannover",
  date: datetime(year: 2027, month: 3, day: 26),
  confidentiality-notice: false, // Sperrvermerk
)
