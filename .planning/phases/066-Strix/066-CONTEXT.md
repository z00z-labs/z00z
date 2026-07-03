# Phase 066: Local Pentest Orchestration - Context

**Gathered:** 2026-07-02
**Status:** Reviewed for plan coverage
**Source:** PRD Express Path (`066-TODO.md`)

<domain>
## Phase Boundary

Phase 066 implements a local-only penetration-testing orchestration system. The
canonical human and agent entrypoint is `./z00z_penetration_tests.sh`; reusable
workflow logic lives in `.github/skills/pentest-*`; optional specialist review
lanes live in `.github/agents/pentest-*`; reusable invocation text lives in
`.github/prompts/pentest-*`; third-party tool state lives under
`tools/penetration/`; and generated evidence is split between
`.security-artifacts/<timestamp>/` for machine-readable artifacts and
`reports/z00z-pentests_report-<timestamp>/` for human-facing reports.

This phase does not run Strix as a product, does not run HexStrike as an MCP
server, and does not require external LLM API keys for the default workflow.
Strix and HexStrike are source material only: Strix contributes methodology and
reference playbooks, while HexStrike contributes tool taxonomy and timeout or
artifact discipline.

## 🎯 Executive Conclusion

Build a local skill-and-script orchestration layer. Do not run Strix as a
product, do not run HexStrike as an MCP server, and do not add external LLM
API-key requirements.
Build a local skill-and-script orchestration layer. Do not run Strix as a product, do not run HexStrike as an MCP server, and do not add external LLM API-key requirements.

- The correct migration target is:

```text
Codex or GitHub Copilot = reasoning and orchestration layer
./z00z_penetration_tests.sh = canonical human/agent entrypoint, including --docker-sandbox
.github/skills/pentest-* = reusable workflow definitions
.github/agents/pentest-* = optional parallel review specialists
.github/prompts/pentest-* = reusable invocation prompts
tools/penetration/ = all newly installed third-party security tools and caches
scripts/penetration/ = thin repo commands that invoke local tools safely
.codex/* = symlinked compatibility surface to .github/*
```

This phase must preserve the useful value of both upstream repositories:

- Strix value to preserve: source-aware white-box methodology, skill taxonomy, SAST-to-dynamic-validation workflow, and bounded CLI playbooks.
- Strix value to preserve beyond tools: framework, protocol, technology, vulnerability, and scan-mode playbooks as a reference library and routing matrix.
- HexStrike value to preserve: tool inventory, category taxonomy, process/artifact discipline, timeout awareness, and local tool execution patterns.
- Strix value not to preserve: Strix runtime, Docker sandbox product flow, provider configuration, `openai-agents[litellm]`, Caido SDK runtime coupling, external API-key model.
- HexStrike value not to preserve: MCP server/client, Flask API server, arbitrary command wrapper, file operation wrappers, credential attacks, password cracking, exploit generation, payload generation, public recon defaults, AI evasion/payload generation, and any default workflow that can affect non-owned targets.
</domain>

<decisions>
## Implementation Decisions

### D-066-01 Scope And Safety First
- `066-TODO.md` is normative. Planning must cover all fourteen workstreams
  `WS-01` through `WS-14` exactly once.
- No literal `TASK-NNN` identifiers exist in `066-TODO.md`; the required task
  rows for this planning pass are the `WS-NN` workstreams.
- Dynamic testing must be local-only by default and must read scope from
  `.security/scope.yaml`.
- Scanner output is a hypothesis until linked to source evidence, controlled
  local reproduction, regression tests, or explicit false-positive status.

### D-066-02 Tooling And Runtime Ownership
- All new third-party security tools, cloned upstream sources, local tool
  caches, and tool wrappers must live under `tools/penetration/`.
- Formal-verification tooling under `tools/formal_verification/**` must not be
  installed or executed by the pentest Docker path.
- Tool installation must avoid global pollution where feasible and record
  versions under `tools/penetration/manifests/`.

### D-066-03 Skills, Agents, And Prompts
- Generic reusable skills are limited to the `pentest-*` family under
  `.github/skills/`.
- Z00Z-specific audit logic is isolated in `.github/skills/z00z-pentest-profile/`.
- Optional parallel review specialists live under `.github/agents/`; agents do
  not directly run tools unless the orchestrator prompt instructs them.
- `.codex/*` remains a symlink compatibility surface for `.github/*`.

### D-066-04 Scripts And Reports
- `scripts/penetration/run_local_pentest.sh` is the local orchestrator invoked
  by skills and `./z00z_penetration_tests.sh`.
- The report builder must create human-facing reports under
  `reports/z00z-pentests_report-<timestamp>/`.
- The machine-readable artifact tree must include `manifest.json`,
  `scope.normalized.json`, `tool-status.json`, `sast/`, `rust/`, `secrets/`,
  `dast/`, `raw/`, `normalized/`, `report/security-report.md`, and `logs/`.

### D-066-05 Docker Is Optional And Archive-Driven
- Normal local static scans must work without Docker.
- `./z00z_penetration_tests.sh --docker-sandbox` is the canonical Docker
  entrypoint and must run from a portable archive produced by
  `pack_z00z_project.sh`.
- The Docker path must keep stdout and stderr attached to the invoking
  terminal and copy all generated human-facing reports back to the paired host
  report directory.
- The Docker path must not delegate to `unpack_z00z_project.sh --docker-sandbox`.

### D-066-06 Evidence And Verification
- Every implementation task must include real artifacts, tests, expected
  results, simulation requirements, and anti-placeholder proof.
- Each executable plan must run the smart bootstrap gate first in its verify
  section and must stop, fix, and rerun if it fails.
- Runtime behavior cannot be closed by compile-only proof. Code behavior cannot
  be closed by docs-only proof.

### D-066-07 No Parallel Layer And No Concept Drift
- Phase 066 must extend the existing `.github/*`, `.codex/*`, `scripts/`, and
  `pack_z00z_project.sh` surfaces instead of introducing a second skill root,
  duplicate prompt tree, alternate Docker runner authority, or parallel report
  contract.
- Phase 066 must not introduce alternate skill roots, duplicate prompt trees,
  alternate Docker runner authorities, or a parallel report contract.
- Pack or unpack behavior may be extended only through pentest-specific wrapper
  logic that reuses safe extraction and symlink verification patterns without
  reactivating unrelated formal-verification flows.
</decisions>

<source_corpus>
## 🔑 Source Corpus

- `SRC-002`: `https://github.com/usestrix/strix` at commit
  `f342808d2b4af551a17e47af744d2f19dee3c443`.
  Evidence used: `README.md`, `pyproject.toml`, and `strix/skills/**`
  including `coordination`, `custom`, `tooling`, `scan_modes`, `frameworks`,
  `protocols`, `technologies`, and `vulnerabilities`.
  Planning meaning: Strix is not the runtime target, but its skill corpus is a
  reference library to preserve.
- `SRC-003`: `https://github.com/0x4m4/hexstrike-ai` at commit
  `9b8c780f324ce5145a322bfa23c98886f8424ba3`.
  Evidence used: `README.md`, `hexstrike_mcp.py`, `hexstrike_server.py`,
  `requirements.txt`, and `LICENSE`.
  Planning meaning: use only allowlisted tool taxonomy, timeout patterns, and
  safe local CLI ideas; do not implement MCP or Flask runtime paths.
- `SRC-004`: `.github/skills/attack-surfaces-create/SKILL.md` and
  `REFERENCE.md`.
  Planning meaning: inventory-first, source-backed attack-surface mapping is
  the Z00Z baseline for candidate admission and targeted review.
- `SRC-005`: `.github/skills/z00z-crypto-auditor/SKILL.md` and `FORMS.md`.
  Planning meaning: reuse as the Z00Z-specific protocol, wallet, proof, and
  secrecy audit adapter.
- `SRC-006`: `.github/prompts/gsd-audit-4.prompt.md`.
  Planning meaning: heavy closeout path after implementation or remediation.
- `SRC-007`: `pack_z00z_project.sh` and `unpack_z00z_project.sh`.
  Planning meaning: reuse portable archive, symlink-manifest, and safe extract
  ideas, but never reuse `unpack_z00z_project.sh --docker-sandbox` as the
  pentest Docker runner because it installs and executes formal-verification
  flows.
- `SRC-008`: `.codex/skills -> ../.github/skills` and related `.codex`
  symlinks.
  Planning meaning: new skills, agents, prompts, hooks, scripts, requirements,
  instructions, and plugins remain canonical under `.github/*` and exposed
  through symlink compatibility only.
</source_corpus>

<constraints>
## 🛑 Non-Negotiable Constraints

- `REQ-001`: All repository artifacts created by this phase must be English-only.
- `REQ-002`: New third-party penetration tools, cloned upstream sources, local tool caches, and tool wrappers must live under `tools/penetration/`.
- `REQ-003`: Reusable skills must live under `.github/skills/`; no new canonical skill root may be introduced.
- `REQ-004`: `.codex/skills`, `.codex/agents`, `.codex/prompts`, `.codex/hooks`, `.codex/instructions`, `.codex/requirements`, `.codex/scripts`, and `.codex/plugins` must remain symlink compatibility surfaces for `.github/*`.
- `REQ-005`: No MCP server, no HexStrike server, no Strix runtime, and no external LLM API keys may be required for the default workflow.
- `REQ-006`: Dynamic testing must be local-only by default and must read allowed targets from a scope file.
- `REQ-007`: Scanner output is only a hypothesis until linked to source evidence, controlled local reproduction, regression test, or explicit false-positive classification.
- `REQ-008`: Any copied upstream reference must preserve license/provenance metadata and must be marked as reference-only when it must not be executed.
- `REQ-009`: `crates/z00z_crypto/tari/**` remains read-only and out of scope for modification.
- `REQ-010`: All install scripts must avoid global tool pollution where feasible and must install new tools into `tools/penetration/`.
- `REQ-011`: The workflow must be reusable outside Z00Z through generic `pentest-*` skills, with Z00Z-specific audit logic isolated in a profile skill.
- `REQ-012`: Every task below is incomplete until its acceptance checks and doublecheck evidence are recorded.
- `REQ-013`: Do not flatten Strix into tool wrappers only; preserve framework, protocol, technology, vulnerability, and scan-mode playbooks as reference-only knowledge and routing guidance.
- `REQ-014`: Keep the default active skill surface small: target seven generic `pentest-*` skills plus one project profile. Extra upstream playbooks belong under `references/`, not as dozens of auto-triggering skills.
- `REQ-015`: Any grep-based safety check must distinguish active execution paths from reference-only upstream material and must not fail merely because a forbidden tool is named in a denylist or provenance note.
- `REQ-016`: Pentest Docker workflows must not install, bootstrap, or run formal-verification tooling. Do not call `scripts/install-verification-tools.sh`, `tools/formal_verification/**`, `z00z-full-verify-gate`, or verification-orchestrator report flows from the pentest Docker path.
- `REQ-017`: `./z00z_penetration_tests.sh --docker-sandbox` must be the canonical Docker entrypoint for penetration tests. It must be purpose-built for pentests and must not delegate to `unpack_z00z_project.sh --docker-sandbox`.
- `REQ-018`: Docker pentests must run from the same portable tarball artifact produced by `pack_z00z_project.sh`. The Docker path must not implicitly use mutable live checkout state as its source of truth.
- `REQ-019`: Final human-facing pentest reports must be generated under `reports/z00z-pentests_report-<timestamp>/` on the host. `.security-artifacts/<timestamp>/` remains the machine-readable artifact tree and must reference the exported host report location.
- `REQ-020`: Docker pentest runs must stream container stdout/stderr to the invoking terminal and must copy every report they generate back to the matching host `reports/z00z-pentests_report-<timestamp>/` directory before exit.
</constraints>

<canonical_refs>
## Canonical References

### Phase Authority
- Primary source note: `.planning/phases/066-Strix/strix.md` is an input, not the final authority after this file is accepted.
- `.planning/phases/066-Strix/066-TODO.md` - normative Phase 066 backlog and
  acceptance source.
- `.planning/phases/066-Strix/066-COVERAGE.md` - generated planning coverage
  ledger for the zero `TASK-NNN` / fourteen `WS-NN` mapping.
- `.planning/phases/066-Strix/strix.md` - historical input named by
  `066-TODO.md`, but absent from the current workspace; do not recreate it as a
  second authority layer after `066-TODO.md` acceptance.

### Repository Rules
- `.github/copilot-instructions.md` - Z00Z operational rules, English-only
  repository artifacts, safe file operations, protected Tari vendor directory,
  and final tone requirement.
- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` - design, testing,
  naming, NASA-style, and one-source-of-truth rules.

### Existing Security Sources
- `.github/skills/attack-surfaces-create/SKILL.md` - deterministic inventory
  and attack-surface admission method.
- `.github/skills/attack-surfaces-create/REFERENCE.md` - Z00Z architecture
  invariants and attack-surface reference checklist.
- `.github/skills/z00z-crypto-auditor/SKILL.md` - Z00Z-specific crypto and
  security audit adapter.
- `.github/skills/z00z-crypto-auditor/FORMS.md` - Z00Z audit output forms.
- `.github/prompts/gsd-audit-4.prompt.md` - heavy audit closeout prompt.
- `.github/prompts/gsd-review-tasks-execution.prompt.md` - mandatory task
  execution review prompt referenced by every plan verify block.
- `.github/skills/smart-tests-bootstrap/SKILL.md` - bootstrap test gate.

### Current Code Anchors
- `pack_z00z_project.sh` - portable archive source for Docker pentest runs.
- `unpack_z00z_project.sh` - restore behavior to avoid reusing as the pentest
  Docker runner.
- `.codex/skills`, `.codex/agents`, `.codex/prompts`, `.codex/hooks`,
  `.codex/instructions`, `.codex/requirements`, `.codex/scripts`, and
  `.codex/plugins` - symlink compatibility surfaces.
- `scripts/` - current script home; Phase 066 adds `scripts/penetration/`.
- `tools/formal_verification/` - explicit non-pentest tool root that Docker
  pentest scripts must not call.
</canonical_refs>

<architecture_snapshot>
## ⚙️ Target Architecture

```text
tools/
  penetration/
    README.md
    bin/
    cargo/
    go/
    python/
    cache/
    rules/
    templates/
    wordlists/
    upstream/
      strix/
      hexstrike-ai/
    manifests/
      tool-versions.lock
      upstream-sources.lock
      checksums.sha256
    docker/
      README.md
      Dockerfile
      run_pentest_container.sh

z00z_penetration_tests.sh

scripts/
  penetration/
    install_pentest_tools.sh
    check_pentest_tools.sh
    run_local_pentest.sh
    run_parallel_static.sh
    run_source_sast.sh
    run_rust_security.sh
    run_secrets_supply_chain.sh
    run_local_dast.sh
    build_pentest_report.py
    validate_scope.py
    validate_artifacts.py

.github/
  skills/
    pentest-local-orchestrator/
      references/
        strix/
          coordination/
          scan_modes/
          frameworks/
          protocols/
          technologies/
          vulnerabilities/
    pentest-source-aware-sast/
    pentest-rust-security/
    pentest-secrets-supply-chain/
    pentest-local-dast/
    pentest-report/
    pentest-tool-installer/
    z00z-pentest-profile/
  agents/
    pentest-rust-reviewer.agent.md
    pentest-crypto-reviewer.agent.md
    pentest-storage-reviewer.agent.md
    pentest-rpc-dast-reviewer.agent.md
    pentest-supply-chain-reviewer.agent.md
  prompts/
    pentest-local.prompt.md
    pentest-parallel-review.prompt.md
    pentest-report-doublecheck.prompt.md

.security/
  scope.yaml
  allowed-targets.txt
  denied-tools.txt
  report-template.md

.security-artifacts/
  <timestamp>/

reports/
  z00z-pentests_report-<timestamp>/
```
</architecture_snapshot>

<migration_map>
## 📦 Upstream Migration Map

| Upstream Area | Action | Destination | Rationale |
| --- | --- | --- | --- |
| `strix/skills/coordination/source_aware_whitebox.md` | Copy as reference with provenance header | `.github/skills/pentest-local-orchestrator/references/strix/source_aware_whitebox.md` | Preserves source-first triage and evidence-driven validation. |
| `strix/skills/custom/source_aware_sast.md` | Adapt into local skill and script requirements | `.github/skills/pentest-source-aware-sast/` | Provides Semgrep, AST, secrets, and Trivy baseline. |
| `strix/skills/tooling/semgrep.md` | Copy as reference | `.github/skills/pentest-source-aware-sast/references/strix/semgrep.md` | Preserves automation-safe Semgrep flags, especially `--metrics=off`. |
| `strix/skills/tooling/nmap.md` | Copy as reference | `.github/skills/pentest-local-dast/references/strix/nmap.md` | Preserves bounded two-pass scan model. |
| `strix/skills/tooling/nuclei.md` | Copy as reference | `.github/skills/pentest-local-dast/references/strix/nuclei.md` | Preserves rate, concurrency, severity, and `-ni` guidance. |
| `strix/skills/tooling/httpx.md`, `katana.md`, `ffuf.md`, `naabu.md` | Copy as references | `.github/skills/pentest-local-dast/references/strix/` | Preserves local web probing, crawling, bounded fuzzing, and bounded port discovery patterns. |
| `strix/skills/tooling/sqlmap.md`, `subfinder.md`, `agent_browser.md`, `python.md` | Copy only as reference-only material or skip | `.github/skills/pentest-local-dast/references/strix/reference-only/` | These depend on public recon, Caido/sandbox assumptions, or higher-risk exploitation paths and must not become default runners. |
| `strix/skills/scan_modes/*.md` | Copy as references | `.github/skills/pentest-local-orchestrator/references/strix/scan_modes/` | Preserves quick/standard/deep mode distinctions. |
| `strix/skills/frameworks/*.md` | Copy as references and index | `.github/skills/pentest-local-orchestrator/references/strix/frameworks/` | Preserves framework-specific review prompts without creating one skill per framework. |
| `strix/skills/protocols/*.md` | Copy as references and index | `.github/skills/pentest-local-orchestrator/references/strix/protocols/` | Preserves protocol-specific testing guidance such as GraphQL/API review patterns. |
| `strix/skills/technologies/*.md` | Copy as references and index | `.github/skills/pentest-local-orchestrator/references/strix/technologies/` | Preserves technology-specific review hints while keeping the active skill set compact. |
| `strix/skills/vulnerabilities/*.md` | Copy as references and index | `.github/skills/pentest-local-orchestrator/references/strix/vulnerabilities/` | Preserves high-value vulnerability playbooks for source-backed manual validation without granting direct execution permission. |
| Strix reference index | Create native routing matrix | `.github/skills/pentest-local-orchestrator/references/strix/routing-matrix.md` | Maps languages, frameworks, services, and findings to the copied reference playbooks. |
| Strix runtime, CLI, Docker product flow | Do not execute | N/A | Runtime requires provider setup and does not match the local no-API-key target. |
| HexStrike `README.md` tool lists | Extract allowlist, optional-lab list, and denylist | `.github/skills/pentest-tool-installer/references/hexstrike/tool-inventory.md` | Preserves category value without importing unsafe execution wrappers. |
| HexStrike `hexstrike_mcp.py` | Copy only as reference if needed, with warning header | `.github/skills/pentest-tool-installer/references/hexstrike/hexstrike_mcp_reference_only.py` | Useful for tool name inventory; must never be run. |
| HexStrike `hexstrike_server.py` | Do not copy by default | `tools/penetration/upstream/hexstrike-ai/` only | Contains broad command execution, payload, and exploit endpoints. |
| HexStrike MCP client/server flow | Do not implement | N/A | Conflicts with the no-MCP requirement. |
</migration_map>

<tool_policy>
## 🚫 Tool Policy

### ✅ Allowed By Default
- Source SAST: `semgrep`, `sg` or `ast-grep`, `tree-sitter`.
- Secrets: `gitleaks`, `trufflehog`.
- Supply chain: `trivy fs`, `cargo audit`, `cargo deny`, `cargo geiger`.
- Rust verification: `cargo fmt`, `cargo clippy`, `cargo test`,
  `cargo nextest`, selective `cargo miri`, selective `cargo fuzz`.
- Local DAST: `nmap`, `nuclei`, `httpx`, `katana`, `ffuf`, `gobuster`.
- Optional bounded local-only DAST: `naabu`, `dirsearch`, `nikto` only with
  scope validation and rate limits.
- Optional web or frontend SAST: `retire`, `eslint` security rules, and
  package-manager audits only when matching manifests exist.
- Containers or IaC: `trivy config`, optional `checkov` for repo-local config.
- Reporting: local Python scripts under `scripts/penetration/`.

### ⚠️ Reference-Only Or Future Lab Profile
- `sqlmap` is reference-only with no dumping, OS shell, destructive flags, or
  default execution path.
- Public recon and domain enumeration tools such as `subfinder`, `amass`,
  `theHarvester`, Shodan, Censys, HIBP, and breach or leak APIs require a
  separate explicit authorized-target profile.
- High-speed discovery such as `masscan`, `rustscan`, wide `naabu`, and broad
  CIDR workflows are not default.
- Browser and proxy automation such as Strix `agent_browser`, Caido SDK flows,
  OAST or interactsh, and replay workflows are reference-only unless a local
  lab profile explicitly enables them.

### 🚫 Forbidden By Default
- MCP or API runtime: HexStrike MCP, HexStrike server, Strix runtime, external
  LLM API keys.
- Credential attacks: `hydra`, `john`, `hashcat`, `medusa`, `patator`,
  credential stuffing, password cracking.
- Network exploitation and harvesting: `responder`, `netexec`,
  `crackmapexec`, credential harvesting, SMB exploitation, poisoners.
- Exploit frameworks: Metasploit, `msfvenom`, exploitation runners, exploit
  generation, payload generation, evasion payloads.
- Public recon: wide internet recon, Shodan or Censys API OSINT, masscan
  outside explicit local lab scope.
- Automated exploitation: `commix`, `tplmap`, destructive `sqlmap` modes, AI
  exploit generation, AI payload generation, advanced payload evasion.
- Dangerous file or system wrappers: arbitrary command execution and
  create/modify/delete wrappers copied from HexStrike.
- Destructive activity: data exfiltration, persistence, malware, destructive
  exploitation.

Any forbidden tool may only be added later behind a separate explicit
user-approved lab profile and must never be part of the default Phase 066
workflow.
Any forbidden tool may only be added later behind a separate explicit user-approved lab profile and must never be part of the default Phase 066 workflow.
</tool_policy>

<workstream_traceability>
## 🧭 Workstreams

- `WS-01` through `WS-14` remain the normative execution groups because
  `066-TODO.md` contains zero literal `TASK-NNN` identifiers.
- Each workstream is copied into exactly one `066-NN-PLAN.md` in its
  `<copied_task_rows>` section.
- No workstream is merged away, renumbered, or moved into a parallel planning
  authority.
</workstream_traceability>

<specifics>
## Specific Ideas

- Build the system in dependency order: scope and tool root first, upstream
  references second, skills/prompts next, runners and reports together, Codex
  wiring before portability checks, Docker after local workflow, then tests and
  migration docs.
- Keep static scan, secrets/supply-chain scan, local DAST skip behavior, report
  generation, artifact validation, and Docker report export testable without
  internet access or preinstalled third-party tools.
- Keep default DAST constrained to localhost or explicit local URLs and record
  `dast/skipped.json` when no local target is present.
</specifics>

<todo_literal_mirror>
## 🧾 TODO Literal Mirror

This section mirrors the authoritative workstream rows and closure bullets from
`066-TODO.md` for traceability only. `066-TODO.md` remains normative if any
wording ever diverges.

## 🧭 Workstreams

### 🛑 WS-01: Scope, Safety, And Rules Of Engagement

Goal:
Define the local-only authorized testing boundary that every skill, script, and
agent must enforce.

Required files:

- `.security/scope.yaml`
- `.security/allowed-targets.txt`
- `.security/denied-tools.txt`
- `.github/skills/pentest-local-orchestrator/references/safety-policy.md`
- `scripts/penetration/validate_scope.py`

Implementation requirements:

- `scope.yaml` must include `mode`, `allowed_paths`, `excluded_paths`, `allowed_hosts`, `allowed_urls`, `forbidden`, `rate_limits`, and `evidence_required`.
- Default `allowed_hosts` must be only `127.0.0.1` and `localhost`.
- Default `allowed_urls` must be empty or localhost URLs only.
- `validate_scope.py` must reject public IPs, public DNS names, empty scope for DAST, wildcard hosts, CIDR ranges broader than loopback, and denied tools.
- Dynamic scan scripts must call `validate_scope.py` before running.
- The safety policy must state that source scanning may run without local service targets, but DAST must skip when no valid local targets exist.

Acceptance checks:

- `python3 scripts/penetration/validate_scope.py .security/scope.yaml` returns success for local-only scope.
- A test scope containing `https://example.com` fails unless an explicit future authorization mode is added.
- `rg -n "hydra|john|hashcat|medusa|patator|metasploit|msfvenom|pacu|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security/denied-tools.txt` shows those names only in denylist/safety text and never in an active execution command.

### ⚙️ WS-02: Tool Root And Installation Model

Goal:
Install and manage all new third-party security tools under `tools/penetration/` without contaminating the host or the existing formal-verification tool root.

Required files:

- `tools/penetration/README.md`
- `tools/penetration/manifests/tool-versions.lock`
- `tools/penetration/manifests/upstream-sources.lock`
- `tools/penetration/manifests/checksums.sha256`
- `scripts/penetration/install_pentest_tools.sh`
- `scripts/penetration/check_pentest_tools.sh`
- `.github/skills/pentest-tool-installer/SKILL.md`

Implementation requirements:

- Use `TOOLS_DIR="${Z00Z_PENTEST_TOOLS_DIR:-$ROOT/tools/penetration}"`.
- Create local tool homes: `bin/`, `cargo/`, `go/`, `python/`, `cache/`, `rules/`, `templates/`, `wordlists/`, `upstream/`, and `manifests/`.
- Set `CARGO_HOME`, `GOBIN`, `GOMODCACHE`, `UV_TOOL_DIR`, `UV_TOOL_BIN_DIR`, `PIPX_HOME`, `PIPX_BIN_DIR`, `TRIVY_CACHE_DIR`, and `NUCLEI_TEMPLATES_DIR` under `tools/penetration/`.
- Prefer repository-local wrappers in `tools/penetration/bin/`.
- Record every installed tool version in `tool-versions.lock`.
- Do not use destructive delete commands. Use safe replacement through staging directories and backup suffixes for text manifest rewrites when full overwrite is required.
- System package installation may install minimal prerequisites only when needed, but tool payloads and caches must stay under `tools/penetration/`.

Acceptance checks:

- `bash scripts/penetration/check_pentest_tools.sh --json` writes a machine-readable status artifact.
- `find tools/penetration -maxdepth 3 \( -type f -o -type l \)` shows local tool/wrapper state after install.
- `which semgrep` is not required to point to a global path when `tools/penetration/bin/semgrep` exists.

### 🔑 WS-03: Upstream Reference Capture And Provenance

Goal:
Capture the useful Strix and HexStrike source material with pinned provenance and license awareness.

Required files:

- `tools/penetration/upstream/strix/`
- `tools/penetration/upstream/hexstrike-ai/`
- `tools/penetration/manifests/upstream-sources.lock`
- `.github/skills/pentest-local-orchestrator/references/UPSTREAM-SOURCES.md`
- `.github/skills/pentest-local-orchestrator/references/strix/routing-matrix.md`
- `.github/skills/pentest-tool-installer/references/hexstrike/tool-inventory.md`

Implementation requirements:

- Clone or refresh Strix and HexStrike into `tools/penetration/upstream/` at pinned commits.
- Record repository URL, commit, retrieval timestamp, license, and copied reference paths.
- Mark HexStrike executable Python files as `REFERENCE ONLY - DO NOT RUN` if copied into any skill reference folder.
- Do not activate imported Strix skills directly as `.github/skills` entries; copy only curated references or adapt them into native `pentest-*` skills.
- Preserve Strix framework, protocol, technology, vulnerability, and scan-mode playbooks as references with a routing matrix.
- Mark Strix `sqlmap`, `subfinder`, `agent_browser`, and Caido/Python automation material as reference-only unless a separate lab profile is implemented.
- License summary must state Strix is Apache-2.0 and HexStrike is MIT, backed by their `LICENSE` files.

Acceptance checks:

- `tools/penetration/manifests/upstream-sources.lock` contains both commit hashes.
- `rg -n "REFERENCE ONLY - DO NOT RUN" .github/skills/pentest-*` finds warnings for any copied HexStrike execution reference outside the verbatim upstream clone.
- No warning header is required inside `tools/penetration/upstream/hexstrike-ai/` itself because that directory is a provenance mirror, not an active skill surface.
- `rg -n "Apache License|MIT License" .github/skills/pentest-* tools/penetration/manifests/upstream-sources.lock` finds provenance evidence.
- `rg -n "frameworks|protocols|technologies|vulnerabilities|scan_modes" .github/skills/pentest-local-orchestrator/references/strix/routing-matrix.md` proves the Strix knowledge corpus was not reduced to CLI tools only.

### 📌 WS-04: Generic Skill Family

Goal:
Create reusable Codex/Copilot skills that can migrate to other projects without Z00Z-specific assumptions.

Required skill directories:

- `.github/skills/pentest-local-orchestrator/`
- `.github/skills/pentest-source-aware-sast/`
- `.github/skills/pentest-rust-security/`
- `.github/skills/pentest-secrets-supply-chain/`
- `.github/skills/pentest-local-dast/`
- `.github/skills/pentest-report/`
- `.github/skills/pentest-tool-installer/`

Implementation requirements:

- Every skill must have a complete `SKILL.md` with YAML front matter.
- Every skill description must include trigger language and safety boundaries.
- Skills must call scripts by path rather than retyping long command blocks.
- Skills must write artifacts under `.security-artifacts/<timestamp>/`.
- Skills must be generic by default; any Z00Z-only logic belongs in `z00z-pentest-profile`.
- Do not create a separate active skill for each Strix framework, technology, or vulnerability file. Keep those files as routed references to avoid context bloat and trigger ambiguity.
- `pentest-local-orchestrator` must define quick, standard, and deep modes.
- `pentest-source-aware-sast` must adapt Strix source-aware SAST workflow: Semgrep, deterministic AST targets from Semgrep scope, `sg` or `tree-sitter`, Gitleaks, TruffleHog, and Trivy.
- `pentest-local-dast` must use scoped local-only DAST with bounded `nmap`, `nuclei`, `httpx`, `katana`, `ffuf`, and `gobuster`.
- `pentest-report` must deduplicate scanner results and require evidence mapping before final findings.

Acceptance checks:

- `find .github/skills/pentest-* -maxdepth 2 -name SKILL.md | wc -l` reports at least `7`.
- `find .github/skills -maxdepth 1 -name 'pentest-*' -type d | wc -l` stays intentionally small; any expansion beyond `10` directories requires a written rationale.
- `rg -n "MCP|API key|public target|scanner output" .github/skills/pentest-*` finds explicit safety and evidence rules.
- `rg -n "tools/penetration|scripts/penetration|.security-artifacts" .github/skills/pentest-*` finds consistent paths.

### 🔑 WS-05: Z00Z Profile Skill And Existing Skill Integration

Goal:
Add a Z00Z adapter that composes the generic pentest workflow with existing repository security skills.

Required files:

- `.github/skills/z00z-pentest-profile/SKILL.md`
- `.github/skills/z00z-pentest-profile/references/z00z-invariants.md`
- `.github/skills/z00z-pentest-profile/references/profile-routing.md`
- `.github/prompts/pentest-local-z00z.prompt.md`

Implementation requirements:

- `z00z-pentest-profile` must load `.github/copilot-instructions.md` and `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` before any Z00Z-specific analysis.
- It must route Rust/repo attack-surface discovery through `.github/skills/attack-surfaces-create`.
- It must route cryptography, wallet secrecy, checkpoint, nullifier, settlement, and proof-boundary review through `.github/skills/z00z-crypto-auditor`.
- It must offer `.github/prompts/gsd-audit-4.prompt.md` as the heavy closure path after implementation or remediation.
- It must define Z00Z review lanes: crypto/proof, wallet/keys, storage/checkpoints, rollup/DA, RPC/network, simulator/fixtures, dependencies/supply-chain.
- It must explicitly forbid edits under `crates/z00z_crypto/tari/**`.

Acceptance checks:

- `rg -n "attack-surfaces-create|z00z-crypto-auditor|gsd-audit-4|Z00Z_DESIGN_FOUNDATION|crates/z00z_crypto/tari" .github/skills/z00z-pentest-profile .github/prompts/pentest-local-z00z.prompt.md` finds all required integrations.
- Running the profile in documentation-only dry run produces a lane map without executing DAST.

### ⚙️ WS-06: Local Script Orchestration

Goal:
Provide deterministic local commands that skills can invoke and that humans can run directly.

Required scripts:

- `scripts/penetration/run_local_pentest.sh`
- `scripts/penetration/run_parallel_static.sh`
- `scripts/penetration/run_source_sast.sh`
- `scripts/penetration/run_rust_security.sh`
- `scripts/penetration/run_secrets_supply_chain.sh`
- `scripts/penetration/run_local_dast.sh`
- `scripts/penetration/build_pentest_report.py`
- `scripts/penetration/validate_artifacts.py`

Implementation requirements:

- Scripts must use `set -euo pipefail` where shell is used.
- Scripts must create `.security-artifacts/<timestamp>/` with subdirectories: `sast`, `rust`, `secrets`, `dast`, `report`, `logs`, `raw`, and `normalized`.
- Scripts must derive one shared run timestamp and use it for both `.security-artifacts/<timestamp>/` and `reports/z00z-pentests_report-<timestamp>/`.
- Scripts must never treat tool failure as proof of no issue; they must write `.exit`, `.out`, `.err`, or JSON status files.
- Static runners may run in parallel; DAST must stay bounded and scope-validated.
- `run_local_pentest.sh` must accept `--mode quick|standard|deep`, `--scope .security/scope.yaml`, `--no-dast`, `--static-only`, `--profile generic|z00z`, and `--artifact-dir`.
- `run_parallel_static.sh` must wait for all child processes and preserve each exit code.
- `build_pentest_report.py` must create a preliminary report without exposing secrets and must write the final report set under `reports/z00z-pentests_report-<timestamp>/`.

Acceptance checks:

- `bash -n scripts/penetration/*.sh` passes.
- `python3 -m py_compile scripts/penetration/*.py` passes.
- `scripts/penetration/run_local_pentest.sh --static-only --mode quick --profile generic` creates an artifact tree even when some tools are missing.
- `scripts/penetration/run_local_pentest.sh --static-only --mode quick --profile generic` also creates `reports/z00z-pentests_report-<timestamp>/` for the same run.

### 📌 WS-07: Artifact Schema And Report Contract

Goal:
Make outputs usable by Codex, Copilot, humans, and future automation without reading raw scanner noise.

Required files:

- `.security/report-template.md`
- `.github/skills/pentest-report/references/report-schema.md`
- `scripts/penetration/build_pentest_report.py`
- `scripts/penetration/validate_artifacts.py`

Required artifact shape:

```text
.security-artifacts/<timestamp>/
  manifest.json
  scope.normalized.json
  tool-status.json
  sast/
  rust/
  secrets/
  dast/
  raw/
  normalized/
  report/security-report.md
  logs/
```

Canonical host report root:

```text
reports/z00z-pentests_report-<timestamp>/
  security-report.md
  ...
```

Report requirements:

- The canonical host report root is `reports/z00z-pentests_report-<timestamp>/`; every generated Markdown, HTML, JSON, or other human-facing report artifact must be written there.
- Scope and authorization.
- Tool versions and exact commands.
- Findings summary ordered by severity.
- Per-finding evidence: source file, scanner artifact, local reproduction or proof, confidence, false-positive status.
- Fix recommendation and required regression tests.
- Redaction notes for secrets.
- Open questions and skipped scans with explicit reason.
- Doublecheck section.

Acceptance checks:

- `python3 scripts/penetration/validate_artifacts.py .security-artifacts/<timestamp>` validates manifest, tool status, and report path.
- The artifact manifest records the paired host report directory `reports/z00z-pentests_report-<timestamp>/`.
- A report cannot classify a scanner hit as confirmed unless it has evidence mapping.

### 🛑 WS-08: Local DAST Scope Runner

Goal:
Implement controlled localhost/devnet/staging dynamic scans without public scanning or credential attacks.

Required files:

- `.github/skills/pentest-local-dast/SKILL.md`
- `scripts/penetration/run_local_dast.sh`
- `.github/skills/pentest-local-dast/references/strix/nmap.md`
- `.github/skills/pentest-local-dast/references/strix/nuclei.md`
- `.github/skills/pentest-local-dast/references/strix/httpx.md`
- `.github/skills/pentest-local-dast/references/strix/katana.md`
- `.github/skills/pentest-local-dast/references/strix/ffuf.md`

Implementation requirements:

- Only scan hosts accepted by `validate_scope.py`.
- Use Nmap two-pass model: small discovery first, then service enrichment only on discovered ports.
- Use Nuclei with explicit severity/template/tag bounds, `-ni`, rate limits, concurrency limits, timeouts, retries, and JSONL output.
- Use `httpx` and `katana` only on scoped local URLs.
- Use `ffuf` or `gobuster` with small wordlists, explicit rates, timeouts, and JSON output.
- Do not run Strix Caido proxy automation, `agent_browser`, OAST/interactsh, `sqlmap`, public subdomain discovery, or high-speed network discovery in the default DAST path.
- Skip DAST with a clear artifact when no allowed local target is present.

Acceptance checks:

- A default repo with no running local target produces `dast/skipped.json` rather than a failed run.
- A public target in `scope.yaml` is rejected before any network call.
- `rg -n -- "(^|[[:space:]/])(masscan|rustscan|subfinder|amass|hydra|hashcat|metasploit|msfvenom|sqlmap|commix|tplmap|interactsh)([[:space:]]|$)" scripts/penetration/run_local_dast.sh` finds no command invocation.
- `rg -n "reference-only|Do not run|default DAST path" .github/skills/pentest-local-dast/SKILL.md` documents why those higher-risk tools are absent from the default runner.

### 🔗 WS-09: Codex And Copilot Surface Wiring

Goal:
Expose the new skills, agents, prompts, and scripts consistently through `.github` and `.codex`.

Required files:

- `.github/agents/pentest-rust-reviewer.agent.md`
- `.github/agents/pentest-crypto-reviewer.agent.md`
- `.github/agents/pentest-storage-reviewer.agent.md`
- `.github/agents/pentest-rpc-dast-reviewer.agent.md`
- `.github/agents/pentest-supply-chain-reviewer.agent.md`
- `.github/prompts/pentest-local.prompt.md`
- `.github/prompts/pentest-parallel-review.prompt.md`
- `.github/prompts/pentest-report-doublecheck.prompt.md`
- `.codex` symlink verification documentation, if not already sufficient.

Implementation requirements:

- Keep canonical files under `.github/*`.
- Preserve `.codex/skills -> ../.github/skills`, `.codex/agents -> ../.github/agents`, and `.codex/prompts -> ../.github/prompts`.
- Preserve existing `.codex/hooks`, `.codex/instructions`, `.codex/requirements`, `.codex/scripts`, and `.codex/plugins` symlinks when adding new surfaces.
- Do not create duplicate real directories under `.codex` for surfaces already symlinked.
- Add prompt examples for generic repo mode and Z00Z profile mode.
- Agents must have bounded responsibilities and must not run tools directly unless instructed by the orchestrator prompt.

Acceptance checks:

- `test -L .codex/skills && test "$(readlink .codex/skills)" = "../.github/skills"`.
- `test -L .codex/agents && test "$(readlink .codex/agents)" = "../.github/agents"`.
- `test -L .codex/prompts && test "$(readlink .codex/prompts)" = "../.github/prompts"`.
- `test -L .codex/hooks && test -L .codex/instructions && test -L .codex/requirements && test -L .codex/scripts && test -L .codex/plugins`.
- `find .github/agents -maxdepth 1 -name 'pentest-*.agent.md' | wc -l` reports at least `5`.

### 📦 WS-10: Portable Pack/Unpack Integration

Goal:
Make penetration tooling portable across machines without breaking existing archive behavior or invoking unrelated formal-verification setup.

Required changes:

- `z00z_penetration_tests.sh`
- `pack_z00z_project.sh`
- `unpack_z00z_project.sh`
- `scripts/penetration/check_pentest_tools.sh`
- `scripts/penetration/validate_pentest_docker_scope.py`
- `tools/penetration/docker/run_pentest_container.sh`
- `tools/penetration/manifests/tool-versions.lock`

Implementation requirements:

- Do not pack large generated caches by default: exclude `tools/penetration/cache`, heavy downloaded DBs, and tool build artifacts unless a future `--with-pentest-cache` flag is explicitly added.
- Pack source scripts, skills, prompts, agents, manifests, rules, templates, and lightweight wrappers.
- Preserve symlink manifest behavior for `.codex/*`.
- Add optional restore hook that runs `scripts/penetration/check_pentest_tools.sh` after extraction and reports missing tools without failing the whole restore unless a strict flag is added.
- Add `./z00z_penetration_tests.sh` as the canonical local and Docker entrypoint for penetration tests.
- `./z00z_penetration_tests.sh --docker-sandbox` must call the pentest-only Docker path, not `unpack_z00z_project.sh --docker-sandbox`.
- `./z00z_penetration_tests.sh --docker-sandbox` must create or accept a `pack_z00z_project.sh` tarball artifact and pass that archive into `tools/penetration/docker/run_pentest_container.sh`.
- Supported Docker modes must include `--archive <path>` to reuse an existing packed artifact and `--pack` or default auto-pack behavior to create a fresh artifact before container launch.
- The Docker container must extract the packed artifact into an internal workspace and run pentest checks against that extracted workspace, not against the host checkout mounted as the source tree.
- Docker runs must allocate or accept the paired host report directory `reports/z00z-pentests_report-<timestamp>/`, mount it for output, and copy every report generated in the container into that directory before exit.
- Docker runs must keep container stdout/stderr attached to the invoking terminal so tool progress, warnings, and report-generation messages are visible live on the host.
- `./z00z_penetration_tests.sh` must forward common flags to `scripts/penetration/run_local_pentest.sh`: `--mode quick|standard|deep`, `--scope`, `--static-only`, `--no-dast`, `--profile generic|z00z`, `--artifact-dir`, and `--check-only`.
- Do not use `unpack_z00z_project.sh --docker-sandbox` as the pentest Docker runner because that path runs formal-verification installer and full verification gates.
- Either add an explicit `--profile pentest` or `--skip-formal-verification` restore mode, or create a dedicated `tools/penetration/docker/run_pentest_container.sh` wrapper that reuses only safe extraction, symlink verification, and penetration-tool checks.
- The pentest restore/container path must skip `scripts/install-verification-tools.sh`, `tools/formal_verification/**`, `.github/skills/z00z-full-verify-gate/scripts/full_verify.sh`, and `.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh`.
- If a dedicated pentest Docker image is needed, define it under `tools/penetration/docker/` and make it optional.

Acceptance checks:

- `./pack_z00z_project.sh --output /tmp/z00z-pentest-plan-check.tar.gz --keep-tmp` includes new skills/scripts/manifests and excludes heavy caches.
- `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-plan-check.tar.gz --mode quick --static-only --check-only` runs the pentest-only Docker check path from the packed artifact.
- `./z00z_penetration_tests.sh --docker-sandbox --mode quick --static-only --check-only` creates a fresh packed artifact before launching Docker when `--archive` is not provided.
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-plan-check.tar.gz --mode check-only` verifies `.codex` symlinks and reports penetration tool status without running formal verification.
- Docker report runs leave exported host reports under `reports/z00z-pentests_report-<timestamp>/`.
- `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration` fails if executable script lines invoke `install-verification-tools`, `tools/formal_verification`, `z00z-full-verify-gate`, or `z00z-verification-orchestrator`.
- `tools/penetration/cache` is not present in packed output unless explicitly requested by a future flag.

### 🐳 WS-11: Docker Decision And Isolation Model

Goal:
Use Docker where it adds safety and reproducibility for penetration testing, without making Docker the only path or pulling in formal-verification setup.

Decision:

- Do not use `unpack_z00z_project.sh --docker-sandbox` as-is for pentest work; it is a full project restore verifier, not a pentest sandbox.
- Reuse only safe pieces from pack/unpack: archive extraction model, non-root container user, `.codex` symlink checks, and minimal package bootstrap.
- Do not require Docker for normal local static scans.
- Make Docker optional for dynamic local service testing and heavy toolchain verification.
- For pentest Docker, "heavy toolchain verification" means penetration tool availability and artifact/report checks only, not formal-verification toolchain verification.
- Pentest Docker must be archive-driven: the input is the portable `.tar.gz` produced by `pack_z00z_project.sh`, and the extracted container workspace is the scan root.

Implementation requirements:

- Add `./z00z_penetration_tests.sh` with `--docker-sandbox` support specifically for penetration tests.
- Add `--archive <path>` support to `./z00z_penetration_tests.sh --docker-sandbox`.
- When `--docker-sandbox` is used without `--archive`, `./z00z_penetration_tests.sh` must call `./pack_z00z_project.sh --output <tmp-artifact>` first, then run Docker from that artifact.
- `tools/penetration/docker/run_pentest_container.sh` must require an explicit archive path and must reject direct live-checkout scan mode unless a future `--unsafe-live-checkout` debug flag is added.
- Add `tools/penetration/docker/README.md` describing when to use Docker.
- Add `tools/penetration/docker/run_pentest_container.sh` for a pentest-only container entrypoint.
- Add optional `tools/penetration/docker/Dockerfile` if implementers need a pinned pentest environment after the base local workflow exists.
- `./z00z_penetration_tests.sh --docker-sandbox` must mount the packed artifact read-only, extract it inside the container, and write results only to the configured `.security-artifacts/<timestamp>/` output mount.
- `./z00z_penetration_tests.sh --docker-sandbox` must also mount the paired host report directory `reports/z00z-pentests_report-<timestamp>/` and copy all generated reports there before the container exits.
- Docker container stdout/stderr must remain visible in the host terminal for the full run; detached-only execution is not acceptable for the default workflow.
- Any Docker image must mount the repository read-only for scan-only modes unless a remediation mode is explicitly requested.
- Do not run privileged containers by default.
- Do not mount host Docker socket into the pentest container by default.
- Do not install or execute formal-verification tooling in the pentest Docker image or entrypoint.

Acceptance checks:

- Static scan works without Docker.
- `./z00z_penetration_tests.sh --mode quick --static-only --check-only` works without Docker.
- `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-plan-check.tar.gz --mode quick --static-only --check-only` works with the pentest-only container path.
- `./z00z_penetration_tests.sh --docker-sandbox --mode quick --static-only --check-only` produces a temporary packed artifact and records its path in the run manifest.
- Pentest Docker check mode can run `scripts/penetration/check_pentest_tools.sh` and verify symlinks.
- Pentest Docker logs contain no `install-verification-tools`, `tools/formal_verification`, `full_verify.sh`, or `verification-orchestrator` execution.
- Pentest Docker output is visible in the invoking terminal and generated reports are present on the host under `reports/z00z-pentests_report-<timestamp>/`.
- Any Dockerfile has no default public recon, no credential attack tools, and no MCP server.

### ✅ WS-12: Regression Tests And Self-Tests

Goal:
Prove that the orchestration layer behaves safely before it is used for real audits.

Required tests:

- `tests/penetration/test_scope_validation.py`
- `tests/penetration/test_artifact_schema.py`
- `tests/penetration/test_report_builder.py`
- `tests/penetration/test_tool_manifest.py`
- optional shell tests for `scripts/penetration/*.sh`

Implementation requirements:

- Tests must cover local scope accepted, public scope rejected, denied tool rejected, missing tools recorded, skipped DAST recorded, report redaction, and symlink presence.
- Tests must not require internet access or installed third-party security tools.
- Tests must use fixtures under `tests/penetration/fixtures/`.

Acceptance checks:

- `python3 -m pytest tests/penetration` passes when pytest is available.
- If pytest is unavailable, `python3 -m unittest discover tests/penetration` or a documented fallback runs the core validation tests.
- `bash -n scripts/penetration/*.sh` and `python3 -m py_compile scripts/penetration/*.py` pass.

### 🔑 WS-13: Z00Z Security Execution Prompts

Goal:
Provide repeatable prompts that orchestrate the skills and keep outputs evidence-based.

Required prompt files:

- `.github/prompts/pentest-local.prompt.md`
- `.github/prompts/pentest-parallel-review.prompt.md`
- `.github/prompts/pentest-report-doublecheck.prompt.md`
- `.github/prompts/pentest-local-z00z.prompt.md`

Prompt requirements:

- `pentest-local.prompt.md` must run generic local-only assessment.
- `pentest-parallel-review.prompt.md` must define optional specialist agent lanes and merging rules.
- `pentest-report-doublecheck.prompt.md` must verify report claims against local artifacts and source files.
- `pentest-local-z00z.prompt.md` must include `z00z-pentest-profile`, `attack-surfaces-create`, `z00z-crypto-auditor`, and `gsd-audit-4` routing.

Acceptance checks:

- Prompts mention no MCP default path.
- Prompts tell agents to wait for all parallel lanes and merge findings with dedupe.
- Prompts require scanner findings to be validated before confirmation.

### 📚 WS-14: Documentation And Migration Guide

Goal:
Make the system reusable in other projects without carrying Z00Z-only assumptions.

Required files:

- `tools/penetration/README.md`
- `.github/skills/pentest-local-orchestrator/references/migration-guide.md`
- `.github/skills/pentest-local-orchestrator/references/new-project-checklist.md`
- `.github/skills/z00z-pentest-profile/references/z00z-invariants.md`

Migration guide requirements:

- Explain generic core skill copy procedure.
- Explain how another repository creates its own `project-pentest-profile`.
- Explain required symlinks for Codex compatibility.
- Explain how to replace `z00z-pentest-profile` with another project profile.
- Explain how to add language-specific checks without changing orchestrator semantics.
- Explain tool cache portability and offline mode limitations.

Acceptance checks:

- A reader can identify which files are generic and which are Z00Z-only.
- The guide includes a minimal invocation for Codex and a minimal invocation for GitHub Copilot.
- The guide includes failure modes: missing tools, no local target, public target rejected, scanner false positive, stale upstream reference.

## 🧪 Canonical Verification Commands

Run these after implementation:

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
rg -n "MCP|HexStrike server|Strix runtime|LLM_API_KEY|hydra|john|hashcat|medusa|patator|metasploit|msfvenom|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security
```

Inspect every match from the final `rg` command. Matches are acceptable only when they are denylist entries, safety rationale, or explicit reference-only warnings; they must not be active execution commands.

For Z00Z integration after code is added:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run
```

These Z00Z integration checks are separate from the pentest Docker path and must not be invoked by `tools/penetration/docker/run_pentest_container.sh`.

For pentest Docker portability verification after code is added:

```bash
./pack_z00z_project.sh --output /tmp/z00z-pentest-portable.tar.gz
./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-portable.tar.gz --mode quick --static-only --check-only
tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-portable.tar.gz --mode check-only
python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration
```

## ✅ Completion Definition

Phase 066 is complete only when all of the following are true:

- Generic `pentest-*` skills exist and are usable without Z00Z context.
- `z00z-pentest-profile` exists and routes to existing Z00Z security skills.
- All new third-party tool state lives under `tools/penetration/`.
- `.codex` symlink exposure works without duplicate real skill trees.
- The default workflow requires no MCP, no HexStrike server, no Strix runtime, and no external LLM API key.
- `./z00z_penetration_tests.sh` exists as the canonical pentest entrypoint and supports `--docker-sandbox` without delegating to `unpack_z00z_project.sh --docker-sandbox`.
- `./z00z_penetration_tests.sh --docker-sandbox` runs from a `pack_z00z_project.sh` artifact, either supplied through `--archive` or created before Docker launch.
- Pentest Docker workflow, if implemented, does not install or run formal-verification tooling.
- Final reports are generated under `reports/z00z-pentests_report-<timestamp>/`, and Docker-generated reports are copied back there on the host.
- Docker pentest execution mirrors container output to the invoking terminal instead of hiding it inside detached-only logs.
- Strix framework, protocol, technology, vulnerability, and scan-mode playbooks are preserved as routed references, not dropped and not exploded into dozens of active skills.
- Static scan, secrets/supply-chain scan, local DAST skip behavior, report generation, and artifact validation have tests.
- Pack/unpack either preserves or explicitly reconstructs the penetration tool surfaces.
- `doublecheck` has verified the final documentation, scripts, and generated sample report claims against local source and artifacts.

## 📌 Implementation Order

1. Implement `WS-01` and `WS-02` first; no scan runner should exist before the scope and tool-root contract exists.
2. Implement `WS-03` before copying or adapting upstream references.
3. Implement `WS-04`, `WS-05`, and `WS-13` together so skills and prompts agree.
4. Implement `WS-06`, `WS-07`, and `WS-08` together so runners and reports share artifact schema.
5. Implement `WS-09` before portable verification so symlink behavior is stable.
6. Implement `WS-10` and `WS-11` after local workflow works without Docker, and keep the Docker path pentest-only rather than reusing full project restore verification as-is.
7. Implement `WS-12` and `WS-14` before phase closure.
8. Run full doublecheck and record results in the phase closeout artifact.
</todo_literal_mirror>

<verification_contract>
## 🧪 Canonical Verification Commands
Run these after implementation:
- `bash -n scripts/penetration/*.sh`
- `python3 -m py_compile scripts/penetration/*.py`
- `python3 scripts/penetration/validate_scope.py .security/scope.yaml`
- `bash scripts/penetration/check_pentest_tools.sh --json`
- `test -L .codex/skills`
- `test -L .codex/agents`
- `test -L .codex/prompts`
- `test -L .codex/hooks`
- `test -L .codex/instructions`
- `test -L .codex/requirements`
- `test -L .codex/scripts`
- `test -L .codex/plugins`
- `rg -n "MCP|HexStrike server|Strix runtime|LLM_API_KEY|hydra|john|hashcat|medusa|patator|metasploit|msfvenom|responder|netexec|crackmapexec|commix|tplmap" scripts/penetration .github/skills/pentest-*/SKILL.md .github/prompts/pentest-*.prompt.md .security`

Inspect every match from the final `rg` command. Matches are acceptable only
when they are denylist entries, safety rationale, or explicit reference-only
warnings; they must never be active execution commands.

For Z00Z integration after code is added:
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`

These Z00Z integration checks are separate from the pentest Docker path and
must never be invoked by `tools/penetration/docker/run_pentest_container.sh`.

For pentest Docker portability verification after code is added:
- `./pack_z00z_project.sh --output /tmp/z00z-pentest-portable.tar.gz`
- `./z00z_penetration_tests.sh --docker-sandbox --archive /tmp/z00z-pentest-portable.tar.gz --mode quick --static-only --check-only`
- `tools/penetration/docker/run_pentest_container.sh --archive /tmp/z00z-pentest-portable.tar.gz --mode check-only`
- `python3 scripts/penetration/validate_pentest_docker_scope.py tools/penetration/docker scripts/penetration`

## ✅ Completion Definition

Phase 066 is complete only when all of the following are true:

- Generic `pentest-*` skills exist and are usable without Z00Z context.
- `z00z-pentest-profile` exists and routes to existing Z00Z security skills.
- All new third-party tool state lives under `tools/penetration/`.
- `.codex` symlink exposure works without duplicate real skill trees.
- The default workflow requires no MCP, no HexStrike server, no Strix runtime, and no external LLM API key.
- `./z00z_penetration_tests.sh` exists as the canonical pentest entrypoint and supports `--docker-sandbox` without delegating to `unpack_z00z_project.sh --docker-sandbox`.
- `./z00z_penetration_tests.sh --docker-sandbox` runs from a `pack_z00z_project.sh` artifact, either supplied through `--archive` or created before Docker launch.
- Pentest Docker workflow, if implemented, does not install or run formal-verification tooling.
- Final reports are generated under `reports/z00z-pentests_report-<timestamp>/`, and Docker-generated reports are copied back there on the host.
- Docker pentest execution mirrors container output to the invoking terminal instead of hiding it inside detached-only logs.
- Strix framework, protocol, technology, vulnerability, and scan-mode playbooks are preserved as routed references, not dropped and not exploded into dozens of active skills.
- Static scan, secrets/supply-chain scan, local DAST skip behavior, report generation, and artifact validation have tests.
- Pack/unpack either preserves or explicitly reconstructs the penetration tool surfaces.
- `doublecheck` has verified the final documentation, scripts, and generated sample report claims against local source and artifacts.

## 📌 Implementation Order

1. Implement `WS-01` and `WS-02` first; no scan runner should exist before the
   scope and tool-root contract exists.
2. Implement `WS-03` before copying or adapting upstream references.
3. Implement `WS-04`, `WS-05`, and `WS-13` together so skills and prompts
   agree.
4. Implement `WS-06`, `WS-07`, and `WS-08` together so runners and reports
   share artifact schema.
5. Implement `WS-09` before portable verification so symlink behavior is
   stable.
6. Implement `WS-10` and `WS-11` after local workflow works without Docker, and
   keep the Docker path pentest-only rather than reusing full project restore
   verification as-is.
7. Implement `WS-12` and `WS-14` before phase closure.
8. Run full doublecheck and record results in the phase closeout artifact.
</verification_contract>

<deferred>
## Deferred Ideas

None for local correctness. Future lab profiles may add higher-risk public
recon or exploitation tools only behind explicit separate authorization, but
Phase 066 plans must complete the local default workflow without relying on
those future profiles.
</deferred>

---

*Phase: 066-Strix*
*Context gathered: 2026-07-02 via PRD Express Path*
