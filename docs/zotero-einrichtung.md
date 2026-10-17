# Zotero + Better BibTeX einrichten

Ziel: Zotero verwaltet die Quellen; Better BibTeX (BBT) schreibt sie automatisch nach
`System/quellen.bib` im Arbeits-Vault. In Typst zitierst du mit `@citekey`, z. B. `@mueller2024`.
Der Citekey verbindet Zotero, die spätere Quellenseite und den Text (siehe `docs/GLOSSARY.md`).

## 1. Installieren

Getestet am 2026-10-17 mit **Zotero 10.0.6** und **Better BibTeX 9.0.71** (freigegeben für Zotero 8–10).

1. Zotero installieren: <https://www.zotero.org/download/>
   (oder `brew install --cask zotero`).
2. Better BibTeX herunterladen: neueste `.xpi`-Datei unter
   <https://github.com/retorquere/zotero-better-bibtex/releases/latest>.
3. In Zotero: **Werkzeuge → Plugins** → Zahnrad → **Plugin aus Datei installieren …** →
   die `.xpi` wählen → Zotero neu starten.
4. Optional für den Browser: Zotero Connector (Safari/Chrome), um Quellen mit einem Klick zu erfassen.

## 2. Citekey-Format `<autor><jahr>`

**Zotero → Einstellungen → Better BibTeX → Citation keys**

- **Citation key formula:**

  ```
  auth.lower + year
  ```

  Ergebnis: `sutton2018`, `greenyer2013`. Bei gleichem Autor und Jahr hängt BBT
  automatisch `a`, `b`, … an (`mueller2024a`).
- Danach bestehende Einträge einmal neu verschlüsseln: alle Einträge markieren →
  Rechtsklick → **Better BibTeX → Refresh BibTeX key**.
- Einen Citekey nicht mehr ändern, sobald er im Text verwendet wird. Bei Bedarf festpinnen:
  Rechtsklick → **Better BibTeX → Pin BibTeX key**.

## 3. Einstellungen, ohne die Angaben verloren gehen

Im Test (P4) fielen beim Weg Zotero → `quellen.bib` Angaben weg, die die FHDW verlangt.
Die Namen der Optionen sind sinngemäß angegeben und können je nach Version leicht abweichen:

| Problem | Folge im Quellverzeichnis | Einstellung |
|---|---|---|
| Better BibTeX schreibt bei Internetquellen (`@misc`) die URL standardmäßig nicht | URL und Zugriffsdatum fehlen (FHDW S. 10) | **Einstellungen → Better BibTeX → Export → „URLs in den BibTeX-Export aufnehmen“: im `url`-Feld** |
| Ohne Sprache behandelt Better BibTeX Titel als Englisch und schreibt jedes Wort groß | „Vom Softwaremonolithen **Zu** Microservices … **Am** Praxisbeispiel“ | bei deutschen Quellen in Zotero das Feld **Sprache = `de`** setzen |
| Beim Import einer `.bib` schreibt Zotero Titel klein („softwaremonolithen“) | Substantive klein | vor einem Import **Better BibTeX → Import → „Titel in Satzschreibweise umwandeln“ ausschalten**; besser Quellen über den Browser erfassen |

## 4. Automatischer Export nach `System/quellen.bib`

1. In Zotero eine **Sammlung** für die Arbeit anlegen (z. B. „Praxisarbeit“) und die Quellen hineinlegen.
   **Erst die Quellen, dann der Export:** Der Export überschreibt `quellen.bib` mit dem Inhalt
   der Sammlung; eine leere Sammlung ergäbe eine leere Datei.
2. Rechtsklick auf die Sammlung → **Sammlung exportieren …**
3. Format **Better BibTeX** wählen, Häkchen bei **Keep updated** (Aktualisieren) setzen → OK.
4. Als Ziel `System/quellen.bib` im Arbeits-Vault wählen.
5. **Zotero → Einstellungen → Better BibTeX → Automatic export:** auf **On Change**
   („bei Änderung“) stellen. Jede Änderung in Zotero landet dann sofort in der `.bib`.

Kontrolle: Unter **Einstellungen → Better BibTeX → Automatic export** steht der Export mit
Pfad `…/System/quellen.bib`. Eine Änderung an einem Titel in Zotero muss nach wenigen
Sekunden in der Datei stehen.

## 5. In Typst verwenden

```typst
#show: fhdw-thesis.with(
  …,
  bibliography: bibliography("../System/quellen.bib"),
)

Wie Sutton und Barto zeigen @sutton2018 …
„Wörtliches Zitat.“ @greenyer2013[S. 1]
```

Kompilieren mit dem Arbeits-Vault als Root, damit Typst `System/` erreicht:

```sh
typst compile --root <Arbeits-Vault> <Arbeits-Vault>/Arbeit/main.typ
```

In VS Code mit Tinymist: den Arbeits-Vault als Ordner öffnen, dann ist er automatisch der Root.

## 6. Angaben vollständig halten (FHDW-Hinweise S. 9–10)

Die Vorlage gibt aus, was in Zotero steht. Fehlt ein Feld, fehlt es auch im Quellverzeichnis.

| Zotero-Typ | Pflichtfelder laut FHDW |
|---|---|
| Buch | Autoren, Titel, Verlag, Ort (optional), Auflage (falls nicht erste), Jahr |
| Zeitschriftenartikel | Autoren, Titel, Zeitschrift, Band, Ausgabe (Heft), Seiten, Jahr |
| Konferenzbeitrag | Autoren, Titel, Herausgeber, Tagungsband, Reihe/Band (optional), Seiten, Jahr |
| Webseite / Blogpost | Autoren (falls vorhanden), Titel, Name der Website, Datum, URL, **Zugriffsdatum** |

Verlags-BibTeX (Springer, ACM, IEEE) ist meist vollständig; Einträge von anderen Seiten prüfen.
