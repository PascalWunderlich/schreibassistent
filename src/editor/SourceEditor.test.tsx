import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SourceEditor } from "./SourceEditor";

const MAIN = '#import "@local/fhdw-arbeit:0.1.0": *\n#show: fhdw-thesis.with(title: "T")\n';
const invoke = vi.fn(async (command: string, args?: { path?: string }): Promise<unknown> => {
  if (command === "projekt_read") return args?.path === "Arbeit/main.typ" ? MAIN : "= Kapitel\n";
  if (command === "lsp_field_diagnostics") return [];
  return null;
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: (c: string, a?: { path?: string }) => invoke(c, a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => () => {}) }));

describe("SourceEditor", () => {
  it("zeigt den Quelltext genau einmal, auch mit einem Sprungziel aus einer anderen Datei", async () => {
    const { container } = render(
      <SourceEditor
        path="Arbeit/main.typ"
        jump={{ file: "Arbeit/kapitel/04-loesungskonzept.typ", offset: 5000, nonce: 1 }}
        onSaved={vi.fn()}
        onSaving={vi.fn()}
        onWritten={vi.fn()}
        onDefinition={vi.fn()}
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-content")?.textContent).toContain("fhdw-thesis"));
    expect(container.querySelectorAll(".cm-editor")).toHaveLength(1);
  });
});
