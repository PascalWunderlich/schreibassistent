= Umsetzung und Evaluierung <kap:umsetzung>

#lorem(60) Die Modellierung erfolgt in UML @omg2017.

== Anforderungen

@tab:anforderungen fasst die Anforderungen zusammen, die in @kap:grundlagen hergeleitet wurden.

#figure(
  table(
    columns: (auto, 1fr, auto),
    table.header[ID][Anforderung][Art],
    [A1], [Das System lädt eine Konfiguration beim Start.], [funktional],
    [A2], [Antwortzeit unter 200 ms bei 100 Anfragen/s.], [nicht-funktional],
    [A3], [Ausfälle einzelner Komponenten werden protokolliert.], [funktional],
  ),
  caption: [Anforderungen an das Beispielsystem],
) <tab:anforderungen>

#lorem(80) Der Aufbau aus @abb:architektur bleibt dabei unverändert.

== Bewertung

#lorem(90) Die Konventionen dazu stehen in @sec:struktur.

== Betrieb

#lorem(150)

#lorem(130)

#lorem(110)
