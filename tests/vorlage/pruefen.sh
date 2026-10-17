#!/bin/sh
# Kompiliert die Beispielarbeit und die Vorlagentests; Abbruch beim ersten Fehler.
# Aufruf aus dem Repo-Root: tests/vorlage/pruefen.sh
set -u
cd "$(dirname "$0")/../.."
typst/installieren.sh >/dev/null
aus=$(mktemp -d)
fehler=0

erwarte_ok() {
  if typst compile --root "$1" "$2" "$aus/x.pdf" 2>"$aus/log"; then
    echo "ok      $2"
  else
    echo "FEHLER  $2"; cat "$aus/log"; fehler=1
  fi
}

erwarte_abbruch() {
  if typst compile --root "$1" "$2" "$aus/x.pdf" 2>"$aus/log"; then
    echo "FEHLER  $2 (hätte abbrechen müssen)"; fehler=1
  elif grep -q "$3" "$aus/log"; then
    echo "ok      $2 (bricht wie erwartet ab)"
  else
    echo "FEHLER  $2 (falsche Meldung)"; cat "$aus/log"; fehler=1
  fi
}

erwarte_ok beispiel-arbeit beispiel-arbeit/Arbeit/main.typ
if python3 tests/vorlage/inhalt_pruefen.py "$aus/x.pdf"; then
  echo "ok      Beispiel-PDF: Seitenzahlen, Verweise, Verzeichnisse"
else
  echo "FEHLER  Beispiel-PDF: Inhalt"; fehler=1
fi
erwarte_ok beispiel-arbeit beispiel-arbeit/Arbeit/erklaerung-einzeln.typ
erwarte_ok . tests/vorlage/bachelor-voll.typ
erwarte_abbruch . tests/vorlage/bachelor-ohne-zweitpruefer.typ "zweitpruefer"

rm -rf "$aus"
exit $fehler
