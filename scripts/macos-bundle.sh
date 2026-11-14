#!/usr/bin/env bash
# Copies Homebrew's libmpv (and its deps) into the .app, then fixes rpaths and re-signs.
# Used by release.yml.
set -euo pipefail

APP="$1"
EXECUTABLE=$(find "$APP/Contents/MacOS" -type f | head -n 1)

dylibbundler -cd -of -b \
  -x "$EXECUTABLE" \
  -d "$APP/Contents/Frameworks" \
  -p "@executable_path/../Frameworks/"

"$(dirname "$0")/macos-fix-rpaths.sh" "$APP"
