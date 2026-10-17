/** Rechts: oben feste Prüfungen (P5), unten Assistent auf Nachfrage (P6). Noch Platzhalter. */
export function RightPanel() {
  return (
    <aside className="right-panel">
      <section>
        <h2>Prüfungen</h2>
        <p className="empty">Folgt in P5: feste Prüfungen ohne KI.</p>
      </section>
      <section>
        <h2>Assistent</h2>
        <p className="empty">Folgt in P6: Fragen auf Nachfrage, mit Protokoll.</p>
      </section>
    </aside>
  );
}
