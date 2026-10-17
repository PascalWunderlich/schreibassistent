# Entscheidungen (Kurzform; ausführlich im Vault „Schreibassistent Typst“)
| Datum | Entscheidung | Begründung |
|---|---|---|
| 2026-10-17 | KI ist Assistent, schreibt nie Text der Arbeit | Eigenleistung, FHDW-Regeln |
| 2026-10-17 | Eigene Tauri-App, Typst als Bibliothek, kein Fork | Kontrolle über Oberfläche, Typst ist Apache-2.0 |
| 2026-10-17 | Tinymist für Editor-Intelligenz | aufwendig, kein Alleinstellungsmerkmal |
| 2026-10-17 | Lokal, ohne Konten; KI-Anbieter austauschbar inkl. Ollama | Unabhängigkeit, Datenschutz, Kosten |
| 2026-10-17 | Regelprofil per KI aus Vorgaben-PDF, vom Menschen bestätigt | keine vorgefertigten Profile |
| 2026-10-17 | Zotero + Better BibTeX, Citekey als Schlüssel | Erfassung nicht nachbauen |
| 2026-10-17 | Wissensseiten = Wegweiser + grober Überblick, einmal beim Ingest erzeugt | Stellen finden; Token-Kosten |
| 2026-10-17 | Werkzeuge über Homebrew, Tauri-CLI als Projekt-Abhängigkeit (`@tauri-apps/cli`) | ohne Admin-Rechte; Version im Lockfile festgelegt |
| 2026-10-17 | Node 24 (LTS) über `node@24` | aktuelle LTS-Linie, stabil für Vite/Tauri |
| 2026-10-17 | Frontend-Tests mit Vitest + jsdom + Testing Library | passt zu Vite, keine eigene Build-Konfiguration |
| 2026-10-17 | Opener-Plugin und Demo-Code aus der Tauri-Vorlage entfernt | nicht benötigt; weniger Rechte (nur `core:default`) |
| 2026-10-17 | Ränder l 3 / r 4 / o 4 / u 4 cm, nur einseitig | jeweils kleinster erlaubter Wert (FHDW S. 6); für zweiseitig keine Angaben |
| 2026-10-17 | Erklärung wörtlich inkl. Fußnote 1 und Schreibweise „selbständig“/„Selbstständigkeit“ | Pflicht „genauer Wortlaut“ (FHDW S. 4) |
| 2026-10-17 | Matrikelnummer und Firma nur optional auf dem Titelblatt | keine FHDW-Vorgabe (S. 4) |
| 2026-10-17 | Zitierstil alphanumerisch als Standard, numerisch wählbar; eigene CSL-Stile | wie im FHDW-Dokument; Typst hat keinen alphanumerischen Stil für Verzeichnisse, IEEE kürzt Vornamen entgegen S. 9 |
| 2026-10-17 | Vorlage als `@local/fhdw-arbeit` per Symlink, Kompilieren mit `--root <Arbeits-Vault>` | gleiche Vorlage für Beispiel und echte Arbeit; `System/quellen.bib` liegt außerhalb von `Arbeit/` |
| 2026-10-17 | Schrift New Computer Modern 11pt | mit Typst ausgeliefert, entspricht Latin Modern der FHDW-LaTeX-Vorlage (S. 5) |
| 2026-10-17 | Nur glossarium als Fremdpaket; codelst und hydra aus clean-dhbw entfernt | weniger Abhängigkeiten und Downloads |
| 2026-10-17 | Parameter und Funktionen der Vorlage auf Englisch (`title`, `examiner`, `fhdw-thesis` …); ausgegebene Werte und Dateinamen deutsch | „Bezeichner Englisch“ (CLAUDE.md), von Pascal bestätigt; Dateinamen wie `erklaerung.typ` laut DATA-FORMATS |
| 2026-10-17 | Editor: Seiteneditor (A) als Hauptansicht, Quelltextmodus für Tabellen, Abbildungen, Code | in der App 44–68 ms statt erlaubter 300 ms; Tippgefühl „Top“; passt zur Vision und liefert Seitenpositionen für P5 (docs/editor-test.md) |
| 2026-10-17 | Seiten als Bilder (`<img>` mit SVG), nicht als Inline-SVG | 23 Inline-SVGs ließen die Webansicht hängen |
| 2026-10-17 | Typst-Pakete nur aus lokalem Cache und `@local`, kein Download | Leitplanke 4; fehlendes Paket → Fehlermeldung |
| 2026-10-17 | Absätze mit `#show`, `#set`, `#let`, `#import`, `#include` sind im Seiteneditor nie editierbar | schützt die Einbindung der Vorlage vor versehentlichem Überschreiben |
