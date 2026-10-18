import { afterEach, describe, expect, it, vi } from "vitest";
import { isMac, keys, mod, modInit, modLabel } from "./platform";

function platform(name: string) {
  vi.spyOn(navigator, "platform", "get").mockReturnValue(name);
}

afterEach(() => vi.restoreAllMocks());

describe("macOS", () => {
  it("nimmt ⌘ als Kürzel-Taste", () => {
    platform("MacIntel");
    expect(isMac()).toBe(true);
    expect(mod({ metaKey: true, ctrlKey: false })).toBe(true);
    expect(mod({ metaKey: false, ctrlKey: true })).toBe(false);
    expect(modLabel()).toBe("⌘");
    expect(modInit()).toEqual({ metaKey: true });
    expect(keys("⌘⇧F")).toBe("⌘⇧F");
  });
});

describe.each(["Win32", "Linux x86_64"])("%s", (name) => {
  it("nimmt Strg als Kürzel-Taste", () => {
    platform(name);
    expect(isMac()).toBe(false);
    expect(mod({ metaKey: false, ctrlKey: true })).toBe(true);
    expect(mod({ metaKey: true, ctrlKey: false })).toBe(false);
    expect(modLabel()).toBe("Strg");
    expect(modInit()).toEqual({ ctrlKey: true });
  });

  it("schreibt Kürzel aus", () => {
    platform(name);
    expect(keys("⌘K")).toBe("Strg+K");
    expect(keys("⌘⇧F")).toBe("Strg+Umschalt+F");
    expect(keys("⌘1 bis ⌘9")).toBe("Strg+1 bis Strg+9");
    expect(keys("⌘⌥←")).toBe("Strg+Alt+←");
    expect(keys("Frage stellen (⌘↩ senden)")).toBe("Frage stellen (Strg+↩ senden)");
  });
});
