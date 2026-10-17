# sidecars/

Externe Programme, die die App als Tauri-Sidecar startet:

- `tinymist` – Typst-Sprachserver für den Editor (ab P3, Modul `lsp`)
- `docling/` – PDF-Extraktion mit Seitenangaben, Python über uv (ab P7, Modul `ingest`)

Binärdateien kommen nicht ins Repo; ein Skript lädt sie für den Build.
