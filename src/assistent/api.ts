// Typen und Aufrufe der Rust-Module `ki` und `protokoll`.
import { invoke } from "@tauri-apps/api/core";

export type Anbieter = "ollama" | "openrouter" | "deepseek" | "anthropic" | "openai";
export type Aufgabe = "frage" | "belegpruefung" | "feedback" | "ingest" | "regelprofil";

export const ANBIETER: { id: Anbieter; name: string; vorschlag: string }[] = [
  { id: "ollama", name: "Ollama (lokal)", vorschlag: "qwen3:8b" },
  { id: "openrouter", name: "OpenRouter", vorschlag: "anthropic/claude-sonnet-5.5" },
  { id: "deepseek", name: "DeepSeek", vorschlag: "deepseek-chat" },
  { id: "anthropic", name: "Anthropic", vorschlag: "claude-sonnet-5-5" },
  { id: "openai", name: "OpenAI-kompatibel", vorschlag: "" },
];

export const AUFGABEN: { id: Aufgabe; name: string }[] = [
  { id: "frage", name: "Fragen im Assistenten" },
  { id: "belegpruefung", name: "Belegprüfung (P8)" },
  { id: "feedback", name: "Feedback (P8)" },
  { id: "ingest", name: "Quellen erschließen (P7)" },
  { id: "regelprofil", name: "Regelprofil aus PDF (P8)" },
];

export interface Wahl {
  anbieter: Anbieter;
  modell: string;
}

export interface KiSettings {
  aufgaben: Record<Aufgabe, Wahl>;
  openai_url: string;
  ollama_url: string;
}

export interface KiInfo {
  settings: KiSettings;
  /** Je Anbieter: Schlüssel gesetzt? */
  schluessel: Record<string, boolean>;
  ollama_modelle: string[];
}

export type KontextWahl =
  | { art: "keiner" }
  | { art: "absatz"; datei: string; offset: number }
  | { art: "kapitel"; datei: string };

export interface FrageErgebnis {
  text: string;
  anbieter: string;
  modell: string;
  tokens: { ein: number | null; aus: number | null };
  protokoll_id: string;
}

export interface ChainCheck {
  eintraege: number;
  fehler: [number, string] | null;
}

export const kiEinstellungen = () => invoke<KiInfo>("ki_einstellungen");
export const kiEinstellungenSpeichern = (ki: KiSettings) => invoke<void>("ki_einstellungen_speichern", { ki });
export const kiSchluesselSetzen = (anbieter: Anbieter, schluessel: string) =>
  invoke<void>("ki_schluessel_setzen", { anbieter, schluessel });
export const kiSchluesselLoeschen = (anbieter: Anbieter) => invoke<void>("ki_schluessel_loeschen", { anbieter });
export const kiFrage = (frage: string, kontext: KontextWahl) => invoke<FrageErgebnis>("ki_frage", { frage, kontext });
export const protokollPruefen = () => invoke<ChainCheck>("protokoll_pruefen");
export const protokollExport = () => invoke<string>("protokoll_export");

// ---------- KI-Prüfungen (P8) ----------

export interface Ergebnis<T> {
  hinweise: T[];
  entfernt: number;
  modell: string;
  datei: string;
  sonstiges: string[];
}

export interface BelegHinweis {
  nr: number;
  start: number;
  satz: string;
  grund: string;
}

export interface Fund {
  citekey: string;
  label: string;
  pdf_seite: number;
  text: string;
  link: string | null;
}

export interface AussageHinweis {
  start: number;
  satz: string;
  citekey: string;
  ergaenzung: string | null;
  abschnitte: Fund[];
  einschaetzung: string | null;
  seite: string | null;
  begruendung: string;
  problem: string | null;
}

export interface AbsatzHinweis {
  absatz: number | null;
  start: number | null;
  regel: string | null;
  text: string;
}

export interface Hierzu {
  quellen: [string, string | null][];
  seiten: string[];
  fundstellen: Fund[];
  absatz: string;
}

export interface Vorschlag {
  regel: Record<string, unknown>;
  yaml: string;
  zitat: string;
  zitat_ok: boolean;
  problem: string | null;
}

export const kiBelegbedarf = (datei: string, offset: number | null) =>
  invoke<Ergebnis<BelegHinweis>>("ki_belegbedarf", { datei, offset });
export const kiQuelleAussage = (datei: string, offset: number) =>
  invoke<Ergebnis<AussageHinweis>>("ki_quelle_aussage", { datei, offset });
export const kiRoterFaden = (datei: string) => invoke<Ergebnis<AbsatzHinweis>>("ki_roter_faden", { datei });
export const kiUrteilsregeln = (datei: string) => invoke<Ergebnis<AbsatzHinweis>>("ki_urteilsregeln", { datei });
export const kiHierzu = (datei: string, offset: number) => invoke<Hierzu>("ki_hierzu", { datei, offset });
export const kiHierzuEinordnen = (datei: string, offset: number) =>
  invoke<{ text: string; entfernt: number; modell: string }>("ki_hierzu_einordnen", { datei, offset });
export const kiRegelprofilAusPdf = (pdf: string) => invoke<{ regeln: Vorschlag[]; modell: string }>("ki_regelprofil_aus_pdf", { pdf });
export const kiRegelprofilSpeichern = (name: string, quelle: string, regeln: Record<string, unknown>[], ziel: string) =>
  invoke<string>("ki_regelprofil_speichern", { name, quelle, regeln, ziel });

export interface ProtokollEintrag {
  id: string;
  zeit: string;
  anbieter: string;
  modell: string;
  aufgabe: string;
  kontext: { datei: string | null; absatz: number | null };
  anfrage: string;
  tokens: { ein: number | null; aus: number | null };
  /** `ok`, `fehler` oder `abgelehnt`. */
  ergebnis: string;
}
export const protokollListe = () => invoke<ProtokollEintrag[]>("protokoll_liste");
