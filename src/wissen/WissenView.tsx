import { ReactNode, useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { readFile } from "../editor/api";

interface Props {
  path: string;
  /** `[[Name]]` öffnen (Quellen-, Konzeptseite oder Notiz). */
  onOpenLink: (name: string) => void;
  /** Wird erhöht, wenn sich Dateien geändert haben. */
  version: number;
}

/** Inline: [Text](zotero://…), [[Link]], *kursiv*, **fett**. */
function inline(text: string, onOpenLink: (name: string) => void): ReactNode[] {
  const out: ReactNode[] = [];
  const pattern = /\[([^\]]+)\]\(([^)]+)\)|\[\[([^\]|]+)(?:\|([^\]]+))?\]\]|\*\*([^*]+)\*\*|\*([^*]+)\*/g;
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = pattern.exec(text))) {
    if (m.index > last) out.push(text.slice(last, m.index));
    const key = out.length;
    if (m[1] !== undefined) {
      const url = m[2];
      out.push(
        <a
          key={key}
          href={url}
          title={url.startsWith("zotero://") ? "PDF in Zotero an dieser Seite öffnen" : url}
          onClick={(e) => {
            e.preventDefault();
            openUrl(url).catch(() => {});
          }}
        >
          {m[1]}
        </a>,
      );
    } else if (m[3] !== undefined) {
      const name = m[3];
      out.push(
        <a
          key={key}
          href="#"
          className="wikilink"
          onClick={(e) => {
            e.preventDefault();
            onOpenLink(name);
          }}
        >
          {m[4] ?? name}
        </a>,
      );
    } else if (m[5] !== undefined) {
      out.push(<b key={key}>{m[5]}</b>);
    } else {
      out.push(<i key={key}>{m[6]}</i>);
    }
    last = pattern.lastIndex;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

/** Wissensseiten (KI-gepflegt, nur lesbar) als Lesefassung mit klickbaren Stellen. */
export function WissenView({ path, onOpenLink, version }: Props) {
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    readFile(path)
      .then(setText)
      .catch((e) => setError(String(e)));
  }, [path, version]);

  if (error) return <div className="wissen-view warn">{error}</div>;
  if (text === null) return <div className="wissen-view meta">lädt …</div>;

  let body = text;
  const meta: Record<string, string> = {};
  if (body.startsWith("---\n")) {
    const end = body.indexOf("\n---", 4);
    if (end > 0) {
      for (const line of body.slice(4, end).split("\n")) {
        const i = line.indexOf(":");
        if (i > 0) meta[line.slice(0, i).trim()] = line.slice(i + 1).trim();
      }
      body = body.slice(end + 4);
    }
  }

  const blocks: ReactNode[] = [];
  let list: ReactNode[] = [];
  const flush = () => {
    if (list.length) blocks.push(<ul key={blocks.length}>{list}</ul>);
    list = [];
  };
  for (const raw of body.split("\n")) {
    const line = raw.trimEnd();
    if (/^\s*[-*] /.test(line)) {
      const content = line.replace(/^\s*[-*] /, "");
      list.push(
        <li key={list.length} className={content.startsWith("⚠") ? "unconfirmed" : undefined}>
          {inline(content, onOpenLink)}
        </li>,
      );
      continue;
    }
    flush();
    if (!line.trim()) continue;
    if (line.startsWith("## ")) blocks.push(<h2 key={blocks.length}>{inline(line.slice(3), onOpenLink)}</h2>);
    else if (line.startsWith("# ")) blocks.push(<h1 key={blocks.length}>{inline(line.slice(2), onOpenLink)}</h1>);
    else if (line.startsWith(">")) blocks.push(<blockquote key={blocks.length}>{inline(line.replace(/^>\s?/, ""), onOpenLink)}</blockquote>);
    else blocks.push(<p key={blocks.length}>{inline(line, onOpenLink)}</p>);
  }
  flush();

  return (
    <article className="wissen-view">
      <div className="meta wissen-meta">
        {path} · nur lesbar
        {meta.author === "ai" && " · von der KI erzeugt"}
        {meta.modell && ` · ${meta.modell}`}
        {meta.erzeugt && ` · ${meta.erzeugt}`}
      </div>
      {blocks}
    </article>
  );
}
