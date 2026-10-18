import { useEffect, useState } from "react";

export type Theme = "system" | "light" | "dark";
const KEY = "sa-theme";

function stored(): Theme {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* kein Speicher: Vorgabe */
  }
  return "system";
}

/** Setzt die Klasse `dark` am Wurzelelement; „system“ folgt dem Betriebssystem. */
export function useTheme(): [Theme, (t: Theme) => void] {
  const [theme, setTheme] = useState<Theme>(stored);
  useEffect(() => {
    const media = window.matchMedia?.("(prefers-color-scheme: dark)");
    const apply = () => document.documentElement.classList.toggle("dark", theme === "dark" || (theme === "system" && !!media?.matches));
    apply();
    media?.addEventListener("change", apply);
    try {
      localStorage.setItem(KEY, theme);
    } catch {
      /* ignorieren */
    }
    return () => media?.removeEventListener("change", apply);
  }, [theme]);
  return [theme, setTheme];
}
