import { useEffect, useState } from "react";
import { Check, History, TriangleAlert } from "lucide-react";
import { protokollListe } from "../assistent/api";
import { ProtokollDialog } from "./ProtokollDialog";

export type SaveState = "gespeichert" | "speichert …" | "Fehler beim Speichern";

interface Props {
  /** Seitenvorgabe aus dem Regelprofil (für die Art der Arbeit). */
  range: [number, number] | null;
  /** Offene Hinweise der Prüfungen. */
  openHints: number;
  /** Seiten des Inhaltsteils. */
  contentPages: number | null;
  lastAutosave: number | null;
  git: boolean;
  gitError: string | null;
  /** Fehler laut Tinymist in der ganzen Arbeit. */
  errorCount: number;
  /** Rechtschreibprüfung (LTeX+) läuft oder startet noch. */
  spellchecking: boolean;
  /** Zählt hoch, wenn ein Protokolleintrag dazugekommen ist. */
  protokollVersion: number;
}

const time = (seconds: number) =>
  new Date(seconds * 1000).toLocaleTimeString("de-DE", { hour: "2-digit", minute: "2-digit" });

/** Wo steht die Arbeit gegenüber der Vorgabe: „14 / 20–22“, bei Abweichung mit Wort. */
export function pageStatus(pages: number | null, range: [number, number] | null): { text: string; verdict: "ok" | "zu kurz" | "zu lang" | null } {
  if (pages === null) return { text: "Seiten: –", verdict: null };
  if (!range) return { text: `${pages} Seiten`, verdict: null };
  const verdict = pages < range[0] ? "zu kurz" : pages > range[1] ? "zu lang" : "ok";
  return { text: `Seite ${pages} / ${range[0]}–${range[1]}`, verdict };
}

/** Unten: Stand der Arbeit, Hintergrundvorgänge und das KI-Protokoll. */
export function StatusBar({ range, openHints, contentPages, lastAutosave, git, gitError, errorCount, spellchecking, protokollVersion }: Props) {
  const [entries, setEntries] = useState<number | null>(null);
  const [logOpen, setLogOpen] = useState(false);
  useEffect(() => {
    protokollListe()
      .then((l) => setEntries(l.length))
      .catch(() => setEntries(null));
  }, [protokollVersion, logOpen]);

  const pages = pageStatus(contentPages, range);
  const off = pages.verdict === "zu kurz" || pages.verdict === "zu lang";
  const item = "flex items-center gap-1";

  return (
    <footer className="flex h-7 shrink-0 items-center gap-4 border-t border-border bg-background px-3 text-xs text-muted-foreground">
      <span className={`${item} ${off ? "font-semibold text-destructive" : ""}`} title="Inhaltsteil: von Seite 1 bis vor das Quellverzeichnis">
        {off ? <TriangleAlert className="size-3.5" /> : pages.verdict === "ok" ? <Check className="size-3.5 text-success" /> : null}
        {pages.text}
        {off && `, ${pages.verdict}`}
      </span>
      <span className={openHints > 0 ? "text-warning" : ""}>{openHints === 1 ? "1 offener Hinweis" : `${openHints} offene Hinweise`}</span>
      {errorCount > 0 && <span className="text-destructive">{errorCount === 1 ? "1 Fehler" : `${errorCount} Fehler`}</span>}
      {spellchecking && <span>Rechtschreibung startet …</span>}
      <span className="flex-1" />
      <span
        className={gitError ? "text-destructive" : ""}
        title={gitError ?? "Automatische Sicherung der Arbeit (Git)"}
      >
        {!git ? "Keine Sicherung (Git)" : gitError ? "Sicherung fehlgeschlagen" : lastAutosave ? `Sicherung ${time(lastAutosave)}` : "Sicherung bereit"}
      </span>
      <button onClick={() => setLogOpen(true)} className={`${item} rounded-sm px-1 hover:bg-muted hover:text-foreground`} title="Alle KI-Anfragen dieser Arbeit anzeigen">
        <History className="size-3.5" />
        KI-Protokoll: {entries === null ? "–" : entries === 1 ? "1 Eintrag" : `${entries} Einträge`}
      </button>
      {logOpen && <ProtokollDialog onClose={() => setLogOpen(false)} />}
    </footer>
  );
}
