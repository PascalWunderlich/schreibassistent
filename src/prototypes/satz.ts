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
}

export const openExample = () => invoke<CompileReport>("satz_open_example");

export const paragraphAt = (page: number, x: number, y: number) =>
  invoke<ParagraphHit | null>("satz_paragraph_at", { page, x, y });

export const replaceText = (file: string, start: number, end: number, text: string) =>
  invoke<CompileReport>("satz_replace", { file, start, end, text });

/** Fehler aus Rust kommen als Liste von Meldungen. */
export function errorMessages(error: unknown): string[] {
  return Array.isArray(error) ? error.map(String) : [String(error)];
}
