// Tinymist in CodeMirror: Vervollständigung, Hover, ⌘-Klick zur Definition, Fehler.
// Gemeinsam für das Feld im Seiteneditor und den Quelltextmodus.
import { Extension } from "@codemirror/state";
import { EditorView, hoverTooltip } from "@codemirror/view";
import { autocompletion, CompletionContext, CompletionResult } from "@codemirror/autocomplete";
import { setDiagnostics } from "@codemirror/lint";
import { listen } from "@tauri-apps/api/event";
import { Definition, Field, lspCompletion, lspDefinition, lspFieldDiagnostics, lspHover } from "./api";

/** Vervollständigung nur nach `@` (Verweise, Citekeys) und `#` (Funktionen) oder mit Ctrl+Leertaste. */
const TRIGGER = /[@#][\p{L}\p{N}_:.-]*$/u;

export function lspExtensions(field: (doc: string) => Field, onDefinition: (d: Definition) => void): Extension[] {
  async function complete(context: CompletionContext): Promise<CompletionResult | null> {
    const line = context.state.doc.lineAt(context.pos);
    const match = TRIGGER.exec(line.text.slice(0, context.pos - line.from));
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
        apply: (view: EditorView) => {
          const start = Math.min(item.from, from);
          view.dispatch({
            changes: { from: start, to: view.state.selection.main.head, insert: item.insert },
            selection: { anchor: start + item.insert.length },
          });
        },
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

  const definition = EditorView.domEventHandlers({
    mousedown: (event, view) => {
      if (!event.metaKey) return false;
      const pos = view.posAtCoords({ x: event.clientX, y: event.clientY });
      if (pos === null) return false;
      lspDefinition(field(view.state.doc.toString()), pos)
        .then((d) => d && onDefinition(d))
        .catch(() => {});
      return true;
    },
  });

  return [autocompletion({ override: [complete], activateOnTyping: true }), hover, definition];
}

/** Fehler von Tinymist im Editor anzeigen, sobald Tinymist `file` neu geprüft hat. Gibt Aufräumfunktion zurück. */
export function watchDiagnostics(view: EditorView, file: string, field: (doc: string) => Field): () => void {
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
  };
}
