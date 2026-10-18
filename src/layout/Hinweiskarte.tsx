import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import { Hinweis } from "../editor/api";
import { ART_LABEL, artOf, placeOf } from "./hinweise";

const ETIKETT = {
  form: "bg-hint-form text-hint-form-foreground",
  stil: "bg-hint-stil text-hint-stil-foreground",
  quelle: "bg-hint-quelle text-hint-quelle-foreground",
  sprache: "bg-hint-sprache text-hint-sprache-foreground",
} as const;

const SCHWERE = {
  fehler: { Icon: CircleAlert, wort: "Fehler", farbe: "text-destructive" },
  warnung: { Icon: TriangleAlert, wort: "Warnung", farbe: "text-warning" },
  hinweis: { Icon: Info, wort: "Hinweis", farbe: "text-muted-foreground" },
} as const;

/** Eine Karte je Prüfhinweis: Art als Etikett, Ort rechts, ein Satz. Klick springt zur Stelle. */
export function Hinweiskarte({ hinweis, onJump }: { hinweis: Hinweis; onJump: (h: Hinweis) => void }) {
  const art = artOf(hinweis);
  const { Icon, wort, farbe } = SCHWERE[hinweis.schwere];
  return (
    <button
      onClick={() => onJump(hinweis)}
      disabled={!hinweis.datei}
      title={hinweis.fundstelle}
      className="flex w-full flex-col gap-1.5 rounded-lg border border-border bg-card p-3 text-left text-sm text-card-foreground transition-colors enabled:hover:bg-muted disabled:cursor-default"
    >
      <span className="flex items-center justify-between gap-2">
        <span className={`rounded-sm px-1.5 py-0.5 text-xs font-medium ${ETIKETT[art]}`}>{ART_LABEL[art]}</span>
        <span className="truncate text-xs text-muted-foreground">{placeOf(hinweis)}</span>
      </span>
      <span className="flex gap-2">
        <Icon className={`mt-0.5 size-4 shrink-0 ${farbe}`} aria-hidden />
        <span>
          <span className="sr-only">{wort}: </span>
          {hinweis.text}
        </span>
      </span>
    </button>
  );
}
