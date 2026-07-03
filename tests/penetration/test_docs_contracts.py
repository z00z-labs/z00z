"""Documentation contract tests for the Phase 066 migration surfaces."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
README = ROOT / "tools" / "penetration" / "README.md"
MIGRATION_GUIDE = ROOT / ".github" / "skills" / "pentest-local-orchestrator" / "references" / "migration-guide.md"
CHECKLIST = ROOT / ".github" / "skills" / "pentest-local-orchestrator" / "references" / "new-project-checklist.md"
Z00Z_INVARIANTS = ROOT / ".github" / "skills" / "z00z-pentest-profile" / "references" / "z00z-invariants.md"
CYRILLIC_PATTERN = re.compile(r"[\u0400-\u04FF]")


class DocsContractsTest(unittest.TestCase):
    """Validate generic-vs-Z00Z docs split and migration coverage."""

    maxDiff = None

    def test_readme_separates_generic_core_from_z00z_overlay(self) -> None:
        text = README.read_text(encoding="utf-8")
        self.assertIn("## Generic Core", text)
        self.assertIn("## Z00Z-Only Overlay", text)
        self.assertIn(".github/skills/pentest-local-orchestrator/SKILL.md", text)
        self.assertIn(".github/skills/z00z-pentest-profile/SKILL.md", text)

    def test_migration_guide_includes_codex_and_copilot_invocations(self) -> None:
        text = MIGRATION_GUIDE.read_text(encoding="utf-8")
        self.assertIn("Codex minimal generic invocation", text)
        self.assertIn("GitHub Copilot minimal generic invocation", text)
        self.assertIn("project-pentest-profile", text)
        self.assertIn("./z00z_penetration_tests.sh", text)

    def test_docs_cover_required_failure_modes(self) -> None:
        combined = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (README, MIGRATION_GUIDE, CHECKLIST)
        )
        for fragment in (
            "missing tools",
            "no local target",
            "public target rejected",
            "scanner false positive",
            "stale upstream reference",
        ):
            self.assertIn(fragment, combined)

    def test_docs_reference_real_phase_paths(self) -> None:
        required_paths = (
            ROOT / "z00z_penetration_tests.sh",
            ROOT / "scripts" / "penetration" / "run_local_pentest.sh",
            ROOT / ".github" / "prompts" / "pentest-local.prompt.md",
            ROOT / ".github" / "prompts" / "pentest-report-doublecheck.prompt.md",
            ROOT / ".github" / "skills" / "pentest-local-orchestrator" / "SKILL.md",
            ROOT / ".github" / "skills" / "z00z-pentest-profile" / "SKILL.md",
        )
        for path in required_paths:
            self.assertTrue(path.exists(), path)

    def test_z00z_invariants_doc_stays_project_specific(self) -> None:
        text = Z00Z_INVARIANTS.read_text(encoding="utf-8")
        self.assertIn("This file is Z00Z-only.", text)
        self.assertIn("must not copy", text)
        self.assertIn("it unchanged as its own default", text)
        self.assertIn("crates/z00z_crypto/tari/**", text)

    def test_phase_local_artifacts_stay_english_only(self) -> None:
        phase_paths = [
            README,
            ROOT / "tools" / "penetration" / "docker" / "README.md",
            ROOT / ".security" / "report-template.md",
            ROOT / ".security" / "scope.yaml",
            ROOT / ".security" / "allowed-targets.txt",
            ROOT / ".security" / "denied-tools.txt",
            ROOT / "z00z_penetration_tests.sh",
            ROOT / "pack_z00z_project.sh",
            ROOT / "unpack_z00z_project.sh",
            MIGRATION_GUIDE,
            CHECKLIST,
            Z00Z_INVARIANTS,
            ROOT / ".planning" / "phases" / "066-Strix" / "066-SECURITY.md",
            ROOT / ".planning" / "phases" / "066-Strix" / "066-TEST-SPEC.md",
            ROOT / ".planning" / "phases" / "066-Strix" / "066-TESTS-TASKS.md",
        ]

        for path in phase_paths:
            text = path.read_text(encoding="utf-8")
            self.assertIsNone(CYRILLIC_PATTERN.search(text), path.as_posix())


if __name__ == "__main__":
    unittest.main()
