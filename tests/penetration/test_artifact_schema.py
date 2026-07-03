"""Regression tests for the Phase 066 artifact validator."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUILD_SCRIPT = ROOT / "scripts" / "penetration" / "build_pentest_report.py"
VALIDATE_SCRIPT = ROOT / "scripts" / "penetration" / "validate_artifacts.py"
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures"
SCANNER_FIXTURES_DIR = FIXTURES_DIR / "scanner_outputs"
REPORT_FIXTURES_DIR = FIXTURES_DIR / "reports"


def write_json(path: Path, payload: object) -> None:
    """Write deterministic JSON fixtures."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def copy_fixture(source: Path, destination: Path) -> None:
    """Copy a small text fixture into the temporary artifact tree."""

    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(source.read_text(encoding="utf-8"), encoding="utf-8")


def load_contract() -> dict[str, object]:
    """Load the artifact contract fixture."""

    return json.loads((REPORT_FIXTURES_DIR / "artifact_contract.json").read_text(encoding="utf-8"))


def create_artifact_tree(base_dir: Path, run_id: str, *, findings_fixture: str | None = None) -> tuple[Path, Path]:
    """Create a minimal artifact tree that satisfies the validator contract."""

    artifact_dir = base_dir / ".security-artifacts" / run_id
    report_dir = base_dir / "reports" / f"z00z-pentests_report-{run_id}"
    for directory_name in ("sast", "rust", "secrets", "dast", "report", "logs", "raw", "normalized"):
        (artifact_dir / directory_name).mkdir(parents=True, exist_ok=True)
    report_dir.mkdir(parents=True, exist_ok=True)

    write_json(
        artifact_dir / "manifest.json",
        {
            "version": 1,
            "run_id": run_id,
            "artifact_dir": artifact_dir.as_posix(),
            "report_dir": report_dir.as_posix(),
            "mode": "quick",
            "profile": "generic",
            "scope_path": (ROOT / ".security" / "scope.yaml").as_posix(),
            "status": "running",
            "flags": {"check_only": False, "no_dast": False, "static_only": True},
            "commands": [],
        },
    )
    write_json(
        artifact_dir / "scope.normalized.json",
        {
            "status": "OK",
            "mode": "local-only",
            "scope_path": (ROOT / ".security" / "scope.yaml").as_posix(),
            "normalized_paths": ["crates", "scripts"],
            "normalized_hosts": ["127.0.0.1", "localhost"],
            "normalized_urls": [],
            "dast_targets_present": False,
            "errors": [],
            "warnings": [],
            "requested_tool": None,
            "skip_reason": None,
        },
    )
    write_json(
        artifact_dir / "tool-status.json",
        {
            "version": 1,
            "summary": {"present": 0, "missing": 1, "missing_required": 1},
            "tools": [],
        },
    )
    write_json(
        artifact_dir / "normalized" / "sast.semgrep.status.json",
        {
            "version": 1,
            "lane": "sast",
            "tool": "semgrep",
            "status": "missing",
            "exit_code": 127,
            "command_display": "",
            "stdout_path": "",
            "stderr_path": "",
            "exit_path": "",
            "raw_output_path": "",
            "summary": "semgrep is not available in tools/penetration or PATH",
        },
    )
    write_json(
        artifact_dir / "dast" / "skipped.json",
        {
            "version": 1,
            "artifact": "dast-skip",
            "status": "skipped",
            "summary": "static-only mode requested",
        },
    )
    write_json(
        artifact_dir / "dast" / "summary.json",
        {
            "version": 1,
            "lane": "dast",
            "tool": "nuclei",
            "status": "skipped",
            "summary": "static-only mode requested",
            "status_files": ["dast/skipped.json"],
        },
    )
    copy_fixture(
        SCANNER_FIXTURES_DIR / "raw_semgrep_secret.json",
        artifact_dir / "raw" / "semgrep" / "auth-bypass.json",
    )
    copy_fixture(
        SCANNER_FIXTURES_DIR / "raw_gitleaks_secret.json",
        artifact_dir / "raw" / "gitleaks" / "report.json",
    )
    write_json(
        artifact_dir / "raw" / "trivy" / "trivy.json",
        {
            "artifact": "raw-trivy",
            "summary": "fixture-only archive finding",
        },
    )
    write_json(
        artifact_dir / "raw" / "nuclei" / "skipped.json",
        {
            "artifact": "raw-dast-skip",
            "summary": "no bounded local DAST target was present",
        },
    )

    if findings_fixture is not None:
        copy_fixture(
            SCANNER_FIXTURES_DIR / findings_fixture,
            artifact_dir / "normalized" / "findings.json",
        )

    return artifact_dir, report_dir


class ArtifactSchemaTest(unittest.TestCase):
    """Exercise the public artifact-validator CLI."""

    maxDiff = None

    def run_builder(self, artifact_dir: Path, report_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(BUILD_SCRIPT),
                "--artifact-dir",
                str(artifact_dir),
                "--report-dir",
                str(report_dir),
                "--profile",
                "generic",
            ],
            check=False,
            capture_output=True,
            text=True,
        )

    def run_validator(self, artifact_dir: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(VALIDATE_SCRIPT), str(artifact_dir), "--json"],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_validator_accepts_built_artifact_tree(self) -> None:
        contract = load_contract()
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(
                Path(temp_root),
                "20260703T000011Z",
                findings_fixture="findings_mixed.json",
            )
            build_process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(build_process.returncode, 0, build_process.stderr)

            validate_process = self.run_validator(artifact_dir)
            payload = json.loads(validate_process.stdout)
            self.assertEqual(validate_process.returncode, 0, validate_process.stderr)
            self.assertEqual(payload["status"], "OK")
            self.assertEqual(payload["report_dir"], report_dir.as_posix())
            for relative_dir in contract["required_directories"]:
                self.assertIn((artifact_dir / relative_dir).as_posix(), payload["checked_paths"])
            for relative_file in contract["required_files"]:
                self.assertIn((artifact_dir / relative_file).as_posix(), payload["checked_paths"])

    def test_validator_rejects_missing_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(Path(temp_root), "20260703T000012Z")
            build_process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(build_process.returncode, 0, build_process.stderr)

            (artifact_dir / "manifest.json").unlink()
            validate_process = self.run_validator(artifact_dir)
            payload = json.loads(validate_process.stdout)
            self.assertEqual(validate_process.returncode, 1)
            self.assertTrue(
                any("required file is missing" in error and "manifest.json" in error for error in payload["errors"]),
                payload["errors"],
            )

    def test_validator_rejects_missing_report_dir_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(Path(temp_root), "20260703T000013Z")
            build_process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(build_process.returncode, 0, build_process.stderr)

            manifest_path = artifact_dir / "manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["report_dir"] = ""
            write_json(manifest_path, manifest)

            validate_process = self.run_validator(artifact_dir)
            payload = json.loads(validate_process.stdout)
            self.assertEqual(validate_process.returncode, 1)
            self.assertIn("manifest.report_dir must be a non-empty string", payload["errors"])

    def test_validator_rejects_missing_host_report_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(Path(temp_root), "20260703T000014Z")
            build_process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(build_process.returncode, 0, build_process.stderr)

            shutil.rmtree(report_dir)
            validate_process = self.run_validator(artifact_dir)
            payload = json.loads(validate_process.stdout)
            self.assertEqual(validate_process.returncode, 1)
            self.assertTrue(
                any("host report directory is missing" in error for error in payload["errors"]),
                payload["errors"],
            )

    def test_validator_rejects_confirmed_report_without_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(Path(temp_root), "20260703T000015Z")
            build_process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(build_process.returncode, 0, build_process.stderr)

            write_json(
                artifact_dir / "report" / "report-metadata.json",
                {
                    "version": 1,
                    "run_id": "20260703T000015Z",
                    "generated_at": "2026-07-03T00:00:15Z",
                    "artifact_dir": artifact_dir.as_posix(),
                    "report_dir": report_dir.as_posix(),
                    "profile": "generic",
                    "summary": {"overall_status": "completed", "status_counts": {}},
                    "scope_status": "OK",
                    "tool_rows": [],
                    "tools_inventory_summary": {},
                    "findings": [
                        {
                            "id": "F-002",
                            "title": "Broken evidence",
                            "severity": "high",
                            "status": "confirmed",
                            "lane": "sast",
                            "tool": "semgrep",
                            "summary": "This record is missing proof fields.",
                        }
                    ],
                },
            )

            validate_process = self.run_validator(artifact_dir)
            payload = json.loads(validate_process.stdout)
            self.assertEqual(validate_process.returncode, 1)
            self.assertEqual(payload["status"], "FAIL")
            self.assertTrue(
                any("confirmed finding is missing evidence fields" in error for error in payload["errors"]),
                payload["errors"],
            )


if __name__ == "__main__":
    unittest.main()
