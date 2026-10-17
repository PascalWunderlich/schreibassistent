// Aufrufe des Rust-Moduls `lsp` (Tinymist mit Vorschau), Prototyp B.
import { invoke } from "@tauri-apps/api/core";

export interface SourceFile {
  path: string;
  text: string;
}

export interface PreviewSession {
  preview_url: string;
  files: SourceFile[];
}

export interface LspNotification {
  method: string;
  params: unknown;
}

// Nur einmal starten, auch wenn React den Effekt im Entwicklungsmodus doppelt ausführt.
let opening: Promise<PreviewSession> | null = null;
export const openPreview = () => (opening ??= invoke<PreviewSession>("lsp_open_example"));

export const changeFile = (path: string, text: string) => invoke<void>("lsp_change", { path, text });

export const scrollPreview = (path: string, line: number, character: number) =>
  invoke<void>("lsp_scroll_preview", { path, line, character });

export const relativePath = (absolute: string) => invoke<string | null>("lsp_relative_path", { absolute });

/** Position aus einer Tinymist-Meldung lesen: [Zeile, Zeichen] oder {line, character}. */
export function readPosition(value: unknown): { line: number; character: number } | null {
  if (Array.isArray(value) && value.length >= 2) return { line: Number(value[0]), character: Number(value[1]) };
  if (value && typeof value === "object" && "line" in value) {
    const v = value as { line: number; character?: number };
    return { line: Number(v.line), character: Number(v.character ?? 0) };
  }
  return null;
}
