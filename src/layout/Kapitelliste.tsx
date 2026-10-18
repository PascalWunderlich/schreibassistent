import { useState } from "react";
import { Plus } from "lucide-react";
import { ChapterInfo } from "../editor/api";
import { keys } from "../lib/platform";

interface Props {
  chapters: ChapterInfo[];
  current: string | null;
  onOpen: (file: string) => void;
  onNew: (title: string) => void;
}

/** Oben links: die Kapitel der Arbeit. ⌘1 bis ⌘9 wechseln das Kapitel. */
export function Kapitelliste({ chapters, current, onOpen, onNew }: Props) {
  const [naming, setNaming] = useState(false);

  return (
    <nav aria-label="Kapitel" className="border-b border-border px-2 pb-2 pt-3">
      <h2 className="mx-2 mb-1 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Kapitel</h2>
      <ol className="m-0 list-none p-0">
        {chapters.map((c, i) => (
          <li key={c.file}>
            <button
              onClick={() => onOpen(c.file)}
              aria-current={c.file === current ? "true" : undefined}
              title={i < 9 ? keys(`⌘${i + 1}`) : undefined}
              className={`flex w-full items-baseline gap-2 rounded-md px-2 py-1.5 text-left text-sm ${c.file === current ? "bg-accent font-medium text-accent-foreground" : "hover:bg-muted"}`}
            >
              <span className="w-4 shrink-0 text-muted-foreground">{i + 1}</span>
              <span className="min-w-0 flex-1 truncate">{c.title ?? c.file.split("/").pop()}</span>
              {c.first_page !== null && <span className="text-xs text-muted-foreground">S. {c.first_page}</span>}
            </button>
          </li>
        ))}
      </ol>
      {chapters.length === 0 && <p className="mx-2 text-sm text-muted-foreground">Noch kein Kapitel.</p>}
      {naming ? (
        <input
          autoFocus
          placeholder="Titel des Kapitels, Enter"
          className="mt-1 h-8 w-full rounded-md border border-input bg-background px-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
          onKeyDown={(e) => {
            if (e.key === "Enter" && e.currentTarget.value.trim()) {
              onNew(e.currentTarget.value.trim());
              setNaming(false);
            } else if (e.key === "Escape") setNaming(false);
          }}
          onBlur={() => setNaming(false)}
        />
      ) : (
        <button onClick={() => setNaming(true)} className="mt-1 flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:bg-muted">
          <Plus className="size-4" />
          Kapitel
        </button>
      )}
    </nav>
  );
}
