// Erklärung der Selbstständigkeit
//
// Wortlaut 1:1 aus „FHDW Hinweise wissenschaftliches Arbeiten (Stand 2026-09)“, S. i
// (PDF-Seite 2) inklusive Fußnote 1. NICHT umformulieren – auch die Schreibweisen
// „selbständig“ (Text) und „Selbstständigkeit“ (Überschrift) stehen so im Original.
// Einzige Anpassung: Der Platzhalter „=Bachelorarbeit / Praxisarbeit=“ wird durch `thesis-type`,
// „Hannover, <Datum>“ durch `city` und `date`, „=Vorname Nachname=“ durch `author` ersetzt.

#import "datum.typ": long-date-de

#let declaration(thesis-type: none, author: none, city: none, date: none) = {
  heading(level: 1, numbering: none, outlined: false)[Erklärung der Selbstständigkeit]

  [Hiermit versichere ich, dass ich die vorliegende #thesis-type selbständig verfasst und keine anderen als die angegebenen Quellen verwendet habe. Alle Textstellen und andere Inhalte wie Bilder, Quellcode oder Daten, die wörtlich oder sinngemäß aus anderen Quellen entnommen sind, habe ich eindeutig mit korrekten Quellenangaben versehen. Über Zitierrichtlinien bin ich schriftlich informiert worden. Diese Arbeit wurde bisher in gleicher oder ähnlicher Form, auch nicht in Teilen, keiner anderen Prüfungsbehörde vorgelegt.]
  parbreak()
  [Falls ich während der Erstellung dieser Arbeit generative KI oder andere KI-gestützte Technologien verwendet habe, die über grundlegende Werkzeuge für Übersetzungen und zur Überprüfung von Grammatik oder Rechtschreibung hinausgehen, erkläre ich, nach der Nutzung dieser Tools den Inhalt meiner #thesis-type überprüft und bearbeitet zu haben und übernehme die volle Verantwortung für die diesbezüglichen Ausführungen. Eine Verwendung solcher Werkzeuge habe ich in meiner Arbeit dokumentiert (bspw. im Abschnitt „Struktur der Arbeit, Methodik“ oder durch separate Erläuterungen zur Nutzung im Anhang#footnote[Eine Nutzungserklärung kann z.B. so aussehen: „Direkte Nutzung von KI Werkzeugen, wie z.B. ChatGPT, Gemini oder Claude, sowie indirekte Nutzung z.B. mittels GitHub Copilot, wurden dafür genutzt, Formulierungen verständlicher zu machen und Unterstützung bei der Anfertigung von Programmcode zu erhalten.“]).]

  v(2em)
  block[#city, #long-date-de(date)]
  v(2.5em)
  block(author)
}
