#!/usr/bin/env python3
"""Check Z00Z invariant IDs and code traceability."""

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
INVARIANT_DIR = SPECS_ROOT / "invariants"
ZINV_RE = re.compile(r"\bZINV[:\s]+([A-Z][A-Z0-9_-]+)\b")
CRITICAL_PREFIXES = (
    "crates/z00z_storage/",
    "crates/z00z_crypto/",
    "crates/z00z_core/",
    "crates/z00z_wallets/",
    "crates/z00z_rollup_node/",
    "crates/z00z_runtime/",
)


def load_yaml(path: Path) -> Any:
    try:
        import yaml  # type: ignore
    except ImportError:
        return parse_minimal_yaml_ids(path)
    with path.open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle) or {}


def parse_minimal_yaml_ids(path: Path) -> dict[str, dict[str, str]]:
    ids: dict[str, dict[str, str]] = {}
    key_re = re.compile(r"^([A-Z][A-Z0-9_-]+):\s*$")
    for line in path.read_text(encoding="utf-8").splitlines():
        match = key_re.match(line)
        if match:
            ids[match.group(1)] = {}
    return ids


def collect_invariant_ids() -> set[str]:
    ids: set[str] = set()
    if not INVARIANT_DIR.exists():
        return ids
    for path in sorted(INVARIANT_DIR.rglob("*")):
        if path.suffix not in {".yaml", ".yml"}:
            continue
        data = load_yaml(path)
        if isinstance(data, dict):
            ids.update(str(key) for key in data.keys())
        elif isinstance(data, list):
            for item in data:
                if isinstance(item, dict) and "id" in item:
                    ids.add(str(item["id"]))
    return ids


def changed_files(path: Path | None) -> list[Path]:
    if path is not None:
        return [ROOT / line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    return [p for p in ROOT.glob("crates/**/*.rs") if p.is_file()]


def is_critical(path: Path) -> bool:
    rel = path.relative_to(ROOT).as_posix()
    return rel.endswith(".rs") and rel.startswith(CRITICAL_PREFIXES)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict", action="store_true", help="fail on missing invariant directory or missing ZINV in changed critical files")
    parser.add_argument("--changed-file-list", type=Path, help="newline-delimited list of changed files relative to repo root")
    parser.add_argument("--json", action="store_true", help="print machine-readable output")
    args = parser.parse_args()

    invariant_ids = collect_invariant_ids()
    files = [p for p in changed_files(args.changed_file_list) if p.exists() and p.is_file()]
    references: dict[str, list[str]] = {}
    missing_refs: dict[str, list[str]] = {}
    critical_without_ref: list[str] = []

    for path in files:
        if path.suffix != ".rs":
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        found = ZINV_RE.findall(text)
        rel = path.relative_to(ROOT).as_posix()
        if found:
            for invariant_id in found:
                references.setdefault(invariant_id, []).append(rel)
                if invariant_ids and invariant_id not in invariant_ids:
                    missing_refs.setdefault(invariant_id, []).append(rel)
        elif args.strict and is_critical(path):
            critical_without_ref.append(rel)

    result = {
        "invariant_count": len(invariant_ids),
        "reference_count": sum(len(paths) for paths in references.values()),
        "missing_references": missing_refs,
        "critical_without_zinv": critical_without_ref,
        "strict": args.strict,
    }

    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(f"[z00z-l0] invariants: {len(invariant_ids)}")
        print(f"[z00z-l0] ZINV references: {result['reference_count']}")
        if missing_refs:
            print(f"[z00z-l0] missing invariant IDs: {', '.join(sorted(missing_refs))}")
        if critical_without_ref:
            print(f"[z00z-l0] critical files without ZINV: {len(critical_without_ref)}")

    if args.strict and not invariant_ids:
        print(f"ERROR: strict mode requires invariant YAML under {INVARIANT_DIR}", file=sys.stderr)
        return 1
    if missing_refs or critical_without_ref:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
