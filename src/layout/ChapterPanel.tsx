import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ChapterView,
  Source,
  ZoteroSource,
  pageFromSupplement,
  sourceLabel,
  zoteroPdfUrl,
  zoteroSource,
} from "../editor/api";
import { Wegweiser } from "../wissen/Wegweiser";
import { ingestKonzepte } from "../wissen/api";

interface Props {
  chapter: ChapterView | null;
  sources: Source[];
  sourcesError: string | null;
  /** Notiz oder Wissensseite öffnen. */
  onOpenFile: (path: string) => void;
}

const fileName = (path: string) => path.split("/").pop()!.replace(/\.md$/, "");

/** Seitenleiste oben: was zum aktuellen Kapitel gehört, darunter alle Quellen. */
export function ChapterPanel({ chapter, sources, sourcesError, onOpenFile }: Props) {
  const [open, setOpen] = useState<string | null>(null);
  const byKey = new Map(sources.map((s) => [s.citekey, s]));
  const toggle = (key: string) => setOpen((current) => (current === key ? null : key));
  const [concepts, setConcepts] = useState<string | null>(null);

  async function createConcepts() {
    if (!window.confirm("Konzeptseiten aus allen Wegweisern erzeugen? Das ist eine KI-Anfrage (Anbieter wie beim Einlesen).")) return;
    setConcepts("erzeugt Konzepte …");
    try {
      const r = await ingestKonzepte();
      setConcepts(`${r.seiten.length} Konzeptseiten (${r.modell})${r.warnungen ? `, ${r.warnungen} Verweise nicht bestätigt` : ""}`);
    } catch (e) {
      setConcepts(String(e));
    }
  }

  return (
    <div className="chapter-panel">
      <section>
        <h2>Dieses Kapitel</h2>
        {!chapter ? (
          <p className="empty">Kein Kapitel ausgewählt.</p>
        ) : (
          <>
            <h3>Quellen</h3>
            {Object.keys(chapter.quellen).length === 0 && <p className="empty">keine zitiert</p>}
            <ul>
              {Object.entries(chapter.quellen).map(([key, pages]) => (
                <li key={key}>
                  <button className="item" onClick={() => toggle(key)}>
                    {byKey.get(key) ? sourceLabel(byKey.get(key)!) : key}
                    {pages.length > 0 && <span className="meta"> · {pages.join(", ")}</span>}
                  </button>
                  {open === key && byKey.get(key) && <SourceDetails source={byKey.get(key)!} pages={pages} onOpenFile={onOpenFile} />}
                </li>
              ))}
            </ul>
            <Related title="Notizen" direct={chapter.notizen_direkt} via={chapter.notizen_ueber} onOpen={onOpenFile} />
            <Related title="Wissen" direct={chapter.wissen_direkt} via={chapter.wissen_ueber} onOpen={onOpenFile} />
          </>
        )}
      </section>
      <details className="all-sources">
        <summary>
          <h2 style={{ display: "inline" }}>Alle Quellen ({sources.length})</h2>
        </summary>
        <button className="h2-action" onClick={createConcepts} title="Konzeptseiten aus allen Wegweisern (KI)">
          Konzepte erzeugen
        </button>
        {concepts && <p className="meta concepts-line">{concepts}</p>}
        {sourcesError && <p className="warn">{sourcesError}</p>}
        {sources.length === 0 && !sourcesError && <p className="empty">System/quellen.bib ist leer</p>}
        <ul>
          {sources.map((source) => (
            <li key={source.citekey}>
              <button
                className={source.chapters.length === 0 ? "item unused" : "item"}
                title={source.chapters.length === 0 ? "ungenutzt" : `zitiert in ${source.chapters.length} Kapitel(n)`}
                onClick={() => toggle(`alle:${source.citekey}`)}
              >
                {sourceLabel(source)}
                {source.chapters.length === 0 && <span className="meta"> · ungenutzt</span>}
              </button>
              {open === `alle:${source.citekey}` && <SourceDetails source={source} pages={[]} onOpenFile={onOpenFile} />}
            </li>
          ))}
        </ul>
      </details>
    </div>
  );
}

function Related({
  title,
  direct,
  via,
  onOpen,
}: {
  title: string;
  direct: string[];
  via: Record<string, string[]>;
  onOpen: (path: string) => void;
}) {
  const entries = Object.entries(via);
  return (
    <>
      <h3>{title}</h3>
      {direct.length === 0 && entries.length === 0 && <p className="empty">keine</p>}
      <ul>
        {direct.map((path) => (
          <li key={path}>
            <button className="item" onClick={() => onOpen(path)}>
              {fileName(path)}
            </button>
          </li>
        ))}
        {entries.map(([path, keys]) => (
          <li key={path}>
            <button className="item" onClick={() => onOpen(path)} title="gehört über eine zitierte Quelle dazu">
              {fileName(path)}
              <span className="meta"> · über {keys.join(", ")}</span>
            </button>
          </li>
        ))}
      </ul>
    </>
  );
}

/** Details aus der `.bib`; dazu PDFs und Markierungen aus Zotero, falls es läuft. */
function SourceDetails({ source, pages, onOpenFile }: { source: Source; pages: string[]; onOpenFile: (path: string) => void }) {
  const [zotero, setZotero] = useState<ZoteroSource | null | "laden">("laden");
  useEffect(() => {
    zoteroSource(source.citekey)
      .then(setZotero)
      .catch(() => setZotero(null));
  }, [source.citekey]);
  const open = (key: string, page: number | null) => openUrl(zoteroPdfUrl(key, page)).catch(() => {});
  const firstPage = pageFromSupplement(pages[0]);

  return (
    <div className="source-details">
      <div className="citekey">@{source.citekey}</div>
      {source.authors.length > 0 && <div>{source.authors.join("; ")}</div>}
      {source.title && <div className="title">{source.title}</div>}
      <div className="meta">
        {[source.container, source.publisher, source.year].filter(Boolean).join(", ")}
        {source.kind && ` · ${source.kind}`}
      </div>
      {source.doi && <div className="meta">DOI {source.doi}</div>}
      {source.url && <div className="meta url">{source.url}</div>}
      <div className="meta">
        {source.chapters.length === 0 ? "nirgends zitiert" : `zitiert in: ${source.chapters.map((c) => c.split("/").pop()!.replace(/\.typ$/, "")).join(", ")}`}
      </div>
      <Wegweiser citekey={source.citekey} onOpenFile={onOpenFile} />
      <div className="zotero">
        {zotero === "laden" && <span className="meta">Zotero …</span>}
        {zotero === null && <span className="meta">Zotero nicht erreichbar oder Quelle dort nicht vorhanden</span>}
        {zotero !== "laden" && zotero !== null && zotero.pdfs.length === 0 && <span className="meta">kein PDF in Zotero</span>}
        {zotero !== "laden" &&
          zotero?.pdfs.map((pdf) => (
            <div key={pdf.key}>
              <button className="pdf" onClick={() => open(pdf.key, firstPage)}>
                PDF öffnen{firstPage ? ` (S. ${firstPage})` : ""}
              </button>
              {pdf.annotations.length > 0 && (
                <ul className="annotations">
                  {pdf.annotations.map((a, i) => (
                    <li key={i}>
                      <button className="item" onClick={() => open(pdf.key, a.page)} title="im PDF an dieser Stelle öffnen">
                        <span className="meta">S. {a.page_label ?? a.page ?? "?"}</span> {a.text ?? a.comment ?? a.kind}
                        {a.text && a.comment && <span className="meta"> – {a.comment}</span>}
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          ))}
      </div>
    </div>
  );
}
