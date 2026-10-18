# Datenformate

## Arbeits-Vault (eine Arbeit = ein Ordner, eigenes Git-Repo)
<Arbeit>/
  Arbeit/        main.typ, kapitel/NN-name.typ          Mensch
  Notizen/       *.md                                   Mensch (KI liest)
  Wissen/
    Quellen/     <citekey>.md                           KI
    Konzepte/    *.md (grober Überblick)                KI
    Synthesen/   *.md (z. B. Widersprüche)              KI
  System/        regelprofil.yaml, quellen.bib,
                 ki-protokoll.jsonl, index.md, log.md
  .schreibassistent/  cache.sqlite, chunks/  (gitignored, neu erzeugbar)
Jede .md trägt `author: ich | ai` im Frontmatter.

## Quellenseite (Wegweiser, kein Beleg)
---
type: quelle
author: ai
citekey: mueller2024
erzeugt: 2026-11-02
modell: <anbieter/modell>
quelle_hash: sha256:<hash des PDFs>
markierungen_hash: sha256:<hash der Zotero-Markierungen> | keine
seitenzahlen: gedruckt | pdf | pdf-seite
stellen_bestaetigt: 7 von 8
---
# mueller2024
> Wegweiser, kein Beleg. Zitiert wird aus dem Original.
## Worum es geht        (2–3 Sätze)
## Interessante Stellen  (- [S. 14](zotero://open-pdf/library/items/<Anhang>?page=<PDF-Seite>): „kurzes Zitat“ – warum interessant;
                         nicht bestätigt: - ⚠ S. 14: „…“ – … *(Grund)*)
## Bezüge               (Konzepte, Kapitel)
## Widersprüche         (zu anderen Citekeys, mit Seiten)
Regel: Einmal beim Ingest erzeugt. Neu nur bei geändertem quelle_hash, neuen
Markierungen oder auf Knopfdruck.

## Notiz (Mensch)
Frontmatter optional: `citekeys: [mueller2024]`, `kapitel: [02-grundlagen]`.

## Regelprofil (System/regelprofil.yaml)
profil: FHDW technische Studiengänge
quelle: FHDW Hinweise wissenschaftliches Arbeiten (Stand 09/2026)
bestaetigt: false            # erst nach Durchsicht true
regeln:
  - id: umfang-praxisarbeit
    klasse: maschinell        # maschinell | vorlage | urteil
    art: seitenumfang         # festes Vokabular, siehe Schema
    bereich: inhaltsteil
    min: 20
    max: 22
    fundstelle: "S. ?, Abschnitt ?"   # aus dem PDF übernehmen
  - id: abbildung-beschriftung
    klasse: maschinell
    art: beschriftung
    element: abbildung
    position: unten
  - id: zitierweise
    klasse: maschinell
    art: zitierstil
    erlaubt: [numerisch, alphabetisch]
    fussnoten: verboten
  - id: erklaerung-wortlaut
    klasse: vorlage
    art: textbaustein
    datei: typst/fhdw-arbeit/erklaerung.typ
  - id: firmenwebsites
    klasse: urteil
    text: Firmenwebsites für Konzepterklärungen meiden
JSON-Schema: System/regelprofil.schema.json (festes Vokabular für `art`).

## KI-Protokoll (System/ki-protokoll.jsonl, nur anhängen)
{"id":"…","zeit":"2026-11-02T10:15:00+01:00","anbieter":"anthropic","modell":"…",
 "aufgabe":"belegpruefung","kontext":{"datei":"Arbeit/kapitel/02-grundlagen.typ",
 "absatz":12},"anfrage":"…","antwort":"…","tokens":{"ein":1234,"aus":321},
 "ergebnis":"ok","vorheriger_hash":"…","hash":"sha256:…"}
- `ergebnis`: `ok`, `fehler` (Anbieter nicht erreichbar, Antwort = Fehlertext) oder
  `abgelehnt` (Schreibversuch der KI außerhalb von `Wissen/`, Aufgabe `schreiben`, ohne Anbieter).
- `hash` = SHA-256 über die Zeile mit leerem `hash`; erste Zeile hat `vorheriger_hash` "".
  Prüfen: Knopf „Protokoll prüfen“ oder `cargo run --example protokoll_pruefen -- <Vault>`.
Export: `Arbeit/anhang/ki-nutzung.typ` mit `ai-usage` der Vorlage
(Datum | Werkzeug/Modell | Zweck | Verwendung), nur auf Klick, nur erfolgreiche Anfragen.
