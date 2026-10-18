import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { StatusBar, pageStatus } from "./StatusBar";

const invoke = vi.fn(async (command: string): Promise<unknown> => {
  if (command === "protokoll_liste")
    return [
      { id: "1", zeit: "2026-10-18T10:00:00Z", anbieter: "ollama", modell: "qwen3:8b", aufgabe: "frage", kontext: { datei: "Arbeit/kapitel/01.typ", absatz: 2 }, anfrage: "Ist das klar?", tokens: { ein: 12, aus: 5 }, ergebnis: "ok" },
      { id: "2", zeit: "2026-10-18T11:00:00Z", anbieter: "", modell: "", aufgabe: "schreiben", kontext: { datei: null, absatz: null }, anfrage: "Schreibversuch", tokens: { ein: null, aus: null }, ergebnis: "abgelehnt" },
    ];
  return null;
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: (c: string) => invoke(c) }));

afterEach(cleanup);

const base = { range: [20, 22] as [number, number], openHints: 0, contentPages: 21, lastAutosave: null, git: true, gitError: null, errorCount: 0, spellchecking: false, protokollVersion: 0 };

describe("pageStatus", () => {
  it("nennt Abweichungen mit Wort", () => {
    expect(pageStatus(14, [20, 22])).toEqual({ text: "Seite 14 / 20–22", verdict: "zu kurz" });
    expect(pageStatus(23, [20, 22]).verdict).toBe("zu lang");
    expect(pageStatus(21, [20, 22]).verdict).toBe("ok");
    expect(pageStatus(null, null).text).toBe("Seiten: –");
  });
});

describe("StatusBar", () => {
  it("zeigt Zahl der Protokolleinträge und öffnet die Liste", async () => {
    render(<StatusBar {...base} />);
    const button = await screen.findByRole("button", { name: /KI-Protokoll: 2 Einträge/ });
    HTMLDialogElement.prototype.showModal ??= function () { this.setAttribute("open", ""); };
    fireEvent.click(button);
    await waitFor(() => expect(screen.getByText("Ist das klar?")).toBeTruthy());
    expect(screen.getByText("abgelehnt")).toBeTruthy();
  });

  it("meldet zu lange Arbeit und startende Rechtschreibung", () => {
    render(<StatusBar {...base} contentPages={23} spellchecking />);
    expect(screen.getByText(/zu lang/)).toBeTruthy();
    expect(screen.getByText("Rechtschreibung startet …")).toBeTruthy();
  });
});
