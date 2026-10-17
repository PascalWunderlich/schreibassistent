import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { debugLog } from "../satz";
import {
  LspNotification,
  PreviewSession,
  changeFile,
  openPreview,
  readPosition,
  relativePath,
  scrollPreview,
} from "../lsp";

const median = (values: number[]) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];

/** Prototyp B: Quelltext links, Tinymist-Vorschau rechts, Sprung in beide Richtungen. */
export function PreviewEditor() {
  const [session, setSession] = useState<PreviewSession | null>(null);
  const [path, setPath] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [timings, setTimings] = useState<number[]>([]);
  const [lastJump, setLastJump] = useState<string>("");
  const texts = useRef(new Map<string, string>());
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const sentAt = useRef<number | null>(null);
  const pendingJump = useRef<{ line: number; character: number } | null>(null);
  const scrollTimer = useRef<number | undefined>(undefined);

  useEffect(() => {
    openPreview()
      .then((s) => {
        s.files.forEach((f) => texts.current.set(f.path, f.text));
        setSession(s);
        setPath(s.files.find((f) => f.path.includes("kapitel/"))?.path ?? s.files[0]?.path ?? "");
      })
      .catch((e) => {
        debugLog(`B: Fehler beim Start ${String(e)}`);
        setError(String(e));
      });

    const unlisten = listen<LspNotification>("lsp_notification", async ({ payload }) => {
      if (payload.method === "tinymist/compileStatus") {
        const status = (payload.params as { status?: string }).status;
        if (status && status !== "compiling" && sentAt.current !== null) {
          const ms = performance.now() - sentAt.current;
          sentAt.current = null;
          debugLog(`B: Änderung bis gesetzt ${ms.toFixed(1)} ms (${status})`);
          setTimings((t) => [...t, ms]);
        }
      } else if (payload.method === "window/showDocument" || payload.method === "tinymist/preview/scrollSource") {
        // Klick in der Vorschau: Tinymist 0.15 schickt `window/showDocument` {uri, selection}.
        const params = payload.params as {
          uri?: string;
          filepath?: string;
          selection?: { start?: unknown };
          start?: unknown;
        };
        setLastJump(JSON.stringify(params));
        debugLog(`B: ${payload.method} ${JSON.stringify(params)}`);
        const file = params.uri ?? params.filepath;
        const target = file ? await relativePath(file) : null;
        const position = readPosition(params.selection?.start ?? params.start);
        if (!target || !position) return;
        pendingJump.current = position;
        setPath((current) => {
          if (current === target) jumpTo(position);
          return target;
        });
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  function jumpTo({ line, character }: { line: number; character: number }) {
    const v = view.current;
    if (!v) return;
    const doc = v.state.doc;
    const lineInfo = doc.line(Math.min(line + 1, doc.lines));
    const pos = Math.min(lineInfo.from + character, lineInfo.to);
    v.dispatch({ selection: { anchor: pos }, effects: EditorView.scrollIntoView(pos, { y: "center" }) });
    v.focus();
    pendingJump.current = null;
  }

  // Editor je Datei neu aufbauen; Text bleibt in `texts` erhalten.
  useEffect(() => {
    if (!path || !host.current) return;
    const current = path;
    const editor = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: texts.current.get(current) ?? "",
        extensions: [
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          EditorView.updateListener.of((update) => {
            if (update.docChanged) {
              const text = update.state.doc.toString();
              texts.current.set(current, text);
              sentAt.current = performance.now();
              changeFile(current, text).catch((e) => setError(String(e)));
            }
          }),
          // Vorschau nur bei einem Klick in den Quelltext springen lassen, nicht beim Tippen:
          // Jeder Sprung lässt Tinymist das Dokument neu an die Vorschau schicken (Flackern).
          EditorView.domEventHandlers({
            mouseup: (_event, v) => {
              window.clearTimeout(scrollTimer.current);
              scrollTimer.current = window.setTimeout(() => {
                const head = v.state.selection.main.head;
                const line = v.state.doc.lineAt(head);
                debugLog(`B: Sprung zur Vorschau ${current}:${line.number - 1}:${head - line.from}`);
                scrollPreview(current, line.number - 1, head - line.from).catch(() => {});
              }, 50);
            },
          }),
        ],
      }),
    });
    view.current = editor;
    if (pendingJump.current) jumpTo(pendingJump.current);
    return () => editor.destroy();
  }, [path]);

  if (error) return <div className="errors">{error}</div>;
  if (!session) return <p className="placeholder">Tinymist startet …</p>;

  return (
    <div className="preview-editor">
      <div className="status">
        <select value={path} onChange={(e) => setPath(e.target.value)}>
          {session.files.map((f) => (
            <option key={f.path} value={f.path}>
              {f.path}
            </option>
          ))}
        </select>
        {timings.length > 0 && (
          <span>
            {" "}
            Änderung → gesetzt: letzte {timings[timings.length - 1].toFixed(0)} ms · Median{" "}
            {median(timings).toFixed(0)} ms · schlechtester {Math.max(...timings).toFixed(0)} ms · n ={" "}
            {timings.length}
          </span>
        )}
        {lastJump && <span className="jump"> Letzter Sprung: {lastJump}</span>}
      </div>
      <div className="split">
        <div className="source" ref={host} />
        <iframe className="preview" src={session.preview_url} title="Vorschau" />
      </div>
    </div>
  );
}
