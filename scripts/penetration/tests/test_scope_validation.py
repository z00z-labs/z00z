"""Regression tests for the Phase 066 local-only scope validator."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "penetration" / "validate_scope.py"
DENYLIST = ROOT / "scripts" / "penetration" / "denied-tools.txt"
SCOPE = ROOT / "scripts" / "penetration" / "scope.yaml"
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures" / "scope"
SAFETY_POLICY = (
    ROOT
    / ".github"
    / "skills"
    / "pentest-local-orchestrator"
    / "references"
    / "safety-policy.md"
)
UPSTREAM_SOURCES = (
    ROOT
    / ".github"
    / "skills"
    / "pentest-local-orchestrator"
    / "references"
    / "UPSTREAM-SOURCES.md"
)


class ScopeValidationTest(unittest.TestCase):
    """Exercise the validator through its public CLI."""

    maxDiff = None

    def run_validator(self, scope_path: Path | None = None, *extra_args: str) -> subprocess.CompletedProcess[str]:
        """Run the validator against the default scope or a fixture path."""

        selected_scope = scope_path or SCOPE
        return subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                str(selected_scope),
                "--denylist",
                str(DENYLIST),
                "--json",
                *extra_args,
            ],
            check=False,
            capture_output=True,
            text=True,
        )

    def parse_result(self, process: subprocess.CompletedProcess[str]) -> dict[str, object]:
        """Decode validator JSON output."""

        self.assertTrue(process.stdout, process.stderr)
        return json.loads(process.stdout)

    def test_default_scope_passes(self) -> None:
        process = self.run_validator()
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(payload["status"], "OK")
        self.assertEqual(payload["normalized_hosts"], ["127.0.0.1", "localhost"])
        self.assertEqual(
            payload["normalized_urls"],
            ["http://127.0.0.1:18080", "http://localhost:3000/health"],
        )

    def test_localhost_urls_are_accepted(self) -> None:
        process = self.run_validator(FIXTURES_DIR / "local_url_scope.yaml")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(payload["status"], "OK")
        self.assertEqual(
            payload["normalized_urls"],
            ["http://127.0.0.1:18080", "http://localhost:3000/health"],
        )

    def test_public_url_is_rejected(self) -> None:
        process = self.run_validator(FIXTURES_DIR / "public_url_scope.yaml")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 1)
        self.assertEqual(payload["status"], "FAIL")
        self.assertTrue(
            any("public DNS names" in error for error in payload["errors"]),
            payload["errors"],
        )

    def test_public_ip_is_rejected(self) -> None:
        process = self.run_validator(FIXTURES_DIR / "public_ip_scope.yaml")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 1)
        self.assertTrue(
            any("non-loopback IP" in error for error in payload["errors"]),
            payload["errors"],
        )

    def test_wildcard_host_is_rejected(self) -> None:
        process = self.run_validator(FIXTURES_DIR / "wildcard_host_scope.yaml")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 1)
        self.assertTrue(
            any("wildcard hosts" in error for error in payload["errors"]),
            payload["errors"],
        )

    def test_broad_cidr_is_rejected(self) -> None:
        process = self.run_validator(FIXTURES_DIR / "broad_cidr_scope.yaml")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 1)
        self.assertTrue(
            any("broader than loopback" in error for error in payload["errors"]),
            payload["errors"],
        )

    def test_denied_tool_is_rejected(self) -> None:
        process = self.run_validator(None, "--tool", "hydra")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 1)
        self.assertTrue(
            any("requested tool is denied" in error for error in payload["errors"]),
            payload["errors"],
        )

    def test_source_scope_no_targets(self) -> None:
        process = self.run_validator(FIXTURES_DIR / "source_only_scope.yaml")
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 0)
        self.assertEqual(payload["status"], "OK")
        self.assertFalse(payload["dast_targets_present"])

    def test_dast_no_target_skip(self) -> None:
        process = self.run_validator(
            FIXTURES_DIR / "source_only_scope.yaml",
            "--require-dast-targets",
        )
        payload = self.parse_result(process)
        self.assertEqual(process.returncode, 3)
        self.assertEqual(payload["status"], "SKIP")
        self.assertEqual(
            payload["skip_reason"],
            "no allowed local DAST targets are present in scope",
        )

    def test_forbidden_tool_names_stay_in_reference_surfaces_only(self) -> None:
        forbidden_tokens = (
            "hydra",
            "john",
            "hashcat",
            "medusa",
            "metasploit",
            "msfvenom",
            "commix",
        )
        reference_text = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (DENYLIST, SAFETY_POLICY, UPSTREAM_SOURCES)
        )
        active_text = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (
                ROOT / "scripts/run_pentest_tools.sh",
                ROOT / "scripts" / "penetration" / "run_local_pentest.sh",
                ROOT / "scripts" / "penetration" / "run_local_dast.sh",
                ROOT / "scripts" / "penetration" / "check_pentest_tools.sh",
                ROOT / "tools" / "penetration" / "docker" / "run_pentest_container.sh",
            )
        )

        for token in forbidden_tokens:
            self.assertIn(token, reference_text)
            self.assertNotIn(token, active_text, token)


if __name__ == "__main__":
    unittest.main()
