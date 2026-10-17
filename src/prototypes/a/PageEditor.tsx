import { useEffect, useRef, useState } from "react";
import { ParagraphEditor } from "./ParagraphEditor";
import {
  CompileReport,
  ParagraphHit,
  debugLog,
  errorMessages,
  loadEditorFont,
  openExample,
  paragraphAt,
  paragraphAtOffset,
  replaceText,
  utf8Length,
} from "../satz";

/** Pixel je pt auf dem Bildschirm. */
const SCALE = 1.4;

interface Page {
  /** Objekt-URL des Seiten-SVG; jede Seite ist ein eigenes Bild (leicht für den Browser). */
  url: string;
  width: number;
  height: number;
  /** Gedruckte Seitenzahl („i“, „1“ …). */
  label: string;
}

interface Timing {
  totalMs: number;
  compileMs: number;
  svgMs: number;
  changedPages: number[];
}

const svgUrl = (svg: string) => URL.createObjectURL(new Blob([svg], { type: "image/svg+xml" }));

function applyReport(pages: Page[], report: CompileReport): Page[] {
  const next = pages.slice(0, report.page_count);
  pages.slice(report.page_count).forEach((p) => p.url && URL.revokeObjectURL(p.url));
  report.page_sizes.forEach(([width, height], i) => {
    next[i] = { url: next[i]?.url ?? "", width, height, label: report.page_labels[i] };
  });
  report.changed_pages.forEach((page, i) => {
    if (next[page].url) URL.revokeObjectURL(next[page].url);
    next[page] = { ...next[page], url: svgUrl(report.svgs[i]) };
  });
  return next;
}

const median = (values: number[]) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];

const TEMPLATE_HINT =
  "Hier ist kein bearbeitbarer Absatz. Titelblatt, Erklärung, Verzeichnisse, Kopfzeilen und Tabellen aus Vorlagenfunktionen kommen aus der Vorlage; dein Text beginnt bei Kapitel 1.";

/** Prototyp A: echte Seiten, Klick auf einen Absatz öffnet seinen Quelltext an Ort und Stelle. */
export function PageEditor() {
  const [pages, setPages] = useState<Page[]>([]);
  const [editing, setEditing] = useState<ParagraphHit | null>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [timings, setTimings] = useState<Timing[]>([]);
  const [hint, setHint] = useState<string | null>(null);
  const busy = useRef(false);
  const currentText = useRef<(() => string) | null>(null);

  useEffect(() => {
    debugLog("A: open");
    openExample()
      .then((report) => {
        debugLog(`A: ${report.page_count} Seiten geladen (${report.compile_ms.toFixed(0)} ms)`);
        setPages(applyReport([], report));
        loadEditorFont().catch((e) => debugLog(`A: Schrift nicht geladen ${errorMessages(e).join(" | ")}`));
        // Direkt beim Inhaltsteil starten: davor steht nur Text aus der Vorlage.
        requestAnimationFrame(() => document.getElementById("first-chapter")?.scrollIntoView?.());
      })
      .catch((e) => {
        debugLog(`A: Fehler beim Öffnen ${errorMessages(e).join(" | ")}`);
        setErrors(errorMessages(e));
      });
  }, []);

  /** Ersetzt den Absatz, misst die Zeit bis zum Neuzeichnen. Gibt `false` bei Fehlern zurück. */
  async function replace(hit: ParagraphHit, text: string, what: string): Promise<boolean> {
    const start = performance.now();
    try {
      const report = await replaceText(hit.file, hit.start, hit.end, text);
      setPages((current) => applyReport(current, report));
      setErrors(report.warnings);
      debugLog(
        `A: ${what}, Satz ${report.compile_ms.toFixed(1)} ms, SVG ${report.svg_ms.toFixed(1)} ms, Seiten ${report.changed_pages.map((p) => p + 1).join(",")}`,
      );
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          const totalMs = performance.now() - start;
          debugLog(`A: ${what} bis gezeichnet ${totalMs.toFixed(1)} ms`);
          setTimings((t) => [
            ...t,
            { totalMs, compileMs: report.compile_ms, svgMs: report.svg_ms, changedPages: report.changed_pages },
          ]);
        }),
      );
      return true;
    } catch (e) {
      // Bei Fehlern (z. B. Syntaxfehler) bleibt das Feld mit dem Text offen.
      debugLog(`A: Fehler (${what}) ${errorMessages(e).join(" | ")}`);
      setErrors(errorMessages(e));
      return false;
    }
  }

  /** Esc oder Klick daneben: übernehmen und schließen. */
  async function commit(text: string) {
    const hit = editing;
    if (!hit || busy.current) return;
    busy.current = true;
    try {
      if (text === hit.text || (await replace(hit, text, "übernommen"))) setEditing(null);
    } finally {
      busy.current = false;
    }
  }

  /** Enter: Absatz an der Cursorposition teilen; der zweite Teil (ggf. leer) wird bearbeitet. */
  async function split(text: string, cursor: number) {
    const hit = editing;
    if (!hit || busy.current) return;
    busy.current = true;
    try {
      const before = text.slice(0, cursor).trimEnd();
      const after = text.slice(cursor).trimStart();
      if (!(await replace(hit, `${before}\n\n${after}`, "Enter (neuer Absatz)"))) return;
      const next = await paragraphAtOffset(hit.file, hit.start + utf8Length(before) + 2);
      debugLog(next ? `A: neuer Absatz ${next.start}..${next.end}` : "A: neuer Absatz nicht gefunden");
      setEditing(next);
    } finally {
      busy.current = false;
    }
  }

  async function handleClick(page: number, event: React.MouseEvent<HTMLDivElement>) {
    const box = event.currentTarget.getBoundingClientRect();
    const x = (event.clientX - box.left) / SCALE;
    const y = (event.clientY - box.top) / SCALE;
    debugLog(`A: Klick Seite ${page + 1} bei (${x.toFixed(0)}, ${y.toFixed(0)}) pt`);
    if (busy.current) return;
    // Offenes Feld zuerst übernehmen.
    if (editing && currentText.current) {
      await commit(currentText.current());
    }
    try {
      const hit = await paragraphAt(page, x, y);
      debugLog(hit ? `A: Treffer ${hit.file} ${hit.start}..${hit.end}` : "A: kein Absatz");
      setEditing(hit);
      setHint(hit ? null : TEMPLATE_HINT);
    } catch (e) {
      debugLog(`A: Fehler beim Klick ${errorMessages(e).join(" | ")}`);
      setErrors(errorMessages(e));
    }
  }

  const last = timings[timings.length - 1];
  const totals = timings.map((t) => t.totalMs);
  const firstChapter = pages.findIndex((p) => p.label === "1");

  return (
    <div className="page-editor">
      <div className="status">
        {last ? (
          <span>
            Letzte Änderung: <b>{last.totalMs.toFixed(0)} ms</b> (Satz {last.compileMs.toFixed(0)} ms, SVG{" "}
            {last.svgMs.toFixed(0)} ms), neu gezeichnet: Seite {last.changedPages.map((p) => p + 1).join(", ") || "–"}{" "}
            · Median {median(totals).toFixed(0)} ms · schlechtester {Math.max(...totals).toFixed(0)} ms · n ={" "}
            {timings.length}
          </span>
        ) : (
          <span>Absatz anklicken. Enter: neuer Absatz · Shift+Enter: Zeilenumbruch · Esc oder Klick daneben: fertig.</span>
        )}{" "}
        <button className="link" onClick={() => document.getElementById("first-chapter")?.scrollIntoView?.()}>
          Zu Kapitel 1
        </button>
      </div>
      {hint && <div className="hint">{hint}</div>}
      {errors.length > 0 && (
        <div className="errors" role="alert">
          {errors.map((e, i) => (
            <div key={i}>{e}</div>
          ))}
        </div>
      )}
      <div className="pages">
        {pages.map((page, i) => (
          <div
            key={i}
            id={i === firstChapter ? "first-chapter" : undefined}
            className="page"
            style={{ width: page.width * SCALE, height: page.height * SCALE }}
            onClick={(e) => handleClick(i, e)}
          >
            {page.url && <img className="page-image" src={page.url} alt={`Seite ${page.label || i + 1}`} draggable={false} />}
            {editing?.rects
              .filter(([p]) => p === i)
              .slice(0, 1)
              .map(([, rect]) => (
                <div
                  key={`${editing.file}:${editing.start}`}
                  className="editor-overlay"
                  style={{
                    // Spaltenbreite wie im Satz; erste Grundlinie auf die gesetzte Grundlinie legen.
                    left: editing.column_x * SCALE,
                    width: editing.column_width * SCALE,
                    top: (rect.y - (editing.line_height - rect.height) / 2) * SCALE,
                    minHeight: Math.max(rect.height, editing.line_height) * SCALE,
                  }}
                  onClick={(e) => e.stopPropagation()}
                >
                  <ParagraphEditor
                    text={editing.text}
                    fontSize={editing.font_size * SCALE}
                    lineHeight={editing.line_height * SCALE}
                    indent={editing.first_line_indent * SCALE}
                    bold={editing.text.trimStart().startsWith("=")}
                    onSplit={split}
                    onCommit={commit}
                    textRef={currentText}
                  />
                </div>
              ))}
          </div>
        ))}
      </div>
    </div>
  );
}
