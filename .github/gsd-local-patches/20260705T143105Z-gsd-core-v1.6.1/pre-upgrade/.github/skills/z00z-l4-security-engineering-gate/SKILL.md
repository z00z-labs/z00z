---
name: z00z-l4-security-engineering-gate
description: Auto-invoked when user wants to check security hygiene, fuzz parsers, audit dependencies, detect unsafe code, or run supply-chain checks for Z00Z. Also triggers on L4 verification, cargo-fuzz, cargo-audit, cargo-deny, cargo-vet, cargo-geiger, cargo-semver-checks, dudect, dependency audit, unsafe report, parser fuzzing, and constant-time testing.
---

# Z00Z L4 Security Engineering Gate

Run fuzzing, dependency, unsafe, coverage, semantic-version, and constant-time engineering checks.

## When to Use

Use this skill when:

- `Cargo.toml`, `Cargo.lock`, dependency policy, fuzz targets, parser/deserializer code, unsafe code, public API, or timing-sensitive code changed.
- The user asks for security hygiene, supply-chain review, fuzzing, constant-time checks, or release readiness.

## Workflow

1. Run `scripts/audit-supply-chain.sh` for RustSec, deny, vet, duplicate dependency, and semver checks where configured.
2. Run `scripts/unsafe-report.sh` for project-owned unsafe usage inventory.
3. For high-assurance, release, or "check everything including vendor" work, run `scripts/unsafe-report.sh --all`. This includes vendored code and writes a fact-based vendor report. Do not auto-fix vendor files; classify each fact as upstream-fix, wrapper-mitigation, accepted test-only, or monitored exception.
4. Run `scripts/run-fuzz-short.sh` before PRs for changed parser/decoder targets.
5. Run `scripts/run-fuzz-nightly.sh` for nightly/release work.
6. Run `scripts/run-constant-time.sh` for explicit timing benches or dudect harnesses.
7. Use `scripts/minimize-crash.sh` to reduce a fuzz crash before filing or fixing it.

## Gate Criteria

- Dependency advisories and deny violations are failures.
- Missing `cargo-vet` configuration is UNKNOWN unless the project has explicitly deferred vet adoption.
- Every externally supplied byte parser should have at least one fuzz target or an explicit tracked exception.
- Cargo-backed fuzzing and unsafe-analysis stages run with release or optimized profiles by default when the underlying tool supports it.
- Orchestrator-managed L4 runs disable wall-clock timeout cutoffs by default and record profiling for shell-managed commands under the verifier run root.
- Unsafe reports are audit inputs; they do not prove vulnerability or safety by themselves.
- Vendor unsafe facts require evidence and a decision, but automated repair must not edit `crates/z00z_crypto/tari/` directly.

## Scripts

```bash
.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh
.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh
.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh --all
.github/skills/z00z-l4-security-engineering-gate/scripts/unsafe-report.sh --vendor-report-only
.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-short.sh
.github/skills/z00z-l4-security-engineering-gate/scripts/run-fuzz-nightly.sh
.github/skills/z00z-l4-security-engineering-gate/scripts/run-constant-time.sh
.github/skills/z00z-l4-security-engineering-gate/scripts/minimize-crash.sh <target> <crash-file>
```

## Examples

```text
User: Cargo.lock changed.
Assistant: Runs supply-chain audit and reports cargo-audit, cargo-deny, cargo-vet, duplicate dependency, and semver status.
```

```text
User: I added a proof decoder.
Assistant: Requires a fuzz target or marks parser coverage UNKNOWN.
```
