---
id: dapps.security-model
title: "dApps: How dApps Work Safely"
route: none
scope: guide
---

# dApps: How dApps Work Safely

[TOC]

## App View {#current-view}

![dApps: Discover dApps application view](help/assets/en/dapps-discover.png)

The dApps catalogue shows bounded application interfaces. A catalogue entry is
not proof that executable third-party code is installed or that its capability
is live.

## Overview

A Z00Z dApp is a plugin that prepares a **typed proposal** for Wallet review. It
is not part of the wallet, does not receive a wallet session, and cannot sign or
submit a transaction.

Z00Z Labs may bundle first-party plugins. Community developers may publish
third-party plugins under the same capability restrictions. The publisher label
explains provenance; it never grants broader Wallet authority.

The security boundary is:

```text
isolated plugin
  → typed intent proposal
  → native Wallet validation and trusted review
  → user confirmation
  → Wallet-selected inputs and canonical package construction
  → Wallet signature and submission
  → protocol validation and settlement
```

A plugin may prepare proposal fields or portable unsigned material. It must not
hand Wallet opaque bytes and ask for `sign(bytes)`. Wallet reconstructs the
meaning and the canonical transaction package from an allowlisted intent.

**Current status:** the Demo contains bundled local descriptors with remote code
and wallet-bridge access disabled. An installable community-plugin host,
publisher registry, signed update and revocation path, and hardened production
sandbox remain target architecture until their native implementation and
security gates are verified.

## How to use this view

When community plugins become available, use this review sequence:

1. Check whether the publisher is **Z00Z Labs**, a verified community publisher,
   or unverified. A valid publisher signature proves origin, not safety.
2. Review the plugin version, content digest, requested intent types, external
   services, disclosed fields, resource limits, and update policy.
3. Enter only information required for the proposal. A plugin can learn data
   that you type into its own interface.
4. Open the proposal in Wallet's trusted review screen. Confirm the exact
   action, objects created or consumed, recipient, value, fee, expiry, rights,
   verifier, external dependencies, and disclosures.
5. Reject the proposal if Wallet cannot express every consequence in a known,
   typed review. Never approve an unexplained or open-ended permission.
6. Treat **prepared**, **submitted**, **accepted**, and **settled** as different
   states. A local proposal is not a completed transaction.

Z00Z Labs plugins and community plugins follow the same Wallet validation path.
First-party provenance must not bypass isolation, review, or protocol checks.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Plugin | Isolated application logic that can prepare typed proposal data but cannot control Wallet. |
| Signed manifest | Content-addressed metadata declaring publisher, version, compatible protocol versions, intent types, capabilities, disclosures, resource limits, and update policy. |
| Publisher status | Provenance information such as Z00Z Labs, verified community, or unverified community. It is not spending authority or a security guarantee. |
| Typed intent | Versioned, size-bounded proposal whose fields and effects are known to Wallet. Unknown fields, actions, or versions fail closed. |
| Action Pool | Registered set of policy-bound actions Wallet and the protocol know how to validate. It is not an arbitrary program or general-purpose VM. |
| Wallet review | Trusted native projection of the canonical action, inputs, outputs, recipients, value, fee, expiry, permissions, verifier assumptions, and disclosures. |
| Transaction package | Canonical package built by Wallet after validation and confirmation. The plugin does not choose private Wallet inputs or provide arbitrary bytes for signing. |
| Brokered service | Allowlisted native adapter that may obtain a quote, resolver result, or external attestation and returns typed, redacted data. The plugin does not receive unrestricted Wallet or renderer networking. |
| Settlement | Protocol acceptance after action, policy, witness, replay, state-root, and delta checks. It is distinct from local preparation and submission. |

## Safety and limits

### Closed actions instead of one VM per application

Z00Z does not need to create an EVM or unrestricted smart-contract runtime for
each plugin. A policy selects registered `ActionDescriptorV1` entries from an
Action Pool. The current closed lifecycle effects include `NoStateChange`,
`Offer`, `Accept`, `Transfer`, `Redeem`, `PartialRedeem`, `Refund`, `Expire`,
`Grant`, `Delegate`, `Use`, `Revoke`, and `Challenge`.

An action descriptor also binds allowed input and output object families,
receiver acceptance, beneficiary and refund preservation, and required
witnesses. Witnesses can include participant signatures, a bounded right,
verifier attestation, acceptance proof, replay nonce, prior state root, and
disclosure commitment.

The protocol rejects an unknown policy, Action Pool, or action; missing
signatures, rights, attestations, acceptance, or replay protection; stale roots;
wrong object families; invalid fees; and invalid state deltas. This protects the
settlement layer even when a proposal originated from an untrusted plugin.

### Required production controls

- Run community code out of the Wallet security process or in a capability
  sandbox with no Tauri/native bridge, key store, wallet session, filesystem,
  clipboard, browser storage, or direct signing access.
- Give plugins no inventory enumeration. Wallet chooses spendable objects after
  local policy checks.
- Route external quotes, solvers, and attestations through typed native brokers
  with explicit endpoints, deadlines, response-size limits, redaction, and
  visible trust assumptions.
- Verify a signed, content-addressed manifest before execution. Pin installed
  versions and support quarantine, revocation, rollback, and security history.
- Apply CPU, memory, wall-clock, output-size, request-count, and storage quotas.
  A timeout or malformed result terminates the plugin without changing Wallet.
- Render the final review in trusted Wallet UI from canonical typed data. Plugin
  branding or HTML must never cover or imitate the confirmation surface.
- Require fresh local confirmation for every value-moving or authority-changing
  action. Do not expose unlimited approvals or reusable `sign(bytes)`.
- Bind accepted proposals to intent version, manifest digest, expiry, replay
  nonce, policy, Action Pool, and reviewed values so they cannot be substituted
  after confirmation.

### What isolation does not guarantee

A malicious plugin can still lie in its own text, waste resources, return poor
quotes, or disclose information the user voluntarily enters. A compromised
publisher key, vulnerable sandbox, malicious external solver, or compromised
operating system also remains a risk. Publisher verification and code review
reduce risk but do not replace Wallet reconstruction, trusted review, protocol
validation, revocation, and resource isolation.

For the protocol model, see
[Why Z00Z is different](https://www.z00z.io/whitepapers/uniqueness),
[Use Cases](https://www.z00z.io/whitepapers/usecases), and
[Selective Disclosure](https://www.z00z.io/docs/protocol/selective-disclosure).
The closed lifecycle and witness vocabulary is defined in the
[Action descriptor source](https://github.com/z00z-labs/z00z/blob/main/crates/z00z_core/src/actions/action_descriptor.rs),
while Action Pool validation is defined in the
[Action Pool source](https://github.com/z00z-labs/z00z/blob/main/crates/z00z_core/src/actions/action_pool.rs).

<!-- help-sync:source {"page_path":"dapps/security-model.md","route_id":"none","screenshot":"help/assets/en/dapps-discover.png","topic_id":"dapps.security-model"} -->
