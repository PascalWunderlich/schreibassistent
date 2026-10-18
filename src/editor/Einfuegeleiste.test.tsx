import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Einfuegeleiste } from "./Einfuegeleiste";
import { setInserter } from "./inserter";

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

afterEach(() => setInserter(null));

describe("Einfuegeleiste", () => {
  it("ist ohne Cursor im Text gesperrt", () => {
    render(<Einfuegeleiste sources={[]} projectRoot="/v" />);
    expect((screen.getByRole("button", { name: "Liste" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("fügt einen Baustein über den aktiven Editor ein", () => {
    const insert = vi.fn();
    render(<Einfuegeleiste sources={[]} projectRoot="/v" />);
    act(() => setInserter({ file: "Arbeit/kapitel/01.typ", insert }));
    fireEvent.click(screen.getByRole("button", { name: "Liste" }));
    expect(insert).toHaveBeenCalledWith(expect.objectContaining({ text: expect.stringContaining("- Punkt") }));
  });
});
