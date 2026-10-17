// Deutsches Datum im Langformat, z. B. „25. September 2026“
// (Format wie auf Titelblatt und Erklärung der FHDW-Vorlage, PDF S. 1 und S. i).

#let months-de = (
  "Januar", "Februar", "März", "April", "Mai", "Juni",
  "Juli", "August", "September", "Oktober", "November", "Dezember",
)

#let long-date-de(date) = {
  str(date.day()) + ". " + months-de.at(date.month() - 1) + " " + str(date.year())
}
