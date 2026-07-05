---
name: z00z-design-foundation-compliance
description: 'Z00Z Design Foundation compliance audit for Rust source files, modules, or entire crates. Use when reviewing code before a commit/PR, checking a new crate or module for architecture violations, verifying ONE_SOURCE_OF_TRUTH adherence, auditing naming conventions, identifier length, error handling, import grouping, and crypto domain separation. Returns a severity-classified violation report with fix suggestions. Trigger words: compliance, design foundation, architecture review, one source of truth, z00z_utils, identifier length, naming convention, audit, violations.'
argument-hint: 'path/to/file.rs or crate name or "all" for workspace scan'
---

# Z00Z Design Foundation Compliance Audit

## When to Use

- The user wants a Z00Z-specific compliance audit before a commit or PR.
- A Rust file, crate, or module must be checked against `Z00Z_DESIGN_FOUNDATION.md` and ONE_SOURCE_OF_TRUTH rules.
- The request is about architectural violations, naming limits, import grouping, or z00z_utils boundary enforcement.
- The output should be a severity-ranked violation report with concrete fix guidance.

## Mission

Act as a Z00Z architecture enforcer. Audit Rust source code against the mandatory rules in
`.github/requirements/Z00Z_DESIGN_FOUNDATION.md` and `.github/requirements/ONE_SOURCE_OF_TRUTH.md`, and report
every violation with severity, location, and an actionable fix.

Be precise. Report only real violations — not style preferences unrelated to the Design Foundation.
Lead with the highest-severity findings. Do not bury blockers at the end.

**Reference files (load if needed):**
- [`.github/requirements/Z00Z_DESIGN_FOUNDATION.md`](../../requirements/Z00Z_DESIGN_FOUNDATION.md)
- [`.github/requirements/ONE_SOURCE_OF_TRUTH.md`](../../requirements/ONE_SOURCE_OF_TRUTH.md)
- [`copilot-instructions.md`](../../copilot-instructions.md)

---

## Severity Taxonomy

| Level | Label | Definition |
|-------|-------|------------|
| V0 | BLOCKER | Violates a FORBIDDEN / ABSOLUTELY FORBIDDEN rule — must fix before merge |
| V1 | HIGH | Directly bypasses a mandatory abstraction (z00z_utils) or crypto safety rule |
| V2 | MEDIUM | Naming, identifier length, import grouping, or error-handling deviation |
| V3 | LOW | Documentation gap, missing doc comment, or missing `#![doc = include_str!]` |
| V4 | INFO | Advisory, suggestion, pattern inconsistency with no hard rule backing |

Always lead with V0 and V1 findings.

---

## Compliance Checklist

### C1 · ONE SOURCE OF TRUTH (V0/V1)

Scan the target scope for **direct use of low-level APIs that MUST go through `z00z_utils`**:

| Forbidden pattern | Must replace with | Severity |
|-------------------|-------------------|----------|
| `use std::fs` (except `std::fs::File`) | `z00z_utils::io::*` | V0 |
| `fs::write`, `fs::read`, `fs::read_to_string` | `z00z_utils::io::write_file`, etc. | V0 |
| `serde_json::to_string`, `serde_json::from_str`, etc. | `z00z_utils::codec::JsonCodec` | V0 |
| `serde_yaml::from_str`, `serde_yaml::to_string` | `z00z_utils::codec::YamlCodec` | V0 |
| `serde_yaml::Value` | `z00z_utils::config::YamlValue` | V0 |
| `serde_yaml::from_value` | `z00z_utils::config::from_yaml_value` | V0 |
| `SystemTime::now()`, `UNIX_EPOCH` | `z00z_utils::time::TimeProvider` | V1 |
| `rand::thread_rng()`, `rand::random()` | `z00z_utils::rng::RngProvider` | V1 |
| `log::info!`, `log::warn!`, etc. (direct) | `z00z_utils::logger::Logger` | V1 |
| `bincode::serialize`, `bincode::deserialize` (direct) | `z00z_utils::codec::BincodeCodec` | V1 |
| `tracing::info!`, `tracing::warn!`, `tracing::error!`, etc. (direct) | `z00z_utils::logger::Logger` | V1 |
| `tokio::fs::write`, `tokio::fs::read`, `tokio::fs::read_to_string` | `z00z_utils::io::*` | V1 |
| `chrono::Utc::now()`, `chrono::Local::now()` | `z00z_utils::time::TimeProvider` | V1 |
| `println!`, `eprintln!` in library crates (non-binary) | `z00z_utils::logger::Logger` | V2 |
| `std::env::var` direct use | `z00z_utils::config::EnvConfig` | V2 |

**Exception:** `std::fs::File` for streaming operations (ZIP, large files) is acceptable. Mark as INFO if used correctly.

**Scope:** Applies to all business-logic crates (`z00z_core`, `z00z_wallets`, `z00z_rollup_node`, `z00z_storage`, `z00z_runtime`, `z00z_simulator`, `z00z_telemetry`, `z00z_networks`, `z00z_da_celestia`, `z00z_extensions`). Does NOT apply to `z00z_utils` (it IS the abstraction layer) or vendored code in `z00z_crypto/tari/`.

### C2 · VENDOR CODE ISOLATION (V0)

- Detect any `write`, `create`, or `modify` operation targeting paths inside `z00z_crypto/tari/`.
- Detect any `use` imports modifying or patching vendored tari modules.
- Flag as V0 — ABSOLUTELY FORBIDDEN.

### C3 · CRYPTO DOMAIN SEPARATION (V1)

For any cryptographic operation using `hash_domain!` or manually constructed hash contexts:

- Each domain string MUST follow `"app/{module}/{operation}/{environment}"`.
- Production and test domains MUST be strictly separate.
- The same domain string MUST NOT appear in more than one logical context.
- Missing `hash_domain!` for a new crypto operation = V1 finding.

### C4 · IDENTIFIER LENGTH (V2)

Count words in all identifiers (functions, methods, constants, trait methods, async fns)
using the split rules: `_`, `-`, and PascalCase transitions.

An identifier with `word_count > 5` is a V2 violation.

**Examples of violations:**
- `CREATE_WALLET_WLT_FAILPOINT_AFTER_DB_CREATE` → 8 words → V2
- `wallet_id_is_deterministic_with_mock_rng_provider` → 8 words → V2

For each violation, provide a `recommended_rename` of ≤ 5 words that preserves meaning and follows project style.

### C5 · NAMING CONVENTIONS (V2)

- Types / structs / enums: `PascalCase` (nouns)
- Functions / methods: `snake_case` (verbs)
- Booleans: `is_*` or `has_*` prefix
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case` (nouns)

Deviations = V2.

### C6 · IMPORT GROUPING (V2)

All `use` imports from the **same crate or module** MUST be grouped into a **single `use` statement with braces**.

```rust
// V2 violation — separate lines
use z00z_crypto::hash_domain;
use z00z_crypto::ByteArray;

// Correct
use z00z_crypto::{hash_domain, ByteArray};
```

Wildcard imports (`use super::*`, `use crate::*`) in production code = V2 (obscures what is in scope).
Exception: `use super::*` inside `#[cfg(test)] mod tests { ... }` is the standard test pattern → V4 only.

### C7 · ERROR HANDLING (V1/V2)

| Pattern | Severity |
|---------|----------|
| `unwrap()` in production code (outside `#[cfg(test)]`) | V1 |
| `expect()` without a documented safety invariant comment | V2 |
| `panic!()` for error conditions | V1 |
| Missing `thiserror::Error` on an error enum | V2 |
| Missing `pub type Result<T, E = Error> = std::result::Result<T, E>;` in `src/lib.rs` or `src/error.rs` | V2 |

### C8 · MODULE ORGANIZATION (V1/V2)

- Module nesting depth exceeding 2 levels = V2.
- Items marked `pub` but only referenced within the same crate (should be `pub(crate)`) = V2.
- Public items not exported through `lib.rs` facade = V2.
- Missing `#![doc = include_str!("../README.md")]` in `lib.rs` = V2 (MANDATORY per Design Foundation).
- Missing `#![forbid(unsafe_code)]` at crate root = V1 (must be present unless crate explicitly requires unsafe).
- `unsafe` block present without `// SAFETY: <invariant reason>` comment = V2.

### C9 · DOCUMENTATION (V2/V3/V4)

Scope: apply only to **externally exported `pub` items visible from the crate's `lib.rs` facade**,
including methods on exported `pub trait` definitions.
Internal `pub(crate)` and private items → V4 (INFO) at most.

- Exported `pub` function / type / trait missing `///` doc comment = V3.
- Exported doc comment missing summary or `# Examples` for non-trivial items = V3.
- Exported crypto-related item missing `# Security` section = V2.
- `cargo doc --no-deps` warning on an exported item = V3.
- `pub(crate)` item missing doc comment = V4 (advisory only).

### C10 · PARALLELISM (V1)

- `std::thread::spawn` or heavy CPU work inside an `async fn` (without `spawn_blocking`) = V1.
- `tokio::time::sleep` or blocking I/O inside `rayon` threads = V1.
- `rayon` used for I/O-bound work = V2.

### C11 · ENGLISH-ONLY (V2)

All identifiers, comments, doc strings, error messages, and string literals that appear in
source code MUST be in English. Non-English content = V2.

### C12 · PUBLIC API DESIGN (V1/V4)

Checks that public API surfaces do not leak external dependency types:

| Pattern | Severity |
|---------|----------|
| Function signature contains `serde_json::Value`, `serde_yaml::Value`, or other external crate types as `pub` parameters or return types | V1 |
| `pub` function accepts/returns `serde_json::Error` or `serde_yaml::Error` directly | V1 |
| `pub` struct field of external unstable type exposed without wrapper | V1 |
| Configurable `pub` type missing builder pattern (when >3 constructor parameters) | V4 (advisory) |

**Note:** `z00z_utils` wrapper types (`YamlValue`, codec types) are NOT external leaks — they are the project's own abstraction.

---

## Execution Workflow

### Step 1 — Scope Resolution

Determine what to audit:
- If the argument is a file path → audit that file only.
- If the argument is a crate name → audit `crates/<name>/src/**/*.rs`.
- If the argument is `"all"` or omitted → audit all `crates/*/src/**/*.rs` (exclude `z00z_crypto/tari/`).

Read the target files. For large scopes, use `grep_search` to locate violations across the tree before doing deep reads.

### Step 2 — Systematic Scan

Run through checklist items C1–C12 in order. For each finding, record:

```
[LEVEL] [C#] <file>:<line>
Pattern: <the offending code or identifier>
Rule: <which rule was violated>
Fix: <exact replacement or rename>
```

### Step 3 — Report

Produce the **Compliance Report** in this format:

```markdown
# Z00Z Design Foundation: Compliance Report
**Scope:** <files or crates audited>
**Date:** YYYY-MM-DD
**Total findings:** N (V0: n, V1: n, V2: n, V3: n, V4: n)

---

## BLOCKER (V0) — N findings
<findings list>

## HIGH (V1) — N findings
<findings list>

## MEDIUM (V2) — N findings
<findings list>

## LOW (V3) — N findings
<findings list>

## INFO (V4) — N findings
<findings list>

---
## Summary
<1–3 sentences on the most critical concerns and recommended first action>
```

### Step 4 — Auto-Fix Offer

After producing the report, ask the user:

> "Should I auto-fix V0/V1 violations?  
> Or specify a severity range to fix (e.g. "V0–V2")."

Apply fixes only after explicit confirmation. Fix one file at a time. Re-run the relevant checklist items after each fix to verify resolution.

### Step 5 — Verify

After any auto-fix, run verify commands scoped to the audited crate:

```bash
# Replace <crate> with the audited crate name, or omit path restriction for workspace-wide scan
BUSINESS_CRATES="z00z_core z00z_wallets z00z_rollup_node z00z_storage z00z_runtime z00z_simulator z00z_telemetry z00z_networks z00z_da_celestia z00z_extensions"
for c in $BUSINESS_CRATES; do
  grep -rn "use std::fs" crates/$c/src/ | grep -v "std::fs::File" | sed "s/^/[$c] /"
  grep -rn "serde_yaml::Value" crates/$c/src/ | sed "s/^/[$c] /"
  grep -rn "serde_json::" crates/$c/src/ | sed "s/^/[$c] /"
  grep -rn "SystemTime::now\|UNIX_EPOCH" crates/$c/src/ | sed "s/^/[$c] /"
done

# Clippy scoped to audited crate (avoids full workspace rebuild)
cargo clippy -p <crate> --all-targets --all-features 2>&1 | head -80
```

Report results. If new violations are found, loop back to Step 2 for the affected files.

---

## Quick-Reference: Forbidden → Correct

| Forbidden | Correct |
|-----------|---------|
| `std::fs::write(p, d)` | `z00z_utils::io::write_file(p, &d)` |
| `std::fs::read_to_string(p)` | `z00z_utils::io::read_to_string(p)` |
| `serde_json::to_string(&v)` | `JsonCodec.serialize(&v)` |
| `serde_json::from_str(s)` | `JsonCodec.deserialize(s.as_bytes())` |
| `serde_yaml::from_str(s)` | `YamlCodec.deserialize(s.as_bytes())` |
| `serde_yaml::Value` | `z00z_utils::config::YamlValue` |
| `SystemTime::now()` | `time_provider.unix_timestamp()` |
| `rand::thread_rng()` | `rng_provider.rng()` |
| `unwrap()` (production) | `?` with typed `Result` |
| `expect("msg")` (no safety doc) | `?` or add `// SAFETY: <reason>` comment |
| Multiple `use z00z_crypto::X;` lines | `use z00z_crypto::{X, Y, Z};` |
| `bincode::serialize(&v)` | `BincodeCodec.serialize(&v)` |
| `tracing::info!(...)` (direct in library) | `logger.log(...)` via `Logger` trait |
| `println!(...)` (in library crate) | `logger.log(...)` via `Logger` trait |
| `std::env::var("KEY")` (direct) | `env_config.get("KEY")?` via `EnvConfig` |
| `chrono::Utc::now()` | `time_provider.unix_timestamp()` |
| `unsafe { ... }` without `// SAFETY:` | `// SAFETY: <reason>\nunsafe { ... }` |

---

## Do Not Flag

- `std::fs::File` used for streaming (ZIP creation, large binary writes) → INFO at most.
- `unwrap()` / `expect()` inside `#[cfg(test)]` blocks → not a violation.
- `serde_json` / `serde_yaml` / `bincode` / `tracing` / `chrono` in `z00z_utils` itself (it IS the abstraction layer).
- Tari crate internal code under `z00z_crypto/tari/` — do not audit vendored code.
- `panic!` in `unreachable!` substitution with documented invariant.
- `pub(crate)` items missing `///` doc → V4, not a merge blocker.
- `#![allow(unsafe_code)]` crates that explicitly document their unsafe contract — flag as V2 review note, not V1 blocker.
- `use super::*` inside `#[cfg(test)] mod tests { ... }` — standard test import pattern → V4 only, never V2.
- `println!` / `eprintln!` in binary entry points (`main.rs`, `bin/*.rs`) for CLI output — not a logger bypass.
- `tracing::*` macros inside `z00z_utils::logger` itself — it IS the logger implementation.

## Example Invocations

### Single file
```
/z00z-design-foundation-compliance crates/z00z_core/src/assets/registry.rs
```
→ Audits one file. Fast. Use before committing a changed file.

### Single crate
```
/z00z-design-foundation-compliance z00z_wallets
```
→ Audits all `crates/z00z_wallets/src/**/*.rs`. Good pre-PR check on the crate you touched.

### Workspace-wide
```
/z00z-design-foundation-compliance all
```
→ Full workspace scan (excludes `z00z_utils` internals and `z00z_crypto/tari/`). Run before a release tag.

### Specific checklist only
```
/z00z-design-foundation-compliance z00z_core --only C1
/z00z-design-foundation-compliance z00z_core --only C4,C5
```
→ Useful when you only care about ONE SOURCE OF TRUTH violations, or only want identifier/naming checks.

### Auto-fix V0/V1 immediately
```
/z00z-design-foundation-compliance crates/z00z_storage/src/db.rs --fix V0-V1
```
→ Runs audit, then auto-applies fixes for BLOCKER and HIGH findings without asking for confirmation.

### New crate onboarding
```
/z00z-design-foundation-compliance z00z_da_celestia
```
→ Use when a new crate has just been scaffolded — catches missing `#![forbid(unsafe_code)]`,
  missing `#![doc = include_str!]`, split imports, and direct `std::fs` before any real logic is written.

### Pre-commit quick check (last edited files)
```
/z00z-design-foundation-compliance crates/z00z_core/src/tx/builder.rs crates/z00z_core/src/tx/validator.rs
```
→ Accepts a space-separated list of files. Audits only what changed in this commit.
