---
name: z00z-l2-crypto-protocol-gate
description: Auto-invoked when user wants to check Z00Z crypto protocols, stealth delivery, inbox replay, proof transcript binding, domain separation, or constant-time leakage risk. Also triggers on L2 verification, Tamarin, ProVerif, hax/hacspec, EasyCrypt, dudect, Fiat-Shamir, challenge binding, proof objects, and cryptographic protocol review.
---

# Z00Z L2 Crypto Protocol Gate

Verify cryptographic protocol structure and reject claims that are not backed by protocol tools, domain registries, transcript binding checks, or human cryptographer review.

## When to Use

Use this skill when changes touch:

- `crates/z00z_crypto/`, proof object parsing, transcript labels, domain separation, commitments, stealth address, inbox, payment request, wallet delivery, or timing-sensitive benches.
- `specs/tamarin/`, `specs/proverif/`, or `specs/crypto/`.
- Any new cryptographic construction or protocol glue.

## Workflow

1. Check domain labels with `scripts/check-domain-separation.py`.
2. Check transcript/proof binding expectations with `scripts/check-transcript-binding.py`.
3. Run ProVerif/Tamarin models when matching specs exist.
4. Run `scripts/run-hax.sh` when verifier-owned HAX or EasyCrypt extraction targets exist for the touched crypto surface.
5. When the orchestrator exposes code-to-logic targets, run the dedicated `z00z-code-to-logic-gate` for Cryptol, SAW, Crux-MIR, Charon, and Aeneas evidence.
6. Run dudect or timing harnesses only for operations with explicit constant-time targets.
7. Mark new cryptographic constructions as NEEDS_HUMAN_CRYPTO_REVIEW unless they are only wiring audited primitives with no new security claim.

## Gate Criteria

- No duplicate proof/challenge/domain labels.
- Fiat-Shamir transcript binding must cover protocol ID, version, root/state context, input/output references, proof type, and domain-specific identifiers when applicable.
- Inbox notification or delivery metadata must never become spend authority.
- Any cargo-backed extraction or timing harnesses invoked by the orchestrator must inherit the release-profile default unless the underlying tool lacks a profile switch.
- Orchestrator-managed L2 shell gates disable wall-clock timeout cutoffs by default and record command-level profiling into the active verifier run root.
- Timing checks are statistical signals, not proofs; failures require investigation.

## Scripts

```bash
python3 .github/skills/z00z-l2-crypto-protocol-gate/scripts/check-domain-separation.py
python3 .github/skills/z00z-l2-crypto-protocol-gate/scripts/check-transcript-binding.py
.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-proverif.sh
.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-tamarin.sh
.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-hax.sh
.github/skills/z00z-l2-crypto-protocol-gate/scripts/run-dudect.sh
```

## Examples

```text
User: I added a new TxProof challenge.
Assistant: Checks domain uniqueness, transcript fields, L3 parsing tests, and flags human crypto review if a new construction is introduced.
```

```text
User: Verify stealth inbox replay resistance.
Assistant: Runs ProVerif/Tamarin models when present and reports UNKNOWN if the model is missing.
```
