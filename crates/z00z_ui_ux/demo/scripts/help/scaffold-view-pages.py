#!/usr/bin/env python3
"""Create canonical English Help page templates without changing authored pages."""

from __future__ import annotations

import argparse
import json
import re
import shutil
from pathlib import Path


DEMO_ROOT = Path(__file__).resolve().parents[2]
HELP_ROOT = DEMO_ROOT / "help"
MANIFEST_PATH = HELP_ROOT / "topics.yaml"
TEMPLATE_PATH = HELP_ROOT / "TEMPLATE.md"
TOPIC_ID = re.compile(r"^[a-z][a-z0-9.-]*$")


def parse_manifest(source: str) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    current: dict[str, str] | None = None

    for raw_line in source.splitlines():
        if raw_line.startswith("  - id: "):
            if current:
                records.append(current)
            current = {"id": raw_line.removeprefix("  - id: ").strip()}
            continue
        if raw_line.startswith("    ") and current:
            key, separator, value = raw_line.strip().partition(": ")
            if separator:
                current[key] = value

    if current:
        records.append(current)

    if not records or any(not TOPIC_ID.fullmatch(record["id"]) for record in records):
        raise ValueError("Help navigation manifest is invalid")
    return records


def title_for(topic_id: str) -> str:
    return " ".join(part.capitalize() for part in re.split(r"[.-]", topic_id))


def asset_for(topic_id: str) -> str:
    return topic_id.replace(".", "-")


def render_page(template: str, record: dict[str, str]) -> str:
    source = json.dumps(
        {
            "page_path": record["file"],
            "route_id": record["route"],
            "screenshot": f"help/assets/en/{asset_for(record['id'])}.png",
            "topic_id": record["id"],
        },
        separators=(",", ":"),
        sort_keys=True,
    )
    replacements = {
        "{{id}}": record["id"],
        "{{title}}": title_for(record["id"]),
        "{{route}}": record["route"],
        "{{scope}}": record["scope"],
        "{{screenshot}}": f"help/assets/en/{asset_for(record['id'])}.png",
        "{{source}}": source,
    }
    result = template
    for marker, value in replacements.items():
        result = result.replace(marker, value)
    return result


def target_for(record: dict[str, str]) -> Path:
    relative = Path(record["file"])
    if relative.suffix != ".md" or ".." in relative.parts:
        raise ValueError(f"Unsafe Help page path: {record['file']}")
    return HELP_ROOT / "en" / relative


def scaffold(records: list[dict[str, str]], template: str, bootstrap: bool) -> tuple[int, int]:
    created = 0
    preserved = 0
    for record in records:
        target = target_for(record)
        if target.exists():
            if not bootstrap:
                preserved += 1
                continue
            backup = target.with_name(f"{target.name}.bak")
            if backup.exists():
                raise FileExistsError(f"Refusing to replace {target}; backup already exists at {backup}")
            shutil.copy2(target, backup)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(render_page(template, record), encoding="utf-8")
        created += 1
    return created, preserved


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bootstrap", action="store_true")
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    records = parse_manifest(MANIFEST_PATH.read_text(encoding="utf-8"))
    template = TEMPLATE_PATH.read_text(encoding="utf-8")

    if arguments.check:
        missing = [record["file"] for record in records if not target_for(record).is_file()]
        if missing:
            raise SystemExit(f"Missing canonical English Help pages: {', '.join(missing)}")
        print(f"Canonical English Help pages ready: {len(records)}")
        return

    created, preserved = scaffold(records, template, arguments.bootstrap)
    print(f"Canonical English Help pages: created={created}, preserved={preserved}")


if __name__ == "__main__":
    main()
