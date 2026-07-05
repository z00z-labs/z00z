---
name: attack-surfaces-resolve
description: Auto-invoked when the user wants to close attack surfaces, fix vulnerabilities from a JSONL database, prove mitigations, add tests, or verify threat findings are resolved. Also triggers on fix attack surface, remediation, vuln mitigation, hardening, closure proof, exploit regression test, anti-test, and defense validation.
argument-hint: 'db_path=<path.jsonl> [surface_id=<id[,id2,...]>] [surface_class=<class[,class2,...]>] [surface_keyword=<kw[,kw2,...]>] [out_spec=<attack-surfaces-resolve-spec.md>] [closure_ledger=<attack-surfaces-closure-ledger.jsonl>] [max_variants=<N>] [top_k=<N>] [plan_only=false]'
---

# Attack Surfaces Resolve

Close verified attack-surface findings from the `attack-surfaces-create` JSONL database by producing a source-backed fix contract, implementing the fix when allowed, adding proof-of-fix tests and anti-tests, rerunning targeted verification, and writing a closure artifact.

This skill must not claim that a finding is `resolved` from reasoning alone. A finding is resolved only after the original attack path is blocked, legitimate behavior is preserved, tests pass, and a focused re-scan or source audit confirms that the same semantic attack surface is not still present.

## When to Use This Skill

Use this skill when the user wants to:

- close one or more findings created by `attack-surfaces-create`
- implement fixes for attack-surface JSONL records
- compare mitigation designs before patching
- add exploit regression tests, negative anti-tests, property tests, fuzz targets, or integration checks
- verify that a mitigation actually closes the violated assumption and trust boundary
- produce a closure ledger for security review

## Required Inputs

- `db_path`: append-only attack-surface JSONL database path

Optional:

- `surface_id`: one or more target finding ids
- `surface_class`: one or more attack classes to focus on
- `surface_keyword`: one or more keywords for targeted filtering
- `out_spec`: default `reports/attack-surfaces/attack-surfaces-resolve-spec.md`
- `closure_ledger`: default `reports/attack-surfaces/attack-surfaces-closure-ledger.jsonl`
- `max_variants`: mitigation candidate count per target finding, default `8`
- `top_k`: retained candidates per target finding, default `3`
- `plan_only`: if `true`, write a plan but mark every target as `plan-only`, not `resolved`

## Input Contract With attack-surfaces-create

Accept records using the persisted fields emitted by `attack-surfaces-create`, especially:

- `id`, `created_at`, `crate`, `severity`, `title`
- `rule_id`, `attack_class`, `confidence`, `exploitability`, `category`
- `scope_level`, `scope_paths`, `affected_symbol`, `affected_module`, `boundary_slice`, `protected_asset`, `trust_boundary`
- `attacker_capability_model`, `existing_control_state`
- `main_vulnerability`, `violated_assumption`, `impact_category`, `blast_radius`
- `implementation_nuance`, `evidence`, `pro_con_audit`, `verification`, `defenses`, `residual_risk`

Do not require or persist deprecated/internal fields such as `signature`, `scan_seed`, `variant_seed`, or `seed_axes`.

## Resolution States

Every selected finding must end with exactly one state:

- `resolved`: patch applied, attack-blocking tests pass, valid-flow tests pass, normal targeted checks pass, and focused re-scan/source audit finds no same semantic surface.
- `mitigated`: exploitability or impact is reduced, but residual risk or uncovered paths remain.
- `blocked`: the finding cannot be safely closed because evidence, reproduction, build/test infrastructure, design authority, or repository access is insufficient.
- `plan-only`: the user requested a plan without code changes, or implementation was explicitly out of scope.

If any targeted finding is not `resolved`, the run status is `partial` or `blocked`, never `fully resolved`.

## Target Coverage Rule

- If filters match no findings, stop with `blocked:no-target`; do not produce a successful closure report.
- If filters match multiple findings, create one closure track for every matched finding.
- Do not hide unresolved findings behind an aggregate candidate or summary.
- An aggregate patch is acceptable only when the report proves closure for each mapped finding independently.

## Closure Gates

Run these gates for each finding. Do not skip gates silently.

1. **Finding Load Gate**
   - Parse the JSONL row.
   - Confirm the target id/filter selects the intended finding.
   - Re-check each evidence path against the current source tree.

2. **Reproduction Or Proof Gate**
   - Reproduce the attack with a failing test, small harness, focused command, or source-backed proof obligation.
   - If the finding cannot be reproduced, write a disproval or uncertainty note and mark `blocked` unless the source proof is still strong enough to justify a defensive patch.

3. **Fix Contract Gate**
   - Map the fix directly to `main_vulnerability`, `violated_assumption`, `trust_boundary`, and `existing_control_state`.
   - State the new invariant in concrete terms.
   - Define fail-closed behavior.

4. **Candidate Selection Gate**
   - Generate diverse mitigation candidates.
   - Keep only candidates with concrete implementation actions, tests, anti-tests, and bounded residual risk.
   - Use pro-con scoring and `doublecheck`; a failed doublecheck blocks selection.

5. **Implementation Gate**
   - Apply the selected patch when write access and task scope allow it.
   - Keep edits scoped to affected modules and tests.
   - Do not modify protected or vendored code unless repository rules explicitly allow it.

6. **Proof-Of-Fix Test Gate**
   - Add at least one negative test or anti-test showing the original attack path is rejected or made unreachable.
   - Add at least one positive test showing legitimate behavior still succeeds.
   - Add property, fuzz, integration, or regression tests when the finding crosses parsing, serialization, crypto, storage, network, or concurrency boundaries.

7. **Verification Gate**
   - Run targeted tests first, then the repository's normal relevant checks.
   - Record exact commands and pass/fail results.
   - Fix failures caused by the patch before claiming closure.

8. **Re-Scan Gate**
   - Re-run `attack-surfaces-create` on the affected scope when feasible, or perform a focused source audit against the original trust boundary.
   - Confirm that no same semantic attack surface remains.

9. **Ledger Gate**
   - Write a closure report and append a closure ledger entry.
   - Keep the original attack-surface database append-only; never delete or rewrite accepted findings to hide history.

## Test And Anti-Test Requirements

Choose tests that match the finding class:

- Parser/serialization: malformed input, trailing bytes, duplicate fields, wrong version, oversized payload, canonical encoding, and valid round trip.
- Crypto/proof: wrong statement, wrong commitment/root, domain mismatch, malformed proof, replayed transcript, nonce reuse, and valid proof acceptance.
- Wallet/secret/privacy: secret export, debug/log serialization, redaction, address/linkability regression, and valid wallet flow.
- Storage/settlement/state: stale root, wrong checkpoint, mismatched proof payload, duplicate application, rollback/replay, and valid state transition.
- RPC/network/operator: unauthenticated access, wrong capability, replayed request, oversized request, debug endpoint exposure, and authorized request.
- Concurrency/state machine: duplicate, reordered, stale, interrupted, and TOCTOU sequences plus a valid ordered sequence.
- Panic/DoS: invalid input returns bounded error instead of panic or unbounded work, plus valid input performance sanity.

## SSoT Candidate Model

Use SSoT as a candidate generator, not as proof of closure.

- Generate internal variant seeds only in memory.
- Never print or persist seeds.
- Rotate mitigation layer, control type, test strategy, and rollout shape.
- For each retained candidate, run three roles:
  - `Generator`: proposes the mitigation.
  - `Critic`: tries bypasses and regression paths.
  - `Selector`: keeps only candidates that can satisfy closure gates.

## Mandatory Doublecheck Gate

For every retained candidate and for the final closure claim, run `doublecheck` against workspace evidence:

```text
/doublecheck verify this attack-surface remediation against the source finding, source code, tests, anti-tests, and re-scan evidence
```

Statuses:

- `pass`: selectable.
- `pass-with-risk`: selectable only if residual risk is explicit and bounded.
- `blocked`: not selectable until missing evidence is produced.
- `fail`: not selectable.

## Output Artifacts

Primary artifact:

- `attack-surfaces-resolve-spec.md`

Default location:

- `reports/attack-surfaces/attack-surfaces-resolve-spec.md`

Closure ledger:

- `reports/attack-surfaces/attack-surfaces-closure-ledger.jsonl`

The report must include:

1. source findings and current evidence validation
2. per-finding lifecycle state
3. candidate mitigations with pro-con, validation, and doublecheck status
4. selected fix contract and implementation summary
5. proof-of-fix test matrix with positive examples and anti-examples
6. exact verification commands and results
7. re-scan or focused source-audit evidence
8. final state per finding and residual risk

Use [FORMS.md](./FORMS.md) for report and ledger structures. Use [REFERENCE.md](./REFERENCE.md) for detailed field semantics and closure rules.

## Reference Execution Sequence

1. Parse inputs and load target findings from `db_path`.
2. Validate evidence paths and current source context.
3. For every selected finding, reproduce the attack or define a source-backed proof obligation.
4. Generate and score mitigation candidates.
5. Run validation A, validation B, and `doublecheck` for retained candidates.
6. Select a candidate per finding, or one aggregate candidate only if it closes all mapped findings with proof.
7. Implement the selected fix unless `plan_only=true`.
8. Add positive tests and adversarial anti-tests.
9. Run targeted checks and relevant normal repository checks.
10. Re-run focused attack-surface scan or source audit for the affected scope.
11. Write the resolve spec and append closure ledger entries.
12. Report `resolved` only for findings that passed every closure gate.

## Acceptance Rules

A finding may be marked `resolved` only if all are true:

- the original JSONL finding is identified by `id` and linked in the report
- current source evidence confirms the affected path still exists or the patch deliberately removed it
- the selected fix maps to the violated assumption and trust boundary
- code changes were applied, unless the finding was already fixed before this run and that is proven by tests and source audit
- at least one anti-test blocks the original attack path
- at least one positive test preserves valid behavior
- all relevant targeted checks pass
- a focused re-scan or source audit finds no same semantic surface
- `doublecheck` passes or passes with explicitly bounded risk

## Examples

### Example 1: Resolve All Findings In A DB

User: "Close everything in the attack-surface DB."

Assistant behavior:

1. Load all accepted findings.
2. Create one closure track per finding.
3. Patch and test each track.
4. Mark the run `partial` if any finding remains blocked or mitigated.
5. Publish the resolve spec and closure ledger.

### Example 2: Close One Specific Surface

User: "Resolve finding as-2026-001 and prove it is fixed."

Assistant behavior:

1. Filter by `surface_id=as-2026-001`.
2. Reproduce the attack or write a source-backed proof obligation.
3. Implement the selected mitigation.
4. Add an anti-test for the original attack and a positive valid-flow test.
5. Run targeted checks and re-scan the affected scope before marking `resolved`.

### Example 3: Plan Only

User: "Give me the mitigation plan, but do not edit files."

Assistant behavior:

1. Generate and doublecheck candidate mitigations.
2. Write implementation tasks and required tests.
3. Mark every finding `plan-only`; do not claim `resolved`.
