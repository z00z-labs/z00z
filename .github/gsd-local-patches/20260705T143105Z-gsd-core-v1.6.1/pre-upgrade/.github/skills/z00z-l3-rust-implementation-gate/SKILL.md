---
name: z00z-l3-rust-implementation-gate
description: Auto-invoked when user wants to check Rust code, fix broken implementation, verify no panic, run tests, or validate Z00Z state/crypto/wallet code. Also triggers on L3 verification, cargo fmt, clippy, nextest, Miri, Kani, Loom, Prusti, Verus, Creusot, proptest, Rust implementation safety, UB, overflow, and concurrency interleavings.
---

# Z00Z L3 Rust Implementation Gate

Verify Rust implementation behavior with formatting, linting, tests, UB checks, bounded model checking, and targeted concurrency/formal checks.

## When to Use

Use this skill when:

- Rust source, tests, benches, examples, public APIs, parsers, state transitions, wallet logic, storage logic, or crypto glue changed.
- The user asks for implementation verification, panic checks, overflow checks, or concurrency review.

## Workflow

1. Run `scripts/verify-fast.sh` for the default Rust gate.
2. Run `scripts/verify-miri.sh` for unsafe-adjacent, parser, crypto, wallet, and storage changes.
3. Run `scripts/verify-kani.sh` when Kani harnesses or safety contracts exist.
4. Run `scripts/verify-loom.sh` when concurrency code changed.
5. Run `scripts/verify-prusti.sh` when Prusti targets or `Z00Z_PRUSTI_PACKAGES` exist.
6. Run `scripts/verify-verus.sh` only for explicit Verus targets; use Creusot as a later alternative, not a default duplicate gate.

## Gate Criteria

- `cargo fmt`, Clippy with zero warnings, and tests must pass for relevant surfaces.
- Heavy workspace and crate compilation/test runs use release mode by default.
- Downstream cargo-based L3 scripts keep that release-profile default unless the underlying tool does not support a release switch.
- Orchestrator-managed L3 runs disable wall-clock timeout cutoffs by default and emit command-level profiling events into the active verifier run root.
- Orchestrator-managed L3 runs prefer `cargo test` over `cargo nextest` unless explicitly forced, because test binaries must inherit the verifier run-root env and keep all mutable artifacts inside `reports/z00z-verification-orchestrator-<timestamp>/`.
- Untrusted input paths must not panic, overflow, or accept malformed data by default.
- Consensus, storage root, checkpoint, and validator logic must remain deterministic.
- Missing harnesses for critical code are UNKNOWN coverage, not PASS.

## Scripts

```bash
.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh
.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-miri.sh
.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-kani.sh
.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-loom.sh
.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-prusti.sh
.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-verus.sh
```

## Examples

```text
User: Verify this storage proof parser.
Assistant: Runs fast Rust checks, Miri if available, and routes parser coverage to L4 fuzzing.
```

```text
User: I changed async validator code.
Assistant: Runs fast Rust checks and Loom-targeted tests if loom instrumentation exists.
```
