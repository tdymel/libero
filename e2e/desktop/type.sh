#!/usr/bin/env bash
# Types into the first field of a fixture page in the desktop WebView (wry) and
# reads it back through the clipboard. WebKitGTK has no DevTools socket, so this
# drives X directly: headless under xvfb-run, never on the desktop display (1026).
#
#   e2e/desktop/type.sh <target-dir> <route> <text> <delay-ms> [runs] [expected]
#   e2e/desktop/type.sh "$PWD/target/dev" /text-field/echo button 12 4
#
# Needs xvfb-run, xdotool and xclip. Exits 1 when a run reads back other than
# `expected` (default: the typed text).
set -u

if [ -z "${LSX_IN_XVFB:-}" ]; then
    [ $# -ge 4 ] || { grep '^#   e2e' "$0"; exit 2; }
    root=$(cd "$(dirname "$0")/../.." && pwd)
    (cd "$root" && RUSTC_WRAPPER=sccache cargo build -q -p e2e-fixtures --features desktop --target-dir "$1") || exit 2
    LSX_IN_XVFB=1 exec xvfb-run -a -s "-screen 0 1280x800x24" "$0" "$@"
fi

bin=$1/debug/e2e-fixtures route=$2 text=$3 delay=$4 runs=${5:-3} expected=${6:-$3}
log=$(mktemp)
E2E_ROUTE=$route "$bin" >"$log" 2>&1 &
app=$!
trap 'kill $app 2>/dev/null; rm -f "$log"' EXIT

win=$(xdotool search --sync --onlyvisible --pid $app | head -1)
# The page loads after the window maps; no ready signal crosses to X.
sleep 3
xdotool windowfocus --sync "$win"
xdotool key Tab
sleep 0.5

status=0
for i in $(seq "$runs"); do
    xdotool key ctrl+a BackSpace
    sleep 0.5
    xdotool type --delay "$delay" "$text"
    # Past the last render's round trip.
    sleep 1.5
    xdotool key ctrl+a ctrl+c
    sleep 0.3
    got=$(xclip -o -selection clipboard)
    echo "run $i: $got"
    [ "$got" = "$expected" ] || status=1
done
grep ERROR "$log"
exit $status
