import { ReactNode, useEffect, useRef, useState, useSyncExternalStore } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { BookMarked, ChartColumn, ChartLine, Code, Heading1, Heading2, Heading3, Image, Link2, List, ListOrdered, Quote, Sigma, Table, Superscript } from "lucide-react";
import { Button } from "../components/ui/button";
import { Source, sourceLabel } from "./api";
import { getInserter, relativePath, subscribeInserter } from "./inserter";
import * as snip from "./snippets";
import { INSERT_EVENT, InsertKind } from "../lib/events";
import { keys, mod } from "../lib/platform";

interface Props {
  sources: Source[];
  /** Absoluter Pfad des Vaults, um gewählte Bilder relativ zur Datei anzugeben. */
  projectRoot: string;
}

type Dialog = InsertKind | null;

/** Leiste über dem Blatt: setzt Typst-Bausteine an die Cursorstelle. Schreibt nur auf Klick des Menschen. */
export function Einfuegeleiste({ sources, projectRoot }: Props) {
  const inserter = useSyncExternalStore(subscribeInserter, getInserter);
  const [dialog, setDialog] = useState<Dialog>(null);
  const ready = inserter !== null;
  const put = (s: snip.Snippet) => getInserter()?.insert(s);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(mod(e) && e.shiftKey)) return;
      const map: Record<string, Dialog> = { t: "tabelle", f: "abbildung", c: "zitat" };
      const d = map[e.key.toLowerCase()];
      if (d && getInserter()) {
        e.preventDefault();
        setDialog(d);
      }
    };
    const onInsert = (e: Event) => setDialog((e as CustomEvent<InsertKind>).detail);
    window.addEventListener("keydown", onKey);
    window.addEventListener(INSERT_EVENT, onInsert);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener(INSERT_EVENT, onInsert);
    };
  }, []);

  const Btn = ({ icon, label, onClick, shortcut }: { icon: ReactNode; label: string; onClick: () => void; shortcut?: string }) => (
    <Button
      variant="ghost"
      size="sm"
      disabled={!ready}
      // Fokus im Editor behalten, damit der Cursor erhalten bleibt.
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      title={ready ? `${label}${shortcut ? ` (${shortcut})` : ""}` : "Zuerst den Cursor in den Text setzen"}
      aria-label={label}
      className="gap-1.5 px-2"
    >
      {icon}
      <span className="hidden xl:inline">{label}</span>
    </Button>
  );
  const Sep = () => <span className="mx-1 h-5 w-px bg-border" aria-hidden />;

  return (
    <div role="toolbar" aria-label="Einfügen" className="flex shrink-0 flex-wrap items-center gap-0.5 border-b border-border bg-background px-2 py-1">
      <Btn icon={<Heading1 />} label="Überschrift 1" onClick={() => put(snip.heading(1))} />
      <Btn icon={<Heading2 />} label="Überschrift 2" onClick={() => put(snip.heading(2))} />
      <Btn icon={<Heading3 />} label="Überschrift 3" onClick={() => put(snip.heading(3))} />
      <Btn icon={<List />} label="Liste" onClick={() => put(snip.bulletList())} />
      <Btn icon={<ListOrdered />} label="Nummerierung" onClick={() => put(snip.numberedList())} />
      <Btn icon={<Quote />} label="Zitat" onClick={() => put(snip.quote())} />
      <Sep />
      <Btn icon={<Image />} label="Abbildung" shortcut={keys("⌘⇧F")} onClick={() => setDialog("abbildung")} />
      <Btn icon={<Table />} label="Tabelle" shortcut={keys("⌘⇧T")} onClick={() => setDialog("tabelle")} />
      <Btn icon={<ChartColumn />} label="Diagramm" onClick={() => setDialog("diagramm")} />
      <Btn icon={<Sigma />} label="Formel" onClick={() => put(snip.equation())} />
      <Btn icon={<Code />} label="Listing" onClick={() => setDialog("listing")} />
      <Btn icon={<Superscript />} label="Fußnote" onClick={() => put(snip.footnote())} />
      <Sep />
      <Btn icon={<BookMarked />} label="Quelle zitieren" shortcut={keys("⌘⇧C")} onClick={() => setDialog("zitat")} />
      <Btn icon={<Link2 />} label="Querverweis" onClick={() => setDialog("verweis")} />
      {dialog && (
        <InsertDialog
          kind={dialog}
          sources={sources}
          onClose={() => setDialog(null)}
          onInsert={(s) => {
            setDialog(null);
            // Nach dem Schließen des Dialogs, damit der Editor den Fokus zurückbekommt.
            window.setTimeout(() => put(s), 0);
          }}
          currentFile={inserter?.file ?? ""}
          projectRoot={projectRoot}
        />
      )}
    </div>
  );
}

const field = "h-8 w-full rounded-md border border-input bg-background px-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring";

function InsertDialog({
  kind,
  sources,
  onClose,
  onInsert,
  currentFile,
  projectRoot,
}: {
  kind: Exclude<Dialog, null>;
  sources: Source[];
  onClose: () => void;
  onInsert: (s: snip.Snippet) => void;
  currentFile: string;
  projectRoot: string;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => ref.current?.showModal(), []);
  const [caption, setCaption] = useState("");
  const [file, setFile] = useState<string | null>(null);
  const [rows, setRows] = useState(3);
  const [cols, setCols] = useState(3);
  const [chartKind, setChartKind] = useState<snip.ChartKind>("balken");
  const [language, setLanguage] = useState("text");
  const [query, setQuery] = useState("");
  const [citekey, setCitekey] = useState("");
  const [page, setPage] = useState("");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);

  const needsCaption = kind === "abbildung" || kind === "tabelle" || kind === "diagramm" || kind === "listing";
  const titles = { abbildung: "Abbildung einfügen", tabelle: "Tabelle einfügen", diagramm: "Diagramm einfügen", listing: "Listing einfügen", zitat: "Quelle zitieren", verweis: "Querverweis einfügen" };

  async function pickImage() {
    const path = await open({ title: "Bild wählen", filters: [{ name: "Bilder", extensions: ["svg", "png", "jpg", "jpeg", "pdf"] }] });
    if (typeof path !== "string") return;
    const root = projectRoot.endsWith("/") ? projectRoot : `${projectRoot}/`;
    if (!path.startsWith(root + "Arbeit/")) {
      setError("Das Bild muss in der Arbeit liegen (z. B. Arbeit/abbildungen/). Lege es dort ab und wähle es erneut.");
      return;
    }
    setError(null);
    setFile(path.slice(root.length));
  }

  function submit() {
    if (needsCaption && !caption.trim()) return setError("Die Vorlage verlangt eine Beschriftung.");
    switch (kind) {
      case "abbildung":
        if (!file) return setError("Bitte ein Bild wählen.");
        return onInsert(snip.figure(relativePath(currentFile, file), caption.trim()));
      case "tabelle":
        return onInsert(snip.table(rows, cols, caption.trim()));
      case "diagramm":
        return onInsert(snip.chart(chartKind, caption.trim()));
      case "listing":
        return onInsert(snip.listing(language.trim() || "text", caption.trim()));
      case "zitat":
        if (!citekey) return setError("Bitte eine Quelle wählen.");
        return onInsert(snip.cite(citekey, page.trim() || undefined));
      case "verweis":
        if (!label.trim()) return setError("Bitte ein Label angeben, z. B. abb:aufbau.");
        return onInsert(snip.crossref(label.trim()));
    }
  }

  const q = query.trim().toLowerCase();
  const hits = sources.filter((s) => !q || s.citekey.toLowerCase().includes(q) || sourceLabel(s).toLowerCase().includes(q) || (s.title ?? "").toLowerCase().includes(q)).slice(0, 8);

  return (
    <dialog
      ref={ref}
      onClose={onClose}
      onClick={(e) => e.target === ref.current && ref.current?.close()}
      className="m-auto w-[380px] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
    >
      <form
        method="dialog"
        className="flex flex-col gap-3 p-4"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <h2 className="text-base font-semibold">{titles[kind]}</h2>
        {kind === "abbildung" && (
          <div className="flex items-center gap-2">
            <Button type="button" variant="outline" size="sm" onClick={pickImage}>
              Bild wählen …
            </Button>
            <span className="truncate text-xs text-muted-foreground">{file ?? "noch keins gewählt"}</span>
          </div>
        )}
        {kind === "tabelle" && (
          <div className="flex gap-3">
            <label className="flex flex-1 flex-col gap-1 text-xs text-muted-foreground">
              Zeilen
              <input className={field} type="number" min={1} max={30} value={rows} onChange={(e) => setRows(+e.target.value)} />
            </label>
            <label className="flex flex-1 flex-col gap-1 text-xs text-muted-foreground">
              Spalten
              <input className={field} type="number" min={1} max={10} value={cols} onChange={(e) => setCols(+e.target.value)} />
            </label>
          </div>
        )}
        {kind === "diagramm" && (
          <div className="flex gap-2" role="group" aria-label="Art">
            {(["balken", "linie"] as const).map((k) => (
              <Button key={k} type="button" size="sm" variant={chartKind === k ? "secondary" : "outline"} aria-pressed={chartKind === k} onClick={() => setChartKind(k)} className="gap-1.5">
                {k === "balken" ? <ChartColumn /> : <ChartLine />}
                {k === "balken" ? "Balken" : "Linie"}
              </Button>
            ))}
          </div>
        )}
        {kind === "listing" && (
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Sprache
            <input className={field} value={language} onChange={(e) => setLanguage(e.target.value)} />
          </label>
        )}
        {needsCaption && (
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Beschriftung
            <input className={field} autoFocus value={caption} onChange={(e) => setCaption(e.target.value)} placeholder="z. B. Aufbau des Systems" />
          </label>
        )}
        {kind === "zitat" && (
          <>
            <input className={field} autoFocus placeholder="Quelle suchen …" value={query} onChange={(e) => setQuery(e.target.value)} />
            <ul className="max-h-40 overflow-auto rounded-md border border-border">
              {hits.length === 0 && <li className="p-2 text-xs text-muted-foreground">keine Quelle gefunden</li>}
              {hits.map((s) => (
                <li key={s.citekey}>
                  <button
                    type="button"
                    onClick={() => setCitekey(s.citekey)}
                    aria-pressed={citekey === s.citekey}
                    className={`flex w-full items-baseline gap-2 px-2 py-1.5 text-left text-sm ${citekey === s.citekey ? "bg-accent text-accent-foreground" : "hover:bg-muted"}`}
                  >
                    {sourceLabel(s)}
                    <span className="font-mono text-xs text-muted-foreground">{s.citekey}</span>
                  </button>
                </li>
              ))}
            </ul>
            <label className="flex flex-col gap-1 text-xs text-muted-foreground">
              Seite (optional)
              <input className={field} value={page} onChange={(e) => setPage(e.target.value)} placeholder="z. B. 14" />
            </label>
          </>
        )}
        {kind === "verweis" && (
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Label
            <input className={field} autoFocus value={label} onChange={(e) => setLabel(e.target.value)} placeholder="abb:aufbau, tab:anforderungen, kap:grundlagen" />
          </label>
        )}
        {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={() => ref.current?.close()}>
            Abbrechen
          </Button>
          <Button type="submit" size="sm">
            Einfügen
          </Button>
        </div>
      </form>
    </dialog>
  );
}
