import { Snippet } from "./snippets";

/** Der Editor, in dem gerade geschrieben wird. Die Einfügeleiste schreibt nur dorthin, und nur auf Handlung des Menschen. */
export interface Inserter {
  /** Datei, in die eingefügt wird (relativ zum Vault, z. B. „Arbeit/kapitel/01-einleitung.typ“). */
  file: string;
  insert: (snippet: Snippet) => void;
}

let active: Inserter | null = null;
const listeners = new Set<() => void>();

export function setInserter(next: Inserter | null) {
  active = next;
  listeners.forEach((l) => l());
}

/** Nur abmelden, wenn der Editor noch der aktive ist (ein neuer Editor kann schon registriert sein). */
export function clearInserter(current: Inserter) {
  if (active === current) setInserter(null);
}

export const getInserter = () => active;
export function subscribeInserter(listener: () => void) {
  listeners.add(listener);
  return () => void listeners.delete(listener);
}

/** Relativer Pfad von der Datei `from` zum Vault-Pfad `to` (beide ohne führenden Schrägstrich). */
export function relativePath(from: string, to: string): string {
  const a = from.split("/").slice(0, -1);
  const b = to.split("/");
  let i = 0;
  while (i < a.length && i < b.length - 1 && a[i] === b[i]) i++;
  return [...Array(a.length - i).fill(".."), ...b.slice(i)].join("/");
}
