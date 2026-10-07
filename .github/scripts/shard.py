#!/usr/bin/env python3
"""Splits the e2e tests into balanced CI shards, keeping each unit (`tests/all/<unit>`) whole.

    shard.py units <index> <total>          `unit::` filters for `cargo run -p e2e`, from main.rs
    shard.py names <index> <total> < list   `--exact` names from libtest `--list` output

A unit's weight is its `#[test]` count; units go greedily to the lightest shard, so
every unit lands in exactly one shard and the split is stable for the same sources.
"""
import re
import sys
from pathlib import Path

ALL = Path(__file__).resolve().parents[2] / "e2e/tests/all"


def unit_weights() -> dict[str, int]:
    mods = re.findall(r"^mod (?:r#)?(\w+);", (ALL / "main.rs").read_text(), re.M)
    weights = {}
    for unit in mods:
        files = [p for p in [ALL / f"{unit}.rs"] if p.is_file()] + list((ALL / unit).rglob("*.rs"))
        weights[unit] = max(1, sum(p.read_text().count("#[test") for p in files))
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
    mode, index, total = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    if not 1 <= index <= total:
        sys.exit(f"shard {index} is not within 1..{total}")
    if mode == "units":
        print(" ".join(f"{unit}::" for unit in assign(unit_weights(), total)[index - 1]))
    elif mode == "names":
        names = [line[: -len(": test")] for line in sys.stdin.read().splitlines() if line.endswith(": test")]
        weights: dict[str, int] = {}
        for name in names:
            weights[name.split("::")[0]] = weights.get(name.split("::")[0], 0) + 1
        mine = set(assign(weights, total)[index - 1])
        picked = [name for name in names if name.split("::")[0] in mine]
        print(" ".join(["--exact", *picked]) if picked else "")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
