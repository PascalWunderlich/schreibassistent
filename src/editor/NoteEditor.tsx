import { useEffect, useRef, useState } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { SaveInfo, errorMessages, readFile, writeFile } from "./api";

const SAVE_DELAY_MS = 1000;

interface Props {
  path: string;
  /** `Wissen/` ist nur lesbar (KI-gepflegt, docs/DATA-FORMATS.md). */
  readOnly: boolean;
  onSaved: (save: SaveInfo) => void;
  onSaving: () => void;
}

/** Markdown-Editor für Notizen; speichert spätestens 1 s nach dem letzten Tastendruck. */
export function NoteEditor({ path, readOnly, onSaved, onSaving }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);
  const callbacks = useRef({ onSaved, onSaving });
  callbacks.current = { onSaved, onSaving };

  useEffect(() => {
    let view: EditorView | null = null;
    let timer: number | undefined;
    let pending: string | null = null;
    const save = () => {
      if (pending === null) return;
      const text = pending;
      pending = null;
      writeFile(path, text).then(callbacks.current.onSaved).catch((e) => setError(errorMessages(e).join(" ")));
    };
    readFile(path)
      .then((text) => {
        view = new EditorView({
          parent: host.current!,
          state: EditorState.create({
            doc: text,
            extensions: [
              history(),
              keymap.of([...defaultKeymap, ...historyKeymap]),
              markdown(),
              EditorView.lineWrapping,
              EditorState.readOnly.of(readOnly),
              EditorView.updateListener.of((u) => {
                if (!u.docChanged || readOnly) return;
                pending = u.state.doc.toString();
                callbacks.current.onSaving();
                window.clearTimeout(timer);
                timer = window.setTimeout(save, SAVE_DELAY_MS);
              }),
            ],
          }),
        });
      })
      .catch((e) => setError(errorMessages(e).join(" ")));
    return () => {
      // Beim Wechsel der Datei sofort speichern, was noch aussteht.
      window.clearTimeout(timer);
      save();
      view?.destroy();
    };
  }, [path, readOnly]);

  return (
    <div className="note-editor">
      <div className="note-title">
        {path}
        {readOnly && <span className="badge">nur lesen</span>}
      </div>
      {error && <div className="errors">{error}</div>}
      <div className="note-body" ref={host} />
    </div>
  );
}
