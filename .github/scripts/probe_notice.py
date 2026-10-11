#!/usr/bin/env python3
"""Turns the platform probe's answers into check-run notices, readable through the API without auth.

    probe_notice.py <artifacts dir>

A short `::notice` (origin, secure context, capability APIs) and the full one per
`platform-probe-*.json`, and one for all `outcomes.log` lines under the dir
(`e2e/tests/all/platform_probe.rs`, `e2e::driver::record_outcome`), and the macOS desktop driver's `calibration.log`. Nothing found says so.
"""
import json
import sys
from pathlib import Path

from ci_annotate import clip, escape


def secure(text: str) -> str:
    """The origin, `isSecureContext` and the capability APIs' types: the answer to todo 3231 first."""
    try:
        facts = json.loads(text)
    except ValueError:
        return "unreadable probe file"
    types = facts.get("typeof", {})
    apis = " ".join(f"{name}={types.get(name)}" for name in ("mediaDevices", "Notification", "geolocation", "clipboard"))
    return f"origin={facts.get('origin')} isSecureContext={facts.get('isSecureContext')} {apis}"


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    root = Path(sys.argv[1])
    probes = sorted(root.rglob("platform-probe-*.json")) if root.is_dir() else []
    for probe in probes:
        text = probe.read_text(encoding="utf-8", errors="replace").strip()
        print(f"::notice title={escape(probe.stem + ' secure', True)}::{escape(secure(text))}")
        print(f"::notice title={escape(probe.stem, True)}::{escape(clip(text, 4000))}")
    outcomes = [
        line
        for log in (sorted([*root.rglob("outcomes.log"), *root.rglob("calibration.log")]) if root.is_dir() else [])
        for line in log.read_text(encoding="utf-8", errors="replace").splitlines()
        if line.strip()
    ]
    if outcomes:
        print(f"::notice title=outcomes::{escape(clip(chr(10).join(outcomes), 4000))}")
    if not probes and not outcomes:
        print(f"::notice title=platform probe::no probe file under {escape(str(root))}")


if __name__ == "__main__":
    main()
