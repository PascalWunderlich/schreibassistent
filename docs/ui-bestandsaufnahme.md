# UI-Bestandsaufnahme (P9, Schritt 1)

Stand 2026-10-18, Screenshot: `docs/ui/2026-10-18-ist-seiteneditor.png` (Testarbeit, Seitenmodus).
Pascal: „null nutzerfreundlich, unübersichtlich“. Maßstab: Skizze im Vault („Oberfläche und Editor“).

## Befunde

| # | Bereich | Befund | Kernaufgabe | Reibungsliste |
|---|---|---|---|---|
| 1 | Seitenleiste | Keine Kapitelnavigation oben (Skizze: „Arbeit: 1 Einleitung, 2 Grundlagen …“); Kapitel nur im Dateibaum weit unten | 2 Kapitel schreiben | Design Seitenleiste |
| 2 | Seitenleiste | Überschriften „Dieses Kapitel: Einleitung“, „Alle Quellen“ riesig und fett, Inhalt klein: Hierarchie verkehrt | 2, 3 | Design Seitenleiste |
| 3 | Seitenleiste | „Konzepte erzeugen“ steht als loser Knopf zwischen Überschrift und Liste | – | – |
| 4 | Kopfzeile | Titel doppelt („Schreibassistent“ im Fenster und in der Leiste), Knöpfe unterschiedlich groß, kein Hinweis auf KI-Anbieter (Skizze: „KI: Ollama lokal · PDF“) | 1, 6 | – |
| 5 | Mitte | Seite nicht als Blatt erkennbar (weiß auf weiß, kein Schatten/Rand); großer Leerraum oben | 2 | – |
| 6 | Mitte | Markierungen am Absatz als graue Striche ohne Erklärung | 4 | – |
| 7 | Prüfungen | 24 Hinweise, davon die meisten Metadaten der `.bib` („Zugriffsdatum fehlt“) – das Wichtige (Seitenumfang) geht unter; jede Zeile mit „WARNUNG“ in Großbuchstaben | 4 Prüfhinweis finden | – |
| 8 | Prüfungen | Feste Regel und KI-Hinweis optisch nicht unterscheidbar (Skizze: Etiketten „Stil“, „Form“, „OK“) | 4, 5 | – |
| 9 | Assistent | Kopf bricht um („deepseek-\nchat“), drei Knöpfe + ⚙ im Kopf, fünf Aufgabenknöpfe brechen um, Eingabefeld unterhalb des sichtbaren Bereichs | 5 Assistent fragen | Kontext-Auswahl übersehen |
| 10 | Assistent | Kontext „kein Absatz gewählt“ als Auswahlliste ganz unten | 5 | Kontext-Auswahl übersehen |
| 11 | Statusleiste | Seitenzahl rot ohne Erklärung; KI-Protokoll fehlt (Skizze: „KI-Protokoll: 3 Einträge“) | 6 | – |
| 12 | Allgemein | Kein dunkles Design, keine Tastenkürzel außer ⌘↩, kein Ersteinstieg | alle | Zotero-Einrichtung |
| 13 | Allgemein | Jede Komponente eigenes CSS (≈ 1 000 Zeilen `App.css`), Farben teils fest im Code | – | – |

## Bibliotheken (Recherche 2026-10-18)
- **shadcn/ui** (MIT): Radix-Komponenten + Tailwind v4, werden ins Repo kopiert (kein Fork, frei anpassbar),
  Design-Tokens als CSS-Variablen hell/dunkel, zugängliche Tastaturbedienung. Dazu **lucide** (Icons),
  **cmdk** (Befehlspalette ⌘K), **react-resizable-panels** (Spalten), **sonner** (Meldungen).
- **AI Elements** (Vercel, auf shadcn): einzelne Bausteine wie Prompt-Eingabe, Quellen/Inline-Zitat passen
  zu „Originalstelle zeigen“; Lizenz vor Übernahme prüfen.
- **assistant-ui** (MIT): vollständige Chat-Laufzeit (Threads, Streaming) – passt schlecht, weil unser
  Assistent kein Chat ist (strukturierte Hinweise, kein Einfügen). Nicht empfohlen.
