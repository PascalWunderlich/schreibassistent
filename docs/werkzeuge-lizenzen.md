# Mitgelieferte Werkzeuge und ihre Lizenzen

Der Schreibassistent liefert diese Programme unverändert mit (Ordner `werkzeuge/` in der App).
Sie werden als eigene Programme gestartet; die App enthält keinen Quelltext von ihnen.
Die Lizenztexte aus den Paketen liegen in `licenses/<Name>/`. Versionen und Prüfsummen: `VERSIONEN.txt`.

| Programm | Zweck | Lizenz | Quelle |
|---|---|---|---|
| Tinymist | Typst-Sprachserver (Vervollständigung, Fehler) | Apache-2.0 | https://github.com/Myriad-Dreamin/tinymist |
| Vale | Stilregeln | MIT | https://github.com/errata-ai/vale |
| typst2vast | Typst-Text für Vale | MIT | https://crates.io/crates/typst2vast |
| LTeX+ (`ltex-ls-plus`) | Rechtschreibung und Grammatik | MPL-2.0 | https://github.com/ltex-plus/ltex-ls-plus |
| LanguageTool (in LTeX+) | Regeln für Rechtschreibung und Grammatik | LGPL-2.1-or-later | https://languagetool.org |
| Java-Laufzeit (in LTeX+) | führt LTeX+ aus | GPL-2.0 mit Classpath-Ausnahme | https://openjdk.org |
| uv | holt bei Bedarf Docling (siehe `docs/installation.md`) | MIT oder Apache-2.0 | https://github.com/astral-sh/uv |

Docling (MIT) wird nicht mitgeliefert, sondern nach Bestätigung beim ersten Einlesen von Quellen geladen.
Die Vorlage `fhdw-arbeit` ist von clean-dhbw (MIT) abgeleitet; der Hinweis liegt in `typst/fhdw-arbeit/LICENSE`.
