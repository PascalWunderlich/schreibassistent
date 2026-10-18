import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ParagraphEditor } from "./ParagraphEditor";
import {
  CompileReport,
  Definition,
  ParagraphHit,
  Rect,
  SaveInfo,
  allPages,
  compile,
  errorMessages,
  firstPage,
  loadEditorFont,
  log,
  paragraphAt,
  paragraphAtOffset,
  replaceText,
  saveDraft,
  utf8Length,
} from "./api";

/** Pixel je pt auf dem Bildschirm (Standard; im Quelltextmodus kleiner). */
const DEFAULT_SCALE = 1.4;

/** Entwurf spätestens nach dieser Zeit ohne Tippen speichern. */
const DRAFT_DELAY_MS = 1000;

interface Page {
  /** Objekt-URL des Seiten-SVG; jede Seite ist ein eigenes Bild (leicht für den Browser). */
  url: string;
  width: number;
  height: number;
  /** Gedruckte Seitenzahl („i“, „1“ …). */
  label: string;
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

const TEMPLATE_HINT =
  "Hier ist kein bearbeitbarer Absatz. Titelblatt, Erklärung, Verzeichnisse und Kopfzeilen kommen aus der Vorlage (Angaben in Arbeit/metadaten.typ).";

interface Props {
  /** Datei, zu deren erster Seite gesprungen werden soll (`nonce` erzwingt erneuten Sprung). */
  scrollTo: { file: string; nonce: number } | null;
  onReport: (report: CompileReport) => void;
  onSaved: (save: SaveInfo) => void;
  onSaving: () => void;
  /** Quelltextmodus: Seiten nur anzeigen; ein Klick meldet den Absatz statt ihn zu öffnen. */
  onPick?: (hit: ParagraphHit) => void;
  /** Quelltextmodus: bei jeder Änderung neu setzen. */
  refresh?: number;
  /** Pixel je pt. */
  scale?: number;
  /** Quelltextmodus: Absatz an dieser Stelle zeigen und kurz hervorheben. */
  reveal?: { file: string; offset: number; nonce: number } | null;
  /** Seite (Index ab 0) in der Mitte des sichtbaren Bereichs, beim Scrollen. */
  onVisiblePage?: (page: number) => void;
  /** Datei des gerade bearbeiteten Absatzes (oder `null`). */
  onEditingFile?: (file: string | null) => void;
  /** Bearbeiteter Absatz (für den Kontext des Assistenten). */
  onEditingParagraph?: (paragraph: { file: string; offset: number } | null) => void;
  /** Markierungen am Absatz (Prüfungen): Seite, Rechteck, Schwere, Texte. */
  marks?: { page: number; rect: Rect; schwere: string; texts: string[] }[];
}

/** Seiteneditor (Entscheidung P2: A): echte Seiten, Klick öffnet den Absatz als Quelltext. */
export function PageEditor({ scrollTo, onReport, onSaved, onSaving, onPick, refresh, scale = DEFAULT_SCALE, reveal, onVisiblePage, onEditingFile, onEditingParagraph, marks = [] }: Props) {
  const [pages, setPages] = useState<Page[]>([]);
  const [editing, setEditing] = useState<ParagraphHit | null>(null);
  /** Leerer Absatz, der gerade per Enter entstanden ist (wird beim Verlassen ohne Text entfernt). */
  const fromSplit = useRef(false);
  const [errors, setErrors] = useState<string[]>([]);
  const [hint, setHint] = useState<string | null>(null);
  /** Kurz hervorgehobener Absatz (Quelltextmodus: Sprung aus dem Quelltext). */
  const [highlight, setHighlight] = useState<ParagraphHit | null>(null);
  const busy = useRef(false);
  const currentText = useRef<(() => string) | null>(null);
  const draftTimer = useRef<number | undefined>(undefined);
  const pageRefs = useRef<(HTMLDivElement | null)[]>([]);

  useEffect(() => {
    allPages()
      .then((report) => {
        setPages(applyReport([], report));
        onReport(report);
        const first = report.page_labels.indexOf("1");
        requestAnimationFrame(() => pageRefs.current[first]?.scrollIntoView?.());
      })
      .catch((e) => setErrors(errorMessages(e)));
    loadEditorFont().catch((e) => log(`Schrift nicht geladen: ${errorMessages(e).join(" | ")}`));
  }, []);

  // Änderung von außen (z. B. Zotero-Export, VS Code): neu setzen.
  const editingRef = useRef(editing);
  editingRef.current = editing;
  useEffect(() => {
    const unlisten = Promise.resolve()
      .then(() =>
        listen<string[]>("datei_geaendert", ({ payload }) => {
          if (editingRef.current && payload.includes(editingRef.current.file)) {
            // Positionen des offenen Absatzes stimmen nicht mehr.
            setEditing(null);
            setHint(`${editingRef.current.file} wurde außerhalb der App geändert. Bitte den Absatz neu anklicken.`);
          }
          compile()
            .then((report) => {
              setPages((current) => applyReport(current, report));
              onReport(report);
            })
            .catch((e) => setErrors(errorMessages(e)));
        }),
      )
      .catch(() => () => {});
    return () => {
      unlisten.then((f) => f?.()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    if (!reveal) return;
    paragraphAtOffset(reveal.file, reveal.offset)
      .then((hit) => {
        if (!hit) return;
        const [page, rect] = hit.rects[0];
        const element = pageRefs.current[page];
        const container = element?.parentElement;
        if (element && container) {
          // Absatz in die Mitte der Vorschau holen (nur diese Spalte scrollt).
          const top = element.offsetTop + rect.y * scale - container.clientHeight / 2;
          container.scrollTo({ top, behavior: "smooth" });
        }
        setHighlight(hit);
        window.setTimeout(() => setHighlight((h) => (h === hit ? null : h)), 1500);
      })
      .catch(() => {});
  }, [reveal]);

  useEffect(() => {
    onEditingFile?.(editing?.file ?? null);
  }, [editing?.file]);

  useEffect(() => {
    onEditingParagraph?.(editing ? { file: editing.file, offset: editing.start } : null);
  }, [editing?.file, editing?.start]);

  const visibleTimer = useRef<number | undefined>(undefined);
  function reportVisiblePage(container: HTMLDivElement) {
    if (!onVisiblePage) return;
    window.clearTimeout(visibleTimer.current);
    visibleTimer.current = window.setTimeout(() => {
      const middle = container.scrollTop + container.clientHeight / 2;
      let page = 0;
      pageRefs.current.forEach((el, i) => {
        if (el && el.offsetTop <= middle) page = i;
      });
      onVisiblePage(page);
    }, 150);
  }

  const scrollRef = useRef(scrollTo);
  scrollRef.current = scrollTo;
  useEffect(() => {
    if (!refresh) return;
    compile()
      .then((report) => {
        setPages((current) => applyReport(current, report));
        setErrors(report.warnings);
        onReport(report);
        // z. B. neues Kapitel: erst nach dem Neusetzen gibt es seine Seite.
        const target = scrollRef.current;
        if (target)
          firstPage(target.file)
            .then((page) => page !== null && requestAnimationFrame(() => pageRefs.current[page]?.scrollIntoView?.({ behavior: "smooth" })))
            .catch(() => {});
      })
      .catch((e) => setErrors(errorMessages(e)));
  }, [refresh]);

  useEffect(() => {
    if (!scrollTo) return;
    firstPage(scrollTo.file)
      .then((page) => page !== null && pageRefs.current[page]?.scrollIntoView?.({ behavior: "smooth" }))
      .catch(() => {});
  }, [scrollTo]);

  function scheduleDraft(text: string) {
    const hit = editing;
    if (!hit) return;
    onSaving();
    window.clearTimeout(draftTimer.current);
    draftTimer.current = window.setTimeout(() => {
      saveDraft(hit.file, hit.start, hit.end, text).then(onSaved).catch((e) => setErrors(errorMessages(e)));
    }, DRAFT_DELAY_MS);
  }

  /** Ersetzt den Absatz, setzt neu, speichert. Gibt `false` bei Fehlern zurück. */
  async function replace(hit: ParagraphHit, text: string): Promise<boolean> {
    window.clearTimeout(draftTimer.current);
    try {
      const { report, save } = await replaceText(hit.file, hit.start, hit.end, text);
      setPages((current) => applyReport(current, report));
      setErrors(report.warnings);
      onReport(report);
      onSaved(save);
      return true;
    } catch (e) {
      // Bei Fehlern (z. B. Syntaxfehler) bleibt das Feld mit dem Text offen.
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
      if (fromSplit.current && hit.start === hit.end && text.trim() === "") {
        // Leer gebliebenen neuen Absatz samt der eingefügten Leerzeile wieder entfernen.
        if (await replace({ ...hit, start: hit.start - 2 }, "")) setEditing(null);
      } else if (text === hit.text || (await replace(hit, text))) {
        setEditing(null);
      }
      fromSplit.current = false;
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
      // Enter in einem leeren Absatz erzeugt keine weiteren Leerzeilen.
      if (before === "" && after === "") return;
      if (!(await replace(hit, `${before}\n\n${after}`))) return;
      setEditing(await paragraphAtOffset(hit.file, hit.start + utf8Length(before) + 2));
      fromSplit.current = after === "";
    } finally {
      busy.current = false;
    }
  }

  /** ⌘-Klick im Feld: zum definierenden Absatz springen (z. B. Label in einem anderen Kapitel). */
  async function goToDefinition(definition: Definition) {
    if (editing && currentText.current) await commit(currentText.current());
    const target = definition.file?.endsWith(".typ") ? await paragraphAtOffset(definition.file, definition.offset) : null;
    if (target) {
      setEditing(target);
      setHint(null);
      pageRefs.current[target.rects[0][0]]?.scrollIntoView?.({ behavior: "smooth", block: "center" });
    } else {
      setHint(`Definiert in ${definition.file ?? definition.absolute}, Zeile ${definition.line + 1}.`);
    }
  }

  async function handleClick(page: number, event: React.MouseEvent<HTMLDivElement>) {
    const box = event.currentTarget.getBoundingClientRect();
    const x = (event.clientX - box.left) / scale;
    const y = (event.clientY - box.top) / scale;
    if (busy.current) return;
    if (onPick) {
      const hit = await paragraphAt(page, x, y).catch(() => null);
      if (hit) onPick(hit);
      return;
    }
    if (editing && currentText.current) await commit(currentText.current());
    try {
      const hit = await paragraphAt(page, x, y);
      setEditing(hit);
      setHint(hit ? null : TEMPLATE_HINT);
    } catch (e) {
      setErrors(errorMessages(e));
    }
  }

  return (
    <div className="page-editor">
      {hint && (
        <div className="hint" onClick={() => setHint(null)}>
          {hint}
        </div>
      )}
      {errors.length > 0 && (
        <div className="errors" role="alert">
          {errors.map((e, i) => (
            <div key={i}>{e}</div>
          ))}
        </div>
      )}
      <div className="pages" onScroll={(e) => reportVisiblePage(e.currentTarget)}>
        {pages.map((page, i) => (
          <div
            key={i}
            ref={(el) => {
              pageRefs.current[i] = el;
            }}
            className="page"
            style={{ width: page.width * scale, height: page.height * scale }}
            onClick={(e) => handleClick(i, e)}
          >
            {page.url && <img className="page-image" src={page.url} alt={`Seite ${page.label || i + 1}`} draggable={false} />}
            {marks
              .filter((m) => m.page === i)
              .map((m, k) => (
                <div
                  key={`mark-${k}`}
                  className={`mark ${m.schwere}`}
                  title={m.texts.join("\n")}
                  style={{ left: (m.rect.x - 12) * scale, top: m.rect.y * scale, height: Math.max(m.rect.height, 10) * scale }}
                />
              ))}
            {highlight?.rects
              .filter(([p]) => p === i)
              .map(([, rect], k) => (
                <div
                  key={k}
                  className="reveal"
                  style={{
                    left: rect.x * scale - 4,
                    top: rect.y * scale - 3,
                    width: rect.width * scale + 8,
                    height: rect.height * scale + 6,
                  }}
                />
              ))}
            {editing?.rects
              .filter(([p]) => p === i)
              .slice(0, 1)
              .map(([, rect]) => (
                <div
                  key={`${editing.file}:${editing.start}`}
                  className="editor-overlay"
                  style={{
                    // Spaltenbreite wie im Satz; erste Grundlinie auf die gesetzte Grundlinie legen.
                    left: editing.column_x * scale,
                    width: editing.column_width * scale,
                    top: (rect.y - (editing.line_height - rect.height) / 2) * scale,
                    minHeight: Math.max(rect.height, editing.line_height) * scale,
                  }}
                  onClick={(e) => e.stopPropagation()}
                >
                  <ParagraphEditor
                    file={editing.file}
                    start={editing.start}
                    end={editing.end}
                    text={editing.text}
                    fontSize={editing.font_size * scale}
                    lineHeight={editing.line_height * scale}
                    indent={editing.first_line_indent * scale}
                    bold={editing.text.trimStart().startsWith("=")}
                    onSplit={split}
                    onCommit={commit}
                    onChange={scheduleDraft}
                    onDefinition={goToDefinition}
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
