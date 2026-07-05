"""Integration coverage for the Phase 066 source-aware SAST runner."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RUNNER = ROOT / "scripts" / "penetration" / "run_source_sast.sh"


def _dedent(text: str) -> str:
    return textwrap.dedent(text).strip() + "\n"


class SourceSastRunnerTest(unittest.TestCase):
    """Exercise the source-aware static lane through its public CLI."""

    maxDiff = None

    def make_tool_status(self, directory: Path, tool_paths: dict[str, Path]) -> Path:
        tool_status = directory / "tool-status.json"
        payload = {
            "version": 1,
            "tools": [],
        }
        for tool_name in ("semgrep", "sg", "tree-sitter", "gitleaks", "trufflehog", "trivy"):
            resolved_path = str(tool_paths.get(tool_name, ""))
            payload["tools"].append(
                {
                    "name": tool_name,
                    "category": "source-sast",
                    "required": tool_name in {"semgrep", "gitleaks", "trufflehog", "trivy"},
                    "status": "present" if resolved_path else "missing",
                    "source": "test-fixture" if resolved_path else "missing",
                    "resolved_path": resolved_path,
                    "wrapper_path": resolved_path,
                    "payload_path": resolved_path,
                    "version": "fixture-1",
                }
            )
        tool_status.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
        return tool_status

    def make_scope_json(self, directory: Path) -> Path:
        scope_path = directory / "scope.normalized.json"
        scope_path.write_text(
            json.dumps(
                {
                    "status": "OK",
                    "normalized_paths": [
                        "scripts/penetration",
                        "scripts/run_pentest_tools.sh",
                    ],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        return scope_path

    def make_fake_tool(self, directory: Path, tool_name: str) -> Path:
        bin_dir = directory / "bin"
        bin_dir.mkdir(parents=True, exist_ok=True)
        script_path = bin_dir / tool_name

        script = """\
        #!/usr/bin/env python3
        import json
        import pathlib
        import sys

        tool = pathlib.Path(sys.argv[0]).name
        args = sys.argv[1:]

        if tool == "semgrep":
            payload = {
                "paths": {
                    "scanned": [
                        ".github/.gsd-profile",
                        "scripts/penetration/scope.yaml",
                        "scripts/penetration/run_local_pentest.sh",
                        "scripts/run_pentest_tools.sh"
                    ]
                },
                "results": [
                    {
                        "path": "scripts/penetration/run_source_sast.sh",
                        "check_id": "fixture.semgrep"
                    }
                ]
            }
            json.dump(payload, sys.stdout)
            sys.stdout.write("\\n")
            sys.exit(0)

        if tool == "sg":
            json.dump({"tool": tool, "args": args}, sys.stdout)
            sys.stdout.write("\\n")
            sys.exit(0)

        if tool == "tree-sitter":
            sys.stdout.write("fixture tree-sitter parse\\n")
            sys.exit(0)

        if tool == "gitleaks":
            report_path = pathlib.Path(args[args.index("--report-path") + 1])
            report_path.parent.mkdir(parents=True, exist_ok=True)
            report_path.write_text("[]\\n", encoding="utf-8")
            sys.exit(0)

        if tool == "trufflehog":
            json.dump({"path": "fixture", "tool": tool}, sys.stdout)
            sys.stdout.write("\\n")
            sys.exit(0)

        if tool == "trivy":
            output_path = pathlib.Path(args[args.index("--output") + 1])
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text('{"Results": []}\\n', encoding="utf-8")
            sys.exit(0)

        raise SystemExit(f"unexpected tool fixture: {tool}")
        """
        script_path.write_text(_dedent(script), encoding="utf-8")
        script_path.chmod(0o755)
        return script_path

    def run_runner(
        self,
        artifact_dir: Path,
        scope_json: Path,
        tool_status_path: Path,
        *,
        mode: str = "standard",
        profile: str = "generic",
    ) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        return subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--artifact-dir",
                str(artifact_dir),
                "--scope-json",
                str(scope_json),
                "--tool-status",
                str(tool_status_path),
                "--mode",
                mode,
                "--profile",
                profile,
            ],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_runner_derives_deterministic_ast_targets_and_executes_secret_tools(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            scope_json = self.make_scope_json(temp_dir)
            tool_paths = {
                tool_name: self.make_fake_tool(temp_dir, tool_name)
                for tool_name in ("semgrep", "sg", "gitleaks", "trufflehog", "trivy")
            }
            tool_status = self.make_tool_status(temp_dir, tool_paths)

            process = self.run_runner(artifact_dir, scope_json, tool_status, profile="z00z")

            self.assertEqual(process.returncode, 0, process.stderr)

            targets_text = (artifact_dir / "raw" / "ast" / "sg-targets.txt").read_text(encoding="utf-8")
            summary = json.loads((artifact_dir / "sast" / "summary.json").read_text(encoding="utf-8"))
            sg_status = json.loads(
                (artifact_dir / "normalized" / "sast.sg.status.json").read_text(encoding="utf-8")
            )
            tree_status = json.loads(
                (artifact_dir / "normalized" / "sast.tree-sitter.status.json").read_text(encoding="utf-8")
            )
            trufflehog_status = json.loads(
                (artifact_dir / "normalized" / "sast.trufflehog.status.json").read_text(encoding="utf-8")
            )

            self.assertIn("scripts/penetration/run_local_pentest.sh", targets_text)
            self.assertIn("scripts/run_pentest_tools.sh", targets_text)
            self.assertIn("scripts/penetration/run_source_sast.sh", targets_text)
            self.assertNotIn(".github/.gsd-profile", targets_text)
            self.assertNotIn("scripts/penetration/scope.yaml", targets_text)
            self.assertEqual(summary["status"], "completed")
            self.assertEqual(sg_status["status"], "passed")
            self.assertEqual(tree_status["status"], "skipped")
            self.assertEqual(trufflehog_status["status"], "passed")
            self.assertTrue((artifact_dir / "raw" / "secrets" / "sast.trufflehog.json").is_file())
            self.assertTrue((artifact_dir / "raw" / "secrets" / "sast.gitleaks.json").is_file())
            self.assertTrue((artifact_dir / "raw" / "secrets" / "sast.trivy.json").is_file())

    def test_runner_records_missing_ast_family_when_no_structural_tool_is_available(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            scope_json = self.make_scope_json(temp_dir)
            tool_paths = {
                tool_name: self.make_fake_tool(temp_dir, tool_name)
                for tool_name in ("semgrep", "gitleaks", "trufflehog", "trivy")
            }
            tool_status = self.make_tool_status(temp_dir, tool_paths)

            process = self.run_runner(artifact_dir, scope_json, tool_status)

            self.assertEqual(process.returncode, 0, process.stderr)

            summary = json.loads((artifact_dir / "sast" / "summary.json").read_text(encoding="utf-8"))
            sg_status = json.loads(
                (artifact_dir / "normalized" / "sast.sg.status.json").read_text(encoding="utf-8")
            )
            tree_status = json.loads(
                (artifact_dir / "normalized" / "sast.tree-sitter.status.json").read_text(encoding="utf-8")
            )

            self.assertEqual(summary["status"], "completed-with-missing-tools")
            self.assertEqual(sg_status["status"], "missing")
            self.assertEqual(tree_status["status"], "missing")

    def test_deep_mode_runs_sg_and_tree_sitter_when_both_are_available(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            scope_json = self.make_scope_json(temp_dir)
            tool_paths = {
                tool_name: self.make_fake_tool(temp_dir, tool_name)
                for tool_name in ("semgrep", "sg", "tree-sitter", "gitleaks", "trufflehog", "trivy")
            }
            tool_status = self.make_tool_status(temp_dir, tool_paths)

            process = self.run_runner(artifact_dir, scope_json, tool_status, mode="deep")

            self.assertEqual(process.returncode, 0, process.stderr)

            summary = json.loads((artifact_dir / "sast" / "summary.json").read_text(encoding="utf-8"))
            sg_status = json.loads(
                (artifact_dir / "normalized" / "sast.sg.status.json").read_text(encoding="utf-8")
            )
            tree_status = json.loads(
                (artifact_dir / "normalized" / "sast.tree-sitter.status.json").read_text(encoding="utf-8")
            )
            tree_output = (artifact_dir / "raw" / "ast" / "tree-sitter.txt").read_text(encoding="utf-8")

            self.assertEqual(summary["status"], "completed")
            self.assertEqual(sg_status["status"], "passed")
            self.assertEqual(tree_status["status"], "passed")
            self.assertIn("fixture tree-sitter parse", tree_output)

    def test_deep_mode_marks_missing_tree_sitter_instead_of_skipping_it(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            scope_json = self.make_scope_json(temp_dir)
            tool_paths = {
                tool_name: self.make_fake_tool(temp_dir, tool_name)
                for tool_name in ("semgrep", "sg", "gitleaks", "trufflehog", "trivy")
            }
            tool_status = self.make_tool_status(temp_dir, tool_paths)

            process = self.run_runner(artifact_dir, scope_json, tool_status, mode="deep")

            self.assertEqual(process.returncode, 0, process.stderr)

            summary = json.loads((artifact_dir / "sast" / "summary.json").read_text(encoding="utf-8"))
            sg_status = json.loads(
                (artifact_dir / "normalized" / "sast.sg.status.json").read_text(encoding="utf-8")
            )
            tree_status = json.loads(
                (artifact_dir / "normalized" / "sast.tree-sitter.status.json").read_text(encoding="utf-8")
            )

            self.assertEqual(summary["status"], "completed-with-missing-tools")
            self.assertEqual(sg_status["status"], "passed")
            self.assertEqual(tree_status["status"], "missing")


if __name__ == "__main__":
    unittest.main()
