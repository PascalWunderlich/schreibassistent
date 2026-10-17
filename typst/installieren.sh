#!/bin/sh
# Macht typst/fhdw-arbeit als @local/fhdw-arbeit:<version> verfügbar (Symlink, kein Kopieren).
# Danach in jeder Arbeit: #import "@local/fhdw-arbeit:0.1.0": *
set -eu
cd "$(dirname "$0")/fhdw-arbeit"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' typst.toml)
case "$(uname)" in
  Darwin) data="$HOME/Library/Application Support" ;;
  *)      data="${XDG_DATA_HOME:-$HOME/.local/share}" ;;
esac
ziel="$data/typst/packages/local/fhdw-arbeit"
mkdir -p "$ziel"
ln -sfn "$(pwd)" "$ziel/$version"
echo "@local/fhdw-arbeit:$version -> $(pwd)"
