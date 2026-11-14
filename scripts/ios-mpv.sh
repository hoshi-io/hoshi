#!/usr/bin/env bash
# Downloads MPVKit's LGPL libmpv build and flattens it into
# hoshi-app/mpv-ios/{device,simulator}/lib*.a for build.rs to link.
set -euo pipefail

MPVKIT_REF="${MPVKIT_REF:-1.1.0-n9.0.2}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/hoshi-app/mpv-ios"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

curl -fsSL "https://raw.githubusercontent.com/mpvkit/MPVKit/$MPVKIT_REF/Package.swift" -o "$WORK/Package.swift" \
  || curl -fsSL "https://raw.githubusercontent.com/mpvkit/MPVKit/main/Package.swift" -o "$WORK/Package.swift"

# LGPL set only; luajit is macOS-only and smbclient is GPL-only.
URLS=$(grep -oE 'https://github.com/[^"]+\.xcframework\.zip' "$WORK/Package.swift" \
  | grep -v -- '-GPL' | grep -viE 'smbclient|luajit' | sort -u)

rm -rf "$OUT"
mkdir -p "$OUT/device" "$OUT/simulator"

for url in $URLS; do
  zip="$WORK/$(basename "$url")"
  echo "-> $(basename "$url")"
  curl -fsSL "$url" -o "$zip"
  dir="$WORK/x/$(basename "$url" .zip)"
  mkdir -p "$dir"
  unzip -q "$zip" -d "$dir"
  xcf=$(find "$dir" -maxdepth 2 -name '*.xcframework' -type d | head -n1)

  for slice_dir in "$xcf"/*/; do
    slice=$(basename "$slice_dir")
    case "$slice" in
      ios-arm64) dest="$OUT/device" ;;
      ios-*simulator) dest="$OUT/simulator" ;;
      *) continue ;;
    esac

    bin=$(find "$slice_dir" -maxdepth 1 -name '*.a' | head -n1)
    if [ -z "$bin" ]; then
      fw=$(find "$slice_dir" -maxdepth 1 -name '*.framework' -type d | head -n1)
      [ -n "$fw" ] && bin="$fw/$(basename "$fw" .framework)"
    fi
    if [ -z "$bin" ] || [ ! -f "$bin" ]; then
      echo "   !! no binary in $slice_dir" >&2
      continue
    fi
    if ! file "$bin" | grep -q 'ar archive'; then
      echo "   !! $(basename "$bin") ($slice) is not a static archive: $(file -b "$bin")" >&2
      continue
    fi

    name=$(basename "$bin" .a)
    name="${name#lib}"
    name="${name#Lib}"
    name=$(echo "$name" | tr '[:upper:]' '[:lower:]')
    cp "$bin" "$dest/lib$name.a"
  done
done

echo "device:    $(ls "$OUT/device" | tr '\n' ' ')"
echo "simulator: $(ls "$OUT/simulator" | tr '\n' ' ')"
test -f "$OUT/device/libmpv.a" && test -f "$OUT/simulator/libmpv.a"
