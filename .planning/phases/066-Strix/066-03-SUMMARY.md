---
phase: 066-Strix
plan: 066-03
status: complete
completed_at: 2026-07-02
next_plan: 066-04
summary_artifact_for: .planning/phases/066-Strix/066-03-PLAN.md
requirements_completed:
  - REQ-005
  - REQ-008
  - REQ-013
  - REQ-014
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-03 Summary: Upstream Reference Capture And Provenance

## 🎯 Outcome

`066-03` is complete.

Phase 066 now has pinned local mirrors for Strix and HexStrike at the exact
commits named by `066-TODO.md`, one real upstream provenance lock under
`tools/penetration/manifests/upstream-sources.lock`, one native upstream
source summary, one Strix routing matrix, one HexStrike tool inventory, and
one reference-only copy of `hexstrike_mcp.py`.

The landed result preserves Strix as a routed source-aware reference corpus
and HexStrike as a curated inventory source without activating either upstream
runtime. No imported Strix directory became an active `.github/skills/*`
entry, and no HexStrike MCP or server flow became a default Phase 066
execution path.

## 📦 Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-03-SUMMARY.md`
- `.github/skills/pentest-local-orchestrator/references/UPSTREAM-SOURCES.md`
- `.github/skills/pentest-local-orchestrator/references/strix/`
- `.github/skills/pentest-tool-installer/references/hexstrike/`
- `tools/penetration/manifests/upstream-sources.lock`
- `tools/penetration/manifests/upstream-sources.lock.bak`
- `tools/penetration/upstream/strix/`
- `tools/penetration/upstream/hexstrike-ai/`

## 🔧 Landed Changes

- Pinned upstream mirrors
  - Cloned `https://github.com/usestrix/strix` into
    `tools/penetration/upstream/strix/` and pinned it to
    `f342808d2b4af551a17e47af744d2f19dee3c443`.
  - Cloned `https://github.com/0x4m4/hexstrike-ai` into
    `tools/penetration/upstream/hexstrike-ai/` and pinned it to
    `9b8c780f324ce5145a322bfa23c98886f8424ba3`.
- Real provenance manifest
  - Replaced the stub `upstream-sources.lock` with a real lock packet that
    records repo URLs, commits, retrieval time, license evidence, consulted
    upstream paths, curated copied paths, mirror-only exclusions, and the
    historical note that `.planning/phases/066-Strix/strix.md` is absent and
    must not be recreated as a second authority.
  - Created `upstream-sources.lock.bak` before the full manifest rewrite.
- Curated Strix reference corpus
  - Copied `source_aware_whitebox.md`.
  - Copied all upstream `scan_modes/`, `frameworks/`, `protocols/`,
    `technologies/`, and `vulnerabilities/` Markdown files into
    `.github/skills/pentest-local-orchestrator/references/strix/`.
  - Added provenance headers to every copied Strix file with source repo,
    source path, commit, license, and `reference-only` disposition.
  - Added `routing-matrix.md` so the active Phase 066 skill surface can route
    into the copied Strix corpus without creating one active skill per
    upstream playbook.
- Curated HexStrike reference surfaces
  - Added `tool-inventory.md` that maps HexStrike's tool taxonomy into the
    Phase 066 canonical local allowlist, optional-lab set, and deny set.
  - Added `hexstrike_mcp_reference_only.py` with the required
    `REFERENCE ONLY - DO NOT RUN` header.
  - Kept `hexstrike_server.py` in the upstream mirror only.
- Runtime exclusion preserved
  - Recorded Strix runtime, Docker, provider, `LLM_API_KEY`, Caido automation,
    `agent_browser`, `sqlmap`, and `subfinder` as mirror-only or
    reference-only surfaces unless a separate local lab profile is added.
  - Recorded HexStrike FastMCP, Flask server, arbitrary command wrappers,
    file-operation wrappers, and exploit or credential-attack flows as
    excluded from the default active path.

## ✅ Validation

Commands and evidence used for `066-03` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `git -C tools/penetration/upstream/strix rev-parse HEAD`
- `git -C tools/penetration/upstream/hexstrike-ai rev-parse HEAD`
- `rg -n "f342808d2b4af551a17e47af744d2f19dee3c443|9b8c780f324ce5145a322bfa23c98886f8424ba3" tools/penetration/manifests/upstream-sources.lock`
- `rg -n "REFERENCE ONLY - DO NOT RUN" .github/skills/pentest-*`
- `rg -n "Apache License|MIT License" .github/skills/pentest-* tools/penetration/manifests/upstream-sources.lock`
- `rg -n "frameworks|protocols|technologies|vulnerabilities|scan_modes" .github/skills/pentest-local-orchestrator/references/strix/routing-matrix.md`
- `diff -u <(find tools/penetration/upstream/strix/strix/skills/<group> -maxdepth 1 -type f -name '*.md' -printf '%f\n' | sort) <(find .github/skills/pentest-local-orchestrator/references/strix/<group> -maxdepth 1 -type f -name '*.md' -printf '%f\n' | sort)` for `scan_modes`, `frameworks`, `protocols`, `technologies`, and `vulnerabilities`
- `find .github/skills -maxdepth 1 -mindepth 1 -type d -printf '%f\n' | rg "^(strix|hexstrike-ai|hexstrike|coordination|frameworks|protocols|technologies|vulnerabilities|scan_modes)$"`
- `rg -n "hexstrike_server\\.py|hexstrike_mcp\\.py|FastMCP|LLM_API_KEY|openai-agents\\[litellm\\]" .github/skills/pentest-*/SKILL.md scripts/penetration tools/penetration/bin`
- `git diff --check -- .github/skills/pentest-local-orchestrator/references .github/skills/pentest-tool-installer/references tools/penetration/manifests/upstream-sources.lock`
- `rg -n "FIXME|panic!\\(|unimplemented!\\(|todo!\\(" .github/skills/pentest-local-orchestrator/references .github/skills/pentest-tool-installer/references tools/penetration/manifests/upstream-sources.lock`
- `rg -n "\\bTODO\\b" .github/skills/pentest-local-orchestrator/references .github/skills/pentest-tool-installer/references tools/penetration/manifests/upstream-sources.lock | rg -v "066-TODO\\.md"`

Observed proof points:

- The mandatory `bootstrap_tests.sh` fail-fast gate completed green before the
  plan closeout proceeded.
- Both pinned commits are recorded in `upstream-sources.lock`.
- The copied Strix reference set has file-for-file parity with the upstream
  `scan_modes`, `frameworks`, `protocols`, `technologies`, and
  `vulnerabilities` directories, plus the required
  `source_aware_whitebox.md`.
- The copied Strix files all carry explicit Apache License 2.0 provenance
  headers.
- `hexstrike_mcp_reference_only.py` carries the required
  `REFERENCE ONLY - DO NOT RUN` header.
- No active `.github/skills/*` directory name was introduced for imported
  Strix or HexStrike runtime material.
- No active skill or script surface references HexStrike MCP or server startup,
  Strix runtime, external LLM API keys, or imported runtime dependencies.
- Diff hygiene checks were clean.

`cargo test --release` was not rerun for `066-03` because this plan changed
provenance mirrors, manifests, and reference docs only; it did not change Rust
or test-affecting runtime code.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime did not provide a usable automated prompt-execution path
for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-03-PLAN.md current_task="Upstream Reference Capture And Provenance" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-03-PLAN.md current_task="Upstream Reference Capture And Provenance" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-03-PLAN.md current_task="Upstream Reference Capture And Provenance" --yolo'`
  - Result: exited with code `1` and reported `402 Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-03-PLAN.md`, `066-TODO.md`, the pinned upstream `README.md`,
    `LICENSE`, `pyproject.toml`, `requirements.txt`, `hexstrike_mcp.py`,
    `hexstrike_server.py`, and the new provenance docs.
  - Checked that every required Strix category file was captured and that
    risky tooling surfaces stayed mirror-only or reference-only.
  - Result: no material drift found.
- Pass 2
  - Re-ran commit or license or reference-only or routing-matrix greps, the
    category parity diffs, and the negative directory-name audit.
  - Result: clean for the `066-03` scope.
- Pass 3
  - Re-ran active-surface exclusion grep, placeholder grep, refined
    `TODO`-name grep, and diff hygiene checks.
  - Result: clean for the `066-03` scope. No placeholder or parallel-runtime
    drift remained.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
patch.

## 🧾 Closeout

`066-03` closes `WS-03` by landing the pinned upstream mirrors, the real
license-aware provenance lock, the routed Strix reference corpus, and the
HexStrike inventory or reference-only capture without importing either
upstream runtime as an active default execution path.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope,
and the next execution lane is `066-04-PLAN.md`.
