"""Integration coverage for the Phase 066 local pentest runners."""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "scripts" / "penetration" / "run_local_pentest.sh"
STATIC_RUNNER = ROOT / "scripts" / "penetration" / "run_parallel_static.sh"
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures" / "scope"


def make_run_id(label: str) -> str:
    """Create a unique run id that remains easy to correlate in assertions."""

    return f"20260703T000000Z-{label}-{uuid.uuid4().hex[:8]}"


def report_run_id(run_id: str) -> str:
    """Convert the timestamp portion of a run id to the host report format."""

    return re.sub(r"(\d{8})T(\d{6})Z", r"\1-\2", run_id, count=1)


def report_dir_name(run_id: str) -> str:
    """Return the canonical host report directory name for a run id."""

    return f"z00z-pentests-report-{report_run_id(run_id)}"


class LocalRunnerIntegrationTest(unittest.TestCase):
    """Exercise the public local runner contracts end to end."""

    maxDiff = None

    def run_local_runner(
        self,
        artifact_dir: Path,
        scope_path: Path,
        tools_root: Path,
        *extra_args: str,
    ) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        env["Z00Z_PENTEST_TOOLS_DIR"] = str(tools_root)
        return subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--artifact-dir",
                str(artifact_dir),
                "--scope",
                str(scope_path),
                *extra_args,
            ],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def run_parallel_static(
        self,
        artifact_dir: Path,
        scope_json: Path,
        tool_status_path: Path,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "bash",
                str(STATIC_RUNNER),
                "--artifact-dir",
                str(artifact_dir),
                "--scope-json",
                str(scope_json),
                "--tool-status",
                str(tool_status_path),
                "--mode",
                "quick",
            ],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )

    def cleanup_report_dir(self, report_dir: Path) -> None:
        """Remove runner-created report output after assertions complete."""

        if report_dir.is_dir():
            shutil.rmtree(report_dir)

    def make_executable(self, path: Path, content: str) -> Path:
        """Write a tiny executable tool fixture."""

        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        path.chmod(0o755)
        return path

    def test_static_only_run_creates_paired_artifact_and_report_dirs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            run_id = make_run_id("static-only")
            artifact_dir = temp_root / report_dir_name(run_id)
            report_dir = artifact_dir
            try:
                process = self.run_local_runner(
                    artifact_dir,
                    FIXTURES_DIR / "source_only_scope.yaml",
                    temp_root / "tools" / "penetration",
                    "--mode",
                    "quick",
                    "--profile",
                    "generic",
                    "--static-only",
                )

                self.assertEqual(process.returncode, 0, process.stderr)

                manifest = json.loads((artifact_dir / "manifest.json").read_text(encoding="utf-8"))
                tool_status = json.loads((artifact_dir / "tool-status.json").read_text(encoding="utf-8"))
                dast_skip = json.loads((artifact_dir / "dast" / "skipped.json").read_text(encoding="utf-8"))

                self.assertEqual(manifest["run_id"], run_id)
                self.assertEqual(manifest["mode"], "quick")
                self.assertEqual(manifest["profile"], "generic")
                self.assertEqual(manifest["artifact_dir"], artifact_dir.as_posix())
                self.assertEqual(manifest["report_dir"], report_dir.as_posix())
                self.assertEqual(manifest["status"], "completed-with-missing-tools")
                self.assertGreater(tool_status["summary"]["missing_required"], 0)
                self.assertEqual(dast_skip["status"], "skipped")
                self.assertEqual(dast_skip["summary"], "static-only mode requested")

                for directory_name in ("sast", "rust", "secrets", "dast", "report", "logs", "raw", "normalized"):
                    self.assertTrue((artifact_dir / directory_name).is_dir(), directory_name)

                self.assertTrue((report_dir / "security-report.md").is_file())
                self.assertTrue((report_dir / "report-metadata.json").is_file())
            finally:
                self.cleanup_report_dir(report_dir)

    def test_no_dast_run_records_scope_profile_and_skip_reason(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            run_id = make_run_id("no-dast")
            artifact_dir = temp_root / report_dir_name(run_id)
            scope_path = FIXTURES_DIR / "local_url_scope.yaml"
            report_dir = artifact_dir
            try:
                process = self.run_local_runner(
                    artifact_dir,
                    scope_path,
                    temp_root / "tools" / "penetration",
                    "--mode",
                    "deep",
                    "--profile",
                    "z00z",
                    "--no-dast",
                )

                self.assertEqual(process.returncode, 0, process.stderr)

                manifest = json.loads((artifact_dir / "manifest.json").read_text(encoding="utf-8"))
                dast_skip = json.loads((artifact_dir / "dast" / "skipped.json").read_text(encoding="utf-8"))
                report_text = (report_dir / "security-report.md").read_text(encoding="utf-8")

                self.assertEqual(manifest["mode"], "deep")
                self.assertEqual(manifest["profile"], "z00z")
                self.assertEqual(manifest["scope_path"], scope_path.as_posix())
                self.assertEqual(manifest["report_dir"], report_dir.as_posix())
                self.assertEqual(dast_skip["summary"], "DAST disabled by --no-dast")
                self.assertIn("- Profile: `z00z`", report_text)
            finally:
                self.cleanup_report_dir(report_dir)

    def test_check_only_run_skips_static_execution_and_still_builds_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            run_id = make_run_id("check-only")
            artifact_dir = temp_root / report_dir_name(run_id)
            report_dir = artifact_dir
            try:
                process = self.run_local_runner(
                    artifact_dir,
                    FIXTURES_DIR / "source_only_scope.yaml",
                    temp_root / "tools" / "penetration",
                    "--mode",
                    "standard",
                    "--profile",
                    "generic",
                    "--check-only",
                )

                self.assertEqual(process.returncode, 0, process.stderr)

                manifest = json.loads((artifact_dir / "manifest.json").read_text(encoding="utf-8"))
                sast_summary = json.loads((artifact_dir / "sast" / "summary.json").read_text(encoding="utf-8"))
                rust_summary = json.loads((artifact_dir / "rust" / "summary.json").read_text(encoding="utf-8"))
                secrets_summary = json.loads((artifact_dir / "secrets" / "summary.json").read_text(encoding="utf-8"))
                dast_skip = json.loads((artifact_dir / "dast" / "skipped.json").read_text(encoding="utf-8"))

                self.assertEqual(manifest["mode"], "standard")
                self.assertEqual(sast_summary["status"], "skipped")
                self.assertEqual(rust_summary["status"], "skipped")
                self.assertEqual(secrets_summary["status"], "skipped")
                self.assertEqual(dast_skip["summary"], "check-only mode requested")
                self.assertTrue((report_dir / "security-report.md").is_file())
            finally:
                self.cleanup_report_dir(report_dir)

    def test_parallel_static_waits_for_all_lanes_and_preserves_tool_failures(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            temp_root = Path(temp_root_name)
            artifact_dir = temp_root / "artifacts"
            scope_json = temp_root / "scope.normalized.json"
            tool_status_path = temp_root / "tool-status.json"

            semgrep_path = self.make_executable(
                temp_root / "tools" / "penetration" / "python" / "bin" / "semgrep",
                "#!/usr/bin/env sh\nexit 9\n",
            )

            scope_json.write_text(
                json.dumps(
                    {
                        "status": "OK",
                        "normalized_paths": ["scripts"],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            tool_status_path.write_text(
                json.dumps(
                    {
                        "version": 1,
                        "summary": {"missing_required": 0},
                        "tools": [
                            {
                                "name": "semgrep",
                                "status": "present",
                                "resolved_path": semgrep_path.as_posix(),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            process = self.run_parallel_static(artifact_dir, scope_json, tool_status_path)

            self.assertEqual(process.returncode, 0, process.stderr)

            orchestration = json.loads(
                (artifact_dir / "normalized" / "static-orchestration.json").read_text(encoding="utf-8")
            )
            children = {row["lane"]: row["exit_code"] for row in orchestration["children"]}
            semgrep_status = json.loads(
                (artifact_dir / "normalized" / "sast.semgrep.status.json").read_text(encoding="utf-8")
            )
            sast_summary = json.loads((artifact_dir / "sast" / "summary.json").read_text(encoding="utf-8"))

            self.assertEqual(orchestration["lane"], "static-orchestration")
            self.assertEqual(children["source-sast"], 0)
            self.assertEqual(children["rust-security"], 0)
            self.assertEqual(children["secrets-supply-chain"], 0)
            self.assertEqual(semgrep_status["status"], "failed")
            self.assertEqual(semgrep_status["exit_code"], 9)
            self.assertEqual(sast_summary["status"], "completed-with-failures")
            self.assertTrue((artifact_dir / "rust" / "summary.json").is_file())
            self.assertTrue((artifact_dir / "secrets" / "summary.json").is_file())


if __name__ == "__main__":
    unittest.main()
