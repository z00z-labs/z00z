---
phase: 066-Strix
plan: 066-13
status: complete
completed_at: 2026-07-03
next_plan: 066-14
summary_artifact_for: .planning/phases/066-Strix/066-13-PLAN.md
requirements_completed:
  - REQ-003
  - REQ-011
  - REQ-012
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-13 Summary: Z00Z Security Execution Prompts

## Outcome

`066-13` is complete.

Phase 066 now has a prompt surface that routes the live local pentest workflow
through one canonical human or agent entrypoint, one bounded Z00Z adapter path,
one wait-for-all parallel merge contract, and one fail-closed report
doublecheck contract. The prompts stay repository-local, require evidence
before confirmation, and do not introduce an MCP path, a Strix runtime, or a
parallel security stack.

The main drift was removed from `pentest-local.prompt.md`: the external
workflow now points to `./z00z_penetration_tests.sh` instead of presenting
`scripts/penetration/run_local_pentest.sh` as a second user-facing path. The
internal runner remains documented only as the substrate behind that canonical
entrypoint.

`WS-13` now also has a dedicated prompt-contract regression suite and the
missing `tests/penetration/fixtures/{prompts,profile}/` subtrees.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-13-SUMMARY.md`
- `.github/prompts/pentest-local.prompt.md`
- `.github/prompts/pentest-parallel-review.prompt.md`
- `.github/prompts/pentest-report-doublecheck.prompt.md`
- `.github/prompts/pentest-local-z00z.prompt.md`
- `tests/penetration/fixtures/profile/z00z_dry_run_expected.json`
- `tests/penetration/fixtures/prompts/prompt_contract_expected.json`
- `tests/penetration/test_codex_surface_integration.py`
- `tests/penetration/test_prompt_contracts.py`

## Landed Changes

- Canonical entrypoint and generic prompt path
  - `pentest-local.prompt.md` now names
    `./z00z_penetration_tests.sh` as the only external human or agent
    entrypoint.
  - The prompt still references `scripts/penetration/run_local_pentest.sh`,
    but only as the internal substrate invoked by the top-level entrypoint,
    which removes the second user-facing path drift.
  - The generic and Z00Z examples now both route through the same top-level
    command and return `entrypoint`, `artifact_dir`, `report_dir`, and scope
    metadata.
- Parallel merge and report doublecheck contract
  - `pentest-parallel-review.prompt.md` now contains the literal
    lowercase `wait for all` wording needed by the acceptance greps and
    explicitly routes merged candidate findings into
    `pentest-report-doublecheck` before any finding is treated as confirmed.
  - `pentest-report-doublecheck.prompt.md` now requires both artifact-side and
    host-side report metadata, rejects scanner-only promotion to confirmed, and
    keeps unsupported claims fail-closed as `unconfirmed` or `false-positive`.
- Z00Z adapter routing
  - `pentest-local-z00z.prompt.md` now explicitly requires routing through
    `z00z-pentest-profile` before any Z00Z-specific dry run or execution.
  - The prompt now includes a canonical Z00Z run example that reuses
    `./z00z_penetration_tests.sh --profile z00z` and keeps heavy closeout on
    `gsd-audit-4.prompt.md`.
- Prompt regression coverage
  - Added `tests/penetration/test_prompt_contracts.py` as the dedicated prompt
    contract suite for `TT-066-07` or `U-066-06`.
  - Added `tests/penetration/fixtures/prompts/prompt_contract_expected.json`
    and `tests/penetration/fixtures/profile/z00z_dry_run_expected.json`.
  - Extended `test_codex_surface_integration.py` so the prompt exposure and
    `.codex` surface suite also proves the canonical entrypoint and updated
    fail-closed wording.

## Validation

Commands and evidence used for `066-13` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `python3 -m unittest tests.penetration.test_codex_surface_integration`
- `python3 -m unittest tests.penetration.test_prompt_contracts tests.penetration.test_codex_surface_integration`
- `python3 -m unittest discover tests/penetration`
- `rg -n 'wait for all|dedupe|deduplicate|validated before confirmation|scanner findings|scanner-only hypothesis' .github/prompts/pentest-*.prompt.md`
- `rg -n './z00z_penetration_tests.sh|z00z-pentest-profile|attack-surfaces-create|z00z-crypto-auditor|gsd-audit-4.prompt.md' .github/prompts/pentest-*.prompt.md`
- `if rg -n 'MCP default|HexStrike MCP|Strix runtime' .github/prompts/pentest-*.prompt.md; then exit 1; fi`
- `git diff --check -- .github/prompts/pentest-local.prompt.md .github/prompts/pentest-parallel-review.prompt.md .github/prompts/pentest-report-doublecheck.prompt.md .github/prompts/pentest-local-z00z.prompt.md tests/penetration/test_codex_surface_integration.py tests/penetration/test_prompt_contracts.py`

Observed proof points:

- The mandatory `bootstrap_tests.sh` fail-fast gate completed green on the
  final `066-13` tree.
- The prompt-specific suites completed green:
  - `python3 -m unittest tests.penetration.test_codex_surface_integration`
    passed with `7` tests.
  - `python3 -m unittest tests.penetration.test_prompt_contracts tests.penetration.test_codex_surface_integration`
    passed with `12` tests.
- The broad penetration regression suite completed green on the final tree with
  `42` passing tests.
- The positive prompt grep checks now show the required `wait for all`,
  dedupe, evidence-validation, scanner-only rejection, canonical entrypoint,
  and Z00Z route fragments.
- The negative grep check for `MCP default`, `HexStrike MCP`, and `Strix runtime`
  completed clean.
- The phase-local fixture inventory now includes the previously missing
  `tests/penetration/fixtures/prompts/` and
  `tests/penetration/fixtures/profile/` subtrees.

`cargo test --release` was not rerun as a separate top-level command for
`066-13` because this slice changed prompt Markdown plus Python tests only; the
mandatory bootstrap gate already reran green on the final tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still could not execute the automated prompt due an
external token-credit ceiling.

- Attempt 1
  - `gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-13-PLAN.md current_task="WS-13" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 2
  - `gsd --no-session -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-13-PLAN.md current_task="WS-13" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 3
  - `gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-13-PLAN.md current_task="WS-13" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83627 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-13-PLAN.md`, the `WS-13` rows in `066-TODO.md` and
    `066-CONTEXT.md`, the existing prompt files, and the live `.codex` prompt
    exposure tests.
  - Landed the canonical entrypoint correction, the explicit
    `z00z-pentest-profile` route, the host-report-metadata check, the
    scanner-only fail-closed wording, and the lowercase `wait for all`
    acceptance phrase.
- Pass 2
  - Re-read the final prompt files against `TT-066-07` and `U-066-06`, noticed
    that prompt contracts were still only indirectly tested, and landed the
    dedicated `test_prompt_contracts.py` suite plus prompt or profile fixtures.
- Pass 3
  - Re-ran the prompt-specific tests, the broad `tests/penetration` suite, and
    the positive or negative grep checks on the final tree.
  - Result: no new significant issues were found.

Passes 2 and 3 ended with consecutive clean reruns on the final `066-13` tree.

## Closeout

`066-13` closes `WS-13` by proving that the Phase 066 execution prompts now
share one canonical external entrypoint, one evidence-before-confirmation
policy, one wait-for-all merge contract, and one Z00Z-specific routing surface
without adding an MCP path, a Strix runtime path, or a second audit stack.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-14-PLAN.md`.
