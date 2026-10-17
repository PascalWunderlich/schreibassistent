# Editor-Test (P2): Seiteneditor (A) oder Vorschau-Editor (B)

Stand 2026-10-17. Testdokument: `beispiel-arbeit/`, 23 Seiten (5 Vorspann, 18 Inhalt),
New Computer Modern 11 pt. Gerät: MacBook (Apple Silicon), macOS 27.
Begriffe: Seiteneditor (A) und Vorschau-Editor (B) wie in `docs/GLOSSARY.md`.

## Prototypen

| | A: Seiteneditor | B: Vorschau-Editor |
|---|---|---|
| Ansicht | echte Typst-Seiten (SVG je Seite als Bild) | Quelltext links (CodeMirror), Tinymist-Vorschau rechts |
| Bearbeiten | Klick auf einen Absatz öffnet seinen Quelltext an Ort und Stelle | ganze Datei als Quelltext |
| Satz | eigenes Modul `satz` (Typst-Crates 0.15.1) | Tinymist 0.15.10 als Sprachserver (`lsp`) |
| Code | `src-tauri/src/satz/` (mapping, world), `src/prototypes/a/` | `src-tauri/src/lsp/`, `src/prototypes/b/` |
| Tasten | Enter: neuer Absatz (teilt an der Cursorstelle) · Shift+Enter: Zeilenumbruch im Quelltext · Esc oder Klick daneben: übernehmen | normale Texteingabe |
| Sprünge | – (man klickt direkt auf die Seite) | Klick im Quelltext → Vorschau; Klick in der Vorschau → Quelltext; beim Tippbeginn nach ≥ 2 s Pause springt die Vorschau einmal mit |

Änderungen werden in beiden Prototypen nur im Speicher gehalten, nicht gespeichert.

## Messwerte

### Ohne Oberfläche: Änderung → gesetzt (und bei A: SVG der geänderten Seiten)

je 30 Änderungen, `cargo run --example satz_messung` bzw. `--example vorschau_messung`

| Szenario | A dev: Median / schlechtester | A release: Median / schlechtester | A neu gezeichnete Seiten | B: Median / schlechtester |
|---|---|---|---|---|
| Wort einfügen, Kapitel 1 | 11 / 14 ms | 9 / 9 ms | 1 | 8 / 25 ms |
| Wort einfügen, Kapitel 3 | 11 / 20 ms | 9 / 17 ms | 1–2 | 8 / 26 ms |
| Wort einfügen, Kapitel 5 | 11 / 36 ms | 9 / 11 ms | 2 | 7 / 61 ms |
| Satz einfügen, Umbruch verschiebt sich | 26 / 59 ms | 23 / 57 ms | 2–14 | 17 / 56 ms |

Erstes Kompilieren A: 160–170 ms. B misst bis zur Meldung „gesetzt“ (`tinymist/compileStatus`),
ohne das Zeichnen in der Vorschau.

### In der App (Pascals Tests, Protokoll `beispiel-arbeit/.schreibassistent/editor-test.log`)

| | n | Median | schlechtester | gemessen |
|---|---|---|---|---|
| A, 1. Test: Enter/übernehmen → Seite gezeichnet | 9 | 68 ms | 86 ms | inkl. Weg zur Oberfläche und zwei Bildaufbauten |
| A, 2. Test: Enter/übernehmen → Seite gezeichnet | 13 | 44 ms | 77 ms | wie oben |
| B: Tastendruck → Tinymist „gesetzt“ | 71 | 11 ms | 88 ms | ohne Zeichnen in der Vorschau |

**Zielwert A (< 300 ms Median auf dem Testdokument): erfüllt** – gemessen 44–68 ms in der App,
9–26 ms im Backend.

### Seitenumbruch-Verschiebung

- A: Verschiebt eine Änderung den Umbruch, werden alle betroffenen Seiten neu gezeichnet
  (bis 14 von 23 Seiten, weil sich auch Seitenzahlen und Verzeichnisse ändern); trotzdem
  schlechtester Fall 59 ms. Seiten ohne Änderung bleiben unangetastet (Vergleich per Hash).
- B: Tinymist aktualisiert die Vorschau inkrementell; kein sichtbarer Unterschied zu kleinen Änderungen.

## Tippgefühl (Pascal)

- A: „Top“, nachdem Breite, Schrift, Größe, Zeilenabstand und Einzug des Feldes dem Satz entsprachen.
- B: „ähnlich gut“. Die Animation der Vorschau bei jedem Tippbeginn nervte – Ursache: jeder Sprung
  lässt Tinymist das Dokument neu an die Vorschau schicken. Behoben (Sprung nur bei Klick); als
  Wunsch umgesetzt: einmaliger Sprung zu Beginn des Tippens.

## Fehlerfälle

| Fall | A | B |
|---|---|---|
| Klick auf Text aus der Vorlage (Titelblatt, Erklärung, Verzeichnisse, Kopfzeilen) | nicht editierbar, Hinweis; Start direkt bei Kapitel 1 | nicht geprüft |
| Inhalt, der an die Vorlage übergeben wird (`summary: […]`) | editierbar; Enter bleibt im Inhaltsblock (Fehler gefunden und behoben, Test) | normal editierbar |
| `#show`, `#set`, `#let`, `#import`, `#include` | werden nie als Absatz geöffnet (Schutz, Test) | normal editierbar |
| Überschrift | eigener Absatz; Label `<…>` bleibt außerhalb des Feldes | normal |
| Abbildungsunterschrift, Tabellenzelle | nur dieser Text wird geöffnet (Test) | normal |
| Absatz über einen Seitenumbruch | wird erkannt (Rechteck auf beiden Seiten), Feld erscheint auf der ersten Seite | normal |
| Syntaxfehler (`#lorem(`) | Meldung, Feld bleibt offen, Quelltext wird zurückgerollt (Fehler gefunden und behoben, Test) | Vorschau zeigt den letzten gültigen Stand, Fehler in der Diagnose |
| Neuer Absatz | Enter am Ende: leerer Absatz darunter; in der Mitte: Absatz wird geteilt | Leerzeile tippen |
| Markup im Absatz (`#lorem(60)`, `@omg2017`, `*fett*`) | Feld zeigt Quelltext; Zeilenumbruch im Feld weicht dann vom Satz ab | Quelltext wie immer |
| Zeilenumbruch reiner Fließtext | gleiche Schrift und Spalte, aber Typst (Knuth-Plass) und Browser trennen/brechen an einzelnen Stellen anders | – |
| Viele große Seiten im Fenster | inline-SVG ließ die Webansicht hängen (Fehler gefunden); als Bilder (`<img>`) behoben | keine Probleme beobachtet |
| Sprung Quelltext → Vorschau nach Kapitelwechsel | – | **unzuverlässig**: Tinymist sendet den Sprung nachweislich (`jump,6 …` auf der Vorschau-Verbindung), die Vorschau setzt ihn nicht immer um; Ursache in Tinymists Vorschau-Oberfläche, offen |
| Klick in die Vorschau → Quelltext | – | funktioniert (`window/showDocument`), auch über Dateien hinweg |

## Restaufwand

### A bis zum Schreibwerkzeug (P3)

| Baustein | Aufwand (Sitzungen) |
|---|---|
| Speichern auf die Platte, automatische Sicherung (Git), Konflikt mit externen Änderungen | 1 |
| Navigation zwischen Absätzen (Pfeil hoch/runter am Feldrand, Tab), Rückgängig über Absätze hinweg | 1–2 |
| Auswahl und Kopieren über mehrere Absätze; Einfügen mit Leerzeilen | 1 |
| Tinymist im Feld: Vervollständigung (`@citekey`, Labels), Diagnosen, Hover | 1–2 |
| Absatz über Seitenumbruch: Feld über beide Seitenteile | 0,5 |
| Abbildungen, Tabellen, Code (Quelltextmodus je Block) | 1 |
| Lange Arbeiten (40–60 Seiten): virtuelles Laden der Seitenbilder | 0,5 |
| **Summe** | **ca. 6–8** |

### B bis zum Schreibwerkzeug (P3)

| Baustein | Aufwand (Sitzungen) |
|---|---|
| Speichern, Sicherung (gleich wie A) | 1 |
| Tinymist-Funktionen im Editor (Vervollständigung, Diagnosen) | 1 |
| Sprung nach Kapitelwechsel klären oder eigene Vorschau aus `satz` statt Tinymist-Vorschau | 1–2 |
| **Summe** | **ca. 3–4** |

Gemeinsam für beide: Modul `satz` (PDF-Export, Seitenpositionen für Prüfungs-Hinweise am Absatz in P5)
und Modul `lsp` (Tinymist) werden ohnehin gebraucht.

## Empfehlung

**A als Hauptansicht, mit einem Quelltextmodus aus B für Stellen mit viel Markup.**

Begründung:
1. Das größte Risiko von A ist ausgeräumt: 44–68 ms in der App statt der erlaubten 300 ms,
   auch bei verschobenem Seitenumbruch unter 100 ms.
2. Das Tippgefühl von A wurde als „Top“ bewertet, sobald das Feld dem Satz entsprach;
   B nur als „ähnlich gut“, mit Flackern und unzuverlässigem Sprung, die von Tinymists
   Vorschau abhängen und sich nur begrenzt beeinflussen lassen.
3. A passt zur Vision („Ich schreibe meine Arbeit selbst“, Seite wie im PDF) und liefert die
   Seitenpositionen, die P5 für Hinweise am Absatz braucht.
4. A hat Grenzen, die bleiben: Markup ist im Feld sichtbar, und der Umbruch im Feld kann vom
   Satz abweichen. Für Tabellen, Abbildungen und Code ist der Quelltextmodus (B-Ansicht)
   besser – als Umschalter, nicht als zweites Programm.
5. Der Mehraufwand gegenüber B (ca. 3–4 Sitzungen) ist vertretbar, weil die Praxisarbeit
   nicht an der App hängt (M1: Vorlage in VS Code + Tinymist).

Falls Zeit wichtiger ist als Bedienung: B ist mit ca. 3–4 Sitzungen schneller einsatzbereit;
dann die Vorschau aus `satz` statt aus Tinymist bauen, um Flackern und Sprungfehler loszuwerden.

## Entscheidung

Offen – trifft Pascal; danach in `docs/DECISIONS.md`.
