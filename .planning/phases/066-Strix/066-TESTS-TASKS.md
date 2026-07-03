---
phase: 066-Strix
artifact: tests-tasks
status: implemented
source: 066-TEST-SPEC.md
updated: 2026-07-03
---

# Phase 066 Test Tasks

**Phase:** `066`
**Status:** Implemented and validated against live repository anchors
**Primary Output:** `066-TEST-SPEC.md`

## Execution Rules

- Treat `066-TODO.md`, `066-CONTEXT.md`, `066-COVERAGE.md`, and
  `066-01-PLAN.md` through `066-14-PLAN.md` as normative.
- Do not implement test code against imaginary workflows or alternate file
  trees.
- Do not create a second orchestration layer for tests.
- If an implementation detail changes, update `066-TEST-SPEC.md` before
  adjusting tests.
- If a test discovers a genuine implementation bug, record the bug and stop
  changing tests to hide it.

## Scope Inputs

- `066-TEST-SPEC.md`
- `066-CONTEXT.md`
- `066-TODO.md`
- `066-COVERAGE.md`
- `066-01-PLAN.md` through `066-14-PLAN.md`
- live repository anchors under `.codex/`, `scripts/`, `crates/*/tests`, and
  `.github/*`

## Execution Strategy

- This phase now has a verification-backed local test surface.
- The first execution gate was "confirm the Phase 066 runtime seams exist and
  match the plan"; that gate is now closed.
- Reuse adjacent Z00Z invariant tests where they already prove HJMT,
  publication-binding, domain-separation, or doc-drift contracts.
- Phase 066-specific test files exist only on the canonical entrypoints,
  scripts, skills, prompts, and Docker wrapper surfaces.
- Do not generate placeholder tests that only assert file absence or planned
  names.

## Implementation Snapshot

- `tests/penetration/test_local_runner_integration.py` now closes the
  previously missing local runner and static-orchestration coverage.
- `tests/penetration/test_profile_routing.py` now closes the previously
  missing Z00Z profile routing and dry-run evidence contract coverage.
- `TT-066-01` through `TT-066-09` are implemented on the live tree.
- `TT-066-10` has a current verification snapshot:
  - `python3 -m unittest discover tests/penetration` passed (`64` tests)
  - `python3 -m pytest tests/penetration` is currently blocked in this
    workspace (`No module named pytest`)
  - `bash -n scripts/penetration/*.sh tools/penetration/docker/run_pentest_container.sh z00z_penetration_tests.sh pack_z00z_project.sh unpack_z00z_project.sh` passed
  - `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py` passed
  - `python3 scripts/penetration/validate_scope.py .security/scope.yaml` passed
  - `bash scripts/penetration/check_pentest_tools.sh --json --strict` passed
    (`present=13`, `missing=1`, `broken=1`, `missing_required=0`;
    all required tools are locally installed, `cargo-geiger` remains optional
    and missing, and the current `sg` payload is isolated as optional-broken)
  - `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration` passed
  - `.codex/{skills,agents,prompts,hooks,instructions,requirements,scripts,plugins}`
    symlink checks passed
  - grep-based denied-tool and runtime guard audits returned only expected
    policy/reference hits
  - `./pack_z00z_project.sh --output /tmp/z00z-pentest-portable.tar.gz`
    passed
  - `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-portable.tar.gz --mode quick --static-only --check-only`
    passed
  - `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-portable.tar.gz --mode check-only`
    passed
- The Docker archive path is no longer hypothetical in this host snapshot;
  only the `pytest` row remains environment-gated until the module is
  installed.

## Task Breakdown

| Task ID | Deliverable | Depends On | Done When |
| --- | --- | --- | --- |
| `TT-066-01` | Create the proposed top-level `tests/penetration/fixtures/` tree and document fixture ownership | none | Fixture directories exist for scope, tool status, scanner outputs, reports, prompts, and profile dry runs; Docker expectations are asserted on live wrapper surfaces instead of a placeholder fixture tree |
| `TT-066-02` | Implement `test_scope_validation.py` | `TT-066-01` | Localhost-only scope passes; public URL, public IP, wildcard host, broad CIDR, denied tool, and empty DAST scope fail |
| `TT-066-03` | Implement `test_tool_manifest.py` and checker contract assertions | `TT-066-01` | Tool root stays under `tools/penetration/`; missing tools are recorded truthfully; no formal-verification tool root is used |
| `TT-066-04` | Implement `test_artifact_schema.py` and `test_report_builder.py` | `TT-066-01` | Artifact tree shape, report pairing, evidence mapping, redaction, and finding classification are proven |
| `TT-066-05` | Implement runner integration suite for `run_local_pentest.sh` and `run_parallel_static.sh` | `TT-066-01`, `TT-066-02`, `TT-066-04` | Shared timestamp, artifact roots, report roots, missing-tool behavior, child exit preservation, and CLI flags `--mode`, `--scope`, `--no-dast`, `--static-only`, `--profile`, and `--artifact-dir` are proven |
| `TT-066-06` | Implement DAST integration suite for allowed-target, no-target, and forbidden-command behavior | `TT-066-01`, `TT-066-02` | Public target rejection, no-target skip artifact, bounded command set, and local fixture target path are proven |
| `TT-066-07` | Implement prompt, profile, and `.codex` surface contract suites | `TT-066-01` | Full `.codex` symlink integrity, prompt merge rules, local-artifact `doublecheck` behavior, Z00Z lane routing, no MCP/HexStrike server/Strix runtime/`LLM_API_KEY` default path, and no Tari vendor modification path are proven |
| `TT-066-08` | Implement pack and Docker portability suites | `TT-066-01`, `TT-066-03`, `TT-066-05` | Pack excludes heavy caches, Docker uses archive input, host report export works, and formal-verification paths are blocked |
| `TT-066-09` | Implement documentation contract suite | `TT-066-01`, `TT-066-07` | Generic vs Z00Z-only docs separation, minimal invocations, required `.codex` compatibility symlinks, and migration failure modes are proven |
| `TT-066-10` | Run the canonical command matrix and record environment-gated outcomes | `TT-066-02` through `TT-066-09` | Unit, integration, and Docker-gated commands are executed or explicitly marked blocked with a real environment reason |

## Task Waves

### Wave T0: Live Seam Audit

- verify that all planned Phase 066 entrypoints and script roots exist before
  any executable tests are written
- confirm which existing Rust tests can be reused as invariant anchors
- stop here if the phase is still planning-only

### Wave T1: Unit And Contract Coverage

- implement `TT-066-02` through `TT-066-04`
- lock down scope validation, tool manifest, artifact schema, and report
  classification before runner-level integration

### Wave T2: Local Runner And DAST Integration

- implement `TT-066-05` and `TT-066-06`
- prove timestamp alignment, artifact creation, CLI flags, skip-vs-fail
  semantics, and bounded command invocation

### Wave T3: Prompt, Profile, And Symlink Surfaces

- implement `TT-066-07` and `TT-066-09`
- prove `.codex` compatibility, prompt merge rules, Z00Z routing, and docs
  separation before portability

### Wave T4: Pack And Docker Portability

- implement `TT-066-08`
- prove archive-driven Docker behavior, no live-checkout mode, and
  formal-verification exclusion

### Wave T5: Command Matrix And Closeout

- implement `TT-066-10`
- run or truthfully block every canonical verification command from the spec

## Implemented Test Files And Ownership

| Proposed File | Primary Coverage | Source Anchors |
| --- | --- | --- |
| `tests/penetration/test_scope_validation.py` | `WS-01`, `WS-08`, `WS-12` | `validate_scope.py`, `.security/scope.yaml`, `.security/denied-tools.txt` |
| `tests/penetration/test_tool_manifest.py` | `WS-02`, `WS-03`, `WS-10`, `WS-12` | installer/checker scripts, manifest locks, provenance notes |
| `tests/penetration/test_artifact_schema.py` | `WS-06`, `WS-07`, `WS-12` | artifact validator, manifest/report contract |
| `tests/penetration/test_report_builder.py` | `WS-06`, `WS-07`, `WS-12`, `WS-13` | report builder, evidence mapping, doublecheck section |
| `tests/penetration/test_local_runner_integration.py` | `WS-01`, `WS-02`, `WS-06`, `WS-07` | `run_local_pentest.sh`, `run_parallel_static.sh` |
| `tests/penetration/test_dast_runner_integration.py` | `WS-01`, `WS-06`, `WS-08`, `WS-12` | `run_local_dast.sh`, local fixture service |
| `tests/penetration/test_codex_surface_integration.py` | `WS-09`, `WS-13` | `.codex` symlinks, prompts, agents |
| `tests/penetration/test_profile_routing.py` | `WS-05`, `WS-13` | `z00z-pentest-profile`, Z00Z prompt, routing references |
| `tests/penetration/test_prompt_contracts.py` | `WS-13` | generic, parallel, report, and Z00Z prompts |
| `tests/penetration/test_packaging_portability.py` | `WS-10` | `pack_z00z_project.sh`, `z00z_penetration_tests.sh` |
| `tests/penetration/test_docker_scope.py` | `WS-10`, `WS-11` | Docker wrapper, Dockerfile, `validate_pentest_docker_scope.py` |
| `tests/penetration/test_docs_contracts.py` | `WS-14` | README, migration guide, checklist, invariants doc |

## Fixture Requirements

### Scope Fixtures

- `source_only_scope.yaml`
- `local_url_scope.yaml`
- `public_url_scope.yaml`
- `public_ip_scope.yaml`
- `wildcard_host_scope.yaml`
- `broad_cidr_scope.yaml`

### Tool Status Fixtures

- `expected_contract.json`
- temporary tool manifests are synthesized inside `test_tool_manifest.py`

### Scanner Output Fixtures

- `findings_mixed.json`
- `findings_scanner_only_confirmed.json`
- `findings_missing_regression_test.json`
- `raw_semgrep_secret.json`
- `raw_gitleaks_secret.json`

### Docker Fixtures

- no static file fixtures are required; Docker expectations are asserted
  directly against the live wrapper, Dockerfile, README, and scope validator

### Profile And Prompt Fixtures

- `z00z_lane_map_expected.json`
- `z00z_dry_run_expected.json`
- `prompt_contract_expected.json`
- `parallel_merge_findings_a.json`
- `parallel_merge_findings_b.json`

## Required Command Matrix

These commands are the minimum implementation-time matrix and must be captured
in test execution notes:

```bash
bash -n scripts/penetration/*.sh
python3 -m py_compile scripts/penetration/*.py
python3 scripts/penetration/validate_scope.py .security/scope.yaml
bash scripts/penetration/check_pentest_tools.sh --json
test -L .codex/skills
test -L .codex/agents
test -L .codex/prompts
test -L .codex/hooks
test -L .codex/instructions
test -L .codex/requirements
test -L .codex/scripts
test -L .codex/plugins
rg -n "hydra|john|hashcat|medusa|patator|metasploit|msfvenom|pacu|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security/denied-tools.txt
rg -n "MCP|HexStrike server|Strix runtime|LLM_API_KEY|hydra|john|hashcat|medusa|patator|metasploit|msfvenom|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security
python3 -m pytest tests/penetration
python3 -m unittest discover tests/penetration
./pack_z00z_project.sh --output /tmp/z00z-pentest-portable.tar.gz
./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-portable.tar.gz --mode quick --static-only --check-only
tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-portable.tar.gz --mode check-only
python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration
```

Current host note: the Docker archive rows above were revalidated on
`2026-07-03`; `python3 -m pytest tests/penetration` remains blocked until the
`pytest` module is installed locally.

## Completion Gate

The test implementation work is complete only when all of the following are
true:

- every task `TT-066-01` through `TT-066-10` is either complete or blocked by
  a concrete environment prerequisite;
- every scenario in `066-TEST-SPEC.md` has a concrete test anchor;
- all rejection paths remain fail-closed;
- Docker-only cases are either green or explicitly skipped with a truthful
  environment reason;
- no test or fixture introduces a parallel authority layer or duplicated
  workflow logic.
