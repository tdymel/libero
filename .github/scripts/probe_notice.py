#!/usr/bin/env python3
"""Turns the platform probe's answers into check-run notices, readable through the API without auth.

    probe_notice.py <artifacts dir>

One `::notice` per `platform-probe-*.json` and one for all `outcomes.log` lines under the dir
(`e2e/tests/all/platform_probe.rs`, `e2e::driver::record_outcome`). Nothing found says so.
"""
import sys
from pathlib import Path

from ci_annotate import clip, escape


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    root = Path(sys.argv[1])
    probes = sorted(root.rglob("platform-probe-*.json")) if root.is_dir() else []
    for probe in probes:
        text = probe.read_text(encoding="utf-8", errors="replace").strip()
        print(f"::notice title={escape(probe.stem, True)}::{escape(clip(text, 4000))}")
    outcomes = [
        line
        for log in (sorted(root.rglob("outcomes.log")) if root.is_dir() else [])
        for line in log.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    ]
    if outcomes:
        print(f"::notice title=outcomes::{escape(clip(chr(10).join(outcomes), 4000))}")
    if not probes and not outcomes:
        print(f"::notice title=platform probe::no probe file under {escape(str(root))}")


if __name__ == "__main__":
    main()
