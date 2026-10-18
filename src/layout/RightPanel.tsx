import { ReactNode, useEffect, useState } from "react";
import { Hinweis, Pruefbericht } from "../editor/api";
import { PruefenPanel } from "./PruefenPanel";

interface Props {
  bericht: Pruefbericht | null;
  rechtschreibung: Hinweis[] | "laeuft" | null;
  onJump: (hinweis: Hinweis) => void;
  /** Tab „Assistent“ (P6). */
  assistant: ReactNode;
}

type Tab = "pruefen" | "assistent";

/** Rechts: zwei Tabs, „Prüfen“ (feste Regeln, ohne KI) und „Assistent“ (auf Nachfrage). ⌘⇧A wechselt zum Assistenten. */
export function RightPanel({ bericht, rechtschreibung, onJump, assistant }: Props) {
  const [tab, setTab] = useState<Tab>("pruefen");
  const count = (bericht?.hinweise.length ?? 0) + (Array.isArray(rechtschreibung) ? rechtschreibung.length : 0);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.shiftKey && e.key.toLowerCase() === "a") {
        e.preventDefault();
        setTab("assistent");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const tabClass = (t: Tab) =>
    `px-3 py-2 text-[13px] font-medium border-b-2 -mb-px ${tab === t ? "border-primary text-foreground" : "border-transparent text-muted-foreground hover:text-foreground"}`;

  return (
    <aside className="flex h-full min-h-0 flex-col border-l border-border bg-sidebar">
      <div role="tablist" className="flex shrink-0 border-b border-border px-2">
        <button role="tab" aria-selected={tab === "pruefen"} className={tabClass("pruefen")} onClick={() => setTab("pruefen")}>
          Prüfen{count > 0 && <span className="ml-1.5 rounded-sm bg-muted px-1.5 text-xs text-muted-foreground">{count}</span>}
        </button>
        <button role="tab" aria-selected={tab === "assistent"} className={tabClass("assistent")} onClick={() => setTab("assistent")}>
          Assistent
        </button>
      </div>
      {/* Beide bleiben eingehängt, damit der Assistent beim Wechsel Eingabe und Verlauf behält. */}
      <div role="tabpanel" hidden={tab !== "pruefen"} className="flex min-h-0 flex-1 flex-col data-[hidden]:hidden" style={{ display: tab === "pruefen" ? undefined : "none" }}>
        <PruefenPanel bericht={bericht} rechtschreibung={rechtschreibung} onJump={onJump} />
      </div>
      <div role="tabpanel" hidden={tab !== "assistent"} className="flex min-h-0 flex-1 flex-col overflow-auto" style={{ display: tab === "assistent" ? undefined : "none" }}>
        {assistant}
      </div>
    </aside>
  );
}
