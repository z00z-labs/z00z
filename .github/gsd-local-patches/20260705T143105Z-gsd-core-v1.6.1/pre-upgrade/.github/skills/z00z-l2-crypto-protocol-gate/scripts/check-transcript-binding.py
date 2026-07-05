#!/usr/bin/env python3
"""Check transcript binding specification coverage when crypto specs exist."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
SPECS_ROOT = Path(
    os.environ.get("Z00Z_SPECS_ROOT", str(ROOT / "specs"))
).resolve()
REQUIRED_FIELDS = {
    "protocol_id",
    "version",
    "proof_type",
    "domain",
    "root",
    "input",
    "output",
}


def load_yaml(path: Path) -> Any:
    try:
        import yaml  # type: ignore
    except ImportError as exc:
        raise RuntimeError("PyYAML is required when proof_objects_schema.yaml exists") from exc
    with path.open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle) or {}


def check_markdown(path: Path) -> dict[str, list[str]]:
    text = path.read_text(encoding="utf-8", errors="replace").lower()
    missing = sorted(field for field in REQUIRED_FIELDS if field not in text)
    return {path.relative_to(ROOT).as_posix(): missing}


def check_schema(path: Path) -> dict[str, list[str]]:
    data = load_yaml(path)
    failures: dict[str, list[str]] = {}
    if not isinstance(data, dict):
        failures[path.relative_to(ROOT).as_posix()] = sorted(REQUIRED_FIELDS)
        return failures

    proof_objects = data.get("proof_objects", data)
    if not isinstance(proof_objects, dict):
        failures[path.relative_to(ROOT).as_posix()] = sorted(REQUIRED_FIELDS)
        return failures

    for proof_name, proof_data in proof_objects.items():
        fields: set[str] = set()
        if isinstance(proof_data, dict):
            raw_fields = proof_data.get("binds", proof_data.get("fields", []))
            if isinstance(raw_fields, list):
                fields.update(str(item).lower() for item in raw_fields)
            elif isinstance(raw_fields, dict):
                fields.update(str(item).lower() for item in raw_fields.keys())
        missing = sorted(field for field in REQUIRED_FIELDS if not any(field in item for item in fields))
        if missing:
            failures[str(proof_name)] = missing
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="print machine-readable output")
    args = parser.parse_args()

    crypto_dir = SPECS_ROOT / "crypto"
    transcript_md = crypto_dir / "transcript_binding.md"
    proof_schema = crypto_dir / "proof_objects_schema.yaml"
    failures: dict[str, list[str]] = {}
    checked: list[str] = []

    if transcript_md.exists():
        checked.append(transcript_md.relative_to(ROOT).as_posix())
        failures.update({key: value for key, value in check_markdown(transcript_md).items() if value})

    if proof_schema.exists():
        checked.append(proof_schema.relative_to(ROOT).as_posix())
        failures.update(check_schema(proof_schema))

    result = {"checked": checked, "failures": failures, "required_fields": sorted(REQUIRED_FIELDS)}

    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        if not checked:
            print("[z00z-l2] UNKNOWN: no specs/crypto transcript binding files found")
        else:
            print(f"[z00z-l2] checked: {', '.join(checked)}")
            if failures:
                print(f"[z00z-l2] transcript binding gaps: {len(failures)}")

    return 1 if failures else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
