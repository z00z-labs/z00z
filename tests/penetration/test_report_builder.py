"""Regression tests for the Phase 066 report builder."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "penetration" / "build_pentest_report.py"
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
    """Load the report-level contract fixture."""

    return json.loads((REPORT_FIXTURES_DIR / "artifact_contract.json").read_text(encoding="utf-8"))


def create_artifact_tree(base_dir: Path, run_id: str, *, findings_fixture: str | None = None) -> tuple[Path, Path]:
    """Create a minimal artifact tree that satisfies the 066-06 contract."""

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
    copy_fixture(
        SCANNER_FIXTURES_DIR / "raw_semgrep_secret.json",
        artifact_dir / "raw" / "semgrep" / "auth-bypass.json",
    )
    copy_fixture(
        SCANNER_FIXTURES_DIR / "raw_gitleaks_secret.json",
        artifact_dir / "raw" / "gitleaks" / "report.json",
    )

    if findings_fixture is not None:
        copy_fixture(
            SCANNER_FIXTURES_DIR / findings_fixture,
            artifact_dir / "normalized" / "findings.json",
        )

    return artifact_dir, report_dir


class ReportBuilderTest(unittest.TestCase):
    """Exercise the public report-builder CLI."""

    maxDiff = None

    def run_builder(self, artifact_dir: Path, report_dir: Path) -> subprocess.CompletedProcess[str]:
        """Invoke the builder against a fixture tree."""

        return subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
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

    def test_builder_writes_host_report_and_manifest_paths(self) -> None:
        contract = load_contract()
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(Path(temp_root), "20260703T000001Z")
            process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(process.returncode, 0, process.stderr)

            manifest = json.loads((artifact_dir / "manifest.json").read_text(encoding="utf-8"))
            report_text = (report_dir / "security-report.md").read_text(encoding="utf-8")
            self.assertEqual(manifest["report_dir"], report_dir.as_posix())
            self.assertEqual(
                manifest["report_files"]["host_report_markdown"],
                (report_dir / "security-report.md").as_posix(),
            )
            self.assertEqual(manifest["normalized_findings_path"], "normalized/findings.json")
            self.assertTrue((artifact_dir / "report" / "report-metadata.json").is_file())
            for section in contract["required_sections"]:
                self.assertIn(section, report_text)

    def test_builder_preserves_classification_redaction_and_order(self) -> None:
        contract = load_contract()
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(
                Path(temp_root),
                "20260703T000002Z",
                findings_fixture="findings_mixed.json",
            )
            process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(process.returncode, 0, process.stderr)

            report_text = (report_dir / "security-report.md").read_text(encoding="utf-8")
            report_metadata = json.loads((report_dir / "report-metadata.json").read_text(encoding="utf-8"))

            for section in contract["required_sections"]:
                self.assertIn(section, report_text)
            for forbidden_string in contract["forbidden_strings"]:
                self.assertNotIn(forbidden_string, report_text)

            heading_offsets = [report_text.index(heading) for heading in contract["expected_heading_order"]]
            self.assertEqual(heading_offsets, sorted(heading_offsets))
            self.assertIn("False-positive reason", report_text)
            self.assertIn("Skip reason", report_text)
            self.assertIn("Regression test", report_text)

            self.assertEqual(
                [finding["id"] for finding in report_metadata["findings"]],
                ["CONF-001", "UNC-001", "FP-001", "SKIP-001"],
            )
            self.assertEqual(report_metadata["summary"]["confirmed_finding_total"], 1)
            self.assertEqual(report_metadata["summary"]["false_positive_total"], 1)
            self.assertEqual(report_metadata["summary"]["unconfirmed_finding_total"], 1)
            self.assertEqual(report_metadata["summary"]["skipped_finding_total"], 1)

    def test_builder_rejects_scanner_only_confirmed_finding(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(
                Path(temp_root),
                "20260703T000003Z",
                findings_fixture="findings_scanner_only_confirmed.json",
            )
            process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(process.returncode, 1)
            self.assertIn("confirmed finding CONF-FAIL-001 is missing evidence fields", process.stderr)

    def test_builder_rejects_confirmed_finding_without_regression_test(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(
                Path(temp_root),
                "20260703T000004Z",
                findings_fixture="findings_missing_regression_test.json",
            )
            process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(process.returncode, 1)
            self.assertIn("regression_test", process.stderr)

    def test_builder_deduplicates_duplicate_finding_rows(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root:
            artifact_dir, report_dir = create_artifact_tree(
                Path(temp_root),
                "20260703T000005Z",
                findings_fixture="findings_duplicate_rows.json",
            )
            process = self.run_builder(artifact_dir, report_dir)
            self.assertEqual(process.returncode, 0, process.stderr)

            report_metadata = json.loads((report_dir / "report-metadata.json").read_text(encoding="utf-8"))
            self.assertEqual(len(report_metadata["findings"]), 1)
            finding = report_metadata["findings"][0]
            self.assertEqual(finding["id"], "DUP-002")
            self.assertEqual(finding["severity"], "critical")
            self.assertEqual(finding["status"], "confirmed")
            self.assertEqual(finding["duplicate_ids"], ["DUP-002", "DUP-001"])


if __name__ == "__main__":
    unittest.main()
