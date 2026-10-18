import { useEffect, useRef, useState } from "react";
import { Button } from "../components/ui/button";
import { KiSettingsDialog } from "../assistent/KiSettingsDialog";
import { RegelprofilDialog } from "../assistent/RegelprofilDialog";
import { Theme } from "../lib/theme";

interface Props {
  theme: Theme;
  onTheme: (t: Theme) => void;
  /** Ohne geöffnete Arbeit gibt es kein Regelprofil. */
  hasProject: boolean;
  onClose: () => void;
}

const THEMES: [Theme, string][] = [
  ["system", "System"],
  ["light", "Hell"],
  ["dark", "Dunkel"],
];

/** Einstellungen (⌘,): Darstellung, KI-Anbieter, Regelprofil. API-Schlüssel liegen im Schlüsselbund, nicht hier. */
export function SettingsDialog({ theme, onTheme, hasProject, onClose }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [sub, setSub] = useState<"ki" | "profil" | null>(null);
  useEffect(() => ref.current?.showModal(), []);

  return (
    <>
      <dialog
        ref={ref}
        onClose={onClose}
        onClick={(e) => e.target === ref.current && ref.current?.close()}
        className="m-auto w-[420px] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
      >
        <div className="flex flex-col gap-4 p-4">
          <h2 className="text-base font-semibold">Einstellungen</h2>
          <section className="flex flex-col gap-2">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Darstellung</h3>
            <div role="radiogroup" aria-label="Darstellung" className="flex gap-2">
              {THEMES.map(([t, label]) => (
                <Button key={t} role="radio" aria-checked={theme === t} size="sm" variant={theme === t ? "secondary" : "outline"} onClick={() => onTheme(t)}>
                  {label}
                </Button>
              ))}
            </div>
          </section>
          <section className="flex flex-col items-start gap-2">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">KI</h3>
            <p className="text-sm text-muted-foreground">Anbieter und Modell je Aufgabe. API-Schlüssel liegen im Schlüsselspeicher des Systems.</p>
            <Button size="sm" variant="outline" onClick={() => setSub("ki")}>
              KI-Einstellungen …
            </Button>
          </section>
          <section className="flex flex-col items-start gap-2">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Regelprofil</h3>
            <p className="text-sm text-muted-foreground">Formvorgaben aus einem Vorgaben-PDF auslesen (KI, mit Bestätigung).</p>
            <Button size="sm" variant="outline" disabled={!hasProject} onClick={() => setSub("profil")}>
              Regelprofil aus PDF …
            </Button>
          </section>
          <div className="flex justify-end">
            <Button size="sm" variant="outline" onClick={() => ref.current?.close()}>
              Schließen
            </Button>
          </div>
        </div>
      </dialog>
      {sub === "ki" && <KiSettingsDialog onClose={() => setSub(null)} />}
      {sub === "profil" && <RegelprofilDialog onClose={() => setSub(null)} />}
    </>
  );
}
