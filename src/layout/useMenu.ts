import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

/** Ruft für jeden Menüeintrag (ID aus `src-tauri/src/menu.rs`) den passenden Handler auf. */
export function useMenu(handlers: Record<string, () => void>) {
  const ref = useRef(handlers);
  ref.current = handlers;
  useEffect(() => {
    const unlisten = listen<string>("menu", (e) => ref.current[e.payload]?.());
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, []);
}
