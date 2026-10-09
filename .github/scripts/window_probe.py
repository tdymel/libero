#!/usr/bin/env python3
"""Launches the platform apps once and reports what the runner gives them (2776): a probe, not a test.

    window_probe.py desktop <fixture-app> <out-dir>   the WebView app, read back over its `E2E_BRIDGE`
    window_probe.py native <docs-app> <out-dir>       the Blitz docs app on wgpu (vello)

Each mode screenshots the screen (`screencapture`, macOS only) and leaves the app's log in <out-dir>.
"""
import json
import os
import shutil
import socket
import subprocess
import sys
import time
from pathlib import Path

LAUNCH = 120
# What the page reports about the WebView it runs in.
QUERY = "return { agent: navigator.userAgent, ratio: devicePixelRatio, size: [innerWidth, innerHeight], ready: document.readyState };"


def screenshot(out: Path, name: str) -> None:
    if shutil.which("screencapture"):
        subprocess.run(["screencapture", "-x", str(out / f"{name}.png")], check=False)


def start(app: str, out: Path, name: str, env: dict[str, str]) -> subprocess.Popen:
    log = open(out / f"{name}.log", "w")
    return subprocess.Popen([app], env={**os.environ, **env}, stdout=log, stderr=subprocess.STDOUT)


def desktop(app: str, out: Path) -> int:
    server = socket.create_server(("127.0.0.1", 0))
    server.settimeout(LAUNCH)
    proc = start(app, out, "desktop", {"E2E_BRIDGE": "127.0.0.1:%d" % server.getsockname()[1]})
    try:
        # The fixture connects from its first effect, so a connection means the WebView runs.
        conn, _ = server.accept()
        conn.settimeout(30)
        stream = conn.makefile("rw")
        stream.write(json.dumps(QUERY) + "\n")
        stream.flush()
        print("desktop: the bridge answered", stream.readline().strip())
        screenshot(out, "desktop")
        return 0
    except OSError as error:
        print(f"desktop: no bridge answer ({error}), exit code {proc.poll()}")
        screenshot(out, "desktop")
        return 1
    finally:
        proc.kill()
        proc.wait()


def native(app: str, out: Path) -> int:
    # The app's logger drops wgpu's adapter line: a panic without an adapter, or alive and painted.
    proc = start(app, out, "native", {})
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline and proc.poll() is None:
        time.sleep(1)
    screenshot(out, "native")
    alive = proc.poll() is None
    print(f"native: {'still running after 30 s' if alive else f'exited with {proc.returncode}'}")
    proc.kill()
    proc.wait()
    return 0 if alive else 1


def main() -> int:
    mode, app, out = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    out.mkdir(parents=True, exist_ok=True)
    code = {"desktop": desktop, "native": native}[mode](app, out)
    print(f"--- {mode} app log (tail)")
    print("".join((out / f"{mode}.log").read_text(errors="replace").splitlines(keepends=True)[-60:]))
    return code


if __name__ == "__main__":
    sys.exit(main())
