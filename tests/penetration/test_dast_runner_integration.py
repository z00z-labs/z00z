"""Integration coverage for the Phase 066 bounded local DAST runner."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUNNER = ROOT / "scripts" / "penetration" / "run_local_dast.sh"


def _dedent(text: str) -> str:
    return textwrap.dedent(text).strip() + "\n"


class DastRunnerIntegrationTest(unittest.TestCase):
    """Exercise the DAST shell runner through its public CLI contract."""

    maxDiff = None

    def write_scope(self, directory: Path, scope_text: str) -> Path:
        scope_path = directory / "scope.yaml"
        scope_path.write_text(_dedent(scope_text), encoding="utf-8")
        return scope_path

    def make_tool_status(self, directory: Path, tool_paths: dict[str, Path]) -> Path:
        tool_status = directory / "tool-status.json"
        payload = {
            "version": 1,
            "tools": [],
        }
        for tool_name in ("nmap", "nuclei", "httpx", "katana", "ffuf", "gobuster"):
            resolved_path = str(tool_paths.get(tool_name, ""))
            payload["tools"].append(
                {
                    "name": tool_name,
                    "category": "local-dast",
                    "required": tool_name in {"nmap", "nuclei", "httpx", "ffuf"},
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

    def make_fake_tool(self, directory: Path, tool_name: str) -> Path:
        bin_dir = directory / "bin"
        bin_dir.mkdir(parents=True, exist_ok=True)
        script_path = bin_dir / tool_name

        script = """\
        #!/usr/bin/env python3
        import json
        import os
        import pathlib
        import sys

        tool = pathlib.Path(sys.argv[0]).name
        args = sys.argv[1:]
        log_dir = pathlib.Path(os.environ["TEST_DAST_LOG_DIR"])
        log_dir.mkdir(parents=True, exist_ok=True)
        (log_dir / f"{tool}.args").write_text(json.dumps(args) + "\\n", encoding="utf-8")

        if tool == "nmap":
            prefix = None
            if "-oA" in args:
                prefix = pathlib.Path(args[args.index("-oA") + 1])
                prefix.parent.mkdir(parents=True, exist_ok=True)
            if prefix is not None:
                last_arg = args[-1]
                if "-iL" in args:
                    host_path = pathlib.Path(args[args.index("-iL") + 1])
                    hosts = [
                        line.strip()
                        for line in host_path.read_text(encoding="utf-8").splitlines()
                        if line.strip()
                    ]
                else:
                    hosts = [last_arg]
                gnmap_lines = []
                for host in hosts:
                    if prefix.name == "nmap.discovery":
                        gnmap_lines.append(
                            f"Host: {host} ()\\tPorts: 8080/open/tcp//http-alt///, 8443/open/tcp//ssl|http-alt///"
                        )
                    else:
                        gnmap_lines.append(
                            f"Host: {host} ()\\tPorts: 8080/open/tcp//http-alt///"
                        )
                pathlib.Path(str(prefix) + ".gnmap").write_text("\\n".join(gnmap_lines) + "\\n", encoding="utf-8")
                pathlib.Path(str(prefix) + ".nmap").write_text("fixture nmap\\n", encoding="utf-8")
                pathlib.Path(str(prefix) + ".xml").write_text("<nmaprun/>\\n", encoding="utf-8")
            sys.exit(0)

        output_path = None
        if "-o" in args:
            output_path = pathlib.Path(args[args.index("-o") + 1])
        if output_path is not None:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            payload = {"tool": tool, "args": args}
            output_path.write_text(json.dumps(payload, indent=2) + "\\n", encoding="utf-8")
        sys.exit(0)
        """
        script_path.write_text(_dedent(script), encoding="utf-8")
        script_path.chmod(0o755)
        return script_path

    def run_runner(
        self,
        artifact_dir: Path,
        scope_path: Path,
        tool_status_path: Path,
        *,
        log_dir: Path,
        mode: str = "standard",
    ) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        env["TEST_DAST_LOG_DIR"] = str(log_dir)
        return subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--artifact-dir",
                str(artifact_dir),
                "--scope",
                str(scope_path),
                "--tool-status",
                str(tool_status_path),
                "--mode",
                mode,
                "--profile",
                "generic",
            ],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_public_target_is_rejected_before_any_tool_runs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            log_dir = temp_dir / "logs"
            scope_path = self.write_scope(
                temp_dir,
                """
                mode: local-only
                allowed_paths: [scripts]
                excluded_paths: [target]
                allowed_hosts: [127.0.0.1]
                allowed_urls: [https://example.com]
                forbidden: [public-targets]
                rate_limits:
                  requests_per_second: 2
                  max_concurrency: 2
                  timeout_seconds: 30
                evidence_required: true
                """,
            )
            fake_httpx = self.make_fake_tool(temp_dir, "httpx")
            tool_status = self.make_tool_status(temp_dir, {"httpx": fake_httpx})

            process = self.run_runner(
                artifact_dir,
                scope_path,
                tool_status,
                log_dir=log_dir,
            )

            self.assertEqual(process.returncode, 1, process.stderr)
            validation_artifact = artifact_dir / "dast" / "validation-failed.json"
            self.assertTrue(validation_artifact.is_file())
            self.assertFalse((log_dir / "httpx.args").exists())

    def test_no_target_scope_creates_skip_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            log_dir = temp_dir / "logs"
            scope_path = self.write_scope(
                temp_dir,
                """
                mode: local-only
                allowed_paths: [scripts]
                excluded_paths: [target]
                allowed_hosts: []
                allowed_urls: []
                forbidden: [public-targets]
                rate_limits:
                  requests_per_second: 2
                  max_concurrency: 2
                  timeout_seconds: 30
                evidence_required: true
                """,
            )
            tool_status = self.make_tool_status(temp_dir, {})

            process = self.run_runner(
                artifact_dir,
                scope_path,
                tool_status,
                log_dir=log_dir,
            )

            self.assertEqual(process.returncode, 0, process.stderr)
            skipped = json.loads((artifact_dir / "dast" / "skipped.json").read_text(encoding="utf-8"))
            self.assertEqual(skipped["status"], "skipped")
            self.assertIn("no allowed local DAST target", skipped["summary"])
            self.assertFalse(any(log_dir.iterdir()) if log_dir.exists() else False)

    def test_bounded_commands_use_two_pass_nmap_and_local_url_tools(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir_name:
            temp_dir = Path(temp_dir_name)
            artifact_dir = temp_dir / "artifacts"
            log_dir = temp_dir / "logs"
            scope_path = self.write_scope(
                temp_dir,
                """
                mode: local-only
                allowed_paths: [scripts]
                excluded_paths: [target]
                allowed_hosts: [127.0.0.1]
                allowed_urls: [http://127.0.0.1:18080]
                forbidden: [public-targets]
                rate_limits:
                  requests_per_second: 2
                  max_concurrency: 2
                  timeout_seconds: 15
                evidence_required: true
                """,
            )
            tool_paths = {
                tool_name: self.make_fake_tool(temp_dir, tool_name)
                for tool_name in ("nmap", "httpx", "nuclei", "katana", "ffuf")
            }
            tool_status = self.make_tool_status(temp_dir, tool_paths)

            process = self.run_runner(
                artifact_dir,
                scope_path,
                tool_status,
                log_dir=log_dir,
            )

            self.assertEqual(process.returncode, 0, process.stderr)

            discovery_status = json.loads(
                (artifact_dir / "normalized" / "dast.nmap.discovery.status.json").read_text(encoding="utf-8")
            )
            self.assertIn("--top-ports", discovery_status["command_display"])
            self.assertIn("-iL", discovery_status["command_display"])

            service_status = json.loads(
                (artifact_dir / "normalized" / "dast.nmap.services.127_0_0_1.status.json").read_text(encoding="utf-8")
            )
            self.assertIn("-sV", service_status["command_display"])
            self.assertIn("-sC", service_status["command_display"])
            self.assertIn("-p 8080\\,8443", service_status["command_display"])

            httpx_status = json.loads(
                (artifact_dir / "normalized" / "dast.httpx.status.json").read_text(encoding="utf-8")
            )
            self.assertIn("-path /\\,/health\\,/readyz\\,/metrics", httpx_status["command_display"])
            self.assertIn("-json", httpx_status["command_display"])

            nuclei_status = json.loads(
                (artifact_dir / "normalized" / "dast.nuclei.status.json").read_text(encoding="utf-8")
            )
            self.assertIn("-ni", nuclei_status["command_display"])
            self.assertIn("-t http/", nuclei_status["command_display"])
            self.assertIn("-tags tech\\,misconfig\\,exposure", nuclei_status["command_display"])
            self.assertIn("-severity low\\,medium\\,high\\,critical", nuclei_status["command_display"])

            ffuf_status = json.loads(
                (artifact_dir / "normalized" / "dast.ffuf.target01.status.json").read_text(encoding="utf-8")
            )
            self.assertIn("-noninteractive", ffuf_status["command_display"])
            self.assertIn("/FUZZ", ffuf_status["command_display"])
            self.assertIn("-of json", ffuf_status["command_display"])

            summary = json.loads((artifact_dir / "dast" / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["status"], "completed")


if __name__ == "__main__":
    unittest.main()
