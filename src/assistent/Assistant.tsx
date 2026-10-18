import { useEffect, useRef, useState } from "react";
import { FileDown, Settings, ShieldCheck, Sparkles } from "lucide-react";
import { Button } from "../components/ui/button";
import {
  FrageErgebnis,
  KontextWahl,
  kiBelegbedarf,
  kiEinstellungen,
  kiFrage,
  kiHierzu,
  kiQuelleAussage,
  kiRoterFaden,
  kiUrteilsregeln,
  protokollExport,
  protokollPruefen,
} from "./api";
import { Aufgabe, AufgabeErgebnis } from "./Aufgaben";
import { KiSettingsDialog } from "./KiSettingsDialog";
import { keys, mod } from "../lib/platform";

interface Props {
  /** Zuletzt bearbeiteter Absatz (Datei und Byte-Offset). */
  paragraph: { file: string; offset: number } | null;
  /** Aktuelles Kapitel (Datei). */
  chapter: string | null;
  onAnswer?: (result: FrageErgebnis) => void;
  /** Nach jeder abgeschlossenen Anfrage (steht dann im Protokoll). */
  onLogged?: () => void;
  /** Nach dem Export (Datei in Arbeit/ geändert). */
  onExported: () => void;
  /** Zur Stelle im Text springen. */
  onJump?: (datei: string, offset: number) => void;
  /** Notiz oder Wissensseite öffnen. */
  onOpenFile?: (path: string) => void;
}

type Art = "absatz" | "kapitel" | "keiner";

interface Turn {
  frage: string;
  kontext: string;
  ergebnis?: FrageErgebnis;
  aufgabe?: Aufgabe;
  fehler?: string;
}

const short = (file: string) => file.split("/").pop() ?? file;

/** Rechts unten: Fragen an die KI auf Nachfrage. Antworten nur als Text, ohne Einfügen. */
export function Assistant({ paragraph, chapter, onAnswer = () => {}, onLogged = () => {}, onExported, onJump = () => {}, onOpenFile = () => {} }: Props) {
  const [art, setArt] = useState<Art>("absatz");
  const [frage, setFrage] = useState("");
  const [turns, setTurns] = useState<Turn[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [modell, setModell] = useState<string | null>(null);
  const listRef = useRef<HTMLDivElement>(null);

  function loadModel() {
    kiEinstellungen()
      .then((info) => setModell(info.settings.aufgaben.frage.modell || null))
      .catch(() => setModell(null));
  }
  useEffect(loadModel, []);

  useEffect(() => {
    const list = listRef.current;
    if (list) list.scrollTop = list.scrollHeight;
  }, [turns, busy]);

  const kontext: KontextWahl | null =
    art === "keiner"
      ? { art: "keiner" }
      : art === "absatz"
        ? paragraph
          ? { art: "absatz", datei: paragraph.file, offset: paragraph.offset }
          : null
        : chapter
          ? { art: "kapitel", datei: chapter }
          : null;
  const kontextName =
    art === "keiner"
      ? "ohne Kontext"
      : art === "absatz"
        ? paragraph
          ? `Absatz in ${short(paragraph.file)}`
          : "kein Absatz gewählt"
        : chapter
          ? `Kapitel ${short(chapter)}`
          : "kein Kapitel";

  async function ask() {
    const text = frage.trim();
    if (!text || !kontext || busy) return;
    setBusy(modell ?? "KI");
    setNotice(null);
    const turn: Turn = { frage: text, kontext: kontextName };
    try {
      turn.ergebnis = await kiFrage(text, kontext);
      onAnswer(turn.ergebnis);
      setFrage("");
    } catch (e) {
      turn.fehler = String(e);
    }
    setTurns((t) => [...t, turn]);
    setBusy(null);
    onLogged();
  }

  /** Prüfaufgabe (P8): eigener Systemprompt, protokolliert, Ergebnis mit Sprung zur Stelle. */
  async function task(name: string, needs: "absatz" | "kapitel" | "beides", run: () => Promise<Aufgabe>) {
    if (busy) return;
    const ort = needs === "kapitel" || (needs === "beides" && art === "kapitel") ? (chapter ? `Kapitel ${short(chapter)}` : null) : paragraph ? `Absatz in ${short(paragraph.file)}` : null;
    if (!ort) {
      setNotice(needs === "kapitel" ? "Erst ein Kapitel öffnen." : "Erst in einen Absatz klicken.");
      return;
    }
    setBusy(`${name} …`);
    setNotice(null);
    const turn: Turn = { frage: name, kontext: ort };
    try {
      turn.aufgabe = await run();
    } catch (e) {
      turn.fehler = String(e);
    }
    setTurns((t) => [...t, turn]);
    setBusy(null);
    onLogged();
  }

  const tasks = [
    {
      name: "Belegbedarf",
      title: "Sätze mit Tatsache, Zahl, Definition oder fremder These ohne Beleg (Absatz oder Kapitel, je nach Kontext)",
      run: () =>
        task("Belegbedarf", "beides", async () => {
          const kapitel = art === "kapitel" && chapter;
          const datei = kapitel ? chapter! : paragraph!.file;
          return { art: "belegbedarf", ergebnis: await kiBelegbedarf(datei, kapitel ? null : paragraph!.offset) };
        }),
    },
    {
      name: "Quelle ↔ Aussage",
      title: "Für jedes Zitat im Absatz: passende Originalabschnitte und ein KI-Hinweis",
      run: () =>
        task("Quelle ↔ Aussage", "absatz", async () => ({
          art: "quelle",
          ergebnis: await kiQuelleAussage(paragraph!.file, paragraph!.offset),
        })),
    },
    {
      name: "Roter Faden",
      title: "Kapitel gegen Gliederung und Einleitung: Fragen mit Absatznummer",
      run: () => task("Roter Faden", "kapitel", async () => ({ art: "faden", ergebnis: await kiRoterFaden(chapter!) })),
    },
    {
      name: "Was habe ich hierzu?",
      title: "Notizen, Wegweiser und Fundstellen zum Absatz (ohne KI)",
      run: () =>
        task("Was habe ich hierzu?", "absatz", async () => ({
          art: "hierzu",
          datei: paragraph!.file,
          offset: paragraph!.offset,
          ergebnis: await kiHierzu(paragraph!.file, paragraph!.offset),
        })),
    },
    {
      name: "Urteilsregeln",
      title: "Regeln der Klasse „urteil“ aus dem Regelprofil am Kapitel prüfen",
      run: () => task("Urteilsregeln", "kapitel", async () => ({ art: "urteil", ergebnis: await kiUrteilsregeln(chapter!) })),
    },
  ];

  async function check() {
    try {
      const r = await protokollPruefen();
      setNotice(
        r.fehler
          ? `Protokoll fehlerhaft in Zeile ${r.fehler[0]}: ${r.fehler[1]}`
          : `Protokoll in Ordnung: ${r.eintraege} Einträge, Kette lückenlos.`,
      );
    } catch (e) {
      setNotice(String(e));
    }
  }

  async function exportLog() {
    try {
      const file = await protokollExport();
      setNotice(`Exportiert nach ${file}. Im Anhang einbinden mit #include "anhang/ki-nutzung.typ".`);
      onExported();
    } catch (e) {
      setNotice(String(e));
    }
  }

  const slash = frage.startsWith("/");
  const query = slash ? frage.slice(1).trim().toLowerCase() : "";
  const listed = tasks.filter((t) => !slash || t.name.toLowerCase().includes(query));
  const showMenu = (menuOpen || slash) && listed.length > 0;
  const runTask = (t: (typeof tasks)[number]) => {
    setMenuOpen(false);
    setFrage("");
    t.run();
  };
  const kontexte: [Art, string][] = [
    ["absatz", "Absatz"],
    ["kapitel", "Kapitel"],
    ["keiner", "Ohne Kontext"],
  ];
  const iconBtn = "inline-flex size-7 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground";

  return (
    <section className="assistant flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-1 border-b border-border px-3 py-1.5">
        <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground" title="KI-Anbieter und Modell für Fragen">
          {modell ? `KI: ${modell}` : "KI nicht eingerichtet"}
        </span>
        <button className={iconBtn} onClick={check} title="Protokoll prüfen (Hash-Kette von System/ki-protokoll.jsonl)" aria-label="Protokoll prüfen">
          <ShieldCheck className="size-4" />
        </button>
        <button className={iconBtn} onClick={exportLog} title="Protokoll nach Arbeit/anhang/ki-nutzung.typ exportieren" aria-label="Protokoll exportieren">
          <FileDown className="size-4" />
        </button>
        <button className={iconBtn} onClick={() => setSettingsOpen(true)} title="KI-Einstellungen" aria-label="KI-Einstellungen">
          <Settings className="size-4" />
        </button>
      </div>
      {notice && <p className="mx-3 mt-2 text-xs text-muted-foreground">{notice}</p>}
      <div className="turns min-h-0 flex-1 overflow-auto px-3 py-3" ref={listRef}>
        {turns.length === 0 && !busy && (
          <p className="text-sm text-muted-foreground">
            Fragen zum eigenen Text. Die KI gibt Hinweise und Fragen, keinen Text für die Arbeit. Jede Anfrage steht im Protokoll. Mit „/“ wählst du
            eine Prüfaufgabe.
          </p>
        )}
        {turns.map((t, i) => (
          <div key={i} className="mb-4 flex flex-col gap-2">
            <div className="text-sm font-medium">
              {t.frage} <span className="text-xs font-normal text-muted-foreground">· {t.kontext}</span>
            </div>
            {t.ergebnis && (
              <div className="flex flex-col gap-1.5 rounded-lg border border-border bg-card p-3 text-sm text-card-foreground">
                <span className="w-fit rounded-sm bg-hint-ki px-1.5 py-0.5 text-xs font-medium text-hint-ki-foreground">KI · auf Nachfrage</span>
                <div className="whitespace-pre-wrap">{t.ergebnis.text}</div>
                <div className="text-xs text-muted-foreground">
                  {t.ergebnis.modell} ({t.ergebnis.anbieter}) · protokolliert
                </div>
              </div>
            )}
            {t.aufgabe && <AufgabeErgebnis aufgabe={t.aufgabe} onJump={onJump} onOpenFile={onOpenFile} />}
            {t.fehler && <div className="text-sm text-destructive">{t.fehler}</div>}
          </div>
        ))}
        {busy && <p className="text-sm text-muted-foreground">Anfrage an {busy} wird protokolliert und gesendet …</p>}
      </div>
      <div className="relative border-t border-border p-3">
        {showMenu && (
          <ul role="menu" aria-label="Prüfaufgaben" className="absolute inset-x-3 bottom-full mb-1 overflow-hidden rounded-lg border border-border bg-popover text-popover-foreground shadow-md">
            {listed.map((t) => (
              <li key={t.name} role="none">
                <button role="menuitem" onClick={() => runTask(t)} disabled={!!busy} title={t.title} className="flex w-full flex-col px-3 py-2 text-left hover:bg-muted disabled:opacity-50">
                  <span className="text-sm font-medium">{t.name}</span>
                  <span className="text-xs text-muted-foreground">{t.title}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <div role="radiogroup" aria-label="Kontext" className="mb-2 flex flex-wrap items-center gap-1">
          {kontexte.map(([k, label]) => (
            <button
              key={k}
              role="radio"
              aria-checked={art === k}
              onClick={() => setArt(k)}
              className={`rounded-md px-2 py-1 text-xs font-medium ${art === k ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-muted"}`}
            >
              {label}
            </button>
          ))}
        </div>
        <p className={`mb-2 text-xs ${kontext ? "text-muted-foreground" : "text-warning"}`}>Kontext: {kontextName}</p>
        <div className="flex gap-2">
          <button
            className={`${iconBtn} size-9 border border-input`}
            onClick={() => setMenuOpen((o) => !o)}
            aria-expanded={showMenu}
            title="Prüfaufgaben (oder „/“ tippen)"
            aria-label="Prüfaufgaben"
          >
            <Sparkles className="size-4" />
          </button>
          <textarea
            value={frage}
            placeholder={keys("Frage stellen oder / für Aufgaben (⌘↩ senden)")}
            rows={2}
            className="min-h-9 flex-1 resize-none rounded-md border border-input bg-background px-2 py-1.5 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onChange={(e) => setFrage(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && mod(e)) {
                e.preventDefault();
                if (slash && listed.length === 1) runTask(listed[0]);
                else ask();
              } else if (e.key === "Enter" && slash && listed.length >= 1) {
                e.preventDefault();
                runTask(listed[0]);
              } else if (e.key === "Escape") {
                setMenuOpen(false);
                if (slash) setFrage("");
              }
            }}
          />
          <Button onClick={ask} disabled={!frage.trim() || slash || !kontext || !!busy} className="self-end">
            Fragen
          </Button>
        </div>
      </div>
      {settingsOpen && (
        <KiSettingsDialog
          onClose={() => {
            setSettingsOpen(false);
            loadModel();
          }}
        />
      )}
    </section>
  );
}
