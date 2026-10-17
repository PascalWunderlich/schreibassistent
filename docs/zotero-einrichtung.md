# Zotero + Better BibTeX einrichten

Ziel: Zotero verwaltet die Quellen; Better BibTeX (BBT) schreibt sie automatisch nach
`System/quellen.bib` im Arbeits-Vault. In Typst zitierst du mit `@citekey`, z. B. `@mueller2024`.
Der Citekey verbindet Zotero, die spätere Quellenseite und den Text (siehe `docs/GLOSSARY.md`).

## 1. Installieren

1. Zotero 7 installieren: <https://www.zotero.org/download/>
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

## 3. Automatischer Export nach `System/quellen.bib`

1. In Zotero eine **Sammlung** für die Arbeit anlegen (z. B. „Praxisarbeit“) und die Quellen hineinlegen.
2. Rechtsklick auf die Sammlung → **Sammlung exportieren …**
3. Format **Better BibTeX** wählen, Häkchen bei **Keep updated** (Aktualisieren) setzen → OK.
4. Als Ziel `System/quellen.bib` im Arbeits-Vault wählen.
5. **Zotero → Einstellungen → Better BibTeX → Automatic export:** auf **On Change**
   („bei Änderung“) stellen. Jede Änderung in Zotero landet dann sofort in der `.bib`.

Kontrolle: Unter **Einstellungen → Better BibTeX → Automatic export** steht der Export mit
Pfad `…/System/quellen.bib`. Eine Änderung an einem Titel in Zotero muss nach wenigen
Sekunden in der Datei stehen.

## 4. In Typst verwenden

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

## 5. Angaben vollständig halten (FHDW-Hinweise S. 9–10)

Die Vorlage gibt aus, was in Zotero steht. Fehlt ein Feld, fehlt es auch im Quellverzeichnis.

| Zotero-Typ | Pflichtfelder laut FHDW |
|---|---|
| Buch | Autoren, Titel, Verlag, Ort (optional), Auflage (falls nicht erste), Jahr |
| Zeitschriftenartikel | Autoren, Titel, Zeitschrift, Band, Ausgabe (Heft), Seiten, Jahr |
| Konferenzbeitrag | Autoren, Titel, Herausgeber, Tagungsband, Reihe/Band (optional), Seiten, Jahr |
| Webseite / Blogpost | Autoren (falls vorhanden), Titel, Name der Website, Datum, URL, **Zugriffsdatum** |

Verlags-BibTeX (Springer, ACM, IEEE) ist meist vollständig; Einträge von anderen Seiten prüfen.
