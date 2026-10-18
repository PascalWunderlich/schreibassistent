/** Lose Kopplung zwischen Befehlspalette und Panels: Ereignisse am Fenster statt Zustand durch die ganze App. */
export const TAB_EVENT = "sa:tab";
export const INSERT_EVENT = "sa:insert";

export type RightTab = "pruefen" | "assistent";
export type InsertKind = "abbildung" | "tabelle" | "diagramm" | "listing" | "zitat" | "verweis";

export const showTab = (tab: RightTab) => window.dispatchEvent(new CustomEvent<RightTab>(TAB_EVENT, { detail: tab }));
export const openInsert = (kind: InsertKind) => window.dispatchEvent(new CustomEvent<InsertKind>(INSERT_EVENT, { detail: kind }));
