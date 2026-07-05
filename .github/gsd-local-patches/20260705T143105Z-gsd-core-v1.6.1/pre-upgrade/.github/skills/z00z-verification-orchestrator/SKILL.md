---
name: z00z-verification-orchestrator
description: Auto-invoked when user wants to check Z00Z changes, verify a crate or the whole project, create a report only, fix issues from a report, or find and fix problems immediately. Also triggers on continuous verification, attestation, formal methods, model checking, fuzzing, Kani, Miri, TLA+, Alloy, Tamarin, ProVerif, cargo-audit, cargo-deny, supply-chain review, and security gate orchestration.
argument-hint: "[report crate <crate-name-or-path>|report project|fix report <report-path>|fix crate <crate-name-or-path>|fix project|find-and-fix crate <crate-name-or-path>|find-and-fix project]"
---

# Z00Z Verification Orchestrator

Coordinate the Z00Z continuous verification stack across L0-L4 gates. This skill is an orchestrator and report writer, not a source of truth for correctness.

## 📌 When to Use

Use this skill when:

- The user asks to verify Z00Z changes, check the project, find holes, or run a verification pipeline.
- A task touches checkpoint, settlement, storage, wallet, crypto, proof, parser, dependency, or specification files.
- The user asks for attestation, risk classification, formal verification, model checking, fuzzing, or supply-chain review.

Do not use this skill as a substitute for machine evidence. The final verdict must come from repository files, tests, model checkers, fuzzers, audit tools, or explicit human crypto review.

## ⚙️ Modes

This skill has exactly three operating modes.

### 1. `report`

Create a report only. Do not edit code.

- Supported scopes:
  - `crate <crate-name-or-path>`
  - `project`
- Use this mode when the user wants attestation, inventory, or a facts-first report before any edits.
- For crate scope, run only the narrow gates that make sense for that crate.
- Do not burn heavy cross-project checks on a single module or file. If the user points at a file or module, expand the target to the owning crate first.

### 2. `fix`

Fix all evidence-backed findings from an existing report, or regenerate the same scoped report first if no report path exists yet.

- Supported inputs:
  - `fix report <report-path>`
  - `fix crate <crate-name-or-path>`
  - `fix project`
- Re-validate every finding before editing.
- Apply the final fix set, rerun the relevant gates, and update the report status.

### 3. `find-and-fix`

Run report generation and fixing as one continuous loop.

- Supported scopes:
  - `crate <crate-name-or-path>`
  - `project`
- Find issues, fix them immediately, rerun the relevant gates, and continue until the scoped pass is clean or blocked by a hard repository rule.

## 🚨 YOLO Execution Contract

All three modes run in YOLO mode:

- Do the work end to end without pausing for plan-only handoffs.
- Run the selected gates, inspect failures, patch code or docs when the mode allows it, rerun, and keep going.
- Stop only when the scoped pass is clean, the remaining issue requires human crypto judgment, or a hard repository rule blocks edits.

YOLO mode does not override repository protections. In particular, `crates/z00z_crypto/tari/` remains read-only vendor code and can only receive report findings, not direct edits.

## 🔑 Scope Rules

- The smallest supported scope is a crate. Do not target sub-crate modules as independent verification units.
- Heavy formal or project-wide supply-chain passes are justified for `project` scope and for crate scopes that directly own the relevant protocol, crypto, or parser surface.
- Vendor crates and project-owned crates are both in scope for reporting.
- Fix modes may edit vendor-like code only when repository rules allow it. Protected vendor trees stay report-only, with explicit remediation guidance.

## 🧭 Core Workflow

1. Load repository rules from `.github/copilot-instructions.md` and the Design Foundation before touching files.
2. Parse the requested mode and scope from the user request or `argument-hint`.
3. Resolve any file or module request to the owning crate before selecting gates.
4. Inspect changed files with `git status --short`, `git diff --name-only`, and untracked files relevant to the request.
5. Classify the affected surface:
   - L0 docs/spec/invariants/traceability: `.md`, `.yaml`, `.toml`, `specs/`, `docs/tech-papers/`
   - L1 protocol/state model: `specs/tla/`, `specs/alloy/`, `crates/z00z_storage/`, `crates/z00z_runtime/`, checkpoint or settlement code
   - L2 crypto protocol: `crates/z00z_crypto/`, stealth, inbox, transcript, proof, domain separation, `specs/tamarin/`, `specs/proverif/`, `specs/crypto/`
   - L3 Rust implementation: any `crates/**/*.rs`, tests, examples, benches, or public Rust API changes
   - L4 security engineering: `Cargo.toml`, `Cargo.lock`, fuzz targets, parser/deserializer surfaces, unsafe code, dependency or CI policy
6. Select the narrowest gates that cover the changed surface. For high-assurance or release work, run all gates.
7. Execute gate scripts:
   - L0: `.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`
   - L1: `.github/skills/z00z-l1-protocol-model-gate/scripts/run-tla.sh`, `run-apalache.sh`, `run-alloy.sh`
   - L2: `.github/skills/z00z-l2-crypto-protocol-gate/scripts/check-domain-separation.py`, `check-transcript-binding.py`, `run-proverif.sh`, `run-tamarin.sh`, `run-hax.sh`
   - L2 code-to-logic bridge: `.github/skills/z00z-code-to-logic-gate/scripts/check-refinement-map.py`, `run-cryptol.sh`, `run-saw.sh`, `run-crux-mir.sh`, `run-charon.sh`, `run-aeneas.sh`
   - L3: `.github/skills/z00z-l3-rust-implementation-gate/scripts/verify-fast.sh`, then targeted Kani/Miri/Loom/Verus scripts as relevant
   - L4: `.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`, `unsafe-report.sh`, fuzz scripts, constant-time checks, and adversarial security brainstorming from `.github` prompt/skill corpus plus real code evidence
8. Write findings with evidence-backed semantics:
   - `TESTED`
   - `BOUNDED_VERIFIED`
   - `MODEL_CHECKED`
   - `FORMALLY_PROVED`
   - `SECURITY_PROTOCOL_PROVED`
   - `FAIL`
   - `SKIPPED`
   - `UNKNOWN`
   - `NEEDS_HUMAN_CRYPTO_REVIEW`
   `UNKNOWN` is required when a tool, model, invariant, or spec does not exist yet.
9. In `report` mode, stop after the report is complete.
10. In `fix` mode, map every still-reproducible finding to a concrete edit set, apply it, rerun the relevant gates, and update the report.
11. In `fix` and `find-and-fix`, if required verification artifacts do not exist yet, bootstrap them first from the real Z00Z codebase before rerunning the gates.
12. In `find-and-fix` mode, generate findings, fix immediately, rerun, and loop until the scoped pass is closed.
13. Report only evidence-backed conclusions. Do not claim a proof unless the underlying tool produced a passing result.

## ❗ Decision Rules

- If no invariant exists for new security-critical code, mark the result UNKNOWN and request an invariant before declaring done.
- If a parser or byte-decoder changed and no fuzz target exists, require L4 follow-up.
- If a proof transcript/domain changed, require L2 domain and transcript checks plus human crypto review for new constructions.
- If `Cargo.lock` changed, require supply-chain checks.
- If only docs changed, run L0 first; do not run expensive Rust/formal gates unless the docs change a security invariant.
- If a report finding points into `crates/z00z_crypto/tari/`, keep it in the report and propose the exact remediation path, but do not edit the code there.
- If a `fix` report contains stale findings that no longer reproduce, mark them stale instead of forcing edits.

## 📝 Report Contract

- Write reports under `reports/`; the canonical per-run Markdown filename is `z00z-verification-report.md`.
- The canonical Markdown structure is defined in `FORMAT.md` beside this skill and must be followed exactly by the orchestrator report writer.
- Each verifier run must create a dedicated root directory at `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/`.
- Put the Markdown report, logs, temporary files, tool state, fuzz corpora, model-checker state, supply-chain outputs, and any other mutable verifier outputs under that run root only.
- By default, trash previous stale `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/` directories before allocating a fresh run root, while preserving any report directory explicitly reused by `fix report`.
- Keep caches under the active run root `.cache/` tree; do not spill mutable verifier cache directories into the repository root.
- Treat `repo/.cache` as production or developer-owned cache space, not verifier-owned space. The orchestrator and its child processes must not write there.
- If a deliberate cross-run verifier cache is ever needed, it may live under `reports/.cache/`, never under `repo/.cache/`.
- Heavy Rust compile-and-test stages must run in release mode by default. Do not use debug-mode whole-workspace runs as the standard verifier path.
- Treat release mode as mandatory for downstream cargo-based verifier skills unless a tool fundamentally does not support it.
- Wall-clock timeout cutoffs are disabled by default for orchestrator-managed gates. Let terminating tools finish naturally; only infinite-style campaigns keep their own semantic stop condition.
- Record profiling events for gate-level and shell-managed command-level work under the active run root, and include the slowest top `5%` in the final report.
- Record profiler tool availability, per-gate resource profiles, run-root disk footprint, and any HJMT runtime metrics available under the active run root, then surface them in the final report.
- For `project` scope, also write a tracked-file coverage manifest so the report can show which repository files were actually mapped to active gates and which remain unmapped.
- For security-critical scopes, also run adversarial brainstorming against the real codebase, using the repository's `.github` skills, agents, prompts, and instructions as a weighted threat-review corpus rather than as authoritative proof.
- A report must separate:
  - project-owned fixable findings
  - protected vendor findings
  - missing-evidence or missing-model findings
- The final report must include a dedicated adversarial-security section with file-level, module-level, crate-level, and cross-crate hypotheses, plus the exact report-local artifact paths for the generated scenario inventory.
- When bootstrap generation ran, the report must also show which verification artifacts were generated or refreshed versus which pre-existing manual artifacts were left untouched.
- For protected vendor findings, include the concrete file path, tool output, risk statement, and recommended upstream or local-wrapper remediation.

## 🔧 Script

Use `scripts/orchestrate.sh` for the local orchestrator entrypoint. It understands the same three modes as this skill and can run in dry mode first:

```bash
.github/skills/z00z-verification-orchestrator/scripts/orchestrate.sh report project --dry-run
```

Current script contract:

- `report` writes a scoped report without edits.
- `fix` can reuse `fix report <report-path>` metadata, rerun the scoped gates, apply bounded mechanical fixes, and rewrite the report.
- `find-and-fix` runs the same loop without requiring a pre-existing report.
- Every invocation creates `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/`, writes `z00z-verification-report.md`, and keeps all mutable runtime outputs there.
- Before allocating a fresh run root, stale prior verifier run roots are trashed by default unless `Z00Z_KEEP_PREVIOUS_RUNS=1`. This cleanup also absorbs legacy verifier-owned `reports/repro-simcache.*`, `reports/repro-nextest-simcache.*`, and `reports/z00z-verification-orchestrator-preflight-*` directories so they do not linger as top-level report-root violations.
- Tool caches live under the active run root `.cache/` tree.
- `repo/.cache` is off-limits for orchestrator-managed writes. External production-cache activity may be observed and reported, but not relocated or rewritten by the verifier.
- A future shared verifier accelerator cache, if enabled, must live under `reports/.cache/`.
- Whole-workspace Rust compile/test gates run in release mode by default, and downstream cargo-based verifier scripts must preserve that release-profile contract.
- Time-cutoff defaults are off for orchestrator-managed verifier commands, with profiling captured in `profiling/events.tsv` and summarized into the final Markdown report.
- Vendor requests resolve to the owning crate scope, but protected vendor findings stay report-only through the dedicated vendor unsafe report path.
- `project` reports must distinguish `SKIPPED`, `UNKNOWN`, and `DRY-RUN` from `PASS`, and emit a file-coverage manifest alongside the Markdown report.
- Runtime-bootstrap security artifacts under `verification<timestamp>/security/` feed the adversarial L4 review gate, which can raise `NEEDS_HUMAN_CRYPTO_REVIEW` when high-risk scenarios are generated.

## 📎 Examples

```text
$z00z-verification-orchestrator report crate crates/z00z_storage
```

```text
$z00z-verification-orchestrator report project
```

```text
$z00z-verification-orchestrator fix report reports/z00z-verification-orchestrator-20260616-175700/z00z-verification-report.md
```

```text
$z00z-verification-orchestrator fix crate crates/z00z_crypto
```

```text
$z00z-verification-orchestrator find-and-fix crate crates/z00z_storage
```

```text
$z00z-verification-orchestrator find-and-fix project
```

```text
User: Run report mode for the Tari vendor subtree.
Assistant: Resolves the request to the owning crate scope, runs reporting gates, and keeps protected Tari findings report-only.
```
