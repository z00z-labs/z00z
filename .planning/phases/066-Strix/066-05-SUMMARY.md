---
phase: 066-Strix
plan: 066-05
status: complete
completed_at: 2026-07-02
next_plan: 066-06
summary_artifact_for: .planning/phases/066-Strix/066-05-PLAN.md
requirements_completed:
  - REQ-009
  - REQ-011
  - REQ-012
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-05 Summary: Z00Z Profile Skill And Existing Skill Integration

## 🎯 Outcome

`066-05` is complete.

Phase 066 now has one explicit Z00Z-only adapter layer that composes the
generic `pentest-*` skill family with existing repository security skills,
existing closure prompts, and explicit local deterministic simulation
requirements for distributed HJMT, wallet, storage, validator, watcher, and
publication-boundary review.

The landed result does not create a parallel audit stack. It routes Z00Z
review through `attack-surfaces-create`, `z00z-crypto-auditor`, the generic
`pentest-*` lanes, and `gsd-audit-4.prompt.md`, while explicitly forbidding
edits under `crates/z00z_crypto/tari/**`.

## 📦 Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-05-SUMMARY.md`
- `.github/skills/z00z-pentest-profile/SKILL.md`
- `.github/skills/z00z-pentest-profile/references/z00z-invariants.md`
- `.github/skills/z00z-pentest-profile/references/profile-routing.md`
- `.github/prompts/pentest-local-z00z.prompt.md`

## 🔧 Landed Changes

- Z00Z-only adapter skill
  - Added `.github/skills/z00z-pentest-profile/SKILL.md` with `dry-run`,
    `run`, and `closeout` modes.
  - The skill now loads `.github/copilot-instructions.md` and
    `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` before any Z00Z-specific
    analysis.
  - The skill now forbids edits under `crates/z00z_crypto/tari/**`.
- Explicit Z00Z invariants
  - Added `references/z00z-invariants.md` with repository, boundary, and
    simulation invariants.
  - The reference records the fake-only external boundaries and the required
    use of real local project primitives for crypto, storage, wallet history,
    publication bindings, validator or watcher checks, and per-component
    state.
- Canonical lane routing
  - Added `references/profile-routing.md` with the Z00Z lane map for
    `crypto/proof`, `wallet/keys`, `storage/checkpoints`, `rollup/DA`,
    `RPC/network`, `simulator/fixtures`, and
    `dependencies/supply-chain`.
  - The routing file makes `attack-surfaces-create` and
    `z00z-crypto-auditor` the canonical Z00Z review routes and keeps DAST off
    by default except for the explicitly gated `RPC/network` lane after local
    scope validation.
- Prompt integration
  - Added `.github/prompts/pentest-local-z00z.prompt.md`.
  - The prompt now exposes a documentation-only dry run that prints the lane
    map and explicitly does not execute DAST.
  - The prompt now points heavy closeout to `gsd-audit-4.prompt.md`.

## ✅ Validation

Commands and evidence used for `066-05` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `rg -n "attack-surfaces-create|z00z-crypto-auditor|gsd-audit-4|Z00Z_DESIGN_FOUNDATION|crates/z00z_crypto/tari" .github/skills/z00z-pentest-profile .github/prompts/pentest-local-z00z.prompt.md`
- `rg -n "dry-run|Do not execute DAST|crypto/proof|wallet/keys|storage/checkpoints|rollup/DA|RPC/network|simulator/fixtures|dependencies/supply-chain" .github/skills/z00z-pentest-profile .github/prompts/pentest-local-z00z.prompt.md`
- `rg -n "replication|quorum|conflict resolution|standby catch-up|route rollout|dispatch|membership|restart|partition/heal|stale lineage|divergent roots|failure telemetry|wallet history|storage commits|publication bindings|validator/watcher checks|per-component state" .github/skills/z00z-pentest-profile/references`
- `rg -n "No dry-run may execute DAST|Do not execute DAST|DAST only after local scope validation|no DAST by default" .github/skills/z00z-pentest-profile .github/prompts/pentest-local-z00z.prompt.md`
- `rg -n "crates/z00z_crypto/tari/\\*\\*|read-only vendor code|forbidden for edits" .github/skills/z00z-pentest-profile .github/prompts/pentest-local-z00z.prompt.md`
- `git diff --check -- .github/skills/z00z-pentest-profile/SKILL.md .github/skills/z00z-pentest-profile/references/z00z-invariants.md .github/skills/z00z-pentest-profile/references/profile-routing.md .github/prompts/pentest-local-z00z.prompt.md`
- `rg -n "FIXME|panic!\\(|unimplemented!\\(|todo!\\(" .github/skills/z00z-pentest-profile .github/prompts/pentest-local-z00z.prompt.md`

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green after the `066-05`
  artifact set landed.
- The integration grep proves that the Z00Z profile routes through
  `attack-surfaces-create`, `z00z-crypto-auditor`,
  `gsd-audit-4.prompt.md`, `Z00Z_DESIGN_FOUNDATION`, and the forbidden Tari
  vendor path.
- The dry-run grep proves that the profile prints the full lane map and states
  that dry run does not execute DAST.
- The simulation grep proves that the distributed HJMT and wallet/storage
  local deterministic simulation requirements are explicit in the profile
  references.
- Diff hygiene checks were clean.

`cargo test --release` was not rerun for `066-05` because this plan changed
skills and prompts only; it did not change Rust or test-affecting runtime
code.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime did not provide a usable automated prompt-execution path
for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-05-PLAN.md current_task="Z00Z Profile Skill And Existing Skill Integration" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-05-PLAN.md current_task="Z00Z Profile Skill And Existing Skill Integration" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-05-PLAN.md current_task="Z00Z Profile Skill And Existing Skill Integration" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-05-PLAN.md`, `066-TODO.md`, `attack-surfaces-create`,
    `z00z-crypto-auditor`, `gsd-audit-4.prompt.md`, and the relevant Design
    Foundation anchors.
  - Checked that the Z00Z profile reuses existing repository skill surfaces
    and does not introduce a parallel audit layer.
  - Result: no material drift found.
- Pass 2
  - Re-ran integration, dry-run, simulation, and vendor-path greps.
  - Result: clean for the `066-05` scope.
- Pass 3
  - Re-ran DAST-exclusion grep, diff hygiene checks, and placeholder grep.
  - Result: clean for the `066-05` scope. No hidden DAST execution path or
    alternate audit stack remained.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## 🧾 Closeout

`066-05` closes `WS-05` by landing the Z00Z-only adapter skill, the
repository-specific invariants packet, the canonical lane-routing map, and the
local prompt surface that prints the lane map without DAST and routes heavy
closeout through existing repository audit tooling.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-06-PLAN.md`.
