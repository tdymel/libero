#!/usr/bin/env python3
"""Splits the e2e tests into balanced CI shards, keeping each unit (`tests/all/<unit>`) whole.

    shard.py units <index> <total>          `unit::` filters for `cargo run -p e2e`, from main.rs
    shard.py smoke                           the macOS web leg's units, where a Mac differs (3444)

A unit's weight is its `#[test]` count; units go greedily to the lightest shard, so
every unit lands in exactly one shard and the split is stable for the same sources. The
performance units (`perf`, `table_perf`) land in none unless `E2E_PERF` is set.
"""
import os
import re
import sys
from pathlib import Path

ALL = Path(__file__).resolve().parents[2] / "e2e/tests/all"
# Performance scenarios never run in a shard (todo 3454); `E2E_PERF=1` brings them back.
PERF = {"perf", "table_perf"}
# Editing, IME and pickers: the whole tier wedges headless Chrome on the 3-core macOS runner.
# Out: `use_hotkeys` and `rich_text_editor_input`, whose Cmd chords lose their keyup and stall Chrome.
SMOKE = [
    "chrono_field", "editor_ime_probe", "select", "shortcut_help",
    "spotlight", "tags_field", "text_field", "theme_switcher",
]


def unit_weights() -> dict[str, int]:
    mods = re.findall(r"^mod (?:r#)?(\w+);", (ALL / "main.rs").read_text(encoding="utf-8"), re.M)
    weights = {}
    for unit in mods:
        if unit in PERF and "E2E_PERF" not in os.environ:
            continue
        files = [p for p in [ALL / f"{unit}.rs"] if p.is_file()] + list((ALL / unit).rglob("*.rs"))
        weights[unit] = max(1, sum(p.read_text(encoding="utf-8").count("#[test") for p in files))
    return weights


def assign(weights: dict[str, int], total: int) -> list[list[str]]:
    shards = [[] for _ in range(total)]
    load = [0] * total
    for unit, weight in sorted(weights.items(), key=lambda item: (-item[1], item[0])):
        lightest = load.index(min(load))
        shards[lightest].append(unit)
        load[lightest] += weight
    return [sorted(shard) for shard in shards]


def main() -> None:
    if sys.argv[1:] == ["smoke"]:
        missing = [unit for unit in SMOKE if unit not in unit_weights()]
        if missing:
            sys.exit(f"smoke units not in main.rs: {', '.join(missing)}")
        print(" ".join(f"{unit}::" for unit in SMOKE))
        return
    mode, index, total = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    if not 1 <= index <= total:
        sys.exit(f"shard {index} is not within 1..{total}")
    if mode == "units":
        print(" ".join(f"{unit}::" for unit in assign(unit_weights(), total)[index - 1]))
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
