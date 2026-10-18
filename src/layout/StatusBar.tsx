import { FrageErgebnis } from "../assistent/api";

export type SaveState = "gespeichert" | "speichert …" | "Fehler beim Speichern";

interface Props {
  /** Seitenvorgabe aus dem Regelprofil (für die Art der Arbeit). */
  range: [number, number] | null;
  /** Offene Hinweise der Prüfungen. */
  openHints: number;
  /** Seiten des Inhaltsteils. */
  contentPages: number | null;
  saveState: SaveState;
  lastAutosave: number | null;
  git: boolean;
  gitError: string | null;
  /** Fehler laut Tinymist in der ganzen Arbeit. */
  errorCount: number;
  /** Letzte KI-Antwort (Tokens). */
  lastAnswer?: FrageErgebnis | null;
}

const time = (seconds: number) =>
  new Date(seconds * 1000).toLocaleTimeString("de-DE", { hour: "2-digit", minute: "2-digit" });

/** Unten: Stand der Arbeit. */
export function StatusBar({ range, openHints, contentPages, saveState, lastAutosave, git, gitError, errorCount, lastAnswer }: Props) {
  return (
    <footer className="statusbar">
      <span
        className={contentPages !== null && range && (contentPages < range[0] || contentPages > range[1]) ? "warn" : "ok"}
        title="Inhaltsteil: von Seite 1 bis vor das Quellverzeichnis"
      >
        {contentPages === null
          ? "Inhaltsteil: –"
          : range
            ? `${contentPages} von ${range[0]}–${range[1]} Seiten`
            : `${contentPages} Seiten`}
      </span>
      <span className={openHints > 0 ? "warn" : ""}>{openHints === 1 ? "1 offener Hinweis" : `${openHints} offene Hinweise`}</span>
      <span className={errorCount > 0 ? "warn" : ""}>{errorCount === 1 ? "1 Fehler" : `${errorCount} Fehler`}</span>
      <span className="spacer" />
      {lastAnswer && (
        <span title={`Letzte KI-Anfrage: ${lastAnswer.modell} (${lastAnswer.anbieter})`}>
          KI: {lastAnswer.tokens.ein ?? "?"} → {lastAnswer.tokens.aus ?? "?"} Tokens
        </span>
      )}
      <span className={saveState === "Fehler beim Speichern" ? "warn" : ""}>{saveState}</span>
      <span className={gitError ? "warn" : ""} title={gitError ?? undefined}>
        {!git ? "keine Sicherung (Git)" : gitError ? "Sicherung fehlgeschlagen" : lastAutosave ? `Sicherung ${time(lastAutosave)}` : "Sicherung bereit"}
      </span>
    </footer>
  );
}
