# /// script
# requires-python = ">=3.11,<3.14"
# dependencies = ["docling>=2.50", "pypdf>=5"]
# ///
"""PDF → Abschnitte mit Seite (P7). Aufruf: uv run extract.py <pdf> <ausgabe.json>   (oder --vorbereiten: Erstinstallation)

Ausgabe:
  {"seiten": n, "labels": ["i", "ii", "1", …],   # gedruckte Seitenzahl je PDF-Seite
   "labels_quelle": "gedruckt",                   # gedruckt (Kopf-/Fußzeile) | pdf (/PageLabels) | pdf-seite
   "abschnitte": [{"seite": 3, "label": "1", "ueberschrift": "Einleitung", "text": "…"}],
   "docling": "<version>"}
`seite` ist die PDF-Seite ab 1 (für das Öffnen), `label` die gedruckte Seite (für Zitate).
Viele PDFs haben keine oder falsche /PageLabels (FODA: Label „7“, gedruckt „2“); deshalb
zählen zuerst die Zahlen in Kopf- und Fußzeilen.
"""

import json
import os
import re
import sys
from collections import Counter
from importlib.metadata import version

ROMAN = {"i": 1, "v": 5, "x": 10, "l": 50, "c": 100}


def roman_value(token: str) -> int | None:
    token = token.lower()
    if not token or any(ch not in ROMAN for ch in token):
        return None
    total = 0
    for a, b in zip(token, token[1:] + " "):
        value = ROMAN[a]
        total += -value if b in ROMAN and ROMAN[b] > value else value
    return total


def to_roman(n: int) -> str:
    out = ""
    for value, sym in ((100, "c"), (90, "xc"), (50, "l"), (40, "xl"), (10, "x"), (9, "ix"), (5, "v"), (4, "iv"), (1, "i")):
        while n >= value:
            out += sym
            n -= value
    return out


def printed_number(texts: list[str]) -> tuple[str, int] | None:
    """Seitenzahl aus Kopf-/Fußzeilen: erstes oder letztes Wort, arabisch oder römisch."""
    # Zuerst Kopf-/Fußzeilen, die nur aus der Zahl bestehen (sonst gewinnt z. B. „26 Nov 2018“).
    texts = sorted(texts, key=lambda t: len(t.split()) != 1)
    for text in texts:
        words = text.split()
        for word in (words[:1] + words[-1:]) if words else []:
            word = word.strip("-–—.,()[]")
            if re.fullmatch(r"\d{1,4}", word):
                return "arabisch", int(word)
            value = roman_value(word)
            if value and len(word) <= 6:
                return "roemisch", value
    return None


def printed_labels(margins: dict[int, list[str]], count: int) -> list[str] | None:
    """Gedruckte Seitenzahl je PDF-Seite aus Kopf-/Fußzeilen. Ausreißer (Versatz weicht von
    beiden Nachbarn ab, z. B. eine Jahreszahl) fallen weg; Lücken erben den Versatz der
    Nachbarn (beidseitig gleich, am Anfang/Ende höchstens 3 Seiten weit; sonst „PDF n“)."""
    found = {}
    for page, texts in margins.items():
        number = printed_number(texts)
        if number:
            found[page] = (number[0], number[1] - page)  # (Zählweise, Versatz)
    pages = sorted(found)
    outliers = set()
    for i, page in enumerate(pages):
        near = [found[p] for p in pages[max(0, i - 2) : i] + pages[i + 1 : i + 3]]
        common = Counter(near).most_common(1)
        if common and common[0][1] >= 2 and found[page] not in near:
            outliers.add(page)
    for page in outliers:
        del found[page]
    if len(found) < max(2, count // 5):
        return None
    pages = sorted(found)

    def label(kind: str, n: int) -> str | None:
        if n < 1:
            return None
        return to_roman(n) if kind == "roemisch" else str(n)

    labels = []
    for page in range(1, count + 1):
        if page in found:
            kind, offset = found[page]
            labels.append(label(kind, page + offset))
            continue
        before = max((p for p in pages if p < page), default=None)
        after = min((p for p in pages if p > page), default=None)
        guess = None
        if before and after and found[before] == found[after]:
            guess = found[before]
        elif before is None and after and after - page <= 3:
            guess = found[after]
        elif after is None and before and page - before <= 3:
            guess = found[before]
        labels.append(label(*(guess[0], page + guess[1])) if guess else None)
    return [l if l is not None else f"PDF {i + 1}" for i, l in enumerate(labels)]


def page_labels(path: str, count: int) -> tuple[list[str], bool]:
    try:
        from pypdf import PdfReader

        reader = PdfReader(path)
        labels = list(reader.page_labels)
        # pypdf liefert ohne /PageLabels einfach "1", "2", … – dann gilt: nicht aus dem PDF.
        has = "/PageLabels" in reader.trailer["/Root"]
        if len(labels) == count:
            return labels, bool(has)
    except Exception as e:  # noqa: BLE001 – Labels sind optional
        print(f"Seitenlabels nicht lesbar: {e}", file=sys.stderr)
    return [str(i + 1) for i in range(count)], False


def prepare() -> None:
    """Erstinstallation: Docling importieren und das Layout-Modell laden (App, Schritt „Einrichten“)."""
    print("Docling wird geladen …", file=sys.stderr, flush=True)
    import docling  # noqa: F401  (stellt sicher, dass alle Pakete da sind)

    try:
        from docling.utils.model_downloader import download_models
    except ImportError:
        print("Hinweis: Modelle werden beim ersten Einlesen geladen.", file=sys.stderr, flush=True)
        return
    print("Layout-Modell wird geladen …", file=sys.stderr, flush=True)
    try:
        download_models(with_layout=True, with_tableformer=False, with_code_formula=False, with_picture_classifier=False, with_rapidocr=False, progress=False)
    except TypeError:
        download_models()
    print("Fertig.", file=sys.stderr, flush=True)


def main() -> None:
    if len(sys.argv) == 2 and sys.argv[1] == "--vorbereiten":
        prepare()
        return
    if len(sys.argv) != 3:
        sys.exit("Aufruf: extract.py <pdf> <ausgabe.json> | extract.py --vorbereiten")
    pdf, out = sys.argv[1], sys.argv[2]

    from docling.datamodel.base_models import InputFormat
    from docling.datamodel.pipeline_options import PdfPipelineOptions
    from docling.document_converter import DocumentConverter, PdfFormatOption
    from docling_core.types.doc import ContentLayer, SectionHeaderItem, TextItem, TitleItem

    options = PdfPipelineOptions(do_ocr=False, do_table_structure=False, generate_page_images=False)
    converter = DocumentConverter(format_options={InputFormat.PDF: PdfFormatOption(pipeline_options=options)})
    doc = converter.convert(pdf).document

    count = doc.num_pages()
    heading = ""
    sections = []
    margins: dict[int, list[str]] = {}
    for item, _level in doc.iterate_items(included_content_layers={*ContentLayer}):
        if not isinstance(item, TextItem) or not item.prov:
            continue
        text = " ".join(item.text.split())
        if not text:
            continue
        page = item.prov[0].page_no
        if item.label in ("page_header", "page_footer"):
            margins.setdefault(page, []).append(text)
            continue
        if item.content_layer != ContentLayer.BODY:
            continue
        if isinstance(item, (SectionHeaderItem, TitleItem)):
            heading = text
            continue
        sections.append({"seite": page, "ueberschrift": heading, "text": text})

    # Gedruckte Seitenzahl: aus Kopf-/Fußzeilen, sonst aus /PageLabels, sonst PDF-Seite.
    if os.environ.get("SA_SEITEN_DEBUG"):
        for page in sorted(margins):
            print(page, printed_number(margins[page]), margins[page], file=sys.stderr)
    labels = printed_labels(margins, count)
    source = "gedruckt"
    if labels is None:
        labels, from_pdf = page_labels(pdf, count)
        trivial = labels == [str(i + 1) for i in range(count)]
        source = "pdf" if from_pdf and not trivial else "pdf-seite"
    for section in sections:
        page = section["seite"]
        section["label"] = labels[page - 1] if 0 < page <= len(labels) else str(page)

    result = {"seiten": count, "labels": labels, "labels_quelle": source, "abschnitte": sections, "docling": version("docling")}
    with open(out, "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False)
    print(f"{len(sections)} Abschnitte auf {count} Seiten")


if __name__ == "__main__":
    main()
