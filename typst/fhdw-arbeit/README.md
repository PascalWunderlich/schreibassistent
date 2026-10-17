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

#show: fhdw-thesis.with(
  title: "Titel exakt wie im FHDW-Portal",
  thesis-type: "Praxisarbeit",    // oder "Bachelorarbeit" (dann second-examiner Pflicht)
  study-program: "Informatik",
  author: "Vorname Nachname",
  examiner: "Name des Prüfers",
  date: datetime(year: 2027, month: 3, day: 26),
  bibliography: bibliography("../System/quellen.bib"),
)
```

| Parameter | Bedeutung | Standard |
|---|---|---|
| `title` | Titel, exakt wie im FHDW-Portal | Pflicht |
| `thesis-type` | `"Praxisarbeit"` oder `"Bachelorarbeit"` | `"Praxisarbeit"` |
| `study-program` | Studiengang | Pflicht |
| `author` | Vorname Nachname | Pflicht |
| `examiner` | Prüfer | Pflicht |
| `second-examiner` | Zweitprüfer | Pflicht bei Bachelorarbeit |
| `student-id` | Matrikelnummer (nur wenn gesetzt) | `none` |
| `company` | Partnerunternehmen (nur wenn gesetzt) | `none` |
| `city`, `date` | Ort und Abgabedatum | `"Hannover"`, heute |
| `confidentiality-notice` | Sperrvermerk | `false` |
| `summary`, `abstract` | Zusammenfassung, englisches Abstract | `none` |
| `glossary` | glossarium-Einträge (mit `long` → Abkürzungsverzeichnis, sonst Glossar) | `()` |
| `bibliography` | `bibliography("…/quellen.bib")` | `none` |
| `citation-style` | `"alphanumeric"` [SB18] oder `"numeric"` [1] | `"alphanumeric"` |
| `list-of-figures`, `list-of-tables` | Abbildungs-, Tabellenverzeichnis | `false` |
| `appendix` | Anhang, eigene Überschriften → „Anhang A“ | `none` |

Weitere Funktionen: `ai-usage(...)` (Tabelle zur KI-Nutzung),
`declaration-document(...)` (nur die Erklärung als eigenes PDF zum Unterschreiben),
`gls`, `glspl` (Abkürzungen und Glossar, aus glossarium).

Kompilieren mit dem Arbeits-Vault als Root, damit `System/quellen.bib` erreichbar ist:

```sh
typst compile --root <Arbeit-Vault> <Arbeit-Vault>/Arbeit/main.typ
```
