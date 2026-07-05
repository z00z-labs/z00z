#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path


FILLER_LINES = [
    r"^\s*sure[,!.\s]*$",
    r"^\s*certainly[,!.\s]*$",
    r"^\s*here('?s| is) .*$",
    r"^\s*let('?s)? dive in\.?\s*$",
    r"^\s*in conclusion[,.:]?\s*.*$",
    r"^\s*to summarize[,.:]?\s*.*$",
]


def compact_markdown(text: str) -> str:
    lines = text.splitlines()
    out = []
    in_code = False
    previous_blank = False

    for line in lines:
        stripped = line.strip()

        if stripped.startswith("```"):
            in_code = not in_code
            out.append(line.rstrip())
            previous_blank = False
            continue

        if in_code:
            out.append(line.rstrip())
            continue

        if any(re.match(pat, stripped, flags=re.I) for pat in FILLER_LINES):
            continue

        line = re.sub(r"\s+$", "", line)

        if not line:
            if previous_blank:
                continue
            previous_blank = True
            out.append("")
            continue

        previous_blank = False
        out.append(line)

    result = "\n".join(out).strip() + "\n"

    result = re.sub(r"\n{3,}", "\n\n", result)
    return result


def main() -> None:
    p = argparse.ArgumentParser(description="Remove obvious filler and collapse blank lines in Markdown.")
    p.add_argument("input", nargs="?", default="-")
    p.add_argument("--out")
    p.add_argument("--in-place", action="store_true")
    args = p.parse_args()

    if args.in_place and (not args.input or args.input == "-"):
        raise SystemExit("--in-place requires a file path")

    if args.in_place and args.out:
        raise SystemExit("Use either --out or --in-place, not both")

    if not args.in_place and args.input != "-" and not args.out:
        raise SystemExit("Provide --out for file-to-file compaction or use --in-place")

    if args.input == "-":
        text = sys.stdin.read()
    else:
        text = Path(args.input).read_text(encoding="utf-8")

    compacted = compact_markdown(text)

    if args.in_place:
        Path(args.input).write_text(compacted, encoding="utf-8")
        print(f"wrote: {args.input}")
        return

    if args.out:
        Path(args.out).write_text(compacted, encoding="utf-8")
        print(f"wrote: {args.out}")
        return

    sys.stdout.write(compacted)


if __name__ == "__main__":
    main()
