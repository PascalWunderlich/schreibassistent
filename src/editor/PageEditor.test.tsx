import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PageEditor } from "./PageEditor";

const report = {
  page_count: 1,
  changed_pages: [0],
  svgs: ['<svg viewBox="0 0 100 100"></svg>'],
  compile_ms: 1,
  svg_ms: 1,
  warnings: [],
  page_sizes: [[100, 100]],
  page_labels: ["1"],
};

const invoke = vi.fn(async (command: string, _args?: unknown): Promise<unknown> => {
  switch (command) {
    case "satz_all_pages":
      return report;
    case "satz_paragraph_at":
      return {
        file: "Arbeit/kapitel/01-einleitung.typ",
        start: 10,
        end: 20,
        text: "Ein Absatz.",
        rects: [[0, { x: 10, y: 10, width: 50, height: 12 }]],
        font_size: 11,
        line_height: 14.8,
        column_x: 10,
        column_width: 80,
        first_line_indent: 0,
      };
    case "satz_font":
      throw "keine Schrift im Test";
    default:
      return null;
  }
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: (c: string, a?: unknown) => invoke(c, a) }));

// jsdom kennt keine Objekt-URLs.
URL.createObjectURL = vi.fn(() => "blob:seite");
URL.revokeObjectURL = vi.fn();

describe("PageEditor", () => {
  it("öffnet nach einem Klick das Feld mit dem Absatz", async () => {
    const onReport = vi.fn();
    const { container } = render(
      <PageEditor scrollTo={null} onReport={onReport} onSaved={vi.fn()} onSaving={vi.fn()} />,
    );
    await waitFor(() => expect(container.querySelector(".page")).not.toBeNull());
    expect(onReport).toHaveBeenCalledWith(report);
    fireEvent.click(container.querySelector(".page")!, { clientX: 20, clientY: 20 });
    await waitFor(() => expect(container.querySelector(".editor-overlay")).not.toBeNull());
    expect(invoke).toHaveBeenCalledWith("satz_paragraph_at", expect.objectContaining({ page: 0 }));
    expect(screen.getByText("Ein Absatz.")).toBeDefined();
  });
});
