# Z00Z Crypto Auditor Forms

Use this file as the canonical output contract for
`z00z-crypto-auditor`.

These forms are intentionally split into two layers:

1. discovery and triage working artifacts;
2. canonical append-only report artifacts.

Do not mix the two layers.

> [!IMPORTANT]
> Discovery output is allowed to be broad and provisional.
>
> The canonical report must stay narrow, append-only, and limited to vetted
> shortlist items.

## When To Use Which Form

| Situation | Form To Use | Output Location |
| --- | --- | --- |
| Generate focus before auditing a mixed scope | `Focus Note And Slice Form` | chat or temporary working note |
| Capture broad candidate gaps | `Candidate Inventory Form` | chat or temporary working note |
| Reduce candidates into decisions | `Shortlist And Triage Form` | chat or temporary working note |
| Audit one trust-boundary seam before merge | `Slice Audit Working Note Form` | temporary slice note |
| Create a new formal report file | `Canonical Report Skeleton` or `templates/z00z-crypto-audit-template.md` | final `report_path` |
| Append one formal audit run | `Append-Only Formal Audit Run Form` | final `report_path` |
| Record one promoted formal finding | `Detailed Finding Card Form` | final `report_path` |
| Close and verify a formal run | `Final Summary Table Form`, `Doublecheck Verification Form`, `Final Verdict And Closure Form` | final `report_path` |

## Core Rules

### Discovery Layer Rules

- Discovery output may include plausible but not-yet-promoted candidates.
- Discovery output must stay repository-backed enough to avoid noise.
- Discovery output must not claim closure.
- Discovery output must classify each candidate as `promote`, `defer`, or
  `drop` before formalization.

### Canonical Report Rules

- The canonical report is append-only.
- The canonical report must contain only vetted shortlist items.
- The canonical report must not embed the full raw candidate inventory unless
  the user explicitly asks for it.
- If `crypto-architect` or `security-audit` ran via manual reasoning, record
  `manual fallback` explicitly.
- If `doublecheck` did not run, the report cannot claim full closure.
- If a field does not apply, write `N/A` and explain why.

## Focus Note And Slice Form

Use this before a broad or mixed-scope audit.

````markdown
## Focus Note

- Primary Focus Note:
  - `<asset + trust boundary + failure mode>`

## Trust-Boundary Slices

1. `<slice 1>`
2. `<slice 2>`
3. `<slice 3>`

## Recommended Starting Slice

- `<slice name>`
- Why:
  - `<why this slice should go first>`
````

## Candidate Inventory Form

Use this for the discovery pass only.

````markdown
## Candidate Inventory

| Candidate ID | Slice | Topic | Suspected Severity | Evidence Stub | Why It Might Matter | Confidence | Promote? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CG-001 | `wallets+storage` | checkpoint proof continuity | HIGH | `store.rs`, `redb_backend_validate.rs` | raw artifact may be misread as canonical evidence | Medium | yes |
````

## Shortlist And Triage Form

Use this after discovery and before any formal report writing.

````markdown
## Triage Summary

- Total Candidates Found: `<count>`
- Promoted To Formal Findings: `<count>`
- Deferred For Manual Follow-Up: `<count>`
- Dropped As Duplicate Or Unsupported: `<count>`

## Shortlist Decisions

| Candidate ID | Slice | Decision | Formal Severity If Promoted | Evidence Basis | Reason |
| --- | --- | --- | --- | --- | --- |
| CG-001 | `wallets+storage` | promote |🟠 HIGH | repository-backed | direct reachability and trust-boundary importance |
| CG-002 | `wallets+simulator` | defer | 🟡 MEDIUM | partial | needs a real call-site proof |
| CG-003 | `crypto+wallets` | drop | N/A | weak | duplicate of CG-001 |
````

## Slice Audit Working Note Form

Use this when auditing one slice before merging only vetted results into the
canonical report.

````markdown
## Slice Audit Working Note

- Slice: `<slice name>`
- Owned Scope:
  - `<path>`
- Adjacent Evidence Used For Context:
  - `<path or N/A>`
- Focus Note:
  - `<focus note>`

### Discovery Output

<insert Candidate Inventory Form>

### Shortlist Output

<insert Shortlist And Triage Form>

### Vetted Findings Ready For Canonical Merge

- `<finding ID and title>`
- `<finding ID and title>`
````

## Canonical Report Skeleton

Use this only for the formal append-only report.

````markdown
# Z00Z Crypto Audit

> [!IMPORTANT]
> This file is append-only. Add each formal audit run as a new dated section.
> Do not overwrite earlier runs.

## Report Metadata

- Report Path: `<path>`
- Scope Origin: `<analyzed_paths summary>`
- Maintainer: `z00z-crypto-auditor`
- Status: `active`

---
````

## Append-Only Formal Audit Run Form

Use this only after shortlist selection.

````markdown
## Audit Run YYYY-MM-DD HH:MM:SS

### Setup

- Report Path: `<path>`
- Focus Note:
  - `<focus note>`
- Output Shape: `<candidates-first | slice-first | formal-only>`
- Audit Passes:
  1. `crypto-architect` or `manual fallback`
  2. `security-audit` or `manual fallback`
  3. repository-first microscopic mapping
  4. `doublecheck` or `verification gap`

### Scope

- Input Type: `<directories | crates | files | phase artifacts | mixed>`
- Analyzed Paths:
  - `<path>`
- Owned In-Scope Artifacts:
  - `<artifact>`
- Adjacent Evidence Used Only For Context:
  - `<artifact or N/A>`
- Explicit Exclusions:
  - `<artifact or none>`
- Entry Points:
  - `<entry point or N/A>`
- High-Value Components:
  - `<component>`
- Scope Derivation Evidence:
  - `<artifact>`

### Trust-Boundary Summary

- Primary Trust Boundaries:
  - `<boundary>`
- Adversaries Or Failure Sources:
  - `<source or N/A>`
- Security And Protocol Invariants:
  - `<invariant>`
- Fail-Closed Requirement:
  - `<statement>`
- Residual Exposure At Start:
  - `<statement or N/A>`

### Verification Model Summary

- Critical User Journeys:
  - `<journey>`
- Security Gaps Under Test:
  - `<gap or N/A>`
- Cryptographic Or Protocol Gaps Under Test:
  - `<gap or N/A>`
- State Transitions Under Test:
  - `<transition>`
- Proof Or Transcript Paths Under Test:
  - `<path>`
- Failure Paths Under Test:
  - `<path>`
- Critical Integration Paths:
  - `<path>`
- Negative Scenarios:
  - `<scenario>`
- Assertions That Prove Correctness:
  - `<assertion>`
- Measurable Success Or Failure Conditions:
  - `<condition>`

### Shortlist Provenance

- Total Candidates Found: `<count>`
- Promoted To Formal Findings: `<count>`
- Deferred For Manual Follow-Up: `<count>`
- Dropped As Duplicate Or Unsupported: `<count>`
- Deferred Topics:
  - `<topic or none>`

### Evidence Reviewed

- Source Code: `<yes/no + note>`
- Tests: `<yes/no + note>`
- Fixtures: `<yes/no + note>`
- Manifests: `<yes/no + note>`
- Dependency Locks: `<yes/no + note>`
- CI Or Workflow Files: `<yes/no + note>`
- Runtime Or Deployment Config: `<yes/no + note>`
- Other: `<artifact or N/A>`

### Confidence Snapshot

- Confidence Level: `<High | Medium | Low>`
- Verified:
  - `<repository-backed statement>`
- Assumed:
  - `<assumption or N/A>`
- Evidence That Would Change This Conclusion:
  - `<evidence>`

### Findings Summary

| Severity | Count | Open Blockers | Notes |
| --- | --- | --- | --- |
| 🔴 CRITICAL | 0 | 0 | |
| 🟠 HIGH | 0 | 0 | |
| 🟡 MEDIUM | 0 | 0 | |
| 🔵 LOW | 0 | 0 | |
| ⚪ INFO | 0 | 0 | |

### Detailed Findings

<append one or more Detailed Finding Card Form blocks here>

### Remediation Guidance

- Immediate Fixes:
  - `<fix or N/A>`
- Deferred Fixes:
  - `<fix or N/A>`
- Dependencies, Refactors, Or Policy Decisions:
  - `<dependency or N/A>`

### Residual Risk And Open Gaps

- Unresolved Blocker-Severity Findings:
  - `<finding or none>`
- Non-Blocking Gaps:
  - `<gap or none>`
- Verification Gaps:
  - `<gap or none>`
- Disputed Or Manual-Review Items:
  - `<item or none>`

### Test Or Verification Next Steps

- Targeted Tests:
  - `<test or N/A>`
- Negative Cases:
  - `<case or N/A>`
- Follow-Up Audit Passes:
  - `<pass or N/A>`
- Re-Run Conditions For Closure:
  - `<condition or N/A>`

### Final Summary Table

<insert Final Summary Table Form>

### Doublecheck Verification

<insert Doublecheck Verification Form>

### Final Verdict

<insert Final Verdict And Closure Form>
````

## Detailed Finding Card Form

````markdown
#### Finding: <ID> -- <Severity Emoji> <Title>

- Severity: `<🔴CRITICAL | 🟠 HIGH | 🟡 MEDIUM | 🔵 LOW | ⚪ INFO>`
- Category: `<crypto | security | protocol | state | documentation | operations | other>`
- Status: `<confirmed issue | likely issue | needs manual review>`
- Confidence: `<🟩 High | 🟧 Medium | 🟦 Low>`
- Component: `<file | module | service | design section>`
- Location: `<path and line reference, or N/A>`
- In-Scope Reason: `<why this belongs to the declared review scope>`
- Evidence Basis: `<repository-backed | tool-reported | inferred | missing evidence>`
- Proof Status: `<proven | partial | inferred | missing evidence>`
- Verification Status: `<self-verified | doublecheck-verified | needs manual verification | verification gap>`
- Blocker To Closure: `<yes | no>`
- Recommended Fix: `<specific corrective action, or N/A>`
- Feasibility In Current Codebase: `<fix now | needs refactor | needs policy decision | needs external dependency | blocked>`
- Residual Risk If Deferred: `<risk that remains if not fixed now, or N/A>`

##### Problem

<precise description of the flaw, gap, or unverifiable claim>

##### Why It Matters

<security, correctness, protocol, operational, or integrity consequence>

##### Impact

<what an attacker, faulty component, or broken assumption can achieve>

##### Reachability / Exploitability

<how the issue is reached, under what preconditions, and whether the path is proven>

##### Violated Boundary Or Assumption

<trust boundary, invariant, threat-model assumption, or design contract that fails>

##### Evidence Reviewed

- <repository-backed artifact>
- <repository-backed artifact>
- <tool output or test evidence, clearly labeled>

##### Relevant Snippet

```text
<code, config, workflow, transcript, or proof snippet if needed>
```

##### Required Follow-Up Validation

- <test, audit pass, property check, negative case, or doublecheck step>

##### Evidence That Would Change This Finding

- Strengthen if: <evidence>
- Weaken or remove if: <evidence>
````

## Final Summary Table Form

````markdown
| ID | Title | Severity | Category | Status | Confidence | Proof Status | Verification Status | Blocker To Closure | Closure State | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CF-001 | <title> | 🟠 HIGH | crypto | confirmed issue | 🟩 High | partial | doublecheck-verified | yes | open | <note> |
````

## Doublecheck Verification Form

````markdown
| Item | Severity | Claim Type | Final Verification Status | Note |
| --- | --- | --- | --- | --- |
| <finding or report section> | <severity> | <finding | summary | verdict | closure state> | <status> | <note> |

> [!IMPORTANT]
> If `doublecheck` did not run, replace the table with a clear verification-gap
> note and keep the report blocked for full closure.
````

## Final Verdict And Closure Form

````markdown
- Audit Result: `<Execution-ready | Ready with conditions | Blocked>`
- Overall Result: `<not closed | partially closed | conditionally closed | closed>`
- Safe To Mark Fully Closed: `<yes | no>`
- Unresolved Blocker Count: `<count>`
- Next Gate:
  - `<next verification gate or N/A>`

- Verdict: `<Execution-ready | Ready with conditions | Blocked>`
- Confidence Level: `<🟩 High | 🟧 Medium | 🟦 Low>`
- Verified:
  - `<repository-backed statement>`
- Remaining Assumptions:
  - `<assumption or N/A>`
- Why This Verdict Applies:
  - `<closure-rule rationale>`
- Evidence That Would Change This Verdict:
  - `<evidence or N/A>`

- Closure State: `<not closed | partially closed | conditionally closed | closed>`
- Unresolved CRITICAL Findings: `<count>`
- Unresolved HIGH Findings: `<count>`
- Non-Blocking Conditions:
  - `<condition or N/A>`
- Closure Reasoning:
  - `<repository-backed statement>`
````
