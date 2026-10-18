// Aufrufe für den Ersteinstieg.
import { invoke } from "@tauri-apps/api/core";

export const ersteinstiegOffen = () => invoke<boolean>("app_ersteinstieg");
export const ersteinstiegBeenden = () => invoke<void>("app_ersteinstieg_beenden");
export const zoteroLaeuft = () => invoke<boolean>("quellen_zotero_laeuft");
export const bibUebernehmen = (path: string) => invoke<number>("quellen_bib_uebernehmen", { path });
