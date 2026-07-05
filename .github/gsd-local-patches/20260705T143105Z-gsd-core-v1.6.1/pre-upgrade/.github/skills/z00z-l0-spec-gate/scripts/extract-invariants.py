#!/usr/bin/env python3
"""Extract invariant-like statements from docs and Rust comments."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
ZINV_LINE_RE = re.compile(r"\bZINV[:\s]+(?P<id>[A-Z][A-Z0-9_-]+)\b(?P<rest>.*)")
INVARIANT_LINE_RE = re.compile(r"\bInvariant:\s*(?P<statement>.+)", re.IGNORECASE)


def candidate_files() -> list[Path]:
    roots = [ROOT / "docs", ROOT / "specs", ROOT / ".github" / "requirements", ROOT / "crates"]
    files: list[Path] = []
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.suffix in {".md", ".rs", ".yaml", ".yml"} and path.is_file():
                files.append(path)
    return sorted(files)


def extract() -> list[dict[str, str | int]]:
    records: list[dict[str, str | int]] = []
    for path in candidate_files():
        rel = path.relative_to(ROOT).as_posix()
        for index, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1):
            zinv = ZINV_LINE_RE.search(line)
            if zinv:
                records.append(
                    {
                        "id": zinv.group("id"),
                        "statement": zinv.group("rest").strip(" :-"),
                        "file": rel,
                        "line": index,
                        "source": "zinv",
                    }
                )
                continue
            invariant = INVARIANT_LINE_RE.search(line)
            if invariant:
                records.append(
                    {
                        "id": "",
                        "statement": invariant.group("statement").strip(),
                        "file": rel,
                        "line": index,
                        "source": "invariant-line",
                    }
                )
    return records


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, help="write JSON output to this path")
    args = parser.parse_args()

    records = extract()
    output = json.dumps({"count": len(records), "records": records}, indent=2, sort_keys=True)
    if args.out:
        out_path = ROOT / args.out if not args.out.is_absolute() else args.out
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(output + "\n", encoding="utf-8")
        print(f"[z00z-l0] wrote {len(records)} records to {out_path.relative_to(ROOT)}")
    else:
        print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
