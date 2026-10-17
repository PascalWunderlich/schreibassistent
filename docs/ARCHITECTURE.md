# Architektur
## Bausteine
| Modul | Ort | Aufgabe |
|---|---|---|
| projekt | src-tauri/src/projekt | Arbeits-Vault öffnen, Dateien beobachten, Git-Sicherung |
| satz | src-tauri/src/satz | Typst-World, Kompilieren, SVG je Seite, PDF, Positions-Mapping |
| lsp | src-tauri/src/lsp + src/editor | Tinymist starten, LSP zwischen CodeMirror und Tinymist |
| quellen | src-tauri/src/quellen | quellen.bib lesen, Zotero Local API (localhost:23119) |
| index | src-tauri/src/index | Kapitel ↔ Citekey ↔ Wissensseite ↔ Notiz, SQLite-Cache |
| pruefung | src-tauri/src/pruefung | Regelprofil laden, feste Prüfungen, Vale/LTeX+ einbinden |
| ki | src-tauri/src/ki | Anbieter (genai), Rechte, Aufgaben, Kontextbau |
| protokoll | src-tauri/src/protokoll | KI-Protokoll schreiben, Export als Typst-Tabelle |
| ingest | src-tauri/src/ingest + sidecars/docling | PDF → Abschnitte mit Seite → FTS5, Quellenseite |
| Oberfläche | src/ | Seitenleiste links, Seite Mitte, Panel rechts (oben Prüfungen, unten Assistent), Statusleiste |

## Datenfluss
Zotero → quellen.bib / Local API → quellen → index
Arbeit/*.typ → satz (Seiten) + lsp (Tinymist) + index (Citekeys, Überschriften)
satz + Regelprofil → pruefung → Hinweise (Panel oben, Markierung am Absatz)
Frage im Panel → ki (Kontext aus index + FTS5 + Wissen) → protokoll → Antwort
Neue Quelle → ingest (Docling) → FTS5 + Wissen/Quellen/<citekey>.md (KI, einmalig)

## Schnittstellen Rust ↔ Oberfläche
Tauri-Commands, benannt `modul_aktion` (z. B. `satz_render_seite`,
`pruefung_ausfuehren`, `ki_frage`). Ereignisse vom Backend: `datei_geaendert`,
`hinweise_aktualisiert`, `index_aktualisiert`.
