import { describe, expect, it } from "vitest";
import { Hinweis } from "../editor/api";
import { artOf, isMetadata, placeOf, sortHints } from "./hinweise";

const h = (o: Partial<Hinweis>): Hinweis => ({
  regel_id: "gliederung", datei: null, zeile: null, offset: null, text: "t", schwere: "hinweis", fundstelle: "", quelle: "regel", stelle: null, ...o,
});

describe("hinweise", () => {
  it("ordnet Hinweise einer Art zu", () => {
    expect(artOf(h({ quelle: "ltex", regel_id: "rechtschreibung" }))).toBe("sprache");
    expect(artOf(h({ quelle: "vale", regel_id: "stil-man" }))).toBe("stil");
    expect(artOf(h({ regel_id: "quellen" }))).toBe("quelle");
    expect(artOf(h({ regel_id: "umfang-praxisarbeit" }))).toBe("form");
  });

  it("erkennt Metadaten der .bib", () => {
    expect(isMetadata(h({ datei: "System/quellen.bib" }))).toBe(true);
    expect(isMetadata(h({ datei: "Arbeit/kapitel/01.typ" }))).toBe(false);
  });

  it("sortiert Fehler vor Warnungen vor Hinweisen", () => {
    const sorted = sortHints([h({ schwere: "hinweis" }), h({ schwere: "fehler" }), h({ schwere: "warnung" })]);
    expect(sorted.map((x) => x.schwere)).toEqual(["fehler", "warnung", "hinweis"]);
  });

  it("nennt den Ort lesbar", () => {
    expect(placeOf(h({}))).toBe("Gesamtdokument");
    expect(placeOf(h({ datei: "Arbeit/kapitel/01-einleitung.typ", zeile: 12 }))).toBe("01-einleitung, Z. 12");
  });
});
