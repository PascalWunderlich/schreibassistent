import { useEffect, useRef, useState } from "react";
import { FrageErgebnis, KontextWahl, kiEinstellungen, kiFrage, protokollExport, protokollPruefen } from "./api";
import { KiSettingsDialog } from "./KiSettingsDialog";

interface Props {
  /** Zuletzt bearbeiteter Absatz (Datei und Byte-Offset). */
  paragraph: { file: string; offset: number } | null;
  /** Aktuelles Kapitel (Datei). */
  chapter: string | null;
  onAnswer: (result: FrageErgebnis) => void;
  /** Nach dem Export (Datei in Arbeit/ geändert). */
  onExported: () => void;
}

type Art = "absatz" | "kapitel" | "keiner";

interface Turn {
  frage: string;
  kontext: string;
  ergebnis?: FrageErgebnis;
  fehler?: string;
}

const short = (file: string) => file.split("/").pop() ?? file;

/** Rechts unten: Fragen an die KI auf Nachfrage. Antworten nur als Text, ohne Einfügen. */
export function Assistant({ paragraph, chapter, onAnswer, onExported }: Props) {
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
