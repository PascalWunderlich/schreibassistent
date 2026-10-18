import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { IngestStatus, Schaetzung, ingestErzeugen, ingestSchaetzen, ingestStatus } from "./api";

interface Props {
  citekey: string;
  onOpenFile: (path: string) => void;
}

const SEITENZAHLEN: Record<string, string> = {
  gedruckt: "gedruckte Seitenzahlen erkannt",
  pdf: "Seitenzahlen aus dem PDF",
  "pdf-seite": "keine gedruckten Seitenzahlen – PDF-Seiten",
};

/** Wegweiser-Seite einer Quelle: einlesen (kostenlos), Schätzung, erst dann KI. */
export function Wegweiser({ citekey, onOpenFile }: Props) {
  const [status, setStatus] = useState<IngestStatus | null>(null);
  const [schaetzung, setSchaetzung] = useState<Schaetzung | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);

  const load = () => ingestStatus(citekey).then(setStatus).catch((e) => setError(String(e)));
  useEffect(() => {
    load();
    const unlisten = listen<[string, string]>("ingest_fortschritt", (e) => {
      if (e.payload[0] === citekey && e.payload[1]) setBusy(e.payload[1]);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [citekey]);

  async function estimate() {
    setError(null);
    setBusy("liest das PDF (Docling) …");
    try {
      setSchaetzung(await ingestSchaetzen(citekey));
      load();
    } catch (e) {
      setError(String(e));
    }
    setBusy(null);
  }

  async function create(neu: boolean) {
    setError(null);
    setBusy("bereitet vor …");
    try {
      const r = await ingestErzeugen(citekey, neu);
      setResult(`${r.bestaetigt} von ${r.gesamt} Stellen im PDF bestätigt (${r.modell}).`);
      setSchaetzung(null);
      await load();
    } catch (e) {
      setError(String(e));
    }
    setBusy(null);
  }

  if (!status) return <div className="wegweiser meta">Wegweiser …</div>;

  return (
    <div className="wegweiser">
      <div className="wegweiser-head">Wegweiser</div>
      {status.seite && (
        <div>
          <button className="item" onClick={() => onOpenFile(status.seite!)}>
            Wegweiser öffnen
          </button>
          <span className="meta">
            {status.stellen ? ` · ${status.stellen} Stellen bestätigt` : ""}
            {status.aktuell ? " · aktuell" : status.grund ? ` · veraltet: ${status.grund}` : ""}
          </span>
        </div>
      )}
      {!status.zotero && !status.seite && <span className="meta">Zotero läuft nicht</span>}
      {status.zotero && !status.pdf && !status.seite && <span className="meta">kein PDF in Zotero – nichts einzulesen</span>}
      {status.pdf && !schaetzung && !busy && (!status.seite || !status.aktuell) && (
        <button onClick={estimate}>{status.seite ? "Neu einlesen …" : "Einlesen …"}</button>
      )}
      {status.pdf && status.seite && status.aktuell && !schaetzung && !busy && (
        <button className="subtle" onClick={estimate} title="Kostet erneut Tokens">
          Neu erzeugen …
        </button>
      )}
      {schaetzung && !busy && (
        <div className="estimate">
          <div>
            {schaetzung.seiten} Seiten, {schaetzung.abschnitte} Abschnitte
            {schaetzung.abschnitte_gewaehlt < schaetzung.abschnitte && `, davon ${schaetzung.abschnitte_gewaehlt} ausgewählt`}
            {schaetzung.markierungen > 0 && ` · ${schaetzung.markierungen} Markierungen`}
          </div>
          <div className="meta">{SEITENZAHLEN[schaetzung.seitenzahlen] ?? schaetzung.seitenzahlen}</div>
          <div>
            KI: <b>{schaetzung.modell}</b> ({schaetzung.anbieter}) · ca. {schaetzung.tokens_ein.toLocaleString("de-DE")} Tokens
            hin, bis {schaetzung.tokens_aus.toLocaleString("de-DE")} zurück
          </div>
          <div className="estimate-buttons">
            <button onClick={() => setSchaetzung(null)}>Abbrechen</button>
            <button className="primary" onClick={() => create(!!status.seite)}>
              Wegweiser erzeugen
            </button>
          </div>
        </div>
      )}
      {busy && <span className="meta">{busy}</span>}
      {result && !busy && <div className="ok">{result}</div>}
      {error && <div className="warn">{error}</div>}
    </div>
  );
}
