# sidecars/

Externe Programme und ihre Konfiguration:

- `tinymist` – Typst-Sprachserver (Modul `lsp`); derzeit aus Homebrew, später als Sidecar (ROADMAP).
- `vale/` – Konfiguration und Stil `FHDW` für Vale (Modul `pruefung`, P5). Vale und
  `typst2vast` kommen derzeit aus Homebrew bzw. `cargo install typst2vast`.
- LTeX+ (`ltex-ls-plus`, MPL-2.0, enthält LanguageTool LGPL-2.1 und Java) – Rechtschreibung
  und Grammatik (P5); derzeit aus Homebrew, beim Bündeln Lizenztexte beilegen.
- `docling/` – PDF-Extraktion mit Seitenangaben, Python über uv (ab P7, Modul `ingest`).

Binärdateien kommen nicht ins Repo.
