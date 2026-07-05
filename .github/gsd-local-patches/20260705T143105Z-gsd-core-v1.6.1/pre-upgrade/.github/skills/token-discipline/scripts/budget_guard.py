#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path

try:
    import tiktoken
except ImportError:
    print("ERROR: install dependency: uv pip install tiktoken", file=sys.stderr)
    sys.exit(2)


BUDGETS = {
    "tiny": 60,
    "compact": 150,
    "standard": 440,
    "deep": 1200,
    "patch_only": 260,
}


def read_text(path: str | None) -> str:
    if not path or path == "-":
        return sys.stdin.read()
    return Path(path).read_text(encoding="utf-8")


def token_count(text: str, model: str) -> int:
    try:
        enc = tiktoken.encoding_for_model(model)
    except KeyError:
        enc = tiktoken.get_encoding("cl100k_base")
    return len(enc.encode(text))


def main() -> None:
    p = argparse.ArgumentParser(
        description="Fail if a draft exceeds the approximate token ceiling for a visible mode budget."
    )
    p.add_argument("file", nargs="?", default="-")
    p.add_argument("--mode", choices=BUDGETS.keys(), default="compact")
    p.add_argument("--max-tokens", type=int)
    p.add_argument("--model", default="gpt-5")
    p.add_argument("--warn-at", type=float, default=0.85)
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    text = read_text(args.file)
    count = token_count(text, args.model)
    limit = args.max_tokens or BUDGETS[args.mode]
    warning = count >= int(limit * args.warn_at)
    payload = {
        "mode": args.mode,
        "tokens": count,
        "limit": limit,
        "warning": warning,
        "ok": count <= limit,
    }

    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"mode: {args.mode}")
        print(f"tokens: {count}")
        print(f"limit: {limit}")
        if warning and count <= limit:
            print("WARN: draft is close to the limit")

    if count > limit:
        print(f"FAIL: token budget exceeded: {count} > {limit}", file=sys.stderr)
        sys.exit(1)

    if not args.json:
        print("OK")


if __name__ == "__main__":
    main()