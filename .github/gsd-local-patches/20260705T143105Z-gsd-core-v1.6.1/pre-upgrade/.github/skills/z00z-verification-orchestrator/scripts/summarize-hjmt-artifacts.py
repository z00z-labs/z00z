#!/usr/bin/env python3
"""Summarize HJMT artifacts found under the active verifier run root."""

from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-root", required=True)
    parser.add_argument("--summary-out", required=True)
    return parser.parse_args()


def rel(path: Path, root: Path) -> str:
    return path.resolve().relative_to(root).as_posix()


def active_candidates(root: Path, pattern: str) -> list[Path]:
    candidates: list[Path] = []
    for path in root.rglob(pattern):
        rel_path = rel(path, root)
        if rel_path.startswith("previous-runs/"):
            continue
        candidates.append(path)
    return sorted(candidates)


def median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def stats(values: list[int | float]) -> dict[str, float | int | None]:
    if not values:
        return {"min": None, "median": None, "max": None, "mean": None}
    numeric = [float(value) for value in values]
    return {
        "min": min(values),
        "median": median(numeric),
        "max": max(values),
        "mean": round(sum(numeric) / len(numeric), 3),
    }


def select_primary(root: Path, candidates: list[Path]) -> Path | None:
    if not candidates:
        return None
    return sorted(
        candidates,
        key=lambda path: (
            0 if "shared_precise" in path.as_posix() else 1,
            len(rel(path, root).split("/")),
            -path.stat().st_mtime,
            rel(path, root),
        ),
    )[0]


def main() -> int:
    args = parse_args()
    run_root = Path(args.run_root).resolve()
    metrics_candidates = active_candidates(run_root, "hjmt_cache_scheduler_metrics.json")
    proof_candidates = active_candidates(run_root, "hjmt_proof_size_report.json")
    throughput_candidates = active_candidates(run_root, "settlement_shard*.md")

    primary_metrics = select_primary(run_root, metrics_candidates)
    primary_proof = select_primary(run_root, proof_candidates)

    summary: dict[str, object] = {
        "available": bool(primary_metrics or primary_proof),
        "metrics_candidates": [rel(path, run_root) for path in metrics_candidates],
        "proof_candidates": [rel(path, run_root) for path in proof_candidates],
        "throughput_candidates": [rel(path, run_root) for path in throughput_candidates],
        "primary_metrics_path": rel(primary_metrics, run_root) if primary_metrics else None,
        "primary_proof_path": rel(primary_proof, run_root) if primary_proof else None,
        "tps": {
            "measured": False,
            "artifact_paths": [rel(path, run_root) for path in throughput_candidates],
            "reason": (
                "No run-root settlement throughput artifact was produced; do not infer TPS "
                "from proof-size or one-shot verify-time samples."
            ),
        },
    }

    if primary_metrics:
        metrics = json.loads(primary_metrics.read_text(encoding="utf-8"))
        hits = int(metrics.get("cache_hit_count", 0))
        misses = int(metrics.get("cache_miss_count", 0))
        total = hits + misses
        summary["cache_scheduler"] = {
            "cache_hit_count": hits,
            "cache_miss_count": misses,
            "cache_hit_ratio": round(hits / total, 6) if total else None,
            "invalidation_count": int(metrics.get("invalidation_count", 0)),
            "root_reuse_ratio": metrics.get("root_reuse_ratio"),
            "proof_segment_reuse_ratio": metrics.get("proof_segment_reuse_ratio"),
            "scheduler_queue_depth": metrics.get("scheduler_queue_depth"),
            "scheduler_backpressure_count": metrics.get("scheduler_backpressure_count"),
            "deterministic_parent_ordering": metrics.get("deterministic_parent_ordering"),
            "max_active": metrics.get("scheduler_metrics", {}).get("max_active"),
            "reject_count": metrics.get("scheduler_metrics", {}).get("reject_count"),
            "cancel_count": metrics.get("scheduler_metrics", {}).get("cancel_count"),
            "max_queued": metrics.get("scheduler_metrics", {}).get("max_queued"),
            "last_blocking_wait_us": metrics.get("scheduler_metrics", {}).get("last_blocking_wait_us"),
        }

    if primary_proof:
        proof = json.loads(primary_proof.read_text(encoding="utf-8"))
        entries = proof.get("entries", [])
        sizes = [int(entry["proof_size_bytes"]) for entry in entries if entry.get("proof_size_bytes") is not None]
        verify = [int(entry["verify_time_us"]) for entry in entries if entry.get("verify_time_us") is not None]
        summary["proof_examples"] = {
            "entry_count": len(entries),
            "proof_size_bytes": stats(sizes),
            "verify_time_us": stats(verify),
            "slowest_examples": [
                {
                    "example_id": entry.get("example_id"),
                    "backend_mode": entry.get("backend_mode"),
                    "proof_size_bytes": entry.get("proof_size_bytes"),
                    "verify_time_us": entry.get("verify_time_us"),
                    "api_surface": entry.get("api_surface"),
                }
                for entry in sorted(
                    entries,
                    key=lambda item: int(item.get("verify_time_us") or 0),
                    reverse=True,
                )[:5]
            ],
            "largest_examples": [
                {
                    "example_id": entry.get("example_id"),
                    "backend_mode": entry.get("backend_mode"),
                    "proof_size_bytes": entry.get("proof_size_bytes"),
                    "verify_time_us": entry.get("verify_time_us"),
                    "api_surface": entry.get("api_surface"),
                }
                for entry in sorted(
                    entries,
                    key=lambda item: int(item.get("proof_size_bytes") or 0),
                    reverse=True,
                )[:5]
            ],
        }

    out_path = Path(args.summary_out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
