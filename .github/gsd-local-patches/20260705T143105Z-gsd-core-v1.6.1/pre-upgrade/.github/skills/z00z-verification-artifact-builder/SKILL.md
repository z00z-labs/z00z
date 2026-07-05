---
name: z00z-verification-artifact-builder
description: Auto-invoked when the orchestrator must create missing verification artifacts, generate specs, bootstrap models, add fuzz targets, or seed Kani harnesses for Z00Z. Also triggers on missing protocol specs, report-local verification runtimes, report-local fuzz scaffolding, and executable harness seeds.
---

# Z00Z Verification Artifact Builder

Build missing Z00Z verification artifacts directly from the repository structure and current crate surfaces.

This helper skill is subordinate to `z00z-verification-orchestrator`. It is not the user-facing entrypoint. The orchestrator remains the main skill and may call this builder when a scoped verification run is blocked by missing models, missing specs, or missing executable security harnesses.

## When to Use

Use this skill when:

- `z00z-verification-orchestrator` finds missing report-local `specs/` artifacts or missing executable verification/fuzz runtime assets for the current report run.
- The repository has real code surfaces but no formal or executable verification seed for them yet.
- The user wants the orchestrator to bootstrap the verification stack instead of only reporting absence.

## Workflow

1. Inspect workspace packages and map them to Z00Z verification domains:
   - checkpoint/root/delta/aggregator state
   - asset/right/voucher/policy model
   - payment request / receiver card / transcript / domain separation
   - decode / parse / compact-wire / artifact ingest surfaces
2. Generate project-specific bootstrap artifacts only where they are missing:
   - report-local `specs<timestamp>/invariants/*.yaml`
   - report-local `specs<timestamp>/tla/*.tla` and matching `.cfg`
   - report-local `specs<timestamp>/alloy/*.als`
   - report-local `specs<timestamp>/crypto/*.yaml` and `*.md`
   - report-local `specs<timestamp>/tamarin/*.spthy`
   - report-local `specs<timestamp>/proverif/*.pv`
   - report-local verifier runtime files under `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/verification<timestamp>/...`
   - report-local security review artifacts under `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/verification<timestamp>/security/...`, including `.github` prompt-corpus extraction and attack-surface registry seeds
   - report-local fuzz runtime files under `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/fuzz<timestamp>/...`
   - crate-local generated Kani proofs under `crates/*/tests/`
3. Never edit protected vendor code under `crates/z00z_crypto/tari/`.
4. Prefer Z00Z-owned APIs and crate-root exports over dependency-specific internals.
5. Emit a machine-readable summary so the orchestrator can include generated artifact evidence in the final report.
6. Treat repository `.github` skills, agents, prompts, and instructions as adversarial-review input material: extract the security-relevant cues into report-local artifacts so the orchestrator can generate weighted attack hypotheses from them.

## Constraints

- Generated artifacts are bootstrap evidence, not final proof of correctness.
- When an artifact already exists and is not generator-owned, leave it untouched.
- When a generated artifact already exists, refresh it in place.
- Do not write mutable spec, verifier, fuzz, or tmp outputs into repository-root `specs/`, `verification/`, `fuzz/`, or `tmp/`; those belong under the active `reports/z00z-verification-orchestrator-<YYYYMMDD-HHMMSS>/` run root.
- Keep generated text in English.

## Script

```bash
.github/skills/z00z-verification-artifact-builder/scripts/bootstrap-artifacts.py
```

## Examples

```text
User: $z00z-verification-orchestrator fix project
Assistant: Resolves missing formal and executable verification surfaces, calls the artifact builder, reruns the selected gates, and updates the report.
```

```text
User: $z00z-verification-orchestrator find-and-fix crate crates/z00z_wallets
Assistant: Generates missing wallet-focused crypto specs, compact-wire fuzz targets, Kani panic harnesses, and reruns the scoped gates.
```
