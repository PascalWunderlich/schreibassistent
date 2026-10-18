import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Vorschlag, kiRegelprofilAusPdf, kiRegelprofilSpeichern } from "./api";

interface Props {
  onClose: () => void;
}

/** Regelprofil aus einem Vorgaben-PDF: KI schlägt vor, ich bestätige Regel für Regel. */
export function RegelprofilDialog({ onClose }: Props) {
  const [pdf, setPdf] = useState<string | null>(null);
  const [vorschlaege, setVorschlaege] = useState<Vorschlag[] | null>(null);
  const [modell, setModell] = useState("");
  const [chosen, setChosen] = useState<Set<number>>(new Set());
  const [name, setName] = useState("");
  const [ziel, setZiel] = useState("System/regelprofil-aus-pdf.yaml");
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);

  async function choose() {
    const path = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }], title: "Vorgaben-PDF wählen" });
    if (typeof path !== "string") return;
    setPdf(path);
    setName(path.split("/").pop()!.replace(/\.pdf$/i, ""));
    setError(null);
    setBusy("liest das PDF (Docling) und fragt die KI … (kann 1–2 Minuten dauern)");
    try {
      const r = await kiRegelprofilAusPdf(path);
      setVorschlaege(r.regeln);
      setModell(r.modell);
      setChosen(new Set(r.regeln.map((v, i) => (!v.problem && v.zitat_ok ? i : -1)).filter((i) => i >= 0)));
    } catch (e) {
      setError(String(e));
    }
    setBusy(null);
  }

  async function save() {
    if (!vorschlaege) return;
    try {
      const regeln = vorschlaege.filter((_, i) => chosen.has(i)).map((v) => v.regel);
      const path = await kiRegelprofilSpeichern(name, pdf!.split("/").pop()!, regeln, ziel);
      setSaved(`${regeln.length} Regeln gespeichert in ${path}.`);
    } catch (e) {
      setError(String(e));
    }
  }

  const toggle = (i: number) =>
    setChosen((c) => {
      const next = new Set(c);
      if (next.has(i)) next.delete(i);
      else next.add(i);
      return next;
    });

  return (
    <div className="dialog-backdrop" onMouseDown={(e) => e.target === e.currentTarget && !busy && onClose()}>
      <div className="dialog regelprofil" role="dialog" aria-label="Regelprofil aus PDF">
        <h3>Regelprofil aus Vorgaben-PDF</h3>
        {!vorschlaege && !busy && (
          <p>
            Die KI liest das PDF und schlägt Regeln mit Fundstelle und Zitat vor. Du bestätigst jede Regel einzeln; gespeichert
            wird erst auf Klick. <button onClick={choose}>PDF wählen …</button>
          </p>
        )}
        {busy && <p className="meta">{busy}</p>}
        {vorschlaege && (
          <>
            <p className="meta">
              {vorschlaege.length} Vorschläge von {modell}. Vorausgewählt sind Regeln, die zum Schema passen und deren Zitat im PDF
              steht.
            </p>
            <ul className="rules">
              {vorschlaege.map((v, i) => (
                <li key={i} className={chosen.has(i) ? "chosen" : ""}>
                  <label>
                    <input type="checkbox" checked={chosen.has(i)} onChange={() => toggle(i)} disabled={!!v.problem && !v.problem.startsWith("Zitat")} />
                    <b>{String(v.regel.id ?? "?")}</b> <span className="meta">{String(v.regel.klasse ?? "")}{v.regel.art ? ` · ${String(v.regel.art)}` : ""} · {String(v.regel.fundstelle ?? "")}</span>
                  </label>
                  <pre>{v.yaml}</pre>
                  <div className={v.zitat_ok ? "zitat ok" : "zitat"}>
                    {v.zitat_ok ? "✓" : "⚠"} „{v.zitat}“
                  </div>
                  {v.problem && <div className="warn">{v.problem}</div>}
                </li>
              ))}
            </ul>
            <label className="field">
              Name des Profils
              <input value={name} onChange={(e) => setName(e.target.value)} />
            </label>
            <label className="field">
              Speichern als
              <select value={ziel} onChange={(e) => setZiel(e.target.value)}>
                <option value="System/regelprofil-aus-pdf.yaml">System/regelprofil-aus-pdf.yaml (zum Vergleichen)</option>
                <option value="System/regelprofil.yaml">System/regelprofil.yaml (aktives Profil ersetzen, alte Fassung wird gesichert)</option>
              </select>
            </label>
          </>
        )}
        {error && <p className="warn">{error}</p>}
        {saved && <p className="ok">{saved}</p>}
        <div className="dialog-buttons">
          <button onClick={onClose} disabled={!!busy}>
            {saved ? "Schließen" : "Abbrechen"}
          </button>
          {vorschlaege && !saved && (
            <button className="primary" onClick={save} disabled={chosen.size === 0 || !name.trim()}>
              {chosen.size} Regeln speichern
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
