#!/bin/bash
# Chrome for Testing's headless shell for the macOS web legs (3636): no AppKit app, so no
# key-equivalent loop to stall CDP. Prints `chrome-path` to $GITHUB_OUTPUT.
set -euo pipefail
version="${1:?Chrome version}"
dir="$RUNNER_TEMP/headless-shell"
curl -fsSL --retry 3 -o "$dir.zip" \
  "https://storage.googleapis.com/chrome-for-testing-public/$version/mac-arm64/chrome-headless-shell-mac-arm64.zip"
unzip -q "$dir.zip" -d "$dir"
echo "chrome-path=$dir/chrome-headless-shell-mac-arm64/chrome-headless-shell" >> "$GITHUB_OUTPUT"
