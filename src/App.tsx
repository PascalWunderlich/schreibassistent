import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { PageEditor } from "./editor/PageEditor";
import { NoteEditor } from "./editor/NoteEditor";
import { SourceEditor } from "./editor/SourceEditor";
import {
  CompileReport,
  Definition,
  ParagraphHit,
  ProjectInfo,
  SaveInfo,
  TreeNode,
  errorMessages,
  exportPdf,
  initGit,
  lastProject,
  lspErrorCount,
  newNote,
  openProject,
  projectTree,
} from "./editor/api";
import { Sidebar } from "./layout/Sidebar";
import { RightPanel } from "./layout/RightPanel";
import { SaveState, StatusBar } from "./layout/StatusBar";
import "./App.css";

/** Was in der Mitte steht: die Seiten der Arbeit oder eine Notiz/Wissensseite. */
type Center = { kind: "pages" } | { kind: "note"; path: string; readOnly: boolean };

function App() {
  const [project, setProject] = useState<ProjectInfo | null>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [tree, setTree] = useState<TreeNode[]>([]);
  const [center, setCenter] = useState<Center>({ kind: "pages" });
  const [scrollTo, setScrollTo] = useState<{ file: string; nonce: number } | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [report, setReport] = useState<CompileReport | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("gespeichert");
  const [lastAutosave, setLastAutosave] = useState<number | null>(null);
  const [gitError, setGitError] = useState<string | null>(null);
  const [errorCount, setErrorCount] = useState(0);
  const [notice, setNotice] = useState<string | null>(null);
  /** Quelltextmodus: Datei links, Seiten rechts. */
  const [sourceMode, setSourceMode] = useState(false);
  const [sourceFile, setSourceFile] = useState("Arbeit/main.typ");
  const [sourceJump, setSourceJump] = useState<{ file: string; offset: number; nonce: number } | null>(null);
  const [refresh, setRefresh] = useState(0);

  // Änderungen von außen: Dateibaum aktualisieren (neue oder gelöschte Dateien).
  useEffect(() => {
    if (!project) return;
    const unlisten = listen("datei_geaendert", () => projectTree().then(setTree).catch(() => {}));
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [project]);

  // Fehlerzahl von Tinymist für die Statusleiste.
  useEffect(() => {
    if (!project) return;
    const unlisten = listen("lsp_diagnostics", () => lspErrorCount().then(setErrorCount).catch(() => {}));
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [project]);

  async function load(path: string) {
    setLoading(true);
    try {
      const info = await openProject(path);
      setProject(info);
      setLastAutosave(info.last_autosave);
      setReport(info.report);
      setTree(await projectTree());
      setCenter({ kind: "pages" });
      setErrors([]);
    } catch (e) {
      setErrors(errorMessages(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    lastProject()
      .then((path) => (path ? load(path) : setLoading(false)))
      .catch(() => setLoading(false));
  }, []);

  async function choose() {
    const path = await open({ directory: true, title: "Arbeits-Vault öffnen" });
    if (typeof path === "string") await load(path);
  }

  async function setupGit() {
    try {
      await initGit();
      setProject((p) => (p ? { ...p, git: true } : p));
    } catch (e) {
      setErrors(errorMessages(e));
    }
  }

  async function exportToPdf() {
    if (!project) return;
    const path = await save({
      title: "PDF exportieren",
      defaultPath: `${project.root}/${project.name}.pdf`,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!path) return;
    try {
      await exportPdf(path);
      setNotice(`PDF gespeichert: ${path}`);
    } catch (e) {
      setErrors(errorMessages(e));
    }
  }

  function saved(save: SaveInfo) {
    setSaveState("gespeichert");
    if (save.autosaved) setLastAutosave(save.autosaved);
    setGitError(save.git_error);
  }

  function openNode(node: TreeNode) {
    setSelected(node.path);
    if (node.path.endsWith(".typ")) {
      setCenter({ kind: "pages" });
      setSourceFile(node.path);
      setSourceJump(null);
      setScrollTo({ file: node.path, nonce: Date.now() });
    } else if (node.path.endsWith(".md")) {
      setCenter({ kind: "note", path: node.path, readOnly: !node.writable });
    }
  }

  async function createNote(name: string) {
    try {
      const path = await newNote(name);
      setTree(await projectTree());
      setSelected(path);
      setCenter({ kind: "note", path, readOnly: false });
    } catch (e) {
      setErrors(errorMessages(e));
    }
  }

  /** Quelltextmodus: Klick auf einen Absatz der Seite öffnet ihn im Quelltext. */
  function pick(hit: ParagraphHit) {
    setSourceFile(hit.file);
    setSelected(hit.file);
    setSourceJump({ file: hit.file, offset: hit.start, nonce: Date.now() });
  }

  function definitionInSource(definition: Definition) {
    if (definition.file?.endsWith(".typ")) {
      setSourceFile(definition.file);
      setSourceJump({ file: definition.file, offset: definition.offset, nonce: Date.now() });
    }
  }

  if (!project) {
    return (
      <div className="start">
        <h1>Schreibassistent</h1>
        {loading ? (
          <p>Öffne Arbeit …</p>
        ) : (
          <>
            <p>Öffne den Ordner deiner Arbeit (Arbeits-Vault mit Arbeit/main.typ).</p>
            <button onClick={choose}>Arbeit öffnen …</button>
          </>
        )}
        {errors.map((e, i) => (
          <p key={i} className="errors">
            {e}
          </p>
        ))}
      </div>
    );
  }

  return (
    <div className="app">
      <header className="topbar">
        <h1>Schreibassistent</h1>
        <span className="project-name">{project.name}</span>
        <span className="spacer" />
        <div className="segmented" role="group" aria-label="Ansicht">
          <button aria-pressed={!sourceMode} onClick={() => setSourceMode(false)}>
            Seite
          </button>
          <button aria-pressed={sourceMode} onClick={() => setSourceMode(true)}>
            Quelltext
          </button>
        </div>
        <button onClick={exportToPdf}>PDF</button>
        <button onClick={choose}>Andere Arbeit …</button>
      </header>
      {notice && (
        <div className="hint" onClick={() => setNotice(null)}>
          {notice}
        </div>
      )}
      {errors.length > 0 && (
        <div className="errors" onClick={() => setErrors([])}>
          {errors.join(" · ")}
        </div>
      )}
      {project.lsp_error && <div className="hint">{project.lsp_error} – ohne Vervollständigung und Fehleranzeige.</div>}
      {!project.git && (
        <div className="hint">
          Diese Arbeit hat noch keine automatische Sicherung. <button onClick={setupGit}>Git-Sicherung einrichten</button>
        </div>
      )}
      <div className="workspace">
        <Sidebar tree={tree} selected={selected} onOpen={openNode} onNewNote={createNote} />
        <main className="center">
          {center.kind === "pages" && sourceMode ? (
            <div className="source-split">
              <SourceEditor
                key={sourceFile}
                path={sourceFile}
                jump={sourceJump}
                onSaved={saved}
                onSaving={() => setSaveState("speichert …")}
                onWritten={() => setRefresh((n) => n + 1)}
                onDefinition={definitionInSource}
              />
              <PageEditor
                key={`${project.root}:quelltext`}
                scrollTo={scrollTo}
                onReport={setReport}
                onSaved={saved}
                onSaving={() => setSaveState("speichert …")}
                onPick={pick}
                refresh={refresh}
                scale={0.9}
              />
            </div>
          ) : center.kind === "pages" ? (
            <PageEditor
              key={project.root}
              scrollTo={scrollTo}
              onReport={setReport}
              onSaved={saved}
              onSaving={() => setSaveState("speichert …")}
            />
          ) : (
            <NoteEditor
              key={center.path}
              path={center.path}
              readOnly={center.readOnly}
              onSaved={saved}
              onSaving={() => setSaveState("speichert …")}
            />
          )}
        </main>
        <RightPanel />
      </div>
      <StatusBar
        contentPages={report?.content_pages ?? null}
        saveState={saveState}
        lastAutosave={lastAutosave}
        git={project.git}
        gitError={gitError}
        errorCount={errorCount}
      />
    </div>
  );
}

export default App;
