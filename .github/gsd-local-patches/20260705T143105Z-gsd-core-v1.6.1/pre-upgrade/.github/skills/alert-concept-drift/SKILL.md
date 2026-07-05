---
name: alert-concept-drift
description: Auto-invoked when user wants to check whether current code drifted from an older commit, compare today's repo against a historical baseline without treating every diff as a bug, or find suspicious semantic drift in security, API contracts, duplication, architecture, or cryptography. Also triggers on concept drift, semantic drift, historical anchor audit, baseline invariant review, regression against old commit, justified evolution vs suspicious change, and commit-to-head drift report.
argument-hint: 'baseline_ref = <commit|tag|branch> scope = <optional path> focus = <security|api|crypto|duplication|architecture|all> report_path = <path>'
---

# Alert Concept Drift

Detect concept drift between a historical Git commit and the current workspace.
Do not stop at line diffs. Reconstruct baseline concepts, compare current
semantics, separate healthy evolution from suspicious drift, and doublecheck
every non-obvious finding before reporting it.

## When to Use

- Use when the user wants more than a raw diff between an old commit and the
  current repository state.
- Use when the task is to tell normal concept evolution apart from security,
  API, cryptography, or architecture drift.
- Use when the user wants a detailed drift report anchored to one historical
  commit, tag, branch tip, or release point.
- Use when suspicious changes need adversarial review before being reported as
  real drift.
- Use when the repository may have evolved correctly in many areas, but the
  user wants to know whether core invariants quietly weakened or forked.

## Required Inputs

- Required: one baseline Git commit, tag, or branch reference.
- Optional: scope limit such as crate path, package, module, or document set.
- Optional: focus dimensions such as `security`, `api`, `crypto`,
  `duplication`, or `architecture`.
- Optional: report output path.

Preferred invocation shape:

- `baseline_ref = <commit|tag|branch>`
- `scope = <optional path>`
- `focus = <security|api|crypto|duplication|architecture|all>`
- `report_path = <path>`

If the baseline reference is missing, stop and ask for it. This skill is
baseline-driven and should not guess the comparison anchor.

## Working Rules

- Do not equate code churn with concept drift.
- Do not report a finding only because a symbol moved, a file was renamed, or a
  refactor changed structure.
- Never switch the current working tree to the baseline branch or commit.
- Never use `git reset --hard`, `git checkout --`, `git clean`, or stash-based
  workspace mutation as part of the comparison workflow.
- Protect uncommitted user files in the current branch: the drift audit must
  not rewrite, discard, or hide them.
- Prefer object-level read-only comparison first: `git show <ref>:<path>`,
  `git diff <ref> -- <path>`, `git ls-tree`, and similar read-only Git access.
- If a materialized historical tree is genuinely required, create an isolated
  detached worktree in a safe scratch location and treat it as read-only.
- Do not compare by opening the baseline directly inside the live workspace
  root; isolate it so current branch state cannot be damaged.
- Treat additive behavior as healthy by default unless it weakens or forks an
  existing invariant.
- Build evidence from both sides of the comparison: historical baseline and
  current workspace.
- Prefer repository evidence over intuition: code, tests, docs, ADRs, planning
  files, and public API surfaces.
- Classify every candidate as one of these outcomes:
  `expected_evolution`, `justified_change`, `suspicious_drift`,
  `critical_regression`, or `ambiguous`.
- Use `spec-to-code-compliance` when a baseline concept comes from specs,
  design docs, README claims, or normative comments.
- Use `security-audit` for trust-boundary, fail-open, secret-handling, or
  authorization drift.
- Use `crypto-architect` for cryptographic formulas, proof semantics, domain
  separation, key derivation, nullifier meaning, commitment handling, or
  signature binding drift.
- Use `code-reviewer` when duplication or architecture drift may be hiding in
  competing logic paths, helper forks, or broken source-of-truth seams.
- Run `doublecheck` on every `suspicious_drift`, `critical_regression`, and
  `ambiguous` candidate before presenting the final verdict.
- Ignore formatting-only changes unless they hide a real semantic move.

## How It Works

1. **Resolve the comparison frame**
   - Confirm the baseline reference and the current review target.
   - Resolve whether the scope is whole-repo or path-bounded.
   - Record the exact references in the report header.
   - Choose the safest Git access mode before reading anything:
     object-level reads first, isolated detached worktree only if object-level
     inspection is not enough.

2. **Establish safe baseline access**
   - Default mode: inspect the historical anchor through read-only Git object
     access without checking out the old branch in the live worktree.
   - Escalation mode: if the audit needs commands that require a physical tree,
     create a detached `git worktree` in a scratch directory such as `/tmp` or
     another isolated path.
   - Never reuse the current worktree for the old branch.
   - Never modify the scratch worktree; it exists only for read-only
     inspection.
   - Record which mode was used in the report so the audit remains reproducible.

3. **Build the historical concept corpus**
   - Read the baseline commit's normative materials with `git show` or
     equivalent repository access.
   - Extract concepts from the baseline state in this priority order:
     public API surfaces, tests that encode invariants, security notes, crypto
     wrappers, requirements or design docs, READMEs, and planning artifacts.
   - Convert those materials into explicit baseline claims instead of keeping
     them as vague impressions.

4. **Build the current concept corpus**
   - Read the current code, tests, docs, and public surfaces for the same
     concept areas.
   - Capture what the current repository actually guarantees, not what naming
     suggests.
   - Separate documented guarantees from undocumented implementation behavior.

5. **Generate candidate drift items**
   - Compare baseline and current concepts across the dimensions in
     `REFERENCE.md`.
   - Create a candidate whenever a guarantee appears weakened, replaced,
     duplicated, undocumented, misordered, or silently reframed.
   - Also create candidates for places where the current system looks stronger
     than the baseline but the change is not clearly explained.

6. **Classify the candidate before escalating**
   - Use the decision matrix in `REFERENCE.md`.
   - `expected_evolution`: same concept, broader capability, invariant intact.
   - `justified_change`: real concept shift, but explicitly documented and
     migrated.
   - `suspicious_drift`: semantic weakening, forked logic, or undocumented
     concept movement.
   - `critical_regression`: security, crypto, or API contract break that
     undermines a baseline guarantee.
   - `ambiguous`: evidence is incomplete or conflicting.

7. **Deepen high-risk dimensions**
   - For security candidates, run `security-audit` on the affected files and
     compare the output to the baseline concept.
   - For cryptographic candidates, run `crypto-architect` and compare formulas,
     domain separation, fail-closed behavior, and proof semantics.
   - For spec or doc claims, run `spec-to-code-compliance` claim-by-claim.
   - For duplication and source-of-truth fractures, inspect whether the current
     repo now contains competing logic paths that can drift independently.

8. **Doublecheck every suspicious item**
   - Send each `suspicious_drift`, `critical_regression`, and `ambiguous`
     candidate through `doublecheck`.
   - Ask `doublecheck` to test alternative explanations: controlled migration,
     renamed seam, stronger enforcement, stale docs, or evidence gap.
   - Do not keep a candidate as a real drift finding unless it survives this
     adversarial pass.

9. **Write the detailed report**
   - Use the templates in `FORMS.md`.
   - Lead with findings, not with the diff summary.
   - Separate healthy evolution from suspicious drift so the user can see which
     changes are acceptable.
   - Include the doublecheck outcome for every non-trivial finding.
   - State whether the audit used object-level Git reads only or an isolated
     detached worktree.

10. **Recommend the next action**
   - For `expected_evolution`, no fix is needed.
   - For `justified_change`, recommend doc or migration reinforcement if the
     explanation is thin.
   - For `suspicious_drift`, propose the narrowest validation or repair step.
   - For `critical_regression`, recommend immediate containment and verification.
   - For `ambiguous`, name the exact evidence still missing.

## Recommended Command Patterns

Use these command patterns when the audit needs reproducible Git access.

### Pattern A: Object-Level Read-Only Audit First

Use this mode by default because it does not materialize the historical tree in
the live workspace.

```bash
# Inspect one historical file
git show <baseline-ref>:path/to/file

# Compare one path without checkout
git diff <baseline-ref> -- path/to/file

# List tree entries under a historical path
git ls-tree -r <baseline-ref> -- path/to/subtree

# Search historical content without switching branches
git grep '<pattern>' <baseline-ref> -- path/to/subtree
```

### Pattern B: Detached Scratch Worktree Escalation

Use this only when object-level reads are not enough.

```bash
# Choose an isolated scratch path
BASELINE_REF='<baseline-ref>'
SCRATCH_ROOT='/tmp/z00z-concept-drift'
SCRATCH_WT="$SCRATCH_ROOT/${BASELINE_REF//\//_}"

# Create a detached worktree without touching the current workspace
mkdir -p "$SCRATCH_ROOT"
git worktree add --detach "$SCRATCH_WT" "$BASELINE_REF"

# Read only from the scratch tree
git -C "$SCRATCH_WT" status --short
git -C "$SCRATCH_WT" rev-parse HEAD
```

### Pattern C: Safe Cleanup Order

Do cleanup only after the audit is complete.

```bash
# Refuse cleanup if the scratch worktree became dirty
if [[ -n "$(git -C "$SCRATCH_WT" status --porcelain)" ]]; then
  echo 'Refusing to remove dirty scratch worktree'
  exit 1
fi

# Remove the detached worktree safely
git worktree remove "$SCRATCH_WT"

# Remove the empty scratch root only if it is empty
rmdir "$SCRATCH_ROOT" 2>/dev/null || true
```

### Pattern D: Forbidden Shortcuts

Never replace the safe flow above with any of these:

```bash
git checkout <baseline-ref>
git switch --detach <baseline-ref>
git reset --hard <baseline-ref>
git clean -fd
git checkout -- path/to/file
git stash push
git stash pop
```

## Output Contract

The result must be a detailed report, not a raw diff dump. By default the
report should contain:

- comparison scope and anchor references
- baseline concept inventory
- current concept inventory summary
- classification table for all material candidates
- confirmed suspicious or critical findings first
- explicitly cleared healthy evolution items
- ambiguous items with missing-evidence notes
- doublecheck ledger
- remediation or follow-up recommendations

When the repository already has a `reports/` directory, prefer writing the
report under a path like:

- `reports/concept-drift/<anchor-slug>-vs-current.md`

Where `<anchor-slug>` is a normalized filename-safe form of the baseline ref:
replace `/`, `\`, `:`, and whitespace with `_`, collapse repeated `_`, and do
not preserve path separators from refs such as `origin/main` or
`release/2026-04`.

Otherwise provide the report in chat and name the recommended output path.

## Supporting Files

- `REFERENCE.md` — source precedence, drift dimensions, decision matrix, and
  doublecheck rules
- `FORMS.md` — detailed report template, candidate table schema, and per-finding
  card templates

## Examples

### Example 1: Whole-Repository Concept Drift Audit

```text
User: Check whether the current repo drifted from commit abc123, but do not give me a raw diff.
Assistant: Builds a baseline concept corpus from commit abc123, compares current security, API, crypto, duplication, and architecture semantics, classifies each candidate, runs doublecheck on suspicious items, and returns a detailed drift report.
```

### Example 2: Crypto-Focused Historical Anchor Review

```text
User: Compare today's wallet and crypto crates against tag v1.7.0 and tell me whether the nullifier and proof semantics drifted or evolved correctly.
Assistant: Limits scope to the relevant crates, extracts the old and new crypto invariants, escalates those candidates through crypto-architect, then doublechecks the suspicious ones before reporting whether the change is healthy evolution or concept drift.
```

### Example 3: API And Duplication Drift Review

```text
User: Audit whether the current API surface and helper structure drifted from release/2026-01 without just flagging refactors.
Assistant: Maps the old public surfaces and source-of-truth seams, checks current equivalents, separates benign moves from duplicated logic forks or undocumented API contract shifts, and writes a report with cleared evolution items distinct from suspicious drift.
```

## Notes

- This skill is for semantic drift and invariant drift, not for ordinary change
  review.
- A large diff can still produce zero real drift findings.
- A tiny diff can still produce a critical regression if it weakens a baseline
  guarantee.
- If the baseline commit itself is inconsistent or under-documented, report the
  ambiguity instead of forcing a confident verdict.