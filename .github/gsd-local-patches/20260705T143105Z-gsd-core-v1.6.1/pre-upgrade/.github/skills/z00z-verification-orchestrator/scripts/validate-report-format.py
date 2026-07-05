#!/usr/bin/env python3
"""Validate z00z-verification-report.md against the canonical report contract."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


REQUIRED_SECTIONS = [
    "# Z00Z Verification Orchestrator Report",
    "## 🎯 Executive Verdict",
    "## 📦 Evidence Provenance",
    "## 🚦 Gate Matrix",
    "## 🧪 Conclusion Ledger",
    "## 🔍 Validity And Doublecheck",
    "## 🏗️ Bootstrap Artifact Provenance",
    "## 📊 Performance And Resource Profiling",
    "## 🌲 HJMT Runtime Evidence",
    "## 🚨 Risk Register",
    "## 🔗 Supply-Chain Highlights",
    "## 🛡️ Adversarial Security Review",
    "## 🧰 Project-Owned Fixable Findings",
    "## 📚 Protected Vendor Findings",
    "## 🧩 Missing Evidence Or Missing Models",
    "## ✅ Recommended Actions",
    "## 📝 Execution Notes",
]

PROJECT_ONLY_SECTION = "## 🗺️ Coverage Inventory"

SECTION_TABLE_SNIPPETS = {
    "## 🚦 Gate Matrix": "| Gate | Checker module | Status | Elapsed (s) | Log | Primary artifacts |",
    "## 🧪 Conclusion Ledger": "| Gate | Checker module | Machine conclusion | Validity ceiling | Anchoring artifact |",
    "## 🚨 Risk Register": "| Class | Source | Severity | Rationale | Anchor |",
}

RUN_ROOT_RE = re.compile(r"reports/z00z-verification-orchestrator-\d{8}-\d{6}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True)
    parser.add_argument("--run-root", required=True)
    parser.add_argument("--root", required=True)
    parser.add_argument("--scope-kind", required=True, choices=["project", "crate", "report"])
    parser.add_argument("--format-path", required=True)
    parser.add_argument("--summary-out", required=True)
    return parser.parse_args()


def collect_headings(text: str) -> list[str]:
    return [line.strip() for line in text.splitlines() if line.startswith("#")]


def relative_to_root(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except Exception:
        return path.as_posix()


def required_artifact_paths(run_root: Path, root: Path, scope_kind: str) -> list[str]:
    candidates: list[Path] = [
        run_root / "logs",
        run_root / "profiling" / "events.tsv",
        run_root / "profiling" / "summary.json",
        run_root / "profiling" / "tool-availability.json",
        run_root / "profiling" / "resources",
        run_root / "profiling" / "resources-summary.json",
        run_root / "profiling" / "run-footprint.json",
        run_root / "profiling" / "hjmt-summary.json",
        run_root / "bootstrap-summary.json",
        run_root / "runtime-bootstrap-summary.json",
        run_root / "security" / "adversarial-summary.json",
        run_root / "security" / "adversarial-review.md",
    ]
    if scope_kind == "project":
        candidates.extend(
            [
                run_root / "coverage" / "manifest.tsv",
                run_root / "coverage" / "summary.json",
            ]
        )
    return [relative_to_root(path, root) for path in candidates if path.exists()]


def section_text(text: str, heading: str) -> str:
    marker = f"{heading}\n"
    start = text.find(marker)
    if start == -1:
        return ""
    start += len(marker)
    next_idx = text.find("\n## ", start)
    if next_idx == -1:
        return text[start:]
    return text[start:next_idx]


def parse_markdown_table_rows(section: str) -> list[list[str]]:
    rows: list[list[str]] = []
    for line in section.splitlines():
        if not line.startswith("|"):
            continue
        parts = [item.strip() for item in line.strip().strip("|").split("|")]
        if len(parts) < 2:
            continue
        if parts[0] in {"Gate", "Class"}:
            continue
        if set("".join(parts)) <= {"-", " "}:
            continue
        rows.append(parts)
    return rows


def validate_gate_and_ledger_paths(
    text: str,
    root: Path,
    errors: list[str],
    checks: dict[str, object],
) -> None:
    gate_rows = parse_markdown_table_rows(section_text(text, "## 🚦 Gate Matrix"))
    missing_matrix_paths: list[str] = []
    checked_matrix_paths = 0
    for row in gate_rows:
        if len(row) < 6:
            continue
        gate_id = row[0].strip("` ")
        log_path = row[4].strip("` ")
        artifact_cell = row[5].strip("` ")
        if log_path and log_path != "-":
            checked_matrix_paths += 1
            if not (root / log_path).exists():
                missing_matrix_paths.append(f"{gate_id}:log:{log_path}")
        for artifact in [item.strip() for item in artifact_cell.split(";")]:
            if not artifact or artifact == "-":
                continue
            checked_matrix_paths += 1
            if not (root / artifact).exists():
                missing_matrix_paths.append(f"{gate_id}:artifact:{artifact}")

    ledger_rows = parse_markdown_table_rows(section_text(text, "## 🧪 Conclusion Ledger"))
    missing_ledger_paths: list[str] = []
    checked_ledger_paths = 0
    for row in ledger_rows:
        if len(row) < 5:
            continue
        gate_id = row[0].strip("` ")
        anchor = row[4].strip("` ")
        if not anchor or anchor == "-":
            continue
        checked_ledger_paths += 1
        if not (root / anchor).exists():
            missing_ledger_paths.append(f"{gate_id}:anchor:{anchor}")

    checks["gate_matrix_paths_checked"] = checked_matrix_paths
    checks["conclusion_ledger_paths_checked"] = checked_ledger_paths
    if missing_matrix_paths:
      errors.append(
          "gate matrix references missing paths: " + ", ".join(missing_matrix_paths[:12])
      )
    if missing_ledger_paths:
      errors.append(
          "conclusion ledger references missing anchors: " + ", ".join(missing_ledger_paths[:12])
      )


def validate_report(
    text: str,
    report_path: Path,
    run_root: Path,
    root: Path,
    scope_kind: str,
) -> tuple[list[str], list[str], dict[str, object]]:
    errors: list[str] = []
    warnings: list[str] = []
    checks: dict[str, object] = {}

    headings = collect_headings(text)
    checks["heading_count"] = len(headings)
    checks["headings"] = headings

    expected = REQUIRED_SECTIONS[:]
    if scope_kind == "project":
        expected.insert(9, PROJECT_ONLY_SECTION)

    cursor = 0
    for heading in expected:
        try:
            idx = headings.index(heading, cursor)
        except ValueError:
            errors.append(f"missing required heading: {heading}")
            continue
        if idx < cursor:
            errors.append(f"heading out of order: {heading}")
            continue
        cursor = idx + 1

    if scope_kind != "project" and PROJECT_ONLY_SECTION in headings:
        warnings.append("coverage section is present for non-project scope")

    for heading, snippet in SECTION_TABLE_SNIPPETS.items():
        if heading in text and snippet not in text:
            errors.append(f"section {heading} is missing expected table header")

    run_root_rel = relative_to_root(run_root, root)
    checks["run_root"] = run_root_rel
    if run_root_rel not in text:
        errors.append(f"report does not mention active run root: {run_root_rel}")

    if relative_to_root(report_path.parent, root) != run_root_rel:
        errors.append("report path is not located under the declared active run root")

    referenced_run_roots = sorted(set(RUN_ROOT_RE.findall(text)))
    checks["referenced_run_roots"] = referenced_run_roots
    foreign = [item for item in referenced_run_roots if item != run_root_rel]
    if foreign:
        errors.append(f"report references foreign run roots: {', '.join(foreign)}")

    required_paths = required_artifact_paths(run_root, root, scope_kind)
    checks["required_artifact_paths"] = required_paths
    missing_paths = [item for item in required_paths if item not in text]
    if missing_paths:
        errors.append(
            "report is missing references to run-root artifacts: " + ", ".join(missing_paths[:12])
        )

    validate_gate_and_ledger_paths(text, root, errors, checks)

    return errors, warnings, checks


def main() -> int:
    args = parse_args()
    report_path = Path(args.report).resolve()
    run_root = Path(args.run_root).resolve()
    root = Path(args.root).resolve()
    summary_path = Path(args.summary_out).resolve()

    text = report_path.read_text(encoding="utf-8", errors="replace")
    errors, warnings, checks = validate_report(text, report_path, run_root, root, args.scope_kind)

    summary = {
        "ok": not errors,
        "errors": errors,
        "warnings": warnings,
        "checks": checks,
        "report": relative_to_root(report_path, root),
        "run_root": relative_to_root(run_root, root),
        "format_path": args.format_path,
    }
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    for warning in warnings:
        print(f"WARNING: {warning}")
    print(f"OK: report contract validated for {relative_to_root(report_path, root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
