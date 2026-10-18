import { useEffect, useRef } from "react";
import { EditorState, Prec } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, insertNewline } from "@codemirror/commands";
import { Definition, Field } from "./api";
import { lspExtensions, watchDiagnostics } from "./lspExtensions";
import { clearInserter, setInserter } from "./inserter";

interface Props {
  /** Datei und Bereich des Absatzes im Quelltext (Stand des Satzes). */
  file: string;
  start: number;
  end: number;
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
  /** Jede Änderung (für das Speichern als Entwurf). */
  onChange: (text: string) => void;
  /** ⌘-Klick: Sprung zur Definition. */
  onDefinition: (definition: Definition) => void;
  /** Zugriff auf den aktuellen Text, z. B. beim Klick außerhalb. */
  textRef: React.RefObject<(() => string) | null>;
}

/**
 * Quelltext eines Absatzes mit Tinymist: Vervollständigung, Fehler, Hover, ⌘-Klick.
 * Enter beginnt einen neuen Absatz, Shift+Enter bricht die Quelltextzeile um,
 * Esc oder ein Klick daneben übernimmt und schließt.
 */
export function ParagraphEditor(props: Props) {
  const { file, start, end, text, fontSize, lineHeight, indent, bold, textRef } = props;
  const host = useRef<HTMLDivElement>(null);
  const callbacks = useRef(props);
  callbacks.current = props;

  useEffect(() => {
    const field = (doc: string): Field => ({ file, start, end, text: doc, whole_file: false });

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
        extensions: [
          ...lspExtensions(field, (d) => callbacks.current.onDefinition(d)),
          keys,
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => u.docChanged && callbacks.current.onChange(u.state.doc.toString())),
        ],
      }),
    });
    textRef.current = () => view.state.doc.toString();
    view.focus();
    view.dispatch({ selection: { anchor: text.length } });

    const stopDiagnostics = watchDiagnostics(view, file, field);
    const inserter = {
      file,
      insert: (snippet: import("./snippets").Snippet) => {
        const { from, to } = view.state.selection.main;
        const [a, b] = snippet.select ?? [snippet.text.length, snippet.text.length];
        view.dispatch({ changes: { from, to, insert: snippet.text }, selection: { anchor: from + a, head: from + b } });
        view.focus();
      },
    };
    setInserter(inserter);

    return () => {
      clearInserter(inserter);
      stopDiagnostics();
      textRef.current = null;
      view.destroy();
    };
  }, [file, start, end, text, textRef]);

  return (
    <div
      className="paragraph-editor"
      lang="de"
      ref={host}
      style={{ fontSize, lineHeight: `${lineHeight}px`, fontWeight: bold ? 700 : 400, ["--indent" as string]: `${indent}px` }}
    />
  );
}
