---
name: constant-time-analysis
description: Auto-invoked when user wants to check crypto code for timing leaks, review whether secret-dependent branches, divisions, comparisons, table lookups, or memory access patterns are safe, or investigate timing attacks and side-channel risks. Also triggers on constant-time, timing attack, side-channel, secret-dependent control flow, KyberSlash, branchless code, cmov, constant-time compare, and secret-derived division.
---

# Constant-Time Analysis

Analyze security-sensitive code for operations whose execution behavior may vary
with secret data and therefore leak information through timing or related
side channels.

## When to Use

- User asks whether cryptographic or authentication code is constant-time.
- The code handles secret keys, passwords, tokens, plaintext, signatures,
  seeds, MAC checks, or confidential state.
- The task is to review timing attacks, side channels, or secret-dependent
  behavior in arithmetic, branching, comparison, or memory access.
- A code review needs to distinguish between safe constant-time structure and
  risky secret-dependent control flow.
- The code may have been written by a human, AI assistant, or mixed workflow.
  This skill is coder-agnostic: audit the artifact, not the author.

## When Not to Use

- The code operates only on public data where timing variance is irrelevant.
- The task is a general performance review rather than a side-channel review.
- The code is only high-level library wiring and does not reimplement or wrap
  security-sensitive primitives.
- The user wants a full cryptographic audit beyond timing behavior.

## How It Works

1. **Define the secret boundary**
   - Identify which inputs, intermediates, and outputs are secret.
   - Separate secret-derived values from public metadata such as lengths,
     counters, protocol versions, and feature flags.
   - Do not treat every flagged low-level instruction as a vulnerability until
     it is tied back to secret data.

2. **Find secret-dependent operations**
   - Look for branches, early returns, divisions, modulo, secret-indexed table
     lookups, variable-time comparisons, data-dependent loop counts, and
     secret-dependent memory access.
   - Include wrappers, helper functions, and error paths because timing leaks
     often appear outside the obvious crypto primitive.
   - Pay attention to compiler-introduced lowering when reviewing optimized
     code or generated artifacts.

3. **Classify by leak shape**
   - `branch_on_secret`: control flow differs based on secret-derived data.
   - `secret_dependent_division`: division or remainder timing may depend on a
     secret operand.
   - `early_exit_compare`: comparison stops on first mismatch.
   - `secret_indexed_lookup`: memory access pattern depends on secret data.
   - `variable_work_factor`: loop trip count or algorithmic work changes with
     secret-derived values.
   - `needs_dataflow_review`: risky primitive is present but secret influence is
     not yet proven.

4. **Trace dataflow before escalating**
   - For every flagged operation, answer one question first: does the operand,
     predicate, index, or loop bound depend on secret data?
   - Public sizes, constant divisors, fixed protocol branches, and validated
     public selectors may be false positives.
   - Secret-derived masks, coefficients, nonce-dependent table indices, and
     authentication decisions usually require escalation.

5. **Check constant-time substitutes**
   - Prefer constant-time compare functions, branchless select patterns,
     masking, constant-work loops, and constructions designed for secret data.
   - Verify that the substitute does not reintroduce secret-dependent behavior
     through helper code, conversions, or fallback paths.
   - If the code delegates to a vetted library, confirm the sensitive operation
     truly stays inside that library boundary.

6. **Use the strongest available evidence**
   - `Source-level`: enough to report obvious secret-dependent branches,
     comparisons, lookup patterns, and suspicious arithmetic.
   - `Build-artifact`: use optimized IR, MIR, assembly, bytecode, or similar
     artifacts when the environment allows, especially when source looks safe
     but code generation may not preserve the intended constant-time shape.
   - `Runtime`: use measurements only as supporting evidence, not as the sole
     proof of safety. Absence of measured timing differences is not proof of
     constant-time behavior.

7. **Report with severity and confidence**
   - State the secret involved, the risky operation, the evidence source, the
     likely attacker-observable effect, and the confidence level.
   - Distinguish `confirmed_timing_risk` from `needs_secret_dataflow_proof`.
   - Prefer precise findings over broad claims like "not constant-time" when
     only one local construct is proven risky.

8. **Recommend fixes at the right layer**
   - Replace the variable-time primitive rather than hiding it behind another
     wrapper.
   - Reduce exposure by keeping secrets out of generic utility code.
   - Add regression tests or focused review notes for secret-dependent control
     flow so later refactors do not reintroduce the leak.

## High-Risk Patterns

- Secret-dependent `if`, `match`, `switch`, or early return.
- Division or modulo where the operand is secret-derived.
- Equality checks that short-circuit on mismatch.
- Table lookup or array indexing driven by secret bytes or bits.
- Parsing or validation loops whose work varies with secret content.
- Fallback logic that uses a constant-time path only sometimes.

## Evidence Rules

- Do not claim a timing vulnerability just because a dangerous primitive exists.
  First tie it to secret influence.
- Do not claim constant-time safety just because code looks branchless at the
  source level. Lowered code may differ.
- Treat source review, compiler artifacts, and runtime observations as
  different evidence levels. Do not collapse them into one certainty bucket.
- When evidence is incomplete, mark the result as `needs verification` rather
  than overstating certainty.

## Review Output

When using this skill, structure the output around these fields:

- Scope
- Secret boundary
- Findings ordered by severity
- Evidence and confidence for each finding
- False-positive triage notes
- Fix recommendation for each finding
- Remaining verification gaps

## Examples

### Example 1: Rust Signature Path

```text
User: Review this Rust signing path for constant-time issues.
Assistant: First define which scalars, keys, and comparison results are secret,
then inspect branches, divisions, and lookups that depend on them before
escalating any timing-risk finding.
```

### Example 2: C Verification Code

```text
User: Check whether this C verifier leaks through timing.
Assistant: Trace secret-dependent conditions, short-circuit compares, and
secret-indexed memory access, then separate true positives from public-length
or constant-operand false positives.
```

### Example 3: Mixed Human And AI Code

```text
User: Audit this feature branch for side-channel risks. Some files were written
by AI, some by people.
Assistant: Keep the review coder-agnostic, evaluate only the code and available
build artifacts, and report evidence-backed findings about secret-dependent
branches, arithmetic, and memory access.
```

## Notes

- Prefer narrow, evidence-backed findings over generic claims about all crypto
  code in a module.
- Constant-time review is not the same as functional correctness review.
- Timing safety depends on dataflow, lowering, and context; do not skip the
  secret-boundary step.
- If the user asks for a review, findings come first and theory stays minimal.