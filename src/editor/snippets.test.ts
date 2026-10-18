import { describe, expect, it } from "vitest";
import { chart, cite, figure, heading, slug, table } from "./snippets";

describe("snippets", () => {
  it("leitet Labels aus der Beschriftung ab", () => {
    expect(slug("Aufbau des Systems für Größe")).toBe("aufbau-des-systems-fuer-groesse");
    expect(slug("!!!")).toBe("ohne-titel");
  });

  it("markiert die erste auszufüllende Stelle", () => {
    const s = heading(2);
    expect(s.text.slice(...s.select!)).toBe("Überschrift");
  });

  it("zitiert mit und ohne Seite", () => {
    expect(cite("gebhard2021").text).toBe("@gebhard2021");
    expect(cite("gebhard2021", "14").text).toBe("@gebhard2021[S. 14]");
  });

  it("setzt Abbildung und Tabelle mit Beschriftung und Label", () => {
    expect(figure("../abbildungen/a.svg", "Aufbau").text).toContain("<abb:aufbau>");
    const t = table(2, 3, "Anforderungen").text;
    expect(t).toContain("columns: 3");
    expect(t).toContain("<tab:anforderungen>");
    expect(t.match(/\[…\]/g)).toHaveLength(6);
  });

  it("erzeugt Diagramme mit Label", () => {
    expect(chart("balken", "Umsatz").text).toContain("<abb:umsatz>");
    expect(chart("linie", "Verlauf").text).toContain("curve.line");
  });
});

import { relativePath } from "./inserter";
describe("relativePath", () => {
  it("bildet Pfade relativ zur Kapiteldatei", () => {
    expect(relativePath("Arbeit/kapitel/01.typ", "Arbeit/abbildungen/a.svg")).toBe("../abbildungen/a.svg");
    expect(relativePath("Arbeit/main.typ", "Arbeit/abbildungen/a.svg")).toBe("abbildungen/a.svg");
  });
});
