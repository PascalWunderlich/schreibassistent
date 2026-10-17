# Schreibassistent – Anweisungen für Claude Code

Lokale Desktop-App für wissenschaftliche Arbeiten (macOS zuerst). Der Mensch schreibt
den Text in Typst; die App setzt ihn nach Vorlage und Regelprofil, verwaltet Quellen
und Wissen, prüft Form und gibt KI-Feedback nur auf Nachfrage.
Product Owner: Pascal. Du schreibst den Code; Pascal gibt Pläne frei und nimmt ab.

## Leitplanken (nicht verhandelbar)
1. KI-Funktionen schreiben NIE in `Arbeit/` (*.typ) oder `Notizen/`. Schreibzugriff
   der KI nur in `Wissen/` und `System/ki-protokoll.jsonl`. Durchgesetzt im Backend
   (`ki::rechte`), nicht nur in der Oberfläche. Kein "Vorschlag übernehmen"-Knopf.
2. Zitiert wird aus dem Original. Jede KI-Aussage über eine Quelle trägt Citekey + Seite.
3. Jede KI-Anfrage wird protokolliert, bevor die Antwort angezeigt wird.
4. Lokal, ohne Konten. Netzwerk nur für gewählte KI-Anbieter.
5. Kein Fork von Typst, Tinymist, CodeMirror, Zotero. Nur über Schnittstellen nutzen.
6. Klartext-Daten. SQLite nur als Cache in `.schreibassistent/`, jederzeit löschbar.
7. Keine Geheimnisse im Repo. API-Schlüssel nur im macOS-Schlüsselbund.

## Stack
Tauri 2 · Rust (stable) · React + TypeScript (Vite, pnpm) · CodeMirror 6 ·
Typst-Crates · Tinymist (Sidecar) · SQLite/FTS5 (rusqlite) · genai-Crate ·
Docling (Python via uv, Sidecar) · Vale + typst2vast · LTeX+

## Befehle
- `pnpm tauri dev` – App starten
- `cargo test --manifest-path src-tauri/Cargo.toml` – Rust-Tests
- `pnpm test` – Frontend-Tests
- `typst compile beispiel-arbeit/Arbeit/main.typ` – Vorlage prüfen

## Arbeitsweise
- Lies zu Beginn `docs/ROADMAP.md` (aktuelle Phase) und die im Prompt genannten docs.
- Lege vor dem Coden einen Plan mit Annahmen vor und warte auf Freigabe.
- Kleine Commits, Conventional Commits, Code und Bezeichner Englisch,
  Oberfläche und Doku Deutsch.
- Zu jedem Modul Tests; `beispiel-arbeit/` enthält absichtliche Fehlerfälle.
- Bei Unklarheit fragen statt raten. Bei Abweichung vom Plan: anhalten und melden.
- Nach jeder Phase: `docs/ROADMAP.md` und `docs/DECISIONS.md` aktualisieren und eine
  Zusammenfassung "Für den Vault" (Ergebnisse, Entscheidungen mit Begründung, offene
  Punkte) ausgeben.
- Lizenzen beachten: clean-dhbw (MIT) Hinweis behalten; keinen GPL-Code kopieren.
