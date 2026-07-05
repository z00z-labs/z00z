#!/usr/bin/env python3
"""Summarize GNU time resource profiles collected per gate."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


KEYS = {
    "user_cpu_secs": "User time (seconds)",
    "system_cpu_secs": "System time (seconds)",
    "cpu_percent": "Percent of CPU this job got",
    "wall_elapsed_raw": "Elapsed (wall clock) time (h:mm:ss or m:ss)",
    "max_rss_kb": "Maximum resident set size (kbytes)",
    "major_page_faults": "Major (requiring I/O) page faults",
    "minor_page_faults": "Minor (reclaiming a frame) page faults",
    "fs_inputs": "File system inputs",
    "fs_outputs": "File system outputs",
    "swaps": "Swaps",
    "exit_status": "Exit status",
    "command": "Command being timed",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profiles-dir", required=True)
    parser.add_argument("--summary-out", required=True)
    return parser.parse_args()


def parse_elapsed(raw: str) -> float:
    if not raw:
        return 0.0
    parts = raw.strip().split(":")
    try:
        if len(parts) == 3:
            hours = int(parts[0])
            minutes = int(parts[1])
            seconds = float(parts[2])
            return hours * 3600 + minutes * 60 + seconds
        if len(parts) == 2:
            minutes = int(parts[0])
            seconds = float(parts[1])
            return minutes * 60 + seconds
        return float(parts[0])
    except ValueError:
        return 0.0


def parse_meta(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}

    meta: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if "\t" not in line:
            continue
        key, value = line.split("\t", 1)
        meta[key] = value
    return meta


def parse_list(raw: str | None) -> list[str]:
    if not raw:
        return []
    return [item for item in raw.split(";") if item]


def parse_int(raw: str | None) -> int:
    if not raw:
        return 0
    try:
        return int(raw)
    except ValueError:
        return 0


def parse_profile(path: Path) -> dict[str, object]:
    raw: dict[str, str] = {}
    meta = parse_meta(path.parent.parent / "resource-meta" / f"{path.stem}.tsv")
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        stripped = line.strip()
        for raw_key in KEYS.values():
            prefix = f"{raw_key}:"
            if stripped.startswith(prefix):
                raw[raw_key] = stripped[len(prefix) :].strip()
                break

    profile: dict[str, object] = {
        "gate_id": path.stem,
        "label": meta.get("label", path.stem),
        "profile_kind": meta.get("kind", "gate"),
        "execution_mode": meta.get("execution_mode", "unknown"),
        "target_roots": parse_list(meta.get("target_roots")),
        "cache_roots": parse_list(meta.get("cache_roots")),
        "cleanup_elapsed_ms": parse_int(meta.get("cleanup_elapsed_ms")),
        "cleanup_scanned_roots": parse_int(meta.get("cleanup_scanned_roots")),
        "cleanup_trimmed_roots": parse_int(meta.get("cleanup_trimmed_roots")),
        "cleanup_trimmed_paths": parse_int(meta.get("cleanup_trimmed_paths")),
        "cleanup_reclaimed_bytes": parse_int(meta.get("cleanup_reclaimed_bytes")),
        "path": path.as_posix(),
        "wall_elapsed_secs": parse_elapsed(raw.get(KEYS["wall_elapsed_raw"], "")),
        "wall_elapsed_raw": raw.get(KEYS["wall_elapsed_raw"], ""),
    }
    for out_key, raw_key in KEYS.items():
        if out_key in {"wall_elapsed_raw"}:
            continue
        value = raw.get(raw_key)
        if value is None:
            profile[out_key] = None
            continue
        if out_key == "cpu_percent":
            cleaned = value.rstrip("%").strip()
            profile[out_key] = float(cleaned) if cleaned else None
        elif out_key == "command":
            profile[out_key] = value
        else:
            try:
                profile[out_key] = int(value) if value.isdigit() or value.startswith("-") else float(value)
            except ValueError:
                profile[out_key] = value

    user_cpu = float(profile.get("user_cpu_secs") or 0.0)
    system_cpu = float(profile.get("system_cpu_secs") or 0.0)
    profile["cpu_total_secs"] = round(user_cpu + system_cpu, 3)
    profile["cleanup_elapsed_secs"] = round(float(profile["cleanup_elapsed_ms"]) / 1000.0, 3)
    fs_inputs = int(profile.get("fs_inputs") or 0)
    fs_outputs = int(profile.get("fs_outputs") or 0)
    profile["fs_io_total"] = fs_inputs + fs_outputs
    return profile


def top_profiles(profiles: list[dict[str, object]], key: str, limit: int = 5) -> list[dict[str, object]]:
    return sorted(
        profiles,
        key=lambda item: (float(item.get(key) or 0.0), str(item.get("gate_id", ""))),
        reverse=True,
    )[:limit]


def build_summary(profiles: list[dict[str, object]]) -> dict[str, object]:
    return {
        "available": bool(profiles),
        "profile_count": len(profiles),
        "profile_kind_count": {
            "gate": sum(1 for item in profiles if item.get("profile_kind") == "gate"),
            "command": sum(1 for item in profiles if item.get("profile_kind") == "command"),
        },
        "units_note": {
            "max_rss_kb": "kilobytes",
            "fs_inputs_outputs": "GNU time raw counters",
            "cpu_times": "seconds",
            "wall_elapsed": "seconds",
            "cleanup_elapsed": "seconds",
            "cleanup_reclaimed_bytes": "bytes",
        },
        "profiles": sorted(
            profiles,
            key=lambda item: (float(item.get("wall_elapsed_secs") or 0.0), str(item.get("gate_id", ""))),
            reverse=True,
        ),
        "top_wall": top_profiles(profiles, "wall_elapsed_secs"),
        "top_cpu_total": top_profiles(profiles, "cpu_total_secs"),
        "top_memory_rss": top_profiles(profiles, "max_rss_kb"),
        "top_fs_io": top_profiles(profiles, "fs_io_total"),
    }


def main() -> int:
    args = parse_args()
    profiles_dir = Path(args.profiles_dir)
    profiles = []
    if profiles_dir.is_dir():
        for path in sorted(profiles_dir.glob("*.time")):
            profiles.append(parse_profile(path))

    out_path = Path(args.summary_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(build_summary(profiles), indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
