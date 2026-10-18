import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { NeueArbeitDialog } from "./NeueArbeitDialog";

const open = vi.fn(async () => "/Users/x/Arbeiten");
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: () => open() }));

beforeAll(() => {
  HTMLDialogElement.prototype.showModal ??= function () { this.setAttribute("open", ""); };
  HTMLDialogElement.prototype.close ??= function () { this.removeAttribute("open"); };
});
afterEach(cleanup);

describe("NeueArbeitDialog", () => {
  it("verlangt Titel und Ort", async () => {
    const onCreate = vi.fn(async () => {});
    render(<NeueArbeitDialog onCreate={onCreate} onClose={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Anlegen" }));
    expect((await screen.findByRole("alert")).textContent).toMatch(/Titel/);
    fireEvent.change(screen.getByPlaceholderText(/Titel wie im/), { target: { value: "Mein Titel für Größe" } });
    fireEvent.click(screen.getByRole("button", { name: "Anlegen" }));
    expect((await screen.findByRole("alert")).textContent).toMatch(/Ort/);
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("legt in einem neuen Ordner mit dem Titel als Namen an", async () => {
    const onCreate = vi.fn(async () => {});
    render(<NeueArbeitDialog onCreate={onCreate} onClose={vi.fn()} />);
    fireEvent.change(screen.getByPlaceholderText(/Titel wie im/), { target: { value: "Mein Titel für Größe" } });
    fireEvent.click(screen.getByRole("radio", { name: "Bachelorarbeit" }));
    fireEvent.click(screen.getByRole("button", { name: "Ort wählen …" }));
    await screen.findByText(/mein-titel-fuer-groesse/);
    fireEvent.click(screen.getByRole("button", { name: "Anlegen" }));
    await waitFor(() =>
      expect(onCreate).toHaveBeenCalledWith("/Users/x/Arbeiten/mein-titel-fuer-groesse", {
        title: "Mein Titel für Größe",
        author: "",
        thesis_type: "Bachelorarbeit",
      }),
    );
  });

  it("zeigt Fehler des Anlegens im Dialog", async () => {
    const onCreate = vi.fn(async () => {
      throw "Ordner ist nicht leer";
    });
    render(<NeueArbeitDialog onCreate={onCreate} onClose={vi.fn()} />);
    fireEvent.change(screen.getByPlaceholderText(/Titel wie im/), { target: { value: "T" } });
    fireEvent.click(screen.getByRole("button", { name: "Ort wählen …" }));
    await screen.findByText(/\/t$/);
    fireEvent.click(screen.getByRole("button", { name: "Anlegen" }));
    expect((await screen.findByRole("alert")).textContent).toMatch(/nicht leer/);
  });
});
