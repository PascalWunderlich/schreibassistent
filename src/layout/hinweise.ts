import { Hinweis } from "../editor/api";

/** Art eines Hinweises: bestimmt Etikett und Farbe (`hint-<art>`). */
export type Art = "form" | "stil" | "quelle" | "sprache";

export const ART_LABEL: Record<Art, string> = { form: "Form", stil: "Stil", quelle: "Quelle", sprache: "Sprache" };

export function artOf(h: Hinweis): Art {
  if (h.quelle === "ltex" || h.regel_id === "rechtschreibung") return "sprache";
  if (h.quelle === "vale" || h.regel_id.startsWith("stil")) return "stil";
  if (h.regel_id === "zitierstil" || h.regel_id === "quellen") return "quelle";
  return "form";
}

/** Angaben der `.bib` („Zugriffsdatum fehlt“): wichtig, aber nicht im Text behebbar, deshalb eingeklappt. */
export const isMetadata = (h: Hinweis) => !!h.datei?.endsWith(".bib");

const RANK = { fehler: 0, warnung: 1, hinweis: 2 } as const;

/** Fehler vor Warnungen vor Hinweisen; innerhalb gleicher Schwere nach Datei und Zeile. */
export function sortHints(list: Hinweis[]): Hinweis[] {
  return [...list].sort(
    (a, b) => RANK[a.schwere] - RANK[b.schwere] || (a.datei ?? "").localeCompare(b.datei ?? "") || (a.zeile ?? 0) - (b.zeile ?? 0),
  );
}

/** Ort für die Karte: „01-einleitung, Z. 12“. */
export function placeOf(h: Hinweis): string {
  if (!h.datei) return "Gesamtdokument";
  const name = h.datei.split("/").pop()!.replace(/\.(typ|md|bib)$/, "");
  return h.zeile ? `${name}, Z. ${h.zeile}` : name;
}
