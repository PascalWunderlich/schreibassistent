export type SaveState = "gespeichert" | "speichert …" | "Fehler beim Speichern";

/** Vorgabe Praxisarbeit (FHDW-Hinweise S. 3); vorerst fest, später aus dem Regelprofil (P5). */
const PAGE_RANGE = { min: 20, max: 22 };

interface Props {
  /** Seiten des Inhaltsteils. */
  contentPages: number | null;
  saveState: SaveState;
  lastAutosave: number | null;
  git: boolean;
  gitError: string | null;
  /** Fehler laut Tinymist in der ganzen Arbeit. */
  errorCount: number;
}

const time = (seconds: number) =>
  new Date(seconds * 1000).toLocaleTimeString("de-DE", { hour: "2-digit", minute: "2-digit" });

/** Unten: Stand der Arbeit. */
export function StatusBar({ contentPages, saveState, lastAutosave, git, gitError, errorCount }: Props) {
  return (
    <footer className="statusbar">
      <span
        className={contentPages !== null && (contentPages < PAGE_RANGE.min || contentPages > PAGE_RANGE.max) ? "warn" : "ok"}
        title="Inhaltsteil: von Seite 1 bis vor das Quellverzeichnis"
      >
        {contentPages === null ? "Inhaltsteil: –" : `${contentPages} von ${PAGE_RANGE.min}–${PAGE_RANGE.max} Seiten`}
      </span>
      <span className={errorCount > 0 ? "warn" : ""}>{errorCount === 1 ? "1 Fehler" : `${errorCount} Fehler`}</span>
      <span className="spacer" />
      <span className={saveState === "Fehler beim Speichern" ? "warn" : ""}>{saveState}</span>
      <span className={gitError ? "warn" : ""} title={gitError ?? undefined}>
        {!git ? "keine Sicherung (Git)" : gitError ? "Sicherung fehlgeschlagen" : lastAutosave ? `Sicherung ${time(lastAutosave)}` : "Sicherung bereit"}
      </span>
    </footer>
  );
}
