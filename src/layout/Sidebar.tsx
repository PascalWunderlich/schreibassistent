import { useState } from "react";
import { TreeNode } from "../editor/api";

interface Props {
  tree: TreeNode[];
  selected: string | null;
  onOpen: (node: TreeNode) => void;
  /** Neue Notiz mit diesem Namen anlegen. */
  onNewNote: (name: string) => void;
  /** Neues Kapitel mit diesem Titel anlegen. */
  onNewChapter: (title: string) => void;
}

/** Ordner mit „+“: Was wird angelegt, wie heißt das Eingabefeld? */
const CREATE: Record<string, { title: string; placeholder: string }> = {
  Arbeit: { title: "Neues Kapitel", placeholder: "Titel des Kapitels, Enter" },
  Notizen: { title: "Neue Notiz", placeholder: "Name der Notiz, Enter" },
};

const LABELS: Record<string, string> = { Arbeit: "Arbeit", Notizen: "Meine Notizen", Wissen: "Wissen (KI-gepflegt)" };

/** Seitenleiste: Dateibaum von Arbeit/, Notizen/, Wissen/. */
export function Sidebar({ tree, selected, onOpen, onNewNote, onNewChapter }: Props) {
  /** Ordner, in dem gerade ein Name eingegeben wird. */
  const [naming, setNaming] = useState<string | null>(null);
  const create = (folder: string, name: string) => (folder === "Arbeit" ? onNewChapter(name) : onNewNote(name));
  return (
    <details className="sidebar" aria-label="Dateien">
      <summary>
        <h2 style={{ display: "inline" }}>Dateien und Notizen</h2>
      </summary>
      {tree.map((root) => (
        <section key={root.path}>
          <h2>
            {LABELS[root.name] ?? root.name}
            {!root.writable && <span className="badge">nur lesen</span>}
            {CREATE[root.name] && (
              <button className="add" title={CREATE[root.name].title} onClick={() => setNaming(root.name)}>
                +
              </button>
            )}
          </h2>
          {naming === root.name && (
            <input
              className="new-note"
              autoFocus
              placeholder={CREATE[root.name].placeholder}
              onKeyDown={(e) => {
                if (e.key === "Enter" && e.currentTarget.value.trim()) {
                  create(root.name, e.currentTarget.value.trim());
                  setNaming(null);
                } else if (e.key === "Escape") {
                  setNaming(null);
                }
              }}
              onBlur={() => setNaming(null)}
            />
          )}
          <Nodes nodes={root.children} depth={0} selected={selected} onOpen={onOpen} />
        </section>
      ))}
    </details>
  );
}

function Nodes({
  nodes,
  depth,
  selected,
  onOpen,
}: { nodes: TreeNode[]; depth: number } & Pick<Props, "selected" | "onOpen">) {
  if (nodes.length === 0) return depth === 0 ? <p className="empty">leer</p> : null;
  return (
    <ul>
      {nodes.map((node) => (
        <li key={node.path}>
          {node.is_dir ? (
            <>
              <span className="folder" style={{ paddingLeft: depth * 12 }}>
                {node.name}/
              </span>
              <Nodes nodes={node.children} depth={depth + 1} selected={selected} onOpen={onOpen} />
            </>
          ) : (
            <button
              className={node.path === selected ? "file selected" : "file"}
              style={{ paddingLeft: depth * 12 + 8 }}
              onClick={() => onOpen(node)}
            >
              {node.name.replace(/\.(typ|md)$/, "")}
            </button>
          )}
        </li>
      ))}
    </ul>
  );
}
