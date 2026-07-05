"""Docker scope validation for the Phase 066 pentest wrapper path."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
VALIDATOR = ROOT / "scripts" / "penetration" / "validate_pentest_docker_scope.py"
README = ROOT / "tools" / "penetration" / "docker" / "README.md"
DOCKERFILE = ROOT / "tools" / "penetration" / "docker" / "Dockerfile"
RUNNER = ROOT / "tools" / "penetration" / "docker" / "run_pentest_container.sh"


class DockerScopeValidationTest(unittest.TestCase):
    """Ensure Docker pentest scripts stay separate from formal verification."""

    maxDiff = None

    def test_live_scope_validator_passes_and_writes_json(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            json_path = Path(tmp_dir) / "scope.json"
            completed = subprocess.run(
                [
                    "python3",
                    str(VALIDATOR),
                    "tools/penetration/docker",
                    "scripts/penetration",
                    "--json-out",
                    str(json_path),
                ],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=True,
            )
            self.assertIn("validated_json=", completed.stdout)
            payload = json.loads(json_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["findings"], [])
            self.assertIn("scripts/run_pentest_tools.sh", payload["checked_files"])
            self.assertIn(
                "tools/penetration/docker/run_pentest_container.sh",
                payload["checked_files"],
            )

    def test_validator_rejects_forbidden_formal_verification_call(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            fixture_path = fixture_dir / "forbidden.sh"
            fixture_path.write_text(
                "#!/usr/bin/env bash\nscripts/verification-tools/install-verification-tools.sh --install --profile research --strict\n",
                encoding="utf-8",
            )
            completed = subprocess.run(
                ["python3", str(VALIDATOR), str(fixture_dir)],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(completed.returncode, 1)
            payload = json.loads(completed.stdout)
            self.assertEqual(len(payload["findings"]), 1)
            self.assertEqual(payload["findings"][0]["pattern"], "install-verification-tools")

    def test_validator_allows_formal_verification_skip_dirs(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            fixture_dir = Path(tmp_dir)
            fixture_path = fixture_dir / "allowed.sh"
            json_path = fixture_dir / "scope.json"
            fixture_path.write_text(
                '#!/usr/bin/env bash\ntrivy fs --skip-dirs "$ROOT/tools/formal_verification" "$ROOT"\n',
                encoding="utf-8",
            )
            completed = subprocess.run(
                ["python3", str(VALIDATOR), str(fixture_dir), "--json-out", str(json_path)],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=True,
            )
            self.assertIn("validated_json=", completed.stdout)
            payload = json.loads(json_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["findings"], [])

    def test_runner_enforces_archive_only_attached_non_privileged_contract(self) -> None:
        script = RUNNER.read_text(encoding="utf-8")
        self.assertIn("ERROR: --archive is required", script)
        self.assertIn('docker run --rm -i \\', script)
        self.assertIn('--user "$CONTAINER_USER" \\', script)
        self.assertIn("--read-only \\", script)
        self.assertIn("--cap-drop=ALL \\", script)
        self.assertIn("--security-opt=no-new-privileges:true \\", script)
        self.assertIn('--tmpfs /workspace:rw,exec,nosuid,nodev,mode=1777 \\', script)
        self.assertIn(' -v "$ARCHIVE_PATH:/input/archive.tar.gz:ro" \\', script)
        self.assertNotIn("--privileged", script)
        self.assertNotIn("docker.sock", script)
        self.assertNotIn("docker run --rm -d", script)

    def test_docker_readme_documents_operator_contract(self) -> None:
        readme = README.read_text(encoding="utf-8")
        self.assertIn("Docker is optional", readme)
        self.assertIn("archive-driven", readme)
        self.assertIn("reports/z00z-pentests-report-", readme)
        self.assertIn("No formal-verification tooling", readme)
        self.assertIn("No `--privileged`", readme)
        self.assertIn("No `/var/run/docker.sock` mount", readme)
        self.assertIn("Logs stay attached", readme)

    def test_optional_dockerfile_stays_minimal_and_non_offensive(self) -> None:
        dockerfile = DOCKERFILE.read_text(encoding="utf-8")
        lowered = dockerfile.lower()
        self.assertIn("FROM python:3.12-slim", dockerfile)
        self.assertIn("USER pentest", dockerfile)
        for forbidden in (
            "nmap",
            "masscan",
            "hydra",
            "medusa",
            "metasploit",
            "mcp",
            "llm_api_key",
            "docker.sock",
        ):
            self.assertNotIn(forbidden, lowered)


if __name__ == "__main__":
    unittest.main()
