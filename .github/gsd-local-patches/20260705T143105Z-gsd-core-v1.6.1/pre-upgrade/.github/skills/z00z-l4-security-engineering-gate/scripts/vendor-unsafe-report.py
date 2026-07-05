#!/usr/bin/env python3

"""Generate a fact-based unsafe report for vendored Rust code."""

from __future__ import annotations

import argparse
import collections
import datetime as dt
import os
import pathlib
import re
from dataclasses import dataclass


@dataclass(frozen=True)
class Finding:
    path: pathlib.Path
    line_number: int
    evidence: str
    kind: str
    package: str
    context: str
    decision: str
    rationale: str
    verified_facts: tuple[str, ...]


def parse_args() -> argparse.Namespace:
    report_stamp = os.environ.get(
        "Z00Z_REPORT_TIMESTAMP",
        dt.datetime.now(dt.UTC).strftime("%Y%m%d-%H%M%S"),
    )
    run_root = os.environ.get("Z00Z_VERIFICATION_RUN_ROOT", f"reports/z00z-verification-orchestrator-{report_stamp}")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vendor-root", default="crates/z00z_crypto/tari", help="Vendored Rust source root.")
    parser.add_argument(
        "--output",
        default=os.environ.get("Z00Z_VENDOR_UNSAFE_REPORT", f"{run_root}/vendor/vendor-unsafe.md"),
        help="Markdown report path.",
    )
    return parser.parse_args()


def read_package_name(manifest: pathlib.Path) -> str | None:
    if not manifest.exists():
        return None
    for line in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
        match = re.match(r'\s*name\s*=\s*"([^"]+)"', line)
        if match:
            return match.group(1)
    return None


def build_package_index(vendor_root: pathlib.Path) -> list[tuple[pathlib.Path, str]]:
    packages: list[tuple[pathlib.Path, str]] = []
    for manifest in sorted(vendor_root.rglob("Cargo.toml")):
        package = read_package_name(manifest)
        if package:
            packages.append((manifest.parent.resolve(), package))
    packages.sort(key=lambda item: len(str(item[0])), reverse=True)
    return packages


def package_for(path: pathlib.Path, package_index: list[tuple[pathlib.Path, str]]) -> str:
    resolved = path.resolve()
    for root, package in package_index:
        try:
            resolved.relative_to(root)
        except ValueError:
            continue
        return package
    return "unknown-vendor-package"


def line_window(lines: list[str], index: int, radius: int = 4) -> str:
    start = max(0, index - radius)
    end = min(len(lines), index + radius + 1)
    return "\n".join(lines[start:end])


def nearest_test_marker(lines: list[str], index: int) -> bool:
    start = max(0, index - 12)
    window = "\n".join(lines[start : index + 1])
    return "#[test]" in window or "#[cfg(test)]" in window or "mod tests" in window


def nearest_match(lines: list[str], index: int, pattern: str, lookback: int = 20) -> tuple[int, str] | None:
    start = max(0, index - lookback)
    regex = re.compile(pattern)
    for cursor in range(index, start - 1, -1):
        if regex.search(lines[cursor]):
            return cursor + 1, lines[cursor].strip()
    return None


def nearby_matches(lines: list[str], index: int, pattern: str, radius: int = 4) -> list[tuple[int, str]]:
    start = max(0, index - radius)
    end = min(len(lines), index + radius + 1)
    regex = re.compile(pattern)
    matches: list[tuple[int, str]] = []
    for cursor in range(start, end):
        if regex.search(lines[cursor]):
            matches.append((cursor + 1, lines[cursor].strip()))
    return matches


def collect_verified_facts(path: pathlib.Path, lines: list[str], index: int, vendor_root: pathlib.Path, package: str) -> tuple[str, ...]:
    facts: list[str] = []
    try:
        path.resolve().relative_to(vendor_root.resolve())
        facts.append(f"File is under configured vendor root `{vendor_root.as_posix()}` and package `{package}`.")
    except ValueError:
        facts.append(f"File is outside configured vendor root `{vendor_root.as_posix()}`; check vendor-root configuration.")

    test_marker = nearest_match(lines, index, r"#\[(test|cfg\(test\))\]", lookback=30)
    if test_marker:
        facts.append(f"Nearest test marker before the unsafe line is line {test_marker[0]}: `{test_marker[1]}`.")

    function = nearest_match(lines, index, r"\bfn\s+[A-Za-z0-9_]+\s*\(", lookback=30)
    if function:
        facts.append(f"Enclosing or nearest preceding function signature is line {function[0]}: `{function[1]}`.")

    for line_number, text in nearby_matches(lines, index, r"cfg!\(debug_assertions\)", radius=8):
        facts.append(f"Nearby debug-only guard is line {line_number}: `{text}`.")

    for line_number, text in nearby_matches(lines, index, r"from_raw_parts|from_raw_parts_mut|as_ptr|as_mut_ptr|ptr::|\*const|\*mut", radius=8):
        facts.append(f"Nearby raw-memory operation is line {line_number}: `{text}`.")

    if not facts:
        facts.append("No additional local supporting fact was detected; manual review is required.")
    return tuple(facts)


def classify(path: pathlib.Path, lines: list[str], index: int) -> tuple[str, str, str, str]:
    line = lines[index]
    stripped = line.strip()
    window = line_window(lines, index).lower()
    is_test = nearest_test_marker(lines, index)

    if stripped.startswith("//"):
        return (
            "comment-only-unsafe-reference",
            "non-executable comment or documentation",
            "accept",
            "No code is executed by this line; keep as context unless it is misleading.",
        )
    if re.search(r"\bunsafe\s+impl\b", line):
        if re.search(r"\b(Send|Sync)\b", line):
            return (
                "unsafe-impl-send-sync",
                "manual thread-safety assertion",
                "review-upstream-first",
                "Do not auto-edit vendor; verify invariants through upstream docs/tests and patch via upstream or wrapper mitigation only.",
            )
        return (
            "unsafe-impl",
            "manual trait-safety assertion",
            "review-upstream-first",
            "Do not auto-edit vendor; verify the trait invariant and prefer upstream remediation.",
        )
    if re.search(r"\bunsafe\s+fn\b", line):
        return (
            "unsafe-fn",
            "caller-visible unsafe contract",
            "review-upstream-first",
            "Document reachability from Z00Z-owned APIs before deciding on wrapper mitigation or upstream update.",
        )
    if is_test and ("from_raw_parts" in window or "as_ptr" in window or "ptr" in window):
        return (
            "test-only-raw-memory-unsafe",
            "unsafe is inside test-only memory inspection",
            "accept-with-monitoring",
            "No production fix is indicated from this fact alone; keep vendor read-only and monitor upstream changes.",
        )
    if any(token in window for token in ("from_raw_parts", "from_raw_parts_mut", "as_ptr", "as_mut_ptr", "ptr::", "*const", "*mut")):
        return (
            "raw-memory-unsafe",
            "raw pointer or slice reconstruction",
            "review-upstream-first",
            "Assess whether Z00Z-owned code can reach this path; fix through upstream update or wrapper constraints, not direct vendor edits.",
        )
    if any(token in window for token in ("libc::", "extern \"c\"", "mlock", "munlock", "virtual", "prctl", "setrlimit")):
        return (
            "ffi-or-os-unsafe",
            "FFI or operating-system boundary",
            "review-upstream-first",
            "Require platform-specific invariant evidence before accepting; prefer upstream patch if behavior is reachable.",
        )
    if any(part in path.as_posix().lower() for part in ("crypto", "ristretto", "bulletproof", "scalar", "curve")):
        return (
            "crypto-vendor-unsafe",
            "unsafe in cryptographic vendor code",
            "review-upstream-first",
            "Treat as high-assurance review input; no local auto-fix without upstream diff or wrapper-level mitigation.",
        )
    return (
        "unsafe-block-or-expression",
        "generic unsafe block or expression",
        "review-upstream-first",
        "Classify reachability and invariants before deciding whether an upstream patch or local wrapper is needed.",
    )


def collect_findings(vendor_root: pathlib.Path) -> list[Finding]:
    package_index = build_package_index(vendor_root)
    findings: list[Finding] = []
    for source in sorted(vendor_root.rglob("*.rs")):
        lines = source.read_text(encoding="utf-8", errors="replace").splitlines()
        for index, line in enumerate(lines):
            if not re.search(r"\bunsafe\b", line):
                continue
            package = package_for(source, package_index)
            kind, context, decision, rationale = classify(source, lines, index)
            findings.append(
                Finding(
                    path=source,
                    line_number=index + 1,
                    evidence=line.strip(),
                    kind=kind,
                    package=package,
                    context=context,
                    decision=decision,
                    rationale=rationale,
                    verified_facts=collect_verified_facts(source, lines, index, vendor_root, package),
                )
            )
    return findings


def markdown_escape(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ")


def relpath(path: pathlib.Path, root: pathlib.Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def write_report(output: pathlib.Path, vendor_root: pathlib.Path, findings: list[Finding]) -> None:
    repo_root = pathlib.Path.cwd()
    output.parent.mkdir(parents=True, exist_ok=True)
    by_kind = collections.Counter(finding.kind for finding in findings)
    by_decision = collections.Counter(finding.decision for finding in findings)
    packages = sorted({finding.package for finding in findings})
    generated_at = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")

    lines: list[str] = []
    lines.append("# Z00Z Vendor Unsafe Evidence Report")
    lines.append("")
    lines.append(f"- Generated UTC: `{generated_at}`")
    lines.append(f"- Vendor root: `{relpath(vendor_root, repo_root)}`")
    lines.append(f"- Rust unsafe facts found: `{len(findings)}`")
    lines.append("- Policy: vendored code is read-only during automated repair; fixes must be upstream updates, wrapper mitigations, or tracked exceptions.")
    lines.append("")
    lines.append("## Summary")
    lines.append("")
    if findings:
        lines.append("| Dimension | Value | Count |")
        lines.append("| --- | --- | ---: |")
        for kind, count in sorted(by_kind.items()):
            lines.append(f"| kind | `{kind}` | {count} |")
        for decision, count in sorted(by_decision.items()):
            lines.append(f"| decision | `{decision}` | {count} |")
        lines.append("")
        lines.append(f"- Packages with unsafe facts: {', '.join(f'`{package}`' for package in packages)}")
    else:
        lines.append("- No vendored Rust lines containing the `unsafe` keyword were found.")
    lines.append("")
    lines.append("## Findings")
    lines.append("")
    if findings:
        lines.append("| Location | Package | Kind | Evidence | Fix Decision | Rationale |")
        lines.append("| --- | --- | --- | --- | --- | --- |")
        for index, finding in enumerate(findings, start=1):
            location = f"{relpath(finding.path, repo_root)}:{finding.line_number}"
            evidence = markdown_escape(f"`{finding.evidence}`")
            rationale = markdown_escape(f"{finding.context}. {finding.rationale}")
            lines.append(
                f"| `{location}` | `{finding.package}` | `{finding.kind}` | {evidence} | `{finding.decision}` | {rationale} |"
            )
            lines.append("")
            lines.append(f"### Finding {index}: `{location}`")
            lines.append("")
            lines.append(f"- Package: `{finding.package}`")
            lines.append(f"- Kind: `{finding.kind}`")
            lines.append(f"- Evidence line: `{finding.evidence}`")
            lines.append(f"- Fix decision: `{finding.decision}`")
            lines.append(f"- Decision rationale: {finding.context}. {finding.rationale}")
            lines.append("- Verified supporting facts:")
            for fact in finding.verified_facts:
                lines.append(f"  - {fact}")
    else:
        lines.append("No findings.")
    lines.append("")
    lines.append("## Required Human Decision")
    lines.append("")
    if findings:
        lines.append("- Do not auto-edit vendor files.")
        lines.append("- For each `review-upstream-first` item, decide whether the code path is reachable through Z00Z-owned APIs.")
        lines.append("- If reachable and risky, prefer updating the vendored upstream snapshot or adding a Z00Z-owned wrapper/test mitigation.")
        lines.append("- If not reachable or test-only, keep as an accepted monitored fact and re-run this report after vendor updates.")
    else:
        lines.append("- No vendor unsafe decision is currently required.")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    vendor_root = pathlib.Path(args.vendor_root)
    output = pathlib.Path(args.output)
    if not vendor_root.exists():
        raise SystemExit(f"vendor root does not exist: {vendor_root}")
    findings = collect_findings(vendor_root)
    write_report(output, vendor_root, findings)
    print(f"[z00z-l4:vendor] wrote {output} with {len(findings)} unsafe fact(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
