import { Download, FolderOpen, Moon, Sun } from "lucide-react";
import { Button } from "../components/ui/button";
import { Theme } from "../lib/theme";

interface Props {
  projectName: string;
  saveLabel: string;
  sourceMode: boolean;
  onSourceMode: (on: boolean) => void;
  onExport: () => void;
  onChoose: () => void;
  theme: Theme;
  onTheme: (t: Theme) => void;
}

/** Schmale Kopfzeile: Arbeit, Ansicht, PDF. Der Titel der App steht nur im Fenster. */
export function Topbar({ projectName, saveLabel, sourceMode, onSourceMode, onExport, onChoose, theme, onTheme }: Props) {
  const next: Theme = theme === "dark" ? "light" : theme === "light" ? "system" : "dark";
  const seg = (active: boolean) =>
    `px-3 h-7 text-[13px] font-medium rounded-md transition-colors ${active ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-muted"}`;
  return (
    <header className="flex h-11 shrink-0 items-center gap-3 border-b border-border bg-background px-3">
      <Button variant="ghost" size="sm" onClick={onChoose} title="Andere Arbeit öffnen" className="gap-1.5 font-semibold">
        <FolderOpen />
        {projectName}
      </Button>
      <span className="text-xs text-muted-foreground">{saveLabel}</span>
      <span className="flex-1" />
      <div role="group" aria-label="Ansicht" className="flex gap-0.5 rounded-lg border border-border p-0.5">
        <button aria-pressed={!sourceMode} className={seg(!sourceMode)} onClick={() => onSourceMode(false)}>
          Seite
        </button>
        <button aria-pressed={sourceMode} className={seg(sourceMode)} onClick={() => onSourceMode(true)}>
          Quelltext
        </button>
      </div>
      <Button variant="outline" size="sm" onClick={onExport} className="gap-1.5">
        <Download />
        PDF
      </Button>
      <Button
        variant="ghost"
        size="icon-sm"
        onClick={() => onTheme(next)}
        title={`Darstellung: ${theme === "system" ? "System" : theme === "dark" ? "Dunkel" : "Hell"} (klicken zum Wechseln)`}
        aria-label="Darstellung wechseln"
      >
        {theme === "dark" ? <Moon /> : <Sun />}
      </Button>
    </header>
  );
}
