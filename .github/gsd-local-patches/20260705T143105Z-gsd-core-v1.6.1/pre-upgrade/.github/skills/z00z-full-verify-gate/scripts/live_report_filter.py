#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path


SUMMARY_RE = re.compile(r"^\[summary\]\s+planned=(\d+)\s+skipped=(\d+)\s+failed=(\d+)$")


def parse_args():
    parser = argparse.ArgumentParser(description="Update full verify live report sections.")
    parser.add_argument("--report", required=True, help="Path to the report file.")
    parser.add_argument("--label", required=True, help="Stage label prefix.")
    parser.add_argument(
        "--threshold",
        type=int,
        default=60,
        help="Minimum harness 'running over N seconds' value to record.",
    )
    return parser.parse_args()


def insert_under_section(report_path: Path, section_name: str, line: str) -> None:
    text = report_path.read_text(encoding="utf-8")
    marker = f"{section_name}:\n"
    if marker not in text:
        if not text.endswith("\n"):
            text += "\n"
        text += f"\n{marker}"

    index = text.index(marker) + len(marker)
    text = text[:index] + line + "\n" + text[index:]
    report_path.write_text(text, encoding="utf-8")


def format_harness_signal(label: str, normalized: str, raw_over_sec: int) -> str:
    return f"{label} harness: {normalized} (raw over={raw_over_sec}s)"


def format_summary_signal(label: str, line: str) -> str:
    match = SUMMARY_RE.match(line)
    if match is None:
        return f"{label} summary: {line}"

    planned, skipped, failed = match.groups()
    return (
        f"{label} summary: {planned} planned, {skipped} skipped, {failed} failed"
    )


def main() -> int:
    args = parse_args()
    report_path = Path(args.report)
    over_re = re.compile(r"has been running for over\s+(\d+)\s+seconds")

    for raw_line in sys.stdin:
        line = raw_line.rstrip("\n")
        match = over_re.search(line)
        if match is not None:
            over_sec = int(match.group(1))
            if over_sec >= args.threshold:
                # Keep report output aligned with configured threshold while preserving raw libtest value.
                normalized = over_re.sub(
                    f"has been running for over {args.threshold} seconds",
                    line,
                    count=1,
                )
                insert_under_section(
                    report_path,
                    "HarnessSignals",
                    format_harness_signal(args.label, normalized, over_sec),
                )
        elif line.startswith("[task] fail:"):
            insert_under_section(report_path, "TaskFailures", f"{args.label} | task-fail | {line}")
        elif line.startswith("[summary] failed ids:"):
            insert_under_section(report_path, "TaskFailures", f"{args.label} | summary | {line}")
        elif line.startswith("[summary] planned="):
            insert_under_section(
                report_path,
                "HarnessSignals",
                format_summary_signal(args.label, line),
            )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())