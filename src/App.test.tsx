import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("App", () => {
  it("zeigt den App-Namen", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "Schreibassistent" })).toBeDefined();
  });
});
