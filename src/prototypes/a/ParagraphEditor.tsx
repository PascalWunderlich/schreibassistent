import { useEffect, useRef } from "react";
import { EditorState, Prec } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, insertNewline } from "@codemirror/commands";

interface Props {
  text: string;
  /** Schriftgröße und Zeilenabstand in px, wie im Satz. */
  fontSize: number;
  lineHeight: number;
  /** Einzug der ersten Zeile in px. */
  indent: number;
  bold: boolean;
  /** Enter: Absatz an der Cursorposition (UTF-16-Index) teilen bzw. neuen Absatz anlegen. */
  onSplit: (text: string, cursor: number) => void;
  /** Esc: übernehmen und schließen. */
  onCommit: (text: string) => void;
  /** Zugriff auf den aktuellen Text, z. B. beim Klick außerhalb. */
  textRef: React.RefObject<(() => string) | null>;
}

/**
 * Quelltext eines Absatzes. Enter beginnt einen neuen Absatz, Shift+Enter bricht die
 * Quelltextzeile um, Esc oder ein Klick daneben übernimmt und schließt.
 */
export function ParagraphEditor({ text, fontSize, lineHeight, indent, bold, onSplit, onCommit, textRef }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const callbacks = useRef({ onSplit, onCommit });
  callbacks.current = { onSplit, onCommit };

  useEffect(() => {
    const keys = Prec.highest(
      keymap.of([
        {
          key: "Enter",
          run: (view) => (callbacks.current.onSplit(view.state.doc.toString(), view.state.selection.main.head), true),
        },
        { key: "Shift-Enter", run: insertNewline },
        { key: "Escape", run: (view) => (callbacks.current.onCommit(view.state.doc.toString()), true) },
      ]),
    );
    const view = new EditorView({
      parent: host.current!,
      state: EditorState.create({
        doc: text,
        extensions: [keys, history(), keymap.of([...defaultKeymap, ...historyKeymap]), EditorView.lineWrapping],
      }),
    });
    textRef.current = () => view.state.doc.toString();
    view.focus();
    view.dispatch({ selection: { anchor: text.length } });
    return () => {
      textRef.current = null;
      view.destroy();
    };
  }, [text, textRef]);

  return (
    <div
      className="paragraph-editor"
      lang="de"
      ref={host}
      style={{ fontSize, lineHeight: `${lineHeight}px`, fontWeight: bold ? 700 : 400, ["--indent" as string]: `${indent}px` }}
    />
  );
}
