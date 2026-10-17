# tests/

Übergreifende Tests, die mehrere Module oder die ganze App betreffen
(z. B. Prüfungen gegen die absichtlichen Fehlerfälle in `beispiel-arbeit/`,
Ende-zu-Ende-Tests der Oberfläche). Unit-Tests liegen direkt bei den Modulen
in `src-tauri/src/<modul>` bzw. neben den Komponenten in `src/`.

## vorlage/

`pruefen.sh` kompiliert die Beispielarbeit, die einzelne Erklärung und Varianten der
Vorlage (Bachelorarbeit mit allen Optionen, numerischer Zitierstil) und erwartet bei
einer Bachelorarbeit ohne Zweitprüfer einen Abbruch. `inhalt_pruefen.py` prüft im
Beispiel-PDF Seitenzahlen, Verweise und Verzeichnisse.
