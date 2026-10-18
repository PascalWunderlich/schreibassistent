import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { CommandPalette } from "./CommandPalette";

beforeAll(() => {
  // cmdk und Radix brauchen im Testumfeld diese Browserfunktionen.
  globalThis.ResizeObserver ??= class { observe() {} unobserve() {} disconnect() {} } as unknown as typeof ResizeObserver;
  Element.prototype.scrollIntoView ??= () => {};
});
afterEach(cleanup);

const props = () => ({
  chapters: [
    { file: "Arbeit/kapitel/01-einleitung.typ", title: "Einleitung", first_page: 1 },
    { file: "Arbeit/kapitel/02-grundlagen.typ", title: "Grundlagen", first_page: 4 },
  ],
  onOpenChapter: vi.fn(),
  sourceMode: false,
  onSourceMode: vi.fn(),
  onExport: vi.fn(),
  onChoose: vi.fn(),
  onTheme: vi.fn(),
  onShortcuts: vi.fn(),
});

describe("CommandPalette", () => {
  it("öffnet mit ⌘K und wechselt das Kapitel", async () => {
    const p = props();
    render(<CommandPalette {...p} />);
    fireEvent.keyDown(window, { key: "k", metaKey: true });
    fireEvent.click(await screen.findByText("2 Grundlagen"));
    await waitFor(() => expect(p.onOpenChapter).toHaveBeenCalledWith("Arbeit/kapitel/02-grundlagen.typ"));
  });

  it("zeigt Einfügen-Befehle nur, wenn ein Editor aktiv ist", async () => {
    const p = props();
    render(<CommandPalette {...p} />);
    fireEvent.keyDown(window, { key: "k", metaKey: true });
    await screen.findByText("Als PDF exportieren");
    expect(screen.queryByText("Tabelle")).toBeNull();
  });

  it("öffnet mit ⌘/ die Tastenkürzel", () => {
    const p = props();
    render(<CommandPalette {...p} />);
    fireEvent.keyDown(window, { key: "/", metaKey: true });
    expect(p.onShortcuts).toHaveBeenCalled();
  });
});
