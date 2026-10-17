"""Prüft Text und Seitenlabels des Beispiel-PDFs (Nummerierung, Verweise, Verzeichnisse).

Aufruf: python3 tests/vorlage/inhalt_pruefen.py <pdf>
"""
import sys

import pypdf

reader = pypdf.PdfReader(sys.argv[1])
text = " ".join(page.extract_text() for page in reader.pages).replace("\xa0", " ")

# FHDW-Hinweise S. 4: Titelblatt ohne Nummer, Vorspann römisch, Inhalt arabisch ab 1.
expected_labels = ["", "i", "ii", "iii", "iv"] + [str(n) for n in range(1, len(reader.pages) - 4)]
expected_texts = [
    "Erklärung der Selbstständigkeit",
    "Kapitel 1",
    "Abschnitt 1.2",                         # Verweis auf Abschnitt
    "Kapitel 2 hergeleitet",                 # Verweis auf Kapitel
    "Abbildung 2.1: Aufbau",                 # Unterschrift
    "Abbildung 2.1 bleibt",                  # Verweis aus Kapitel 3
    "Abbildung 2.1 Aufbau",                  # Abbildungsverzeichnis
    "Tabelle 3.1: Anforderungen",
    "Tabelle A.1: Nutzung",                  # Nummerierung im Anhang
    "Anhang A",
    "Application Programming Interface (API)",  # Abkürzung beim ersten Auftreten
    "[SB18]",
    "Zugriff am 22.12.2021",
]

errors = []
if reader.page_labels != expected_labels:
    errors.append(f"Seitenlabels {reader.page_labels} statt {expected_labels}")
errors += [f"fehlt: {t!r}" for t in expected_texts if t not in text]

for e in errors:
    print("  " + e)
sys.exit(1 if errors else 0)
