import { useEffect, useRef, useState } from "react";
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

interface Props {
  /** Zuletzt bearbeiteter Absatz (Datei und Byte-Offset). */
  paragraph: { file: string; offset: number } | null;
  /** Aktuelles Kapitel (Datei). */
  chapter: string | null;
  onAnswer: (result: FrageErgebnis) => void;
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
export function Assistant({ paragraph, chapter, onAnswer, onExported, onJump = () => {}, onOpenFile = () => {} }: Props) {
  const [art, setArt] = useState<Art>("absatz");
  const [frage, setFrage] = useState("");
  const [turns, setTurns] = useState<Turn[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
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

  return (
    <section className="assistant">
      <h2>
        Assistent {modell && <span className="meta">· {modell}</span>}
        <span className="assistant-actions">
          <button onClick={check} title="Hash-Kette von System/ki-protokoll.jsonl prüfen">
            Protokoll prüfen
          </button>
          <button onClick={exportLog} title="Nach Arbeit/anhang/ki-nutzung.typ exportieren">
            Export
          </button>
          <button onClick={() => setSettingsOpen(true)} title="KI-Einstellungen">
            ⚙
          </button>
        </span>
      </h2>
      <div className="task-buttons">
        {tasks.map((t) => (
          <button key={t.name} onClick={t.run} title={t.title} disabled={!!busy}>
            {t.name}
          </button>
        ))}
      </div>
      {notice && <p className="notice-line">{notice}</p>}
      <div className="turns" ref={listRef}>
        {turns.length === 0 && !busy && (
          <p className="empty">
            Fragen zum eigenen Text. Die KI gibt Hinweise und Fragen, keinen Text für die Arbeit. Jede Anfrage steht im
            Protokoll.
          </p>
        )}
        {turns.map((t, i) => (
          <div key={i} className="turn">
            <div className="turn-question">
              {t.frage} <span className="meta">· {t.kontext}</span>
            </div>
            {t.ergebnis && (
              <>
                <div className="turn-answer">{t.ergebnis.text}</div>
                <div className="meta">
                  {t.ergebnis.modell} ({t.ergebnis.anbieter}) · protokolliert
                </div>
              </>
            )}
            {t.aufgabe && <AufgabeErgebnis aufgabe={t.aufgabe} onJump={onJump} onOpenFile={onOpenFile} />}
            {t.fehler && <div className="warn">{t.fehler}</div>}
          </div>
        ))}
        {busy && <p className="empty">fragt {busy} …</p>}
      </div>
      <div className="ask">
        <select value={art} onChange={(e) => setArt(e.target.value as Art)} aria-label="Kontext">
          <option value="absatz">{art === "absatz" ? kontextName : "Absatz"}</option>
          <option value="kapitel">{art === "kapitel" ? kontextName : "Kapitel"}</option>
          <option value="keiner">ohne Kontext</option>
        </select>
        <textarea
          value={frage}
          placeholder="Frage stellen … (⌘↩ senden)"
          rows={2}
          onChange={(e) => setFrage(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
              e.preventDefault();
              ask();
            }
          }}
        />
        <button onClick={ask} disabled={!frage.trim() || !kontext || !!busy}>
          Fragen
        </button>
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
