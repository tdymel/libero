#!/usr/bin/env python3
"""Turns a red step's logs into check-run annotations, readable through the API without auth.

    ci_annotate.py <log> [<log> ...]

One `::error` per failed test, compile error or runner error (10 per step, the rest joined
into the tenth), each log's tail when nothing matched, and one `::notice` with the runner's facts.
"""
import platform
import re
import subprocess
import sys
from pathlib import Path

CAP = 10
ANSI = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")
BLOCK = re.compile(r"^---- (\S+) std(?:out|err) ----$")
FAILED = re.compile(r"^test (\S+) \.\.\. FAILED$")
PANIC = re.compile(r"panicked at (\S+?):?$|panicked at '(.*)', (\S+)$")
COMPILE = re.compile(r"^(error(?:\[E\d+\])?: .*)$")
# The e2e runners' own red lines: anyhow's `Error:`, a deadline, a bridge that never came.
RUNNER = re.compile(r"^(Error: .*|e2e \w+: .*(?:overran|never connected|ALL-FAIL|PARTIAL).*)$")


def escape(text: str, prop: bool = False) -> str:
    text = text.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")
    return text.replace(":", "%3A").replace(",", "%2C") if prop else text


def clip(text: str, size: int = 400) -> str:
    return text if len(text) <= size else text[: size - 3] + "..."


def failures(lines: list[str]) -> list[tuple[str, str]]:
    """(title, message) per red item, in log order, each title once."""
    found: dict[str, str] = {}
    current = None
    for index, line in enumerate(lines):
        if block := BLOCK.match(line):
            current = block.group(1)
            continue
        if line == "failures:" or line.startswith("test result:"):
            current = None
        if current and not found.get(current):
            if panic := PANIC.search(line):
                if panic.group(2):
                    found[current] = f"{panic.group(2)} ({panic.group(3)})"
                else:
                    rest = [l for l in lines[index + 1 : index + 6] if l.strip()]
                    found[current] = f"{rest[0].strip() if rest else ''} ({panic.group(1)})"
                continue
            if line.startswith("Error: "):
                found[current] = line
                continue
        if failed := FAILED.match(line):
            found.setdefault(failed.group(1), "")
        elif compile_error := COMPILE.match(line):
            where = next((l.strip() for l in lines[index + 1 : index + 4] if "-->" in l), "")
            found.setdefault(f"{compile_error.group(1)[:80]} {where}".strip(), compile_error.group(1))
        elif runner := RUNNER.match(line):
            found.setdefault(clip(runner.group(1), 80), runner.group(1))
    # A test listed FAILED whose block never printed (killed process) keeps a placeholder.
    return [(title, message or "FAILED, no panic message in the log") for title, message in found.items()]


def read(log: Path) -> list[str]:
    text = log.read_text(encoding="utf-8", errors="replace")
    return [ANSI.sub("", l).rstrip("\r") for l in text.splitlines()]


def collect(paths: list[str]) -> list[tuple[str, str]]:
    # A log of a step skipped after an earlier red one is missing; only all missing says something.
    logs = [Path(path) for path in paths if Path(path).is_file()]
    if not logs:
        return [("no log", f"none of {paths} exists: the job failed before its commands ran")]
    if red := [item for log in logs for item in failures(read(log))]:
        return red
    tails = []
    for log in logs:
        tail = [l for l in read(log) if l.strip()][-8:]
        tails.append((f"{log.stem}: red without a parsed failure", "\n".join(tail) or "empty log"))
    return tails


def run(command: list[str]) -> str:
    try:
        out = subprocess.run(command, capture_output=True, text=True, timeout=60)
        return (out.stdout + out.stderr).strip()
    except (OSError, subprocess.SubprocessError) as error:
        return f"{command[0]}: {error}"


def facts() -> str:
    lines = [f"python {sys.version.split()[0]} at {sys.executable}", f"platform {platform.platform()}"]
    if sys.platform == "darwin":
        lines.append(run(["sw_vers"]).replace("\n", "; "))
        runtimes = [l.strip() for l in run(["xcrun", "simctl", "list", "runtimes"]).splitlines() if "iOS" in l]
        lines.append("simctl runtimes: " + "; ".join(runtimes or ["none"]))
    elif sys.platform == "win32":
        lines.append(run(["cmd", "/c", "ver"]))
    return "\n".join(lines)


def main() -> None:
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    red = collect(sys.argv[1:])
    # GitHub keeps 10 error annotations per step: nine single ones, the rest joined.
    for title, message in red[: CAP - 1]:
        print(f"::error title={escape(clip(title, 120), True)}::{escape(clip(message))}")
    if rest := red[CAP - 1 :]:
        joined = "\n".join(f"{title}: {clip(message, 120)}" for title, message in rest)
        print(f"::error title={escape(f'{len(rest)} more', True)}::{escape(clip(joined, 3000))}")
    print(f"::notice title=runner::{escape(facts())}")


if __name__ == "__main__":
    main()
