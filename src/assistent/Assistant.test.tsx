import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Assistant } from "./Assistant";

const SETTINGS = { aufgaben: { frage: { anbieter: "ollama", modell: "qwen3:8b" } }, openai_url: "", ollama_url: "" };
const invoke = vi.fn(async (command: string, _args?: unknown): Promise<unknown> => {
  if (command === "ki_einstellungen") return { settings: SETTINGS, schluessel: {}, ollama_modelle: ["qwen3:8b"] };
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
});
