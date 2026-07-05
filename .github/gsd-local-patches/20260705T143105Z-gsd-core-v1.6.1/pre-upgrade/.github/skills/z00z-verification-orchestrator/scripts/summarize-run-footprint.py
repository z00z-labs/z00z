#!/usr/bin/env python3
"""Summarize disk footprint of a verifier run root."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-root", required=True)
    parser.add_argument("--summary-out", required=True)
    return parser.parse_args()


def file_size(path: Path) -> int:
    if path.is_symlink():
        return 0
    if path.is_file():
        return path.stat().st_size
    total = 0
    for child in path.rglob("*"):
        if child.is_symlink() or not child.is_file():
            continue
        try:
            total += child.stat().st_size
        except OSError:
            continue
    return total


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def load_cache_cleanup(run_root: Path) -> dict[str, object]:
    path = run_root / "profiling" / "cache-maintenance.tsv"
    if not path.is_file():
        return {
            "available": False,
            "invocation_count": 0,
            "total_elapsed_ms": 0,
            "total_elapsed_secs": 0.0,
            "total_scanned_roots": 0,
            "total_trimmed_roots": 0,
            "total_trimmed_paths": 0,
            "total_reclaimed_bytes": 0,
            "max_elapsed_ms": 0,
            "max_reclaimed_bytes": 0,
        }

    rows: list[dict[str, object]] = []
    with path.open(encoding="utf-8", errors="replace", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        for row in reader:
            rows.append(
                {
                    "label": row.get("label", ""),
                    "elapsed_ms": int(row.get("elapsed_ms") or 0),
                    "scanned_roots": int(row.get("scanned_roots") or 0),
                    "trimmed_roots": int(row.get("trimmed_roots") or 0),
                    "trimmed_paths": int(row.get("trimmed_paths") or 0),
                    "reclaimed_bytes": int(row.get("reclaimed_bytes") or 0),
                }
            )

    total_elapsed_ms = sum(int(item["elapsed_ms"]) for item in rows)
    total_reclaimed_bytes = sum(int(item["reclaimed_bytes"]) for item in rows)
    return {
        "available": True,
        "invocation_count": len(rows),
        "total_elapsed_ms": total_elapsed_ms,
        "total_elapsed_secs": round(total_elapsed_ms / 1000.0, 3),
        "total_scanned_roots": sum(int(item["scanned_roots"]) for item in rows),
        "total_trimmed_roots": sum(int(item["trimmed_roots"]) for item in rows),
        "total_trimmed_paths": sum(int(item["trimmed_paths"]) for item in rows),
        "total_reclaimed_bytes": total_reclaimed_bytes,
        "max_elapsed_ms": max((int(item["elapsed_ms"]) for item in rows), default=0),
        "max_reclaimed_bytes": max((int(item["reclaimed_bytes"]) for item in rows), default=0),
        "top_invocations": sorted(
            rows,
            key=lambda item: (int(item["reclaimed_bytes"]), int(item["elapsed_ms"]), str(item["label"])),
            reverse=True,
        )[:5],
    }


def main() -> int:
    args = parse_args()
    run_root = Path(args.run_root).resolve()
    top_level: list[dict[str, object]] = []
    largest_files: list[tuple[int, Path]] = []

    for child in sorted(run_root.iterdir()):
        size = file_size(child)
        top_level.append(
            {
                "name": child.name,
                "path": rel(child, run_root),
                "kind": "dir" if child.is_dir() else "file",
                "bytes": size,
            }
        )

    for child in run_root.rglob("*"):
        if child.is_symlink() or not child.is_file():
            continue
        try:
            largest_files.append((child.stat().st_size, child))
        except OSError:
            continue

    top_level.sort(key=lambda item: (int(item["bytes"]), str(item["name"])), reverse=True)
    total_bytes = sum(int(item["bytes"]) for item in top_level)
    archived_previous_runs_bytes = next(
        (int(item["bytes"]) for item in top_level if item["name"] == "previous-runs"),
        0,
    )
    active_total_bytes = total_bytes - archived_previous_runs_bytes

    largest_files.sort(key=lambda item: (item[0], item[1].as_posix()), reverse=True)
    largest_active_files = [
        (size, path) for size, path in largest_files if "previous-runs/" not in rel(path, run_root)
    ]

    summary = {
        "run_root": run_root.as_posix(),
        "total_bytes": total_bytes,
        "active_total_bytes": active_total_bytes,
        "archived_previous_runs_bytes": archived_previous_runs_bytes,
        "cache_cleanup": load_cache_cleanup(run_root),
        "top_level": top_level,
        "top_level_active": [item for item in top_level if item["name"] != "previous-runs"],
        "largest_files": [
            {"path": rel(path, run_root), "bytes": size} for size, path in largest_active_files[:10]
        ],
    }

    out_path = Path(args.summary_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
