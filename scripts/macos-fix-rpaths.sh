#!/usr/bin/env bash
# dylibbundler re-adds @executable_path/../Frameworks/ even when it's already there, and
# recent dyld refuses to load a binary with duplicate LC_RPATH entries. Drop the extras
# from the executable and every bundled dylib, then re-sign.
set -euo pipefail

APP="$1"

rpaths() {
  otool -l "$1" | awk '$1 == "cmd" && $2 == "LC_RPATH" { getline; getline; print $2 }'
}

dedupe() {
  local f="$1"
  rpaths "$f" | sort | uniq -c | while read -r count path; do
    for ((i = 1; i < count; i++)); do
      install_name_tool -delete_rpath "$path" "$f"
      echo "removed duplicate rpath $path from $(basename "$f")"
    done
  done
}

for f in "$APP"/Contents/MacOS/* "$APP"/Contents/Frameworks/*.dylib; do
  [ -f "$f" ] && dedupe "$f"
done

for f in "$APP"/Contents/MacOS/* "$APP"/Contents/Frameworks/*.dylib; do
  [ -f "$f" ] || continue
  if [ -n "$(rpaths "$f" | sort | uniq -d)" ]; then
    echo "still has duplicate rpaths: $f" >&2
    exit 1
  fi
done

codesign --force --deep -s - "$APP"
