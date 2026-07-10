#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TODO_PATH = ROOT / ".planning/phases/000/067-Sharded-Concensus/067-TODO.md"
VERDICT_PATH = ROOT / ".planning/phases/000/067-Sharded-Concensus/067-verdict.md"
REGISTRY_PATH = ROOT / ".planning/phases/000/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md"
REGISTRY_DIR = REGISTRY_PATH.parent

ALLOWED_LEVELS = {"live", "simulated-full", "live-claim-removed", "not-claimed"}
EXTRA_REQUIRED_TERMS = {
    "Celestia finality",
    "deterministic replicated planner",
    "planner HA",
    "devnet",
    "Transport",
    "Validator certificate gate",
    "Production signature seam",
    "Slashing/economics",
}
VERDICT_ALIASES = {
    "Failover/takeover": "Failover",
    "BFT committee": "BFT",
    "Glossary terms": None,
}


@dataclass(frozen=True)
class ClaimRow:
    term: str
    owner: str
    artifact: str
    positive: str
    negative: str
    claim_level: str
    evidence_refs: str
    plan_id: str


def main() -> int:
    todo_terms = parse_named_table(TODO_PATH, "| Term | Meaning in this document |", 0)
    verdict_terms = parse_named_table(
        VERDICT_PATH, "| Term or capability", 0
    )
    registry_rows = parse_registry(REGISTRY_PATH)

    errors: list[str] = []
    registry_terms: set[str] = set()
    duplicate_terms: set[str] = set()

    for row in registry_rows:
        if row.term in registry_terms:
            duplicate_terms.add(row.term)
        registry_terms.add(row.term)
        validate_row(row, errors)

    for term in sorted(duplicate_terms):
        errors.append(f"duplicate registry term: {term}")

    required_terms = set(todo_terms)
    for term in verdict_terms:
        alias = VERDICT_ALIASES.get(term, term)
        if alias is not None:
            required_terms.add(alias)
    required_terms.update(EXTRA_REQUIRED_TERMS)

    missing_terms = sorted(required_terms - registry_terms)
    for term in missing_terms:
        errors.append(f"missing registry term: {term}")
    errors.extend(find_noncanonical_registry_variants())

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    print(
        "claim audit ok: "
        f"{len(todo_terms)} glossary terms, {len(verdict_terms)} verdict terms, "
        f"{len(registry_rows)} registry rows"
    )
    return 0


def parse_named_table(path: Path, header_prefix: str, field_index: int) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    in_table = False
    values: list[str] = []
    for line in lines:
        if line.startswith(header_prefix):
            in_table = True
            continue
        if not in_table:
            continue
        if not line.startswith("|"):
            break
        if line.startswith("| ---"):
            continue
        parts = [part.strip() for part in line.strip("|").split("|")]
        if len(parts) <= field_index:
            break
        values.append(parts[field_index])
    return values


def parse_registry(path: Path) -> list[ClaimRow]:
    lines = path.read_text(encoding="utf-8").splitlines()
    start = None
    for index, line in enumerate(lines):
        if line.startswith("| term | code owner | artifact/API |"):
            start = index + 2
            break
    if start is None:
        raise RuntimeError(f"registry table not found in {path}")

    rows: list[ClaimRow] = []
    for line in lines[start:]:
        if not line.startswith("|"):
            break
        parts = [part.strip() for part in line.strip("|").split("|")]
        if len(parts) != 8:
            raise RuntimeError(f"registry row must have 8 columns: {line}")
        rows.append(ClaimRow(*parts))
    return rows


def validate_row(row: ClaimRow, errors: list[str]) -> None:
    for field_name, value in [
        ("term", row.term),
        ("code owner", row.owner),
        ("artifact/API", row.artifact),
        ("positive test", row.positive),
        ("negative test", row.negative),
        ("claim level", row.claim_level),
        ("evidence refs", row.evidence_refs),
        ("plan id", row.plan_id),
    ]:
        if not value:
            errors.append(f"{row.term}: empty {field_name}")

    if row.claim_level not in ALLOWED_LEVELS:
        errors.append(f"{row.term}: invalid claim level {row.claim_level}")
    validate_existing_path(row.term, "code owner", row.owner, errors)
    validate_test_ref(row.term, "positive test", row.positive, errors)
    validate_test_ref(row.term, "negative test", row.negative, errors)
    if not re.fullmatch(r"067-\d{2}", row.plan_id):
        errors.append(f"{row.term}: invalid plan id {row.plan_id}")
    if ";" not in row.evidence_refs and row.evidence_refs.endswith(".json") is False:
        errors.append(f"{row.term}: evidence refs must cite executable artifacts")


def find_noncanonical_registry_variants() -> list[str]:
    errors: list[str] = []
    for pattern in ("067-GLOSSARY-CLAIMS.md.*", "067-GLOSSARY-CLAIMS.*"):
        for candidate in sorted(REGISTRY_DIR.glob(pattern)):
            if candidate == REGISTRY_PATH:
                continue
            rel = candidate.relative_to(ROOT)
            errors.append(f"non-canonical glossary registry variant: {rel}")
    return errors


def validate_existing_path(
    term: str, field_name: str, raw_path: str, errors: list[str]
) -> None:
    path = ROOT / raw_path
    if not path.exists():
        errors.append(f"{term}: missing {field_name} path {raw_path}")


def validate_test_ref(term: str, field_name: str, raw_ref: str, errors: list[str]) -> None:
    if "::" not in raw_ref:
        errors.append(f"{term}: {field_name} must reference a test anchor")
        return

    raw_path, anchor = raw_ref.split("::", 1)
    if not raw_path or not anchor:
        errors.append(f"{term}: malformed {field_name} {raw_ref}")
        return

    path = ROOT / raw_path
    if not path.is_file():
        errors.append(f"{term}: missing {field_name} file {raw_path}")
        return

    if anchor not in read_text(path):
        errors.append(f"{term}: missing {field_name} anchor {raw_ref}")


@lru_cache(maxsize=None)
def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


if __name__ == "__main__":
    sys.exit(main())
