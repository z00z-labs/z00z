#!/usr/bin/env python3
"""End-to-end verification for skill-selector CLI artifacts and dispatch output."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
WORKSPACE_ROOT = SCRIPT_DIR.parents[3]
BUILD_SCRIPT = SCRIPT_DIR / "build_skill_index.py"
INDEX_PATH = WORKSPACE_ROOT / ".github/skills/skill-selector/skills-index.json"
CATALOG_PATH = WORKSPACE_ROOT / ".github/skills/SKILLS_CONTENT.MD"

DOCS_QUERY = "document this repo and write a README"
SELF_REVIEW_QUERY = "prover sobstvennuju rabotu na kachestvo sdelaj sebe polnocennoe review"


def run_cli(query: str) -> str:
    result = subprocess.run(
        [sys.executable, str(BUILD_SCRIPT), "--query", query, "--execute", "--top", "3"],
        cwd=WORKSPACE_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def rebuild_index() -> str:
    result = subprocess.run(
        [sys.executable, str(BUILD_SCRIPT), "--rebuild-index"],
        cwd=WORKSPACE_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def load_index() -> dict[str, object]:
    return json.loads(INDEX_PATH.read_text(encoding="utf-8"))


def assert_contains(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise AssertionError(f"Missing {label}: {needle!r}")


def assert_file_exists(path: Path, label: str) -> None:
    if not path.exists():
        raise AssertionError(f"Missing {label}: {path}")


def load_index_skills() -> list[dict[str, object]]:
    index = load_index()
    skills = index.get("skills", [])
    if not isinstance(skills, list) or len(skills) < 300:
        raise AssertionError("Unique indexed skill count is below the expected floor")
    return skills


def assert_selector_manifest(skills: list[dict[str, object]]) -> None:
    selector_entry = next((entry for entry in skills if entry.get("name") == "skill-selector"), None)
    if selector_entry is None:
        raise AssertionError("skill-selector entry is missing from skills-index.json")
    if selector_entry.get("manifest_type") != "agent":
        raise AssertionError("skill-selector manifest_type should be 'agent'")
    metadata = selector_entry.get("metadata")
    if not isinstance(metadata, dict):
        raise AssertionError("skill-selector metadata should be preserved in skills-index.json")
    if metadata.get("argument-hint") != 'task="<task description>" | --rebuild-index':
        raise AssertionError("skill-selector metadata argument-hint mismatch")


def assert_index_sources(index: dict[str, object]) -> None:
    sources = index.get("sources")
    if not isinstance(sources, list):
        raise AssertionError("Index sources must be a list")
    required_sources = [
        str(WORKSPACE_ROOT / ".github/skills"),
        str(WORKSPACE_ROOT / ".agents/skills"),
        "/home/vadim/.agents/skills",
        "/home/vadim/.config/Code/agentPlugins",
        "/home/vadim/.vscode/extensions",
    ]
    for required in required_sources:
        if required not in sources:
            raise AssertionError(f"Missing configured source root in index: {required}")


def assert_shadowed_skills_field(index: dict[str, object]) -> None:
    shadowed_skills = index.get("shadowed_skills")
    if not isinstance(shadowed_skills, list):
        raise AssertionError("Index shadowed_skills field must exist and be a list")
    if len(shadowed_skills) < 1:
        raise AssertionError("Expected at least one recorded skill-name collision")


def assert_generated_artifacts() -> None:
    assert_file_exists(INDEX_PATH, "generated index")
    assert_file_exists(CATALOG_PATH, "generated catalog")
    index = load_index()
    assert_index_sources(index)
    assert_shadowed_skills_field(index)
    assert_selector_manifest(load_index_skills())


def assert_docs_query_output(output: str) -> None:
    assert_contains(output, "Suggested chain:\n1. document-project", "docs chain start")
    assert_contains(output, "2. create-readme", "docs chain second step")
    assert_contains(output, "3. doublecheck", "docs chain final step")
    assert_contains(output, "Executed chain (mock dispatcher):", "docs executed header")
    assert_contains(output, "[agent:Doublecheck]", "Doublecheck agent target")
    assert_contains(output, "executed:agent:Doublecheck:input=Original task:", "Doublecheck agent output")
    assert_contains(output, "Request points to verify:", "doublecheck request points label")
    assert_contains(output, "1. document this repo", "first request point")
    assert_contains(output, "2. write a README", "second request point")
    assert_contains(output, "Verification checklist:", "doublecheck checklist label")
    assert_contains(output, "Do not validate only the last baton", "doublecheck baton guard")
    assert_contains(output, "Final artifact to verify:", "doublecheck artifact label")


def assert_self_review_output(output: str) -> None:
    assert_contains(output, "Suggested chain:\n1. review-extreme-skepticism", "self-review chain start")
    assert_contains(output, "2. doublecheck", "self-review chain final step")
    assert_contains(output, "[skill:review-extreme-skepticism]", "self-review worker target")
    assert_contains(output, "[agent:Doublecheck]", "self-review Doublecheck target")
    assert_contains(output, "Request points to verify:", "self-review request points label")
    assert_contains(output, "Verification checklist:", "self-review checklist label")


def main() -> None:
    rebuild_output = rebuild_index()
    docs_output = run_cli(DOCS_QUERY)
    self_review_output = run_cli(SELF_REVIEW_QUERY)

    assert_generated_artifacts()
    assert_contains(rebuild_output, str(INDEX_PATH), "rebuilt index path")
    assert_contains(rebuild_output, str(CATALOG_PATH), "rebuilt catalog path")
    assert_docs_query_output(docs_output)
    assert_self_review_output(self_review_output)

    print("skill-selector e2e checks passed")
    print(f"index_path={INDEX_PATH}")
    print(f"catalog_path={CATALOG_PATH}")


if __name__ == "__main__":
    main()