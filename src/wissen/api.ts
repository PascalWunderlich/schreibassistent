// Typen und Aufrufe des Rust-Moduls `ingest`.
import { invoke } from "@tauri-apps/api/core";

export interface IngestStatus {
  pdf: string | null;
  zotero: boolean;
  eingelesen: boolean;
  seite: string | null;
  aktuell: boolean;
  grund: string | null;
  stellen: string | null;
}

export interface Schaetzung {
  citekey: string;
  seiten: number;
  abschnitte: number;
  abschnitte_gewaehlt: number;
  tokens_ein: number;
  tokens_aus: number;
  anbieter: string;
  modell: string;
  seitenzahlen: string;
  markierungen: number;
}

export interface IngestErgebnis {
  seite: string;
  bestaetigt: number;
  gesamt: number;
  modell: string;
}

export interface KonzeptErgebnis {
  seiten: string[];
  warnungen: number;
  modell: string;
}

export const ingestStatus = (citekey: string) => invoke<IngestStatus>("ingest_status", { citekey });
export const ingestSchaetzen = (citekey: string) => invoke<Schaetzung>("ingest_schaetzen", { citekey });
export const ingestErzeugen = (citekey: string, neu: boolean) => invoke<IngestErgebnis>("ingest_erzeugen", { citekey, neu });
export const ingestKonzepte = () => invoke<KonzeptErgebnis>("ingest_konzepte");
