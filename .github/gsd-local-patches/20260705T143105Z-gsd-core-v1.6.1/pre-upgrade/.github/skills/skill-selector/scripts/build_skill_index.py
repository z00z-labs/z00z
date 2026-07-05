#!/usr/bin/env python3
"""Build and query the skill index used by the skill-selector skill."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable

import yaml

from executor import ChainExecution, DispatchTarget, RecordingDispatcher, SkillDispatcher, execute_chain
from router import build_chain, detect_primary_actions, rerank_results, score_entry, split_words, tokenize


WORKSPACE_ROOT = Path("/home/vadim/Projects/z00z")
LOCAL_SKILLS_ROOT = WORKSPACE_ROOT / ".github" / "skills"
WORKSPACE_AGENT_SKILLS_ROOT = WORKSPACE_ROOT / ".agents" / "skills"
SKILL_SELECTOR_ROOT = LOCAL_SKILLS_ROOT / "skill-selector"
USER_AGENT_SKILLS_ROOT = Path("/home/vadim/.agents/skills")
AGENT_TARGET_OVERRIDES = {
    "doublecheck": DispatchTarget(
        skill_name="doublecheck",
        target_kind="agent",
        target_name="Doublecheck",
    ),
}
INDEX_PATH = SKILL_SELECTOR_ROOT / "skills-index.json"
CATALOG_PATH = LOCAL_SKILLS_ROOT / "SKILLS_CONTENT.MD"

META_WORDS = {
    "selector",
    "builder",
    "blueprint",
    "generator",
    "planner",
    "plan",
    "orchestrator",
    "workflow",
    "guide",
    "template",
    "prompt",
}

SECTION_NAMES = {
    "when to use",
    "activation",
    "auto-invoke conditions",
    "trigger words",
    "auto-invoke triggers",
    "common triggers",
    "when to use this skill",
}


@dataclass(frozen=True)
class SkillSource:
    root: Path
    scope: str
    source_label: str
    source_priority: int


SKILL_SOURCES = (
    SkillSource(LOCAL_SKILLS_ROOT, "workspace", "workspace-github", 0),
    SkillSource(WORKSPACE_AGENT_SKILLS_ROOT, "workspace", "workspace-agents", 1),
    SkillSource(USER_AGENT_SKILLS_ROOT, "user", "user-agents", 2),
    SkillSource(Path("/home/vadim/.config/Code/agentPlugins"), "external", "external-agentplugins", 3),
    SkillSource(Path("/home/vadim/.vscode/extensions"), "external", "external-vscode-extensions", 4),
)


@dataclass
class SkillEntry:
    name: str
    description: str
    scope: str
    source_root: str
    source_label: str
    source_priority: int
    skill_dir: str
    skill_md: str
    keywords: list[str]
    trigger_phrases: list[str]
    sections: dict[str, str]
    metadata: dict[str, object]
    meta_skill: bool
    manifest_type: str | None
    agent_display_name: str | None

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "description": self.description,
            "scope": self.scope,
            "source_root": self.source_root,
            "source_label": self.source_label,
            "source_priority": self.source_priority,
            "skill_dir": self.skill_dir,
            "skill_md": self.skill_md,
            "keywords": self.keywords,
            "trigger_phrases": self.trigger_phrases,
            "sections": self.sections,
            "metadata": self.metadata,
            "meta_skill": self.meta_skill,
            "manifest_type": self.manifest_type,
            "agent_display_name": self.agent_display_name,
        }


def normalize_phrase(text: str) -> str:
    text = re.sub(r"\s+", " ", text.strip())
    return text.strip("`\"' ")


def meaningful_tokens(text: str) -> list[str]:
    return tokenize(text)


def load_skill_markdown(skill_md: Path) -> tuple[dict[str, object], str]:
    content = skill_md.read_text(encoding="utf-8")
    if not content.startswith("---\n"):
        return {}, content
    parts = content.split("---\n", 2)
    if len(parts) < 3:
        return {}, content
    frontmatter_text = parts[1]
    try:
        frontmatter = yaml.safe_load(frontmatter_text) or {}
    except yaml.YAMLError:
        frontmatter = {}
        for line in frontmatter_text.splitlines():
            if ":" not in line:
                continue
            key, value = line.split(":", 1)
            key = key.strip()
            if key not in {"name", "description"}:
                continue
            frontmatter[key] = normalize_phrase(value)
    body = parts[2]
    return frontmatter, body


def load_skill_manifest(skill_dir: Path) -> dict[str, object]:
    manifest_path = skill_dir / "skill-manifest.yaml"
    if not manifest_path.exists():
        return {}
    try:
        return yaml.safe_load(manifest_path.read_text(encoding="utf-8")) or {}
    except yaml.YAMLError:
        return {}


def normalize_metadata(frontmatter: dict[str, object]) -> dict[str, object]:
    metadata = frontmatter.get("metadata")
    if not isinstance(metadata, dict):
        return {}
    return metadata


def maybe_store_section(
    sections: dict[str, str], current_name: str, current_lines: list[str]
) -> None:
    if current_name and current_name in SECTION_NAMES and current_lines:
        sections[current_name] = "\n".join(current_lines).strip()


def extract_sections(body: str) -> dict[str, str]:
    sections: dict[str, str] = {}
    current_name = ""
    current_lines: list[str] = []

    for raw_line in body.splitlines():
        heading = re.match(r"^##+\s+(.*)$", raw_line)
        if heading:
            maybe_store_section(sections, current_name, current_lines)
            current_name = normalize_phrase(heading.group(1)).lower()
            current_lines = []
            continue
        if current_name:
            current_lines.append(raw_line)

    maybe_store_section(sections, current_name, current_lines)

    return sections


def normalize_trigger_line(line: str) -> str:
    stripped = normalize_phrase(line)
    if not stripped:
        return ""
    if re.match(r"^(-|\*|\d+\.)\s+", stripped):
        stripped = re.sub(r"^(-|\*|\d+\.)\s+", "", stripped)
    return normalize_phrase(stripped).lower()


def is_useful_phrase(phrase: str) -> bool:
    tokens = meaningful_tokens(phrase)
    return 2 <= len(tokens) <= 12


def unique_phrases(phrases: Iterable[str], limit: int) -> list[str]:
    deduped: list[str] = []
    seen: set[str] = set()
    for phrase in phrases:
        if phrase and phrase not in seen:
            deduped.append(phrase)
            seen.add(phrase)
    return deduped[:limit]


def extract_trigger_phrases(description: str, sections: dict[str, str], body: str) -> list[str]:
    phrases: list[str] = []
    for chunk in [description, *sections.values()]:
        for line in chunk.splitlines():
            stripped = normalize_trigger_line(line)
            if not stripped:
                continue
            if is_useful_phrase(stripped):
                phrases.append(stripped)

    quoted = re.findall(r'"([^"]{3,100})"', body)
    phrases.extend(
        normalized
        for normalized in (normalize_phrase(item).lower() for item in quoted)
        if is_useful_phrase(normalized)
    )
    return unique_phrases(phrases, 40)


def extract_keywords(name: str, description: str, phrases: Iterable[str]) -> list[str]:
    keywords: list[str] = []
    seen: set[str] = set()

    for source in [name, description, *phrases]:
        for token in tokenize(source):
            if token not in seen:
                keywords.append(token)
                seen.add(token)
    return keywords[:80]


def is_meta_skill(name: str, description: str) -> bool:
    words = set(split_words(name)) | set(tokenize(description))
    return bool(words & META_WORDS)


def build_entry(skill_md: Path, source: SkillSource) -> SkillEntry:
    frontmatter, body = load_skill_markdown(skill_md)
    manifest = load_skill_manifest(skill_md.parent)
    name = str(frontmatter.get("name") or skill_md.parent.name)
    description = normalize_phrase(str(frontmatter.get("description") or ""))
    sections = extract_sections(body)
    trigger_phrases = extract_trigger_phrases(description, sections, body)
    keywords = extract_keywords(name, description, trigger_phrases)
    metadata = normalize_metadata(frontmatter)

    return SkillEntry(
        name=name,
        description=description,
        scope=source.scope,
        source_root=str(source.root),
        source_label=source.source_label,
        source_priority=source.source_priority,
        skill_dir=str(skill_md.parent),
        skill_md=str(skill_md),
        keywords=keywords,
        trigger_phrases=trigger_phrases,
        sections=sections,
        metadata=metadata,
        meta_skill=is_meta_skill(name, description),
        manifest_type=normalize_phrase(str(manifest.get("type") or "")) or None,
        agent_display_name=normalize_phrase(str(manifest.get("displayName") or manifest.get("name") or "")) or None,
    )


def discover_skills() -> list[SkillEntry]:
    entries: list[SkillEntry] = []
    for source in SKILL_SOURCES:
        if not source.root.exists():
            continue
        for skill_md in sorted(source.root.rglob("SKILL.md")):
            if skill_md.parts[-3:-1] == ("skill-selector", "scripts"):
                continue
            entries.append(build_entry(skill_md, source))
    entries.sort(key=lambda item: (item.source_priority, item.name.casefold(), item.skill_md))
    return entries


def collision_sort_key(entry: SkillEntry) -> tuple[int, str, str]:
    return (entry.source_priority, entry.skill_md.casefold(), entry.source_label)


def render_shadowed_entry(entry: SkillEntry) -> dict[str, object]:
    return {
        "name": entry.name,
        "scope": entry.scope,
        "source_label": entry.source_label,
        "source_priority": entry.source_priority,
        "skill_md": entry.skill_md,
    }


def resolve_skill_collisions(entries: list[SkillEntry]) -> tuple[list[SkillEntry], list[dict[str, object]]]:
    winners: list[SkillEntry] = []
    shadowed: list[dict[str, object]] = []
    grouped: dict[str, list[SkillEntry]] = {}

    for entry in entries:
        grouped.setdefault(entry.name.casefold(), []).append(entry)

    for _name_key, group in sorted(grouped.items(), key=lambda item: item[0]):
        ordered_group = sorted(group, key=collision_sort_key)
        winner = ordered_group[0]
        winners.append(winner)
        if len(ordered_group) == 1:
            continue
        shadowed.append(
            {
                "name": winner.name,
                "winner": render_shadowed_entry(winner),
                "shadowed": [render_shadowed_entry(entry) for entry in ordered_group[1:]],
                "reason": "highest-priority source wins; ties break by normalized skill path",
            }
        )

    winners.sort(key=lambda item: (item.source_priority, item.name.casefold(), item.skill_md))
    return winners, shadowed


def scan_skills() -> list[SkillEntry]:
    winners, _shadowed = resolve_skill_collisions(discover_skills())
    return winners


def format_keywords(entry: SkillEntry) -> str:
    return ", ".join(entry.keywords[:8])


def escape_markdown_table_text(value: str) -> str:
    escaped = re.sub(r"(?<!<)(https?://[^\s|)]+)", r"<\1>", value)
    escaped = escaped.replace("|", r"\|")
    escaped = escaped.replace("*", r"\*")
    return escaped


def append_table_row(lines: list[str], columns: list[str]) -> None:
    lines.append("| " + " | ".join(columns) + " |")


def render_meta_skill_rows(lines: list[str], entries: list[SkillEntry]) -> None:
    meta_names = {"skill-selector", "skill-builder", "doublecheck"}
    for entry in entries:
        if entry.name not in meta_names:
            continue
        append_table_row(
            lines,
            [
                f"`{entry.name}`",
                f"`{entry.scope}`",
                escape_markdown_table_text(entry.description),
                f"`{format_keywords(entry)}`",
            ],
        )


def render_scope_section(lines: list[str], title: str, entries: list[SkillEntry]) -> None:
    lines.extend(["", f"## {title}", "", "| Skill | Description | Trigger keywords |", "| --- | --- | --- |"])
    for entry in entries:
        append_table_row(
            lines,
            [f"`{entry.name}`", escape_markdown_table_text(entry.description), f"`{format_keywords(entry)}`"],
        )


def render_catalog(entries: list[SkillEntry]) -> str:
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    workspace_entries = [entry for entry in entries if entry.scope == "workspace"]
    user_entries = [entry for entry in entries if entry.scope == "user"]
    external_entries = [entry for entry in entries if entry.scope == "external"]

    lines = [
        "<!-- generated by .github/skills/skill-selector/scripts/build_skill_index.py -->",
        "# Skills Content",
        "[TOC]",
        "",
        "**name: skills-content**",
        "**description:** Machine-generated catalog of workspace and external skills, with descriptions and trigger keywords for routing.",
        f"**updated:** {now}",
        "",
        "## Overview",
        "",
        "📌 This catalog is generated from the workspace-local skill directory and the configured external skill directory.",
        "",
        f"📌 Indexed skills: `{len(entries)}` total, `{len(workspace_entries)}` workspace-local, `{len(user_entries)}` user-level, `{len(external_entries)}` external.",
        "",
        "📌 The machine-readable routing source of truth is `.github/skills/skill-selector/skills-index.json`.",
        "",
        "📌 The recommended entry point for delegation is `skill-selector`, which should append `doublecheck` as the final verification step.",
        "",
        "## Routing Meta Skills",
        "",
        "| Skill | Scope | Description | Trigger keywords |",
        "| --- | --- | --- | --- |",
    ]

    render_meta_skill_rows(lines, entries)
    render_scope_section(lines, "Workspace Skills", workspace_entries)
    render_scope_section(lines, "User-Level Agent Skills", user_entries)
    render_scope_section(lines, "External Skills", external_entries)

    lines.append("")
    return "\n".join(lines)


def write_outputs(entries: list[SkillEntry], shadowed_skills: list[dict[str, object]]) -> None:
    index = {
        "version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "sources": [
            str(source.root) for source in SKILL_SOURCES
        ],
        "shadowed_skills": shadowed_skills,
        "skills": [entry.to_dict() for entry in entries],
    }
    INDEX_PATH.write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    CATALOG_PATH.write_text(render_catalog(entries), encoding="utf-8")


def rank_query(entries: list[SkillEntry], query: str) -> list[tuple[int, SkillEntry, list[str]]]:
    ranked: list[tuple[int, SkillEntry, list[str]]] = []
    for entry in entries:
        score, reasons = score_entry(entry, query)
        if score > 0:
            ranked.append((score, entry, reasons))
    ranked.sort(key=lambda item: (-item[0], item[1].source_priority, item[1].name.casefold()))
    return rerank_results(ranked, query)


def print_ranked_matches(ranked: list[tuple[int, SkillEntry, list[str]]], top: int) -> None:
    print("Top matches:")
    for score, entry, reasons in ranked[:top]:
        print(f"- {entry.name} [{entry.scope}] score={score}")
        print(f"  description: {entry.description}")
        print(f"  reasons: {', '.join(reasons[:4])}")


def alternate_bonus(entry: SkillEntry, query: str) -> int:
    query_words = set(split_words(query))
    query_actions = detect_primary_actions(query_words)
    entry_tokens = set(entry.keywords) | set(split_words(entry.name))

    bonus = 0
    if "skill" in query_words and "create" in query_actions:
        if "skill" in entry_tokens:
            bonus += 18
        if entry.name in {"crypto-skill-builder", "agent-customization"}:
            bonus += 12
        if entry.name == "crypto-skill-builder":
            bonus += 10
        if "readme" in entry_tokens:
            bonus -= 18
    return bonus


def select_alternates(ranked: list[tuple[int, SkillEntry, list[str]]], query: str) -> list[str]:
    alternates: list[str] = []
    seen_alternates: set[str] = set()
    ordered = sorted(
        ranked[1:],
        key=lambda item: (-(item[0] + alternate_bonus(item[1], query)), item[1].source_priority, item[1].name.casefold()),
    )
    for _score, entry, _reasons in ordered:
        if entry.name in seen_alternates:
            continue
        alternates.append(entry.name)
        seen_alternates.add(entry.name)
        if len(alternates) == 2:
            break
    return alternates


def print_chain(chain: list[str]) -> None:
    print("\nSuggested chain:")
    for index, skill_name in enumerate(chain, start=1):
        print(f"{index}. {skill_name}")


def resolve_dispatch_target(skill_name: str, ranked: list[tuple[int, SkillEntry, list[str]]]) -> DispatchTarget:
    override = AGENT_TARGET_OVERRIDES.get(skill_name)
    if override is not None:
        return override

    for _score, entry, _reasons in ranked:
        if entry.name != skill_name:
            continue
        if entry.manifest_type == "agent":
            target_name = entry.agent_display_name or entry.name
            return DispatchTarget(skill_name=entry.name, target_kind="agent", target_name=target_name)
        return DispatchTarget(skill_name=entry.name, target_kind="skill", target_name=entry.name)

    return DispatchTarget(skill_name=skill_name, target_kind="skill", target_name=skill_name)


def dispatch_query(
    entries: list[SkillEntry],
    query: str,
    dispatcher: SkillDispatcher,
) -> ChainExecution:
    ranked = rank_query(entries, query)
    if not ranked:
        raise ValueError(f"No matching skills found for query: {query}")
    chain = build_chain(ranked, query)
    targets = [resolve_dispatch_target(skill_name, ranked) for skill_name in chain]
    return execute_chain(targets, query, dispatcher)


def print_execution(execution: ChainExecution, is_mock: bool) -> None:
    label = "Executed chain (mock dispatcher):" if is_mock else "Executed chain:"
    print(f"\n{label}")
    for record in execution.records:
        print(
            f"{record.position}. {record.target.skill_name}"
            f" [{record.target.target_kind}:{record.target.target_name}]"
        )
        print(f"   output: {record.output_text}")


def run_query(entries: list[SkillEntry], query: str, top: int, execute: bool) -> None:
    ranked = rank_query(entries, query)

    print(f"Query: {query}")
    if not ranked:
        print("No matching skills found.")
        return

    print_ranked_matches(ranked, top)
    alternates = select_alternates(ranked, query)
    chain = build_chain(ranked, query)
    print_chain(chain)
    if alternates:
        print(f"Alternates: {', '.join(alternates)}")
    if execute:
        execution = dispatch_query(entries, query, RecordingDispatcher())
        print_execution(execution, is_mock=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--rebuild-index",
        action="store_true",
        help="Rebuild the full machine index and human catalog before any optional query execution",
    )
    parser.add_argument("--query", help="Query the built index instead of only generating it")
    parser.add_argument("--top", type=int, default=5, help="Number of ranked matches to print")
    parser.add_argument(
        "--execute",
        action="store_true",
        help="Execute the selected chain through the built-in recording dispatcher",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    discovered_entries = discover_skills()
    entries, shadowed_skills = resolve_skill_collisions(discovered_entries)
    write_outputs(entries, shadowed_skills)
    print(f"Indexed {len(entries)} skills -> {INDEX_PATH}")
    print(f"Updated catalog -> {CATALOG_PATH}")
    if shadowed_skills:
        print(f"Resolved {len(shadowed_skills)} skill-name collisions")

    if args.query:
        run_query(entries, args.query, args.top, args.execute)


if __name__ == "__main__":
    main()