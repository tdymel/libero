#!/usr/bin/env python3
"""Exact test units for libero (todo 1500).

  scripts/units.py [--root SEAT] [--base main] [FILE ...]
      Maps touched files (default: the seat's diff against --base, committed and not) to
      their test units and prints the exact commands: libero in-file tests, libero `all`,
      e2e browser, e2e native, docs page. Files it cannot map are listed as shared.

  units.py run -- cargo test ... -- [FILTER ...] [LIBTEST FLAGS]
      Runs a cargo test with every `foo::` filter as a prefix, not libtest's substring:
      lists the tests, keeps the names that start with it and passes them `--exact`.
      `table::` no longer runs `sortable_table::`. Exits 1 when nothing matches.

The e2e runner (`cargo run -p e2e -- foo::`) does the same itself.
"""

import os
import re
import subprocess
import sys
from pathlib import Path

TED_RUN = "python3 /home/shino/Repos/dioxus_component_library/.claude/skills/team-common/ted_run.py"

# libtest flags whose next argument is a value, not a filter.
VALUED = {"--skip", "--test-threads", "--format", "--color", "--logfile", "--shuffle-seed", "-Z"}


def split(args):
    filters, skips, rest = [], [], []
    it = iter(args)
    for arg in it:
        if arg.startswith("--skip="):
            skips.append(arg[len("--skip="):])
        elif arg == "--skip":
            skips.append(next(it, ""))
        elif arg in VALUED:
            rest += [arg, next(it, "")]
        elif arg.startswith("-"):
            rest.append(arg)
        else:
            filters.append(arg)
    return filters, skips, rest


def exact(listed, args):
    """The libtest arguments that run exactly what `args` select; [] when nothing does."""
    filters, skips, rest = split(args)
    names = [
        name for name in listed
        if any(name.startswith(f) if f.endswith("::") else f in name for f in filters)
        and not any(s in name for s in skips)
    ]
    return rest + ["--exact"] + names if names else []


def run(argv):
    if "--" not in argv:
        sys.exit("units.py run: give the command after --")
    command = argv[argv.index("--") + 1:]
    cargo, libtest = (command[:command.index("--")], command[command.index("--") + 1:]) \
        if "--" in command else (command, [])
    filters, _, _ = split(libtest)
    env = dict(os.environ)
    if "RUSTC_WRAPPER" not in env and subprocess.run(["which", "sccache"], capture_output=True).returncode == 0:
        env["RUSTC_WRAPPER"] = "sccache"
    if "--exact" in libtest or "--list" in libtest or not any(f.endswith("::") for f in filters):
        sys.exit(subprocess.run(command, env=env).returncode)
    listing = subprocess.run(cargo + ["--", "--list"], env=env, stdout=subprocess.PIPE, text=True)
    if listing.returncode != 0:
        sys.exit(listing.returncode)
    listed = [line[:-len(": test")] for line in listing.stdout.splitlines() if line.endswith(": test")]
    args = exact(listed, libtest)
    if not args:
        sys.exit(f"units.py: no test matches {libtest}")
    print(f"units.py: {len(args) - args.index('--exact') - 1} test(s), run --exact", file=sys.stderr)
    sys.exit(subprocess.run(cargo + ["--"] + args, env=env).returncode)


def git(root, *args):
    out = subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True)
    return out.stdout.split() if out.returncode == 0 else []


def touched(root, base):
    files = set(git(root, "diff", "--name-only", f"{base}...HEAD"))
    files |= set(git(root, "diff", "--name-only", "HEAD"))
    files |= set(git(root, "ls-files", "--others", "--exclude-standard"))
    return sorted(files)


# Paths whose stem names a component: the name decides the units.
NAMED = [
    r"libero/src/components/[^/]+/(?P<n>[^/.]+)",
    r"libero/src/(?:theme/defaults|localization/labels|platform)/(?P<n>[^/.]+)\.rs",
    r"libero/tests/all/(?P<n>[^/.]+)\.rs",
    r"e2e/tests/(?:all|native)/(?P<n>[^/.]+)\.rs",
    r"e2e/fixtures/src/(?P<n>[^/.]+)\.rs",
    r"docs/src/pages/[^/]+/(?P<n>[^/.]+)",
]
NOT_UNITS = {"mod", "main", "lib", "common"}


def lib_module(path):
    """`libero/src/a/b/c.rs` -> `a::b::c::`, the prefix of its in-file tests; a whole
    component for a file inside one (`components/data_display/table/core.rs` -> `...::table::`)."""
    parts = path.removeprefix("libero/src/").removesuffix(".rs").split("/")
    if parts[0] == "components" and len(parts) > 3:
        parts = parts[:3]
    if parts[-1] in ("mod", "lib"):
        parts.pop()
    return "::".join(parts) + "::" if parts else None


def plan(root, files):
    names, lib, shared = {}, set(), []
    for path in files:
        if path.startswith((".agents/", "scripts/")):
            continue
        if not path.endswith(".rs") and not path.startswith("libero/src/components/"):
            shared.append(path)
            continue
        if path.startswith("libero/src/") and (module := lib_module(path)):
            lib.add(module)
        match = next((m for p in NAMED if (m := re.match(p, path))), None)
        if match and match["n"] not in NOT_UNITS:
            names.setdefault(match["n"], []).append(path)
        elif not path.startswith("libero/src/"):
            shared.append(path)
    wrapper = f"python3 {root}/scripts/units.py run --"
    target = f"--target-dir {root}/target/dev"
    lines = []
    for module in sorted(lib):
        lines.append(f"{TED_RUN} cargo -- {wrapper} cargo test -q -p libero --lib {target} -- {module}")
    for name in sorted(names):
        if (root / f"libero/tests/all/{name}.rs").exists():
            lines.append(f"{TED_RUN} cargo -- {wrapper} cargo test -q -p libero --test all {target} -- {name}::")
        if (root / f"e2e/tests/all/{name}.rs").exists():
            lines.append(f"{TED_RUN} e2e -- cargo run -q -p e2e {target} -- {name}::")
        if (root / f"e2e/tests/native/{name}.rs").exists():
            lines.append(f"{TED_RUN} e2e -- {wrapper} cargo test -q -p e2e --features native --test native {target} -- {name}::")
        page = next(root.glob(f"docs/src/pages/*/{name}.rs"), None) or next(root.glob(f"docs/src/pages/*/{name}"), None)
        if page:
            lines.append(f"# docs page: {page.relative_to(root)} (screenshot at default switches)")
        if not any(root.glob(f"e2e/tests/all/{name}.rs")):
            lines.append(f"# {name}: no e2e unit (add e2e/tests/all/{name}.rs, olaf-extra.md)")
    for path in shared:
        lines.append(f"# shared: {path} (theme, stylesheet or harness: the whole browser tier)")
    return lines


def main():
    argv = sys.argv[1:]
    if argv[:1] == ["run"]:
        return run(argv[1:])
    root, base = Path.cwd(), "main"
    files = []
    it = iter(argv)
    for arg in it:
        if arg == "--root":
            root = Path(next(it))
        elif arg == "--base":
            base = next(it)
        elif arg in ("-h", "--help"):
            sys.exit(__doc__)
        else:
            files.append(arg)
    root = Path(git(root, "rev-parse", "--show-toplevel")[0]) if git(root, "rev-parse", "--show-toplevel") else root
    files = [str(Path(f).resolve().relative_to(root)) if Path(f).is_absolute() else f for f in files] or touched(root, base)
    for line in plan(root, files):
        print(line)


if __name__ == "__main__":
    main()
