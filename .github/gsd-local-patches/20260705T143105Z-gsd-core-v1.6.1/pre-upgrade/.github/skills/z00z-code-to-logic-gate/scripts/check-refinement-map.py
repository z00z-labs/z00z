#!/usr/bin/env python3
"""Validate report-local code-to-logic targets before running formal tools."""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
REPORT_STAMP = os.environ.get("Z00Z_REPORT_TIMESTAMP") or datetime.now(
    timezone.utc
).strftime("%Y%m%d-%H%M%S")
RUN_ROOT = Path(
    os.environ.get(
        "Z00Z_VERIFICATION_RUN_ROOT",
        str(ROOT / "reports" / f"z00z-verification-orchestrator-{REPORT_STAMP}"),
    )
).resolve()
VERIFICATION_ROOT = Path(
    os.environ.get(
        "Z00Z_VERIFICATION_RUNTIME_ROOT", str(RUN_ROOT / f"verification{REPORT_STAMP}")
    )
).resolve()

REQUIRED_KEYS = {"id", "tool", "required_status"}
ALLOWED_TOOLS = {"charon", "aeneas", "charon-aeneas", "crux-mir", "saw", "cryptol"}
ALLOWED_STATUSES = {
    "TESTED",
    "BOUNDED_VERIFIED",
    "MODEL_CHECKED",
    "FORMALLY_PROVED",
    "SECURITY_PROTOCOL_PROVED",
}


def load_yaml(path: Path) -> object:
    try:
        import yaml  # type: ignore
    except ImportError as exc:
        raise RuntimeError("PyYAML is required for code-to-logic target validation") from exc
    return yaml.safe_load(path.read_text(encoding="utf-8")) or {}


def resolve_targets_path(explicit: str | None) -> Path:
    if explicit:
        candidate = Path(explicit)
        if not candidate.is_absolute():
            candidate = (ROOT / candidate).resolve()
        return candidate
    return VERIFICATION_ROOT / "code-to-logic" / "targets.yaml"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--targets", help="path to code-to-logic targets.yaml")
    parser.add_argument("--json", action="store_true", help="emit machine-readable output")
    args = parser.parse_args()

    path = resolve_targets_path(args.targets)
    if not path.exists():
        result = {"status": "UNKNOWN", "message": f"targets file not found: {path}", "issues": []}
        if args.json:
            print(json.dumps(result, indent=2, sort_keys=True))
        else:
            print(f"UNKNOWN: {result['message']}")
        return 0

    data = load_yaml(path)
    targets = data.get("targets", []) if isinstance(data, dict) else []
    issues: list[str] = []

    if not isinstance(targets, list) or not targets:
        issues.append("no targets declared")
    else:
        for index, target in enumerate(targets):
            if not isinstance(target, dict):
                issues.append(f"target[{index}] is not a mapping")
                continue
            missing = sorted(REQUIRED_KEYS - set(target))
            if missing:
                issues.append(f"target[{index}] missing keys: {', '.join(missing)}")
            tool = str(target.get("tool", ""))
            if tool and tool not in ALLOWED_TOOLS:
                issues.append(f"target[{index}] uses unsupported tool: {tool}")
            required_status = str(target.get("required_status", ""))
            if required_status and required_status not in ALLOWED_STATUSES:
                issues.append(
                    f"target[{index}] uses unsupported required_status: {required_status}"
                )
            for key in ("manifest_path", "spec", "saw_script", "cryptol_script"):
                value = target.get(key)
                if not value:
                    continue
                resolved = Path(value)
                if not resolved.is_absolute():
                    resolved = (ROOT / value).resolve()
                try:
                    resolved.relative_to(ROOT)
                except ValueError:
                    issues.append(f"target[{index}] path escapes repository for {key}: {value}")
                    continue
                if not resolved.exists():
                    issues.append(f"target[{index}] missing path for {key}: {value}")

    result = {
        "status": "FAIL" if issues else "TESTED",
        "targets_path": path.as_posix(),
        "target_count": len(targets) if isinstance(targets, list) else 0,
        "issues": issues,
    }
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(f"[z00z-code-logic] targets: {result['target_count']}")
        if issues:
            for issue in issues:
                print(f"FAIL: {issue}")
        else:
            print("TESTED: refinement map is structurally valid")
    return 1 if issues else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
