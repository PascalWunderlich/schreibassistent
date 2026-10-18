import { useState } from "react";
import { ChevronRight } from "lucide-react";
import { RegelprofilDialog } from "../assistent/RegelprofilDialog";
import { Hinweis, Pruefbericht } from "../editor/api";
import { ART_LABEL, Art, artOf, isMetadata, sortHints } from "./hinweise";
import { Hinweiskarte } from "./Hinweiskarte";

interface Props {
  bericht: Pruefbericht | null;
  rechtschreibung: Hinweis[] | "laeuft" | null;
  onJump: (hinweis: Hinweis) => void;
}

type Filter = "alle" | Art;

/** Tab „Prüfen“: feste Regeln, Stil und Rechtschreibung. Wichtiges zuerst, Metadaten der `.bib` eingeklappt. */
export function PruefenPanel({ bericht, rechtschreibung, onJump }: Props) {
  const [filter, setFilter] = useState<Filter>("alle");
  const [profilDialog, setProfilDialog] = useState(false);
  const [metaOpen, setMetaOpen] = useState(false);
  const all = [...(bericht?.hinweise ?? []), ...(Array.isArray(rechtschreibung) ? rechtschreibung : [])];
  const count = (f: Filter) => all.filter((h) => f === "alle" || artOf(h) === f).length;
  const shown = all.filter((h) => filter === "alle" || artOf(h) === filter);
  const main = sortHints(shown.filter((h) => !isMetadata(h)));
  const meta = sortHints(shown.filter(isMetadata));

  const chip = (f: Filter, label: string) => (
    <button
      key={f}
      aria-pressed={filter === f}
      onClick={() => setFilter(f)}
      className={`rounded-md px-2 py-1 text-xs font-medium ${filter === f ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-muted"}`}
    >
      {label} {count(f)}
    </button>
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div role="group" aria-label="Filter" className="flex flex-wrap gap-1 border-b border-border px-3 py-2">
        {chip("alle", "Alle")}
        {(Object.keys(ART_LABEL) as Art[]).map((a) => chip(a, ART_LABEL[a]))}
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-auto p-3">
        {!bericht && <p className="text-sm text-muted-foreground">Prüft …</p>}
        {bericht && !bericht.profil && bericht.fehler.length === 0 && (
          <p className="text-sm text-muted-foreground">Kein Regelprofil (System/regelprofil.yaml), nur Rechtschreibung.</p>
        )}
        {bericht?.profil && !bericht.profil.bestaetigt && (
          <p className="text-sm text-warning">Regelprofil „{bericht.profil.profil}“ ist noch nicht bestätigt.</p>
        )}
        {bericht?.fehler.map((f, i) => (
          <p key={i} className="text-sm text-destructive">
            {f}
          </p>
        ))}
        {rechtschreibung === "laeuft" && <p className="text-sm text-muted-foreground">Rechtschreibung startet …</p>}
        {bericht && all.length === 0 && (
          <p className="rounded-md bg-hint-ok px-2 py-1.5 text-sm text-hint-ok-foreground">OK: keine Hinweise.</p>
        )}
        {main.map((h, i) => (
          <Hinweiskarte key={i} hinweis={h} onJump={onJump} />
        ))}
        {meta.length > 0 && (
          <div>
            <button onClick={() => setMetaOpen((o) => !o)} aria-expanded={metaOpen} className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
              <ChevronRight className={`size-4 transition-transform ${metaOpen ? "rotate-90" : ""}`} />
              Quellenangaben ({meta.length})
            </button>
            {metaOpen && (
              <div className="mt-2 flex flex-col gap-2">
                {meta.map((h, i) => (
                  <Hinweiskarte key={i} hinweis={h} onJump={onJump} />
                ))}
              </div>
            )}
          </div>
        )}
      </div>
      <div className="border-t border-border px-3 py-2">
        <button onClick={() => setProfilDialog(true)} className="text-xs text-muted-foreground underline-offset-2 hover:underline" title="Regelprofil aus einem Vorgaben-PDF (KI, mit Bestätigung)">
          Regelprofil aus PDF …
        </button>
      </div>
      {profilDialog && <RegelprofilDialog onClose={() => setProfilDialog(false)} />}
    </div>
  );
}
