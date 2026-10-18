import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Assistant } from "./Assistant";

const SETTINGS = { aufgaben: { frage: { anbieter: "ollama", modell: "qwen3:8b" } }, openai_url: "", ollama_url: "" };
const invoke = vi.fn(async (command: string, _args?: unknown): Promise<unknown> => {
  if (command === "ki_einstellungen") return { settings: SETTINGS, schluessel: {}, ollama_modelle: ["qwen3:8b"] };
  if (command === "ki_belegbedarf")
    return {
      hinweise: [{ nr: 3, start: 120, satz: "Laut einer Umfrage nutzen 63 % Microservices.", grund: "Zahl ohne Beleg" }],
      entfernt: 0,
      modell: "deepseek-chat",
      datei: "Arbeit/kapitel/07-ki-test.typ",
      sonstiges: [],
    };
  if (command === "ki_frage")
    return { text: "- Wer ist gemeint?", anbieter: "ollama", modell: "qwen3:8b", tokens: { ein: 12, aus: 5 }, protokoll_id: "1" };
  return null;
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: (c: string, a?: unknown) => invoke(c, a) }));

describe("Assistant", () => {
  it("schickt den Absatz als Kontext und zeigt die Antwort nur als Text", async () => {
    const onAnswer = vi.fn();
    const { container } = render(
      <Assistant
        paragraph={{ file: "Arbeit/kapitel/01-einleitung.typ", offset: 42 }}
        chapter="Arbeit/kapitel/01-einleitung.typ"
        onAnswer={onAnswer}
        onExported={vi.fn()}
      />,
    );
    fireEvent.change(screen.getByPlaceholderText(/Frage stellen/), { target: { value: "Ist das klar?" } });
    fireEvent.click(screen.getByText("Fragen"));
    await waitFor(() => expect(screen.getByText("- Wer ist gemeint?")).toBeTruthy());
    expect(invoke).toHaveBeenCalledWith("ki_frage", {
      frage: "Ist das klar?",
      kontext: { art: "absatz", datei: "Arbeit/kapitel/01-einleitung.typ", offset: 42 },
    });
    expect(onAnswer).toHaveBeenCalled();
    // Kein Einfügen oder Übernehmen (Leitplanke 1).
    const labels = [...container.querySelectorAll("button")].map((b) => b.textContent ?? "");
    expect(labels.some((l) => /einfügen|übernehmen/i.test(l))).toBe(false);
  });

  it("zeigt Belegbedarf mit Sprung zum Satz", async () => {
    cleanup();
    const onJump = vi.fn();
    render(
      <Assistant
        paragraph={{ file: "Arbeit/kapitel/07-ki-test.typ", offset: 100 }}
        chapter="Arbeit/kapitel/07-ki-test.typ"
        onAnswer={vi.fn()}
        onExported={vi.fn()}
        onJump={onJump}
      />,
    );
    fireEvent.click(screen.getByText("Belegbedarf"));
    await waitFor(() => expect(screen.getByText("Zahl ohne Beleg")).toBeTruthy());
    expect(invoke).toHaveBeenCalledWith("ki_belegbedarf", { datei: "Arbeit/kapitel/07-ki-test.typ", offset: 100 });
    fireEvent.click(screen.getByText(/63 % Microservices/));
    expect(onJump).toHaveBeenCalledWith("Arbeit/kapitel/07-ki-test.typ", 120);
  });
});
