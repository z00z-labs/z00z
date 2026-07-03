---
phase: 066-Strix
plan: 066-04
status: complete
completed_at: 2026-07-02
next_plan: 066-05
summary_artifact_for: .planning/phases/066-Strix/066-04-PLAN.md
requirements_completed:
  - REQ-003
  - REQ-011
  - REQ-014
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-04 Summary: Generic Skill Family

## 🎯 Outcome

`066-04` is complete.

Phase 066 now has seven live `pentest-*` skill roots with concrete `SKILL.md`
contracts, one intentionally small active skill surface, one canonical
script-path contract per pentest lane, and explicit fail-closed rules for the
scripts that later plans still need to implement.

The landed result keeps the active skill layer generic by default and routes
everything through one canonical local path: `scripts/penetration/*` for
execution, `tools/penetration/` for tooling, `.security-artifacts/<timestamp>/`
for machine-readable outputs, and
`reports/z00z-pentests_report-<timestamp>/` for host-facing pentest exports.

## 📦 Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-04-SUMMARY.md`
- `.github/skills/pentest-local-orchestrator/SKILL.md`
- `.github/skills/pentest-source-aware-sast/SKILL.md`
- `.github/skills/pentest-rust-security/SKILL.md`
- `.github/skills/pentest-secrets-supply-chain/SKILL.md`
- `.github/skills/pentest-local-dast/SKILL.md`
- `.github/skills/pentest-report/SKILL.md`
- `.github/skills/pentest-tool-installer/SKILL.md`

## 🔧 Landed Changes

- Generic skill family
  - Added `pentest-local-orchestrator`, `pentest-source-aware-sast`,
    `pentest-rust-security`, `pentest-secrets-supply-chain`,
    `pentest-local-dast`, and `pentest-report` as new repository-local skill
    roots.
  - Updated `pentest-tool-installer` to align with the same path, artifact,
    and fail-closed conventions.
- Canonical script-path contract
  - Every skill now calls one explicit `scripts/penetration/*` entrypoint
    instead of embedding long fallback command blocks.
  - Every skill now states that missing script paths are a stop condition,
    not a reason to create an alternate runtime path.
- Canonical artifact contract
  - The skill family now points to one shared machine-readable artifact root:
    `.security-artifacts/<timestamp>/`.
  - The reporting lane now points to one shared host export root:
    `reports/z00z-pentests_report-<timestamp>/`.
- Safety and evidence rules
  - Every lane now states the default no-MCP, no external API key, and no
    public-target boundary explicitly.
  - `pentest-report` and the execution lanes now make evidence mapping
    explicit and reject raw scanner output as a final finding.
- Intentionally small active surface
  - The active `pentest-*` directory count is exactly `7`, keeping Strix
    framework, protocol, technology, vulnerability, and scan-mode material in
    routed reference folders rather than inflating the active skill surface.

## ✅ Validation

Commands and evidence used for `066-04` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `find .github/skills/pentest-* -maxdepth 2 -name SKILL.md | wc -l`
- `find .github/skills -maxdepth 1 -name 'pentest-*' -type d | wc -l`
- `rg -n "MCP|API key|public target|scanner output" .github/skills/pentest-*`
- `rg -n "tools/penetration|scripts/penetration|.security-artifacts" .github/skills/pentest-*`
- `rg -n "run_local_pentest\\.sh|run_source_sast\\.sh|run_rust_security\\.sh|run_secrets_supply_chain\\.sh|run_local_dast\\.sh|build_pentest_report\\.py|validate_artifacts\\.py|install_pentest_tools\\.sh|check_pentest_tools\\.sh" .github/skills/pentest-*/SKILL.md`
- `rg -n "evidence mapping|deduplicate|final finding|Do not replace|Do not swap" .github/skills/pentest-*/SKILL.md`
- `git diff --check -- .github/skills/pentest-local-orchestrator/SKILL.md .github/skills/pentest-source-aware-sast/SKILL.md .github/skills/pentest-rust-security/SKILL.md .github/skills/pentest-secrets-supply-chain/SKILL.md .github/skills/pentest-local-dast/SKILL.md .github/skills/pentest-report/SKILL.md .github/skills/pentest-tool-installer/SKILL.md .planning/phases/066-Strix/066-03-SUMMARY.md .planning/ROADMAP.md .planning/STATE.md`
- `rg -n "FIXME|panic!\\(|unimplemented!\\(|todo!\\(" .github/skills/pentest-*/SKILL.md`

Observed proof points:

- The mandatory `bootstrap_tests.sh` gate completed green before `066-04`
  validation started and completed green again after the skill edits.
- Exactly `7` `SKILL.md` files exist under `pentest-*`, matching the planned
  generic family size.
- Exactly `7` active `pentest-*` directories exist at the top skill level,
  proving the active surface stayed intentionally small.
- Every lane points to canonical `scripts/penetration/*` paths and the shared
  `.security-artifacts/<timestamp>/` root.
- Evidence-mapping language is present in the reporting and execution lanes.
- Diff hygiene checks were clean.

`cargo test --release` was not rerun for `066-04` because this plan changed
skill contracts only; it did not change Rust or test-affecting runtime code.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime did not provide a usable automated prompt-execution path
for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-04-PLAN.md current_task="Generic Skill Family" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-04-PLAN.md current_task="Generic Skill Family" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-04-PLAN.md current_task="Generic Skill Family" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-04-PLAN.md`, `066-TODO.md`, `066-CONTEXT.md`,
    `pentest-tool-installer/SKILL.md`, `attack-surfaces-create/SKILL.md`, and
    `smart-tests-bootstrap/SKILL.md`.
  - Checked that each generic lane names one canonical script path, one
    artifact root, and one fail-closed rule.
  - Result: no material drift found.
- Pass 2
  - Re-ran skill-count, directory-count, safety-string, path-string, and
    script-reference greps.
  - Result: clean for the `066-04` scope.
- Pass 3
  - Re-ran evidence-mapping grep, diff hygiene checks, and placeholder grep.
  - Result: clean for the `066-04` scope. No alternate runtime path or
    placeholder-only skill surface remained.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## 🧾 Closeout

`066-04` closes `WS-04` by landing the seven generic `pentest-*` skill
contracts, one intentionally small active skill family, one canonical script
entrypoint map, one shared artifact-root contract, and fail-closed behavior
for the scripts that later plans still need to implement.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-05-PLAN.md`.
