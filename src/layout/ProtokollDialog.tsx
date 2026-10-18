import { useEffect, useRef, useState } from "react";
import { Button } from "../components/ui/button";
import { ProtokollEintrag, protokollListe } from "../assistent/api";

const short = (file: string | null) => file?.split("/").pop() ?? "";

/** Liste aller KI-Anfragen, neueste zuerst. Nur lesend: das Protokoll schreibt allein das Backend. */
export function ProtokollDialog({ onClose }: { onClose: () => void }) {
  const ref = useRef<HTMLDialogElement>(null);
  const [list, setList] = useState<ProtokollEintrag[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    ref.current?.showModal();
    protokollListe()
      .then((l) => setList([...l].reverse()))
      .catch((e) => setError(String(e)));
  }, []);

  return (
    <dialog
      ref={ref}
      onClose={onClose}
      onClick={(e) => e.target === ref.current && ref.current?.close()}
      className="m-auto w-[560px] max-w-[92vw] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
    >
      <div className="flex max-h-[70vh] flex-col gap-3 p-4">
        <h2 className="text-base font-semibold">KI-Protokoll</h2>
        {error && <p className="text-sm text-destructive">{error}</p>}
        {list && list.length === 0 && <p className="text-sm text-muted-foreground">Noch keine KI-Anfrage in dieser Arbeit.</p>}
        <ol className="m-0 flex min-h-0 list-none flex-col gap-2 overflow-auto p-0">
          {list?.map((e) => (
            <li key={e.id} className="rounded-lg border border-border bg-card p-3 text-sm text-card-foreground">
              <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
                <span>{new Date(e.zeit).toLocaleString("de-DE", { dateStyle: "short", timeStyle: "short" })}</span>
                <span className={e.ergebnis === "ok" ? "" : "font-medium text-destructive"}>{e.ergebnis === "ok" ? "ok" : e.ergebnis}</span>
              </div>
              <div className="mt-1 font-medium">
                {e.aufgabe}
                {e.kontext.datei && <span className="font-normal text-muted-foreground"> · {short(e.kontext.datei)}{e.kontext.absatz ? `, Absatz ${e.kontext.absatz}` : ""}</span>}
              </div>
              <div className="mt-0.5 line-clamp-2 text-muted-foreground">{e.anfrage}</div>
              <div className="mt-1 text-xs text-muted-foreground">
                {e.modell ? `${e.modell} (${e.anbieter})` : "ohne KI-Aufruf"}
                {e.tokens.ein !== null && ` · ${e.tokens.ein} → ${e.tokens.aus ?? "?"} Tokens`}
              </div>
            </li>
          ))}
        </ol>
        <div className="flex justify-end">
          <Button size="sm" variant="outline" onClick={() => ref.current?.close()}>
            Schließen
          </Button>
        </div>
      </div>
    </dialog>
  );
}
