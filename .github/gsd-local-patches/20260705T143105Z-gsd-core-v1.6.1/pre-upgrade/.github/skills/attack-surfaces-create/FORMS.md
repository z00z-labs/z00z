# Attack Surface Forms

## Inventory-First Audit Report Skeleton

```markdown
# Attack Surface Audit: <scope>

## Deterministic Inventory Summary

- **Scope:** <path[, path2]>
- **Inventory Source:** <helper scripts | cargo metadata | source inspection | mixed>
- **Crates/Modules/Files Inspected:** <counts and names when compact>
- **Symbols Indexed:** <functions, structs, enums, traits, impls, constants, statics, macros>
- **Feature Flags / Build Surfaces:** <summary>
- **Dependency / Cross-Crate Surfaces:** <summary>
- **Coverage Gaps:** <missing tool, generated code, unreadable file, manual review area>

## Architecture Lane Coverage

| Lane | Workspace Owners | High-Risk Trust Path Checked | Boundary Slices | Candidate States | Coverage Result |
| --- | --- | --- | --- | --- | --- |
| <crypto/proof/etc.> | <crate/module/file> | <entry -> sink> | <slices> | <accepted/rejected/deferred/manual> | <covered|partial|manual review> |

## Object And Function Inventory

| Crate | Module/File | Symbol | Kind | Visibility | Security Role | Boundary Slice | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| <crate> | <path> | <symbol> | <fn|struct|enum|trait|impl|flow> | <pub|private> | <validator/parser/crypto/storage/etc.> | <slice> | <notes> |

## Attack Surface By Level

### File Level

| File | Local Invariant | Abuse Case | Control State | Risk |
| --- | --- | --- | --- | --- |

### Module Level

| Module | Boundary Assumption | Illegal Operation | Control State | Risk |
| --- | --- | --- | --- | --- |

### Cross-Module Paths

| Source | Sink | Trust Assumption | Failure Mode | Evidence |
| --- | --- | --- | --- | --- |

### Cross-Crate Paths

| Source Crate | Target Crate | Contract | Drift/Trust Risk | Evidence |
| --- | --- | --- | --- | --- |

## What-If Scenario Matrix

| Component | Level | Scenario | Violated Assumption | Candidate Status | Next Check |
| --- | --- | --- | --- | --- | --- |
| <symbol/path> | <file|module|cross-module|cross-crate> | <what if...> | <assumption> | <accepted|rejected|deferred|needs manual review> | <check> |

## Critical Invariants

- <invariant and owner>

## Abuse Cases And Illegal Operations

- <operation that must be rejected or made unreachable>

## Risk Ranking

| Rank | Title | Severity | Likelihood | Impact Category | Confidence | Status |
| --- | --- | --- | --- | --- | --- | --- |

## Anti-Empty Gate

| Gate | Result | Evidence |
| --- | --- | --- |
| Workspace inventory complete enough | <pass|fail> | <evidence> |
| Mandatory slices covered | <pass|fail> | <evidence> |
| Architecture lanes mapped | <pass|fail> | <evidence> |
| High-impact trust paths checked | <pass|fail> | <evidence> |
| Scenario matrix populated | <pass|fail> | <evidence> |
| Rejected/deferred candidates summarized | <pass|fail> | <evidence> |
| Existing controls reviewed | <pass|fail> | <evidence> |
| DB semantic duplicate check done | <pass|fail> | <evidence> |

## Recommended Tests, Fuzz Targets, And Assertions

- <targeted unit/property/fuzz/integration test or assertion>

## Suggested Hardening Changes

- <code hardening or design boundary change>
```

## Standard Attack Surface Card

```markdown
## Attack Surface: <title>

**Status:** verified
**Severity:** <critical | high | medium | low>
**Confidence:** <high | medium>
**Exploitability:** <high | medium | low>
**Category Domain:** <crypto | auth | replay | privacy | validation | availability | storage | concurrency | supply-chain | configuration>
**Category CWE:** <optional cwe id>
**Attack Class:** <secret-exposure | proof-verification-bypass | fail-open-validation | weak-randomness-for-crypto | constant-time-risk | panic-driven-security-boundary | unsafe-serialization-of-sensitive-state | state-transition-invariant-break | parser-or-deserializer-confusion | cross-module-trust-chain-gap | cross-crate-contract-drift | feature-flag-weakened-path | async-concurrency-state-race | storage-consistency-or-corruption>
**Scope Level:** <file | module | crate | repo>
**Scope Paths:** <path1, path2>
**Affected Symbol:** <function/object/trait/flow>
**Affected Module/Crate:** <module or crate>
**Boundary Slice:** <primary slice>
**Protected Asset:** <asset and security property>
**Trust Boundary:** <boundary>
**Attacker Capability Model:** <realistic attacker assumption>
**Existing Control State:** <missing | partial | present>
**Main Vulnerability:** <single concrete vulnerability statement>
**Violated Assumption:** <assumption that breaks>
**Impact Category:** <fund loss | privacy break | double spend | consensus split | state corruption | DoS | privilege escalation | data leakage | cryptographic unsoundness | invariant violation>
**Blast Radius:** <affected users/assets/modules>

### Threat Model Snapshot

- **Attacker Class:** <external caller | authenticated user | malicious operator | compromised dependency | other>
- **Entry Point:** <attacker-controlled edge>
- **Sink:** <security-relevant sink>
- **Why This Path Is Realistic:** <brief realism statement>

### Implementation Nuance

<why the implementation detail creates or amplifies the attack surface>

### Evidence

- `<path>:<line>` - <evidence summary>
- `<path>:<line>` - <second corroborating point>

### Security Control Review

- **Controls Checked:** <auth, replay, redaction, config, dependency, rate, verifier, etc.>
- **Why Existing Controls Are Insufficient:** <brief explanation>

### Pro-Con Audit

**Pros**
- <why this candidate is credible>

**Cons**
- <what weakens the case>

**Decision:** <accepted | rejected>

### Verification

**Gate:** <passed | failed>
**Reason:** <why>

### Defensive Implementation Contract

- <hardening action 1>
- <hardening action 2>
- <required unit/property/fuzz/integration regression test or verification artifact>

### Residual Risk

<what still remains true after the proposed defense>
```

## Accepted Database Entry Shape

```json
{
  "id": "AS-20260501-001",
  "created_at": "2026-05-01 11:01:13",
  "crate": "z00z_wallets",
  "severity": "high",
  "title": "Sensitive wallet material can cross the operator log boundary",
  "rule_id": "secret-log",
  "attack_class": "secret-exposure",
  "confidence": "high",
  "exploitability": "medium",
  "category": {
    "domain": "privacy",
    "cwe": "CWE-532"
  },
  "scope_level": "crate",
  "scope_paths": ["crates/z00z_wallets"],
  "affected_symbol": "example_symbol",
  "affected_module": "z00z_wallets::example",
  "boundary_slice": "secret handling, storage, and logging",
  "protected_asset": "wallet secret material confidentiality",
  "trust_boundary": "secret handling -> logging",
  "attacker_capability_model": "an operator or log reader can access durable log sinks",
  "existing_control_state": "partial",
  "main_vulnerability": "Plaintext secret or secret-derived material is formatted into a durable sink.",
  "violated_assumption": "debug output never contains secret-derived material",
  "impact_category": "privacy break",
  "blast_radius": "operators and durable log readers can observe wallet-linked secret material",
  "implementation_nuance": "The sink is reachable from production code and the evidence is not confined to tests.",
  "evidence": [
    {"path": "crates/z00z_wallets/src/example.rs", "line": 42, "summary": "debug! macro includes secret token"}
  ],
  "pro_con_audit": {
    "pros": ["multiple corroborating matches"],
    "cons": ["one match is naming-driven only"],
    "decision": "accepted"
  },
  "verification": {
    "passed": true,
    "reason": "candidate exceeded threshold and has production-code evidence"
  },
  "defenses": [
    "replace raw logging with structured redaction",
    "add regression coverage for secret redaction"
  ],
  "residual_risk": "Historical logs may already contain leaked data."
}
```

Notes:

- For new accepted findings, `created_at` must be the real append-time timestamp, not a placeholder such as `00:00:00`.
- If you append into an existing JSONL file, match the whitespace style already used by neighboring accepted rows.

## No-Candidate Report Stub

```markdown
## Scan Result

No candidate passed the pro-con audit and verification gate.

### Anti-Empty Gate

All required coverage gates passed. This is a no-accepted-finding result, not an incomplete scan.

### Rejection Summary

- weak evidence only
- test-only or doc-only match
- no trust boundary
- semantically duplicated existing accepted finding
```

## Incomplete-Audit Report Stub

```markdown
## Scan Result

Incomplete audit. Do not treat this as no candidate.

### Missing Coverage

- <architecture lane, slice, crate, file set, trust path, or DB check not completed>

### Next Required Pass

- <specific command, source file set, or analysis lane required before final admission>
```
