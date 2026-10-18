# P10: Einfache Installation (macOS, Windows, Linux)

Übergabe für die Fortsetzung in einer Cloud-Sitzung. Stand 2026-10-18, freigegeben von Pascal
(inklusive Anpassung von `CLAUDE.md`: Plattformen, Leitplanke 4 und 7).
Abnahme laut Roadmap: frisches macOS-Konto, Download bis PDF < 10 Min. ohne Terminal.

## Freigegebene Entscheidungen
1. **Kein Apple-Developer-Account:** App wird ad-hoc signiert. macOS: einmal Rechtsklick „Öffnen“
   (oder Systemeinstellungen → „Trotzdem öffnen“). Windows: SmartScreen „Trotzdem ausführen“.
   Anleitung je System mit Bildern. Notarisierung später, wenn ein Account da ist.
2. **Nur Apple Silicon** auf macOS (arm64). Windows x64, Linux x64.
3. **LTeX+ (mit Java) und Vale werden mitgeliefert**, dazu Tinymist, `typst2vast`, `uv`.
   LTeX+ startet beim Öffnen einer Arbeit im Hintergrund (Kaltstart ~15 s).
4. **Zotero optional:** Ersteinstieg „Quellen aus Zotero verbinden“ oder „Ohne Quellen starten“;
   `.bib` von Hand bleibt möglich.
5. **Docling wird nicht mitgeliefert** (mehrere GB). `uv` liegt in der App; beim ersten
   „Quellen einlesen“ Bestätigung („lädt einmalig ca. 2 GB“), dann Download mit Fortschritt.
   Leitplanke 4 gilt für diese einmalige Erstinstallation nicht.
6. **Binärdateien kommen nicht ins Repo.** Der Build (CI) holt die Release-Pakete je Ziel.

## Was heute macOS-fest ist (Befund, im Code belegt)
| Stelle | Problem |
|---|---|
| `lsp/mod.rs` `find_tinymist`, `pruefung/ltex.rs` `find_ltex`, `pruefung/vale.rs` `find_vale`/`search_path`, `ingest/extrakt.rs` `find_uv` | suchen nur `PATH` und `/opt/homebrew/bin` (`~/.cargo/bin`, `~/.local/bin`) |
| `projekt/git.rs` | ruft das Programm `git` auf; auf frischem Windows-Konto nicht vorhanden |
| `protokoll/mod.rs`, `ingest/mod.rs` | `Command::new("date")` (nicht unter Windows) |
| `projekt/mod.rs`, `lsp/mod.rs` (Tests) | `cp -R` |
| `satz/mod.rs` (Test) | feste Pfade `/opt/homebrew/bin/typst` |
| `Cargo.toml` | `keyring` nur mit `apple-native` |
| `pruefung/vale.rs` `config()` | Pfad über `CARGO_MANIFEST_DIR` (nur Entwicklung) |
| Frontend | Kürzel prüfen nur `metaKey`; Beschriftungen mit „⌘“; „Im Finder zeigen“ |
| `src-tauri/src/menu.rs` | App-Menü mit `hide`, `hide_others`, `show_all`, nur macOS |
Unkritisch: Typst-Pakete (`satz/world.rs`, typst-kit) und die Schrift New Computer Modern.

## Schritte
### A. Unterbau (läuft auf allen Systemen)
1. **`werkzeuge`-Modul** (`src-tauri/src/werkzeuge.rs`): `init(resource_dir)` beim Start
   (`lib.rs`, `app.path().resource_dir()`), `find(name)` sucht zuerst `<Ressourcen>/werkzeuge/bin/`,
   dann `PATH`; für Entwicklung zusätzlich bekannte Orte je System. Alle vier `find_*` darauf umstellen,
   Fehlermeldungen ohne „brew install“. Vale-Konfiguration und Stil `FHDW` aus den Ressourcen
   (`werkzeuge/vale/`), im Entwicklungsmodus aus `sidecars/vale/`. Tests: Reihenfolge, fehlendes Programm.
2. **Git ohne Programm:** `projekt/git.rs` auf die Bibliothek `git2` (vendored libgit2) umstellen:
   `is_own_repo`, `init`, `autosave(force)` mit 5-Minuten-Regel, Benutzername/-mail mit Rückfall
   („Schreibassistent“), nie in ein fremdes Repo (bestehende Tests als Vorlage, Verhalten gleich lassen).
3. **Plattformneutral:** `date` durch die vorhandene Rust-Zeitfunktion (`satz::datetime_from_unix`) ersetzen;
   Tests ohne `cp -R` (Rust-Kopierfunktion, z. B. `projekt::neu::copy_to`) und ohne feste Pfade; `keyring` mit
   `apple-native`, `windows-native`, `sync-secret-service` je Ziel (`[target.'cfg(...)'.dependencies]`);
   `menu.rs`: App-Menü nur `#[cfg(target_os = "macos")]`, auf Windows/Linux Menü „Datei“ mit „Beenden“,
   „Einstellungen …“ unter „Bearbeiten“ oder „Datei“.
4. **Frontend:** Hilfsfunktion `isMac` und `mod(e)` (`metaKey` auf macOS, `ctrlKey` sonst) in `src/lib/`; alle
   Stellen mit `metaKey` darauf umstellen (`App.tsx`, `Assistant.tsx`, `CommandPalette.tsx`, `RightPanel.tsx`,
   `SourceEditor.tsx`, `Einfuegeleiste.tsx`, `lspExtensions.ts`); Anzeige „⌘“ → „Strg“ per `modLabel()`;
   „Im Finder zeigen“ → „Im Dateimanager zeigen“ außerhalb macOS. Die Menü-Beschleuniger (`CmdOrCtrl`)
   sind schon richtig. Tests für beide Fälle (`navigator.platform` mocken).

### B. Pakete und CI
5. **Skript `scripts/werkzeuge-bauen.sh`** (und `.ps1` für Windows) holt je Ziel in `build/werkzeuge/`
   (nicht im Repo, `.gitignore`): Tinymist, Vale, LTeX+ (Release mit JRE), `uv` (Release-Archive von GitHub),
   baut `typst2vast` (`cargo install --root`), kopiert Vorlage `typst/fhdw-arbeit`, `sidecars/vale/`,
   Lizenztexte (MPL-2.0 LTeX+, LGPL LanguageTool, Vale MIT, Tinymist Apache-2.0 …) nach `build/werkzeuge/`.
   Versionen und Prüfsummen (SHA-256) in `werkzeuge.lock` festschreiben.
6. **Tauri-Konfiguration:** `bundle.resources` auf `build/werkzeuge`; Ziele: macOS `app`/`dmg`, Windows `msi`
   oder `nsis`, Linux `appimage`/`deb`. macOS ad-hoc signieren (`signingIdentity: "-"`).
7. **CI** `.github/workflows/ci.yml` (Matrix `macos-latest`, `windows-latest`, `ubuntu-latest`): `pnpm install`,
   `pnpm test`, `cargo test`, `pnpm tauri build` mit Artefakten. Tests, die LTeX+/Zotero brauchen, nutzen
   die gebündelten Werkzeuge bzw. werden ohne Zotero übersprungen. Linux: `libwebkit2gtk-4.1-dev`,
   `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsecret-1-dev`.
8. **Vorlage beim Start installieren:** `@local/fhdw-arbeit:<Version>` aus den Ressourcen in den
   Typst-Paketordner des Systems kopieren, wenn sie fehlt oder die Version abweicht
   (typst-kit liefert den Pfad; macOS `~/Library/Application Support/typst/packages/local/`,
   Windows `%APPDATA%\typst\packages\local\`, Linux `~/.local/share/typst/packages/local/`).
   Test: leeres Datenverzeichnis → Paket danach da, zweiter Start ändert nichts.

### C. Erstinstallation und Ersteinstieg
9. **LTeX+ vorwärmen:** beim Öffnen einer Arbeit im Hintergrund starten (nicht blockierend, Status
   „Rechtschreibung startet …“ ist schon in der Statusleiste).
10. **Docling-Erstinstallation:** Befehl `ingest_werkzeug_status` (vorhanden/fehlt, geschätzte Größe),
    `ingest_werkzeug_installieren` (führt `uv` mit festem Zielordner in den App-Daten aus, meldet Fortschritt per
    Ereignis); Dialog mit Bestätigung vor dem ersten Einlesen; ohne Netz klare Meldung. Docling-Skript nutzt die
    App-Daten statt des Systemordners.
11. **Ersteinstieg:** Dialog beim ersten Start (Merker in den App-Einstellungen): Arbeit wählen/neu anlegen →
    Quellen (Zotero verbinden / `.bib` wählen / überspringen) → KI-Anbieter (Einstellungen öffnen / überspringen).
    Danach ist „Neue Arbeit“ aus dem Menü sofort setzbar.

### D. Abnahme und Doku
12. Kurze Anleitung `docs/installation.md` je System (Download, erster Start, Warnung umgehen).
13. **macOS-Abnahme:** neues Benutzerkonto auf dem Entwicklungs-Mac, Stoppuhr vom `.dmg` bis PDF, Abweichungen
    in `docs/ROADMAP.md`. **Windows/Linux:** nur CI-Nachweis (Tests grün, Bündel entsteht); Ausprobieren auf
    echter Hardware bleibt offen und gehört in „Später“, bis jemand es testet.
14. `docs/ROADMAP.md` und `docs/DECISIONS.md` aktualisieren, Zusammenfassung „Für den Vault“.

## Arbeitsweise in der Cloud-Sitzung
- Linux-Umgebung: Rust-Tests (`cargo test`) und `pnpm test` laufen dort; Schritte A, B (Skripte, CI-Datei), C
  lassen sich dort bauen und testen. Das Tauri-Fenster, macOS-Menü und die Abnahme auf frischem Konto nicht.
- Reihenfolge: A1 → A2 → A3 → A4 (je eigener Commit, Tests grün) → B → C → D.
- Nicht tun: keine Binärdateien committen; keine Schreibrechte der KI erweitern; Leitplanken in `CLAUDE.md` beachten.
- Der Stand von P9 steht in `docs/p9-zusammenfassung.md`; das Design System liegt als Artefakt
  (https://claude.ai/artifact/CkHwHXszYmysq3NMkVvzYG, privat).
