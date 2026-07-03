"""Portability coverage for the Phase 066 pentest archive path."""

from __future__ import annotations

import json
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def run_command(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )


class PackagingPortabilityTest(unittest.TestCase):
    """Validate archive contents and CLI contracts for WS-10."""

    maxDiff = None

    def test_pack_archive_includes_pentest_sources_and_excludes_heavy_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            archive_path = Path(tmp_dir) / "z00z-pentest-portable.tar.gz"
            run_command("bash", str(ROOT / "pack_z00z_project.sh"), "--output", str(archive_path))

            with tarfile.open(archive_path, "r:gz") as archive:
                names = archive.getnames()
                top_dirs = {name.split("/", 1)[0] for name in names if name}
                self.assertEqual(len(top_dirs), 1)
                top_dir = next(iter(top_dirs))

                expected_members = {
                    f"{top_dir}/z00z_penetration_tests.sh",
                    f"{top_dir}/tools/penetration/docker/run_pentest_container.sh",
                    f"{top_dir}/scripts/penetration/validate_pentest_docker_scope.py",
                    f"{top_dir}/tools/penetration/manifests/tool-versions.lock",
                    f"{top_dir}/.github/prompts/pentest-local.prompt.md",
                    f"{top_dir}/.github/agents/pentest-rust-reviewer.agent.md",
                }
                for member in expected_members:
                    self.assertIn(member, names)

                forbidden_prefixes = (
                    f"{top_dir}/tools/penetration/cache/",
                    f"{top_dir}/tools/penetration/cargo/",
                    f"{top_dir}/tools/penetration/go/",
                    f"{top_dir}/tools/penetration/python/bin/",
                    f"{top_dir}/tools/penetration/python/pipx/",
                    f"{top_dir}/tools/penetration/python/uv-tools/",
                    f"{top_dir}/tools/formal_verification/",
                )
                for name in names:
                    self.assertFalse(
                        any(name.startswith(prefix) for prefix in forbidden_prefixes),
                        name,
                    )

                manifest_member = archive.extractfile(f"{top_dir}/.portable-transfer/manifest.json")
                self.assertIsNotNone(manifest_member)
                manifest = json.loads(manifest_member.read().decode("utf-8"))
                self.assertIn("tools/penetration/cache", manifest["excluded_paths"]["exact_dirs"])
                self.assertIn("tools/penetration/go", manifest["excluded_paths"]["exact_dirs"])
                self.assertIn("tools/penetration/python/uv-tools", manifest["excluded_paths"]["exact_dirs"])

    def test_entrypoint_and_unpack_help_expose_portable_contract(self) -> None:
        entry_help = run_command("bash", str(ROOT / "z00z_penetration_tests.sh"), "--help").stdout
        self.assertIn("--docker-sandbox", entry_help)
        self.assertIn("--archive <path>", entry_help)
        self.assertIn("--pack", entry_help)

        unpack_help = run_command("bash", str(ROOT / "unpack_z00z_project.sh"), "--help").stdout
        self.assertIn("--skip-formal-verification", unpack_help)


if __name__ == "__main__":
    unittest.main()
