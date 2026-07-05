---
name: z00z-code-to-logic-gate
description: Auto-invoked when user wants real code-to-logic verification for Z00Z Rust code, SAW/Cryptol equivalence checks, Crux-MIR symbolic execution, or Charon/Aeneas extraction against project-specific targets. Also triggers on transcript equivalence, canonical encoding proofs, serializer injectivity, refinement-map validation, and code-to-spec bridge work.
argument-hint: "[report crate <crate-name-or-path>|report project|fix crate <crate-name-or-path>|fix project|find-and-fix crate <crate-name-or-path>|find-and-fix project]"
---

# Z00Z Code To Logic Gate

Bridge real Z00Z Rust code into machine-checkable logic artifacts. This gate exists to cover the space between model checking and production Rust implementation.

## 📌 When to Use

Use this skill when:

- transcript, challenge, encoding, proof-object, parser, or serialization code changed
- the user asks for code-to-spec equivalence instead of only tests
- the orchestrator needs SAW/Cryptol/Crux-MIR/Charon/Aeneas evidence
- a crypto or proof review needs project-specific artifacts that point at real Rust functions

## ⚙️ Workflow

1. Load the active verifier environment from `../../../scripts/verify-env.sh`.
2. Validate the runtime target map with `scripts/check-refinement-map.py`.
3. Run Cryptol specs from the active report-local runtime root.
4. Run SAW proofs when `.saw` scripts exist.
5. Run Crux-MIR only for explicit configured targets; build-only extraction is not a proof.
6. Run Charon extraction to LLBC for configured crates.
7. Run Aeneas translation only on configured LLBC outputs and treat translation success as artifact generation, not proof completion.
8. Emit only evidence-backed statuses:
   - `FORMALLY_PROVED` for successful SAW proof scripts
   - `BOUNDED_VERIFIED` for successful Crux-MIR symbolic targets
   - `TESTED` for successful pure extraction/translation artifacts
   - `UNKNOWN` when spec, target, or tool is missing
   - `FAIL` on proof failure or counterexample

## 🔧 Scripts

```bash
python3 .github/skills/z00z-code-to-logic-gate/scripts/check-refinement-map.py
.github/skills/z00z-code-to-logic-gate/scripts/run-cryptol.sh
.github/skills/z00z-code-to-logic-gate/scripts/run-saw.sh
.github/skills/z00z-code-to-logic-gate/scripts/run-crux-mir.sh
.github/skills/z00z-code-to-logic-gate/scripts/run-charon.sh
.github/skills/z00z-code-to-logic-gate/scripts/run-aeneas.sh
```

## 📎 Examples

```text
$z00z-code-to-logic-gate report crate crates/z00z_crypto
```

```text
$z00z-code-to-logic-gate report project
```

```text
User: verify transcript and canonical encoding against formal specs
Assistant: validates the refinement map, runs Cryptol and SAW targets, and reports FORMALLY_PROVED only for completed proof scripts.
```
