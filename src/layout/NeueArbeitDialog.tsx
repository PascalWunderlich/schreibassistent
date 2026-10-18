import { useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/ui/button";
import { NeueArbeit } from "../editor/api";
import { slug } from "../editor/snippets";

interface Props {
  /** Legt die Arbeit an und öffnet sie. */
  onCreate: (folder: string, input: NeueArbeit) => Promise<void>;
  onClose: () => void;
}

const field = "h-8 w-full rounded-md border border-input bg-background px-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring";

/** Neue Arbeit: Titel, Autor, Art, Ort. Es entsteht ein neuer Ordner mit dem Titel im gewählten Ort. */
export function NeueArbeitDialog({ onCreate, onClose }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => ref.current?.showModal(), []);
  const [title, setTitle] = useState("");
  const [author, setAuthor] = useState("");
  const [type, setType] = useState<NeueArbeit["thesis_type"]>("Praxisarbeit");
  const [parent, setParent] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function choose() {
    const path = await open({ directory: true, title: "Wo soll die Arbeit liegen?" });
    if (typeof path === "string") setParent(path);
  }

  async function submit() {
    if (!title.trim()) return setError("Bitte einen Titel angeben.");
    if (!parent) return setError("Bitte einen Ort wählen.");
    setBusy(true);
    setError(null);
    try {
      await onCreate(`${parent}/${slug(title)}`, { title: title.trim(), author: author.trim(), thesis_type: type });
      ref.current?.close();
    } catch (e) {
      setError(Array.isArray(e) ? e.join(" ") : String(e));
      setBusy(false);
    }
  }

  return (
    <dialog
      ref={ref}
      onClose={onClose}
      className="m-auto w-[420px] rounded-xl border border-border bg-popover p-0 text-popover-foreground shadow-lg backdrop:bg-black/30"
    >
      <form
        className="flex flex-col gap-3 p-4"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <h2 className="text-base font-semibold">Neue Arbeit</h2>
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          Titel
          <input className={field} autoFocus value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Titel wie im FHDW-Portal angemeldet" />
        </label>
        <label className="flex flex-col gap-1 text-xs text-muted-foreground">
          Autor
          <input className={field} value={author} onChange={(e) => setAuthor(e.target.value)} placeholder="Vorname Nachname" />
        </label>
        <div role="radiogroup" aria-label="Art der Arbeit" className="flex gap-2">
          {(["Praxisarbeit", "Bachelorarbeit"] as const).map((t) => (
            <Button key={t} type="button" role="radio" aria-checked={type === t} size="sm" variant={type === t ? "secondary" : "outline"} onClick={() => setType(t)}>
              {t}
            </Button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <Button type="button" variant="outline" size="sm" onClick={choose}>
            Ort wählen …
          </Button>
          <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground">
            {parent ? `${parent}/${title.trim() ? slug(title) : "…"}` : "noch kein Ort gewählt"}
          </span>
        </div>
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={() => ref.current?.close()}>
            Abbrechen
          </Button>
          <Button type="submit" size="sm" disabled={busy}>
            Anlegen
          </Button>
        </div>
      </form>
    </dialog>
  );
}
