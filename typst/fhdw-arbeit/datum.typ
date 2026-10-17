// Deutsches Datum im Langformat, z. B. „25. September 2026“
// (Format wie auf Titelblatt und Erklärung der FHDW-Vorlage, PDF S. 1 und S. i).

#let monate = (
  "Januar", "Februar", "März", "April", "Mai", "Juni",
  "Juli", "August", "September", "Oktober", "November", "Dezember",
)

#let datum-lang(datum) = {
  str(datum.day()) + ". " + monate.at(datum.month() - 1) + " " + str(datum.year())
}
