---
name: z00z-crypto-auditor
agent: agent
description: Auto-invoked when the user wants to find suspicious Z00Z crypto or security gaps, build a broad candidate list first, and then convert vetted items into a formal append-only Z00Z-CRYPTO-AUDIT report. Use for exploratory crypto audit, cryptographic gap mapping, nullifier review, checkpoint proof continuity, wallet secret transport, authority or root lifecycle review, proof-gap triage, slice-by-slice audit, and final audit closure.
argument-hint: analyzed_paths=[path1 path2 ...] report_path=<path-to-Z00Z-CRYPTO-AUDIT.md> focus_note="<topic>" expected_output_shape="<candidates-first | slice-first | formal-only>"
---

# 🔐 Z00Z Crypto and Security Auditor

Use this skill to audit scoped Z00Z cryptographic or security surfaces without collapsing immediately into final-report closeout mode.

## 🎯 What This Skill Optimizes For

This skill is intentionally two-phase:

1. Discovery breadth first.
2. Formal defensible reporting second.

The goal is to avoid a common failure mode in crypto audits: over-optimizing for strictness, formal traceability, and post-hoc defensibility so early that the model stops surfacing good candidate gaps.

This skill should behave like:

- a broad hunter of candidate gaps during discovery;
- a strict canonizer only after shortlist selection;
- a fail-closed reporter for the final append-only artifact.

## 📌 When to Use This Skill

Use this skill when the user wants a Z00Z-specific crypto or security audit over known repository paths and expects a repository-backed result.

- The user wants a Z00Z crypto audit or security audit.
- The user wants a broad candidate inventory before formal findings.
- The user wants a cryptographic gap map, proof-gap map, or trust-boundary review.
- The user wants to inspect nullifier semantics, checkpoint proof continuity, wallet secret transport, authority lifecycle, or root lifecycle.
- The user wants a formal `Z00Z-CRYPTO-AUDIT.md` style report after triage.
- The user wants a phase-local slice audit and later merge into a canonical report.

## 🚫 Do Not Use This Skill

Do not use this skill when the user wants a normal bug fix, direct implementation work, or a generic code review without a scoped audit question.

- Do not use it as a substitute for normal refactoring.
- Do not use it when scope cannot be tied to explicit repository paths.
- Do not use it when the user wants only a final polished report and refuses a discovery pass on a large or mixed scope.
- Do not use it to widen ownership beyond the declared audit slice.

## 📥 Required Inputs

Required:

- `analyzed_paths`: one or more repository paths.
- `report_path`: the final append-only report destination.

Operationally required before audit execution:

- `focus_note`: one sentence that names the asset, trust boundary, and failure mode.

Recommended:

- `expected_output_shape`: preferred output mode.

If `focus_note` is missing, generate it before audit execution.

### 💡 Focus Note Rule

If no focus note is provided, invoke the `brainstorming` skill first and synthesize:

- one primary focus note;
- two to five trust-boundary slices;
- one recommended slice to start with.

Good focus notes look like:

- `nullifier semantics at the validator-facing spend boundary`
- `checkpoint proof continuity across wallet and storage persistence seams`
- `wallet secret transport across debug export and session unlock flows`

If focus generation still fails, stop and report the ambiguity instead of auditing a broad undefined surface.

## 🧭 Default Output Shape

If the user does not specify `expected_output_shape`, use:

- `candidates-first` for mixed or large scopes;
- `slice-first` when the scope spans multiple trust boundaries;
- `formal-only` only for scopes that are already narrow, already triaged, and explicitly requested for direct formalization.

Default meaning:

- `candidates-first`: discovery inventory first, shortlist second, formal report last.
- `slice-first`: split by trust boundary, audit one slice at a time, then merge final findings.
- `formal-only`: only for small, already-focused scopes where discovery is intentionally skipped by explicit user choice.

## ⚙️ Core Operating Rules

### 🔎 1. Separate Discovery From Reporting

Do not start by writing heavyweight finding cards into the final report.

Always use this progression unless the scope is already surgically narrow:

1. candidate inventory;
2. triage and shortlist;
3. formal finding cards;
4. append-only closeout;
5. `doublecheck`.

### 🧩 2. Slice Scope By Trust Boundary

Do not audit five related crates as one undifferentiated blob if the actual issues cluster around distinct seams.

Prefer slices such as:

- `wallets + storage` for checkpoint or proof transport;
- `wallets + simulator` for spend or nullifier boundary behavior;
- `crypto + wallets` for authority, key, or root lifecycle.

When the user passes a broad scope, derive slices first and recommend the starting slice explicitly.

### 🔗 3. Allow Adjacent Evidence Without Scope Drift

Do not widen ownership beyond `analyzed_paths`, but do allow adjacent repository evidence from immediate integration seams when it materially explains reachability, misuse risk, or trust-boundary behavior.

Rules:

- adjacent evidence may justify or narrow a finding;
- adjacent evidence must not silently promote a new crate into owned audit scope;
- the report must state when adjacent evidence was used only as explanatory context.

### 📊 5. Small Final Finding Count Is Not Proof Of Safety

Interpret a small formal findings set as a high proof threshold, not as proof that no other suspicious areas exist.

The audit output must distinguish:

- candidates found;
- candidates dropped after triage;
- candidates deferred due to evidence limits;
- formal findings that survived full proof pressure.

## 🛠️ How It Works

### 📌 Phase 0: Normalize Scope

1. Read `analyzed_paths` fully.
2. Derive owned scope, neighboring integration seams, and explicit exclusions.
3. Partition the work into trust-boundary slices when the scope is mixed.
4. State the recommended slice order before any formal reporting begins.

Fail closed if the provided paths do not safely reveal the auditable surface.

### 🔎 Phase 1: Discovery Pass

This phase is read-only and must not append to the final report yet.

Goal:

- collect a broad candidate inventory, typically 15 to 40 candidates for a large scope and fewer for narrow scopes.

Discovery rules:

- use lighter-weight notes instead of full finding cards;
- keep disputed or weak candidates if they are plausible and scoped;
- search for protocol seams, proof seams, state seams, fail-closed gaps, downgrade paths, stub-versus-real drift, comments that overclaim, and tests that pin residual gaps;
- prefer breadth over closure language;
- do not announce verdicts here.

Required discovery output:

- focus note;
- slice list;
- candidate inventory table.

Use this minimum table during discovery:

```markdown
| Candidate ID | Slice | Topic | Suspected Severity | Evidence Stub | Confidence | Promote? |
| --- | --- | --- | --- | --- | --- | --- |
| CG-001 | wallets+storage | checkpoint proof continuity | HIGH | `store.rs`, `redb_backend_validate.rs` | Medium | yes |
```

### ✂️ Phase 2: Triage And Shortlist

Cluster and reduce the candidate inventory.

Promotion criteria:

- direct reachability;
- trust-boundary importance;
- repository-backed evidence quality;
- non-duplication;
- explanatory value;
- realistic remediation path.

Every shortlist decision must classify each candidate as one of:

- `promote to formal finding`;
- `defer for manual follow-up`;
- `drop as duplicate or unsupported`.

Before formalization, state:

- total candidates found;
- how many were promoted;
- how many were deferred;
- how many were dropped.

### 🧾 Phase 3: Formal Audit Pass

Only after shortlist selection may the skill write formal findings to `report_path`.

Rules:

- initialize the report from `templates/z00z-crypto-audit-template.md` only if it does not exist;
- use `FORMS.md` as the canonical report contract;
- append only;
- formalize only vetted shortlist items;
- keep the final report narrow and defensible even if discovery was broad.

For canonical reports, prefer a slice-first approach:

1. audit one trust-boundary slice;
2. optionally draft in a temporary slice note or temporary slice report;
3. merge only vetted findings into the canonical `report_path`.

Do not begin with the final canonical report as if the entire broad scope were already in closeout mode.

### ✅ Phase 4: Verification And Closeout

Run these passes for the formalized findings set:

1. MANDATORY to run `crypto-architect` as auditor;
2. MANDATORY to run `security-audit`;
3. repository-first microscopic mapping;
4. MANDATORY to run `doublecheck` on the final formal report.

If `doublecheck` is unavailable, mark the report blocked for full closure.

## 🧪 Verification Model

Use a lighter model during discovery and a strict model during formalization.

### 🔎 Discovery Verification Model

Discovery only needs enough rigor to avoid garbage candidates.

For each candidate, capture:

- why it is in scope;
- one or more evidence stubs;
- why it might matter;
- what evidence would be needed to promote it.

Do not require the full finding-card burden yet.

### 🧾 Formal Verification Model

### ⚖️ Validate Every Finding

For each finding or material claim, record:

- status: `confirmed issue`, `likely issue`, or `needs manual review`;
- severity, using the skill's severity taxonomy;
- confidence: `🟩 High`, `🟧 Medium`, or `🟦 Low`;
- exploitability or failure impact;
- reachability;
- evidence basis;
- exact blocker to closure, if any;
- what evidence would upgrade, downgrade, or remove the finding.

If the skill has no defined severity taxonomy, use:

- `🔴 CRITICAL`
- `🟠 HIGH`
- `🟡 MEDIUM`
- `🔵 LOW`
- `⚪ INFO`

For promoted findings, prove:

- scope relevance;
- trust boundary or invariant at stake;
- reachability or realistic failure path;
- evidence class used;
- remaining uncertainty;
- what would remove the finding.

During formalization, retain the stricter closure model:

- repository-backed proof over optimistic language;
- fail closed on ambiguity;
- distinguish proven;
- state confidence explicitly.

## 🧱 Report Contract

The formal report remains the product of Phase 3 onward.

### ⚖️ Severity Legend

If the skill has no defined severity taxonomy, use:

- `🔴 CRITICAL`
- `🟠 HIGH`
- `🟡 MEDIUM`
- `🔵 LOW`
- `⚪ INFO`

### 🔔 Allowed Alert Blocks

Use GitHub alert blocks where they materially improve readability.

> [!NOTE]
> Highlights information that users should take into account, even when skimming.

> [!TIP]
> Optional information to help a user be more successful.

> [!IMPORTANT]
> Crucial information necessary for users to succeed.

> [!WARNING]
> Critical content demanding immediate user attention due to potential risks.

> [!CAUTION]
> Negative potential consequences of an action.

Use:

- `FORMS.md` for the canonical layout;
- `templates/z00z-crypto-audit-template.md` for new reports.

The report must include:

1. setup and scope;
2. trust-boundary summary;
3. verification model summary;
4. shortlist provenance;
5. findings summary;
6. detailed finding cards;
7. remediation guidance;
8. residual risk;
9. test or verification next steps;
10. final summary table;
11. `doublecheck` verification block;
12. final verdict and closure state.

Do not store the raw broad candidate inventory inside the canonical append-only report unless the user explicitly asks for it.

## 📚 Mandatory Context

Read and follow these files before and during execution:

- [Z00Z Design Foundation](../../requirements/Z00Z_DESIGN_FOUNDATION.md)
- [Z00Z Copilot Instructions](../../copilot-instructions.md)
- [FORMS.md](./FORMS.md)
- [Report Template](./templates/z00z-crypto-audit-template.md)

## ✅ Completion Criteria

This skill is complete only when all of the following are true:

- `analyzed_paths` were read fully;
- a focus note exists;
- trust-boundary slices were derived when needed;
- discovery output was produced before formal closeout on broad scopes;
- candidates, deferred items, and dropped items were accounted for;
- only vetted shortlist items were formalized;
- the final report was appended in canonical format;
- `doublecheck` verified the final formal report.

## 🛑 Non-Negotiable Rules

- Do not skip the discovery pass on broad or mixed scope.
- Do not skip `doublecheck` on the final formal report.
- Do not overwrite the canonical audit report.
- Do not confuse adjacent evidence with scope ownership.
- Do not convert uncertainty into closure language.
- Keep repository artifacts in English.

## 📎 Examples

Broad exploratory audit with candidates-first behavior:

```text
/z00z-crypto-auditor analyzed_paths=[crates/z00z_wallets crates/z00z_storage] report_path=.planning/phases/057-crypto-fixes/Z00Z-CRYPTO-AUDIT.md focus_note="checkpoint proof continuity across wallet and storage persistence seams" expected_output_shape="candidates-first"
```

Trust-boundary slice first, then formal merge:

```text
/z00z-crypto-auditor analyzed_paths=[crates/z00z_wallets crates/z00z_simulator] report_path=.planning/phases/057-crypto-fixes/Z00Z-CRYPTO-AUDIT.md focus_note="nullifier semantics at the validator-facing spend boundary" expected_output_shape="slice-first"
```

Already-triaged narrow scope:

```text
/z00z-crypto-auditor analyzed_paths=[crates/z00z_crypto crates/z00z_wallets] report_path=docs/code-review/Z00Z-CRYPTO-AUDIT.md focus_note="authority root lifecycle for wallet-bound claim flows" expected_output_shape="formal-only"
```

