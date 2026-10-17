// Typen und Aufrufe der Rust-Module `projekt` und `satz`.
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
  /** Seiten des Inhaltsteils (Seite „1“ bis vor das Quellverzeichnis). */
  content_pages: number | null;
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

export interface SaveInfo {
  /** Zeitpunkt eines neuen Autosave-Commits (Sekunden seit 1970). */
  autosaved: number | null;
  git_error: string | null;
}

export interface ProjectInfo {
  root: string;
  name: string;
  git: boolean;
  last_autosave: number | null;
  report: CompileReport;
  lsp_error: string | null;
  /** Fehler beim Setzen; die Arbeit ist trotzdem offen. */
  compile_errors: string[];
  /** Ort des ersten Fehlers: Datei (relativ) und Byte-Position. */
  error_location: [string, number] | null;
}

export interface TreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  writable: boolean;
  children: TreeNode[];
}

// ---------- projekt ----------
export const lastProject = () => invoke<string | null>("projekt_last");
export const openProject = (path: string) => invoke<ProjectInfo>("projekt_open", { path });
export const initGit = () => invoke<void>("projekt_git_init");
export const projectTree = () => invoke<TreeNode[]>("projekt_tree");
export const readFile = (path: string) => invoke<string>("projekt_read", { path });
export const exportDir = () => invoke<string>("projekt_export_dir");
export const newChapter = (title: string) => invoke<string>("projekt_new_chapter", { title });
export const newNote = (name: string) => invoke<string>("projekt_new_note", { name });
export const writeFile = (path: string, text: string) => invoke<SaveInfo>("projekt_write", { path, text });

// ---------- satz ----------
export const allPages = () => invoke<CompileReport>("satz_all_pages");
export const compile = () => invoke<CompileReport>("satz_compile");
export const exportPdf = (path: string) => invoke<void>("satz_export_pdf", { path });
export const firstPage = (file: string) => invoke<number | null>("satz_first_page", { file });

export const paragraphAt = (page: number, x: number, y: number) =>
  invoke<ParagraphHit | null>("satz_paragraph_at", { page, x, y });

export const paragraphAtOffset = (file: string, offset: number) =>
  invoke<ParagraphHit | null>("satz_paragraph_at_offset", { file, offset });

export const replaceText = (file: string, start: number, end: number, text: string) =>
  invoke<{ report: CompileReport; save: SaveInfo }>("satz_replace", { file, start, end, text });

/** Entwurf während des Tippens speichern (ohne neu zu setzen). */
export const saveDraft = (file: string, start: number, end: number, text: string) =>
  invoke<SaveInfo>("satz_draft", { file, start, end, text });

/** Länge in UTF-8-Bytes (Rust rechnet in Bytes, JavaScript in UTF-16). */
export const utf8Length = (text: string) => new TextEncoder().encode(text).length;

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

/** Fehler aus Rust kommen als Text oder als Liste von Meldungen. */
export function errorMessages(error: unknown): string[] {
  return Array.isArray(error) ? error.map(String) : [String(error)];
}

/** Protokolliert in `<Vault>/.schreibassistent/app.log` (nicht im Git). */
export function log(message: string) {
  invoke("projekt_log", { message }).catch(() => {});
}

// Unbehandelte Fehler der Oberfläche protokollieren (Fehlersuche ohne Entwicklerwerkzeuge).
if (typeof window !== "undefined") {
  window.addEventListener("error", (e) => log(`Fehler in der Oberfläche: ${e.message} (${e.filename}:${e.lineno})`));
  window.addEventListener("unhandledrejection", (e) => log(`Unbehandelte Ablehnung: ${String(e.reason)}`));
}

// ---------- lsp (Tinymist im Feld) ----------
/** Absatz im Feld: `start..end` im Quelltext von `file` wird durch `text` ersetzt. */
export interface Field {
  file: string;
  start: number;
  end: number;
  text: string;
  /** Quelltextmodus: `text` ist die ganze Datei. */
  whole_file: boolean;
}

export interface Completion {
  label: string;
  detail: string | null;
  kind: number | null;
  /** Bereich im Feld (UTF-16) und Ersatztext. */
  from: number;
  to: number;
  insert: string;
}

export interface FieldDiagnostic {
  from: number;
  to: number;
  severity: "error" | "warning" | "info";
  message: string;
}

export interface Definition {
  file: string | null;
  absolute: string;
  offset: number;
  line: number;
}

export const lspCompletion = (field: Field, cursor: number) => invoke<Completion[]>("lsp_completion", { field, cursor });
export const lspHover = (field: Field, cursor: number) => invoke<string | null>("lsp_hover", { field, cursor });
export const lspDefinition = (field: Field, cursor: number) => invoke<Definition | null>("lsp_definition", { field, cursor });
export const lspFieldDiagnostics = (field: Field) => invoke<FieldDiagnostic[]>("lsp_field_diagnostics", { field });
export const lspErrorCount = () => invoke<number>("lsp_error_count");

// ---------- quellen und index ----------
export interface Source {
  citekey: string;
  kind: string;
  authors: string[];
  title: string | null;
  year: string | null;
  container: string | null;
  publisher: string | null;
  url: string | null;
  doi: string | null;
  file: string | null;
  /** Kapitel, die die Quelle zitieren (leer = ungenutzt). */
  chapters: string[];
}

export interface SourcesOverview {
  sources: Source[];
  error: string | null;
}

export interface ChapterInfo {
  file: string;
  title: string | null;
  first_page: number | null;
}

export interface ChapterView {
  file: string;
  title: string | null;
  /** Citekey → Ergänzungen wie „S. 14“. */
  quellen: Record<string, string[]>;
  notizen_direkt: string[];
  /** Notiz → Quellen, über die sie zum Kapitel gehört. */
  notizen_ueber: Record<string, string[]>;
  wissen_direkt: string[];
  wissen_ueber: Record<string, string[]>;
}

export const indexChapters = () => invoke<ChapterInfo[]>("index_chapters");
export const indexChapter = (file: string) => invoke<ChapterView | null>("index_chapter", { file });
export const indexSources = () => invoke<SourcesOverview>("index_sources");

/** Kurzform für die Anzeige: „Sutton 2018“. */
export function sourceLabel(source: Source): string {
  const surname = source.authors[0]?.split(",")[0]?.replace(/[{}]/g, "").trim();
  return [surname, source.year].filter(Boolean).join(" ") || source.citekey;
}
