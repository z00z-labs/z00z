"""Integration checks for the Phase 066 .codex surface and prompt wiring."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
AGENTS_DIR = ROOT / ".github" / "agents"
PROMPTS_DIR = ROOT / ".github" / "prompts"
SKILLS_DIR = ROOT / ".github" / "skills"
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures"

SYMLINK_TARGETS = {
    "skills": "../.github/skills",
    "agents": "../.github/agents",
    "prompts": "../.github/prompts",
}

SYMLINK_ONLY_NAMES = [
    "hooks",
    "instructions",
    "requirements",
    "scripts",
    "plugins",
]

PENTEST_AGENTS = [
    "pentest-rust-reviewer.agent.md",
    "pentest-crypto-reviewer.agent.md",
    "pentest-storage-reviewer.agent.md",
    "pentest-rpc-dast-reviewer.agent.md",
    "pentest-supply-chain-reviewer.agent.md",
]

ACTIVE_PENTEST_SKILLS = [
    "pentest-local-dast",
    "pentest-local-orchestrator",
    "pentest-report",
    "pentest-rust-security",
    "pentest-secrets-supply-chain",
    "pentest-source-aware-sast",
    "pentest-tools-installer",
    "pentest-tools-runner",
]

PROMPT_FILES = {
    "pentest-local.prompt.md": (
        "Generic Repo Mode",
        "Z00Z Profile Mode",
        "./scripts/run_pentest_tools.sh",
        "run_local_pentest.sh",
    ),
    "pentest-parallel-review.prompt.md": (
        "Wait-For-All Rule",
        "wait for all requested lanes to finish before merging findings.",
        "Deduplicate findings by `path`, `line`, `rule_id`, and `evidence_anchor`.",
        "No lane may claim direct tool execution by default.",
    ),
    "pentest-report-doublecheck.prompt.md": (
        "local artifact tree",
        "Keep scanner findings validated before confirmation.",
        "Never upgrade a scanner-only hypothesis to confirmed without repository-backed",
        "source files",
    ),
    "pentest-local-z00z.prompt.md": (
        "z00z-pentest-profile",
        "attack-surfaces-create",
        "z00z-crypto-auditor",
        "gsd-audit-4.prompt.md",
        "./scripts/run_pentest_tools.sh",
    ),
}


class CodexSurfaceIntegrationTest(unittest.TestCase):
    """Validate .codex symlink compatibility and Phase 066 prompt wiring."""

    maxDiff = None

    def assert_codex_surface(self, root: Path) -> None:
        """Check that the .codex compatibility layer points at live canonical targets."""

        for name, target in SYMLINK_TARGETS.items():
            path = root / ".codex" / name
            self.assertTrue(path.is_symlink(), path)
            self.assertEqual(path.readlink().as_posix(), target)
            self.assertTrue(path.exists(), path)

        for name in SYMLINK_ONLY_NAMES:
            path = root / ".codex" / name
            self.assertTrue(path.is_symlink(), path)
            self.assertTrue(path.exists(), path)

    def test_codex_symlink_targets_remain_canonical(self) -> None:
        self.assert_codex_surface(ROOT)

    def test_broken_codex_symlink_is_detected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            codex_dir = temp_root / ".codex"
            github_dir = temp_root / ".github"
            codex_dir.mkdir(parents=True, exist_ok=True)
            for name in ("skills", "agents", "hooks", "instructions", "requirements", "scripts", "plugins"):
                (github_dir / name).mkdir(parents=True, exist_ok=True)

            for name, target in SYMLINK_TARGETS.items():
                (codex_dir / name).symlink_to(target)
            for name in SYMLINK_ONLY_NAMES:
                (codex_dir / name).symlink_to(f"../.github/{name}")

            with self.assertRaises(AssertionError):
                self.assert_codex_surface(temp_root)

    def test_pentest_agent_files_exist_and_stay_bounded(self) -> None:
        for agent_name in PENTEST_AGENTS:
            path = AGENTS_DIR / agent_name
            text = path.read_text(encoding="utf-8")
            self.assertIn("Do not execute tools unless the orchestrator prompt explicitly asks for it.", text)
            self.assertIn("## Output Contract", text)
            self.assertNotIn("I will run tools directly", text)

    def test_active_pentest_skill_surface_stays_small_and_generic(self) -> None:
        observed = sorted(path.name for path in SKILLS_DIR.glob("pentest-*") if path.is_dir())
        self.assertEqual(observed, ACTIVE_PENTEST_SKILLS)
        for disallowed_name in (
            "strix",
            "hexstrike",
            "hexstrike-ai",
            "frameworks",
            "protocols",
            "technologies",
            "vulnerabilities",
            "scan_modes",
        ):
            self.assertFalse((SKILLS_DIR / disallowed_name).exists(), disallowed_name)

    def test_prompt_files_cover_generic_and_z00z_examples(self) -> None:
        for prompt_name, required_fragments in PROMPT_FILES.items():
            path = PROMPTS_DIR / prompt_name
            text = path.read_text(encoding="utf-8")
            for fragment in required_fragments:
                self.assertIn(fragment, text, f"{prompt_name} missing {fragment!r}")

    def test_prompt_text_avoids_default_upstream_runtime_paths(self) -> None:
        disallowed = (
            "HexStrike server",
            "Strix runtime",
            "LLM_API_KEY",
            "run tools directly by default",
        )
        for prompt_name in (
            "pentest-local.prompt.md",
            "pentest-parallel-review.prompt.md",
            "pentest-report-doublecheck.prompt.md",
        ):
            text = (PROMPTS_DIR / prompt_name).read_text(encoding="utf-8")
            for fragment in disallowed:
                self.assertNotIn(fragment, text, f"{prompt_name} leaked {fragment!r}")

    def test_z00z_lane_map_matches_expected_fixture(self) -> None:
        payload = json.loads((FIXTURES_DIR / "z00z_lane_map_expected.json").read_text(encoding="utf-8"))
        prompt_text = (PROMPTS_DIR / "pentest-local-z00z.prompt.md").read_text(encoding="utf-8")
        for line in payload["lines"]:
            self.assertIn(line, prompt_text)

    def test_parallel_merge_fixture_collapses_duplicates_by_prompt_contract(self) -> None:
        findings_a = json.loads((FIXTURES_DIR / "parallel_merge_findings_a.json").read_text(encoding="utf-8"))["findings"]
        findings_b = json.loads((FIXTURES_DIR / "parallel_merge_findings_b.json").read_text(encoding="utf-8"))["findings"]

        merged: dict[tuple[str, int, str, str], dict[str, object]] = {}
        for finding in findings_a + findings_b:
            key = (
                str(finding["path"]),
                int(finding["line"]),
                str(finding["rule_id"]),
                str(finding["evidence_anchor"]),
            )
            current = merged.setdefault(
                key,
                {
                    "path": finding["path"],
                    "line": finding["line"],
                    "rule_id": finding["rule_id"],
                    "evidence_anchor": finding["evidence_anchor"],
                    "lanes": [],
                },
            )
            current["lanes"].append(finding["lane"])

        self.assertEqual(len(merged), 3)
        rpc_key = ("crates/z00z_wallets/src/rpc/api.rs", 48, "rpc-auth-guard", "rpc-api-auth-48")
        self.assertCountEqual(
            merged[rpc_key]["lanes"],
            ["pentest-rust-reviewer", "pentest-rpc-dast-reviewer"],
        )


if __name__ == "__main__":
    unittest.main()
