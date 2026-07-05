#!/usr/bin/env python3
"""Inspect profiler tool availability for the active verifier run."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path


TOOL_SPECS = (
    ("gnu_time", (("/usr/bin/time", "--version"),)),
    ("perf", (("perf", "--version"),)),
    ("strace", (("strace", "--version"),)),
    ("valgrind", (("valgrind", "--version"),)),
    ("flamegraph", (("flamegraph", "--version"), ("flamegraph", "-V"))),
    ("cargo-flamegraph", (("cargo-flamegraph", "-V"), ("cargo-flamegraph", "--help"))),
    ("hyperfine", (("hyperfine", "--version"),)),
    ("heaptrack", (("heaptrack", "--version"),)),
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--summary-out", required=True)
    return parser.parse_args()


def resolve_path(command: str) -> str | None:
    if command.startswith("/"):
        path = Path(command)
        return str(path) if path.exists() else None
    return shutil.which(command)


def version_line(candidates: tuple[tuple[str, ...], ...], resolved: str | None) -> str | None:
    if not resolved:
        return None
    for argv in candidates:
        try:
            completed = subprocess.run(
                [resolved, *argv[1:]],
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )
        except Exception:
            continue
        output = completed.stdout.strip() or completed.stderr.strip()
        if output:
            first = output.splitlines()[0]
            if first.lower().startswith("error:"):
                continue
            return first
    return None


def main() -> int:
    args = parse_args()
    items: list[dict[str, object]] = []
    for name, argv_candidates in TOOL_SPECS:
        resolved = resolve_path(argv_candidates[0][0])
        items.append(
            {
                "name": name,
                "command": argv_candidates[0][0],
                "available": bool(resolved),
                "path": resolved,
                "version": version_line(argv_candidates, resolved),
            }
        )

    summary = {
        "tool_count": len(items),
        "available_count": sum(1 for item in items if item["available"]),
        "missing_count": sum(1 for item in items if not item["available"]),
        "tools": items,
    }

    out_path = Path(args.summary_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
