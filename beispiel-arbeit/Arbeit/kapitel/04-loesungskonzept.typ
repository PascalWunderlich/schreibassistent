= Lösungskonzept <kap:konzept>

#lorem(70)

== Architektur

#lorem(110)

#lorem(95)

#figure(
  image("../abbildungen/architektur.svg", width: 70%),
  caption: [Komponenten des Lösungskonzepts mit Aufrufrichtung],
) <abb:konzept>

@abb:konzept zeigt die Komponenten. #lorem(60)

// FEHLERFALL beschriftung-abbildung (kein Label, nicht referenziert)
#figure(
  image("../abbildungen/architektur.svg", width: 50%),
  caption: [Dieselben Komponenten, verkleinert],
)

== Abläufe

=== Anmeldung

#lorem(130)

#lorem(85)

=== Datenabgleich

// Langer Absatz: läuft über einen Seitenumbruch (Testfall für den Seiteneditor).
#lorem(900)

#lorem(75)

// FEHLERFALL gliederung (vierte Ebene, zugleich einziger Unterabschnitt)
==== Feinabstimmung

#lorem(40)

== Alternativen

#figure(
  table(
    columns: (auto, 1fr, 1fr),
    table.header[Variante][Vorteile][Nachteile],
    [Monolith], [Einfacher Betrieb, eine Codebasis.], [Schwer skalierbar, lange Releasezyklen.],
    [Microservices], [Unabhängige Teams und Deployments @gebhard2021.], [Verteilte Fehlerbilder, höherer Betriebsaufwand.],
    [Modularer Monolith], [Klare Grenzen ohne Netzwerkaufrufe.], [Disziplin bei Abhängigkeiten nötig.],
  ),
  caption: [Bewertung der Architekturvarianten],
) <tab:varianten>

#lorem(100)

#lorem(90)
