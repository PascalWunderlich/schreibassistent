# sidecars/

Konfiguration und Skripte für Fremdprogramme. Die Programme selbst kommen **nicht** ins Repo:
`scripts/werkzeuge-bauen.sh` (macOS, Linux) bzw. `.ps1` (Windows) holt sie nach `build/werkzeuge/`
(Versionen und Prüfsummen: `werkzeuge.lock`, Lizenzen: `docs/werkzeuge-lizenzen.md`). Die App findet sie
über das Modul `werkzeuge` (App-Ressourcen, dann `PATH`).

- `vale/` – Konfiguration und Stil `FHDW` für Vale (Modul `pruefung`). Vale, `typst2vast`, Tinymist,
  LTeX+ (`ltex-ls-plus`, MPL-2.0, enthält LanguageTool LGPL-2.1 und Java) und `uv` werden mitgeliefert.
- `docling/` – PDF-Extraktion mit Seitenangaben, Python über `uv` (Modul `ingest`). Docling selbst wird nicht
  mitgeliefert, sondern nach Bestätigung beim ersten Einlesen geladen (`ingest::docling`).
