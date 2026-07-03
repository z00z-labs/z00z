---
phase: 066-Strix
plan: 066-02
status: complete
completed_at: 2026-07-02
next_plan: 066-03
summary_artifact_for: .planning/phases/066-Strix/066-02-PLAN.md
requirements_completed:
  - REQ-002
  - REQ-010
  - REQ-012
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 066-02 Summary: Tool Root And Installation Model

## 🎯 Outcome

`066-02` is complete.

Phase 066 now has one canonical pentest tool root under `tools/penetration/`,
one installer that creates the local tool-home layout and repository-local
wrappers, and one checker that emits truthful machine-readable status instead
of pretending that missing tool payloads are already installed.

The landed model does not touch `tools/formal_verification/`. It keeps the
default pentest execution surface inside `tools/penetration/` and records when
only wrapper paths exist or when no local payload exists yet.

## 📦 Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/066-Strix/066-02-SUMMARY.md`
- `.github/skills/pentest-tool-installer/SKILL.md`
- `scripts/penetration/check_pentest_tools.sh`
- `scripts/penetration/install_pentest_tools.sh`
- `tools/penetration/README.md`
- `tools/penetration/manifests/checksums.sha256`
- `tools/penetration/manifests/tool-status.json`
- `tools/penetration/manifests/tool-versions.lock`
- `tools/penetration/manifests/upstream-sources.lock`
- `tools/penetration/bin/*`

## 🔧 Landed Changes

- Canonical root contract
  - Added `tools/penetration/README.md` to define the local root layout,
    environment contract, and ownership boundary against
    `tools/formal_verification/`.
- Local tool-home model
  - Added installer support for the required local homes: `bin/`, `cargo/`,
    `go/`, `python/`, `cache/`, `rules/`, `templates/`, `wordlists/`,
    `upstream/`, and `manifests/`.
  - Bound `CARGO_HOME`, `GOBIN`, `GOMODCACHE`, `UV_TOOL_DIR`,
    `UV_TOOL_BIN_DIR`, `PIPX_HOME`, `PIPX_BIN_DIR`, `TRIVY_CACHE_DIR`, and
    `NUCLEI_TEMPLATES_DIR` under `tools/penetration/`.
- Repository-local wrappers
  - Added wrapper entrypoints under `tools/penetration/bin/` for the default
    allowlisted tool set.
  - Wrappers fail closed when the corresponding local payload is absent instead
    of silently delegating to unrelated global tool homes.
- Truthful checker and manifests
  - Added `scripts/penetration/check_pentest_tools.sh`.
  - The checker writes `tools/penetration/manifests/tool-status.json` and
    refreshes `tool-versions.lock` from actual local payload and wrapper
    discovery.
  - The checker exits successfully in baseline mode even when tools are
    missing, but `--strict` fails closed on missing required local payloads.
- Safe manifest refresh
  - Added safe overwrite behavior with `.bak` backups for manifest rewrites.
  - Added `checksums.sha256` generation for the manifest set.
- Skill surface
  - Added `.github/skills/pentest-tool-installer/SKILL.md` that calls the
    installer and checker by path and keeps the tool-root contract explicit.

## ✅ Validation

Commands and evidence used for `066-02` closeout:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `bash -n scripts/penetration/install_pentest_tools.sh scripts/penetration/check_pentest_tools.sh`
- `bash scripts/penetration/install_pentest_tools.sh --json`
- `bash scripts/penetration/check_pentest_tools.sh --json`
- `bash scripts/penetration/check_pentest_tools.sh --json --strict`
- `find tools/penetration -maxdepth 3 \( -type f -o -type l \) | sort`
- `PATH="$PWD/tools/penetration/bin:$PATH" command -v semgrep`
- `python3 -m json.tool tools/penetration/manifests/tool-status.json`
- `sed -n '1,220p' tools/penetration/manifests/tool-versions.lock`
- `rg -n "tools/formal_verification|Z00Z_VERIFY_TOOLS_DIR" scripts/penetration/install_pentest_tools.sh scripts/penetration/check_pentest_tools.sh`
- `git diff --check -- tools/penetration scripts/penetration .github/skills/pentest-tool-installer/SKILL.md .planning/STATE.md .planning/ROADMAP.md`
- `rg -n "TODO|FIXME|panic!\\(|unimplemented!\\(|todo!\\(" tools/penetration scripts/penetration .github/skills/pentest-tool-installer/SKILL.md`

Observed proof points:

- The Phase 066 fail-fast bootstrap gate was green before the `066-02`
  implementation pass started.
- The installer creates the local wrapper and manifest state under
  `tools/penetration/`.
- `check_pentest_tools.sh --json` exits successfully and records missing local
  payloads explicitly.
- `check_pentest_tools.sh --json --strict` exits non-zero because required
  local payloads are still absent, proving that baseline success is not
  concealed as all-green.
- `command -v semgrep` resolves to `tools/penetration/bin/semgrep` when the
  local wrapper directory is prepended to `PATH`.
- No pentest installer or checker path references `tools/formal_verification/`
  or verification-tool env names.

## 🔍 Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times, but
the current runtime did not provide a usable automated prompt-execution path
for this slice.

- Attempt 1
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-02-PLAN.md current_task="Tool Root And Installation Model" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 2
  - `timeout 45s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-02-PLAN.md current_task="Tool Root And Installation Model" --yolo'`
  - Result: exited with code `1` and reported `Prompt tokens limit exceeded`
- Attempt 3
  - `timeout 45s gsd --no-session --extension .github -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/066-Strix/066-02-PLAN.md current_task="Tool Root And Installation Model" --yolo'`
  - Result: exited with code `1` and reported `Prompt tokens limit exceeded`

Equivalent workspace-first review was executed manually against the same scope.

- Pass 1
  - Re-read `066-02-PLAN.md`, `066-TODO.md`, `scripts/install-verification-tools.sh`,
    `pack_z00z_project.sh`, and `.github/copilot-instructions.md`.
  - Reviewed the installer or checker env contract, wrapper model, manifest
    outputs, and the prohibition against `tools/formal_verification/` reuse.
  - Result: found one wrapper error-message drift that referred to the
    generator environment instead of the runtime local root, then fixed it in
    scope.
- Pass 2
  - Re-ran shell syntax checks, installer and checker JSON output, wrapper
    discovery, manifest inspection, path audits, strict negative mode, JSON
    parsing, placeholder grep, and diff hygiene checks.
  - Result: clean for the `066-02` scope after the fix.
- Pass 3
  - Re-ran the installer once more to refresh wrapper content and repeated the
    acceptance probes against the final filesystem state.
  - Result: clean for the `066-02` scope. No remaining material drift or fake
    status behavior was found.

Passes 2 and 3 were consecutive clean review runs after the final in-scope
fix.

## 🧾 Closeout

`066-02` closes `WS-02` by landing the canonical `tools/penetration/` root
contract, repository-local wrappers, truthful status manifests, and the
installer skill surface without reusing or polluting the formal-verification
tool root.

Phase `066` remains in progress on the existing `.planning/phases/066-Strix/`
folder only. `066-TODO.md` remains the normative authority, future-only and
target-design wording in the Phase 066 corpus remains live mandatory scope, and
the next execution lane is `066-03-PLAN.md`.
