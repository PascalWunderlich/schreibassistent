# fhdw-arbeit

Typst-Vorlage für Praxis- und Bachelorarbeiten an der FHDW Hannover (technische
Studiengänge) nach „FHDW Hinweise wissenschaftliches Arbeiten (Stand 2026-09)“.
Abgeleitet von [clean-dhbw](https://github.com/roland-KA/clean-dhbw-typst-template) 0.6.0 (MIT, siehe `LICENSE`).
Welche Vorgabe wie umgesetzt ist: `docs/fhdw-checkliste.md`.

## Einrichten

```sh
typst/installieren.sh      # verlinkt das Paket als @local/fhdw-arbeit:0.1.0
```

## Benutzen

```typst
#import "@local/fhdw-arbeit:0.1.0": *

#show: fhdw-arbeit.with(
  titel: "Titel exakt wie im FHDW-Portal",
  art: "Praxisarbeit",            // oder "Bachelorarbeit" (dann zweitpruefer Pflicht)
  studiengang: "Informatik",
  autor: "Vorname Nachname",
  pruefer: "Name des Prüfers",
  datum: datetime(year: 2027, month: 3, day: 26),
  quellen: bibliography("../System/quellen.bib"),
)
```

Optionale Parameter: `matrikelnummer`, `firma`, `zweitpruefer`, `ort` (Standard
Hannover), `sperrvermerk`, `zusammenfassung`, `abstract`, `glossar`,
`zitierstil` (`"alphanumerisch"` [SB18] oder `"numerisch"` [1]),
`abbildungsverzeichnis`, `tabellenverzeichnis`, `anhang`.

Weitere Funktionen: `ki-nutzung(...)` (Tabelle zur KI-Nutzung),
`erklaerung-dokument(...)` (nur die Erklärung als eigenes PDF zum Unterschreiben),
`gls`, `glspl` (Abkürzungen und Glossar, aus glossarium).

Kompilieren mit dem Arbeits-Vault als Root, damit `System/quellen.bib` erreichbar ist:

```sh
typst compile --root <Arbeit-Vault> <Arbeit-Vault>/Arbeit/main.typ
```
