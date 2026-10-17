// Typen und Aufrufe des Rust-Moduls `satz` (src-tauri/src/satz).
import { invoke } from "@tauri-apps/api/core";

export interface CompileReport {
  page_count: number;
  changed_pages: number[];
  svgs: string[];
  compile_ms: number;
  svg_ms: number;
  warnings: string[];
  page_sizes: [number, number][];
  page_labels: string[];
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ParagraphHit {
  file: string;
  start: number;
  end: number;
  text: string;
  rects: [number, Rect][];
  /** Typografie im Satz, in pt. */
  font_size: number;
  line_height: number;
  /** Textspalte der Arbeit, in pt. */
  column_x: number;
  column_width: number;
  /** Einzug der ersten Zeile, in pt. */
  first_line_indent: number;
}

export const openExample = () => invoke<CompileReport>("satz_open_example");

export const paragraphAt = (page: number, x: number, y: number) =>
  invoke<ParagraphHit | null>("satz_paragraph_at", { page, x, y });

export const paragraphAtOffset = (file: string, offset: number) =>
  invoke<ParagraphHit | null>("satz_paragraph_at_offset", { file, offset });

/** Länge in UTF-8-Bytes (Rust rechnet in Bytes, JavaScript in UTF-16). */
export const utf8Length = (text: string) => new TextEncoder().encode(text).length;

export const replaceText = (file: string, start: number, end: number, text: string) =>
  invoke<CompileReport>("satz_replace", { file, start, end, text });

/** Familie, unter der die Satzschrift im Editierfeld verfügbar ist. */
export const EDITOR_FONT = "Satzschrift";

/** Lädt die Schrift, mit der Typst setzt (New Computer Modern), in die Oberfläche. */
export async function loadEditorFont() {
  for (const bold of [false, true]) {
    const data = await invoke<ArrayBuffer>("satz_font", { family: "New Computer Modern", bold });
    const face = new FontFace(EDITOR_FONT, data, { weight: bold ? "700" : "400" });
    document.fonts.add(await face.load());
  }
}

/** Fehler aus Rust kommen als Liste von Meldungen. */
export function errorMessages(error: unknown): string[] {
  return Array.isArray(error) ? error.map(String) : [String(error)];
}

/** Protokolliert in beispiel-arbeit/.schreibassistent/editor-test.log (P2-Auswertung). */
export function debugLog(message: string) {
  invoke("satz_debug_log", { message }).catch(() => {});
}

// Unbehandelte Fehler der Oberfläche ebenfalls protokollieren.
if (typeof window !== "undefined") {
  window.addEventListener("error", (e) => debugLog(`window.error ${e.message}`));
  window.addEventListener("unhandledrejection", (e) => debugLog(`unhandledrejection ${String(e.reason)}`));
}

// Jeder Mausklick im Fenster, um zu sehen, ob Klicks überhaupt ankommen (P2-Diagnose).
if (typeof document !== "undefined") {
  document.addEventListener(
    "pointerdown",
    (e) => {
      const target = e.target as Element | null;
      debugLog(`pointerdown (${e.clientX}, ${e.clientY}) auf <${target?.tagName?.toLowerCase()} class="${target?.getAttribute?.("class") ?? ""}">`);
    },
    true,
  );
}
