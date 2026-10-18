import { useEffect, useRef } from "react";
import { keys as keysText } from "../lib/platform";
import { Button } from "../components/ui/button";

const SHORTCUTS: [string, string][] = [
  ["⌘K", "Befehlspalette"],
  ["⌘1 bis ⌘9", "Kapitel wechseln"],
  ["⌘⇧T", "Tabelle einfügen"],
  ["⌘⇧F", "Abbildung einfügen"],
  ["⌘⇧C", "Quelle zitieren"],
  ["⌘⇧A", "Assistent anzeigen"],
  ["⌘↩", "Frage an den Assistenten senden"],
  ["⌘⌥←", "Linke Spalte ein- und ausblenden"],
  ["⌘⌥→", "Rechte Spalte ein- und ausblenden"],
  ["⌘/", "Diese Übersicht"],
];

export function ShortcutsDialog({ onClose }: { onClose: () => void }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => ref.current?.showModal(), []);
  return (
    <dialog
      ref={ref}
      onClose={onClose}
      onClick={(e) => e.target === ref.current && ref.current?.close()}
      className="m-auto w-[380px] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
    >
      <div className="flex flex-col gap-3 p-4">
        <h2 className="text-base font-semibold">Tastenkürzel</h2>
        <dl className="m-0 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm">
          {SHORTCUTS.map(([shortcut, what]) => (
            <div key={shortcut} className="contents">
              <dt className="font-mono text-xs text-muted-foreground">{keysText(shortcut)}</dt>
              <dd className="m-0">{what}</dd>
            </div>
          ))}
        </dl>
        <div className="flex justify-end">
          <Button size="sm" variant="outline" onClick={() => ref.current?.close()}>
            Schließen
          </Button>
        </div>
      </div>
    </dialog>
  );
}
