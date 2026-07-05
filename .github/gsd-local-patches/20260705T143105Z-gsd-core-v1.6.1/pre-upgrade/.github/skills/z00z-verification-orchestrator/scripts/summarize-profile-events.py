#!/usr/bin/env python3
"""Summarize Z00Z verification profiling events."""

from __future__ import annotations

import argparse
import csv
import json
import math
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--events", required=True)
    parser.add_argument("--summary-out", required=True)
    return parser.parse_args()


def load_events(path: Path) -> list[dict[str, object]]:
    if not path.exists() or path.stat().st_size == 0:
        return []

    with path.open(encoding="utf-8", errors="replace", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        events: list[dict[str, object]] = []
        for row in reader:
            elapsed_ms = int(row.get("elapsed_ms") or 0)
            elapsed_secs = float(row.get("elapsed_secs") or 0.0)
            events.append(
                {
                    "kind": row.get("kind", ""),
                    "label": row.get("label", ""),
                    "status": row.get("status", ""),
                    "elapsed_ms": elapsed_ms,
                    "elapsed_secs": elapsed_secs,
                    "started_at": row.get("started_at", ""),
                    "ended_at": row.get("ended_at", ""),
                    "command": row.get("command", ""),
                }
            )
        return events


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def guidance_source_rel(root: Path) -> str | None:
    candidate = root / ".planning/phases/profiling-comprehensive.md"
    if candidate.is_file():
        return candidate.relative_to(root).as_posix()
    return None


def unique(items: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        if not item or item in seen:
            continue
        seen.add(item)
        ordered.append(item)
    return ordered


def recommendations_for(event: dict[str, object]) -> list[str]:
    label = str(event.get("label", "")).lower()
    command = str(event.get("command", "")).lower()
    text = f"{label} {command}"
    recommendations: list[str] = []

    if "cargo " in text or label.startswith("l3-"):
        recommendations.extend(
            [
                "Keep one stable release feature set so Cargo can reuse compiled artifacts across gates.",
                "Prebuild shared test binaries once and prefer reuse over repeating compile+run cycles.",
            ]
        )
    if any(token in text for token in ("clippy", "nextest", "cargo test", "verify-fast")):
        recommendations.append(
            "Split compile-heavy Rust gates from execution-heavy Rust gates so repeated analysis does not rebuild unchanged crates."
        )
    if any(token in text for token in ("kani", "miri", "verus", "prusti", "loom")):
        recommendations.append(
            "Narrow proof/interpreter targets to the smallest crate or harness set that still covers the intended invariant."
        )
    if any(token in text for token in ("tla", "apalache", "alloy", "tamarin", "proverif")):
        recommendations.extend(
            [
                "Reuse generated report-local specs across adjacent model-checker stages instead of regenerating them per tool.",
                "Parallelize independent models only after confirming they do not share mutable temp state.",
            ]
        )
    if any(token in text for token in ("charon", "aeneas", "crux", "cryptol", "saw", "refinement")):
        recommendations.extend(
            [
                "Cache code-to-logic intermediate artifacts such as linked MIR, LLBC, and generated harness crates inside the run root.",
                "Avoid repeated cargo metadata and wrapper startup for the same manifest set in one run.",
            ]
        )
    if "fuzz" in text:
        recommendations.extend(
            [
                "Reuse corpora and compiled fuzz targets across iterations; only rotate artifacts, not the full build.",
                "Keep exhaustive fuzz campaigns separate from short gate sanity checks.",
            ]
        )
    if any(token in text for token in ("dudect", "constant-time")):
        recommendations.append(
            "Reuse built constant-time benches and separate leak-detection smoke from longer statistical campaigns."
        )
    if any(token in text for token in ("supply-chain", "cargo deny", "cargo vet", "cargo audit", "geiger")):
        recommendations.append(
            "Batch dependency and unsafe scans after one metadata resolution pass so the same workspace graph is not recomputed repeatedly."
        )
    if any(token in text for token in ("json", "snapshot", "artifact", "report")):
        recommendations.append(
            "Prefer memory-first handoff for intermediate state and checkpoint to disk only for final evidence artifacts."
        )

    if not recommendations:
        recommendations.append(
            "Inspect whether this stage can reuse prior artifacts or be parallelized without weakening evidence quality."
        )
    return unique(recommendations)[:3]


def annotate_slowest(events: list[dict[str, object]]) -> list[dict[str, object]]:
    annotated: list[dict[str, object]] = []
    for event in events:
        enriched = dict(event)
        enriched["recommendations"] = recommendations_for(event)
        annotated.append(enriched)
    return annotated


def aggregate_recommendations(events: list[dict[str, object]], limit: int = 6) -> list[str]:
    merged: list[str] = []
    for event in events:
        for recommendation in recommendations_for(event):
            if recommendation not in merged:
                merged.append(recommendation)
            if len(merged) >= limit:
                return merged
    return merged


def build_summary(events: list[dict[str, object]]) -> dict[str, object]:
    count = len(events)
    gate_count = sum(1 for event in events if event["kind"] == "gate")
    command_count = sum(1 for event in events if event["kind"] == "command")
    total_elapsed_ms = sum(int(event["elapsed_ms"]) for event in events)
    top_n = max(1, math.ceil(count * 0.05)) if count else 0
    slowest = sorted(
        events,
        key=lambda event: (
            float(event["elapsed_secs"]),
            str(event["ended_at"]),
            str(event["kind"]),
            str(event["label"]),
        ),
        reverse=True,
    )[:top_n]
    slowest_total_ms = sum(int(event["elapsed_ms"]) for event in slowest)
    slowest_fraction_percent = 0.0
    if total_elapsed_ms > 0:
        slowest_fraction_percent = round((slowest_total_ms / total_elapsed_ms) * 100.0, 2)
    root = repo_root()
    return {
        "event_count": count,
        "gate_event_count": gate_count,
        "command_event_count": command_count,
        "top_percent": 5,
        "top_n": top_n,
        "total_elapsed_ms": total_elapsed_ms,
        "total_elapsed_secs": round(total_elapsed_ms / 1000.0, 3),
        "slowest_total_ms": slowest_total_ms,
        "slowest_total_secs": round(slowest_total_ms / 1000.0, 3),
        "slowest_fraction_percent": slowest_fraction_percent,
        "guidance_source": guidance_source_rel(root),
        "aggregate_recommendations": aggregate_recommendations(slowest),
        "slowest": annotate_slowest(slowest),
    }


def main() -> int:
    args = parse_args()
    events_path = Path(args.events)
    summary_path = Path(args.summary_out)
    summary = build_summary(load_events(events_path))
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
