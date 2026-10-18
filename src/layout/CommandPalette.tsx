import { Command } from "cmdk";
import { ReactNode, useEffect, useState } from "react";
import { ChapterInfo } from "../editor/api";
import { getInserter } from "../editor/inserter";
import * as snip from "../editor/snippets";
import { InsertKind, openInsert, showTab } from "../lib/events";
import { Theme } from "../lib/theme";
import { keys, mod } from "../lib/platform";

interface Props {
  chapters: ChapterInfo[];
  onOpenChapter: (file: string) => void;
  sourceMode: boolean;
  onSourceMode: (on: boolean) => void;
  onExport: () => void;
  onChoose: () => void;
  onTheme: (t: Theme) => void;
  onShortcuts: () => void;
}

interface Entry {
  id: string;
  label: string;
  hint?: string;
  run: () => void;
}

/** ⌘K: alles, was die Oberfläche kann, mit Tastatur erreichbar. Einfügen setzt nur an der Cursorstelle des Menschen. */
export function CommandPalette(p: Props) {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (mod(e) && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((o) => !o);
      } else if (mod(e) && e.key === "/") {
        e.preventDefault();
        p.onShortcuts();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [p.onShortcuts]);

  const put = (s: snip.Snippet) => getInserter()?.insert(s);
  const dialog = (kind: InsertKind) => () => openInsert(kind);
  const canInsert = getInserter() !== null;

  const groups: { title: string; entries: Entry[] }[] = [
    {
      title: "Kapitel",
      entries: p.chapters.map((c, i) => ({
        id: `kap-${c.file}`,
        label: `${i + 1} ${c.title ?? c.file.split("/").pop()}`,
        hint: i < 9 ? keys(`⌘${i + 1}`) : undefined,
        run: () => p.onOpenChapter(c.file),
      })),
    },
    {
      title: "Einfügen",
      entries: canInsert
        ? [
            { id: "e-h1", label: "Überschrift 1", run: () => put(snip.heading(1)) },
            { id: "e-h2", label: "Überschrift 2", run: () => put(snip.heading(2)) },
            { id: "e-h3", label: "Überschrift 3", run: () => put(snip.heading(3)) },
            { id: "e-liste", label: "Liste", run: () => put(snip.bulletList()) },
            { id: "e-num", label: "Nummerierung", run: () => put(snip.numberedList()) },
            { id: "e-abb", label: "Abbildung", hint: keys("⌘⇧F"), run: dialog("abbildung") },
            { id: "e-tab", label: "Tabelle", hint: keys("⌘⇧T"), run: dialog("tabelle") },
            { id: "e-dia", label: "Diagramm", run: dialog("diagramm") },
            { id: "e-formel", label: "Formel", run: () => put(snip.equation()) },
            { id: "e-listing", label: "Listing", run: dialog("listing") },
            { id: "e-fuss", label: "Fußnote", run: () => put(snip.footnote()) },
            { id: "e-zit", label: "Quelle zitieren", hint: keys("⌘⇧C"), run: dialog("zitat") },
            { id: "e-ver", label: "Querverweis", run: dialog("verweis") },
          ]
        : [],
    },
    {
      title: "Ansicht",
      entries: [
        { id: "a-seite", label: "Ansicht: Seite", run: () => p.onSourceMode(false) },
        { id: "a-quell", label: "Ansicht: Quelltext", run: () => p.onSourceMode(true) },
        { id: "a-pruefen", label: "Prüfen anzeigen", run: () => showTab("pruefen") },
        { id: "a-assistent", label: "Assistent anzeigen", hint: keys("⌘⇧A"), run: () => showTab("assistent") },
        { id: "a-hell", label: "Darstellung: Hell", run: () => p.onTheme("light") },
        { id: "a-dunkel", label: "Darstellung: Dunkel", run: () => p.onTheme("dark") },
        { id: "a-system", label: "Darstellung: System", run: () => p.onTheme("system") },
      ],
    },
    {
      title: "Arbeit",
      entries: [
        { id: "w-pdf", label: "Als PDF exportieren", run: p.onExport },
        { id: "w-andere", label: "Andere Arbeit öffnen …", run: p.onChoose },
        { id: "w-kuerzel", label: "Tastenkürzel anzeigen", hint: keys("⌘/"), run: p.onShortcuts },
      ],
    },
  ];

  const item = "flex cursor-pointer items-center justify-between rounded-md px-2 py-1.5 text-sm aria-selected:bg-accent aria-selected:text-accent-foreground";
  const wrap = (children: ReactNode) => children;

  return (
    <Command.Dialog
      open={open}
      onOpenChange={setOpen}
      label="Befehle"
      overlayClassName="fixed inset-0 z-50 bg-black/30"
      contentClassName="fixed left-1/2 top-[18%] z-50 w-[520px] max-w-[92vw] -translate-x-1/2 overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-lg"
    >
      <Command.Input placeholder="Befehl oder Kapitel suchen …" className="w-full border-b border-border bg-transparent px-3 py-3 text-sm outline-none" />
      <Command.List className="max-h-[50vh] overflow-auto p-1">
        <Command.Empty className="px-3 py-6 text-center text-sm text-muted-foreground">Nichts gefunden.</Command.Empty>
        {groups
          .filter((g) => g.entries.length > 0)
          .map((g) =>
            wrap(
              <Command.Group key={g.title} heading={g.title} className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:uppercase [&_[cmdk-group-heading]]:tracking-wide [&_[cmdk-group-heading]]:text-muted-foreground">
                {g.entries.map((e) => (
                  <Command.Item
                    key={e.id}
                    value={`${g.title} ${e.label}`}
                    className={item}
                    onSelect={() => {
                      setOpen(false);
                      // Dialog zuerst schließen, damit der Editor den Fokus zurückbekommt.
                      window.setTimeout(e.run, 0);
                    }}
                  >
                    {e.label}
                    {e.hint && <span className="text-xs text-muted-foreground">{e.hint}</span>}
                  </Command.Item>
                ))}
              </Command.Group>,
            ),
          )}
      </Command.List>
    </Command.Dialog>
  );
}
