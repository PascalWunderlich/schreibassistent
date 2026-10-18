# Installation (macOS, Windows, Linux)

Textfassung ohne Bilder. Die Schritte sind noch **nicht auf frischen Konten ausprobiert**; die Abnahme
(macOS: Stoppuhr vom `.dmg` bis zum PDF, Ziel unter 10 Minuten ohne Terminal) steht in `docs/P10-plan.md`, Schritt D.
Windows und Linux sind bisher nur über die CI (Tests grün, Paket entsteht) belegt.

## Was du brauchst

| System | Paket | Voraussetzung |
|---|---|---|
| macOS (Apple Silicon, M1 oder neuer) | `.dmg` | – |
| Windows 10/11 (64 Bit) | `.exe` (Installer, ohne Administratorrechte) | WebView2 (in Windows 11 und aktuellem Windows 10 vorhanden) |
| Linux (64 Bit) | `.AppImage` oder `.deb` | WebKitGTK 4.1, ein Schlüsselspeicher (GNOME Keyring oder KWallet) für KI-Schlüssel |

Alles andere liegt in der App: Typst, Tinymist (Editor-Hilfen), Vale und LTeX+ (Prüfungen, LTeX+ samt Java), die Vorlage
„fhdw-arbeit“ und `uv`. Zotero ist freiwillig und wird nicht mitgeliefert.
Platzbedarf: rund 0,5 GB für die App; dazu einmalig ca. 2 GB, wenn du zum ersten Mal PDFs einliest (Docling, siehe unten).

## Herunterladen

Solange es keine Veröffentlichungen (Releases) gibt, kommen die Pakete aus der CI: auf GitHub unter **Actions** den letzten
grünen Lauf des Zweigs öffnen und unter **Artifacts** das Paket für dein System laden
(`schreibassistent-macos-arm64`, `-windows-x64`, `-linux-x64`; die Datei ist in einer ZIP-Datei).

## Erster Start: Warnung des Systems umgehen

Die App ist **nicht notarisiert und nicht von einem Herausgeber signiert** (kein Entwicklerkonto). Das System warnt deshalb
einmal. Das ist erwartet; die Warnung sagt nichts über den Inhalt.

### macOS
1. `.dmg` öffnen, **Schreibassistent** in den Ordner **Programme** ziehen.
2. Im Ordner Programme mit **Rechtsklick → Öffnen** starten und im Dialog erneut **Öffnen** wählen.
3. Zeigt macOS nur „… kann nicht geöffnet werden“: **Systemeinstellungen → Datenschutz & Sicherheit**, nach unten
   zu „Schreibassistent wurde blockiert“ und **Trotzdem öffnen** wählen, dann bestätigen.
Ab dem zweiten Start öffnet sich die App normal.

### Windows
1. Installer ausführen. Bei „Der Computer wurde durch Windows geschützt“ (SmartScreen) auf **Weitere Informationen** und
   dann **Trotzdem ausführen** klicken.
2. Die App wird für dein Benutzerkonto installiert (kein Administrator nötig) und erscheint im Startmenü.

### Linux
- **AppImage:** Datei ausführbar machen (Rechtsklick → Eigenschaften → „Ausführen erlauben“, oder `chmod +x`) und starten.
- **deb:** über den Software-Installer des Systems öffnen (Ubuntu/Debian).

## Ersteinstieg

Beim allerersten Start fragt ein Fenster in drei Schritten, jeder lässt sich überspringen:
1. **Arbeit:** vorhandenen Ordner öffnen oder „Neue Arbeit …“ (Titel, Autor, Art, Ort).
2. **Quellen:** „Quellen aus Zotero verbinden“ prüft, ob Zotero läuft (Einrichtung: `docs/zotero-einrichtung.md`), oder
   „.bib-Datei wählen …“ übernimmt eine vorhandene Literaturdatei. „Ohne Quellen starten“ geht ebenso.
3. **KI:** Einstellungen öffnen und einen Anbieter samt API-Schlüssel eintragen, oder überspringen. Der Schlüssel liegt im
   Schlüsselspeicher deines Systems (macOS Schlüsselbund, Windows Anmeldeinformationsverwaltung, Linux Secret Service).

Danach schreibst du in Typst; **Neue Arbeit …** und **Arbeit öffnen …** stehen im Menü Ablage bzw. Datei.
Beim ersten Öffnen einer Arbeit startet im Hintergrund die Rechtschreibprüfung (LTeX+, kalt rund 15 Sekunden).

## Quellen einlesen: Docling einmalig laden

Zum ersten „Einlesen …“ einer Quelle fragt die App: **„Docling einrichten“**. Mit „Jetzt laden“ lädt sie einmalig Python,
Docling und ein Layout-Modell aus dem Internet (ca. 2 GB; unter Linux kann es mehr sein, weil PyTorch dort GPU-Pakete mitbringt)
in den Datenordner der App und zeigt den Fortschritt. Danach liest die App PDFs lokal. Ohne Netz erscheint eine klare Meldung;
du kannst es später erneut versuchen. Wer nichts einliest, braucht Docling nie.

## Wo liegen meine Daten?

- **Deine Arbeit** (`Arbeit/`, `Notizen/`, `Wissen/`, `System/`) liegt in dem Ordner, den du gewählt hast (reiner Text, mit Git gesichert).
- **App-Einstellungen** (ohne Schlüssel): Konfigurationsordner der App
  (macOS `~/Library/Application Support/de.double-efficency.schreibassistent`, Windows `%APPDATA%\de.double-efficency.schreibassistent`,
  Linux `~/.config/de.double-efficency.schreibassistent`).
- **Docling** (nach der Einrichtung): Unterordner `docling` im Datenordner der App.
- **Vorlage und Typst-Pakete:** beim Start in den Typst-Paketordner des Systems kopiert
  (macOS `~/Library/Application Support/typst/packages`, Windows `%APPDATA%\typst\packages`, Linux `~/.local/share/typst/packages`).

## Deinstallieren

App löschen (macOS: aus Programme in den Papierkorb; Windows: Einstellungen → Apps; Linux: AppImage löschen bzw. `sudo apt remove schreibassistent`).
Deine Arbeit bleibt, wo sie liegt. Wer alles entfernen will, löscht zusätzlich die oben genannten App-Ordner.

## Probleme

| Meldung | Was tun |
|---|---|
| „Vorlage nicht installiert“ im Protokoll, Satz zeigt „package not found“ | App neu starten; der Typst-Paketordner muss beschreibbar sein |
| „Tinymist/Vale/LTeX+ nicht gefunden (Installation der App unvollständig)“ | Paket neu installieren; die Werkzeuge liegen im Ordner `werkzeuge` der App |
| „Keine Internetverbindung“ bei Docling | mit dem Netz verbinden und „Erneut versuchen“ |
| Kein Schlüsselspeicher unter Linux | GNOME Keyring oder KWallet installieren und anmelden; ohne ihn lassen sich keine API-Schlüssel speichern |
