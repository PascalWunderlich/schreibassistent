// Angaben zur Arbeit – gemeinsam genutzt von main.typ und erklaerung-einzeln.typ.
#let arbeit = (
  titel: "Beispieltitel exakt wie im FHDW-Portal angemeldet",
  art: "Praxisarbeit",
  studiengang: "Informatik",
  autor: "Vorname Nachname",
  matrikelnummer: none,          // optional, z. B. "123456"
  firma: none,                   // optional, z. B. "Beispiel GmbH"
  pruefer: "Name des Prüfers",
  zweitpruefer: none,            // Pflicht bei "Bachelorarbeit"
  ort: "Hannover",
  datum: datetime(year: 2027, month: 3, day: 26),
  sperrvermerk: false,
)
