# Cleanup Decision Matrix

Use this matrix when a cleanup candidate is real but the safe action is not
obvious.

The goal is not to maximize deletions. The goal is to preserve behavior while
reducing ambiguity.

## How To Use This Matrix

1. Match the finding to the closest case.
2. Check the required evidence.
3. Apply the listed default action.
4. Escalate to `requires clarification` when the evidence is incomplete.

If multiple rows apply, choose the more conservative action.

## Decision Table

| Case | Default action | Required evidence | Auto-fix allowed |
| --- | --- | --- | --- |
| Unused local variable, import, or private helper with no references | Remove | Compiler, linter, or call-site search | Yes |
| Unreachable branch after exhaustive control flow | Remove | Control-flow proof or compiler evidence | Yes |
| Private compatibility shim with no live call sites | Remove | Call-site search plus nearby tests still pass | Yes |
| Legacy or unused module, package, crate, or cross-file component | Remove only after whole-codebase review and per-element validation | Whole-workspace reference search, tests and fixtures search, config and script search, docs search, and dependent compile or test validation | Yes, but only one element at a time |
| Exported function, type, trait, constant, or module that looks unused | Keep or mark `requires clarification` | Cross-workspace usage search and confidence that no external users depend on it | No by default |
| Code referenced by macros, generated code, build scripts, reflection, or external tools | Mark `requires clarification` unless fully proven dead | Direct evidence from macro expansion, generator output, build config, or tool contract | No by default |
| Duplicate logic inside one internal module with the same trust boundary and error semantics | Extract or deduplicate | Side-by-side behavior check and tests or call-path review | Yes |
| Duplicate logic across security boundaries, trust boundaries, or different lifecycle stages | Mark `requires clarification` | Explicit proof that the branches are semantically interchangeable | No by default |
| Legacy path and new path both exist, but compatibility expectations are unclear | Mark `requires clarification` | Changelog, docs, or tests proving the old path is no longer required | No by default |
| Serialization, persistence, protocol, config, or wire-format cleanup | Treat as behavior change | Format contract proof and compatibility proof | No |
| Symbol rename that may affect scripts, tooling, logs, CLI, config, or public APIs | Mark `requires clarification` | Full impact search across code, docs, scripts, and user-facing surfaces | No by default |
| Imports from the same crate, module, package, or namespace are split across many lines without a language-specific reason | Group them into one import statement when the language idiom allows it | Local style check, formatter or linter expectations, and proof that aliases or side-effect imports stay intact | Yes |
| Stale or incorrect comment that contradicts the code | Update or delete comment | Direct code comparison | Yes |
| Missing critical explanation near misuse-prone code | Add concise explanation | Local code evidence showing the misuse risk | Yes |
| Old code that is verbose but still the clearest correct implementation | Leave it alone | None beyond review | Not applicable |

## High-Risk Cases

These cases should default to `requires clarification` unless the evidence is
exceptionally strong:

- module, package, or crate deletion without whole-codebase proof
- exported API cleanup
- config key or environment variable cleanup
- error taxonomy changes
- ordering or retry behavior changes
- cleanup around auth, crypto, permissions, persistence, or boundary validation
- cross-module deduplication that merges independently evolving workflows
- deletion of several legacy elements in one batch

## Evidence Standard

Prefer concrete local evidence over inference.

Strong evidence:

- compiler or linter output
- passing targeted tests that cover the touched path
- exact call-site search showing zero references
- whole-workspace search showing no remaining references before module removal
- module and type analysis showing a branch is unreachable
- docs that still match the current implementation
- formatter or linter rules confirming the grouped import form is acceptable

Weak evidence:

- "this looks old"
- "this seems redundant"
- "no one probably uses this"
- style-based assumptions without call-site proof

Weak evidence is not enough for auto-fix.

## Tie-Break Rules

When a case is still ambiguous after review, apply these tie-break rules in
order:

1. Preserve behavior.
2. Preserve compatibility.
3. Preserve security boundaries.
4. Prefer one-element-at-a-time cleanup.
5. Prefer a smaller change.
6. Prefer deletion only after proof.

## Output Expectations

When this matrix changes the default action, the cleanup report should say so
explicitly.

Example:

- `Marked requires clarification: exported helper appears unused, but the decision matrix blocks auto-removal without external usage proof.`
