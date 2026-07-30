---
name: smart-tests-bootstrap
description: 'Run the canonical release-only Phase 069 bootstrap-fast gate. It compiles storage once, executes the ordinary non-recursive storage suite plus curated Nova and Plonky3 structural/security smoke packets, records selected and deferred coverage under the canonical checkpoint output root, and never runs a real heavyweight prover. Trigger words: fast tests, quick tests, bootstrap tests, smart tests, fast-fail, debug tests, quick check.'
argument-hint: '[run|show|refresh]'
---

# Smart Tests Bootstrap

## When to Use

- The user wants a fast-fail sanity test pass instead of the full workspace suite.
- The task is quick debugging, pre-commit validation, CI triage, or high-signal regression checking.
- The bootstrap gate should run from the canonical colocated script rather than generating a new helper.
- The user asks for smart tests, quick tests, bootstrap tests, or a representative subset run.

## Mission

Use the bootstrap script that lives inside this skill:
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

This skill is self-contained. It does not generate a script in `scripts/` and it does not depend on external skill assets.

The default script is the mandatory Phase 069 Tier-0 regression detector:

- all commands use Cargo's `--release` profile;
- storage release artifacts reuse the canonical physical cache at
  `.cache/phase-069/plan-08/cargo-release`;
- curated Nova source/owner/R1CS smoke, non-recursive storage units, and
  Plonky3 structural/security smoke run;
- real Nova milestones, real Plonky3 proving, Plan-08 integration semantics,
  complete foundational crate units, wallets, benches, examples, and the broad
  workspace suite are explicitly deferred and listed in machine-readable
  evidence;
- the cache-warm promotion target is `12` seconds, the hard wall is `60`
  seconds, and the post-compile packet is capped at `30` seconds with at most
  eight test threads;
- the ordinary storage packet inventories the release libtest binary, removes
  only ignored tests and the explicitly deferred Nova/Plonky3 namespaces, then
  executes every remaining test exactly once through a bounded process pool
  with one test thread per process; its selected manifest and digest are
  evidence, so scheduling cannot silently reduce coverage;
- a recognized cold compile that reaches the hard wall returns typed
  `prewarm_required`; only then may one isolated compile-only prewarm populate
  the same cache before bootstrap is rerun. Prewarm is never acceptance
  evidence.

Every run writes its source digest, timing, selected gates, and deferred gates
under
`crates/z00z_storage/outputs/checkpoint/069-08/task-1/test-pyramid/bootstrap/`.
This gate is early regression evidence, not milestone or full acceptance.

Plan-08 higher tiers use
`./.github/skills/smart-tests-bootstrap/scripts/plonky3_milestone_tests.sh`.
Its `semantic` mode executes all four bounded release-only epoch/history
integration targets in one Cargo invocation with a `10`-second warm promotion
target and `30`-second hard wall. Every mode that performs real proving
delegates exactly one named test to `plonky3_resource_worker.sh`; there is no
command that automatically chains heavy proofs.

The speedup is workflow-only. It may come from ordered tier selection, Cargo
fingerprint reuse, current-boot isolation-preflight reuse, and deterministic
changed-surface routing. It may never come from reduced fixtures, theorem or
mutation coverage, security parameters, non-release builds, skipped final
acceptance, or treating a lower tier as equivalent to a higher tier.

---

## Primary Output

When this skill is invoked, run or inspect the script at:
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

Supported behaviors:

1. `run` or no argument: execute the script.
2. `show`: print the script path and summarize what it covers.
3. `refresh`: update the colocated script in place when test selection rules change.

If the user asks to run it, use:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

---

## Canonical Script

The script content lives at:
`./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

After refreshing the script, keep it executable:

```bash
chmod +x ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```
