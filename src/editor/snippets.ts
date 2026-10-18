/** Typst-Bausteine für die Einfügeleiste. Reine Funktionen: Eingabe → Text, `select` markiert die erste auszufüllende Stelle. */
export interface Snippet {
  text: string;
  /** Bereich (UTF-16) im eingefügten Text, der nach dem Einfügen markiert wird. */
  select?: [number, number];
}

const block = (text: string, placeholder?: string): Snippet => {
  const at = placeholder ? text.indexOf(placeholder) : -1;
  return { text, select: at >= 0 ? [at, at + placeholder!.length] : undefined };
};

/** Label aus einer Beschriftung: „Aufbau des Systems“ → „aufbau-des-systems“. */
export function slug(text: string): string {
  return (
    text
      .toLowerCase()
      .replace(/ä/g, "ae")
      .replace(/ö/g, "oe")
      .replace(/ü/g, "ue")
      .replace(/ß/g, "ss")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "ohne-titel"
  );
}

export const heading = (level: 1 | 2 | 3): Snippet => block(`${"=".repeat(level)} Überschrift`, "Überschrift");
export const bulletList = (): Snippet => block("- Punkt\n- Punkt", "Punkt");
export const numberedList = (): Snippet => block("+ Schritt\n+ Schritt", "Schritt");
export const quote = (): Snippet => block('#quote(block: true, attribution: [@quelle])[Zitat]', "Zitat");
export const footnote = (): Snippet => block("#footnote[Text der Fußnote]", "Text der Fußnote");
export const equation = (): Snippet => block("$ a^2 + b^2 = c^2 $", "a^2 + b^2 = c^2");

export function cite(citekey: string, page?: string): Snippet {
  const text = page ? `@${citekey}[S. ${page}]` : `@${citekey}`;
  return { text };
}

export function crossref(label: string): Snippet {
  return { text: `@${label}` };
}

/** Abbildung mit Beschriftung und Label (`abb:`); `path` relativ zur Kapiteldatei. */
export function figure(path: string, caption: string): Snippet {
  return {
    text: `#figure(\n  image("${path}", width: 80%),\n  caption: [${caption}],\n) <abb:${slug(caption)}>`,
  };
}

/** Tabelle mit Kopfzeile, Beschriftung und Label (`tab:`). */
export function table(rows: number, cols: number, caption: string): Snippet {
  const r = Math.max(1, Math.floor(rows));
  const c = Math.max(1, Math.floor(cols));
  const cell = (head: boolean) => `[${head ? "Spalte" : "…"}]`;
  const line = (head: boolean) => "    " + Array.from({ length: c }, () => cell(head)).join(", ") + ",";
  const body = Array.from({ length: r }, () => line(false)).join("\n");
  const header = `    table.header(${Array.from({ length: c }, () => "[Spalte]").join(", ")}),`;
  return {
    text: `#figure(\n  table(\n    columns: ${c},\n${header}\n${body}\n  ),\n  caption: [${caption}],\n) <tab:${slug(caption)}>`,
  };
}

export function listing(language: string, caption: string): Snippet {
  return {
    text: `#figure(\n  \`\`\`${language}\n  code\n  \`\`\`,\n  caption: [${caption}],\n) <lst:${slug(caption)}>`,
  };
}

export type ChartKind = "balken" | "linie";

const nice = (n: number) => Math.round(n * 10) / 10;

/** Diagramm aus Typst-Bordmitteln (ohne Pakete, funktioniert offline). Werte als Beispieldaten. */
export function chart(kind: ChartKind, caption: string, data: [string, number][] = [["A", 3], ["B", 5], ["C", 2]]): Snippet {
  const max = Math.max(...data.map(([, v]) => v), 1);
  const label = `<abb:${slug(caption)}>`;
  let body: string;
  if (kind === "balken") {
    const bars = data
      .map(([name, v]) => `    stack(dir: ttb, spacing: 4pt, align(center + bottom, box(width: 24pt, height: ${nice((v / max) * 100)}pt, fill: luma(60%))), [${name}]),`)
      .join("\n");
    body = `  grid(columns: ${data.length}, column-gutter: 16pt, align: bottom,\n${bars}\n  )`;
  } else {
    const step = 60;
    const pts = data.map(([, v], i) => `curve.${i === 0 ? "move" : "line"}((${i * step}pt, ${nice(100 - (v / max) * 100)}pt))`).join(", ");
    const labels = data.map(([name], i) => `    place(dx: ${i * step - 4}pt, dy: 106pt, [${name}])`).join("\n");
    body = `  block(width: ${(data.length - 1) * step + 20}pt, height: 130pt, {\n    place(curve(stroke: 1.5pt, ${pts}))\n${labels}\n  })`;
  }
  return { text: `#figure(\n${body},\n  caption: [${caption}],\n) ${label}`, select: undefined };
}
