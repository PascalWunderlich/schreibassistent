import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { EditorState, Prec } from "@codemirror/state";
import { EditorView, hoverTooltip, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, insertNewline } from "@codemirror/commands";
import { autocompletion, CompletionContext, CompletionResult } from "@codemirror/autocomplete";
import { setDiagnostics } from "@codemirror/lint";
import { Definition, Field, lspCompletion, lspDefinition, lspFieldDiagnostics, lspHover } from "./api";

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

/** Vervollständigung nur nach `@` (Verweise, Citekeys) und `#` (Funktionen) oder mit Ctrl+Leertaste. */
const TRIGGER = /[@#][\p{L}\p{N}_:.-]*$/u;

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
    const field = (doc: string): Field => ({ file, start, end, text: doc });

    async function complete(context: CompletionContext): Promise<CompletionResult | null> {
      const line = context.state.doc.lineAt(context.pos);
      const before = line.text.slice(0, context.pos - line.from);
      const match = TRIGGER.exec(before);
      if (!match && !context.explicit) return null;
      const items = await lspCompletion(field(context.state.doc.toString()), context.pos).catch(() => []);
      if (items.length === 0) return null;
      const from = match ? context.pos - match[0].length + 1 : context.pos;
      return {
        from,
        options: items.map((item) => ({
          label: item.label,
          detail: item.detail ?? undefined,
          // Bereich von Tinymist bis zur aktuellen Cursorposition ersetzen.
          apply: (view: EditorView) =>
            view.dispatch({
              changes: { from: Math.min(item.from, from), to: view.state.selection.main.head, insert: item.insert },
              selection: { anchor: Math.min(item.from, from) + item.insert.length },
            }),
        })),
        validFor: /^[\p{L}\p{N}_:.-]*$/u,
      };
    }

    const hover = hoverTooltip(
      async (view, pos) => {
        const text = await lspHover(field(view.state.doc.toString()), pos).catch(() => null);
        if (!text) return null;
        return {
          pos,
          create: () => {
            const dom = document.createElement("div");
            dom.className = "lsp-hover";
            dom.textContent = text.replace(/```\w*\n?/g, "").replace(/\n---\n/g, "\n");
            return { dom };
          },
        };
      },
      { hoverTime: 500 },
    );

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
          autocompletion({ override: [complete], activateOnTyping: true }),
          keys,
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          EditorView.lineWrapping,
          hover,
          EditorView.updateListener.of((u) => u.docChanged && callbacks.current.onChange(u.state.doc.toString())),
          EditorView.domEventHandlers({
            mousedown: (event, v) => {
              if (!event.metaKey) return false;
              const pos = v.posAtCoords({ x: event.clientX, y: event.clientY });
              if (pos === null) return false;
              lspDefinition(field(v.state.doc.toString()), pos)
                .then((definition) => definition && callbacks.current.onDefinition(definition))
                .catch(() => {});
              return true;
            },
          }),
        ],
      }),
    });
    textRef.current = () => view.state.doc.toString();
    view.focus();
    view.dispatch({ selection: { anchor: text.length } });

    // Fehler von Tinymist, sobald es die Datei neu geprüft hat.
    let active = true;
    const refresh = () =>
      lspFieldDiagnostics(field(view.state.doc.toString()))
        .then((list) => active && view.dispatch(setDiagnostics(view.state, list)))
        .catch(() => {});
    refresh();
    const unlisten = Promise.resolve()
      .then(() => listen<string>("lsp_diagnostics", ({ payload }) => payload === file && refresh()))
      .catch(() => () => {});

    return () => {
      active = false;
      unlisten.then((f) => f?.()).catch(() => {});
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
