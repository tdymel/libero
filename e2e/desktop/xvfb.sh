#!/usr/bin/env bash
# `xvfb-run -a` for shared boxes: Xvfb binds the first free display itself (-displayfd), so two
# runs never share one (3417, 3445, 5381). Usage: xvfb.sh [-s "<Xvfb screen args>"] <command...>
set -u

screen="0 1280x800x24"
if [ "${1:-}" = -s ]; then
    [ $# -ge 2 ] || { echo "xvfb.sh: -s needs an argument" >&2; exit 2; }
    screen=${2#-screen }
    shift 2
fi
[ $# -ge 1 ] || { echo "usage: xvfb.sh [-s \"-screen 0 WxHxD\"] <command...>" >&2; exit 2; }

fifo=$(mktemp -u)
mkfifo "$fifo"
# shellcheck disable=SC2086
Xvfb -displayfd 3 -screen $screen -nolisten tcp 3>"$fifo" &
server=$!
trap 'kill $server 2>/dev/null; rm -f "$fifo"' EXIT
trap 'exit 143' TERM INT

read -r number <"$fifo"
[ -n "$number" ] || { echo "xvfb.sh: Xvfb named no display" >&2; exit 1; }

DISPLAY=":$number" "$@"
