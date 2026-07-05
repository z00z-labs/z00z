#!/usr/bin/env python3
"""Forward to the L2 domain separation checker."""

from __future__ import annotations

import runpy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
runpy.run_path(str(ROOT / ".github/skills/z00z-l2-crypto-protocol-gate/scripts/check-domain-separation.py"), run_name="__main__")
