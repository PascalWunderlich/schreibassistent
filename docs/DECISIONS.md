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
| 2026-10-17 | Speichern ohne Knopf: Feld spätestens nach 1 s, Enter/Esc sofort, atomar (temporäre Datei + Umbenennen) | kein Datenverlust bei Absturz außer höchstens der letzten Sekunde; Speichern von Hand später (ROADMAP) |
| 2026-10-17 | Git-Autosave höchstens alle 5 Minuten, auch nicht erzwungen beim Schließen; nie in ein fremdes Repo | Vorgabe P3; Text liegt ohnehin sofort auf der Platte; App-Repo bleibt sauber |
| 2026-10-17 | Tinymist erhält den Stand über die Platte (`didChangeWatchedFiles`), nicht über `didOpen`/`didChange` | Tinymist 0.15.10 übernahm Texte im Speicher nicht zuverlässig, liest geänderte Dateien aber nach ~50 ms |
| 2026-10-17 | Satz lädt Systemschriften wie die CLI | sonst fehlen z. B. Helvetica in SVG-Abbildungen und das PDF weicht ab |
| 2026-10-17 | PDF „identisch zur CLI“ = gleicher Inhalt aller Datenströme; Byte-Gleichheit nicht erreichbar | CLI (Homebrew) nutzt eine andere Deflate-Bibliothek; Unterschied nur in Kompression und davon abgeleiteter XMP-InstanceID |
| 2026-10-17 | Quelltextmodus mit eigenen Seiten aus `satz` statt Tinymist-Vorschau | kein Flackern, gleiche Seiten wie im Seiteneditor |
| 2026-10-17 | Regelprofil `System/regelprofil.yaml` mit Schema und festem Vokabular; FHDW-Profil von Hand, von Pascal freigegeben | Prüfungen ohne KI; jede Regel mit Fundstelle; „man“/Füllwörter ausdrücklich als „keine FHDW-Vorgabe“ markiert |
| 2026-10-17 | LTeX+ (MPL-2.0, mit LanguageTool LGPL-2.1 und Java) unverändert als eigenes Programm, in P5 nicht gebündelt | Lizenz frei und weitergebbar; kein fremder Code im eigenen; 315 MB |
| 2026-10-17 | LTeX+-Treffer nur im Fließtext; Grammatik nicht in Tabellenzellen, Argument-Blöcken und direkt nach Code | LTeX+ liest Typst nur grob; sonst 37 Fehlalarme im Beispiel |
| 2026-10-17 | Fehlerfälle, die das Setzen verhindern, in eigenen Test-Vaults (`tests/pruefung/`) | Beispielarbeit muss setzbar bleiben (Vorlagentests P1) |
| 2026-10-18 | KI über genai 0.6.5 (stabil), nicht 0.7 (nur Vorabversionen) | Vault-Vorgabe genai; 0.6.5 hat Ollama, OpenRouter, Anthropic, OpenAI-kompatibel |
| 2026-10-18 | Anbieter Ollama, OpenRouter, Anthropic, OpenAI-kompatibel; Standard für alle Aufgaben Ollama `qwen3:8b`, Vorschlag bei OpenRouter `anthropic/claude-sonnet-5.5` | Pascal: kein Anthropic-Schlüssel, online realistisch OpenRouter; Ollama läuft ohne Konto (Leitplanke 4) |
| 2026-10-18 | DeepSeek als weiterer Anbieter (eigener Schlüssel, Vorschlag `deepseek-chat`) | Pascal: schneller und günstiger zum Testen |
| 2026-10-18 | Keine Kostenanzeige, nur Tokens der letzten Anfrage in der Statusleiste | Pascal, Planung P6 |
| 2026-10-18 | Ollama über seine OpenAI-kompatible Schnittstelle mit `reasoning_effort: none` | genais eigene Ollama-Anbindung kann das Nachdenken nicht abschalten; qwen3 brauchte 108 s statt 15 s je Frage |
| 2026-10-18 | API-Schlüssel mit keyring 3 (`apple-native`) im Schlüsselbund, Dienst „schreibassistent“, Konto = Anbieter; Oberfläche sieht nur „gesetzt/nicht gesetzt“ | Leitplanke 7; keyring 4 ist nur noch ein Sammelpaket vieler Speicher |
| 2026-10-18 | KI-Schreibrecht nur für `.md` unter genau `Wissen/` (Byte für Byte), ohne `..`, `.`, absolute Pfade, versteckte Teile, Symlinks (auch `Wissen/` selbst) | Leitplanke 1; macOS unterscheidet Groß/klein nicht, daher Vergleich am Text und zusätzlich am aufgelösten Ordner |
| 2026-10-18 | Protokoll: SHA-256 je Zeile über den Eintrag ohne `hash`, `vorheriger_hash` verkettet, `deny_unknown_fields`, fsync vor der Antwort; auch Fehlschläge und abgelehnte Schreibversuche stehen darin (Feld `ergebnis`) | Leitplanke 3; Änderung, Löschung und eingeschmuggelte Felder fallen auf (Tests) |
| 2026-10-18 | Export nutzt `ai-usage` der Vorlage, zusammengefasst je Tag, Modell und Zweck; Verwendung fest „Hinweise gelesen, kein Text übernommen“; nur erfolgreiche Anfragen; Export verweigert bei beschädigter Kette | Tabelle bleibt kurz; Einbinden von Hand mit `#include "anhang/ki-nutzung.typ"` |
