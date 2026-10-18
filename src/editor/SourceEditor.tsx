import { useEffect, useRef, useState } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { Definition, Field, SaveInfo, errorMessages, log, readFile, utf8Length, writeFile } from "./api";
import { lspExtensions, watchDiagnostics } from "./lspExtensions";
import { Inserter, clearInserter, setInserter } from "./inserter";

/** Speichern und neu setzen kurz nach dem letzten Tastendruck. */
const SAVE_DELAY_MS = 300;

interface Props {
  path: string;
  /** Sprungziel im Quelltext (Byte-Offset in `file`); `nonce` erzwingt erneuten Sprung. */
  jump: { file: string; offset: number; nonce: number } | null;
  onSaved: (save: SaveInfo) => void;
  onSaving: () => void;
  /** Nach dem Speichern: Seiten neu setzen. */
  onWritten: () => void;
  onDefinition: (definition: Definition) => void;
  /** Klick in den Quelltext: Byte-Position, damit die Vorschau dorthin springt. */
  onCursor: (offset: number) => void;
}

/** Byte-Offset (Rust) → UTF-16-Index (CodeMirror). */
function utf16Index(text: string, byte: number): number {
  const bytes = new TextEncoder().encode(text);
  return new TextDecoder().decode(bytes.slice(0, byte)).length;
}

function jumpTo(view: EditorView, offset: number) {
  log(`Quelltext: Sprung zu Byte ${offset}`);
  const pos = Math.min(utf16Index(view.state.doc.toString(), offset), view.state.doc.length);
  view.dispatch({ selection: { anchor: pos }, effects: EditorView.scrollIntoView(pos, { y: "center" }) });
  view.focus();
}

/** Quelltextmodus: ganze Datei mit Tinymist, z. B. für Tabellen, Abbildungen und Code. */
export function SourceEditor({ path, jump, onSaved, onSaving, onWritten, onDefinition, onCursor }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const jumpRef = useRef(jump);
  jumpRef.current = jump;
  const [error, setError] = useState<string | null>(null);
  const callbacks = useRef({ onSaved, onSaving, onWritten, onDefinition, onCursor });
  callbacks.current = { onSaved, onSaving, onWritten, onDefinition, onCursor };

  useEffect(() => {
    // Wurde der Effekt schon wieder abgebaut, bevor die Datei gelesen war (z. B. doppelter
    // Effekt im Entwicklungsmodus), keinen verwaisten Editor erzeugen.
    let cancelled = false;
    let timer: number | undefined;
    let pending: string | null = null;
    let stopDiagnostics = () => {};
    let inserter: Inserter | null = null;
    const field = (doc: string): Field => ({ file: path, start: 0, end: 0, text: doc, whole_file: true });
    const save = () => {
      if (pending === null) return;
      const text = pending;
      pending = null;
      writeFile(path, text)
        .then((info) => {
          callbacks.current.onSaved(info);
          callbacks.current.onWritten();
        })
        .catch((e) => setError(errorMessages(e).join(" ")));
    };
    readFile(path)
      .then((text) => {
        log(`Quelltext: ${path} gelesen (${text.length} Zeichen)${cancelled ? ", Effekt schon abgebaut" : ""}`);
        if (cancelled) return;
        const editor = new EditorView({
          parent: host.current!,
          state: EditorState.create({
            doc: text,
            extensions: [
              lineNumbers(),
              history(),
              keymap.of([...defaultKeymap, ...historyKeymap]),
              EditorView.lineWrapping,
              ...lspExtensions(field, (d) => callbacks.current.onDefinition(d)),
              // Klick (ohne ⌘) in den Quelltext: Vorschau zur Stelle bewegen.
              EditorView.domEventHandlers({
                mouseup: (event, v) => {
                  if (event.metaKey) return false;
                  window.setTimeout(() => {
                    const head = v.state.selection.main.head;
                    callbacks.current.onCursor(utf8Length(v.state.doc.sliceString(0, head)));
                  }, 0);
                  return false;
                },
              }),
              EditorView.updateListener.of((u) => {
                if (!u.docChanged) return;
                pending = u.state.doc.toString();
                callbacks.current.onSaving();
                window.clearTimeout(timer);
                timer = window.setTimeout(save, SAVE_DELAY_MS);
              }),
            ],
          }),
        });
        view.current = editor;
        inserter = {
          file: path,
          insert: (snippet) => {
            const { from, to } = editor.state.selection.main;
            const [a, b] = snippet.select ?? [snippet.text.length, snippet.text.length];
            editor.dispatch({ changes: { from, to, insert: snippet.text }, selection: { anchor: from + a, head: from + b }, scrollIntoView: true });
            editor.focus();
          },
        };
        setInserter(inserter);
        const box = host.current!.getBoundingClientRect();
        log(`Quelltext: Editor für ${path} erstellt, Bereich ${box.width.toFixed(0)}×${box.height.toFixed(0)} px, Editoren: ${host.current!.querySelectorAll(".cm-editor").length}`);
        stopDiagnostics = watchDiagnostics(editor, path, field);
        // Nur springen, wenn das Ziel in dieser Datei liegt; sonst steht Zeile 1 oben.
        if (jumpRef.current?.file === path) jumpTo(editor, jumpRef.current.offset);
      })
      .catch((e) => {
        log(`Quelltext: ${path} nicht lesbar: ${errorMessages(e).join(" ")}`);
        setError(errorMessages(e).join(" "));
      });
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
      save();
      stopDiagnostics();
      if (inserter) clearInserter(inserter);
      view.current?.destroy();
      view.current = null;
    };
  }, [path]);

  useEffect(() => {
    if (jump?.file === path && view.current) jumpTo(view.current, jump.offset);
  }, [jump, path]);

  return (
    <div className="source-editor">
      <div className="note-title">{path}</div>
      {error && <div className="errors">{error}</div>}
      <div className="source-body" ref={host} />
    </div>
  );
}
