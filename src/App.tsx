import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { PageEditor } from "./editor/PageEditor";
import { NoteEditor } from "./editor/NoteEditor";
import { WissenView } from "./wissen/WissenView";
import { SourceEditor } from "./editor/SourceEditor";
import {
  ChapterInfo,
  ChapterView,
  Hinweis,
  Pruefbericht,
  CompileReport,
  Definition,
  ParagraphHit,
  ProjectInfo,
  SaveInfo,
  TreeNode,
  SourcesOverview,
  errorMessages,
  exportDir,
  indexChapter,
  indexChapters,
  indexSources,
  exportPdf,
  initGit,
  lastProject,
  lspErrorCount,
  newChapter,
  newNote,
  openProject,
  projectTree,
  pruefungRechtschreibung,
  pruefungRun,
} from "./editor/api";
import { Sidebar } from "./layout/Sidebar";
import { Kapitelliste } from "./layout/Kapitelliste";
import { ChapterPanel } from "./layout/ChapterPanel";
import { RightPanel } from "./layout/RightPanel";
import { Assistant } from "./assistent/Assistant";
import { SaveState, StatusBar } from "./layout/StatusBar";
import { Einfuegeleiste } from "./editor/Einfuegeleiste";
import { Topbar } from "./layout/Topbar";
import { Workspace } from "./layout/Workspace";
import { useTheme } from "./lib/theme";
import "./App.css";

/** Was in der Mitte steht: die Seiten der Arbeit oder eine Notiz/Wissensseite. */
type Center = { kind: "pages" } | { kind: "note"; path: string; readOnly: boolean };

function App() {
  const [theme, setTheme] = useTheme();
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
  const [reveal, setReveal] = useState<{ file: string; offset: number; nonce: number } | null>(null);
  // Index: Kapitel, aktuelles Kapitel, Quellen.
  const [chapters, setChapters] = useState<ChapterInfo[]>([]);
  const [visiblePage, setVisiblePage] = useState<number | null>(null);
  const [editingFile, setEditingFile] = useState<string | null>(null);
  /** Zuletzt bearbeiteter Absatz; bleibt nach dem Verlassen des Felds (Kontext für den Assistenten). */
  const [lastParagraph, setLastParagraph] = useState<{ file: string; offset: number } | null>(null);
  const [protokollVersion, setProtokollVersion] = useState(0);
  const [chapterView, setChapterView] = useState<ChapterView | null>(null);
  const [sources, setSources] = useState<SourcesOverview>({ sources: [], error: null });
  const [indexVersion, setIndexVersion] = useState(0);
  const refreshIndex = () => setIndexVersion((n) => n + 1);
  // Prüfungen: feste Regeln und Vale nach jedem Setzen, Rechtschreibung (LTeX+) nach Pausen.
  const [bericht, setBericht] = useState<Pruefbericht | null>(null);
  const [rechtschreibung, setRechtschreibung] = useState<Hinweis[] | "laeuft" | null>(null);

  // Änderungen von außen: Dateibaum aktualisieren (neue oder gelöschte Dateien).
  useEffect(() => {
    if (!project) return;
    const unlisten = listen("datei_geaendert", () => projectTree().then(setTree).catch(() => {}));
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [project]);

  useEffect(() => {
    if (!project) return;
    const timer = window.setTimeout(() => pruefungRun().then(setBericht).catch((e) => setErrors(errorMessages(e))), 600);
    return () => window.clearTimeout(timer);
  }, [project, report, indexVersion]);

  useEffect(() => {
    if (!project) return;
    const timer = window.setTimeout(() => {
      setRechtschreibung((r) => (Array.isArray(r) ? r : "laeuft"));
      pruefungRechtschreibung()
        .then(setRechtschreibung)
        .catch((e) => {
          setRechtschreibung(null);
          setErrors(errorMessages(e));
        });
    }, 2500);
    return () => window.clearTimeout(timer);
  }, [project, indexVersion]);

  const allHints = [...(bericht?.hinweise ?? []), ...(Array.isArray(rechtschreibung) ? rechtschreibung : [])];
  // Markierungen am Absatz: je Stelle eine, mit der höchsten Schwere.
  const marks = (() => {
    const rank: Record<string, number> = { fehler: 0, warnung: 1, hinweis: 2 };
    const byPlace = new Map<string, { page: number; rect: import("./editor/api").Rect; schwere: string; texts: string[] }>();
    for (const h of allHints) {
      if (!h.stelle) continue;
      const [page, rect] = h.stelle;
      const key = `${page}:${rect.x}:${rect.y}`;
      const mark = byPlace.get(key) ?? { page, rect, schwere: h.schwere, texts: [] };
      if (rank[h.schwere] < rank[mark.schwere]) mark.schwere = h.schwere;
      mark.texts.push(h.text);
      byPlace.set(key, mark);
    }
    return [...byPlace.values()];
  })();

  /** Klick auf einen Hinweis: zur Stelle springen. */
  function jumpToHint(h: Hinweis) {
    if (!h.datei) return;
    if (!h.datei.endsWith(".typ")) {
      setSelected(h.datei);
      setCenter({ kind: "note", path: h.datei, readOnly: true });
      return;
    }
    setCenter({ kind: "pages" });
    setSelected(h.datei);
    const target = { file: h.datei, offset: h.offset ?? 0, nonce: Date.now() };
    if (sourceMode) {
      setSourceFile(h.datei);
      setSourceJump(target);
    }
    setReveal(target);
  }

  // Index neu laden: nach dem Öffnen, nach dem Speichern, bei Änderungen von außen.
  useEffect(() => {
    if (!project) return;
    const unlisten = listen("index_aktualisiert", refreshIndex);
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [project]);

  useEffect(() => {
    if (!project) return;
    indexChapters().then(setChapters).catch(() => {});
    indexSources().then(setSources).catch(() => {});
  }, [project, indexVersion, report]);

  /** Aktuelles Kapitel: Quelltextmodus → offene Datei; sonst bearbeiteter Absatz oder sichtbare Seite. */
  const isChapter = (file: string | null) => !!file && chapters.some((c) => c.file === file);
  const currentChapter = (() => {
    if (sourceMode && center.kind === "pages") return isChapter(sourceFile) ? sourceFile : null;
    if (isChapter(editingFile)) return editingFile;
    if (visiblePage === null) return chapters[0]?.file ?? null;
    const before = chapters.filter((c) => c.first_page !== null && c.first_page <= visiblePage);
    return before.length > 0 ? before[before.length - 1].file : null;
  })();

  useEffect(() => {
    if (!project || !currentChapter) {
      setChapterView(null);
      return;
    }
    indexChapter(currentChapter).then(setChapterView).catch(() => setChapterView(null));
  }, [project, currentChapter, indexVersion]);

  /** `[[Name]]` aus einer Wissensseite: Datei gleichen Namens, zuerst in Wissen/, dann Notizen/. */
  function openLink(name: string) {
    const all: string[] = [];
    const walk = (nodes: TreeNode[]) => nodes.forEach((n) => (n.is_dir ? walk(n.children) : all.push(n.path)));
    walk(tree);
    const match = (p: string) => p.split("/").pop() === `${name}.md`;
    const found = all.find((p) => p.startsWith("Wissen/") && match(p)) ?? all.find(match);
    if (found) openFromIndex(found);
    else setNotice(`„${name}“ gibt es noch nicht als Seite.`);
  }

  function openChapter(file: string) {
    setCenter({ kind: "pages" });
    setSelected(file);
    setSourceFile(file);
    setSourceJump(null);
    setScrollTo({ file, nonce: Date.now() });
  }

  // ⌘1 bis ⌘9: Kapitel wechseln.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.metaKey || e.shiftKey || e.altKey || !/^[1-9]$/.test(e.key)) return;
      const chapter = chapters[Number(e.key) - 1];
      if (!chapter) return;
      e.preventDefault();
      openChapter(chapter.file);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  function openFromIndex(path: string) {
    setSelected(path);
    setCenter({ kind: "note", path, readOnly: !path.startsWith("Notizen/") });
  }

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
      setErrors(info.compile_errors);
      // Lässt sich die Arbeit nicht setzen, direkt an die Fehlerstelle im Quelltext.
      if (info.error_location) {
        const [file, offset] = info.error_location;
        setSourceMode(true);
        setSourceFile(file);
        setSelected(file);
        setSourceJump({ file, offset, nonce: Date.now() });
      }
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
      defaultPath: `${await exportDir()}/${project.name}.pdf`,
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
    refreshIndex();
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

  async function createChapter(title: string) {
    try {
      const path = await newChapter(title);
      setTree(await projectTree());
      setSelected(path);
      setSourceFile(path);
      setSourceJump(null);
      setCenter({ kind: "pages" });
      setScrollTo({ file: path, nonce: Date.now() });
      setRefresh((n) => n + 1);
    } catch (e) {
      setErrors(errorMessages(e));
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
      <Topbar
        projectName={project.name}
        saveLabel={saveState}
        sourceMode={sourceMode}
        onSourceMode={setSourceMode}
        onExport={exportToPdf}
        onChoose={choose}
        theme={theme}
        onTheme={setTheme}
      />
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
      <Workspace
        left={
        <div className="left">
          <Kapitelliste chapters={chapters} current={currentChapter} onOpen={openChapter} onNew={createChapter} />
          <ChapterPanel
            chapter={chapterView}
            sources={sources.sources}
            sourcesError={sources.error}
            onOpenFile={openFromIndex}
          />
          <Sidebar tree={tree} selected={selected} onOpen={openNode} onNewNote={createNote} onNewChapter={createChapter} />
        </div>
        }
        center={
        <main className="center flex flex-col">
          {center.kind === "pages" && <Einfuegeleiste sources={sources.sources} projectRoot={project.root} />}
          <div className="min-h-0 flex-1" style={{ overflow: "clip" }}>
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
                onCursor={(offset) => setReveal({ file: sourceFile, offset, nonce: Date.now() })}
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
                reveal={reveal}
                marks={marks}
              />
            </div>
          ) : center.kind === "pages" ? (
            <PageEditor
              key={project.root}
              scrollTo={scrollTo}
              onReport={setReport}
              onSaved={saved}
              onSaving={() => setSaveState("speichert …")}
              refresh={refresh}
              onVisiblePage={setVisiblePage}
              onEditingFile={setEditingFile}
              onEditingParagraph={(p) => p && setLastParagraph(p)}
              reveal={reveal}
              marks={marks}
            />
          ) : center.path.startsWith("Wissen/") ? (
            <WissenView path={center.path} onOpenLink={openLink} version={indexVersion} />
          ) : (
            <NoteEditor
              key={center.path}
              path={center.path}
              readOnly={center.readOnly}
              onSaved={saved}
              onSaving={() => setSaveState("speichert …")}
            />
          )}
          </div>
        </main>
        }
        right={
        <RightPanel
          bericht={bericht}
          rechtschreibung={rechtschreibung}
          onJump={jumpToHint}
          assistant={
            <Assistant
              key={project.root}
              paragraph={lastParagraph}
              chapter={currentChapter}
              onLogged={() => setProtokollVersion((n) => n + 1)}
              onJump={(datei, offset) => jumpToHint({ datei, offset } as Hinweis)}
              onOpenFile={openFromIndex}
              onExported={() => {
                projectTree().then(setTree).catch(() => {});
                setRefresh((n) => n + 1);
              }}
            />
          }
        />
        }
      />
      <StatusBar
        contentPages={report?.content_pages ?? null}
        range={bericht?.umfang ?? null}
        openHints={allHints.length}
        lastAutosave={lastAutosave}
        git={project.git}
        gitError={gitError}
        errorCount={errorCount}
        spellchecking={rechtschreibung === "laeuft"}
        protokollVersion={protokollVersion}
      />
    </div>
  );
}

export default App;
