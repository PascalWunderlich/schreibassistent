import { useEffect, useRef } from "react";
import { EditorState, Prec } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, insertNewline } from "@codemirror/commands";

interface Props {
  text: string;
  fontSize: number;
  onCommit: (text: string) => void;
  onCancel: () => void;
}

/** Quelltext eines Absatzes. Enter übernimmt, Shift+Enter bricht um, Esc verwirft. */
export function ParagraphEditor({ text, fontSize, onCommit, onCancel }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const callbacks = useRef({ onCommit, onCancel });
  callbacks.current = { onCommit, onCancel };

  useEffect(() => {
    const keys = Prec.highest(
      keymap.of([
        { key: "Enter", run: (view) => (callbacks.current.onCommit(view.state.doc.toString()), true) },
        { key: "Shift-Enter", run: insertNewline },
        { key: "Escape", run: () => (callbacks.current.onCancel(), true) },
      ]),
    );
    const view = new EditorView({
      parent: host.current!,
      state: EditorState.create({
        doc: text,
        extensions: [keys, history(), keymap.of([...defaultKeymap, ...historyKeymap]), EditorView.lineWrapping],
      }),
    });
    view.focus();
    view.dispatch({ selection: { anchor: text.length } });
    return () => view.destroy();
  }, [text]);

  return <div className="paragraph-editor" ref={host} style={{ fontSize }} />;
}
