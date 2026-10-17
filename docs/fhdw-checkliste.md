# FHDW-Checkliste für die Vorlage `fhdw-arbeit`

Quelle: „FHDW Hinweise wissenschaftliches Arbeiten (Stand 2026-09)“ (`referenz/`).
Seitenangaben = gedruckte Seitenzahlen im PDF (römisch im Vorspann, z. B. S. i = PDF-Seite 2;
arabisch ab Kapitel 1, z. B. S. 4 = PDF-Seite 10).

**Art:** P = Pflicht („müssen“, „Verwenden Sie“), E = Empfehlung („sollten“, „Empfehlung“).
**Status:**
- ✅ erfüllt – die Vorlage setzt die Regel automatisch um
- 👤 Mensch – hängt vom Inhalt ab, die Vorlage kann es nicht erzwingen
- 🔎 P5 – wird in Phase 5 als feste Prüfung umgesetzt (Regelprofil)
- ❓ offen – unklar oder fehlende Angabe, siehe unten

## Bestandteile und Reihenfolge (S. 4, Abschnitt 2.4)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 1 | Reihenfolge: Titelblatt → Erklärung → Sperrvermerk → Zusammenfassung → Inhaltsverzeichnis → Abkürzungsverzeichnis → Inhaltsteil → Quellverzeichnis → Glossar → Abbildungs-/Tabellenverzeichnis → Anhang | P | S. 4 | `lib.typ`, Abschnitte 1–11 in genau dieser Reihenfolge | ✅ |
| 2 | Titelblatt mit Name, Studiengang, Prüfer (Bachelor: Erst- und Zweitprüfer), Abgabedatum | P | S. 4, Muster S. 1 | `titelblatt.typ`; Abbruch bei Bachelorarbeit ohne `zweitpruefer` | ✅ |
| 3 | Titel exakt wie im FHDW-Portal angemeldet | P | S. 4, S. 25 | Parameter `titel`; Abgleich mit dem Portal macht der Mensch | 👤 |
| 4 | Ehrenwörtliche Erklärung im genauen Wortlaut der Vorlage | P | S. 4, Wortlaut S. i | `erklaerung.typ`, Text und Fußnote 1:1 (maschinell gegen das PDF verglichen); Platzhalter durch `art`, `ort`, `datum`, `autor` ersetzt | ✅ (bitte gegenlesen) |
| 5 | Optional: Geheimhaltungserklärung / Sperrvermerk | E | S. 4 | `sperrvermerk: true` erzeugt die Seite; **Wortlaut fehlt** | ❓ U3 |
| 6 | Optional: Zusammenfassung, max. eine Seite, optional englisch (Abstract) | E | S. 4 | `zusammenfassung`, `abstract` | ✅ / Länge 🔎 |
| 7 | Inhaltsverzeichnis mit Seitenangaben, in jedem Fall | P | S. 4, S. 21 | immer erzeugt, Tiefe 3 | ✅ |
| 8 | Optional: Abkürzungsverzeichnis | E | S. 4 | Einträge mit `long` in `glossar` | ✅ |
| 9 | Inhaltsteil inkl. Abbildungen und Tabellen | P | S. 4 | `body` | ✅ |
| 10 | Quellverzeichnis | P | S. 4 | `quellen: bibliography(...)`, Titel „Quellverzeichnis“ | ✅ |
| 11 | Optional: Glossar | E | S. 4 | Einträge ohne `long` in `glossar` | ✅ |
| 12 | Optional: Abbildungs- und/oder Tabellenverzeichnis | E | S. 4 | `abbildungsverzeichnis`, `tabellenverzeichnis` | ✅ |
| 13 | Optional: Anhang | E | S. 4 | `anhang`, Überschriften als „Anhang A“, Nummerierung A.1 | ✅ |
| 14 | Anhang im Regelfall nicht über 50 % des Inhaltsteils; keine ausgelagerten Abbildungen | E | S. 32 (Anhang A) | – | 🔎 / 👤 |

## Seitenzählung (S. 4)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 15 | Seiten vor dem Inhaltsteil (Erklärung, Inhaltsverzeichnis …) klein römisch i, ii, iii | P | S. 4 | Vorspann `numbering: "i"`, Erklärung = i; Titelblatt ohne Nummer wie im Muster | ✅ |
| 16 | Arabisch ab erster Seite des Inhaltsteils, fortlaufend bis zum Ende inkl. Quellverzeichnis und Anhang | P | S. 4 | ab Kapitel 1 `numbering: "1"` ab 1; PDF-Seitenlabels geprüft (`'', i–iv, 1–9`) | ✅ |

## Gliederung und Nummerierung (S. 4–5)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 17 | Inhalt in Kapitel und Abschnitte gegliedert | P | S. 4 | – | 👤 |
| 18 | Hat ein Abschnitt einen Unterabschnitt, dann mindestens einen zweiten | E | S. 4 | – | 🔎 |
| 19 | Dezimalnummerierung 1, 1.1, 1.1.1 | P | S. 4 | `heading(numbering: "1.1.1")` | ✅ |
| 20 | Abbildungen und Tabellen im Dezimalsystem nummeriert, optional mit Kapitelnummer (1.1); Listings ebenso | P | S. 5 | „Abbildung 2.1“, „Tabelle 3.1“, im Anhang „Tabelle A.1“; Zähler je Kapitel zurückgesetzt | ✅ |
| 21 | Abbildungen haben Unterschriften, Tabellen und Listings Überschriften | P | S. 5 | Beschriftung unter Abbildungen, über Tabellen und Code | ✅ |
| 22 | Verweise über automatische Mechanismen, mit Art der Referenz („Abschnitt 2.7“) | P | S. 5 | `@label` ergibt „Kapitel 2“, „Abschnitt 1.2“, „Abbildung 2.1“, „Tabelle 3.1“, „Anhang A“; hart getippte Nummern erkennt erst P5 | ✅ / 🔎 |
| 23 | Aussagekräftige Überschriften und Beschriftungen | E | S. 5 | – | 👤 |
| 24 | Absatzbeginn eingerückt, nach Überschriften nicht | E | S. 5 | `first-line-indent: 1.5em` (Typst rückt nach Überschriften nicht ein) | ✅ |

## Schrift, Seitenformat, Ränder (S. 5–6)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 25 | Gängige, gut lesbare Schrift in 11pt, keine „Narrow“-Schriften | P | S. 5 | New Computer Modern 11pt (Latin-Modern-Nachfolger, mit Typst ausgeliefert) | ✅ |
| 26 | Neue Begriffe kursiv, Schlagworte sparsam fett; Konventionen in „Struktur der Arbeit, Methodik“ erläutern | E | S. 5 | – | 👤 |
| 27 | Seitenformat A4 | P | S. 5 | `paper: "a4"` | ✅ |
| 28 | Ein- oder zweiseitiges Format wählbar | E | S. 5 | nur einseitig umgesetzt | ❓ U2 |
| 29 | Blocksatz | P | S. 5 | `justify: true` | ✅ |
| 30 | Ränder einseitig: links 3–4 cm, rechts 4–5 cm, oben und unten 4–5 cm | P | S. 6 | links 3, rechts 4, oben 4, unten 4 cm (Entscheidung 2026-10-17, U1) | ✅ |
| 31 | Silbentrennung für die richtige Sprache | E | S. 3 | `lang: "de"`, `hyphenate: true`; im Titel ausgeschaltet | ✅ |
| 32 | Arbeit im Regelfall auf Deutsch, Englisch nach Absprache | E | S. 3 | nur Deutsch umgesetzt | ❓ U7 |

## Abbildungen (S. 6–7)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 33 | Grafiken möglichst als Vektorformat, JPG/PNG nur ausnahmsweise | E | S. 6 | Beispiel nutzt SVG | 🔎 |
| 34 | Jede Abbildung lesbar, nicht über den Rand des Textbereichs | P | S. 6 | – | 👤 / 🔎 |
| 35 | Jede Abbildung im Text referenziert und beschrieben | P | S. 6 | Beispiel zeigt `@abb:…`; Prüfung „nicht referenziert“ in P5 | 🔎 |
| 36 | Abbildungen nah an der Stelle der Referenz | E | S. 6 | Typst setzt Abbildungen an die Stelle im Text (kein Gleiten) | ✅ |
| 37 | In Abbildungen serifenlose Schrift, ca. 60–90 % der Textgröße, einheitlich | E | S. 7 | – | 👤 |

## Quellen und Zitieren (S. 8–12)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 38 | Numerischer oder alphabetischer Verweisstil | P | S. 8 | `zitierstil: "alphanumerisch"` (Standard, [SB18]) oder `"numerisch"` ([1]); eigene CSL-Stile | ✅ |
| 39 | Keine Fußnoten für Zitate | P | S. 9 | Vorlage zitiert im Text in eckigen Klammern | ✅ / 🔎 |
| 40 | Im Quellverzeichnis alle zitierten Quellen und nur diese | P | S. 9 | Typst druckt nur zitierte Einträge | ✅ |
| 41 | Optional mehrere Quellverzeichnisse nach Publikationstyp | E | S. 9 | nicht umgesetzt (optional) | – |
| 42 | Buch: Autoren (Nachname, Vorname), Punkt oder Doppelpunkt konsistent, Titel, Verlag, optional Ort, Auflage falls nicht erste, Jahr | P | S. 9 | `fhdw-*.csl`, Zweig `book`; Trenner einheitlich Punkt | ✅ |
| 43 | Zeitschriftenartikel: Autoren, Titel, Zeitschrift, Heft, Band, Seitenbereich, Jahr | P | S. 9 | Zweig `article-journal`: „Zeitschrift, Band(Heft), Seiten, Jahr“ | ✅ |
| 44 | Konferenzbeitrag: Autoren, Titel, Herausgeber, Konferenzband, optional Serie/Heft/Band, Seitenbereich, Jahr | P | S. 9 | Zweig `paper-conference`: „In: Hrsg., Band, Serie, Band-Nr., Seiten, Verlag, Jahr“ | ✅ |
| 45 | Internetquelle: Autoren, Titel, Online-Magazin, Veröffentlichungsdatum, URL, Zugriffsdatum | P | S. 9–10 | sonstige Typen: `howpublished`, Datum, `url`, `urldate` („Zugriff am“) | ✅ |
| 46 | Vollständige und geprüfte Angaben je Quelle | P | S. 9 | fehlende Felder in `quellen.bib` meldet erst P5 | 🔎 |
| 47 | Wörtliche Zitate eindeutig in Anführungszeichen, mit Seite | P | S. 10 | Beispiel `„…“ @greenyer2013[S. 1]` | 👤 |
| 48 | Fußnoten nur in Ausnahmefällen | E | S. 10 | – | 🔎 (Hinweis) |
| 49 | Übernommene Bilder, Code, Daten kennzeichnen („Grafik entnommen aus …“) | P | S. 11 | – | 👤 |
| 50 | Wörtlich übernommene KI-Antworten in Anführungszeichen | P | S. 12 | – | 👤 |
| 51 | KI-Nutzung dokumentieren (Methodik-Abschnitt oder Anhang) | P | S. i, S. 12, S. 14 | `ki-nutzung(...)`, im Beispiel als Anhang A; später aus `System/ki-protokoll.jsonl` (P6) | ✅ |

## Abkürzungen und Glossar (S. 21)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 52 | Bedeutung einer Abkürzung oder eines Fachbegriffs beim ersten Auftreten erläutern | E | S. 21 | `#gls("api")` schreibt beim ersten Mal „Application Programming Interface (API)“ | ✅ |
| 53 | Gängige Begriffe (Testfall, GmbH) nicht ins Glossar | E | S. 21 | – | 👤 |

## Abgabe (S. 25)

| # | Regel | Art | Fundstelle | Umsetzung | Status |
|---|---|---|---|---|---|
| 54 | Hauptdokument als PDF inkl. Anhang | P | S. 25 | `typst compile` | ✅ |
| 55 | Ehrenwörtliche Erklärung eigenhändig unterschrieben, eingescannt, als eigene PDF | P | S. 25 | `Arbeit/erklaerung-einzeln.typ` erzeugt nur die Erklärung zum Ausdrucken | ✅ |

## Inhaltliche Vorgaben (Kapitel 3–4)

Struktur der Praxisarbeit (S. 14–15), Struktur der Bachelorarbeit (S. 18–19),
Forschungsfrage (S. 19–20), Grundlagen (S. 20), verwandte Arbeiten (S. 21) und
Bewertungskriterien (S. 22–24) sind Urteilsfragen → Regelprofil-Klasse `urteil`, KI-Hinweise in P8.
Firmenwebsites für Konzepterklärungen meiden (S. 11) → `urteil`.

## Unklare und offene Punkte

| ID | Punkt | Fundstelle | Stand |
|---|---|---|---|
| U1 | Ränder nur als Bereiche angegeben | S. 6 | entschieden 2026-10-17: jeweils kleinster Wert (l 3, r 4, o 4, u 4 cm) |
| U2 | Zweiseitiges Format erlaubt, aber ohne Randangaben | S. 5–6 | offen; Vorlage nur einseitig |
| U3 | Wortlaut des Sperrvermerks steht nur in der LaTeX-Vorlage (`misc/datarestriction.sty`) | S. 4 | offen; Platzhalter mit Hinweis „Wortlaut fehlt“. Overleaf-ZIP in `referenz/` legen |
| U4 | Kopfzeile (Kapitel, Seitenzahl) liegt im oberen Rand; dazu gibt es keine Vorgabe | S. 6 | Annahme: zulässig, wie in der FHDW-LaTeX-Vorlage |
| U5 | Form der alphabetischen Kürzel nicht geregelt; Typst bildet [Gre+13], die LaTeX-Vorlage [GMSCH13] | S. 8 | Annahme: beides zulässig, da nur „alphabetisch“ verlangt ist |
| U6 | Gliederungstiefe über 1.1.1 hinaus nicht geregelt | S. 4 | Inhaltsverzeichnis bis Ebene 3 |
| U7 | Englischsprachige Arbeit nach Absprache | S. 3 | offen; Vorlage nur Deutsch |
| U8 | Erklärung: „selbständig“ (Text) und „Selbstständigkeit“ (Überschrift) unterschiedlich geschrieben | S. i | wörtlich übernommen (Entscheidung 2026-10-17) |
