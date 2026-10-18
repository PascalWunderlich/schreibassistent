# Roadmap
Aktuelle Phase: P8
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
| P8 KI-Prüfungen | in Abnahme | Hinweise mit Originalstelle |
Details je Phase: Vault „Schreibassistent – Implementierungsplan“.

## Später (notiert)
| Datum | Punkt | Herkunft |
|---|---|---|
| 2026-10-17 | Speichern von Hand / benannte Versionen zusätzlich zum automatischen Speichern | Pascal, Planung P3 |
| 2026-10-17 | Startbare `.app` ohne Terminal, Tinymist als Sidecar eingebaut (T14) | Pascal, Planung P3 |
| 2026-10-17 | Seitenvorgabe (20–22) aus dem Regelprofil statt fest im Code | P3, erledigt in P5 |
| 2026-10-17 | Notiz von außen geändert, während sie offen ist: neu laden statt überschreiben | P3, offen |
| 2026-10-17 | URLs fehlen im Better-BibTeX-Auto-Export (`@misc`): Einstellung je Auto-Export in Zotero setzen oder Export neu anlegen; vor der Praxisarbeit klären | P4, offen |
| 2026-10-17 | Design der Seitenleiste besprechen | Pascal, P4 |
| 2026-10-17 | „PDF öffnen“ aus einem Zitat nimmt die gedruckte Seitenzahl als PDF-Seite; bei römischem Vorspann daneben (Markierungen öffnen richtig) | P4, Schwäche |
| 2026-10-17 | Zotero-Testdaten: Titel von gebhard2021/weber2022 falsch geschrieben (Import), Testbuch mit langem Citekey; vor echter Nutzung bereinigen | P4, Testdaten |
| 2026-10-18 | LTeX+ startet kalt in ~15 s (Java); früher im Hintergrund starten oder beim Öffnen vorwärmen | P5 |
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
