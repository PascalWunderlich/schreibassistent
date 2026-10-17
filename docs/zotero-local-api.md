# Zotero Local API – was sie liefert (P4)

Stand 2026-10-17. Getestet mit **Zotero 10.0.6** und **Better BibTeX 9.0.71** auf macOS.
Code: `src-tauri/src/quellen/zotero.rs`.

## Voraussetzung

Zotero muss laufen, und in den Einstellungen unter „Erweitert“ muss
„Anderen Anwendungen auf diesem Computer erlauben, mit Zotero zu kommunizieren“ aktiv sein.
Die API ist dann unter `http://localhost:23119/api/users/0/…` erreichbar. Sie ist **nur lesend**
und nur lokal (Leitplanke 4). Läuft Zotero nicht, wartet die App höchstens 0,8 s und zeigt
die Quellen nur aus der `.bib` (Test `unavailable_zotero_is_silent`).

## Ergebnisse

| Frage | Ergebnis | geprüft |
|---|---|---|
| Liefert die API den Citekey? | Ja, als eigenes Feld `citationKey` (Zotero 10 hat das Feld nativ; Better BibTeX nutzt es). Damit ordnet die App Zotero-Einträge über den Citekey zu. | ✅ mit 7 echten Einträgen |
| Einträge einer Sammlung? | Ja: `collections/<key>/items/top` | ✅ |
| PDF-Anhänge? | `items?itemType=attachment`, Feld `parentItem` = Eintrag, `contentType = application/pdf`. | ⚠️ Abfrage geprüft, lieferte mangels PDF eine leere Liste |
| **Markierungen mit Seite?** | Laut Zotero-Datenmodell (Tabelle `itemAnnotations`): `annotationText`, `annotationComment`, `annotationPageLabel` (gedruckte Seitenzahl, z. B. „14“), `annotationPosition` mit `pageIndex` (PDF-Seite ab 0). Abfrage: `items?itemType=annotation`, `parentItem` = Anhang. Die App zeigt „S. <pageLabel>“ und öffnet das PDF an `pageIndex + 1`. | ⚠️ **nicht mit echten Markierungen geprüft** (siehe unten) |
| PDF an einer Seite öffnen? | `zotero://open-pdf/library/items/<Anhang-Schlüssel>?page=<n>` | ⚠️ ohne PDF nicht geprüft |
| Neue Quelle ohne Neustart? | Neuer Eintrag in der Sammlung → Better BibTeX schreibt `System/quellen.bib` nach wenigen Sekunden neu → Dateibeobachtung der App lädt Quellen und Index neu. | ✅ bis zur `.bib` (Eintrag über die Connector-Schnittstelle angelegt); Anzeige in der App: Pascal |
| Schreiben (Titel ändern, Anhänge, Markierungen)? | Nein, die Local API ist nur lesend. Neue Einträge gehen über die Connector-Schnittstelle (`/connector/saveItems`, Ziel-Sammlung per `/connector/updateSession`). | ✅ |

## Grenzen

- **Seitenzahl im Zitat ≠ PDF-Seite:** „PDF öffnen“ aus einem Zitat wie `@key[S. 14]` übergibt 14 als
  Seite. Bei PDFs mit Vorspann (römische Seiten) landet man daneben. Markierungen haben beides
  (Seitenlabel und `pageIndex`) und öffnen deshalb richtig.
- **Better BibTeX verändert Angaben beim Export,** siehe `docs/zotero-einrichtung.md` Abschnitt 3:
  ohne Einstellung keine URLs bei `@misc`, englische Großschreibung deutscher Titel ohne `Sprache = de`.
- **Offen:** Prüfung mit einem echten PDF und echten Markierungen (Anhang an eine Quelle ziehen,
  zwei Stellen markieren). Erst danach ist die Zeile „Markierungen mit Seite“ bestätigt.
