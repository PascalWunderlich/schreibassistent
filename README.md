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

## Dokumentation

- `CLAUDE.md` – Leitplanken und Arbeitsweise
- `docs/VISION.md`, `docs/ARCHITECTURE.md`, `docs/DATA-FORMATS.md`
- `docs/ROADMAP.md` – Phasen und aktueller Stand
- `docs/DECISIONS.md`, `docs/GLOSSARY.md`
- `docs/vault.md` – Verweis auf die ausführliche Planung im Obsidian-Vault
