# Alert Concept Drift Reference

Detailed guidance for separating healthy evolution from suspicious drift when
comparing the current repository to a historical Git anchor.

## Source Precedence

When reconstructing concepts, prefer these evidence sources in order:

1. Public API surfaces and exported contracts
2. Tests that encode invariants or rejection behavior
3. Security or crypto wrapper code that constrains usage
4. Requirements, ADRs, design notes, and normative README sections
5. Planning artifacts that explicitly define accepted behavior
6. Comments and naming as supporting evidence only

Do not classify a drift finding from naming alone when executable evidence is
available.

## Safe Git Access Model

The audit must preserve the user's live worktree exactly as it was before the
comparison started.

### Default Mode: Object-Level Read-Only Access

Prefer these operations first:

- `git show <ref>:<path>`
- `git diff <ref> -- <path>`
- `git ls-tree <ref> <path>`
- `git grep <pattern> <ref> -- <path>`

This mode is safest because it does not switch branches, rewrite files, or
touch uncommitted work.

### Escalation Mode: Isolated Detached Worktree

Only when object-level inspection is insufficient, create a detached worktree
for the historical anchor in an isolated scratch path. A scratch path under
`/tmp` is acceptable, but the key property is isolation, not the exact folder.

Rules:

- Use a detached worktree, not a checkout in the live workspace.
- Treat the detached worktree as read-only.
- Never copy historical files over the live worktree.
- Never stash, clean, or reset the live branch to make room for the audit.
- Remove the scratch worktree only after the audit is complete and only if that
  cleanup does not touch the live workspace.
- Before removing the scratch worktree, require a clean
  `git -C <scratch-worktree> status --porcelain` result.
- Prefer `git worktree remove <scratch-worktree>` for cleanup rather than raw
  filesystem deletion.

### Safe Command Sequence

Recommended command order:

1. object-level read-only Git access
2. detached scratch worktree only if required
3. read-only inspection inside the scratch worktree
4. dirty-state check before cleanup
5. `git worktree remove` cleanup

The safe sequence should never mutate the active workspace branch in order to
make historical comparison easier.

### Forbidden Operations

These operations are out of bounds for this skill:

- `git reset --hard`
- `git checkout -- <path>` on the live workspace
- `git clean -fd` or stronger cleanup forms
- baseline branch checkout in the active worktree
- any restore flow that could overwrite uncommitted user files

## Concept Dimensions

Review these dimensions explicitly.

### Security And Trust Boundaries

Look for:

- fail-open behavior replacing fail-closed behavior
- removed authorization or permission checks
- weaker validation before dangerous actions
- new secret exposure in logs, errors, serialization, or persistence
- widened trust in user input, network data, or config data

Healthy evolution usually means stronger validation, narrower trust, safer
defaults, or clearly documented operational trade-offs.

### Public API And Behavioral Contract

Look for:

- removed or silently reinterpreted public functions, types, fields, or RPCs
- compatibility shims that now behave differently from their names
- migration without versioning, release notes, or compatibility fences
- documentation promising behavior that no longer exists

Healthy evolution usually means additive APIs, explicit deprecation, or a
documented migration path.

### Cryptography And Proof Semantics

Look for:

- formula changes without proof or domain-separation rationale
- changed nullifier meaning, digest framing, or signing scope
- weaker binding between proofs, identities, commitments, or roots
- public serialization or exposure crossing a previously private boundary
- secret-derived behavior moving into non-secret surfaces

Healthy evolution usually means same invariant with a stronger implementation,
or an explicitly documented replacement with matching tests.

### Duplication And Source-Of-Truth Fracture

Look for:

- two helpers that now implement the same concept differently
- copy-pasted validation or encoding logic drifting across files
- local bypasses around canonical utility or crypto wrappers
- compatibility paths becoming silent second implementations

Healthy evolution usually reduces duplication or keeps one canonical source of
truth with explicit adapters around it.

### Architecture And Concept Ownership

Look for:

- concept ownership moving to a new layer without docs, tests, or clear seams
- wrappers leaking dependency-specific types or vendor assumptions
- cross-cutting concerns bypassing the approved shared abstraction layer
- old design constraints still documented but no longer enforced

Healthy evolution usually keeps the same ownership model or documents the new
one clearly enough that another engineer can explain the move.

## Decision Matrix

Use this matrix before escalating a candidate.

| Final Class | Use When | Typical Evidence Pattern |
| --- | --- | --- |
| `expected_evolution` | concept preserved, capability extended, invariant intact | additive code, updated tests, no security or semantic weakening |
| `justified_change` | concept intentionally changed with evidence | ADR, plan, migration note, version gate, replacement tests |
| `suspicious_drift` | concept appears weaker, forked, or undocumented | baseline invariant exists, current behavior moved, docs and tests lag or diverge |
| `critical_regression` | baseline guarantee is materially broken | fail-open, secret leak, signature scope shrink, public API break without fence |
| `ambiguous` | evidence is incomplete or contradictory | stale docs, unclear runtime path, baseline not strong enough |

## Evidence Minimum For A Suspicious Finding

Do not keep a `suspicious_drift` or `critical_regression` item unless the
report includes all of these:

1. the historical concept statement
2. the current code or doc evidence
3. why this is not just additive growth or refactoring
4. the specific invariant or concept that moved
5. the `doublecheck` outcome

If any of the five pieces is missing, downgrade the item to `ambiguous`.

## Healthy Evolution Heuristics

Use these heuristics to clear candidates early when appropriate.

- A rename with equivalent tests and unchanged semantics is not drift.
- A refactor that narrows or centralizes logic is not drift.
- Stronger validation replacing weaker validation is not drift.
- A documented migration with compatibility fences is usually a justified
  change, not suspicious drift.
- Extra telemetry, docs, or helper wrappers are not drift unless they alter the
  underlying behavior or trust model.

## Doublecheck Escalation Rules

Every `suspicious_drift`, `critical_regression`, or `ambiguous` item must be
adversarially re-reviewed.

Ask `doublecheck` to test these alternative explanations:

- stale or lagging documentation
- refactor-only movement with equivalent semantics
- stronger enforcement mislabeled as breakage
- compatibility shim intentionally preserving older contract behavior
- test gaps creating a false appearance of drift

Possible outcomes after doublecheck:

- `confirmed` — keep the finding with its current severity
- `downgraded` — move `critical_regression` to `suspicious_drift` or
  `suspicious_drift` to `ambiguous`
- `cleared` — move the item to `expected_evolution` or `justified_change`
- `blocked` — keep as `ambiguous` and name missing evidence

## Severity Guidance

Use severity only for real drift findings.

- `critical` — broken security or crypto invariant, trust-boundary failure, or
  severe public contract regression
- `high` — strong suspicious drift in architecture, API, or duplication that
  can cause future breakage or hidden divergence
- `medium` — partial semantic weakening, mismatch, or documentation drift with
  operational risk
- `low` — minor conceptual inconsistency that should be cleaned up but does not
  currently threaten correctness

## Review Boundaries

- Do not let a large diff force a large findings list.
- Do not pad the report with harmless churn.
- Do not silently collapse a real concept change into `expected_evolution`
  merely because the code compiles.
- Do not skip historical docs, tests, or planning artifacts if they are needed
  to understand whether the current behavior is still conceptually aligned.
