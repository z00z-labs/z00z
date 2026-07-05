#!/usr/bin/env python3
"""Build a tracked-file coverage manifest for the Z00Z verification report."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from collections import Counter
from pathlib import Path


DEFAULT_TOP_LEVEL_EXCLUDES = {
    ".git",
    ".cache",
    ".codeviz",
    ".reviews",
    ".venv",
    ".temp",
    ".planning",
    ".portable-transfer",
    ".codex",
    ".bg-shell",
    "logs",
    "reports",
    "target",
}
DEFAULT_RECURSIVE_DIR_EXCLUDES = {
    ".git",
    "__pycache__",
    "outputs",
    ".cache",
    ".planning",
    "target",
    "fuzz_target",
    "target_fuzz",
    ".codeviz",
    ".reviews",
    ".venv",
    ".temp",
    ".z00z-storage-redb",
    "logs",
    "reports",
    "node_modules",
}
DEFAULT_EXACT_DIR_EXCLUDES = {
    ".agents/.install-backups",
    "tools/formal_verification/.probe-saw-suite",
    "tools/formal_verification/aeneas/bin",
    "tools/formal_verification/aeneas/src/bin",
    "tools/formal_verification/alloy",
    "tools/formal_verification/apalache",
    "tools/formal_verification/bin",
    "tools/formal_verification/bitwuzla",
    "tools/formal_verification/cargo",
    "tools/formal_verification/charon/bin",
    "tools/formal_verification/charon/src/bin",
    "tools/formal_verification/creusot/cache",
    "tools/formal_verification/creusot/config",
    "tools/formal_verification/creusot/data",
    "tools/formal_verification/cryptol",
    "tools/formal_verification/cvc5",
    "tools/formal_verification/easycrypt",
    "tools/formal_verification/kani",
    "tools/formal_verification/maude",
    "tools/formal_verification/mir-json/bin",
    "tools/formal_verification/mir-json/rlibs",
    "tools/formal_verification/mir-json/rlibs_real",
    "tools/formal_verification/mir-json/src/rlibs",
    "tools/formal_verification/mir-json/src/rlibs_real",
    "tools/formal_verification/miri",
    "tools/formal_verification/node",
    "tools/formal_verification/opam",
    "tools/formal_verification/prusti",
    "tools/formal_verification/python",
    "tools/formal_verification/rg",
    "tools/formal_verification/rustup",
    "tools/formal_verification/saw",
    "tools/formal_verification/saw-suite",
    "tools/formal_verification/tamarin",
    "tools/formal_verification/tla",
    "tools/formal_verification/verus",
    "wiki/node_modules",
    "website/website_2025-09-30/node_modules",
}
DEFAULT_FILE_SUFFIX_EXCLUDES = {".pyc", ".pyo"}


def normalize_rel_path(value: str) -> str:
    cleaned = value.strip().strip("/")
    if not cleaned:
        return ""
    return Path(cleaned).as_posix()


def load_portable_exclusions(root: Path) -> tuple[set[str], set[str], set[str], set[str], str]:
    top_level = set(DEFAULT_TOP_LEVEL_EXCLUDES)
    recursive = set(DEFAULT_RECURSIVE_DIR_EXCLUDES)
    exact_dirs = {normalize_rel_path(item) for item in DEFAULT_EXACT_DIR_EXCLUDES}
    suffixes = set(DEFAULT_FILE_SUFFIX_EXCLUDES)
    archive_name = ""

    manifest_path = root / ".portable-transfer" / "manifest.json"
    if manifest_path.is_file():
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
        excluded = data.get("excluded_paths", {})
        top_level.update(str(item) for item in excluded.get("top_level", []))
        recursive.update(str(item) for item in excluded.get("recursive_dir_names", []))
        exact_dirs.update(
            normalize_rel_path(str(item))
            for item in excluded.get("exact_dirs", [])
            if str(item).strip()
        )
        suffixes.update(str(item) for item in excluded.get("generated_file_suffixes", []))
        archive_name = str(data.get("archive_name", "")).strip()

    return top_level, recursive, exact_dirs, suffixes, archive_name


def filesystem_tracked_files(root: Path) -> list[str]:
    top_level, recursive, exact_dirs, suffixes, archive_name = load_portable_exclusions(root)
    rows: list[str] = []

    for current_root, dirnames, filenames in os.walk(root):
        current_path = Path(current_root)
        rel_dir = "" if current_path == root else current_path.relative_to(root).as_posix()
        kept_dirs: list[str] = []
        for name in dirnames:
            rel_path = name if not rel_dir else f"{rel_dir}/{name}"
            if (not rel_dir and name in top_level) or name in recursive or rel_path in exact_dirs:
                continue
            kept_dirs.append(name)
        dirnames[:] = kept_dirs

        for name in filenames:
            if archive_name and not rel_dir and name == archive_name:
                continue
            if name.endswith(tuple(suffixes)):
                continue
            rel_path = name if not rel_dir else f"{rel_dir}/{name}"
            rows.append(rel_path)

    return sorted(rows)


def tracked_files(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        return filesystem_tracked_files(root)
    return sorted(
        entry.decode("utf-8", errors="surrogateescape")
        for entry in result.stdout.split(b"\0")
        if entry
    )


def parse_statuses(raw_values: list[str]) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for raw in raw_values:
        gate, sep, status = raw.partition("=")
        if not sep:
            raise ValueError(f"invalid gate status mapping: {raw}")
        statuses[gate] = status
    return statuses


def is_under(path: str, prefix: str) -> bool:
    prefix = prefix.rstrip("/")
    return path == prefix or path.startswith(prefix + "/")


def repo_flags(
    root: Path,
    verification_runtime_root: Path | None,
) -> dict[str, bool]:
    has_loom = subprocess.run(
        ["rg", "-q", r"loom::|cfg\(loom\)|cfg\([^)]*loom", "crates", "tests"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    ).returncode == 0
    verification_roots = [root / "verification"]
    if verification_runtime_root is not None:
        verification_roots.insert(0, verification_runtime_root)

    def any_exists(relative_path: str) -> bool:
        return any((base / relative_path).exists() for base in verification_roots)

    return {
        "has_loom": has_loom,
        "has_hax": any_exists("hax/targets.json"),
        "has_code_to_logic": any_exists("code-to-logic/targets.yaml"),
        "has_constant_time": any((base / "dudect").is_dir() for base in verification_roots),
        "has_security_review": any((base / "security").is_dir() for base in verification_roots),
    }


def classify(path: str, vendor_root: str, statuses: dict[str, str], flags: dict[str, bool]) -> tuple[list[str], list[str]]:
    gates: list[str] = []
    tags: list[str] = []
    suffix = Path(path).suffix.lower()
    is_vendor = is_under(path, vendor_root)
    name = Path(path).name.lower()
    in_crates = path.startswith("crates/")
    adversarial_scanned_path = (
        in_crates
        and (
            (suffix == ".rs" and ("/src/" in path or "/bin/" in path) and not any(marker in path for marker in ("/tests/", "/benches/", "/examples/", "/fuzz/", "/fixture_support/")))
            or path.endswith("/Cargo.toml")
        )
    )

    if path == "README.md" or (is_under(path, "docs") and suffix == ".md") or (is_under(path, "specs") and suffix == ".md") or (is_under(path, ".github/requirements") and suffix == ".md"):
        gates.append("l0-docs")
        tags.append("docs")

    if suffix == ".toml":
        gates.append("l0-docs")
        tags.append("toml")
        if path == "Cargo.toml" or path.endswith("/Cargo.toml"):
            gates.append("l4-supply-chain")
            tags.append("cargo-manifest")

    if path == "Cargo.lock":
        gates.append("l4-supply-chain")
        tags.append("cargo-lock")

    if path.startswith("specs/tla/") and suffix == ".tla":
        gates.extend(["l1-tla", "l1-apalache"])
        tags.append("tla")

    if path.startswith("specs/alloy/") and suffix == ".als":
        gates.append("l1-alloy")
        tags.append("alloy")

    if path.startswith("specs/proverif/") and suffix == ".pv":
        gates.append("l2-proverif")
        tags.append("proverif")

    if path.startswith("specs/tamarin/") and suffix == ".spthy":
        gates.append("l2-tamarin")
        tags.append("tamarin")

    if path in {"specs/crypto/transcript_binding.md", "specs/crypto/proof_objects_schema.yaml"}:
        gates.append("l2-transcript")
        tags.append("crypto-transcript")
        if flags["has_hax"]:
            gates.append("l2-hax")
            tags.append("crypto-extraction")

    if path == "specs/crypto/domain_separation_registry.yaml":
        gates.append("l2-domain")
        tags.append("crypto-domain-registry")

    if path == "specs/crypto/leakage_model.yaml":
        gates.extend(["l2-domain", "l4-constant-time"])
        tags.extend(["crypto-leakage-model", "constant-time-contract"])

    if path.startswith("specs/cryptol/") and suffix == ".cry":
        gates.append("l2-cryptol")
        tags.append("cryptol")

    if path.startswith("specs/saw/") and suffix == ".saw":
        gates.append("l2-saw")
        tags.append("saw")

    if path.startswith("crates/") and suffix == ".rs":
        if is_vendor:
            gates.append("l4-vendor-unsafe" if "l4-vendor-unsafe" in statuses else "l4-unsafe")
            tags.append("vendor-rust")
            gates.append("l4-adversarial-review")
            tags.append("security-brainstorm")
        else:
            gates.append("l3-verify-fast")
            tags.append("rust")
            if path.startswith(
                (
                    "crates/z00z_core/",
                    "crates/z00z_storage/",
                    "crates/z00z_wallets/",
                    "crates/z00z_crypto/",
                    "crates/z00z_runtime/validators/",
                )
            ):
                gates.append("l3-miri")
                tags.append("miri-surface")
            if path.startswith(
                (
                    "crates/z00z_core/tests/generated_kani_",
                    "crates/z00z_storage/tests/generated_kani_",
                    "crates/z00z_wallets/tests/generated_kani_",
                    "crates/z00z_runtime/validators/tests/generated_kani_",
                )
            ):
                gates.append("l3-kani")
                tags.append("kani-harness")
            if flags["has_loom"] and path.startswith(("verification/loom/",)):
                gates.append("l3-loom")
                tags.append("loom-surface")
            if not any(marker in path for marker in ("/tests/", "/benches/", "/examples/", "/fixture_support/", "/test_")):
                gates.append("l2-domain")
                tags.append("domain-scan")
            if adversarial_scanned_path and (
                path.startswith(
                (
                    "crates/z00z_core/",
                    "crates/z00z_storage/",
                    "crates/z00z_wallets/",
                    "crates/z00z_crypto/",
                    "crates/z00z_runtime/validators/",
                    "crates/z00z_networks_rpc/",
                )
                )
                or any(term in path for term in ("checkpoint", "settlement", "voucher", "transcript", "proof", "stealth", "payment_request", "decode", "compact"))
            ):
                gates.append("l4-adversarial-review")
                tags.append("security-brainstorm")

    if path.startswith("verification/kani/"):
        gates.append("l3-kani")
        tags.append("verification-kani")

    if path.startswith("verification/verus/"):
        gates.append("l3-verus")
        tags.append("verification-verus")

    if path.startswith("verification/hax/"):
        gates.append("l2-hax")
        tags.append("verification-hax")

    if path == "verification/model_coverage.yaml":
        gates.extend(["l1-tla", "l1-apalache", "l1-alloy"])
        tags.append("verification-model-coverage")

    if path.startswith("verification/code-to-logic/"):
        tags.append("verification-code-to-logic")
        if path.endswith("targets.yaml"):
            gates.append("l2-refinement-map")
        elif "/llbc/" in path or path.endswith(".llbc"):
            gates.append("l2-charon")
        elif "/aeneas/" in path:
            gates.append("l2-aeneas")
        else:
            if flags["has_code_to_logic"]:
                gates.extend(["l2-cryptol", "l2-saw", "l2-crux-mir"])

    if path.startswith("verification/prusti/"):
        gates.append("l3-prusti")
        tags.append("verification-prusti")

    if path.startswith("verification/dudect/"):
        gates.append("l4-constant-time")
        tags.append("verification-dudect")

    if path.startswith("verification/security/"):
        gates.append("l4-adversarial-review")
        tags.append("verification-security")

    if path.startswith("fuzz/") or "/fuzz/" in path:
        gates.append("l4-fuzz")
        tags.append("fuzz")

    if "/benches/" in path and ("timing" in name or ("constant" in name and "time" in name)):
        gates.append("l4-constant-time")
        tags.append("constant-time-bench")

    if in_crates and not gates:
        if is_vendor:
            gates.append("l4-vendor-unsafe" if "l4-vendor-unsafe" in statuses else "l4-unsafe")
            tags.append("vendor-aux")
        else:
            gates.append("l3-verify-fast")
            tags.append("crate-owned-aux")

        if suffix in {".md", ".txt", ".pdf"} or name in {"readme.md", "license", "changelog.md"} or "/docs/" in path or "/.todo/" in path:
            gates.append("l0-docs")
            tags.append("crate-doc")

        if path.endswith("/Cargo.lock") or name == "cargo.lock":
            gates.append("l4-supply-chain")
            tags.append("cargo-lock")

        if any(segment in path for segment in ("/tests/", "/fixtures/", "/examples/", "/benches/", "/bin/", "/scripts/")):
            tags.append("crate-runtime-aux")

    unique_gates = sorted(dict.fromkeys(gates))
    unique_tags = sorted(dict.fromkeys(tags)) or ["unmapped"]
    return unique_gates, unique_tags


def aggregate_status(gates: list[str], statuses: dict[str, str]) -> str:
    if not gates:
        return "UNMAPPED"

    values = [statuses.get(gate, "UNKNOWN") for gate in gates]
    if any(value == "FAIL" for value in values):
        return "FAIL"
    if any(value == "NEEDS_HUMAN_CRYPTO_REVIEW" for value in values):
        return "NEEDS_HUMAN_CRYPTO_REVIEW"
    if any(value == "UNKNOWN" for value in values):
        return "UNKNOWN"
    if any(value == "SKIPPED" for value in values):
        return "SKIPPED"
    if any(value == "DRY-RUN" for value in values):
        return "DRY-RUN"
    return "PASS"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True)
    parser.add_argument("--vendor-root", required=True)
    parser.add_argument("--manifest-out", required=True)
    parser.add_argument("--summary-out", required=True)
    parser.add_argument("--verification-runtime-root")
    parser.add_argument("--gate-status", action="append", default=[])
    args = parser.parse_args()

    root = Path(args.root).resolve()
    vendor_root = Path(args.vendor_root).resolve()
    verification_runtime_root = (
        Path(args.verification_runtime_root).resolve()
        if args.verification_runtime_root
        else None
    )
    vendor_rel = vendor_root.relative_to(root).as_posix()
    statuses = parse_statuses(args.gate_status)
    flags = repo_flags(root, verification_runtime_root)

    rows: list[tuple[str, str, str, str]] = []
    counts: Counter[str] = Counter()
    crate_counts: Counter[str] = Counter()
    tags_counter: Counter[str] = Counter()
    unmapped_sample: list[str] = []
    crate_unmapped_sample: list[str] = []
    skipped_sample: list[str] = []
    unknown_sample: list[str] = []

    for rel_path in tracked_files(root):
        gates, tags = classify(rel_path, vendor_rel, statuses, flags)
        status = aggregate_status(gates, statuses)
        gate_text = ",".join(gates) if gates else "-"
        tag_text = ",".join(tags)
        rows.append((status, rel_path, gate_text, tag_text))
        counts[status] += 1
        if rel_path.startswith("crates/"):
            crate_counts[status] += 1
        for tag in tags:
            tags_counter[tag] += 1
        if status == "UNMAPPED" and len(unmapped_sample) < 50:
            unmapped_sample.append(rel_path)
            if rel_path.startswith("crates/") and len(crate_unmapped_sample) < 50:
                crate_unmapped_sample.append(rel_path)
        if status == "SKIPPED" and len(skipped_sample) < 50:
            skipped_sample.append(rel_path)
        if status == "UNKNOWN" and len(unknown_sample) < 50:
            unknown_sample.append(rel_path)

    manifest_path = Path(args.manifest_out)
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    with manifest_path.open("w", encoding="utf-8") as handle:
        handle.write("status\tpath\tgates\ttags\n")
        for row in rows:
            handle.write("\t".join(row) + "\n")

    summary = {
        "tracked_files": len(rows),
        "status_counts": dict(sorted(counts.items())),
        "crate_status_counts": dict(sorted(crate_counts.items())),
        "tag_counts": dict(sorted(tags_counter.items())),
        "samples": {
            "unmapped": unmapped_sample,
            "crate_unmapped": crate_unmapped_sample,
            "skipped": skipped_sample,
            "unknown": unknown_sample,
        },
    }
    summary_path = Path(args.summary_out)
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        print(error, file=sys.stderr)
        raise SystemExit(error.returncode)
    except ValueError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
