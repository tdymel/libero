#!/usr/bin/env python3
"""The ci-probe workflow's helpers (3315): one platform x tier job, one test filter.

    ci_probe.py plan                   job outputs from `.github/ci-probe` or the dispatch inputs
    ci_probe.py names <filter> < list  `--exact` names of the `::native` arms `<filter>` selects
    ci_probe.py summary <log> ...      a `::notice` with every `test result:` line

The probe file (`ted_ci.py probe` writes it on a `ci/*` branch) holds `key=value` lines:
`os` linux|macos|windows, `tier` unit|native|web|desktop|ios|none, `filter` e.g. `tabs::` or `shard:1:6`, and
optionally `env` (`NAME=value` pairs for the test run) and `dev`, whose probes on a shared `ci/probe-*` branch cancel only each other.
"""
import os
import re
import sys
from pathlib import Path

PROBE = Path(__file__).resolve().parents[1] / "ci-probe"
RUNNERS = {"linux": "ubuntu-24.04", "macos": "macos-26", "windows": "windows-2025"}
TIERS = ("unit", "native", "web", "desktop", "ios")
# The cargo cache each leg restores in ci.yml or platforms.yml; only main saves them (3413).
CACHES = {
    ("linux", "unit"): "test",
    ("linux", "native"): "native",
    ("linux", "web"): "e2e",
    ("linux", "desktop"): "e2e",
    ("macos", "unit"): "platforms-macos-desktop",
    ("macos", "native"): "platforms-macos-desktop",
    ("macos", "web"): "platforms-ios-e2e",
    ("macos", "desktop"): "platforms-macos-desktop",
    ("macos", "ios"): "platforms-ios-e2e",
    ("windows", "unit"): "platforms-windows-native",
    ("windows", "native"): "platforms-windows-native",
    ("windows", "web"): "platforms-windows-web",
    ("windows", "desktop"): "platforms-windows-desktop",
}
# Reaches a shell unquoted only after this check.
FILTER = re.compile(r"^[A-Za-z0-9_:]+$")
# Space-separated `NAME=value` pairs exported to the probe's test run, e.g. `RUST_TEST_THREADS=1`.
ENV = re.compile(r"^([A-Z][A-Z0-9_]*=[A-Za-z0-9_.]*)?( [A-Z][A-Z0-9_]*=[A-Za-z0-9_.]*)*$")
# Names the dev's concurrency group on a shared `ci/probe-*` branch; empty is one group per branch.
DEV = re.compile(r"^[A-Za-z0-9_-]*$")


def fail(message: str) -> None:
    # Workflow commands are read from stdout.
    print(f"::error title=ci-probe::{message}")
    sys.exit(1)


def read_probe() -> dict[str, str]:
    if os.environ.get("GITHUB_EVENT_NAME") == "workflow_dispatch":
        return {key: os.environ.get(f"PROBE_{key.upper()}", "") for key in ("os", "tier", "filter", "env")}
    if not PROBE.is_file():
        fail(f"no .github/{PROBE.name} file on this branch; write it with ted_ci.py probe")
    probe = {}
    for line in PROBE.read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.lstrip().startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            probe[key.strip()] = value.strip()
    return probe


def plan() -> None:
    probe = read_probe()
    target, tier, test = probe.get("os", ""), probe.get("tier", ""), probe.get("filter", "")
    dev = probe.get("dev", "")
    if not DEV.match(dev):
        fail(f"dev {dev!r} is not a name like `olaf-53`")
    if tier != "none":
        if target not in RUNNERS:
            fail(f"os {target!r} is not one of {', '.join(RUNNERS)}")
        if tier not in TIERS:
            fail(f"tier {tier!r} is not one of {', '.join(TIERS)}, none")
        if tier == "ios" and target != "macos":
            fail("tier ios runs on macos only")
        if not FILTER.match(test):
            fail(f"filter {test!r} is not a test path like `tabs::`")
        if not ENV.match(probe.get("env", "")):
            fail(f"env {probe.get('env')!r} is not `NAME=value` pairs")
    outputs = {
        "os": target,
        "tier": tier,
        "filter": test,
        "env": probe.get("env", ""),
        "runner": RUNNERS.get(target, "ubuntu-24.04"),
        "cache": CACHES.get((target, tier), ""),
        "dev": dev,
    }
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as out:
        out.writelines(f"{key}={value}\n" for key, value in outputs.items())
    print(f"probe: {target} {tier} {test}")


def names(test: str) -> None:
    # The whole set as platforms.yml runs it: 500 `--exact` names pass Windows' command-line limit.
    if test == "::native":
        sys.stdin.read()
        print("::native")
        return
    listed = [line.rstrip("\r")[: -len(": test")] for line in sys.stdin.read().splitlines()
              if line.rstrip("\r").endswith(": test")]
    # A filter ending in `::` is a prefix, as units.py and the e2e runner read it.
    hit = [name for name in listed if name.endswith("::native")
           and (name.startswith(test) if test.endswith("::") else test in name)]
    print(" ".join(["--exact", *hit]) if hit else "")


def summary(paths: list[str]) -> None:
    lines = []
    for path in map(Path, paths):
        if path.is_file():
            text = path.read_text(encoding="utf-8", errors="replace")
            lines += [f"{path.stem}: {l.strip()}" for l in re.sub(r"\x1b\[[0-9;?]*[A-Za-z]", "", text).splitlines()
                      if l.strip().startswith("test result:") and " 0 passed; 0 failed;" not in l]
    body = "\n".join(lines).replace("%", "%25").replace("\n", "%0A") or "no `test result:` line in the logs"
    print(f"::notice title=probe result::{body}")


def main() -> None:
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "plan":
        plan()
    elif mode == "names" and len(sys.argv) == 3:
        names(sys.argv[2])
    elif mode == "summary":
        summary(sys.argv[2:])
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
