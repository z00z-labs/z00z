---
name: z00z-l1-protocol-model-gate
description: Auto-invoked when user wants to find logic holes, double-spend bugs, invalid checkpoint transitions, broken voucher rights, or protocol model counterexamples in Z00Z. Also triggers on L1 verification, TLA+, TLC, Apalache, Alloy, model checking, checkpoint, settlement, asset, rights, voucher, policy, and state-machine checks.
---

# Z00Z L1 Protocol Model Gate

Run counterexample-oriented protocol and object-model checks for Z00Z state machines and relational models.

## When to Use

Use this skill when changes touch:

- Checkpoint, settlement, HJMT, storage roots, delta application, validator, aggregator, or offline bundle logic.
- Asset, rights, voucher, policy, or ownership models.
- `specs/tla/` or `specs/alloy/` files.

## Workflow

1. Identify the model surface: checkpoint/state machine logic goes to TLA+/TLC and Apalache; rights/voucher/policy goes to Alloy.
2. Run `scripts/run-tla.sh` for TLC checks or syntax checks.
3. Run `scripts/run-apalache.sh` for symbolic bounded checks.
4. Run `scripts/run-alloy.sh` for Alloy checks when a headless runner is configured.
5. If a model does not exist for a security-critical change, report UNKNOWN and name the missing model.

## Gate Criteria

- No double spend, duplicate terminal input, invalid root transition, or unverifiable accepted checkpoint.
- No orphan asset/right, dead voucher, policy escalation, or transfer bypass for non-transferable rights.
- Counterexamples are failures unless the model itself is obsolete and updated in the same change.

## Scripts

```bash
.github/skills/z00z-l1-protocol-model-gate/scripts/run-tla.sh
.github/skills/z00z-l1-protocol-model-gate/scripts/run-apalache.sh
.github/skills/z00z-l1-protocol-model-gate/scripts/run-alloy.sh
```

## Examples

```text
User: Verify this HJMT checkpoint transition change.
Assistant: Runs or requests TLA+/Apalache model checks, then pairs the result with L3 Rust checks.
```

```text
User: Add a new voucher state.
Assistant: Requires an Alloy model update before implementation can be considered verified.
```
