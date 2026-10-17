import { TreeNode } from "../editor/api";

interface Props {
  tree: TreeNode[];
  selected: string | null;
  onOpen: (node: TreeNode) => void;
}

const LABELS: Record<string, string> = { Arbeit: "Arbeit", Notizen: "Meine Notizen", Wissen: "Wissen (KI-gepflegt)" };

/** Seitenleiste: Dateibaum von Arbeit/, Notizen/, Wissen/. */
export function Sidebar({ tree, selected, onOpen }: Props) {
  return (
    <nav className="sidebar" aria-label="Dateien">
      {tree.map((root) => (
        <section key={root.path}>
          <h2>
            {LABELS[root.name] ?? root.name}
            {!root.writable && <span className="badge">nur lesen</span>}
          </h2>
          <Nodes nodes={root.children} depth={0} selected={selected} onOpen={onOpen} />
        </section>
      ))}
    </nav>
  );
}

function Nodes({ nodes, depth, selected, onOpen }: { nodes: TreeNode[]; depth: number } & Omit<Props, "tree">) {
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
