import { useEffect, useState } from "react";
import {
  ANBIETER,
  AUFGABEN,
  Anbieter,
  KiInfo,
  KiSettings,
  kiEinstellungen,
  kiEinstellungenSpeichern,
  kiSchluesselLoeschen,
  kiSchluesselSetzen,
} from "./api";

interface Props {
  onClose: () => void;
}

/** Anbieter und Modell je Aufgabe; API-Schlüssel gehen direkt in den Schlüsselbund. */
export function KiSettingsDialog({ onClose }: Props) {
  const [info, setInfo] = useState<KiInfo | null>(null);
  const [settings, setSettings] = useState<KiSettings | null>(null);
  const [keys, setKeys] = useState<Partial<Record<Anbieter, string>>>({});
  const [error, setError] = useState<string | null>(null);

  function load() {
    kiEinstellungen()
      .then((i) => {
        setInfo(i);
        setSettings((s) => s ?? i.settings);
      })
      .catch((e) => setError(String(e)));
  }
  useEffect(load, []);

  if (!settings || !info) {
    return (
      <div className="dialog-backdrop">
        <div className="dialog">{error ?? "lädt …"}</div>
      </div>
    );
  }

  const used = new Set(Object.values(settings.aufgaben).map((w) => w.anbieter));

  async function save() {
    try {
      await kiEinstellungenSpeichern(settings!);
      onClose();
    } catch (e) {
      setError(String(e));
    }
  }

  async function storeKey(anbieter: Anbieter) {
    try {
      await kiSchluesselSetzen(anbieter, keys[anbieter] ?? "");
      setKeys((k) => ({ ...k, [anbieter]: "" }));
      load();
    } catch (e) {
      setError(String(e));
    }
  }

  async function removeKey(anbieter: Anbieter) {
    try {
      await kiSchluesselLoeschen(anbieter);
      load();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="dialog-backdrop" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="dialog ki-settings" role="dialog" aria-label="KI-Einstellungen">
        <h3>KI-Einstellungen</h3>
        <table>
          <thead>
            <tr>
              <th>Aufgabe</th>
              <th>Anbieter</th>
              <th>Modell</th>
            </tr>
          </thead>
          <tbody>
            {AUFGABEN.map((a) => {
              const wahl = settings.aufgaben[a.id];
              return (
                <tr key={a.id}>
                  <td>{a.name}</td>
                  <td>
                    <select
                      value={wahl.anbieter}
                      onChange={(e) => {
                        const anbieter = e.target.value as Anbieter;
                        const vorschlag = ANBIETER.find((x) => x.id === anbieter)!.vorschlag;
                        setSettings({ ...settings, aufgaben: { ...settings.aufgaben, [a.id]: { anbieter, modell: vorschlag } } });
                      }}
                    >
                      {ANBIETER.map((x) => (
                        <option key={x.id} value={x.id}>
                          {x.name}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td>
                    <input
                      value={wahl.modell}
                      list={wahl.anbieter === "ollama" ? "ollama-modelle" : undefined}
                      onChange={(e) =>
                        setSettings({ ...settings, aufgaben: { ...settings.aufgaben, [a.id]: { ...wahl, modell: e.target.value } } })
                      }
                    />
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        <datalist id="ollama-modelle">
          {info.ollama_modelle.map((m) => (
            <option key={m} value={m} />
          ))}
        </datalist>
        <p className="meta">
          Ollama: {info.ollama_modelle.length > 0 ? info.ollama_modelle.join(", ") : "läuft nicht oder keine Modelle"}
        </p>
        {used.has("openai") && (
          <label className="field">
            Adresse OpenAI-kompatibel
            <input
              value={settings.openai_url}
              placeholder="https://…/v1/"
              onChange={(e) => setSettings({ ...settings, openai_url: e.target.value })}
            />
          </label>
        )}
        <h4>API-Schlüssel (macOS-Schlüsselbund)</h4>
        {ANBIETER.filter((x) => x.id !== "ollama").map((x) => (
          <div key={x.id} className="key-row">
            <span>{x.name}</span>
            <span className={info.schluessel[x.id] ? "ok" : "meta"}>{info.schluessel[x.id] ? "gesetzt" : "nicht gesetzt"}</span>
            <input
              type="password"
              autoComplete="off"
              placeholder={info.schluessel[x.id] ? "neuer Schlüssel" : "Schlüssel"}
              value={keys[x.id] ?? ""}
              onChange={(e) => setKeys({ ...keys, [x.id]: e.target.value })}
            />
            <button onClick={() => storeKey(x.id)} disabled={!keys[x.id]?.trim()}>
              Speichern
            </button>
            {info.schluessel[x.id] && <button onClick={() => removeKey(x.id)}>Löschen</button>}
          </div>
        ))}
        {error && <p className="warn">{error}</p>}
        <div className="dialog-buttons">
          <button onClick={onClose}>Abbrechen</button>
          <button className="primary" onClick={save}>
            Übernehmen
          </button>
        </div>
      </div>
    </div>
  );
}
