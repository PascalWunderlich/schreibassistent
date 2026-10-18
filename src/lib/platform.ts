// Betriebssystem-Unterschiede der Oberfläche: Kürzel-Taste (⌘ oder Strg) und deren Beschriftung.

/** macOS? Wird bei jedem Aufruf gelesen, damit Tests die Plattform wechseln können. */
export function isMac(): boolean {
  return typeof navigator !== "undefined" && /^Mac/i.test(navigator.platform);
}

/** Ist die Kürzel-Taste gedrückt (⌘ auf macOS, sonst Strg)? */
export function mod(e: { metaKey: boolean; ctrlKey: boolean }): boolean {
  return isMac() ? e.metaKey : e.ctrlKey;
}

/** Beschriftung der Kürzel-Taste. */
export function modLabel(): string {
  return isMac() ? "⌘" : "Strg";
}

/** Eigenschaft für ein künstliches Tastenereignis mit gedrückter Kürzel-Taste. */
export function modInit(): { metaKey?: boolean; ctrlKey?: boolean } {
  return isMac() ? { metaKey: true } : { ctrlKey: true };
}

const WORDS: Record<string, string> = { "⌘": "Strg", "⇧": "Umschalt", "⌥": "Alt" };

/** Kürzel in Mac-Schreibweise (`⌘⇧F`, `⌘1 bis ⌘9`) für das System anzeigen: `Strg+Umschalt+F`. */
export function keys(text: string): string {
  if (isMac()) return text;
  return text.replace(/([⌘⇧⌥]+)(\S)/g, (_, mods: string, key: string) => [...[...mods].map((m) => WORDS[m]), key].join("+"));
}
