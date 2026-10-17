import { useState } from "react";
import { ChapterView, Source, sourceLabel } from "../editor/api";

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

  return (
    <div className="chapter-panel">
      <section>
        <h2>Dieses Kapitel{chapter?.title ? `: ${chapter.title}` : ""}</h2>
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
                  {open === key && byKey.get(key) && <SourceDetails source={byKey.get(key)!} />}
                </li>
              ))}
            </ul>
            <Related title="Notizen" direct={chapter.notizen_direkt} via={chapter.notizen_ueber} onOpen={onOpenFile} />
            <Related title="Wissen" direct={chapter.wissen_direkt} via={chapter.wissen_ueber} onOpen={onOpenFile} />
          </>
        )}
      </section>
      <section>
        <h2>Alle Quellen</h2>
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
              {open === `alle:${source.citekey}` && <SourceDetails source={source} />}
            </li>
          ))}
        </ul>
      </section>
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

function SourceDetails({ source }: { source: Source }) {
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
    </div>
  );
}
