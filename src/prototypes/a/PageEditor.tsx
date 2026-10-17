import { useEffect, useState } from "react";
import { ParagraphEditor } from "./ParagraphEditor";
import {
  CompileReport,
  ParagraphHit,
  errorMessages,
  openExample,
  paragraphAt,
  replaceText,
} from "../satz";

/** Pixel je pt auf dem Bildschirm. */
const SCALE = 1.4;

interface Page {
  svg: string;
  width: number;
  height: number;
}

interface Timing {
  totalMs: number;
  compileMs: number;
  svgMs: number;
  changedPages: number[];
}

interface Editing {
  hit: ParagraphHit;
  page: number;
}

function applyReport(pages: Page[], report: CompileReport): Page[] {
  const next = pages.slice(0, report.page_count);
  report.page_sizes.forEach(([width, height], i) => {
    next[i] = { svg: next[i]?.svg ?? "", width, height };
  });
  report.changed_pages.forEach((page, i) => {
    next[page] = { ...next[page], svg: report.svgs[i] };
  });
  return next;
}

const median = (values: number[]) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];

/** Prototyp A: echte Seiten, Klick auf einen Absatz öffnet seinen Quelltext an Ort und Stelle. */
export function PageEditor() {
  const [pages, setPages] = useState<Page[]>([]);
  const [editing, setEditing] = useState<Editing | null>(null);
  const [errors, setErrors] = useState<string[]>([]);
  const [timings, setTimings] = useState<Timing[]>([]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    openExample()
      .then((report) => setPages(applyReport([], report)))
      .catch((e) => setErrors(errorMessages(e)));
  }, []);

  async function handleClick(page: number, event: React.MouseEvent<HTMLDivElement>) {
    if (busy) return;
    const box = event.currentTarget.getBoundingClientRect();
    const x = (event.clientX - box.left) / SCALE;
    const y = (event.clientY - box.top) / SCALE;
    try {
      const hit = await paragraphAt(page, x, y);
      setEditing(hit ? { hit, page } : null);
    } catch (e) {
      setErrors(errorMessages(e));
    }
  }

  async function commit(text: string) {
    if (!editing) return;
    const { hit } = editing;
    if (text === hit.text) {
      setEditing(null);
      return;
    }
    setBusy(true);
    const start = performance.now();
    try {
      const report = await replaceText(hit.file, hit.start, hit.end, text);
      setPages((current) => applyReport(current, report));
      setEditing(null);
      setErrors(report.warnings);
      // Zeit bis die neuen Seiten tatsächlich gezeichnet sind.
      requestAnimationFrame(() =>
        requestAnimationFrame(() =>
          setTimings((t) => [
            ...t,
            {
              totalMs: performance.now() - start,
              compileMs: report.compile_ms,
              svgMs: report.svg_ms,
              changedPages: report.changed_pages,
            },
          ]),
        ),
      );
    } catch (e) {
      // Bei Fehlern (z. B. Syntaxfehler) bleibt das Feld offen.
      setErrors(errorMessages(e));
    } finally {
      setBusy(false);
    }
  }

  const last = timings[timings.length - 1];
  const totals = timings.map((t) => t.totalMs);

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
          <span>Absatz anklicken. Enter übernimmt, Shift+Enter bricht um, Esc verwirft.</span>
        )}
      </div>
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
            className="page"
            style={{ width: page.width * SCALE, height: page.height * SCALE }}
            onClick={(e) => handleClick(i, e)}
          >
            <div className="page-svg" dangerouslySetInnerHTML={{ __html: page.svg }} />
            {editing &&
              editing.hit.rects
                .filter(([p]) => p === i)
                .slice(0, 1)
                .map(([, rect]) => (
                  <div
                    key="editor"
                    className="editor-overlay"
                    style={{
                      left: rect.x * SCALE - 6,
                      top: rect.y * SCALE - 6,
                      width: rect.width * SCALE + 12,
                      minHeight: rect.height * SCALE + 12,
                    }}
                    onClick={(e) => e.stopPropagation()}
                  >
                    <ParagraphEditor
                      text={editing.hit.text}
                      fontSize={11 * SCALE}
                      onCommit={commit}
                      onCancel={() => setEditing(null)}
                    />
                  </div>
                ))}
          </div>
        ))}
      </div>
    </div>
  );
}
