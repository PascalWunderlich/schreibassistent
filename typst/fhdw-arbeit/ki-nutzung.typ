// Dokumentation der KI-Nutzung (FHDW-Hinweise S. 12, Abschnitt 2.11, und Erklärung S. i:
// „im Abschnitt ‚Struktur der Arbeit, Methodik' oder […] im Anhang“).
// Spalten wie der spätere Export aus System/ki-protokoll.jsonl (docs/DATA-FORMATS.md).
//
// Aufruf: #ai-usage(
//   ("2026-11-02", "Claude (Anthropic)", "Belegprüfung Kap. 2", "Hinweise geprüft, Text selbst überarbeitet"),
// )

#let ai-usage(caption: [Nutzung von KI-Werkzeugen], ..entries) = figure(
  kind: table,
  caption: caption,
  table(
    columns: (auto, auto, 1fr, 1fr),
    align: left,
    table.header[Datum][Werkzeug/Modell][Zweck][Verwendung],
    ..entries.pos().flatten(),
  ),
)
