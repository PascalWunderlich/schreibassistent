import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { Ersteinstieg } from "./Ersteinstieg";

const invoke = vi.fn();
const open = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: (...a: unknown[]) => open(...a) }));

beforeAll(() => {
  HTMLDialogElement.prototype.showModal ??= function () { this.setAttribute("open", ""); };
  HTMLDialogElement.prototype.close ??= function () { this.removeAttribute("open"); this.dispatchEvent(new Event("close")); };
});
afterEach(() => {
  cleanup();
  invoke.mockReset();
  open.mockReset();
});

const props = () => ({ projectName: null as string | null, onOpen: vi.fn(), onNew: vi.fn(), onSettings: vi.fn(), onFinish: vi.fn() });

describe("Ersteinstieg", () => {
  it("startet bei der Arbeit und ruft Öffnen und Neu auf", () => {
    const p = props();
    render(<Ersteinstieg {...p} />);
    expect(screen.getByText("1. Arbeit").getAttribute("aria-current")).toBe("step");
    fireEvent.click(screen.getByRole("button", { name: "Arbeit öffnen …" }));
    fireEvent.click(screen.getByRole("button", { name: "Neue Arbeit …" }));
    expect(p.onOpen).toHaveBeenCalled();
    expect(p.onNew).toHaveBeenCalled();
  });

  it("geht weiter zu den Quellen, sobald eine Arbeit offen ist", () => {
    const p = props();
    const { rerender } = render(<Ersteinstieg {...p} />);
    rerender(<Ersteinstieg {...p} projectName="meine-arbeit" />);
    expect(screen.getByText("2. Quellen").getAttribute("aria-current")).toBe("step");
  });

  it("lässt sich komplett überspringen und meldet das Ende einmal", () => {
    const p = props();
    render(<Ersteinstieg {...p} />);
    fireEvent.click(screen.getByRole("button", { name: "Überspringen" }));
    fireEvent.click(screen.getByRole("button", { name: "Ohne Quellen starten" }));
    expect(screen.getByText("3. KI").getAttribute("aria-current")).toBe("step");
    fireEvent.click(screen.getByRole("button", { name: "Fertig" }));
    expect(p.onFinish).toHaveBeenCalledTimes(1);
  });

  it("meldet, ob Zotero erreichbar ist", async () => {
    invoke.mockResolvedValueOnce(false);
    const p = props();
    render(<Ersteinstieg {...p} projectName="a" />);
    fireEvent.click(screen.getByRole("button", { name: "Quellen aus Zotero verbinden" }));
    await screen.findByText(/Zotero ist nicht erreichbar/);
    expect(invoke).toHaveBeenCalledWith("quellen_zotero_laeuft");
    invoke.mockResolvedValueOnce(true);
    fireEvent.click(screen.getByRole("button", { name: "Quellen aus Zotero verbinden" }));
    await screen.findByText(/Zotero läuft und ist erreichbar/);
  });

  it("übernimmt eine .bib-Datei in die Arbeit und zeigt Fehler an", async () => {
    open.mockResolvedValue("/x/quellen.bib");
    invoke.mockResolvedValueOnce(3);
    render(<Ersteinstieg {...props()} projectName="a" />);
    fireEvent.click(screen.getByRole("button", { name: ".bib-Datei wählen …" }));
    await screen.findByText("3 Quellen übernommen.");
    expect(invoke).toHaveBeenCalledWith("quellen_bib_uebernehmen", { path: "/x/quellen.bib" });
    invoke.mockRejectedValueOnce("Die Arbeit hat schon Quellen (System/quellen.bib). Sie werden nicht überschrieben.");
    fireEvent.click(screen.getByRole("button", { name: ".bib-Datei wählen …" }));
    await screen.findByRole("alert");
  });

  it("verlangt für die .bib-Datei eine geöffnete Arbeit", () => {
    render(<Ersteinstieg {...props()} />);
    fireEvent.click(screen.getByRole("button", { name: "Überspringen" }));
    expect((screen.getByRole("button", { name: ".bib-Datei wählen …" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("öffnet die Einstellungen im KI-Schritt", async () => {
    const p = props();
    render(<Ersteinstieg {...p} projectName="a" />);
    fireEvent.click(screen.getByRole("button", { name: "Weiter" }));
    fireEvent.click(screen.getByRole("button", { name: "Einstellungen öffnen …" }));
    await waitFor(() => expect(p.onSettings).toHaveBeenCalled());
  });
});
