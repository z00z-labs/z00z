#!/usr/bin/env python3
"""Create canonical English Help page templates without changing authored pages."""

from __future__ import annotations

import argparse
import json
import re
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
        "{{title}}": record["title"],
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


def scaffold(records: list[dict[str, str]], template: str) -> tuple[int, int]:
    created = 0
    preserved = 0
    for record in records:
        target = target_for(record)
        if target.exists():
            preserved += 1
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(render_page(template, record), encoding="utf-8")
        created += 1
    return created, preserved


def page_uses_title(source: str, title: str) -> bool:
    return all(
        marker in source
        for marker in (
            f"title: {title}\n",
            f"# {title}\n",
            f"![{title} application view](",
        )
    )


def synchronize_titles(records: list[dict[str, str]]) -> tuple[int, int]:
    updated = 0
    unchanged = 0

    for record in records:
        target = target_for(record)
        source = target.read_text(encoding="utf-8")
        title_match = re.search(r"^title: (.+)$", source, re.MULTILINE)
        if not title_match:
            raise ValueError(f"Missing Help title: {target}")

        old_title = title_match.group(1)
        new_title = record["title"]
        if page_uses_title(source, new_title):
            unchanged += 1
            continue

        expected_markers = (
            f"title: {old_title}",
            f"# {old_title}",
            f"![{old_title} application view]",
        )
        if any(marker not in source for marker in expected_markers):
            raise ValueError(f"Refusing to rewrite non-canonical Help title markers: {target}")

        source = source.replace(f"title: {old_title}", f"title: {new_title}", 1)
        source = source.replace(f"# {old_title}", f"# {new_title}", 1)
        source = source.replace(
            f"![{old_title} application view]",
            f"![{new_title} application view]",
            1,
        )
        target.write_text(source, encoding="utf-8")
        updated += 1

    return updated, unchanged


def main() -> None:
    parser = argparse.ArgumentParser()
    operation = parser.add_mutually_exclusive_group()
    operation.add_argument("--check", action="store_true")
    operation.add_argument("--sync-titles", action="store_true")
    arguments = parser.parse_args()
    records = parse_manifest(MANIFEST_PATH.read_text(encoding="utf-8"))
    template = TEMPLATE_PATH.read_text(encoding="utf-8")

    if arguments.check:
        missing = [record["file"] for record in records if not target_for(record).is_file()]
        if missing:
            raise SystemExit(f"Missing canonical English Help pages: {', '.join(missing)}")
        stale_titles = [
            record["file"]
            for record in records
            if not page_uses_title(
                target_for(record).read_text(encoding="utf-8"),
                record["title"],
            )
        ]
        if stale_titles:
            raise SystemExit(f"Stale canonical English Help titles: {', '.join(stale_titles)}")
        print(f"Canonical English Help pages ready: {len(records)}")
        return

    if arguments.sync_titles:
        updated, unchanged = synchronize_titles(records)
        print(f"Canonical English Help titles: updated={updated}, unchanged={unchanged}")
        return

    created, preserved = scaffold(records, template)
    print(f"Canonical English Help pages: created={created}, preserved={preserved}")


if __name__ == "__main__":
    main()
