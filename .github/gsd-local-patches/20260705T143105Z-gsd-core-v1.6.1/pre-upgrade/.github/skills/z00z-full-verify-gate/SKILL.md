---
name: z00z-full-verify-gate
description: 'Run the canonical Z00Z quality gate (full_verify.sh) with the right variant for the situation. Use when preparing a commit or PR, checking whether the workspace is clean, running a targeted fast check on a single crate, triaging a build failure, analyzing slow tests, or running the max-safe sweep. Trigger words: verify, full verify, quality gate, build check, clippy, fmt, tests pass, before commit, before PR, slow tests, max safe run.'
argument-hint: 'fast | standard | max-safe | triage | report'
---

# Z00Z Full Verify Gate

## When to Use

- The user wants the canonical Z00Z verification pipeline before a commit, push, or PR.
- A build, clippy, test, or max-safe gate failure needs structured triage.
- The task is to choose between fast, standard, triage, report, or max-safe verification modes.
- The user asks whether the workspace is clean enough to merge or release.

## Mission

Run and interpret the canonical Z00Z quality gate pipeline. Choose the right variant for
the situation, parse failures immediately, and report what must be fixed before a merge.

This skill wraps `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`
and its ecosystem:
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh` — the canonical pipeline
- `./.github/skills/z00z-full-verify-gate/scripts/run_runnable_targets.py` — whitelisted bin/example runner
- `./.github/skills/z00z-full-verify-gate/scripts/runnable_targets.toml` — whitelisted run targets
- `./.github/skills/z00z-full-verify-gate/scripts/run_max_safe_targets.py` — max-safe sweep
- Slow-test report: `reports/full_verify-report-long-running-tests.txt`

---

## Pipeline Stages (in order)

| Step | Command | Fails fast? |
|------|---------|-------------|
| 1 | `cargo fmt --check` | yes |
| 2 | `cargo clippy --workspace --release --all-targets --all-features -- -D warnings` | yes |
| 3 | `cargo test --workspace --release --lib --bins --tests --examples --all-features` | yes |
| 4 | `cargo test --workspace --release --all-features --doc` | yes |
| 5 | `cargo bench --workspace --all-features --no-run` | yes |
| 6 | run whitelisted targets from `runnable_targets.toml` via build reuse | yes |
| 7 | collect slow tests → `reports/full_verify-report-long-running-tests.txt` | no |
| 8 | (optional) heavy simulator replay-contract suite | yes |
| 9 | (optional) max-safe target sweep via prebuilt-artifact reuse | yes |

---

## Variants and When to Use

### `fast` — single crate, skip clean

Use when iterating on one crate and no global Cargo.lock changes were made.

```bash
# Format + clippy + tests for one crate only
cargo fmt --check -p <crate>
cargo clippy -p <crate> --release --all-targets --all-features -- -D warnings
cargo test -p <crate> --release --all-targets --all-features
```

When to use: mid-session iteration, single-file change, no dependency changes.

---

### `standard` — full pipeline without max-safe sweep

Use before every commit and PR. This is the default gate.

```bash
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

Environment defaults:
- `LONG_TEST_SEC=20` — tests slower than 20s appear in the slow-test report
- `Z00Z_DENY_WARNINGS=1` — `-D warnings` active
- `Z00Z_RUN_TARGETS=1` — whitelisted runnable targets execute
- `Z00Z_MAX_SAFE_RUN=0` — max-safe sweep skipped
- The generic test stage runs `--lib --bins --tests --examples`; benchmark targets are compiled in the later `cargo bench --no-run` stage instead of being executed as part of `cargo test`

When to use: before `git commit`, before pushing a branch.

---

### `max-safe` — full pipeline + max-safe target sweep

Use before merging to `main` or when touching cross-crate dependencies.

```bash
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run
```
  This enables the optional max-safe stage.
  That stage uses prebuilt-artifact reuse.

Behavior note:
- Stage 6 uses `run_runnable_targets.py --reuse-build`
- Stage 8 runs `test_stage4_digest_replay_heavy` explicitly from the `scenario_1` test target with `--ignored`
- Stage 9 uses `run_max_safe_targets.py --keep-going --reuse-build --prebuilt-only`
- The orchestration goal is to let the earlier workspace stages build the shared graph once, then reuse or narrowly extend artifacts instead of paying for an extra workspace-wide compile wave
- In practice, `--max-safe-run` enables the optional max-safe stage, and that stage executes through prebuilt-artifact reuse.

When to use: pre-merge, after adding a new public API, after dependency bumps.

---

### `triage` — targeted diagnosis after a failure

When a `standard` or `max-safe` run fails, use targeted commands to isolate the failure stage:

```bash
# Stage 2 — format
cargo fmt --check 2>&1 | head -30

# Stage 3 — clippy only (faster than full build)
cargo clippy --workspace --release --all-targets --all-features -- -D warnings 2>&1 | head -80

# Stage 4/5 — build + test for failing crate only
cargo build -p <crate> --release --all-targets --all-features
cargo test -p <crate> --release --all-targets --all-features 2>&1 | tail -40

# Stage 8 — run a specific whitelisted target manually
cargo run --release -p <package> --bin <name> --features <features> -- <args>

# Stage 8 — reproduce gate behavior with prebuilt artifacts
python3 .github/skills/z00z-full-verify-gate/scripts/run_runnable_targets.py --manifest \
  .github/skills/z00z-full-verify-gate/scripts/runnable_targets.toml --reuse-build --prebuilt-only
```

Simulator-specific commands (from project preferences):
```bash
# Release guard audit
bash scripts/audit/audit_release_feature_guards.sh

# Default release-safe simulator tests
cargo test -p z00z_simulator --release

# Local debug-only secret-export lane (never use for release-capable validation)
cargo run -p z00z_simulator --bin scenario_1 --features wallet_debug_tools
```

---

### `report` — analyze the slow-test report

After any run that completes stage 9, the slow-test inventory is at:
`reports/full_verify-report-long-running-tests.txt`

Read it to find tests above the threshold:

```bash
cat reports/full_verify-report-long-running-tests.txt
# or filter to just the slow ones
grep -v '^#\|^Generated\|^Threshold\|^Workspace\|^Note\|^$' \
  reports/full_verify-report-long-running-tests.txt | head -40
```

---

## Environment Variable Overrides

| Variable | Default | Override purpose |
|----------|---------|-----------------|
| `LONG_TEST_SEC` | `20` | Lower to catch more slow tests; raise to reduce noise |
| `Z00Z_DENY_WARNINGS` | `1` | Set to `0` to allow warnings in CI investigation mode |
| `Z00Z_RUN_TARGETS` | `1` | Set to `0` to skip whitelisted target execution |
| `Z00Z_MAX_SAFE_RUN` | `0` | Set to `1` to enable max-safe sweep without `--max-safe-run` flag |
| `Z00Z_ALL_FEATURES_FLAG` | `--all-features` | Set to empty to build without all features |
| `Z00Z_VERIFY_FEATURES` | `` | Comma-separated features when not using `--all-features` |

Example — run without denying warnings, skip targets:
```bash
Z00Z_DENY_WARNINGS=0 Z00Z_RUN_TARGETS=0 ./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh
```

---

## Failure Triage Decision Tree

After a failed run, identify which stage failed from the output prefix `[full-verify] <stage>`:

```
fmt failed?
  → Run: cargo fmt --check
  → Fix: cargo fmt (auto-format)
  → Re-run stage 2 only

clippy failed?
  → Run: cargo clippy -p <failing_crate> --release --all-targets --all-features -- -D warnings
  → Read first error only, fix, repeat
  → Common causes: unused imports, missing error handling, identifier violations

build failed?
  → Run: cargo build -p <failing_crate> --release --all-features
  → If it's a dependency: check Cargo.toml version constraints
  → If it's a missing symbol: check z00z_utils abstraction layer (run compliance skill)

test failed?
  → Run: cargo test -p <failing_crate> --release --all-features -- --nocapture <test_name>
  → Check if it's a flaky test (re-run 2-3 times)
  → Check reports/full_verify-report-long-running-tests.txt for slow-test timeouts

whitelisted target failed?
  → Check .github/skills/z00z-full-verify-gate/scripts/runnable_targets.toml for the failing target's `allowed_exit_codes`
  → Run the target manually with --help to confirm CLI contract
  → If expected non-zero exit: update `allowed_exit_codes` in the TOML

max-safe sweep failed?
  → Re-run with the same orchestration mode as the gate to isolate failures without per-target cargo churn:
    python3 .github/skills/z00z-full-verify-gate/scripts/run_max_safe_targets.py --manifest .github/skills/z00z-full-verify-gate/scripts/runnable_targets.toml --all-features --reuse-build --prebuilt-only
```

---

## Execution Workflow

### Step 1 — Choose Variant

Ask (or infer from context) which variant applies:
- "before commit / quick check" → `fast` or `standard`
- "before PR / merge" → `standard`
- "cross-crate change / dependency bump" → `max-safe`
- "something failed" → `triage`
- "slow tests" → `report`

### Step 2 — Run

Execute the chosen command. Stream output. If running in context window, cap output display at 60 lines but note the full command completed.

### Step 3 — Interpret

Parse `[full-verify]` prefix lines to identify the stage. If the run succeeds:

```
✅ Full verify passed. Workspace is gate-clean.
```

If it fails, identify the stage and apply the triage branch above.

### Step 4 — Fix Loop

For clippy / fmt failures: apply fix → re-run `fast` variant on affected crate → confirm clean.

For test failures: isolate the failing test → diagnose → fix → re-run test in isolation → re-run full `standard`.

### Step 5 — Gate Confirmation

Before reporting "ready for commit/PR", confirm ALL of the following:
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy ... -- -D warnings` passes with zero warnings
- [ ] All tests pass
- [ ] Doc tests pass
- [ ] Benchmarks compile
- [ ] Whitelisted targets run successfully

If `max-safe` was requested: also confirm max-safe sweep passes.

### Step 6 — Version Bump (if applicable)

If this verify run is the final pre-commit gate, remind to use version-manager for the commit:

```bash
./.github/skills/z00z-git-versioning/scripts/version-manager.sh patch -m "Description of changes"
# or
./.github/skills/z00z-git-versioning/scripts/version-manager.sh minor -m "Description of changes"
```

---

## Integration with Other Skills

- **Before running verify**: optionally run `/z00z-design-foundation-compliance` on modified files first — catches architecture violations before the full compile cycle.
- **After a test failure involving crypto code**: invoke `/crypto-architect` to review the failing test's cryptographic assumptions.

---

## Common Pitfalls

| Pitfall | Cause | Fix |
|---------|-------|-----|
| Clippy passes locally, fails in full_verify | `--release` flag changes monomorphization, exposing different dead-code warnings | Always run clippy with `--release` |
| Tests pass, doc tests fail | Example code in `///` uses outdated API | Update doc examples to match current signatures |
| Slow test threshold exceeded | Test does heavy crypto computation without mocked time/rng | Inject `MockTimeProvider` / `MockRngProvider` in the test |
| Whitelisted target exits non-zero unexpectedly | New CLI flag changed exit behavior | Update `allowed_exit_codes` or `args` in `runnable_targets.toml` |
| Max-safe sweep catches a crate that standard misses | Standard covers workspace lib/bin/test/example surfaces plus benches/doc tests; max-safe adds extra curated reuse/execution combinations | Do not skip max-safe for cross-crate API changes |

---

## Example Invocations

### Before every commit
```
/z00z-full-verify-gate standard
```
→ Runs the full pipeline. This is the default gate before any `git commit`.

### Fast loop while iterating on one crate
```
/z00z-full-verify-gate fast
```
→ Prompts for crate name, then runs fmt + clippy + tests scoped to that crate only.  
  Use mid-session when only one crate was changed and no `Cargo.lock` changes were made.

### Before merging to main or after a dependency bump
```
/z00z-full-verify-gate max-safe
```
→ Full pipeline plus max-safe target sweep. Required before merging cross-crate API changes.

### Something failed — isolate the stage
```
/z00z-full-verify-gate triage
```
→ Runs targeted per-stage commands to identify whether the failure is fmt, clippy, build, test, or a whitelisted target.

### Review slow test inventory after a full run
```
/z00z-full-verify-gate report
```
→ Reads `reports/full_verify-report-long-running-tests.txt` and lists tests above the slow threshold.

### Skip warnings denial for CI investigation
```
/z00z-full-verify-gate standard Z00Z_DENY_WARNINGS=0
```
→ Runs standard pipeline without `-D warnings`, useful when investigating a noisy clippy lint.

### Skip whitelisted target execution during fmt/clippy-only iteration
```
/z00z-full-verify-gate standard Z00Z_RUN_TARGETS=0
```
→ Runs all stages but skips the binary execution step — faster if you're only fixing lint issues.
