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


def read_text(path: str | None) -> str:
    if not path or path == "-":
        return sys.stdin.read()
    return Path(path).read_text(encoding="utf-8")


def count_text(text: str, model: str) -> dict[str, int | str]:
    try:
        enc = tiktoken.encoding_for_model(model)
    except KeyError:
        enc = tiktoken.get_encoding("cl100k_base")

    return {
        "model": model,
        "tokens": len(enc.encode(text)),
        "chars": len(text),
        "words": len(text.split()),
    }


def main() -> None:
    p = argparse.ArgumentParser(description="Count tokens, words, and characters.")
    p.add_argument("file", nargs="?", default="-")
    p.add_argument("--model", default="gpt-5")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    text = read_text(args.file)
    stats = count_text(text, args.model)

    if args.json:
        print(json.dumps(stats, indent=2))
        return

    print(f"model: {stats['model']}")
    print(f"tokens: {stats['tokens']}")
    print(f"chars: {stats['chars']}")
    print(f"words: {stats['words']}")


if __name__ == "__main__":
    main()
