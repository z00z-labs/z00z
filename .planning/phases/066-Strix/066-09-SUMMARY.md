---
phase: 066-Strix
plan: 066-09
status: complete
completed_at: 2026-07-02
next_plan: 066-10
summary_artifact_for: .planning/phases/066-Strix/066-09-PLAN.md
requirements_completed:
  - REQ-003
  - REQ-004
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-09 Summary: Codex And Copilot Surface Wiring

## Outcome

`066-09` is complete.

Phase 066 now exposes the local pentest prompt and specialist-review surface
through one canonical `.github/*` path while preserving the existing `.codex/*`
symlink compatibility model. The lane lands the five bounded pentest review
agents, the three generic prompt contracts required by `WS-09` and `WS-13`,
and regression coverage that proves the symlink targets, bounded agent
behavior, prompt fragments, negative runtime exclusions, and fixture-backed
parallel-merge rules.

The closeout keeps one authority path only: canonical files live under
`.github/*`, `.codex/*` remains a compatibility surface, and the shared prompt
semantics stay aligned with the Phase 066 prompt contract instead of creating a
second orchestration layer.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-09-SUMMARY.md`
- `.github/agents/pentest-rust-reviewer.agent.md`
- `.github/agents/pentest-crypto-reviewer.agent.md`
- `.github/agents/pentest-storage-reviewer.agent.md`
- `.github/agents/pentest-rpc-dast-reviewer.agent.md`
- `.github/agents/pentest-supply-chain-reviewer.agent.md`
- `.github/prompts/pentest-local.prompt.md`
- `.github/prompts/pentest-parallel-review.prompt.md`
- `.github/prompts/pentest-report-doublecheck.prompt.md`
- `.github/prompts/pentest-local-z00z.prompt.md`
- `tests/penetration/test_codex_surface_integration.py`
- `tests/penetration/fixtures/parallel_merge_findings_a.json`
- `tests/penetration/fixtures/parallel_merge_findings_b.json`
- `tests/penetration/fixtures/z00z_lane_map_expected.json`

## Landed Changes

- Canonical `.github` surface wiring
  - Added the five required `pentest-*.agent.md` reviewer files under
    `.github/agents/`.
  - Added the generic prompt contracts
    `pentest-local.prompt.md`, `pentest-parallel-review.prompt.md`, and
    `pentest-report-doublecheck.prompt.md` under `.github/prompts/`.
  - Kept `.github/prompts/pentest-local-z00z.prompt.md` aligned with the
    required Z00Z lane map and canonical audit-routing wording.
- Bounded review-agent contracts
  - Every new pentest reviewer is read-only, bounded to one evidence surface,
    defines an explicit output contract, and states that it must not execute
    tools unless the orchestrator explicitly asks for it.
- One-path prompt semantics
  - The generic local prompt documents both generic-repo and Z00Z profile
    examples on the same `run_local_pentest.sh` runner.
  - The parallel-review prompt defines wait-for-all behavior plus dedupe by
    `path`, `line`, `rule_id`, and `evidence_anchor`.
  - The report-doublecheck prompt verifies claims against local artifacts and
    source files before allowing confirmation.
- Regression coverage
  - Added `tests/penetration/test_codex_surface_integration.py`.
  - Added merge and lane-map fixtures that prove prompt contracts without
    inventing a second runtime path.

## Validation

Commands and evidence used for `066-09` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `python3 -m unittest tests.penetration.test_codex_surface_integration`
- `test -L .codex/skills && test "$(readlink .codex/skills)" = "../.github/skills" && test -L .codex/agents && test "$(readlink .codex/agents)" = "../.github/agents" && test -L .codex/prompts && test "$(readlink .codex/prompts)" = "../.github/prompts" && test -L .codex/hooks && test -L .codex/instructions && test -L .codex/requirements && test -L .codex/scripts && test -L .codex/plugins`
- `find .github/agents -maxdepth 1 -name 'pentest-*.agent.md' | wc -l`
- prompt fragment assertions for the four pentest prompt files
- negative prompt assertions for `HexStrike server`, `Strix runtime`,
  `LLM_API_KEY`, and `run tools directly by default`
- ASCII hygiene assertion across the new agent files and
  `pentest-local-z00z.prompt.md`
- `git diff --check`

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green before work
  continued, and it completed green again after the final `066-09` edits.
- The new integration suite passed green and proved all six codex-surface
  checks: symlink targets, bounded agent files, required prompt fragments,
  forbidden generic-runtime strings, Z00Z lane-map coverage, and prompt-level
  merge deduplication.
- The symlink checks proved that `.codex/skills`, `.codex/agents`, and
  `.codex/prompts` still point at `../.github/*` and that the remaining
  compatibility surfaces stay symlinked.
- The prompt grep checks proved that generic and Z00Z examples, wait-for-all
  merge rules, and report doublecheck inputs are present on the canonical
  prompt path.
- The diff hygiene check was clean.

`cargo test --release` was not rerun as a separate top-level command for
`066-09` because no Rust runtime code changed; the mandatory bootstrap gate
already reran green on release-mode workspace targets after the final edits.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime still did not provide a usable automated prompt-execution
path for this slice.

- Attempt 1
  - `./.github/prompts/gsd-review-tasks-execution.prompt.md current_spec=.planning/phases/066-Strix/066-09-PLAN.md current_task="WS-09" --yolo`
  - Result: failed with `Permission denied` because the prompt file is not an
    executable entrypoint.
- Attempt 2
  - `timeout 60s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-09-PLAN.md current_task="WS-09" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 66668 > 38936`
- Attempt 3
  - `timeout 60s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-09-PLAN.md current_task="WS-09" --yolo'`
  - Result: exited with code `1` and reported
    `402 Prompt tokens limit exceeded: 83627 > 38936`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-09-PLAN.md`, the `WS-09` and shared `WS-13` prompt rows in
    `066-TODO.md`, the created agent files, the created prompt files, and the
    codex-surface integration test.
  - Found and fixed two in-scope issues: a case-sensitive test fragment mismatch
    in `test_codex_surface_integration.py` and non-ASCII section headings in
    `pentest-local-z00z.prompt.md`.
- Pass 2
  - Re-ran the codex-surface integration suite, symlink assertions, prompt
    fragment or negative assertions, agent-count checks, ASCII hygiene, and
    diff hygiene.
  - Result: clean for the `066-09` scope.
- Pass 3
  - Re-ran the mandatory bootstrap gate after the final patch, then re-checked
    the `WS-09` and `WS-13` contract lines against the landed prompt or agent
    files and fixture-backed tests.
  - Result: clean for the `066-09` scope.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## Closeout

`066-09` closes `WS-09` by landing the canonical pentest agent surfaces, the
generic prompt surfaces, and executable codex-surface regression coverage while
preserving the existing `.codex` compatibility symlinks and avoiding a second
authority path.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-10-PLAN.md`.
