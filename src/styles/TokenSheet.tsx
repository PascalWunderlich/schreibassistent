import { Button } from "@/components/ui/button";

/** Musteransicht der Design-Tokens (P9, Freigabe). Aufruf im Browser: http://localhost:1420/?tokens */
const HINTS = [
  { art: "form", label: "Form", text: "Inhaltsteil hat 16 Seiten, vorgegeben sind 20–22." },
  { art: "stil", label: "Stil", text: "„man“ vermeiden" },
  { art: "quelle", label: "Quelle", text: "weber2022 wird nicht zitiert" },
  { art: "sprache", label: "Rechtschreibung", text: "Möglicher Tippfehler: „ssnfdbsd“" },
  { art: "ok", label: "OK", text: "Alle Zitate im Verzeichnis" },
] as const;

const chip: Record<string, string> = {
  form: "bg-hint-form text-hint-form-foreground",
  stil: "bg-hint-stil text-hint-stil-foreground",
  quelle: "bg-hint-quelle text-hint-quelle-foreground",
  sprache: "bg-hint-sprache text-hint-sprache-foreground",
  ok: "bg-hint-ok text-hint-ok-foreground",
  ki: "bg-hint-ki text-hint-ki-foreground",
};

function Mock({ dark }: { dark: boolean }) {
  return (
    <div className={`${dark ? "dark" : ""} flex-1`}>
      <div className="bg-background text-foreground text-[length:var(--text-ui)] rounded-lg border border-border overflow-hidden">
        <header className="flex items-center gap-3 px-4 h-11 border-b border-border">
          <span className="font-medium">Testarbeit</span>
          <span className="text-muted-foreground">Kapitel 1 · Einleitung</span>
          <span className="flex-1" />
          <span className="text-muted-foreground">KI: deepseek-chat</span>
          <kbd className="text-xs text-muted-foreground border border-border rounded px-1.5">⌘K</kbd>
          <Button size="sm">PDF</Button>
        </header>
        <div className="grid grid-cols-[180px_1fr_260px] h-[420px]">
          <nav className="border-r border-border p-3 space-y-3">
            {[
              ["Arbeit", ["1 Einleitung", "2 Grundlagen", "3 Umsetzung"]],
              ["Wissen", ["dragoni2016", "fritzsch2019"]],
              ["Notizen", ["gliederung"]],
              ["Quellen · 11", ["Dragoni 2016", "Weber 2022 · ungenutzt"]],
            ].map(([title, items]) => (
              <div key={title as string}>
                <div className="text-xs font-medium text-muted-foreground mb-1">{title}</div>
                {(items as string[]).map((item, i) => (
                  <div
                    key={item}
                    className={`px-2 py-1 rounded-md ${title === "Arbeit" && i === 0 ? "bg-accent text-accent-foreground" : ""} ${item.includes("ungenutzt") ? "text-muted-foreground" : ""}`}
                  >
                    {item}
                  </div>
                ))}
              </div>
            ))}
          </nav>
          <main className="bg-canvas p-6 overflow-hidden">
            <div className="bg-page text-black mx-auto max-w-[340px] p-8 rounded-sm" style={{ boxShadow: "var(--page-shadow)", fontFamily: "'New Computer Modern', Georgia, serif" }}>
              <div className="text-xl font-bold mb-3">1 Einleitung</div>
              <p className="text-sm leading-relaxed mb-2">Interne Kommunikation gilt als Erfolgsfaktor für Veränderung [Mül24, S. 12].</p>
              <div className="text-sm font-mono bg-hint-ki/60 rounded px-2 py-1 border-l-2 border-primary">== Begriffsabgrenzung▍</div>
            </div>
          </main>
          <aside className="border-l border-border p-[var(--space-panel)] space-y-2 overflow-hidden">
            <div className="flex gap-1 text-xs">
              <span className="px-2 py-1 rounded-md bg-card border border-border font-medium">Prüfungen · 5</span>
              <span className="px-2 py-1 rounded-md text-muted-foreground">Assistent</span>
            </div>
            {HINTS.map((h) => (
              <div key={h.art} className="bg-card border border-border rounded-lg p-2 flex gap-2 items-start">
                <span className={`text-xs px-1.5 py-0.5 rounded ${chip[h.art]}`}>{h.label}</span>
                <span className="text-sm">{h.text}</span>
              </div>
            ))}
            <div className="bg-hint-ki text-hint-ki-foreground rounded-lg p-2 text-sm">
              <span className="text-xs font-medium">KI-Hinweis · deepseek-chat</span>
              <div>Satz 3 nennt eine Zahl ohne Beleg.</div>
            </div>
          </aside>
        </div>
        <footer className="flex gap-4 px-4 h-8 items-center border-t border-border text-xs text-muted-foreground">
          <span className="text-warning">16 von 20–22 Seiten</span>
          <span>11 Quellen</span>
          <span>5 Hinweise</span>
          <span className="flex-1" />
          <span>KI-Protokoll: 11 Einträge</span>
        </footer>
      </div>
      <div className="flex gap-2 mt-3 flex-wrap">
        <Button>Wegweiser erzeugen</Button>
        <Button variant="secondary">Abbrechen</Button>
        <Button variant="outline">Einlesen …</Button>
        <Button variant="ghost">Mehr</Button>
        <Button variant="destructive">Löschen</Button>
        <input className="h-9 px-3 rounded-md border border-input bg-card text-foreground text-sm" placeholder="Frage stellen …" />
      </div>
    </div>
  );
}

export function TokenSheet() {
  return (
    <div className="flex gap-6 p-6 bg-[#d9d9d9] min-h-screen">
      <Mock dark={false} />
      <Mock dark />
    </div>
  );
}
