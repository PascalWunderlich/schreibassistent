# Roadmap
Aktuelle Phase: P10
| Phase | Status | Abnahme |
|---|---|---|
| P0 Grundlage | erledigt (2026-10-17) | Tauri-Fenster startet, Kontextdateien vorhanden |
| P1 Vorlage | erledigt (2026-10-17) | Beispiel-PDF erfüllt FHDW-Checkliste |
| P2 Editor-Test | erledigt (2026-10-17, Entscheidung A) | Entscheidung A/B mit Messwerten |
| P3 App-Gerüst | erledigt (2026-10-17) | Kapitel in der App geschrieben und exportiert |
| P4 Quellen & Index | erledigt (2026-10-17) | Seitenleiste zeigt Quellen/Notizen je Kapitel |
| P5 Feste Prüfungen | erledigt (2026-10-18) | alle Fehlerfälle erkannt, keine Fehlalarme |
| P6 KI-Grundlage | erledigt (2026-10-18; Frage mit Claude offen) | Schreibsperre nachgewiesen, Protokoll vollständig |
| P7 Ingest & Wissen | erledigt (2026-10-18) | Stellen führen zur richtigen Seite |
| P8 KI-Prüfungen | erledigt (2026-10-18) | Hinweise mit Originalstelle |
| P9 UI-Überarbeitung | umgesetzt (2026-10-18), Abnahme durch Pascal offen | 6 Kernaufgaben ohne Nachfragen, Reibungsliste abgearbeitet |
| P10 Einfache Installation (macOS, Windows, Linux) | Schritte A bis C umgesetzt (2026-10-18, Zweig `p10-installation`), CI grün auf macOS arm64, Windows x64, Linux x64 (Tests und Pakete .dmg/.exe/.deb, Lauf 8); Abnahme (D) offen; Plan in `docs/P10-plan.md`, Anleitung `docs/installation.md` | frisches macOS-Konto: Download bis PDF < 10 Min. ohne Terminal |
Reibungen beim Benutzen: docs/ui-reibung.md (Eingabe für P9).
Details je Phase: Vault „Schreibassistent – Implementierungsplan“.

## Später (notiert)
| Datum | Punkt | Herkunft |
|---|---|---|
| 2026-10-17 | Speichern von Hand / benannte Versionen zusätzlich zum automatischen Speichern | Pascal, Planung P3 |
| 2026-10-17 | Startbare `.app` ohne Terminal, Tinymist als Sidecar eingebaut (T14) | Pascal, Planung P3; in P10 umgesetzt (Bündel `build/werkzeuge`), auf frischem Konto noch nicht abgenommen |
| 2026-10-17 | Seitenvorgabe (20–22) aus dem Regelprofil statt fest im Code | P3, erledigt in P5 |
| 2026-10-17 | Notiz von außen geändert, während sie offen ist: neu laden statt überschreiben | P3, offen |
| 2026-10-17 | URLs fehlen im Better-BibTeX-Auto-Export (`@misc`): Einstellung je Auto-Export in Zotero setzen oder Export neu anlegen; vor der Praxisarbeit klären | P4, offen |
| 2026-10-17 | Design der Seitenleiste besprechen | Pascal, P4 |
| 2026-10-17 | „PDF öffnen“ aus einem Zitat nimmt die gedruckte Seitenzahl als PDF-Seite; bei römischem Vorspann daneben (Markierungen öffnen richtig) | P4, Schwäche |
| 2026-10-17 | Zotero-Testdaten: Titel von gebhard2021/weber2022 falsch geschrieben (Import), Testbuch mit langem Citekey; vor echter Nutzung bereinigen | P4, Testdaten |
| 2026-10-18 | LTeX+ startet kalt in ~15 s (Java); früher im Hintergrund starten oder beim Öffnen vorwärmen | P5, erledigt in P10 (Vorwärmen beim Öffnen) |
| 2026-10-18 | Regel „zitierstil: erlaubt“ (numerisch/alphabetisch) wird nicht geprüft, die Vorlage erlaubt ohnehin nur diese beiden | P5 |
| 2026-10-18 | Abnahme „eine Frage mit Claude“ offen: kein API-Schlüssel; nachholen, sobald ein OpenRouter-Schlüssel da ist | P6 |
| 2026-10-18 | „Verwendung“ im KI-Export ist fest; später je Anfrage selbst eintragen (z. B. „Gliederung geändert“) | P6 |
| 2026-10-18 | Antworten nicht gestreamt; lokale Modelle brauchen ~15 s ohne Zwischenstand | P6 |
| 2026-10-18 | Einlesen mit Ollama: Standard-Kontext 4096 Tokens reicht nicht; Kontextlänge in Ollama erhöhen oder online-Anbieter | P7, später (Pascal: bei DeepSeek bleiben) |
| 2026-10-18 | Schlüssel im Schlüsselbund nur für die App lesbar; Testprogramme (`examples/`) können keine Online-Anbieter nutzen | P7 |
| 2026-10-18 | Anhang von greenyer2013 in Zotero ist kein Artikel, sondern eine Klausurterminliste; austauschen | P7, Testdaten |
| 2026-10-18 | Prüfungstests mit LTeX+ scheitern selten unter Last (Start von Java parallel zu anderen schweren Tests) | P7 |
| 2026-10-18 | Texterkennung (OCR) für gescannte PDFs fehlt | P7 |
| 2026-10-18 | „Konzepte erzeugen“ in der App noch nicht ausprobiert (Knopf gebaut, Tests grün) | P7 |
| 2026-10-18 | Mit „…“ gekürzte Zitate gelten als nicht bestätigt; Auslassungen beim Vergleich erlauben | P7 |
| 2026-10-18 | Handtestseiten aus P4 in Wissen/ (greenyer2013, sutton2018, Konzepte/produktlinien) aufräumen | P7, Testdaten |
| 2026-10-18 | Regelprofil aus PDF: Gliederungstiefe ohne `max`, Satzspiegel (A4/11 pt/Ränder) und 4 Urteilsregeln (Firmenwebsites, wissenschaftliche Quellen, Abbildung beschreiben, Zitate kennzeichnen) fehlten gegenüber dem Handprofil; dafür Sprache, Aktiv/Passiv, Gendern, Anhang ≤ 50 % neu | P8, Vergleich |
| 2026-10-18 | Quelle ↔ Aussage findet ohne Seitenangabe englische Quellen zu deutschen Sätzen schlecht (Volltextsuche) | P8 |
| 2026-10-18 | Ersteinstieg (3 überspringbare Schritte: Arbeit, Quellenordner, KI-Anbieter) | P9, erledigt in P10 (Dialog gebaut, Tests grün, im Fenster nicht gesehen) |
| 2026-10-18 | Menü „Zuletzt geöffnet“ in der Ablage | P9 |
| 2026-10-18 | Einfügeleiste: Kreis- und Ablaufdiagramm; Bild beim Einfügen automatisch nach `Arbeit/abbildungen/` kopieren | P9 |
| 2026-10-18 | Notizen und Wissensseiten brauchen noch dieselbe Einfügeleiste/Formatierung (nur Typst-Dateien haben sie) | P9 |
| 2026-10-18 | P9 in `pnpm tauri dev` mit Pascal durchgehen (Layout, Menü, Dunkel): bisher nur Tests und Typprüfung, kein Blick ins laufende Fenster | P9, Abnahme |
| 2026-10-18 | Prüfungstests (LTeX+) und `quellen::zotero::tests::unavailable_zotero_is_silent` schlugen im ersten Volllauf fehl, im zweiten liefen alle 107 | P9, flackernd (Last bzw. laufendes Zotero) |
| 2026-10-18 | P10: Windows und Linux nur über die CI belegt (Tests, Paket); auf echter Hardware nie gestartet (Menü, Schlüsselspeicher, NSIS-Installer, deb, SmartScreen) | P10, offen bis jemand es ausprobiert |
| 2026-10-18 | P10: `werkzeuge.lock`: Prüfsumme des Typst-Pakets `preview/glossarium` fehlt (`-`); das Skript meldet die berechnete, bitte eintragen | P10, erledigt: Prüfsumme aus dem ersten CI-Lauf übernommen (Erstbenutzung, nicht unabhängig gegengeprüft) |
| 2026-10-18 | P10: Versionen in `werkzeuge.lock` (Vale 3.12.0, LTeX+ 18.5.1, uv 0.9.0, typst2vast 0.1.0) sind geprüft vorhandene, nicht die neuesten; bei Bedarf anheben und Prüfsumme neu eintragen | P10 |
| 2026-10-18 | P10: Docling-Erstinstallation nur bis zum Fortschritt der Paketliste ausprobiert (Linux, abgebrochen wegen Speicherplatz); Modell-Download (`download_models`) und Gesamtdauer nicht geprüft; Linux lädt PyTorch mit GPU-Paketen (> 2 GB) | P10, offen |
| 2026-10-18 | P10: Pakete entstehen nur in der CI als Artefakte; ein Release-Ablauf (Tag → Release mit Dateien) fehlt | P10 |
| 2026-10-18 | P10: Notarisierung und Windows-Signatur, sobald Konten da sind (Warnungen entfallen dann) | P10, Entscheidung 1 |
| 2026-10-18 | P10: App ~0,5 GB groß (LTeX+ mit Java dominiert); kleinere Variante (Java-Laufzeit teilen, Sprachen kürzen) prüfen | P10 |
| 2026-10-18 | P10: AppImage für Linux (linuxdeploy findet `libjvm.so` des mitgelieferten JDK nicht); bis dahin nur deb | P10 |
| 2026-10-18 | P10: Test `refuses_symlinks_out_of_knowledge` (Schreibsperre gegen Symlinks) läuft nur auf macOS/Linux; unter Windows braucht das Anlegen von Symlinks besondere Rechte | P10 |
