#!/usr/bin/env bash
# Regenerates the committed `public/og-image.png` (1200x630 link-preview card) and
# `public/favicon.png` from `assets/logo.svg`. Needs `rsvg-convert` and ImageMagick.
set -euo pipefail

docs="$(cd "$(dirname "$0")/.." && pwd)"
logo="$docs/assets/logo.svg"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

rsvg-convert --width 880 "$logo" -o "$tmp/card-logo.png"
magick -size 1200x630 xc:white "$tmp/card-logo.png" -gravity center -composite -depth 8 \
    -strip -define png:exclude-chunks=date,time "$docs/public/og-image.png"

rsvg-convert --width 192 "$logo" -o "$tmp/icon-logo.png"
magick -size 192x192 xc:none "$tmp/icon-logo.png" -gravity center -composite -depth 8 \
    -strip -define png:exclude-chunks=date,time "$docs/public/favicon.png"
