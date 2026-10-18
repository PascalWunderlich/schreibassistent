#!/usr/bin/env python3
"""Kontrast der Design-Tokens prüfen (WCAG AA: Text 4,5:1). Aufruf: python3 scripts/kontrast.py"""
import re
import sys
from pathlib import Path

CSS = Path(__file__).resolve().parent.parent / "src/styles/tokens.css"

# (Text, Fläche): Paare, die in der Oberfläche vorkommen.
PAIRS = [
    ("foreground", "background"), ("foreground", "card"), ("muted-foreground", "background"),
    ("muted-foreground", "card"), ("muted-foreground", "muted"), ("primary-foreground", "primary"),
    ("primary", "background"), ("primary", "card"), ("accent-foreground", "accent"),
    ("destructive", "background"), ("destructive", "card"), ("destructive-foreground", "destructive"),
    ("success", "background"), ("warning", "background"), ("warning", "card"),
    ("secondary-foreground", "secondary"), ("sidebar-accent-foreground", "sidebar-accent"),
] + [(f"hint-{k}-foreground", f"hint-{k}") for k in ("form", "stil", "quelle", "sprache", "ki", "ok")]


def blocks(text: str) -> dict[str, dict[str, str]]:
    out = {}
    for name, body in re.findall(r"(:root|\.dark)\s*\{(.*?)\n\}", text, re.S):
        out[name] = dict(re.findall(r"--([\w-]+):\s*(#[0-9a-fA-F]{6})", body))
    return out


def luminance(hex_color: str) -> float:
    def channel(c: int) -> float:
        c = c / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (int(hex_color[i : i + 2], 16) for i in (1, 3, 5))
    return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)


def ratio(a: str, b: str) -> float:
    la, lb = sorted((luminance(a), luminance(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


def main() -> int:
    failed = 0
    for mode, tokens in blocks(CSS.read_text()).items():
        print(f"== {mode}")
        for text, surface in PAIRS:
            r = ratio(tokens[text], tokens[surface])
            ok = r >= 4.5
            failed += not ok
            print(f"  {'ok ' if ok else 'ZU WENIG'} {r:5.2f}  {text} auf {surface}")
    print("Alle Paare erfüllen WCAG AA." if not failed else f"{failed} Paare unter 4,5:1.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
