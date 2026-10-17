export type SaveState = "gespeichert" | "speichert …" | "Fehler beim Speichern";

interface Props {
  pageCount: number | null;
  saveState: SaveState;
  lastAutosave: number | null;
  git: boolean;
  gitError: string | null;
  /** Fehler laut Tinymist in der ganzen Arbeit. */
  errorCount: number;
}

const time = (seconds: number) =>
  new Date(seconds * 1000).toLocaleTimeString("de-DE", { hour: "2-digit", minute: "2-digit" });

/** Unten: Stand der Arbeit. Seitenumfang des Inhaltsteils folgt in Sitzung 5. */
export function StatusBar({ pageCount, saveState, lastAutosave, git, gitError, errorCount }: Props) {
  return (
    <footer className="statusbar">
      <span>{pageCount === null ? "–" : `${pageCount} Seiten`}</span>
      <span className={errorCount > 0 ? "warn" : ""}>{errorCount === 1 ? "1 Fehler" : `${errorCount} Fehler`}</span>
      <span className="spacer" />
      <span className={saveState === "Fehler beim Speichern" ? "warn" : ""}>{saveState}</span>
      <span className={gitError ? "warn" : ""} title={gitError ?? undefined}>
        {!git ? "keine Sicherung (Git)" : gitError ? "Sicherung fehlgeschlagen" : lastAutosave ? `Sicherung ${time(lastAutosave)}` : "Sicherung bereit"}
      </span>
    </footer>
  );
}
