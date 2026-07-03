"""Regression tests for the Phase 066 local tool manifest checker."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "penetration" / "check_pentest_tools.sh"
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures" / "tool_status"
UPSTREAM_LOCK = ROOT / "tools" / "penetration" / "manifests" / "upstream-sources.lock"
UPSTREAM_SOURCES = (
    ROOT
    / ".github"
    / "skills"
    / "pentest-local-orchestrator"
    / "references"
    / "UPSTREAM-SOURCES.md"
)
STRIX_REFERENCE = (
    ROOT
    / ".github"
    / "skills"
    / "pentest-local-orchestrator"
    / "references"
    / "strix"
    / "source_aware_whitebox.md"
)
HEXSTRIKE_REFERENCE = (
    ROOT
    / ".github"
    / "skills"
    / "pentest-tool-installer"
    / "references"
    / "hexstrike"
    / "hexstrike_mcp_reference_only.py"
)


def load_contract() -> dict[str, object]:
    """Load the expected checker contract."""

    return json.loads((FIXTURES_DIR / "expected_contract.json").read_text(encoding="utf-8"))


def make_executable(path: Path, content: str) -> Path:
    """Write a tiny executable fixture."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)
    return path


class ToolManifestTest(unittest.TestCase):
    """Exercise the tool manifest shell contract through its public CLI."""

    maxDiff = None

    def run_checker(
        self,
        tools_root: Path,
        *,
        extra_path: Path | None = None,
        strict: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        """Run the checker with an isolated tools root."""

        env = dict(os.environ)
        env["Z00Z_PENTEST_TOOLS_DIR"] = str(tools_root)
        if extra_path is not None:
            env["PATH"] = f"{extra_path}{os.pathsep}{env['PATH']}"

        args = ["bash", str(SCRIPT), "--json"]
        if strict:
            args.append("--strict")

        return subprocess.run(
            args,
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_machine_readable_output_stays_under_tools_penetration_root(self) -> None:
        contract = load_contract()
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            tools_root = temp_root / "tools" / "penetration"
            local_semgrep = make_executable(
                tools_root / "python" / "bin" / "semgrep",
                "#!/usr/bin/env sh\necho semgrep-fixture-1\n",
            )

            process = self.run_checker(tools_root)
            payload = json.loads(process.stdout)
            tool_rows = {row["name"]: row for row in payload["tools"]}

            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(payload["root"], tools_root.as_posix())
            self.assertEqual(Path(payload["root"]).parent.name, "tools")
            self.assertEqual(Path(payload["root"]).name, "penetration")

            for env_key in contract["required_env_keys"]:
                env_value = payload["env"][env_key]
                self.assertTrue(env_value.startswith(tools_root.as_posix()), (env_key, env_value))
                for fragment in contract["forbidden_path_fragments"]:
                    self.assertNotIn(fragment, env_value)

            for tool_name, expected in contract["required_tools"].items():
                row = tool_rows[tool_name]
                self.assertEqual(row["required"], expected["required"])
                self.assertTrue(row["payload_path"].endswith(expected["payload_suffix"]), row)
                self.assertTrue(row["wrapper_path"].startswith((tools_root / "bin").as_posix()), row)
                self.assertNotIn("tools/formal_verification", row["payload_path"])

            self.assertEqual(tool_rows["semgrep"]["status"], "present")
            self.assertEqual(tool_rows["semgrep"]["source"], "local-payload")
            self.assertEqual(tool_rows["semgrep"]["resolved_path"], local_semgrep.as_posix())
            self.assertGreater(payload["summary"]["missing_required"], 0)

            versions_lock = (tools_root / "manifests" / "tool-versions.lock").read_text(encoding="utf-8")
            self.assertIn("root: tools/penetration", versions_lock)

    def test_broken_local_payload_is_not_reported_as_present(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            tools_root = temp_root / "tools" / "penetration"
            make_executable(
                tools_root / "python" / "bin" / "semgrep",
                "#!/usr/bin/env sh\necho semgrep-fixture-broken >&2\nexit 9\n",
            )

            process = self.run_checker(tools_root)
            payload = json.loads(process.stdout)
            tool_rows = {row["name"]: row for row in payload["tools"]}

            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(tool_rows["semgrep"]["status"], "broken")
            self.assertEqual(tool_rows["semgrep"]["source"], "local-payload")
            self.assertEqual(tool_rows["semgrep"]["version"], "semgrep-fixture-broken")
            self.assertEqual(payload["summary"]["broken"], 1)
            self.assertGreater(payload["summary"]["missing_required"], 0)

    def test_missing_tools_are_recorded_and_strict_mode_fails(self) -> None:
        contract = load_contract()
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            tools_root = temp_root / "tools" / "penetration"
            external_bin = temp_root / "external-bin"
            fake_nuclei = make_executable(
                external_bin / "nuclei",
                "#!/usr/bin/env sh\necho nuclei-fixture-1\n",
            )

            process = self.run_checker(tools_root, extra_path=external_bin, strict=True)
            payload = json.loads(process.stdout)
            tool_rows = {row["name"]: row for row in payload["tools"]}

            self.assertEqual(process.returncode, contract["strict_missing_exit_code"])
            self.assertGreater(payload["summary"]["missing_required"], 0)
            self.assertEqual(tool_rows["nuclei"]["status"], "missing")
            self.assertEqual(tool_rows["nuclei"]["source"], "external-path")
            self.assertEqual(tool_rows["nuclei"]["resolved_path"], fake_nuclei.as_posix())
            self.assertTrue((tools_root / "manifests" / "tool-status.json").is_file())
            self.assertNotEqual(tool_rows["nuclei"]["payload_path"], fake_nuclei.as_posix())

    def test_name_collision_without_version_output_is_not_treated_as_valid_tool(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            tools_root = temp_root / "tools" / "penetration"
            external_bin = temp_root / "external-bin"
            make_executable(
                external_bin / "sg",
                "#!/usr/bin/env sh\nexit 1\n",
            )

            process = self.run_checker(tools_root, extra_path=external_bin)
            payload = json.loads(process.stdout)
            tool_rows = {row["name"]: row for row in payload["tools"]}

            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(tool_rows["sg"]["status"], "missing")
            self.assertEqual(tool_rows["sg"]["source"], "missing")
            self.assertEqual(tool_rows["sg"]["resolved_path"], "")

    def test_upstream_provenance_stays_license_aware_and_reference_only(self) -> None:
        upstream_lock_text = UPSTREAM_LOCK.read_text(encoding="utf-8")
        upstream_sources_text = UPSTREAM_SOURCES.read_text(encoding="utf-8")
        strix_reference_text = STRIX_REFERENCE.read_text(encoding="utf-8")
        hexstrike_reference_text = HEXSTRIKE_REFERENCE.read_text(encoding="utf-8")

        self.assertIn("license_name: Apache License 2.0", upstream_lock_text)
        self.assertIn("license_name: MIT License", upstream_lock_text)
        self.assertIn("reference-only", upstream_sources_text)
        self.assertIn("LLM_API_KEY", upstream_sources_text)

        for fragment in (
            "Source repository:",
            "Source commit:",
            "License: Apache License 2.0",
            "Disposition: reference-only",
        ):
            self.assertIn(fragment, strix_reference_text)

        for fragment in (
            "REFERENCE ONLY - DO NOT RUN",
            "License: MIT License",
            "Disposition: reference-only",
            "Active runtime status: MCP client flow is excluded from the default workflow",
        ):
            self.assertIn(fragment, hexstrike_reference_text)


if __name__ == "__main__":
    unittest.main()
