"""Contract coverage for the Phase 066 Z00Z profile routing surface."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures" / "profile"
SKILL = ROOT / ".github" / "skills" / "z00z-pentest-profile" / "SKILL.md"
PROMPT = ROOT / ".github" / "prompts" / "pentest-local-z00z.prompt.md"
ROUTING = ROOT / ".github" / "skills" / "z00z-pentest-profile" / "references" / "profile-routing.md"
INVARIANTS = ROOT / ".github" / "skills" / "z00z-pentest-profile" / "references" / "z00z-invariants.md"


def load_lane_fixture() -> dict[str, object]:
    """Load the expected documentation-only dry-run lane map."""

    return json.loads((FIXTURES_DIR / "z00z_dry_run_expected.json").read_text(encoding="utf-8"))


class ProfileRoutingTest(unittest.TestCase):
    """Validate the live Z00Z profile routing contract."""

    maxDiff = None

    def setUp(self) -> None:
        self.skill_text = SKILL.read_text(encoding="utf-8")
        self.prompt_text = PROMPT.read_text(encoding="utf-8")
        self.routing_text = ROUTING.read_text(encoding="utf-8")
        self.invariants_text = INVARIANTS.read_text(encoding="utf-8")
        self.combined_text = "\n".join(
            (self.skill_text, self.prompt_text, self.routing_text, self.invariants_text)
        )
        self.combined_lower = self.combined_text.lower()
        self.combined_normalized = " ".join(self.combined_text.split()).lower()

    def test_dry_run_lane_map_matches_fixture_across_live_surfaces(self) -> None:
        payload = load_lane_fixture()
        for line in payload["lines"]:
            self.assertIn(line, self.skill_text)
            self.assertIn(line, self.prompt_text)
            self.assertIn(line, self.routing_text)

    def test_profile_loads_mandatory_context_and_reuses_existing_security_surfaces(self) -> None:
        for fragment in (
            ".github/copilot-instructions.md",
            ".github/requirements/Z00Z_DESIGN_FOUNDATION.md",
            "attack-surfaces-create",
            "z00z-crypto-auditor",
            "gsd-audit-4.prompt.md",
            "pentest-source-aware-sast",
            "pentest-local-dast",
            "pentest-rust-security",
            "pentest-secrets-supply-chain",
        ):
            self.assertIn(fragment, self.combined_text)

    def test_profile_forbids_vendor_mutation_and_default_runtime_drift(self) -> None:
        for fragment in (
            "crates/z00z_crypto/tari/**",
            "No MCP runtime is allowed in the default path.",
            "No external API key is required in the default path.",
            "No public target is allowed in the default path.",
            "Do not execute DAST or modify files.",
            "parallel audit stack",
            "public recon",
            "external API",
        ):
            self.assertIn(fragment, self.combined_text)

        for fragment in ("HexStrike server", "Strix runtime", "LLM_API_KEY"):
            self.assertNotIn(fragment, self.combined_text)

    def test_profile_keeps_proof_evidence_and_simulation_terms_explicit(self) -> None:
        for fragment in (
            "proof, root, signature, commitment, nullifier, or settlement evidence",
            "publication bindings",
            "validator/watcher checks",
            "replication",
            "quorum",
            "conflict resolution",
            "standby catch-up",
            "route rollout",
            "dispatch",
            "membership",
            "restart",
            "partition/heal",
            "stale lineage",
            "divergent roots",
            "failure telemetry",
            "wallet history",
            "storage commits",
            "per-component state",
        ):
            self.assertIn(fragment.lower(), self.combined_normalized)


if __name__ == "__main__":
    unittest.main()
