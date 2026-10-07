#!/usr/bin/env bash
# Injects a preload link for the wasm into a built index.html.
#
# dx emits `<link rel="preload" as="script">` for the entry JS but nothing for
# the wasm, which that JS fetches from its own module body - so the wasm
# request cannot start until the JS has arrived and been parsed. The filename
# is content-hashed at build time, so the link cannot be written by hand; this
# reads the real name out of the JS the built index.html points at.
set -euo pipefail

public="${1:?usage: [BASE_PATH=<p>] preload_wasm.sh <public dir>}"
index="$public/index.html"

marker='<!-- LIBERO_WASM_PRELOAD -->'
# Not an error: dx regenerates index.html from the template on every build, so
# a missing marker just means this already ran. Failing here would break a
# chained `just` recipe for no reason.
grep -qF "$marker" "$index" || {
    echo "preload_wasm: marker already replaced in $index, nothing to do"
    exit 0
}

# The <script type="module"> dx injected, and the wasm URL inside it.
js_href=$(grep -oE '<script type="module"[^>]*src="[^"]+"' "$index" | grep -oE 'src="[^"]+"' | cut -d'"' -f2)
js_rel="${js_href#/}"
# A `dx --base-path <p>` build prefixes the URLs with `/<p>/` but keeps the files at the root.
[ -n "${BASE_PATH:-}" ] && js_rel="${js_rel#"${BASE_PATH#/}/"}"
js_file="$public/$js_rel"
js_file="${js_file//\/.\//\/}"

wasm_href=$(grep -oE '"[^"]*docs_bg[^"]*\.wasm"' "$js_file" | tr -d '"' | grep '/' | head -1)
[ -n "$wasm_href" ] || { echo "preload_wasm: no hashed wasm URL in $js_file" >&2; exit 1; }

# The preload only avoids a second download if its mode matches the fetch the
# JS actually makes. The entry JS calls a bare `fetch(url)` (mode "cors",
# credentials "same-origin"); `crossorigin` on the link means credentials
# "omit". Whether Chrome treats those as the same request for a *same-origin*
# URL is the one thing here not verified in a browser - see the README note.
# If DevTools warns the preload went unused, re-run with PRELOAD_CROSSORIGIN=0.
if [ "${PRELOAD_CROSSORIGIN:-1}" = "1" ]; then
    attrs=' crossorigin'
else
    attrs=''
fi
link="<link rel=\"preload\" as=\"fetch\" type=\"application/wasm\" href=\"$wasm_href\"$attrs>"
python3 - "$index" "$marker" "$link" <<'PY'
import sys
index, marker, link = sys.argv[1:4]
with open(index) as f:
    html = f.read()
with open(index, 'w') as f:
    f.write(html.replace(marker, link))
PY

echo "preload_wasm: injected $wasm_href"
