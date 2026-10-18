#!/bin/sh
# Holt die Werkzeuge, die die App mitliefert, nach build/werkzeuge/ (macOS arm64, Linux x64).
# Windows: scripts/werkzeuge-bauen.ps1. Versionen und Prüfsummen: werkzeuge.lock.
#
# Aufruf: scripts/werkzeuge-bauen.sh [macos-arm64|linux-x64]
# Ergebnis (nicht im Repo, siehe .gitignore):
#   build/werkzeuge/bin/{tinymist,vale,uv,typst2vast}
#   build/werkzeuge/ltex-ls-plus/        LTeX+ mit eigener Java-Laufzeit
#   build/werkzeuge/vale/                Konfiguration und Stil FHDW
#   build/werkzeuge/typst/fhdw-arbeit/   Vorlage (wird beim Start installiert)
#   build/werkzeuge/licenses/            Lizenztexte
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
lock="$root/werkzeuge.lock"
out="$root/build/werkzeuge"
cache="$root/build/cache"

target=${1:-}
if [ -z "$target" ]; then
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) target=macos-arm64 ;;
    Linux-x86_64) target=linux-x64 ;;
    *) echo "Unbekanntes System $(uname -s)-$(uname -m); Ziel als Argument angeben." >&2; exit 1 ;;
  esac
fi
case "$target" in macos-arm64|linux-x64) ;; *) echo "Ziel '$target' nicht unterstützt (macos-arm64, linux-x64)." >&2; exit 1 ;; esac

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# Datei laden (falls nicht im Cache) und gegen die Prüfsumme aus werkzeuge.lock prüfen.
fetch() { # name url sha256 -> Pfad im Cache
  file="$cache/$1-$target-$(basename "$2")"
  if [ ! -f "$file" ]; then
    echo "Lade $1 ..." >&2
    curl -fsSL --retry 3 -o "$file.part" "$2"
    mv "$file.part" "$file"
  fi
  actual=$(sha256 "$file")
  if [ "$actual" != "$3" ]; then
    rm -f "$file"
    echo "Prüfsumme von $1 stimmt nicht (erwartet $3, erhalten $actual)." >&2
    exit 1
  fi
  echo "$file"
}

rm -rf "$out"
mkdir -p "$out/bin" "$out/licenses" "$cache"

while read -r kind name tgt sha url; do
  case "$kind" in ''|'#'*) continue ;; esac
  [ "$kind" = "download" ] && [ "$tgt" = "$target" ] || continue
  file=$(fetch "$name" "$url" "$sha")
  tmp="$cache/entpackt-$name"
  rm -rf "$tmp"; mkdir -p "$tmp"
  case "$name" in
    tinymist)
      cp "$file" "$out/bin/tinymist"; chmod +x "$out/bin/tinymist" ;;
    vale)
      tar -xzf "$file" -C "$tmp"; cp "$tmp/vale" "$out/bin/vale" ;;
    uv)
      tar -xzf "$file" -C "$tmp"; cp "$tmp"/*/uv "$out/bin/uv" ;;
    ltex-ls-plus)
      tar -xzf "$file" -C "$tmp"
      mkdir -p "$out/ltex-ls-plus"; cp -R "$tmp"/*/. "$out/ltex-ls-plus/"
      # Hunspell für JNA auspacken; die App setzt jna.library.path darauf (siehe pruefung/ltex.rs).
      case "$target" in macos-arm64) plat=darwin-aarch64 ;; linux-x64) plat=linux-x86-64 ;; esac
      mkdir -p "$out/ltex-ls-plus/native"
      unzip -j -o -q "$out"/ltex-ls-plus/lib/hunspell-*.jar "$plat/*" -d "$out/ltex-ls-plus/native" ;;
  esac
  # Lizenztexte aus dem Paket beilegen.
  mkdir -p "$out/licenses/$name"
  find "$tmp" -maxdepth 3 -type f \( -iname 'LICENSE*' -o -iname 'NOTICE*' -o -iname 'COPYING*' \) \
    -exec cp {} "$out/licenses/$name/" \; 2>/dev/null || true
  rm -rf "$tmp"
done < "$lock"

# Typst-Pakete, die die Vorlage braucht (die App lädt selbst keine Pakete nach).
while read -r kind spec version sha url; do
  [ "$kind" = "package" ] || continue
  name=$(echo "$spec" | tr '/' '-')
  file="$cache/$name-$version.tar.gz"
  [ -f "$file" ] || curl -fsSL --retry 3 -o "$file" "$url"
  actual=$(sha256 "$file")
  if [ "$sha" = "-" ]; then
    echo "Hinweis: Für $spec $version ist keine Prüfsumme festgelegt. Berechnet: $actual (in werkzeuge.lock eintragen)." >&2
  elif [ "$actual" != "$sha" ]; then
    rm -f "$file"; echo "Prüfsumme von $spec stimmt nicht (erwartet $sha, erhalten $actual)." >&2; exit 1
  fi
  dest="$out/typst/packages/$spec/$version"
  mkdir -p "$dest"
  tar -xzf "$file" -C "$dest"
done < "$lock"

# typst2vast (Typst → Text für Vale): aus dem Quelltext bauen.
crate_version=$(awk '$1 == "crate" && $2 == "typst2vast" { print $3 }' "$lock")
echo "Baue typst2vast $crate_version ..." >&2
cargo install typst2vast --version "$crate_version" --locked --root "$cache/typst2vast"
cp "$cache/typst2vast/bin/typst2vast" "$out/bin/typst2vast"

# Konfiguration, Vorlage, Lizenzen, Versionen.
mkdir -p "$out/vale" "$out/typst"
cp -R "$root/sidecars/vale/." "$out/vale/"
# Ohne Punkt am Namen, damit kein Paketierer die „versteckte“ Datei auslässt.
mv "$out/vale/.vale.ini" "$out/vale/vale.ini"
cp -R "$root/typst/fhdw-arbeit" "$out/typst/fhdw-arbeit"
cp "$root/docs/werkzeuge-lizenzen.md" "$out/licenses/README.md"
cp "$lock" "$out/VERSIONEN.txt"

# macOS (arm64): Programme brauchen eine Signatur, sonst startet das System sie nicht.
if [ "$target" = "macos-arm64" ] && command -v codesign >/dev/null 2>&1; then
  find "$out" -type f | while read -r f; do
    if file "$f" | grep -q 'Mach-O'; then codesign --force --sign - "$f" >/dev/null 2>&1 || true; fi
  done
fi

echo "Fertig: $out" >&2
