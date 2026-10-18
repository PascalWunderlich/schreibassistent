import { useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/ui/button";
import { bibUebernehmen, zoteroLaeuft } from "./api";

interface Props {
  /** Ist schon eine Arbeit geöffnet? Dann geht es nach dem ersten Schritt weiter. */
  projectName: string | null;
  onOpen: () => void;
  onNew: () => void;
  onSettings: () => void;
  /** Ersteinstieg beenden (fertig oder übersprungen). */
  onFinish: () => void;
}

const STEPS = ["Arbeit", "Quellen", "KI"] as const;

/** Ersteinstieg beim allerersten Start: Arbeit, Quellen, KI-Anbieter – jeder Schritt lässt sich überspringen. */
export function Ersteinstieg({ projectName, onOpen, onNew, onSettings, onFinish }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [step, setStep] = useState(0);
  const [zotero, setZotero] = useState<"unbekannt" | "prueft" | "laeuft" | "fehlt">("unbekannt");
  const [bib, setBib] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => ref.current?.showModal(), []);

  // Sobald eine Arbeit offen ist, weiter zu den Quellen.
  useEffect(() => {
    if (projectName) setStep((s) => (s === 0 ? 1 : s));
  }, [projectName]);

  async function checkZotero() {
    setZotero("prueft");
    setZotero((await zoteroLaeuft().catch(() => false)) ? "laeuft" : "fehlt");
  }

  async function chooseBib() {
    setError(null);
    const path = await open({ title: ".bib-Datei wählen", filters: [{ name: "BibTeX", extensions: ["bib"] }] });
    if (typeof path !== "string") return;
    try {
      const n = await bibUebernehmen(path);
      setBib(`${n} ${n === 1 ? "Quelle" : "Quellen"} übernommen.`);
    } catch (e) {
      setError(String(e));
    }
  }

  const next = () => setStep((s) => s + 1);
  return (
    <dialog
      ref={ref}
      aria-labelledby="ersteinstieg-titel"
      onClose={onFinish}
      className="m-auto w-[480px] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
    >
      <div className="flex flex-col gap-3 p-4">
        <h2 id="ersteinstieg-titel" className="text-base font-semibold">
          Willkommen im Schreibassistenten
        </h2>
        <ol className="m-0 flex list-none gap-3 p-0 text-xs text-muted-foreground">
          {STEPS.map((name, i) => (
            <li key={name} aria-current={i === step ? "step" : undefined} className={i === step ? "font-semibold text-foreground" : ""}>
              {i + 1}. {name}
            </li>
          ))}
        </ol>

        {step === 0 && (
          <>
            <p className="text-sm">Öffne eine vorhandene Arbeit oder lege eine neue an. Du kannst das auch später im Menü tun.</p>
            <div className="flex justify-end gap-2">
              <Button size="sm" variant="outline" onClick={next}>
                Überspringen
              </Button>
              <Button size="sm" variant="outline" onClick={onOpen}>
                Arbeit öffnen …
              </Button>
              <Button size="sm" onClick={onNew}>
                Neue Arbeit …
              </Button>
            </div>
          </>
        )}

        {step === 1 && (
          <>
            <p className="text-sm">
              Quellen verwaltet Zotero. Die App liest sie aus <code>System/quellen.bib</code> und fragt Zotero lokal nach PDFs und
              Markierungen. Das ist freiwillig; Typst-Arbeit und Prüfungen gehen auch ohne Quellen.
            </p>
            <div className="flex flex-col items-start gap-2">
              <Button size="sm" variant="outline" onClick={checkZotero} disabled={zotero === "prueft"}>
                Quellen aus Zotero verbinden
              </Button>
              {zotero === "laeuft" && <p className="text-xs text-muted-foreground">Zotero läuft und ist erreichbar. Better BibTeX muss <code>quellen.bib</code> der Arbeit schreiben (Anleitung: docs/zotero-einrichtung.md).</p>}
              {zotero === "fehlt" && <p className="text-xs text-destructive">Zotero ist nicht erreichbar. Starte Zotero und erlaube unter Einstellungen → Erweitert die Kommunikation mit anderen Anwendungen.</p>}
              <Button size="sm" variant="outline" onClick={chooseBib} disabled={!projectName}>
                .bib-Datei wählen …
              </Button>
              {!projectName && <p className="text-xs text-muted-foreground">Dafür zuerst eine Arbeit öffnen.</p>}
              {bib && <p className="text-xs">{bib}</p>}
              {error && (
                <p role="alert" className="text-xs text-destructive">
                  {error}
                </p>
              )}
            </div>
            <div className="flex justify-end gap-2">
              <Button size="sm" variant="outline" onClick={next}>
                Ohne Quellen starten
              </Button>
              <Button size="sm" onClick={next}>
                Weiter
              </Button>
            </div>
          </>
        )}

        {step === 2 && (
          <>
            <p className="text-sm">
              KI-Funktionen sind freiwillig und antworten nur auf Nachfrage. Sie schreiben nie in deinen Text. Für einen Online-Anbieter
              trägst du in den Einstellungen deinen API-Schlüssel ein; er liegt im Schlüsselspeicher deines Systems.
            </p>
            <div className="flex justify-end gap-2">
              <Button size="sm" variant="outline" onClick={onSettings}>
                Einstellungen öffnen …
              </Button>
              <Button size="sm" onClick={() => ref.current?.close()}>
                Fertig
              </Button>
            </div>
          </>
        )}
      </div>
    </dialog>
  );
}
