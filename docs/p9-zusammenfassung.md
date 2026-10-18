# P9 UI-Überarbeitung: Zusammenfassung „Für den Vault“

Stand 2026-10-18. Design System (Tokens, Komponenten, UX-Leitlinien): https://claude.ai/artifact/CkHwHXszYmysq3NMkVvzYG (privat).

## Ergebnisse
- **Layout:** drei Spalten (Kapitel | Blatt | Prüfen/Assistent), ziehbar, Breiten gemerkt; schlanke Kopfzeile; Hell/Dunkel/System.
- **Kapitelleiste:** Kapitelliste mit Seitenzahl, „+ Kapitel“, ⌘1–9; „Dieses Kapitel“ mit Quellen und Notizen; „Alle Quellen“ und Dateibaum eingeklappt.
- **Einfügeleiste:** Überschriften, Listen, Zitat, Abbildung, Tabelle, Balken-/Liniendiagramm, Formel, Listing, Fußnote, Quelle zitieren, Querverweis. Typst-Bausteine, mit `typst compile` geprüft.
- **Prüfen:** Hinweiskarten mit Etikett (Form, Stil, Quelle, Sprache), Filter mit Zähler, Fehler vor Warnungen, `.bib`-Metadaten eingeklappt.
- **Assistent:** Eingabe immer unten, Kontext darüber, Aufgaben über „/“ oder ✨, Antworten mit „KI · auf Nachfrage“.
- **Statusleiste:** „Seite 14 / 20–22“ (Abweichung mit Icon und Wort), Hintergrundvorgänge, Sicherung, „KI-Protokoll: n Einträge“ mit Liste.
- **Querschnitt:** ⌘K-Befehlspalette, ⌘/ Tastenkürzel, Toasts statt Banner.
- **Menüleiste (macOS):** Ablage mit Neue Arbeit, Öffnen, Speichern, Speichern unter, PDF, Finder, Schließen; Bearbeiten; Darstellung; Fenster; Hilfe; Einstellungen (⌘,).

## Entscheidungen (Begründung in docs/DECISIONS.md)
Native Menüleiste statt eigener; Diagramme ohne Pakete; Hinweisarten im Frontend abgeleitet; „Speichern“ = Git-Sicherung sofort, „Speichern unter“ = Kopie; Einfügeleiste nur auf Klick des Menschen.

## Leitplanken
Unberührt: kein „Übernehmen“-Knopf, die KI hat keinen Zugriff auf die Einfügeleiste, Protokoll nur lesend angezeigt. Neue Backend-Befehle: `protokoll_liste`, `projekt_create`, `projekt_copy_to`, `projekt_sichern`.

## Nachweise
`pnpm test` 29 Tests grün, `tsc` und `pnpm build` ok, `cargo test` 107 grün im zweiten Lauf (erster Lauf: 4 Tests flackerten, s. Roadmap „Später“). Neue Arbeit kompiliert mit `typst compile`.

## Offen
- **Abnahme durch Pascal im laufenden Fenster (`pnpm tauri dev`):** bisher kein Blick ins echte Fenster, nur Tests.
- Sechs Kernaufgaben ohne Nachfragen: von Pascal zu bestätigen.
- Ersteinstieg und „Zuletzt geöffnet“, Sammelaktion „alle Quellen einlesen“, Einfügeleiste für Notizen, weitere Diagrammarten (siehe ROADMAP „Später“).
