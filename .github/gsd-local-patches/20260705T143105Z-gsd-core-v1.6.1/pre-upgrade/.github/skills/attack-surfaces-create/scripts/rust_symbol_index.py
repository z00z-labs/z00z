#!/usr/bin/env python3
"""Create a lightweight Rust symbol inventory without external packages."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Iterable


SKIP_DIRS = {
    ".git",
    ".hg",
    ".svn",
    "target",
    "node_modules",
    ".planning",
    ".graphify",
}

ITEM_RE = re.compile(
    r"^\s*"
    r"(?P<vis>pub(?:\([^)]*\))?\s+)?"
    r"(?P<prefix>(?:(?:async|const|unsafe)\s+)*)"
    r"(?P<kind>fn|struct|enum|trait|impl|type|const|static|mod)\b"
    r"\s*(?P<rest>.*)"
)

MACRO_RE = re.compile(r"^\s*(?P<vis>pub\s+)?macro_rules!\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")

ROLE_PATTERNS = [
    ("parser", re.compile(r"parse|parser|decode|from_bytes|deserialize|serde", re.I)),
    ("serializer", re.compile(r"serialize|encode|to_bytes|canonical", re.I)),
    ("validator", re.compile(r"validate|verify|check|assert|ensure", re.I)),
    ("auth", re.compile(r"auth|permission|capability|role|policy", re.I)),
    ("crypto", re.compile(r"crypto|hash|sign|verify|proof|commit|nonce|random|rng|key|secret|transcript|nullifier", re.I)),
    ("storage", re.compile(r"store|storage|db|database|persist|read|write|state", re.I)),
    ("network", re.compile(r"network|http|rpc|api|socket|request|response|peer", re.I)),
    ("concurrency", re.compile(r"async|await|spawn|mutex|rwlock|atomic|channel|thread|tokio", re.I)),
    ("unsafe", re.compile(r"\bunsafe\b|ffi|extern\s+\"C\"|static\s+mut", re.I)),
    ("panic-path", re.compile(r"\bpanic!\b|\.unwrap\(|\.expect\(", re.I)),
]


def iter_rs_files(paths: list[Path]) -> Iterable[Path]:
    for start in paths:
        if start.is_file() and start.suffix == ".rs":
            yield start
            continue
        if not start.is_dir():
            continue
        for root, dirs, files in os.walk(start):
            dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
            for name in files:
                path = Path(root) / name
                if path.suffix == ".rs":
                    yield path


def find_crate_name(path: Path) -> str:
    for parent in [path.parent, *path.parents]:
        manifest = parent / "Cargo.toml"
        if not manifest.exists():
            continue
        try:
            text = manifest.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return parent.name
        in_package = False
        for line in text.splitlines():
            stripped = line.strip()
            if stripped == "[package]":
                in_package = True
                continue
            if stripped.startswith("[") and stripped != "[package]":
                in_package = False
            if in_package and stripped.startswith("name"):
                parts = stripped.split("=", 1)
                if len(parts) == 2:
                    return parts[1].strip().strip('"')
        return parent.name
    return ""


def extract_name(kind: str, prefix: str, rest: str) -> tuple[str, str]:
    if kind == "const" and rest.lstrip().startswith("fn "):
        kind = "fn"
        prefix = f"{prefix}const "
        rest = rest.lstrip()[3:]

    if kind == "impl":
        name = rest.split("{", 1)[0].strip()
        return kind, name or "<anonymous impl>"

    match = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", rest)
    if match:
        return kind, match.group(1)
    return kind, "<anonymous>"


def classify_roles(text: str) -> list[str]:
    roles: list[str] = []
    for role, pattern in ROLE_PATTERNS:
        if pattern.search(text):
            roles.append(role)
    return roles


def index_file(path: Path, root: Path) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    attrs: list[str] = []
    brace_depth = 0
    test_module_depth: int | None = None
    crate = find_crate_name(path)
    rel_path = str(path.relative_to(root)) if path.is_relative_to(root) else str(path)
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError as exc:
        return [{
            "path": rel_path,
            "line": 0,
            "crate": crate,
            "kind": "read-error",
            "visibility": "",
            "name": str(exc),
            "roles": [],
            "test_only": False,
        }]

    for line_no, line in enumerate(lines, start=1):
        stripped = line.strip()
        in_test_module = test_module_depth is not None
        if stripped.startswith("#["):
            attrs.append(stripped)
            continue

        macro_match = MACRO_RE.match(line)
        item_match = ITEM_RE.match(line)
        if not macro_match and not item_match:
            attrs.clear()
            brace_depth += line.count("{") - line.count("}")
            if test_module_depth is not None and brace_depth < test_module_depth:
                test_module_depth = None
            continue

        module_is_test = False
        if macro_match:
            kind = "macro_rules"
            prefix = ""
            rest = macro_match.group("name")
            visibility = "pub" if macro_match.group("vis") else "private"
            name = rest
        else:
            assert item_match is not None
            visibility = item_match.group("vis").strip() if item_match.group("vis") else "private"
            prefix = item_match.group("prefix") or ""
            kind, name = extract_name(item_match.group("kind"), prefix, item_match.group("rest"))
            module_is_test = kind == "mod" and (name == "tests" or name.startswith("test_"))

        role_text = " ".join([line, name, *attrs])
        test_only = (
            any("cfg(test)" in attr or "test]" in attr for attr in attrs)
            or "/tests/" in rel_path
            or in_test_module
            or module_is_test
            or name.startswith("test_")
        )
        records.append({
            "path": rel_path,
            "line": line_no,
            "crate": crate,
            "kind": f"{prefix.strip()} {kind}".strip(),
            "visibility": visibility,
            "name": name,
            "roles": classify_roles(role_text),
            "test_only": test_only,
        })

        if module_is_test or any("cfg(test)" in attr for attr in attrs):
            test_module_depth = brace_depth + max(line.count("{"), 1)
        brace_depth += line.count("{") - line.count("}")
        if test_module_depth is not None and brace_depth < test_module_depth:
            test_module_depth = None
        attrs.clear()
    return records


def markdown(records: list[dict[str, object]]) -> str:
    def cell(value: object) -> str:
        if isinstance(value, list):
            value = ", ".join(str(v) for v in value)
        return str(value).replace("|", "\\|")

    lines = [
        "# Rust Symbol Index",
        "",
        "| Path | Line | Crate | Kind | Visibility | Name | Roles | Test Only |",
        "| --- | ---: | --- | --- | --- | --- | --- | --- |",
    ]
    for record in records:
        lines.append(
            "| {path} | {line} | {crate} | {kind} | {visibility} | {name} | {roles} | {test_only} |".format(
                path=cell(record["path"]),
                line=cell(record["line"]),
                crate=cell(record["crate"]),
                kind=cell(record["kind"]),
                visibility=cell(record["visibility"]),
                name=cell(record["name"]),
                roles=cell(record["roles"]),
                test_only=cell(record["test_only"]),
            )
        )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Index Rust symbols for attack-surface inventory.")
    parser.add_argument("scope", nargs="+", help="Rust file, crate, or workspace path")
    parser.add_argument("--format", choices=["markdown", "jsonl"], default="markdown")
    args = parser.parse_args()

    root = Path.cwd().resolve()
    scopes = [Path(p).resolve() for p in args.scope]
    records: list[dict[str, object]] = []
    for path in sorted(set(iter_rs_files(scopes))):
        records.extend(index_file(path, root))

    if args.format == "jsonl":
        for record in records:
            print(json.dumps(record, ensure_ascii=True))
    else:
        print(markdown(records))
    return 0


if __name__ == "__main__":
    sys.exit(main())
