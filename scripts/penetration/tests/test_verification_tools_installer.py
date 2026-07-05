"""Regression coverage for verification-tools installer repo-config cleanup."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "scripts" / "verification-tools" / "install-verification-tools.sh"


class VerificationToolsInstallerTest(unittest.TestCase):
    """Exercise the public installer CLI around versions.env authority."""

    maxDiff = None

    def run_installer(self, tools_dir: Path, *args: str) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        env["Z00Z_VERIFY_TOOLS_DIR"] = str(tools_dir)
        return subprocess.run(
            [
                "bash",
                str(SCRIPT),
                *args,
            ],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_check_removes_stale_shadow_versions_env_without_requiring_install(self) -> None:
        with tempfile.TemporaryDirectory() as temp_root_name:
            tools_dir = Path(temp_root_name) / "tools" / "formal_verification"
            tools_dir.mkdir(parents=True, exist_ok=True)
            stale_versions = tools_dir / "versions.env"
            stale_versions.write_text("export Z00Z_VERUS_TOOLCHAIN=stale\n", encoding="utf-8")

            process = self.run_installer(tools_dir, "--check")

            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertFalse(stale_versions.exists(), process.stdout)


if __name__ == "__main__":
    unittest.main()
