import { ReactNode, useState } from "react";
import { RegelprofilDialog } from "../assistent/RegelprofilDialog";
import { Hinweis, Pruefbericht } from "../editor/api";

interface Props {
  bericht: Pruefbericht | null;
  rechtschreibung: Hinweis[] | "laeuft" | null;
  onJump: (hinweis: Hinweis) => void;
  /** Unten: Assistent (P6). */
  assistant: ReactNode;
}

/** Lesbare Namen der Regeln; unbekannte zeigen ihre ID. */
const RULE_NAMES: Record<string, string> = {
  "umfang-praxisarbeit": "Seitenumfang",
  "umfang-bachelorarbeit": "Seitenumfang",
  "beschriftung-abbildung": "Abbildungen",
  "beschriftung-tabelle": "Tabellen",
  "beschriftung-listing": "Listings",
  gliederung: "Gliederung",
  zitierstil: "Zitieren",
  quellen: "Quellen",
  "erklaerung-wortlaut": "Erklärung",
  "stil-man": "Stil: „man“",
  "stil-fuellwoerter": "Stil: Füllwörter",
  rechtschreibung: "Rechtschreibung",
};

const ORDER: Record<string, number> = { fehler: 0, warnung: 1, hinweis: 2 };

/** Rechts: oben feste Prüfungen ohne KI (P5), unten Assistent auf Nachfrage (P6). */
export function RightPanel({ bericht, rechtschreibung, onJump, assistant }: Props) {
  const [profilDialog, setProfilDialog] = useState(false);
  const all = [...(bericht?.hinweise ?? []), ...(Array.isArray(rechtschreibung) ? rechtschreibung : [])];
  const groups = new Map<string, Hinweis[]>();
  for (const h of all) {
    const name = RULE_NAMES[h.regel_id] ?? h.regel_id;
    groups.set(name, [...(groups.get(name) ?? []), h]);
  }
  const sorted = [...groups.entries()].sort(
    ([, a], [, b]) => Math.min(...a.map((h) => ORDER[h.schwere])) - Math.min(...b.map((h) => ORDER[h.schwere])),
  );

  return (
    <aside className="right-panel">
      <section className="checks">
        <h2>
          Prüfungen <span className="meta">· {all.length}</span>
          <button className="h2-action" onClick={() => setProfilDialog(true)} title="Regelprofil aus einem Vorgaben-PDF (KI, mit Bestätigung)">
            Profil aus PDF …
          </button>
        </h2>
        {profilDialog && <RegelprofilDialog onClose={() => setProfilDialog(false)} />}
        {!bericht && <p className="empty">prüft …</p>}
        {bericht && !bericht.profil && bericht.fehler.length === 0 && (
          <p className="empty">Kein Regelprofil (System/regelprofil.yaml) – nur Rechtschreibung.</p>
        )}
        {bericht?.profil && !bericht.profil.bestaetigt && (
          <p className="warn">Regelprofil „{bericht.profil.profil}“ ist noch nicht bestätigt.</p>
        )}
        {bericht?.fehler.map((f, i) => (
          <p key={i} className="warn">
            {f}
          </p>
        ))}
        {rechtschreibung === "laeuft" && <p className="empty">Rechtschreibung wird geprüft …</p>}
        {bericht && all.length === 0 && <p className="ok">Keine Hinweise.</p>}
        {sorted.map(([name, list]) => (
          <div key={name} className="check-group">
            <h3>
              {name} <span className="meta">{list.length}</span>
            </h3>
            <ul>
              {list.map((h, i) => (
                <li key={i}>
                  <button
                    className={`check ${h.schwere}`}
                    onClick={() => onJump(h)}
                    disabled={!h.datei}
                    title={`${h.fundstelle}${h.quelle !== "regel" ? ` · ${h.quelle}` : ""}`}
                  >
                    <span className="badge-schwere">{h.schwere}</span> {h.text}
                    {h.datei && (
                      <span className="meta">
                        {" "}
                        · {h.datei.split("/").pop()}
                        {h.zeile ? `:${h.zeile}` : ""}
                      </span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </section>
      {assistant}
    </aside>
  );
}
