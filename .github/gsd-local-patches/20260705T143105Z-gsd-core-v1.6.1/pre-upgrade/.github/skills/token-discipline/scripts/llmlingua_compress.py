#!/usr/bin/env python3
import argparse
import io
import json
import os
import sys
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

os.environ.setdefault("HF_HUB_OFFLINE", "1")
os.environ.setdefault("HF_HUB_DISABLE_PROGRESS_BARS", "1")
os.environ.setdefault("TRANSFORMERS_VERBOSITY", "error")

try:
    from transformers.utils import logging as transformers_logging
except ImportError:
    transformers_logging = None
else:
    transformers_logging.set_verbosity_error()

try:
    from llmlingua import PromptCompressor
except ImportError:
    print("ERROR: install dependency: uv pip install llmlingua", file=sys.stderr)
    sys.exit(2)


def read_text(path: str | None) -> str:
    if not path or path == "-":
        return sys.stdin.read()
    return Path(path).read_text(encoding="utf-8")


def build_context_segments(text: str) -> list[str]:
    segments = [line.strip() for line in text.splitlines() if line.strip()]
    return segments or [text]


def main() -> None:
    parser = argparse.ArgumentParser(description="Compress prompt text with llmlingua.")
    parser.add_argument("file", nargs="?", default="-")
    parser.add_argument("--instruction", default="Compress while preserving critical steps.")
    parser.add_argument("--question", default="What is the minimum correct answer?")
    parser.add_argument("--model", default="sshleifer/tiny-gpt2")
    parser.add_argument("--target-token", type=int, default=80)
    parser.add_argument("--rate", type=float, default=0.5)
    parser.add_argument(
        "--allow-download",
        action="store_true",
        help="Allow Hugging Face downloads when the model is not already cached locally.",
    )
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    text = read_text(args.file)
    context = build_context_segments(text)
    model_config = {"local_files_only": not args.allow_download}
    captured_output = io.StringIO()

    if args.allow_download:
        os.environ.pop("HF_HUB_OFFLINE", None)

    try:
        with redirect_stdout(captured_output), redirect_stderr(captured_output):
            compressor = PromptCompressor(
                model_name=args.model,
                device_map="cpu",
                model_config=model_config,
            )
            result = compressor.compress_prompt(
                context,
                instruction=args.instruction,
                question=args.question,
                target_token=args.target_token,
                rate=args.rate,
            )
    except OSError as exc:
        if not args.allow_download:
            print(
                "ERROR: model is not cached locally. Re-run with --allow-download to warm the cache once.",
                file=sys.stderr,
            )
            raise SystemExit(2) from exc
        raise
    except Exception:
        buffered_text = captured_output.getvalue().strip()
        if buffered_text:
            print(buffered_text, file=sys.stderr)
        raise

    payload = {
        "model": args.model,
        "cache_only": not args.allow_download,
        "context_segments": len(context),
        "origin_tokens": result.get("origin_tokens"),
        "compressed_tokens": result.get("compressed_tokens"),
        "ratio": result.get("ratio"),
        "rate": result.get("rate"),
        "saving": result.get("saving"),
        "compressed_prompt": result.get("compressed_prompt", ""),
    }

    if args.json:
        print(json.dumps(payload, indent=2))
        return

    print(f"model: {payload['model']}")
    print(f"origin_tokens: {payload['origin_tokens']}")
    print(f"compressed_tokens: {payload['compressed_tokens']}")
    print(f"ratio: {payload['ratio']}")
    print(payload["compressed_prompt"])


if __name__ == "__main__":
    main()