import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PageEditor } from "./PageEditor";

const invoke = vi.fn(async (command: string, _args?: unknown): Promise<unknown> => {
  switch (command) {
    case "satz_open_example":
      return {
        page_count: 1,
        changed_pages: [0],
        svgs: ['<svg viewBox="0 0 100 100"></svg>'],
        compile_ms: 1,
        svg_ms: 1,
        warnings: [],
        page_sizes: [[100, 100]],
        page_labels: ["1"],
      };
    case "satz_paragraph_at":
      return {
        file: "Arbeit/kapitel/01-einleitung.typ",
        start: 10,
        end: 20,
        text: "Ein Absatz.",
        rects: [[0, { x: 10, y: 10, width: 50, height: 12 }]],
      };
    default:
      throw [`unerwartet: ${command}`];
  }
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: (c: string, a?: unknown) => invoke(c, a) }));

describe("PageEditor", () => {
  it("öffnet nach einem Klick das Feld mit dem Absatz", async () => {
    const { container } = render(<PageEditor />);
    await waitFor(() => expect(container.querySelector(".page")).not.toBeNull());
    fireEvent.click(container.querySelector(".page")!, { clientX: 20, clientY: 20 });
    await waitFor(() => expect(container.querySelector(".editor-overlay")).not.toBeNull());
    expect(invoke).toHaveBeenCalledWith("satz_paragraph_at", expect.objectContaining({ page: 0 }));
    expect(screen.getByText("Ein Absatz.")).toBeDefined();
  });
});
