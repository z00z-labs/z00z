---
phase: 066-Strix
plan: 066-14
status: complete
completed_at: 2026-07-03
next_plan: complete
summary_artifact_for: .planning/phases/066-Strix/066-14-PLAN.md
requirements_completed:
  - REQ-011
  - REQ-012
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-14 Summary: Documentation And Migration Guide

## Outcome

`066-14` is complete.

Phase 066 now has an operational documentation surface that explains how the
local pentest orchestration path is used inside Z00Z and how the reusable
generic core is migrated into another repository without carrying Z00Z-only
assumptions forward.

The final drift in this lane was documentation-shaped, not runtime-shaped:
`tools/penetration/README.md` did not yet separate the generic core from the
Z00Z-only overlay, the migration references did not exist yet, and the
Z00Z-specific invariants note did not explicitly warn another repository not to
copy it unchanged as a default profile.

After the closeout:

- operators have one canonical README for entrypoints, artifacts, report roots,
  tool-root rules, Docker notes, and failure modes;
- migration consumers have one concrete guide and one adoption checklist;
- the Z00Z invariants document is explicitly marked project-specific rather
  than generic default material;
- the docs contract is covered by executable tests;
- Phase 066 has no remaining open execution packet.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-14-SUMMARY.md`
- `tools/penetration/README.md`
- `.github/skills/pentest-local-orchestrator/references/migration-guide.md`
- `.github/skills/pentest-local-orchestrator/references/new-project-checklist.md`
- `.github/skills/z00z-pentest-profile/references/z00z-invariants.md`
- `tests/penetration/test_docs_contracts.py`

## Landed Changes

- Operator README split and canonical usage contract
  - `tools/penetration/README.md` now separates `Generic Core` from the
    `Z00Z-Only Overlay`, identifies `./z00z_penetration_tests.sh` as the only
    external human or agent entrypoint, documents the tool-root contract,
    records the artifact and host-report roots, and keeps the required failure
    modes explicit.
- Migration guide and new-project checklist
  - Added
    `.github/skills/pentest-local-orchestrator/references/migration-guide.md`
    with concrete generic-core copy steps, `project-pentest-profile`
    replacement instructions, required `.codex` symlink coverage, minimal
    Codex or GitHub Copilot invocations, bounded language-specific extension
    rules, and offline or tool-cache portability limits.
  - Added
    `.github/skills/pentest-local-orchestrator/references/new-project-checklist.md`
    as the direct adoption artifact for another engineer or agent.
- Z00Z-only invariant isolation
  - `.github/skills/z00z-pentest-profile/references/z00z-invariants.md` now
    contains `Scope Of This File` and `Consumer Notes`, explicitly states that
    the file is Z00Z-only, and warns that another repository must not copy it
    unchanged as its own default.
- Executable documentation contract coverage
  - Added `tests/penetration/test_docs_contracts.py` to prove the generic
    versus Z00Z split, required minimal invocations, required failure modes,
    real local path anchors, and the project-specific invariant warning.

## Validation

Commands and evidence used for `066-14` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `python3 -m unittest tests.penetration.test_docs_contracts`
- `python3 -m unittest discover tests/penetration`
- `bash -n scripts/penetration/*.sh`
- `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py`
- `python3 scripts/penetration/validate_scope.py .security/scope.yaml`
- `bash scripts/penetration/check_pentest_tools.sh --json`
- `test -L .codex/skills && test -L .codex/agents && test -L .codex/prompts && test -L .codex/hooks && test -L .codex/instructions && test -L .codex/requirements && test -L .codex/scripts && test -L .codex/plugins`
- `rg -n "MCP|HexStrike server|Strix runtime|LLM_API_KEY|hydra|john|hashcat|medusa|patator|metasploit|msfvenom|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security`
- `git diff --check -- tools/penetration/README.md .github/skills/pentest-local-orchestrator/references/migration-guide.md .github/skills/pentest-local-orchestrator/references/new-project-checklist.md .github/skills/z00z-pentest-profile/references/z00z-invariants.md tests/penetration/test_docs_contracts.py`

Observed proof points:

- The mandatory `bootstrap_tests.sh` fail-fast gate completed green on the
  final `066-14` tree and ended with `=== BOOTSTRAP COMPLETE ===`.
- `python3 -m unittest tests.penetration.test_docs_contracts` completed green
  twice on the final tree with `5` passing tests per run.
- The broad penetration regression suite completed green twice on the final
  tree with `47` passing tests per run.
- `bash -n scripts/penetration/*.sh` completed green.
- `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py`
  completed green.
- `python3 scripts/penetration/validate_scope.py .security/scope.yaml`
  completed green and confirmed local-only scope with `hosts=2 urls=0`.
- `bash scripts/penetration/check_pentest_tools.sh --json` completed green and
  truthfully recorded the current local tool state under `tools/penetration/`,
  including `present=0`, `missing=12`, and `missing_required=9`.
- The `.codex` compatibility-symlink checks completed green.
- The broad safety grep showed matches only in denylist entries or safety
  rationale text, not in active execution commands.
- The targeted `git diff --check` completed clean on the final docs slice.

`cargo test --release` was not rerun as a separate top-level command for
`066-14` because this slice changed Markdown plus Python tests only; the
mandatory bootstrap gate already reran green on the final tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still could not execute the automated prompt due an
external token-credit ceiling.

- Attempt 1
  - `timeout 60s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-14-PLAN.md current_task="WS-14" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 2
  - `timeout 60s gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-14-PLAN.md current_task="WS-14" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 3
  - `timeout 60s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-14-PLAN.md current_task="WS-14" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83627 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-14-PLAN.md`, the `WS-14` row in `066-TODO.md`, the live
    operator README, the pentest skill surfaces, the prompt surfaces, and the
    Docker notes.
  - Landed the missing generic-versus-Z00Z split in the README, the missing
    migration guide, the missing new-project checklist, and the explicit
    Z00Z-only consumer warning in `z00z-invariants.md`.
- Pass 2
  - Re-ran `test_docs_contracts.py`, the full `tests/penetration` suite,
    scope validation, tool-root truth checks, `.codex` symlink checks, the
    safety grep, and diff hygiene on the updated tree.
  - Result: no new significant issues were found.
- Pass 3
  - Re-ran `test_docs_contracts.py`, the full `tests/penetration` suite, and
    targeted diff hygiene again on the same final tree.
  - Result: no new significant issues were found.

Passes 2 and 3 ended with consecutive clean reruns on the final `066-14` tree.

## Closeout

`066-14` closes `WS-14` by proving that the local pentest orchestration system
now has one operational operator README, one concrete migration guide, one
direct new-project checklist, and one explicitly project-specific Z00Z
invariants note without creating a second authority path or a second
orchestration stack.

Phase `066` is now complete on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
all `066-01` through `066-14` packets are summary-backed complete, and Phase
046 remains paused after `046-04`.
