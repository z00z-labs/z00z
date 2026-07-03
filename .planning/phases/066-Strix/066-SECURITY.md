---
phase: 066
slug: 066-Strix
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-03
---

# Phase 066 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

Retroactive STRIDE audit. Phase 066 has executed summaries but no formal
`<threat_model>` blocks in its numbered plans and no `## Threat Flags` sections
in its numbered summaries, so the register below was built from the live
implementation, the executed summary artifacts, and direct verification commands
run on 2026-07-03.

## 🔒 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Operator or agent -> local scope gate | `./z00z_penetration_tests.sh` and `scripts/penetration/run_local_pentest.sh` accept user-selected mode, scope, and profile | scope path, mode, profile, artifact root |
| Orchestrator -> local tool root | runner scripts consult `tools/penetration/` wrappers and payload manifests only | tool availability, wrapper paths, cached payload locations |
| Orchestrator -> artifact or report stores | runner, report builder, and validator write `.security-artifacts/<timestamp>/` and `reports/z00z-pentests_report-<timestamp>/` | manifests, status JSON, logs, normalized findings, Markdown report |
| Host -> Docker pentest sandbox | Docker wrapper mounts a packed archive read-only and copies exported artifacts back to the host | packed workspace archive, scope path, run id, host artifact and report roots |
| Generic core -> Z00Z adapter | `z00z-pentest-profile` routes Z00Z review lanes through existing repository skills only | lane selection, invariant requirements, simulation obligations, closeout prompt path |

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-066-01 | Spoofing | local scope and DAST admission | mitigate | Fail closed unless scope stays `local-only`, hosts stay loopback-only, and DAST has validated local targets | `.security/scope.yaml:1`, `scripts/penetration/validate_scope.py:269`, `scripts/penetration/validate_scope.py:327`, `scripts/penetration/validate_scope.py:347`, `scripts/penetration/run_local_pentest.sh:148`, `scripts/penetration/run_local_dast.sh:163` | closed |
| T-066-02 | Spoofing | external runtime and forbidden-tool surface | mitigate | Keep one canonical entrypoint, no MCP runtime, no external API key, and no forbidden-tool execution path in the default workflow | `.security/denied-tools.txt:1`, `.github/skills/pentest-local-orchestrator/SKILL.md:23`, `.github/prompts/pentest-local.prompt.md:14`, `.github/prompts/pentest-local.prompt.md:55`, `.github/skills/z00z-pentest-profile/SKILL.md:45`, `.github/prompts/pentest-local-z00z.prompt.md:23` | closed |
| T-066-03 | Tampering | pentest tool root and verification-tool overlap | mitigate | Keep all pentest payloads, wrappers, caches, and manifests under `tools/penetration/`; exclude `tools/formal_verification`; forbid Docker pentest scripts from invoking formal-verification flows | `.security/scope.yaml:14`, `scripts/penetration/install_pentest_tools.sh:6`, `scripts/penetration/install_pentest_tools.sh:63`, `scripts/penetration/check_pentest_tools.sh:6`, `scripts/penetration/check_pentest_tools.sh:153`, `scripts/penetration/validate_pentest_docker_scope.py:11`, `tools/penetration/docker/run_pentest_container.sh:247` | closed |
| T-066-04 | Tampering | finding classification and report integrity | mitigate | Reject any confirmed finding that lacks source evidence, scanner artifact, proof, confidence, fix guidance, and regression coverage; keep scanner-only output unconfirmed until validated | `.security/report-template.md:47`, `scripts/penetration/build_pentest_report.py:237`, `scripts/penetration/build_pentest_report.py:436`, `scripts/penetration/validate_artifacts.py:75`, `scripts/penetration/validate_artifacts.py:171`, `.github/prompts/pentest-report-doublecheck.prompt.md:21` | closed |
| T-066-05 | Repudiation | artifact or report identity and audit trace | mitigate | Bind artifact tree and host report root to one run id, keep command ledger paths, and require paired host-side report exports | `scripts/penetration/run_local_pentest.sh:32`, `scripts/penetration/run_local_pentest.sh:133`, `scripts/penetration/build_pentest_report.py:369`, `scripts/penetration/build_pentest_report.py:534`, `scripts/penetration/validate_artifacts.py:120`, `scripts/penetration/validate_artifacts.py:171` | closed |
| T-066-06 | Elevation of privilege | Docker sandbox execution | mitigate | Run as non-root with read-only rootfs, drop all capabilities, keep no-new-privileges, mount the archive read-only, and keep logs attached to the caller | `tools/penetration/docker/run_pentest_container.sh:24`, `tools/penetration/docker/run_pentest_container.sh:192`, `tools/penetration/docker/run_pentest_container.sh:214`, `tools/penetration/docker/run_pentest_container.sh:294`, `z00z_penetration_tests.sh:24`, `z00z_penetration_tests.sh:180` | closed |
| T-066-07 | Tampering | Z00Z-specific routing and vendor-code boundary | mitigate | Route Z00Z lanes through existing repository security skills only, forbid `crates/z00z_crypto/tari/**` edits, and keep dry runs documentation-only with deterministic simulation obligations | `.github/skills/z00z-pentest-profile/SKILL.md:16`, `.github/skills/z00z-pentest-profile/SKILL.md:35`, `.github/skills/z00z-pentest-profile/SKILL.md:47`, `.github/skills/z00z-pentest-profile/SKILL.md:59`, `.github/prompts/pentest-local-z00z.prompt.md:20`, `.github/prompts/pentest-local-z00z.prompt.md:72` | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## ✅ Accepted Risks Log

No accepted risks.

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-03 | 7 | 7 | 0 | Codex `/gsd-secure-phase 066` |

## 🔍 Audit Evidence

Commands executed during this audit pass:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `python3 -m unittest discover tests/penetration`
- `python3 scripts/penetration/validate_scope.py .security/scope.yaml`
- `bash scripts/penetration/check_pentest_tools.sh --json`
- `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration`
- `./pack_z00z_project.sh --output /tmp/z00z-pentest-portable.tar.gz`
- `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-portable.tar.gz --mode quick --static-only --check-only`
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-portable.tar.gz --mode check-only`
- `git diff --check`

Observed results used for closure:

- `bootstrap_tests.sh` completed green and ended with `=== BOOTSTRAP COMPLETE ===`.
- `python3 -m unittest discover tests/penetration` completed green with `59` passing tests.
- `validate_scope.py` confirmed local-only scope with `hosts=2 urls=0`.
- `check_pentest_tools.sh --json` remained truthful and reported missing local
  tools rather than silently promoting them to runnable.
- The Docker-scope validator completed green with `validated_files=12`.
- Both Docker check-only entrypoints completed green, exported a paired host
  artifact tree and host report tree, and wrote `docker-run.json` plus the
  copied host report files.

## 👍 Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-03
