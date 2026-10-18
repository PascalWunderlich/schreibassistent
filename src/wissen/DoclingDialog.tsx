import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Button } from "../components/ui/button";
import { DoclingStatus, ingestWerkzeugInstallieren } from "./api";

interface Props {
  status: DoclingStatus;
  /** Docling ist eingerichtet. */
  onDone: () => void;
  onCancel: () => void;
}

type Stage = "frage" | "laeuft" | "fehler";

/** Erstinstallation von Docling: erst fragen (Netz, Größe), dann laden mit Fortschritt. Das Einlesen läuft danach lokal. */
export function DoclingDialog({ status, onDone, onCancel }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [stage, setStage] = useState<Stage>("frage");
  const [line, setLine] = useState("");
  const [error, setError] = useState<string | null>(null);
  useEffect(() => ref.current?.showModal(), []);

  useEffect(() => {
    const unlisten = listen<string>("docling_fortschritt", (e) => setLine(e.payload));
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  async function install() {
    setError(null);
    setLine("");
    setStage("laeuft");
    try {
      await ingestWerkzeugInstallieren();
      ref.current?.close();
      onDone();
    } catch (e) {
      setError(String(e));
      setStage("fehler");
    }
  }

  const gb = (status.groesse_mb / 1000).toLocaleString("de-DE", { maximumFractionDigits: 1 });
  return (
    <dialog
      ref={ref}
      aria-labelledby="docling-titel"
      // Während des Ladens nicht wegklicken: der Vorgang ist nicht abbrechbar.
      onCancel={(e) => stage === "laeuft" && e.preventDefault()}
      onClose={() => stage !== "laeuft" && onCancel()}
      className="m-auto w-[440px] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
    >
      <div className="flex flex-col gap-3 p-4">
        <h2 id="docling-titel" className="text-base font-semibold">
          Docling einrichten
        </h2>
        {stage === "frage" && (
          <>
            <p className="text-sm">
              Zum Einlesen von PDFs braucht der Schreibassistent einmalig das Werkzeug Docling. Es wird aus dem Internet geladen
              (ca. {gb} GB, je nach Verbindung einige Minuten) und danach lokal genutzt. Beim Laden werden keine Dateien von dir übertragen.
            </p>
            {!status.uv && <p className="text-sm text-destructive">Das Hilfsprogramm uv fehlt (Installation der App unvollständig).</p>}
            <div className="flex justify-end gap-2">
              <Button size="sm" variant="outline" onClick={() => ref.current?.close()}>
                Abbrechen
              </Button>
              <Button size="sm" disabled={!status.uv} onClick={install}>
                Jetzt laden
              </Button>
            </div>
          </>
        )}
        {stage === "laeuft" && (
          <>
            <p className="text-sm">Docling wird geladen. Bitte das Fenster offen lassen.</p>
            <progress aria-label="Docling wird geladen" className="h-2 w-full" />
            <p className="min-h-5 truncate text-xs text-muted-foreground" aria-live="polite">
              {line}
            </p>
          </>
        )}
        {stage === "fehler" && (
          <>
            <p role="alert" className="text-sm text-destructive">
              {error}
            </p>
            <div className="flex justify-end gap-2">
              <Button size="sm" variant="outline" onClick={() => ref.current?.close()}>
                Schließen
              </Button>
              <Button size="sm" onClick={install}>
                Erneut versuchen
              </Button>
            </div>
          </>
        )}
      </div>
    </dialog>
  );
}
