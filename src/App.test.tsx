import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";

// Kein Tauri im Test: noch keine Arbeit geöffnet.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(async () => null) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

describe("App", () => {
  it("zeigt ohne geöffnete Arbeit den Startbildschirm", async () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "Schreibassistent" })).toBeDefined();
    await waitFor(() => expect(screen.getByRole("button", { name: "Arbeit öffnen …" })).toBeDefined());
  });
});
