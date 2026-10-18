import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useMenu } from "./useMenu";

let callback: ((e: { payload: string }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_name: string, cb: (e: { payload: string }) => void) => {
    callback = cb;
    return () => {};
  }),
}));

function Probe({ handlers }: { handlers: Record<string, () => void> }) {
  useMenu(handlers);
  return null;
}

describe("useMenu", () => {
  it("ruft den Handler zur Menü-ID auf und ignoriert unbekannte IDs", () => {
    const neu = vi.fn();
    render(<Probe handlers={{ neu }} />);
    callback!({ payload: "neu" });
    callback!({ payload: "unbekannt" });
    expect(neu).toHaveBeenCalledTimes(1);
  });
});
