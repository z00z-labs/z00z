---
phase: 066
slug: 066-Strix
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-03
---

# Phase 066 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Python `unittest` plus shell and Python command checks |
| **Config file** | none — stdlib unittest discovery under `tests/penetration` |
| **Quick run command** | `python3 -m unittest discover tests/penetration` |
| **Full suite command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && python3 -m unittest discover tests/penetration` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run `python3 -m unittest discover tests/penetration`
- **After every plan wave:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && python3 -m unittest discover tests/penetration`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `WS-01` | 066-01 | 1 | `REQ-001` | `T-066-07` | Phase 066 artifacts stay English-only | doc contract | `python3 -m unittest tests.penetration.test_docs_contracts` | ✅ | ✅ green |
| `WS-02` | 066-02 | 1 | `REQ-002` | `T-066-03` | Tool roots, wrappers, and manifests stay under `tools/penetration/` | contract | `python3 -m unittest tests.penetration.test_tool_manifest` | ✅ | ✅ green |
| `WS-04` | 066-04 | 3 | `REQ-003` | `T-066-07` | Active reusable skills stay under `.github/skills/` only | integration | `python3 -m unittest tests.penetration.test_codex_surface_integration` | ✅ | ✅ green |
| `WS-09` | 066-09 | 5 | `REQ-004` | `T-066-07` | `.codex/*` remains a symlink compatibility surface to `.github/*` | integration | `python3 -m unittest tests.penetration.test_codex_surface_integration` | ✅ | ✅ green |
| `WS-03` | 066-03 | 2 | `REQ-005` | `T-066-02` | Default workflow stays no-MCP, no HexStrike server, no Strix runtime, and no external LLM key | contract | `python3 -m unittest tests.penetration.test_tool_manifest tests.penetration.test_prompt_contracts` | ✅ | ✅ green |
| `WS-01` | 066-01 | 1 | `REQ-006` | `T-066-01` | Dynamic testing is local-only and scope-driven by default | unit plus integration | `python3 -m unittest tests.penetration.test_scope_validation tests.penetration.test_dast_runner_integration` | ✅ | ✅ green |
| `WS-07` | 066-07 | 4 | `REQ-007` | `T-066-04` | Findings remain hypotheses until evidence, repro, regression, or false-positive classification exists | unit plus integration | `python3 -m unittest tests.penetration.test_report_builder tests.penetration.test_artifact_schema tests.penetration.test_local_runner_integration` | ✅ | ✅ green |
| `WS-03` | 066-03 | 2 | `REQ-008` | `T-066-03` | Copied upstream material stays license-aware, provenance-pinned, and reference-only when non-executable | contract | `python3 -m unittest tests.penetration.test_tool_manifest` | ✅ | ✅ green |
| `WS-05` | 066-05 | 3 | `REQ-009` | `T-066-07` | `crates/z00z_crypto/tari/**` remains read-only and out of modification scope | routing contract | `python3 -m unittest tests.penetration.test_profile_routing` | ✅ | ✅ green |
| `WS-02` | 066-02 | 1 | `REQ-010` | `T-066-03` | Install flows avoid global pollution and stay rooted in `tools/penetration/` | contract | `python3 -m unittest tests.penetration.test_tool_manifest` | ✅ | ✅ green |
| `WS-05` | 066-05 | 3 | `REQ-011` | `T-066-07` | Generic `pentest-*` workflow stays reusable while Z00Z logic remains isolated in the profile skill | integration | `python3 -m unittest tests.penetration.test_profile_routing tests.penetration.test_docs_contracts` | ✅ | ✅ green |
| `WS-12` | 066-12 | 7 | `REQ-012` | `T-066-05` | Every plan closes only with executable acceptance evidence and recorded verification | regression | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && python3 -m unittest discover tests/penetration` | ✅ | ✅ green |
| `WS-03` | 066-03 | 2 | `REQ-013` | `T-066-03` | Strix playbooks stay preserved as routed references, not flattened into wrappers only | contract | `python3 -m unittest tests.penetration.test_tool_manifest` | ✅ | ✅ green |
| `WS-04` | 066-04 | 3 | `REQ-014` | `T-066-02` | Active pentest skill surface stays small: seven generic skills plus one profile | surface audit | `python3 -m unittest tests.penetration.test_codex_surface_integration` | ✅ | ✅ green |
| `WS-01` | 066-01 | 1 | `REQ-015` | `T-066-02` | Safety greps distinguish active execution paths from denylist or provenance references | contract | `python3 -m unittest tests.penetration.test_scope_validation` | ✅ | ✅ green |
| `WS-10` | 066-10 | 6 | `REQ-016` | `T-066-06` | Pentest Docker path excludes formal-verification install and execution flows | contract | `python3 -m unittest tests.penetration.test_docker_scope && python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration` | ✅ | ✅ green |
| `WS-10` | 066-10 | 6 | `REQ-017` | `T-066-06` | `./z00z_penetration_tests.sh --docker-sandbox` remains the canonical pentest Docker entrypoint | e2e | `python3 -m unittest tests.penetration.test_packaging_portability tests.penetration.test_docker_scope` | ✅ | ✅ green |
| `WS-10` | 066-10 | 6 | `REQ-018` | `T-066-06` | Docker pentests run from `pack_z00z_project.sh` tarballs, not mutable live checkout state | e2e | `./pack_z00z_project.sh --output /tmp/z00z-pentest-validate.tar.gz && ./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-validate.tar.gz --mode quick --static-only --check-only` | ✅ | ✅ green |
| `WS-07` | 066-07 | 4 | `REQ-019` | `T-066-05` | Host report roots stay canonical and machine-readable artifacts record the exported report location | unit plus integration | `python3 -m unittest tests.penetration.test_artifact_schema tests.penetration.test_local_runner_integration tests.penetration.test_docker_scope` | ✅ | ✅ green |
| `WS-11` | 066-11 | 6 | `REQ-020` | `T-066-06` | Docker runs stream logs to the caller and copy reports back to matching host report roots before exit | e2e | `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-validate.tar.gz --mode check-only` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-07-03

| Metric | Count |
|--------|-------|
| Gaps found | 5 |
| Resolved | 5 |
| Escalated | 0 |

### Audit Evidence

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` completed green and ended with `=== BOOTSTRAP COMPLETE ===`.
- `python3 -m unittest discover tests/penetration` completed green with `59` passing tests.
- `bash -n scripts/penetration/*.sh tools/penetration/docker/run_pentest_container.sh z00z_penetration_tests.sh pack_z00z_project.sh unpack_z00z_project.sh` completed green.
- `python3 -m py_compile scripts/penetration/*.py tests/penetration/*.py` completed green.
- `python3 scripts/penetration/validate_scope.py .security/scope.yaml` completed green with `OK: scope validated for local-only testing`, `hosts=2`, and `urls=0`.
- `bash scripts/penetration/check_pentest_tools.sh --json` completed green and remained truthful with `present: 0`, `missing: 12`, and `missing_required: 9`.
- `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration` completed green with `validated_files=12`.
- `.codex/skills`, `.codex/agents`, `.codex/prompts`, `.codex/hooks`, `.codex/instructions`, `.codex/requirements`, `.codex/scripts`, and `.codex/plugins` all passed symlink compatibility checks.
- `rg -n "MCP|HexStrike server|Strix runtime|LLM_API_KEY|hydra|john|hashcat|medusa|patator|metasploit|msfvenom|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security` produced only expected safety, denylist, and reference-only matches.
- `./pack_z00z_project.sh --output /tmp/z00z-pentest-validate.tar.gz` completed green and produced a portable archive.
- `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-validate.tar.gz --mode quick --static-only --check-only` completed green and exported host artifacts plus host reports from the packed archive path.
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-validate.tar.gz --mode check-only` completed green and preserved attached stdout/stderr while exporting the report directory back to the host.
- `python3 -m pytest tests/penetration` is environment-blocked because `pytest` is not installed locally; this is non-blocking because every Phase 066 test module executes through the stdlib `unittest` runner and the phase plans already allow `pytest` or `unittest` fallback.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-03
