#!/bin/bash
# macOS only (3636): when Chrome's browser process holds a core for 45 s, `sample` it and its busiest
# renderer into the artifacts dir, to see where the stall spins. Run in the background; one round per stall.
out="${1:?artifacts dir}"
mkdir -p "$out"
busy=0
rounds=0
while sleep 15; do
  line=$(ps -A -o %cpu=,pid=,command= | grep 'Google Chrome for Testing' | grep -v -e '--type=' -e grep | sort -rn | head -1)
  cpu=${line%%.*}
  pid=$(awk '{print $2}' <<< "$line")
  if [ "${cpu:-0}" -ge 80 ]; then busy=$((busy + 1)); else busy=0; fi
  if [ "$busy" -ge 3 ] && [ "$rounds" -lt 3 ]; then
    rounds=$((rounds + 1))
    date -u +%T > "$out/stall-$rounds.txt"
    ps -A -o %cpu=,pid=,ppid=,command= | sort -rn | head -8 | cut -c1-200 >> "$out/stall-$rounds.txt"
    sample "$pid" 5 -file "$out/sample-browser-$rounds.txt" > /dev/null 2>&1
    rend=$(ps -A -o %cpu=,pid=,command= | grep -e '--type=renderer' | grep -v grep | sort -rn | head -1 | awk '{print $2}')
    if [ -n "$rend" ]; then sample "$rend" 5 -file "$out/sample-renderer-$rounds.txt" > /dev/null 2>&1; fi
    busy=0
  fi
done
