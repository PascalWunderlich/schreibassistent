import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";

// Kein Tauri im Test: Backend-Aufrufe liefern ein leeres Dokument.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => ({
    page_count: 0,
    changed_pages: [],
    svgs: [],
    compile_ms: 0,
    svg_ms: 0,
    warnings: [],
    page_sizes: [],
    page_labels: [],
  })),
}));

describe("App", () => {
  it("zeigt den App-Namen und beide Prototypen", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "Schreibassistent" })).toBeDefined();
    expect(screen.getByRole("button", { name: "A: Seiteneditor" })).toBeDefined();
    expect(screen.getByRole("button", { name: "B: Vorschau" })).toBeDefined();
  });
});
