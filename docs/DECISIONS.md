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
