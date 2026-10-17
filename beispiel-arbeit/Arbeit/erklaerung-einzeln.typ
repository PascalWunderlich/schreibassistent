// Erzeugt nur die Erklärung als eigenes PDF zum Unterschreiben und Einscannen
// (FHDW-Hinweise S. 25: separate Datei bei der Abgabe).
// typst compile --root beispiel-arbeit beispiel-arbeit/Arbeit/erklaerung-einzeln.typ
#import "@local/fhdw-arbeit:0.1.0": erklaerung-dokument
#import "metadaten.typ": arbeit

#erklaerung-dokument(..arbeit)
