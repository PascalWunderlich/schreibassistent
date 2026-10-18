import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { AbsatzHinweis, AussageHinweis, BelegHinweis, Ergebnis, Fund, Hierzu, kiHierzuEinordnen } from "./api";

export type Aufgabe =
  | { art: "belegbedarf"; ergebnis: Ergebnis<BelegHinweis> }
  | { art: "quelle"; ergebnis: Ergebnis<AussageHinweis> }
  | { art: "faden"; ergebnis: Ergebnis<AbsatzHinweis> }
  | { art: "urteil"; ergebnis: Ergebnis<AbsatzHinweis> }
  | { art: "hierzu"; datei: string; offset: number; ergebnis: Hierzu };

interface Props {
  aufgabe: Aufgabe;
  onJump: (datei: string, offset: number) => void;
  onOpenFile: (path: string) => void;
}

const EINSCHAETZUNG: Record<string, string> = {
  stützt: "stützt",
  "stützt teilweise": "stützt teilweise",
  "nicht gefunden": "nicht gefunden",
  unsicher: "unsicher",
};

function Original({ fund }: { fund: Fund }) {
  const [open, setOpen] = useState(false);
  const text = open || fund.text.length < 220 ? fund.text : `${fund.text.slice(0, 220)} …`;
  return (
    <div className="original">
      {fund.link ? (
        <a
          href={fund.link}
          title="PDF in Zotero an dieser Seite öffnen"
          onClick={(e) => {
            e.preventDefault();
            openUrl(fund.link!).catch(() => {});
          }}
        >
          {fund.citekey} S. {fund.label}
        </a>
      ) : (
        <span className="meta">
          {fund.citekey} S. {fund.label}
        </span>
      )}{" "}
      <span className="original-text" onClick={() => setOpen(!open)}>
        {text}
      </span>
    </div>
  );
}

function Removed({ n }: { n: number }) {
  return n > 0 ? <div className="meta">{n} Formulierung(en) der KI entfernt</div> : null;
}

/** Ergebnis einer Prüfaufgabe: Hinweise mit Sprung zur Stelle und Originaltext, nie Formulierungen. */
export function AufgabeErgebnis({ aufgabe, onJump, onOpenFile }: Props) {
  const [einordnung, setEinordnung] = useState<string | null>(null);

  if (aufgabe.art === "belegbedarf") {
    const e = aufgabe.ergebnis;
    return (
      <div className="task-result">
        {e.hinweise.length === 0 && <div className="ok">Kein Satz ohne nötigen Beleg gefunden.</div>}
        <ul>
          {e.hinweise.map((h) => (
            <li key={h.nr}>
              <button className="item" onClick={() => onJump(e.datei, h.start)} title="zur Stelle springen">
                <span className="meta">Satz {h.nr}:</span> {h.satz}
              </button>
              <div className="reason">{h.grund}</div>
            </li>
          ))}
        </ul>
        {e.sonstiges.map((s, i) => (
          <div key={i} className="meta">
            {s}
          </div>
        ))}
        <Removed n={e.entfernt} />
      </div>
    );
  }

  if (aufgabe.art === "quelle") {
    const e = aufgabe.ergebnis;
    return (
      <div className="task-result">
        <ul>
          {e.hinweise.map((h, i) => (
            <li key={i}>
              <button className="item" onClick={() => onJump(e.datei, h.start)} title="zur Stelle springen">
                {h.satz}
              </button>
              <div>
                <span className={`badge-einschaetzung ${(h.einschaetzung ?? "keine").replace(/\s/g, "-")}`}>
                  KI-Hinweis: {h.einschaetzung ? EINSCHAETZUNG[h.einschaetzung] ?? h.einschaetzung : "–"}
                  {h.seite ? `, S. ${h.seite}` : ""}
                </span>{" "}
                <span className="meta">@{h.citekey}{h.ergaenzung ? ` [${h.ergaenzung}]` : ""}</span>
              </div>
              {h.begruendung && <div className="reason">{h.begruendung}</div>}
              {h.problem && <div className="warn">⚠ {h.problem}</div>}
              {h.abschnitte.map((f, j) => (
                <Original key={j} fund={f} />
              ))}
            </li>
          ))}
        </ul>
        <Removed n={e.entfernt} />
      </div>
    );
  }

  if (aufgabe.art === "faden" || aufgabe.art === "urteil") {
    const e = aufgabe.ergebnis;
    return (
      <div className="task-result">
        {e.hinweise.length === 0 && <div className="ok">Nichts aufgefallen.</div>}
        <ul>
          {e.hinweise.map((h, i) => (
            <li key={i}>
              {h.start !== null ? (
                <button className="item" onClick={() => onJump(e.datei, h.start!)} title="zum Absatz springen">
                  <span className="meta">
                    {h.regel ? `[${h.regel}] ` : ""}Absatz {h.absatz}:
                  </span>{" "}
                  {h.text}
                </button>
              ) : (
                <div>
                  {h.regel && <span className="meta">[{h.regel}] </span>}
                  {h.text}
                </div>
              )}
            </li>
          ))}
        </ul>
        <Removed n={e.entfernt} />
      </div>
    );
  }

  if (aufgabe.art !== "hierzu") return null;
  const h = aufgabe.ergebnis;
  const nothing = h.quellen.length === 0 && h.seiten.length === 0 && h.fundstellen.length === 0;
  return (
    <div className="task-result">
      {nothing && <div className="meta">Nichts gefunden.</div>}
      {h.quellen.length > 0 && (
        <div>
          <span className="meta">Zitiert: </span>
          {h.quellen.map(([key, page]) =>
            page ? (
              <button key={key} className="item inline" onClick={() => onOpenFile(page)}>
                {key} (Wegweiser)
              </button>
            ) : (
              <span key={key} className="meta">
                {key} (kein Wegweiser){" "}
              </span>
            ),
          )}
        </div>
      )}
      {h.seiten.length > 0 && (
        <div>
          <span className="meta">Notizen/Wissen: </span>
          {h.seiten.map((p) => (
            <button key={p} className="item inline" onClick={() => onOpenFile(p)}>
              {p.split("/").pop()!.replace(/\.md$/, "")}
            </button>
          ))}
        </div>
      )}
      {h.fundstellen.length > 0 && <div className="meta">Fundstellen (Volltextsuche):</div>}
      {h.fundstellen.map((f, i) => (
        <Original key={i} fund={f} />
      ))}
      {!nothing && einordnung === null && (
        <button
          onClick={() => {
            setEinordnung("ordnet ein …");
            kiHierzuEinordnen(aufgabe.datei, aufgabe.offset)
              .then((r) => setEinordnung(`${r.text}${r.entfernt ? `\n(${r.entfernt} Formulierung(en) entfernt)` : ""}\n– ${r.modell}`))
              .catch((e) => setEinordnung(String(e)));
          }}
        >
          Mit KI einordnen
        </button>
      )}
      {einordnung && <div className="turn-answer">{einordnung}</div>}
    </div>
  );
}
