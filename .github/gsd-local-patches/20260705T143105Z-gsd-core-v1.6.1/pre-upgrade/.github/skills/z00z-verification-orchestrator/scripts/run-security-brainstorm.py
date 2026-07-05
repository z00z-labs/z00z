#!/usr/bin/env python3
"""Generate adversarial security scenarios for Z00Z from code evidence and .github prompt sources."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from collections import Counter, defaultdict
from pathlib import Path


VENDOR_ROOT = "crates/z00z_crypto/tari"
PROMPT_FILE_SUFFIXES = {".md", ".txt", ".yaml", ".yml", ".py", ".sh", ".toml", ".json"}
SECURITY_PROMPT_TERMS = {
    "adversarial-review": ("adversarial", "red-team", "break", "jailbreak", "abuse", "bypass"),
    "attack-surface": ("attack surface", "threat", "exploit", "misuse", "replay", "forg", "tamper"),
    "crypto-review": ("crypto", "cryptographic", "transcript", "domain separation", "proof", "signature", "stealth"),
    "fuzz-parser": ("fuzz", "parser", "decode", "deserialize", "compact", "untrusted input"),
    "unsafe-ub": ("unsafe", "ub", "miri", "undefined behavior"),
    "supply-chain": ("supply-chain", "dependency", "audit", "semver"),
}
DEFAULT_FAMILIES = [
    {
        "id": "checkpoint-lineage",
        "title": "Checkpoint lineage and delta integrity",
        "severity": "high",
        "categories": ["adversarial-review", "attack-surface", "crypto-review"],
        "probe_questions": [
            "Can one crate accept a root transition that another crate later rejects as stale or already spent?",
            "Can spent and created deltas stay locally balanced while cross-crate lineage still forks?",
        ],
    },
    {
        "id": "payment-request-replay",
        "title": "PaymentRequest replay and compact-request rebinding",
        "severity": "high",
        "categories": ["adversarial-review", "attack-surface", "crypto-review", "fuzz-parser"],
        "probe_questions": [
            "Can the same compact request be replayed across epochs, chain ids, or validator contexts?",
            "Can a request stay signature-valid while being rebound to a different domain or receiver context?",
        ],
    },
    {
        "id": "voucher-rights-transfer",
        "title": "Voucher acceptance and rights-transfer bypass",
        "severity": "high",
        "categories": ["adversarial-review", "attack-surface", "crypto-review"],
        "probe_questions": [
            "Can a non-transferable right become redeemable through wrapper, cache, or voucher indirection?",
            "Can refusal and acceptance states diverge across crates for the same voucher lineage?",
        ],
    },
    {
        "id": "stealth-inbox-delivery",
        "title": "Stealth delivery and inbox notification confusion",
        "severity": "high",
        "categories": ["adversarial-review", "attack-surface", "crypto-review"],
        "probe_questions": [
            "Can a notification be rebound to a different receiver while preserving a plausible owner tag?",
            "Can delivery and observation paths disagree on what metadata authenticates a stealth notice?",
        ],
    },
    {
        "id": "transcript-domain-binding",
        "title": "Transcript and domain-separation drift",
        "severity": "high",
        "categories": ["adversarial-review", "crypto-review"],
        "probe_questions": [
            "Can two proof families share a challenge transcript layout while believing they are domain-separated?",
            "Can version or serialization drift keep one crate's challenge binding compatible with another crate's older meaning?",
        ],
    },
    {
        "id": "compact-wire-parsers",
        "title": "Compact-wire and decoder abuse surface",
        "severity": "medium",
        "categories": ["adversarial-review", "attack-surface", "fuzz-parser", "unsafe-ub"],
        "probe_questions": [
            "Can malformed bytes trigger panic-only rejection instead of structured failure?",
            "Can parser disagreement across crates create type confusion or silent truncation?",
        ],
    },
]
SEVERITY_RANK = {"high": 3, "medium": 2, "low": 1}
CLASS_RANK = {"cross-crate": 4, "crate": 3, "module": 2, "file": 1}
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


def repo_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


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


def git_ls_files(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        return filesystem_tracked_files(root)
    return sorted(
        item.decode("utf-8", errors="surrogateescape")
        for item in result.stdout.split(b"\0")
        if item
    )


def cargo_package_roots(root: Path) -> list[str]:
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    data = json.loads(result.stdout)
    roots = []
    for package in data.get("packages", []):
        manifest = Path(package["manifest_path"]).resolve()
        roots.append(manifest.parent.relative_to(root).as_posix())
    return sorted(set(roots), key=lambda item: (-len(item), item))


def classify_security_prompt_text(text: str) -> list[str]:
    lower = text.lower()
    categories = []
    for category, terms in SECURITY_PROMPT_TERMS.items():
        if any(term in lower for term in terms):
            categories.append(category)
    return categories


def security_prompt_excerpts(text: str, limit: int = 8) -> list[str]:
    excerpts = []
    for raw_line in text.splitlines():
        line = " ".join(raw_line.strip().split())
        if len(line) < 16:
            continue
        lower = line.lower()
        if not any(term in lower for terms in SECURITY_PROMPT_TERMS.values() for term in terms):
            continue
        excerpts.append(line[:220])
        if len(excerpts) >= limit:
            break
    return excerpts


def build_prompt_corpus(root: Path) -> dict:
    entries = []
    scanned_files = 0
    relevant_files = 0
    github_root = root / ".github"
    prompt_files: list[Path] = []
    if github_root.exists():
        for path in sorted(github_root.rglob("*")):
            if not path.is_file():
                continue
            if "__pycache__" in path.parts:
                continue
            if path.suffix.lower() not in PROMPT_FILE_SUFFIXES:
                continue
            prompt_files.append(path)

    for path in prompt_files:
        if not path.is_file():
            continue
        scanned_files += 1
        text = repo_text(path)
        categories = classify_security_prompt_text(text)
        if not categories:
            continue
        relevant_files += 1
        excerpts = security_prompt_excerpts(text)
        rel_path = path.relative_to(root).as_posix()
        if "/scripts/" in rel_path:
            kind = "script"
        elif "/templates/" in rel_path:
            kind = "template"
        elif rel_path.startswith(".github/skills/"):
            kind = "skill"
        elif rel_path.startswith(".github/agents/"):
            kind = "agent"
        elif rel_path.startswith(".github/requirements/"):
            kind = "requirement"
        elif rel_path.startswith(".github/instructions/"):
            kind = "instruction"
        elif rel_path.startswith(".github/prompts/"):
            kind = "prompt"
        elif rel_path.startswith(".github/gsd-core/workflows/"):
            kind = "workflow"
        elif rel_path == ".github/copilot-instructions.md":
            kind = "instructions"
        else:
            kind = "other"
        entries.append(
            {
                "path": rel_path,
                "kind": kind,
                "categories": categories,
                "score": len(categories) + len(excerpts),
                "excerpt_count": len(excerpts),
                "excerpts": excerpts,
            }
        )
    return {
        "scanned_files": scanned_files,
        "relevant_files": relevant_files,
        "sources": sorted(entries, key=lambda item: (-int(item["score"]), str(item["path"]))),
    }


def read_json_if_exists(path: Path) -> dict | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def resolve_verification_artifact(
    preferred_path: Path, root: Path, name: str
) -> tuple[dict | None, Path | None]:
    preferred = read_json_if_exists(preferred_path)
    if preferred is not None:
        return preferred, preferred_path

    reports_root = root / "reports"
    if not reports_root.exists():
        return None, None
    candidates = sorted(
        reports_root.glob(f"z00z-verification-orchestrator-*/verification*/security/{name}")
    )
    for path in reversed(candidates):
        data = read_json_if_exists(path)
        if data is not None:
            return data, path
    return None, None


def scoped_code_files(root: Path, scope_kind: str, target_root_rel: str) -> list[str]:
    selected = []
    for rel_path in git_ls_files(root):
        if scope_kind == "crate" and target_root_rel and not (rel_path == target_root_rel or rel_path.startswith(target_root_rel + "/")):
            continue
        if rel_path.startswith("crates/") and rel_path.endswith(".rs") and (
            "/src/" in rel_path or "/bin/" in rel_path
        ) and not any(marker in rel_path for marker in ("/tests/", "/benches/", "/examples/", "/fuzz/", "/fixture_support/")):
            selected.append(rel_path)
        elif rel_path.startswith("crates/") and rel_path.endswith("/Cargo.toml"):
            selected.append(rel_path)
        elif scope_kind == "project" and rel_path in {"Cargo.toml", "Cargo.lock"}:
            selected.append(rel_path)
    return selected


def owning_package(rel_path: str, package_roots: list[str]) -> str:
    for package_root in package_roots:
        if rel_path == package_root or rel_path.startswith(package_root + "/"):
            return package_root
    return "."


def module_bucket(rel_path: str) -> str:
    path = Path(rel_path)
    parent = path.parent.as_posix()
    return parent or "."


def ownership_for_files(paths: list[str]) -> str:
    if not paths:
        return "project-owned"
    vendor_flags = [item == VENDOR_ROOT or item.startswith(VENDOR_ROOT + "/") for item in paths]
    if all(vendor_flags):
        return "protected-vendor"
    if any(vendor_flags):
        return "mixed"
    return "project-owned"


def critical_surface(path_lower: str) -> bool:
    return path_lower.startswith(
        (
            "crates/z00z_crypto/",
            "crates/z00z_wallets/",
            "crates/z00z_storage/",
            "crates/z00z_core/",
            "crates/z00z_runtime/validators/",
            "crates/z00z_networks_rpc/",
            f"{VENDOR_ROOT}/",
        )
    )


def parser_surface(path_lower: str, text_lower: str) -> bool:
    return any(term in path_lower or term in text_lower for term in ("decode", "deserialize", "compact", "from_bytes", "codec", "serde"))


def validator_surface(path_lower: str) -> bool:
    return "validators" in path_lower or "checkpoint" in path_lower or "settlement" in path_lower


def crypto_surface(path_lower: str) -> bool:
    return any(term in path_lower for term in ("z00z_crypto", "wallets", "proof", "transcript", "stealth", "signature"))


def line_hits(text: str, patterns: list[str], limit: int = 3) -> list[str]:
    hits = []
    regexes = [re.compile(pattern, re.IGNORECASE) for pattern in patterns]
    for line_no, raw in enumerate(text.splitlines(), start=1):
        for regex in regexes:
            if regex.search(raw):
                hits.append(f"{line_no}: {' '.join(raw.strip().split())[:220]}")
                break
        if len(hits) >= limit:
            break
    return hits


def logging_secret_hits(text: str, limit: int = 3) -> list[str]:
    hits = []
    for line_no, raw in enumerate(text.splitlines(), start=1):
        stripped = raw.strip()
        if stripped.startswith("//"):
            continue
        lower = stripped.lower()
        if not any(token in lower for token in ("tracing::", "log::", "println!", "eprintln!")):
            continue
        if not any(token in lower for token in ("secret", "seed", "private", "witness", "proof", "nonce", "key")):
            continue
        hits.append(f"{line_no}: {' '.join(stripped.split())[:220]}")
        if len(hits) >= limit:
            break
    return hits


def unsafe_signal_hits(text: str, limit: int = 3) -> list[str]:
    hits = []
    for line_no, raw in enumerate(text.splitlines(), start=1):
        stripped = raw.strip()
        if not stripped or stripped.startswith("//"):
            continue
        if "unsafe_code" in stripped:
            continue
        if "unsafe {" not in stripped and not re.search(r"\bunsafe\b", stripped):
            continue
        hits.append(f"{line_no}: {' '.join(stripped.split())[:220]}")
        if len(hits) >= limit:
            break
    return hits


def select_prompt_sources(prompt_corpus: dict, categories: list[str], limit: int = 5) -> list[str]:
    sources = []
    wanted = set(categories)
    for source in prompt_corpus.get("sources", []):
        if wanted.intersection(source.get("categories", [])):
            sources.append(str(source["path"]))
        if len(sources) >= limit:
            break
    if sources:
        return sources
    return [str(source["path"]) for source in prompt_corpus.get("sources", [])[:limit]]


def add_finding(store: dict, finding: dict) -> None:
    key = (
        finding["id"],
        finding["class"],
        tuple(finding["evidence_files"]),
    )
    if key in store:
        return
    store[key] = finding


def file_findings(root: Path, files: list[str], package_roots: list[str], prompt_corpus: dict) -> list[dict]:
    store: dict[tuple, dict] = {}

    for rel_path in files:
        abs_path = root / rel_path
        if not abs_path.is_file():
            continue
        text = repo_text(abs_path)
        text_lower = text.lower()
        path_lower = rel_path.lower()
        package = owning_package(rel_path, package_roots)
        module = module_bucket(rel_path)
        ownership = ownership_for_files([rel_path])

        def emit(
            rule_id: str,
            title: str,
            severity: str,
            categories: list[str],
            hypothesis: str,
            probes: list[str],
            patterns: list[str],
            evidence_override: list[str] | None = None,
        ) -> None:
            add_finding(
                store,
                {
                    "id": rule_id,
                    "class": "file",
                    "severity": severity,
                    "title": title,
                    "ownership": ownership,
                    "package": package,
                    "module": module,
                    "evidence_files": [rel_path],
                    "evidence_lines": evidence_override if evidence_override is not None else line_hits(text, patterns),
                    "prompt_sources": select_prompt_sources(prompt_corpus, categories),
                    "categories": categories,
                    "hypothesis": hypothesis,
                    "probes": probes,
                },
            )

        if "/src/" in path_lower and parser_surface(path_lower, text_lower) and re.search(r"\b(?:unwrap|expect)\s*\(|panic!\s*\(|assert!\s*\(", text):
            emit(
                "panic-on-untrusted-input",
                f"Panic-capable parser or decode path in {rel_path}",
                "medium",
                ["adversarial-review", "attack-surface", "fuzz-parser"],
                "Attacker-controlled bytes may push rejection into panic-only paths, yielding denial of service or parser-oracle behavior.",
                [
                    "Feed undersized, oversized, and version-skewed compact payloads and compare panic vs structured error behavior.",
                    "Mutate authenticated and unauthenticated fields independently to see whether panics leak semantic boundary information.",
                ],
                [r"\b(?:unwrap|expect)\s*\(", r"panic!\s*\(", r"assert!\s*\("],
            )

        if "/src/" in path_lower and critical_surface(path_lower) and re.search(r"\bTODO\b|todo!\s*\(|unimplemented!\s*\(|FIXME", text, re.IGNORECASE):
            emit(
                "todo-in-critical-surface",
                f"Incomplete critical-path logic marker in {rel_path}",
                "medium",
                ["adversarial-review", "attack-surface"],
                "Unfinished logic inside a critical path can degrade into an implicit bypass under edge inputs, refactors, or partial feature rollouts.",
                [
                    "Probe every TODO-guarded branch with malformed and cross-version inputs.",
                    "Check whether placeholder logic changes accept/reject semantics between crates or build features.",
                ],
                [r"\bTODO\b", r"todo!\s*\(", r"unimplemented!\s*\(", r"FIXME"],
            )

        unsafe_hits = unsafe_signal_hits(text)
        if "/src/" in path_lower and critical_surface(path_lower) and unsafe_hits:
            emit(
                "unsafe-in-critical-surface",
                f"Unsafe code in critical security surface {rel_path}",
                "high",
                ["adversarial-review", "unsafe-ub", "crypto-review"],
                "Unsafe code at a crypto, state, or validator boundary can invalidate assumptions behind higher-level proofs and allow memory- or layout-driven invariant breaks.",
                [
                    "Trace whether attacker-influenced bytes can reach this unsafe region with partial validation.",
                    "Cross-check aliasing, initialization, and bounds assumptions against neighboring crates that trust this type or buffer shape.",
                ],
                [r"\bunsafe\b"],
                evidence_override=unsafe_hits,
            )

        if critical_surface(path_lower) and any(token in text_lower for token in ("skip_", "disable_", "insecure", "bypass", "unchecked", "allow_invalid", "allow_unverified")):
            emit(
                "config-bypass-knob",
                f"Security-sensitive bypass knob in {rel_path}",
                "medium",
                ["adversarial-review", "attack-surface"],
                "A skip, disable, unchecked, or insecure control can become a jailbreak knob if configuration drift, wrappers, or test-only logic expose it unexpectedly.",
                [
                    "Search callers and configs for non-test paths that can toggle the bypass.",
                    "Check whether feature composition or environment wiring can enable the knob in production builds.",
                ],
                [r"skip_", r"disable_", r"insecure", r"bypass", r"unchecked", r"allow_invalid", r"allow_unverified"],
            )

        if "/src/" in path_lower and validator_surface(path_lower) and any(token in text_lower for token in ("systemtime", "instant::now", "utc::now", "thread_rng", "rand::random", "osrng")):
            emit(
                "nondeterministic-validation-source",
                f"Nondeterministic source in validator-like surface {rel_path}",
                "high",
                ["adversarial-review", "attack-surface"],
                "Time- or randomness-dependent validation can split accept/reject semantics across nodes and create consensus or replay edge cases.",
                [
                    "Force repeated validation under different clocks, seeds, or scheduling and compare accept/reject outputs.",
                    "Check whether nondeterministic values influence only logging or actually feed verification decisions.",
                ],
                [r"SystemTime", r"Instant::now", r"Utc::now", r"thread_rng", r"rand::random", r"OsRng"],
            )

        secret_hits = logging_secret_hits(text)
        if "/src/" in path_lower and secret_hits:
            emit(
                "secret-or-proof-logging",
                f"Potential secret or proof-adjacent logging in {rel_path}",
                "high",
                ["adversarial-review", "crypto-review"],
                "Observability output may leak secret, nonce, witness, or proof-adjacent material into logs and downstream telemetry.",
                [
                    "Inspect emitted log values under realistic error paths and trace-level configs.",
                    "Verify whether sanitized wrappers or redaction are applied consistently across crates.",
                ],
                [r"tracing::", r"log::", r"println!", r"eprintln!"],
                evidence_override=secret_hits,
            )

        if "/src/" in path_lower and crypto_surface(path_lower) and any(token in text_lower for token in ("transcript", "challenge", "hash")) and any(token in text_lower for token in ("extend_from_slice", "format!(", "concat!(", ".push(", "push(")) and "hash_domain!" not in text:
            emit(
                "raw-transcript-packing",
                f"Raw transcript or challenge packing in {rel_path}",
                "medium",
                ["adversarial-review", "crypto-review"],
                "Raw byte packing without an explicit domain-binding primitive raises the risk of transcript collision, re-interpretation, or proof-family rebinding.",
                [
                    "Compare transcript construction across neighboring proof families for identical field orderings with different semantic meaning.",
                    "Inject ambiguous byte boundaries and version skew to test whether two logical statements can hash to the same challenge input.",
                ],
                [r"transcript", r"challenge", r"hash", r"extend_from_slice", r"format!\(", r"concat!\(", r"\.push\(", r"push\("],
            )

        if "/src/" in path_lower and critical_surface(path_lower) and "cfg(feature" in text_lower and re.search(r"(verify|validate|signature|proof|checkpoint)", text, re.IGNORECASE):
            emit(
                "feature-gated-security-logic",
                f"Feature-gated security logic in {rel_path}",
                "medium",
                ["adversarial-review", "attack-surface", "crypto-review"],
                "Feature composition may create build-dependent security semantics where one artifact verifies a condition that another build omits.",
                [
                    "Enumerate feature sets and compare whether the same artifact is accepted or rejected differently.",
                    "Check whether integration tests cover the exact production feature matrix.",
                ],
                [r"cfg\(feature", r"verify", r"validate", r"signature", r"proof", r"checkpoint"],
            )

        if "/src/" in path_lower and parser_surface(path_lower, text_lower) and re.search(r"(bincode::deserialize|serde_json::from_slice|postcard::from_bytes|rmp_serde::from_)", text) and not re.search(r"(limit|max_len|max_size|checked|bounded)", text, re.IGNORECASE):
            emit(
                "deserialize-without-explicit-limits",
                f"Deserializer without explicit size guard in {rel_path}",
                "medium",
                ["adversarial-review", "fuzz-parser", "attack-surface"],
                "A deserializer without visible size or structure bounding may admit oversized payloads, deep nesting, or ambiguous decode behavior.",
                [
                    "Feed oversized and recursively nested payloads to test memory and CPU amplification.",
                    "Compare accepted payload shape across crates that deserialize the same logical object.",
                ],
                [r"bincode::deserialize", r"serde_json::from_slice", r"postcard::from_bytes", r"rmp_serde::from_"],
            )

    return list(store.values())


def aggregate_findings(file_level: list[dict], family_registry: dict, prompt_corpus: dict, scope_kind: str) -> list[dict]:
    store: dict[tuple, dict] = {}
    for finding in file_level:
        add_finding(store, finding)

    module_groups: dict[str, list[dict]] = defaultdict(list)
    crate_groups: dict[str, list[dict]] = defaultdict(list)
    for finding in file_level:
        module_groups[finding["module"]].append(finding)
        crate_groups[finding["package"]].append(finding)

    for module, items in module_groups.items():
        if module == ".":
            continue
        high_count = sum(1 for item in items if item["severity"] == "high")
        unique_rules = {item["id"] for item in items}
        evidence_files = sorted({path for item in items for path in item["evidence_files"]})
        if not any(critical_surface(path.lower()) for path in evidence_files):
            continue
        if len(items) < 2 or high_count == 0 or len(unique_rules) < 2:
            continue
        severity = "high" if high_count else "medium"
        categories = sorted({category for item in items for category in item["categories"]})
        add_finding(
            store,
            {
                "id": f"module:{module}",
                "class": "module",
                "severity": severity,
                "title": f"Concentrated adversarial surface in module {module}",
                "ownership": ownership_for_files(evidence_files),
                "package": items[0]["package"],
                "module": module,
                "evidence_files": evidence_files[:8],
                "evidence_lines": [],
                "prompt_sources": select_prompt_sources(prompt_corpus, categories),
                "categories": categories,
                "hypothesis": "Multiple file-level signals cluster in this module, so per-file local checks may miss a module-wide invariant break or bypass path.",
                "probes": [
                    "Exercise module entry points with cross-version, replayed, and partially authenticated inputs.",
                    "Compare happy-path and reject-path behavior across neighboring files in the same module.",
                ],
            },
        )

    for package, items in crate_groups.items():
        if package == ".":
            continue
        high_count = sum(1 for item in items if item["severity"] == "high")
        unique_rules = {item["id"] for item in items}
        evidence_files = sorted({path for item in items for path in item["evidence_files"]})
        if not any(critical_surface(path.lower()) for path in evidence_files):
            continue
        if len(items) < 3 or high_count < 2 or len(unique_rules) < 2:
            continue
        categories = sorted({category for item in items for category in item["categories"]})
        add_finding(
            store,
            {
                "id": f"crate:{package}",
                "class": "crate",
                "severity": "high" if high_count else "medium",
                "title": f"Crate-level concentration of adversarial signals in {package}",
                "ownership": ownership_for_files(evidence_files),
                "package": package,
                "module": ".",
                "evidence_files": evidence_files[:12],
                "evidence_lines": [],
                "prompt_sources": select_prompt_sources(prompt_corpus, categories),
                "categories": categories,
                "hypothesis": "The same crate owns multiple suspicious parser, crypto, config, or validation surfaces, so local proofs may still leave a crate-level bypass or semantic split.",
                "probes": [
                    "Run end-to-end adversarial flows that start in one file and terminate in another file inside the same crate.",
                    "Check whether crate-level public APIs normalize or amplify inconsistent lower-level rejection rules.",
                ],
            },
        )

    families = family_registry.get("families", DEFAULT_FAMILIES)
    for family in families:
        source_files = list(family.get("source_files", []))
        crates = sorted(set(family.get("crates", [])))
        if len(source_files) < 2 or len(crates) < 2:
            continue
        add_finding(
            store,
            {
                "id": f"family:{family['id']}",
                "class": "cross-crate",
                "severity": str(family.get("severity", "medium")),
                "title": str(family["title"]),
                "ownership": str(family.get("ownership", ownership_for_files(source_files))),
                "package": crates[0] if crates else ".",
                "module": ".",
                "evidence_files": source_files[:12],
                "evidence_lines": [],
                "prompt_sources": select_prompt_sources(prompt_corpus, list(family.get("categories", []))),
                "categories": list(family.get("categories", [])),
                "hypothesis": "Individual crates can satisfy their own local checks while still disagreeing on the security meaning of the same protocol object across the boundary.",
                "probes": list(family.get("probe_questions", [])),
            },
        )

    findings = list(store.values())
    findings.sort(
        key=lambda item: (
            -SEVERITY_RANK.get(item["severity"], 0),
            -CLASS_RANK.get(item["class"], 0),
            item["title"],
        )
    )
    return findings


def summarize(
    findings: list[dict],
    prompt_corpus: dict,
    files_scanned: int,
    family_registry: dict,
    report_path: Path,
    json_path: Path,
    prompt_corpus_path: Path,
    family_registry_path: Path,
) -> dict:
    severity_counts = Counter(item["severity"] for item in findings)
    class_counts = Counter(item["class"] for item in findings)
    ownership_counts = Counter(item["ownership"] for item in findings)
    prompt_kind_counts = Counter(item.get("kind", "other") for item in prompt_corpus.get("sources", []))
    top_findings = []
    selected_findings = [item for item in findings if item["severity"] == "high"]
    if not selected_findings:
        selected_findings = findings[:10]
    for item in selected_findings:
        top_findings.append(
            {
                "id": item["id"],
                "severity": item["severity"],
                "class": item["class"],
                "title": item["title"],
                "ownership": item["ownership"],
                "evidence_files": item["evidence_files"][:5],
            }
        )
    top_prompt_sources = []
    for source in prompt_corpus.get("sources", [])[:10]:
        top_prompt_sources.append(
            {
                "path": source.get("path", ""),
                "kind": source.get("kind", "other"),
                "categories": source.get("categories", []),
                "excerpt_count": source.get("excerpt_count", 0),
            }
        )
    return {
        "files_scanned": files_scanned,
        "prompt_sources_scanned": prompt_corpus.get("scanned_files", 0),
        "prompt_sources_relevant": prompt_corpus.get("relevant_files", 0),
        "prompt_sources_by_kind": dict(sorted(prompt_kind_counts.items())),
        "prompt_corpus_path": prompt_corpus_path.as_posix(),
        "family_registry_path": family_registry_path.as_posix(),
        "top_prompt_sources": top_prompt_sources,
        "family_count": len(family_registry.get("families", [])),
        "findings_total": len(findings),
        "high_risk_count": severity_counts.get("high", 0),
        "medium_risk_count": severity_counts.get("medium", 0),
        "low_risk_count": severity_counts.get("low", 0),
        "findings_by_severity": dict(sorted(severity_counts.items())),
        "findings_by_class": dict(sorted(class_counts.items())),
        "ownership_counts": dict(sorted(ownership_counts.items())),
        "top_findings": top_findings,
        "report_markdown": report_path.as_posix(),
        "report_json": json_path.as_posix(),
    }


def write_markdown(report_path: Path, scope_kind: str, target_root_rel: str, summary: dict, findings: list[dict]) -> None:
    report_path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Z00Z Adversarial Security Brainstorming",
        "",
        f"- Scope: `{scope_kind}`",
        f"- Target: `{target_root_rel or 'project'}`",
        f"- Files scanned: `{summary['files_scanned']}`",
        f"- Prompt sources scanned under `.github/`: `{summary['prompt_sources_scanned']}`",
        f"- Security-relevant prompt sources: `{summary['prompt_sources_relevant']}`",
        f"- Findings: `{summary['findings_total']}` total; high `{summary['high_risk_count']}`, medium `{summary['medium_risk_count']}`, low `{summary['low_risk_count']}`",
        "",
        "These are adversarial hypotheses backed by code signals and prompt-corpus cues. They are not formal proofs of exploitability.",
        "",
    ]

    by_severity: dict[str, list[dict]] = defaultdict(list)
    for finding in findings:
        by_severity[finding["severity"]].append(finding)

    for severity in ("high", "medium", "low"):
        items = by_severity.get(severity, [])
        if not items:
            continue
        lines.extend([f"## {severity.title()} Findings", ""])
        for item in items:
            lines.append(f"### {item['class']} :: {item['title']}")
            lines.append(f"- Ownership: `{item['ownership']}`")
            if item["package"] and item["package"] != ".":
                lines.append(f"- Package: `{item['package']}`")
            if item["module"] and item["module"] not in {".", item["package"]}:
                lines.append(f"- Module: `{item['module']}`")
            lines.append(f"- Evidence files: {', '.join(f'`{path}`' for path in item['evidence_files'][:8])}")
            if item["evidence_lines"]:
                lines.append("- Code signals:")
                for line in item["evidence_lines"]:
                    lines.append(f"  - `{line}`")
            if item["prompt_sources"]:
                lines.append(f"- Prompt sources: {', '.join(f'`{path}`' for path in item['prompt_sources'])}")
            lines.append(f"- Hypothesis: {item['hypothesis']}")
            if item["probes"]:
                lines.append("- Adversarial probes:")
                for probe in item["probes"]:
                    lines.append(f"  - {probe}")
            lines.append("")

    report_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True)
    parser.add_argument("--scope-kind", required=True, choices=["project", "crate"])
    parser.add_argument("--target-root-rel", default="")
    parser.add_argument("--verification-root", required=True)
    parser.add_argument("--summary-out", required=True)
    parser.add_argument("--report-out", required=True)
    args = parser.parse_args()

    root = Path(args.root).resolve()
    verification_root = Path(args.verification_root).resolve()
    summary_path = Path(args.summary_out).resolve()
    report_path = Path(args.report_out).resolve()

    prompt_corpus_path = verification_root / "security" / "prompt_corpus.json"
    family_registry_path = verification_root / "security" / "attack_surface_registry.json"
    prompt_corpus, prompt_corpus_source = resolve_verification_artifact(
        prompt_corpus_path, root, "prompt_corpus.json"
    )
    if prompt_corpus is None:
        prompt_corpus = build_prompt_corpus(root)
        prompt_corpus_source = prompt_corpus_path

    family_registry, family_registry_source = resolve_verification_artifact(
        family_registry_path, root, "attack_surface_registry.json"
    )
    if family_registry is None:
        family_registry = {"families": DEFAULT_FAMILIES}
        family_registry_source = family_registry_path

    package_roots = cargo_package_roots(root)
    files = scoped_code_files(root, args.scope_kind, args.target_root_rel)
    file_level = file_findings(root, files, package_roots, prompt_corpus)
    findings = aggregate_findings(file_level, family_registry, prompt_corpus, args.scope_kind)
    summary = summarize(
        findings,
        prompt_corpus,
        len(files),
        family_registry,
        report_path,
        summary_path,
        prompt_corpus_source or prompt_corpus_path,
        family_registry_source or family_registry_path,
    )

    summary_path.parent.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_markdown(report_path, args.scope_kind, args.target_root_rel, summary, findings)

    print(f"[z00z-l4:adversarial] scanned code files: {summary['files_scanned']}")
    print(f"[z00z-l4:adversarial] prompt sources: {summary['prompt_sources_scanned']} scanned, {summary['prompt_sources_relevant']} security-relevant")
    print(
        "[z00z-l4:adversarial] findings: "
        f"{summary['findings_total']} total, high={summary['high_risk_count']}, "
        f"medium={summary['medium_risk_count']}, low={summary['low_risk_count']}"
    )
    print(f"[z00z-l4:adversarial] report: {report_path}")
    if summary["high_risk_count"] > 0:
        print(
            "NEEDS_HUMAN_CRYPTO_REVIEW: "
            f"{summary['high_risk_count']} high-risk adversarial security scenarios need human crypto review"
        )
    elif summary["medium_risk_count"] > 0:
        print(
            "UNKNOWN: "
            f"{summary['medium_risk_count']} medium-risk adversarial security scenarios were generated heuristically"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
