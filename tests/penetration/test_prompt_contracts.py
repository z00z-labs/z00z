"""Prompt contract tests for the Phase 066 execution prompts."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROMPTS_DIR = ROOT / ".github" / "prompts"
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures"
PROMPT_FIXTURES_DIR = FIXTURES_DIR / "prompts"
PROFILE_FIXTURES_DIR = FIXTURES_DIR / "profile"


def load_prompt_contract() -> dict[str, object]:
    """Load the prompt contract fixture."""

    return json.loads((PROMPT_FIXTURES_DIR / "prompt_contract_expected.json").read_text(encoding="utf-8"))


def load_profile_contract() -> dict[str, object]:
    """Load the Z00Z dry-run lane map fixture."""

    return json.loads((PROFILE_FIXTURES_DIR / "z00z_dry_run_expected.json").read_text(encoding="utf-8"))


class PromptContractsTest(unittest.TestCase):
    """Validate prompt merge rules, entrypoints, and fail-closed wording."""

    maxDiff = None

    def test_required_fragments_exist_in_each_prompt(self) -> None:
        contract = load_prompt_contract()
        for prompt_name, fragments in contract["required_fragments"].items():
            text = (PROMPTS_DIR / prompt_name).read_text(encoding="utf-8")
            for fragment in fragments:
                self.assertIn(fragment, text, f"{prompt_name} missing {fragment!r}")

    def test_prompt_files_avoid_default_runtime_drift(self) -> None:
        contract = load_prompt_contract()
        for prompt_name in contract["required_fragments"]:
            text = (PROMPTS_DIR / prompt_name).read_text(encoding="utf-8")
            for fragment in contract["disallowed_fragments"]:
                self.assertNotIn(fragment, text, f"{prompt_name} leaked {fragment!r}")

    def test_generic_prompt_keeps_one_external_entrypoint(self) -> None:
        text = (PROMPTS_DIR / "pentest-local.prompt.md").read_text(encoding="utf-8")
        self.assertIn("./z00z_penetration_tests.sh", text)
        self.assertIn("run_local_pentest.sh", text)
        self.assertIn("only external entrypoint", text)

    def test_report_doublecheck_prompt_rejects_scanner_only_confirmation(self) -> None:
        text = (PROMPTS_DIR / "pentest-report-doublecheck.prompt.md").read_text(encoding="utf-8")
        self.assertIn("scanner-only hypothesis", text)
        self.assertIn("unconfirmed or false-positive", text)
        self.assertIn("report-metadata.json", text)

    def test_z00z_profile_prompt_lane_map_matches_profile_fixture(self) -> None:
        payload = load_profile_contract()
        text = (PROMPTS_DIR / "pentest-local-z00z.prompt.md").read_text(encoding="utf-8")
        for line in payload["lines"]:
            self.assertIn(line, text)


if __name__ == "__main__":
    unittest.main()
