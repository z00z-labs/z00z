# Attack Surfaces Resolve Reference

## Objective

Convert verified `attack-surfaces-create` findings into closed remediation tracks. A track is closed only when the implementation, tests, anti-tests, verification commands, and focused re-scan/source audit prove that the original violated assumption is no longer exploitable.

This skill can guarantee claim discipline, not universal fixability: it must refuse to say `resolved` when proof is missing.

## Input Semantics

Expected JSONL fields from `attack-surfaces-create`:

- Identity: `id`, `created_at`, `crate`, `title`, optional `rule_id`
- Risk: `severity`, `confidence`, `exploitability`, `category`, `attack_class`
- Scope: `scope_level`, `scope_paths`, `boundary_slice`, `affected_symbol`, `affected_module`
- Threat model: `protected_asset`, `trust_boundary`, `attacker_capability_model`
- Vulnerability: `existing_control_state`, `main_vulnerability`, `violated_assumption`, `impact_category`, `blast_radius`
- Evidence: `implementation_nuance`, `evidence`, `pro_con_audit`, `verification`, `defenses`, `residual_risk`
- Hints: `recommended_fix`, `recommended_unit_test`, `recommended_property_test`, `recommended_fuzz_test`, `recommended_integration_test`

Missing fields must be treated as uncertainty. Fill gaps from source inspection when possible. If a required closure fact cannot be proven, mark the finding `blocked` or `mitigated`, not `resolved`.

Do not depend on or persist `signature`, `scan_seed`, `variant_seed`, or `seed_axes`.

## Finding Coverage Model

Every selected finding needs its own closure row, even when several findings share one patch.

If filters match no findings, stop with `blocked:no-target`. An empty target set is not a successful resolution run.

Required per-finding coverage:

1. source finding id
2. current evidence check
3. reproduction or source-backed proof obligation
4. selected fix contract
5. implementation diff summary
6. positive example
7. negative anti-example
8. targeted verification commands
9. re-scan/source-audit result
10. final state and residual risk

An aggregate patch is valid only if each mapped finding has independent proof that its original attack path is blocked.

## Resolution States

- `resolved`: all closure gates pass.
- `mitigated`: meaningful hardening landed, but a bypass, uncovered path, operational dependency, or residual risk remains.
- `blocked`: closure cannot proceed safely.
- `plan-only`: no implementation was performed.

Use `mitigated` instead of `resolved` when:

- only monitoring, logging, documentation, or operational process was added
- enforcement is feature-gated or not default-on
- tests cover a narrower path than the original finding
- re-scan finds related semantic risk
- residual risk is material to the protected asset

Use `blocked` when:

- evidence paths are missing and cannot be reconstructed
- the finding cannot be reproduced and source proof is insufficient
- tests cannot run for the affected component
- required architectural authority is missing
- fixing would require modifying protected or vendored code
- a candidate fails doublecheck

## SSoT Candidate Generation

Use SSoT to explore candidate fixes broadly, then prove one fix concretely.

Rotate axes:

- mitigation layer: input boundary, parser, verifier, core invariant, storage, network, runtime, operator workflow
- control type: fail-closed guard, canonicalization, capability check, replay control, cryptographic binding, state transition assertion, rate/bounds check, redaction
- test strategy: unit anti-test, valid-flow unit test, property test, fuzz target, integration adversarial test, source-audit assertion
- rollout shape: direct enforcement, compatibility shim, migration, staged default-on, removal of unsafe path

Each candidate must include:

- implementation actions
- invariant/fix contract
- positive examples
- negative anti-examples
- residual risk
- rollback or migration notes when relevant

## Pro-Con Scoring

Suggested weights:

- closes violated assumption: 30
- covers trust boundary end to end: 25
- attack-blocking anti-test strength: 20
- valid-flow regression protection: 15
- implementation complexity penalty: -10
- operational/migration risk penalty: -10

Candidates with no anti-test or no valid-flow test are non-selectable.

## Double Validation Contract

### Validation A: Structural Completeness

Pass criteria:

- directly names the finding id and source evidence
- states the invariant to enforce
- includes concrete code changes
- includes at least one positive test and one negative anti-test
- defines fail-closed behavior
- avoids placeholder-only language

### Validation B: Adversarial Robustness

Pass criteria:

- attacks realistic bypass paths
- ties controls to attacker capabilities and trust boundary
- handles malformed, replayed, stale, oversized, unauthorized, or mismatched inputs when relevant
- states residual risk honestly
- avoids weakening another security boundary

## Test And Anti-Test Matrix

Use the smallest tests that prove the property, but cover the original attack.

| Finding Area | Required Anti-Examples | Required Positive Examples |
| --- | --- | --- |
| Parser/serialization | malformed bytes, trailing bytes, duplicate fields, wrong version, oversized payload | canonical valid decode, valid round trip |
| Crypto/proof | wrong statement, wrong commitment/root, domain mismatch, malformed proof, replayed transcript, nonce reuse | valid proof/signature succeeds |
| Wallet/secret/privacy | secret in debug/log/export, unsafe serialization, linkability regression | redacted output, valid wallet operation |
| Storage/settlement/state | stale root, wrong checkpoint, mismatched proof payload, duplicate application, rollback/replay | valid transition/checkpoint |
| RPC/network/operator | unauthenticated route, wrong capability, replayed request, oversized request, debug route | authorized bounded request |
| Concurrency/state machine | duplicate event, reordering, stale read, interruption, TOCTOU | valid ordered sequence |
| Panic/DoS | invalid input panics, unbounded loop/work, allocation blowup | bounded error and valid input |

If the original attack class does not fit the table, define a custom negative anti-example that would have exploited the finding before the fix.

## Verification Commands

Record exact commands in the resolve spec:

- focused test command for the anti-test
- focused test command for the positive path
- relevant crate or package test command
- lint/format/build command when normal for the repository
- focused attack-surface re-scan or source-audit command

If a command cannot run, record the blocker and do not mark `resolved`.

## Re-Scan Rule

Preferred: re-run `attack-surfaces-create` scoped to the affected files/modules after the patch.

Acceptable fallback: perform a focused source audit when re-running the full skill is too expensive or unavailable. The source audit must check:

- the original entry point
- the original sink
- the trust boundary
- the newly enforced invariant
- alternate callers and cross-module paths
- whether a semantic duplicate of the same finding still exists

## Closure Ledger Semantics

The original attack-surface JSONL database is append-only and records accepted findings. Do not delete or mutate historical rows to hide resolved findings.

Write a separate closure ledger entry per finding. Ledger entries should include:

- `finding_id`
- `closed_at`
- `state`
- `resolver`
- `source_db_path`
- `changed_files`
- `tests`
- `anti_tests`
- `verification_commands`
- `rescan_result`
- `residual_risk`
- `doublecheck_status`

Only `state=resolved` entries require all gates to pass. `mitigated`, `blocked`, and `plan-only` entries must explain what is missing.

## Output Rule

Primary artifact:

- `attack-surfaces-resolve-spec.md`

Required sections:

- metadata
- target findings
- finding coverage summary
- closure gate matrix
- candidate solutions
- selected fix contract
- implementation patch summary
- test and anti-test matrix
- verification command log
- re-scan/source-audit evidence
- closure decisions
- closure ledger entries
