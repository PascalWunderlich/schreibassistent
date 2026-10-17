# Schreibassistent

Lokale Desktop-App (macOS zuerst) für wissenschaftliche Arbeiten mit Typst.
Du schreibst den Text selbst; die App setzt ihn nach Vorlage und Regelprofil,
verwaltet Quellen und Wissen und prüft die Form.
KI-Feedback gibt es nur auf Nachfrage – die KI schreibt nie einen Satz der Arbeit.

## Voraussetzungen

- macOS (Apple Silicon) mit Xcode Command Line Tools
- Rust stable (über `rustup`)
- Node LTS (24) und pnpm
- Typst-CLI und Tinymist

Installation mit Homebrew:

```sh
brew install rustup node@24 pnpm typst tinymist
rustup default stable
```

`node@24` und `rustup` sind bei Homebrew „keg-only“, deshalb in `~/.zprofile`:

```sh
export PATH="/opt/homebrew/opt/node@24/bin:/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
```

## Starten

```sh
pnpm install
pnpm tauri dev                                      # App starten
cargo test --manifest-path src-tauri/Cargo.toml     # Rust-Tests
pnpm test                                           # Frontend-Tests
```

## Benutzen

1. `typst/installieren.sh` einmal ausführen (Vorlage `@local/fhdw-arbeit`).
2. `pnpm tauri dev`, „Arbeit öffnen …“ und den Ordner der Arbeit wählen
   (Arbeits-Vault mit `Arbeit/main.typ`, Aufbau siehe `docs/DATA-FORMATS.md`).
3. Auf einen Absatz klicken und schreiben. Enter: neuer Absatz, Shift+Enter:
   Zeilenumbruch im Quelltext, Esc oder Klick daneben: fertig. `@` bzw. `#` schlägt
   Citekeys, Labels und Funktionen vor; ⌘-Klick springt zur Definition.
4. „Quelltext“ oben zeigt die ganze Datei mit den Seiten daneben (Tabellen, Abbildungen).
5. Gespeichert wird automatisch; mit eigenem Git-Repo entsteht höchstens alle
   5 Minuten ein lokaler Commit „autosave“. „PDF“ exportiert die Arbeit.

## Dokumentation

- `CLAUDE.md` – Leitplanken und Arbeitsweise
- `docs/VISION.md`, `docs/ARCHITECTURE.md`, `docs/DATA-FORMATS.md`
- `docs/ROADMAP.md` – Phasen und aktueller Stand
- `docs/DECISIONS.md`, `docs/GLOSSARY.md`
- `docs/vault.md` – Verweis auf die ausführliche Planung im Obsidian-Vault
