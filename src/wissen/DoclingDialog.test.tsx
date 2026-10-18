import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { DoclingDialog } from "./DoclingDialog";

const invoke = vi.fn();
let emit: (line: string) => void = () => {};
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_name: string, cb: (e: { payload: string }) => void) => {
    emit = (line) => cb({ payload: line });
    return () => {};
  }),
}));

beforeAll(() => {
  // jsdom kennt <dialog> nur teilweise.
  HTMLDialogElement.prototype.showModal ??= function () { this.setAttribute("open", ""); };
  HTMLDialogElement.prototype.close ??= function () { this.removeAttribute("open"); this.dispatchEvent(new Event("close")); };
});
afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const status = { bereit: false, uv: true, groesse_mb: 2000, ordner: "/daten/docling" };

describe("DoclingDialog", () => {
  it("fragt zuerst und lädt erst nach Bestätigung", () => {
    render(<DoclingDialog status={status} onDone={vi.fn()} onCancel={vi.fn()} />);
    expect(screen.getByText(/ca\. 2 GB/)).toBeDefined();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("zeigt den Fortschritt und meldet das Ende", async () => {
    let finish: () => void = () => {};
    invoke.mockReturnValue(new Promise<void>((resolve) => (finish = resolve)));
    const onDone = vi.fn();
    render(<DoclingDialog status={status} onDone={onDone} onCancel={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Jetzt laden" }));
    await screen.findByLabelText("Docling wird geladen");
    expect(invoke).toHaveBeenCalledWith("ingest_werkzeug_installieren");
    emit("Downloading torch");
    await screen.findByText("Downloading torch");
    finish();
    await waitFor(() => expect(onDone).toHaveBeenCalled());
  });

  it("zeigt Fehler wie fehlendes Netz und erlaubt einen neuen Versuch", async () => {
    invoke.mockRejectedValueOnce("Keine Internetverbindung. Bitte mit dem Netz verbinden und erneut versuchen.");
    render(<DoclingDialog status={status} onDone={vi.fn()} onCancel={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Jetzt laden" }));
    await screen.findByText(/Keine Internetverbindung/);
    invoke.mockResolvedValueOnce(undefined);
    fireEvent.click(screen.getByRole("button", { name: "Erneut versuchen" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
  });

  it("sperrt den Knopf, wenn uv fehlt", () => {
    render(<DoclingDialog status={{ ...status, uv: false }} onDone={vi.fn()} onCancel={vi.fn()} />);
    expect((screen.getByRole("button", { name: "Jetzt laden" }) as HTMLButtonElement).disabled).toBe(true);
  });
});
