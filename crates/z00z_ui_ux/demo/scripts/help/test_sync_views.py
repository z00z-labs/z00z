#!/usr/bin/env python3
"""Unit coverage for the canonical-only English Help view synchronizer."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("sync_views.py")
SPEC = importlib.util.spec_from_file_location("sync_views", SCRIPT)
assert SPEC and SPEC.loader
SYNC = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SYNC)


class ViewSynchronizerTests(unittest.TestCase):
    def test_extractor_tracks_visible_sections_terms_and_controls(self) -> None:
        extractor = SYNC.ViewExtractor()
        extractor.feed("""
          <main id="main-content">
            <h2>Wallet Assets</h2><p>Available balance</p>
            <button type="button" aria-label="Send asset" data-demo-action="send">Send</button>
            <span aria-hidden="true">hidden icon text</span>
          </main>
        """)
        snapshot = extractor.snapshot()

        self.assertEqual(snapshot["sections"], ["Wallet Assets"])
        self.assertIn("Available balance", snapshot["terms"])
        self.assertIn("Send asset", snapshot["terms"])
        self.assertNotIn("hidden icon text", snapshot["terms"])
        self.assertIn("button|type=button|aria-label=Send asset|data-demo-action=send", snapshot["components"])

    def test_changes_report_additions_and_removals(self) -> None:
        previous = {"components": ["button|type=button"], "sections": ["Before"], "terms": ["Old"]}
        current = {"components": ["button|type=button", "input|name=amount"], "sections": ["After"], "terms": ["New"]}

        observed = SYNC.changes(previous, current)

        self.assertEqual(observed["components"], ["input|name=amount"])
        self.assertEqual(observed["sections_removed"], ["Before"])
        self.assertEqual(observed["terms_removed"], ["Old"])

    def test_changes_report_a_presentation_update_after_baseline_migration(self) -> None:
        previous = {
            "components": [],
            "sections": [],
            "terms": [],
            "presentation_sha256": "old",
            "version": 3,
        }
        current = {
            "components": [],
            "sections": [],
            "terms": [],
            "presentation_sha256": "new",
            "version": 3,
        }

        self.assertEqual(SYNC.changes(previous, current)["presentation"], ["App View layout or presentation changed"])

    def test_write_result_updates_capture_state_without_creating_markdown(self) -> None:
        original_roots = (SYNC.DEMO_ROOT, SYNC.HELP_ROOT, SYNC.STATE_ROOT)
        try:
            with tempfile.TemporaryDirectory() as temporary_directory:
                temporary_root = Path(temporary_directory)
                SYNC.DEMO_ROOT = temporary_root
                SYNC.HELP_ROOT = temporary_root / "help"
                SYNC.STATE_ROOT = SYNC.HELP_ROOT / "en" / "_generated"
                view = {
                    "id": "wallet.assets",
                    "pagePath": "wallet/assets.md",
                    "routeId": "wallet.assets",
                    "scope": "context",
                    "screenshot": "help/assets/en/wallet-assets.png",
                }
                canonical = SYNC.page_path(view)
                canonical.parent.mkdir(parents=True)
                canonical.write_text("authored canonical content\n", encoding="utf-8")
                asset = temporary_root / view["screenshot"]
                asset.parent.mkdir(parents=True)
                image = b"PNG"
                asset.write_bytes(image)
                snapshot = {
                    "components": ["button|type=button"],
                    "sections": ["Assets"],
                    "screenshot_sha256": SYNC.hashlib.sha256(image).hexdigest(),
                    "terms": ["Balance"],
                    "screenshot": view["screenshot"],
                    "topic_id": view["id"],
                }

                changed, preserved = SYNC.write_result(view, snapshot, image, False)

                self.assertEqual((changed, preserved), (1, 0))
                self.assertEqual(canonical.read_text(encoding="utf-8"), "authored canonical content\n")
                self.assertEqual(list(canonical.parent.glob("*.md")), [canonical])
                self.assertEqual(SYNC.load_state(SYNC.state_path(view))["terms"], ["Balance"])

                updated_snapshot = {**snapshot, "terms": ["Balance", "Available"]}
                changed, preserved = SYNC.write_result(view, updated_snapshot, image, False)
                self.assertEqual((changed, preserved), (1, 0))
                self.assertEqual(list(canonical.parent.glob("*.md")), [canonical])
                self.assertEqual(SYNC.load_state(SYNC.state_path(view))["terms"], ["Balance", "Available"])
                self.assertEqual(canonical.read_text(encoding="utf-8"), "authored canonical content\n")
        finally:
            SYNC.DEMO_ROOT, SYNC.HELP_ROOT, SYNC.STATE_ROOT = original_roots


if __name__ == "__main__":
    unittest.main(verbosity=2)
