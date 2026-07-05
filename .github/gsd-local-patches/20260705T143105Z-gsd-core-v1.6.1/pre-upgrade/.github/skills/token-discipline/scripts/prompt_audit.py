#!/usr/bin/env python3
import argparse
import json
import re
import sys
from pathlib import Path


VERBOSE_PATTERNS = [
    r"\bsure[,! ]",
    r"\bcertainly[,! ]",
    r"\bas an ai\b",
    r"\bit is important to note\b",
    r"\bhere('?s| is)\b",
    r"\bin conclusion\b",
    r"\bto summarize\b",
    r"\blet'?s dive in\b",
    r"\bcomprehensive\b",
    r"\bdetailed explanation\b",
    r"\bstep[- ]by[- ]step guide\b",
    r"\bif you want\b",
    r"\bfeel free to\b",
]

WEAK_PATTERNS = [
    r"\bmaybe\b",
    r"\bpossibly\b",
    r"\bkind of\b",
    r"\bsort of\b",
    r"\bvarious\b",
    r"\bstuff\b",
    r"\bthings\b",
]


def line_hits(text: str, patterns: list[str]) -> list[tuple[int, str, str]]:
    hits = []
    for i, line in enumerate(text.splitlines(), start=1):
        low = line.lower()
        for pat in patterns:
            if re.search(pat, low):
                hits.append((i, pat, line.strip()))
    return hits


def duplicate_headings(text: str) -> list[str]:
    seen = set()
    dupes = []

    for line in text.splitlines():
        if line.lstrip().startswith("#"):
            normalized = re.sub(r"\s+", " ", line.strip().lower())
            if normalized in seen:
                dupes.append(line.strip())
            seen.add(normalized)

    return dupes


def main() -> None:
    p = argparse.ArgumentParser(description="Audit prompt or markdown text for avoidable verbosity.")
    p.add_argument("file")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    path = Path(args.file)
    text = path.read_text(encoding="utf-8")

    verbose = line_hits(text, VERBOSE_PATTERNS)
    weak = line_hits(text, WEAK_PATTERNS)
    dupes = duplicate_headings(text)
    payload = {
        "verbose_hits": [
            {"line": line_no, "pattern": pat, "text": line}
            for line_no, pat, line in verbose
        ],
        "weak_hits": [
            {"line": line_no, "pattern": pat, "text": line}
            for line_no, pat, line in weak
        ],
        "duplicate_headings": dupes,
        "ok": not verbose and not dupes,
    }

    failed = False

    if args.json:
        print(json.dumps(payload, indent=2))
        if not payload["ok"]:
            sys.exit(1)
        return

    if verbose:
        failed = True
        print("VERBOSE_PHRASES:")
        for line_no, pat, line in verbose:
            print(f"  L{line_no}: {line}")

    if weak:
        print("WEAK_PHRASES:")
        for line_no, pat, line in weak:
            print(f"  L{line_no}: {line}")

    if dupes:
        failed = True
        print("DUPLICATE_HEADINGS:")
        for h in dupes:
            print(f"  {h}")

    if failed:
        print("FAIL: prompt/instruction file contains avoidable verbosity", file=sys.stderr)
        sys.exit(1)

    print("OK: no major verbosity problems found")


if __name__ == "__main__":
    main()
