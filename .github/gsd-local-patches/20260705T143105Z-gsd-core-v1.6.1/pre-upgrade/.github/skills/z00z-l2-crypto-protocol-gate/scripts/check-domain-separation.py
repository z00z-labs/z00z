#!/usr/bin/env python3
"""Check Rust hash_domain labels and optional domain registry consistency."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
SPECS_ROOT = Path(
    os.environ.get("Z00Z_SPECS_ROOT", str(ROOT / "specs"))
).resolve()
DOMAIN_RE = re.compile(
    r"hash_domain!\s*\(\s*(?P<name>[A-Za-z0-9_]+)\s*,\s*\"(?P<label>[^\"]+)\"\s*,\s*(?P<version>[0-9]+)\s*\)",
    re.DOTALL,
)


def load_yaml(path: Path) -> Any:
    try:
        import yaml  # type: ignore
    except ImportError as exc:
        raise RuntimeError("PyYAML is required when specs/crypto/domain_separation_registry.yaml exists") from exc
    with path.open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle) or {}


def rust_domains(include_tests: bool) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        path_text = path.as_posix()
        if "/tari/" in path_text:
            continue
        if not include_tests and any(
            marker in path_text
            for marker in ("/tests/", "/benches/", "/examples/", "/fixture_support/", "/test_")
        ):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for match in DOMAIN_RE.finditer(text):
            records.append(
                {
                    "name": match.group("name"),
                    "label": match.group("label"),
                    "version": match.group("version"),
                    "file": path.relative_to(ROOT).as_posix(),
                }
            )
    return records


def registry_labels() -> set[str]:
    path = SPECS_ROOT / "crypto" / "domain_separation_registry.yaml"
    if not path.exists():
        return set()
    data = load_yaml(path)
    labels: set[str] = set()
    if isinstance(data, dict):
        for value in data.values():
            if isinstance(value, dict) and "label" in value:
                labels.add(str(value["label"]))
            elif isinstance(value, str):
                labels.add(value)
    elif isinstance(data, list):
        for item in data:
            if isinstance(item, dict) and "label" in item:
                labels.add(str(item["label"]))
    return labels


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="print machine-readable output")
    parser.add_argument("--strict-registry", action="store_true", help="fail when Rust labels are absent from the registry")
    parser.add_argument("--include-tests", action="store_true", help="include tests, benches, examples, and fixture support domains")
    args = parser.parse_args()

    records = rust_domains(args.include_tests)
    by_label: dict[str, list[dict[str, str]]] = {}
    for record in records:
        by_label.setdefault(record["label"], []).append(record)

    collisions: dict[str, list[dict[str, str]]] = {}
    repeated_same_name: dict[str, list[dict[str, str]]] = {}
    for label, items in by_label.items():
        names = {item["name"] for item in items}
        if len(names) > 1:
            collisions[label] = items
        elif len(items) > 1:
            repeated_same_name[label] = items

    registry = registry_labels()
    missing_from_registry = sorted({item["label"] for item in records} - registry) if registry else []

    result = {
        "domain_count": len(records),
        "label_count": len(by_label),
        "collisions": collisions,
        "repeated_same_name": repeated_same_name,
        "registry_label_count": len(registry),
        "missing_from_registry": missing_from_registry,
        "include_tests": args.include_tests,
    }

    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(f"[z00z-l2] Rust domains: {len(records)}")
        print(f"[z00z-l2] unique labels: {len(by_label)}")
        if repeated_same_name:
            print(f"[z00z-l2] repeated same-name labels: {', '.join(sorted(repeated_same_name))}")
        if collisions:
            print(f"[z00z-l2] label collisions: {', '.join(sorted(collisions))}")
        if registry and missing_from_registry:
            print(f"[z00z-l2] labels missing from registry: {len(missing_from_registry)}")

    if collisions:
        return 1
    if args.strict_registry and missing_from_registry:
        return 1
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
