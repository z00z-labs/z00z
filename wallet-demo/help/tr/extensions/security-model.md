---
id: extensions.security-model
title: "Extensions: How Extensions Work Safely"
route: none
scope: guide
---

# Extensions: How Extensions Work Safely

[TOC]

## App View {#current-view}

![Extensions: Discover Extensions application view](help/assets/en/extensions-discover.png)

The Extensions catalogue shows bounded application interfaces. A catalogue entry is
not proof that executable third-party code is installed or that its capability
is live.

## Overview

A **Z00Z Extension** is a reviewed **Declarative Extension Package**. It contains
data and bounded presentation/proposal definitions, not executable code. It is
not part of Wallet, receives no App API client or wallet session, and cannot
select inputs, sign, submit, or settle a transaction.

Z00Z Labs may bundle first-party Extensions. Community developers may publish
third-party Extensions under the same capability restrictions. An
`ExtensionPublisher` label and signature establish package provenance; they
never grant Wallet authority or prove that an Extension is safe.

The security boundary is:

```text
untrusted Declarative Extension Package
  → canonical parser and ExtensionValidator
  → immutable ExtensionRegistry entry
  → ExtensionHost emits a bounded ExtensionArtifactEnvelope
  → trusted host copies allowlisted fields into a fresh typed proposal
  → native Wallet validation and trusted review
  → user confirmation
  → Wallet-selected inputs and canonical package construction
  → Wallet signature and submission
  → protocol validation and settlement
```

An Extension may describe proposal fields and produce only a bounded, inert
`ExtensionArtifactEnvelope`. The trusted host validates that envelope and copies
allowlisted fields into a new source-neutral proposal. The original envelope
never enters Wallet services. No Extension can hand Wallet opaque bytes and ask
for `sign(bytes)`.

**Current status:** the Demo contains bundled local descriptors with remote code
and wallet-bridge access disabled. It does not install, update, remove, or
execute packages. The production `ExtensionRegistry`, `ExtensionValidator`,
`ExtensionHost`, signed update/revocation path, quarantine, and immutable
content-addressed storage remain target architecture until their native
implementation and security gates are verified.

## How to use this view

When community Extensions become available, use this review sequence:

1. Check whether the publisher is **Z00Z Labs**, a verified community publisher,
   or unverified. A valid publisher signature proves origin, not safety.
2. Review the `ExtensionManifest`, version, content digest, requested artifact
   types, disclosed fields, resource limits, and update policy.
3. Enter only information required for the proposal. Extension-defined
   presentation can learn data entered into its own fields.
4. Open the proposal in Wallet's trusted review screen. Confirm the exact
   action, objects created or consumed, recipient, value, fee, expiry, rights,
   verifier, external dependencies, and disclosures.
5. Reject the proposal if Wallet cannot express every consequence in a known,
   typed review. Never approve an unexplained or open-ended permission.
6. Treat **prepared**, **submitted**, **accepted**, and **settled** as different
   states. A local proposal is not a completed transaction.

Z00Z Labs Extensions and community Extensions follow the same validation path.
First-party provenance must not bypass isolation, review, or protocol checks.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Z00Z Extension | A reviewed local package identity and its safe projection. It has no executable code or Wallet authority. |
| Declarative Extension Package | Canonical, signed, data-only package containing an `ExtensionManifest` and bounded presentation/artifact definitions. |
| ExtensionManifest | Content-addressed metadata declaring `ExtensionId`, publisher, version, compatible protocol versions, artifact types, capabilities, disclosures, resource limits, and update policy. |
| ExtensionPublisher | Public provenance identity and trust metadata. Publisher status is not spending authority or a safety guarantee. |
| ExtensionRegistry | Trusted index of validated, immutable package versions and their enabled, blocked, quarantined, or revoked state. |
| ExtensionValidator | Canonical parser and policy validator that rejects executable material, unknown fields, invalid signatures, rollback, traversal, and oversized input. |
| ExtensionHost | Trusted service that reads a validated package and may emit only a bounded `ExtensionArtifactEnvelope`. |
| ExtensionArtifactEnvelope | Inert, size-bounded, versioned data copied by the trusted host into a fresh source-neutral proposal after revalidation. |
| Publisher status | Provenance information such as Z00Z Labs, verified community, or unverified community. It is not spending authority or a security guarantee. |
| Typed intent | Versioned, size-bounded proposal whose fields and effects are known to Wallet. Unknown fields, actions, or versions fail closed. |
| Action Pool | Registered set of policy-bound actions Wallet and the protocol know how to validate. It is not an arbitrary program or general-purpose VM. |
| Wallet review | Trusted native projection of the canonical action, inputs, outputs, recipients, value, fee, expiry, permissions, verifier assumptions, and disclosures. |
| Transaction package | Canonical package built by Wallet after validation and confirmation. The Extension does not choose private Wallet inputs or provide arbitrary bytes for signing. |
| Brokered service | Allowlisted native adapter that may obtain a quote, resolver result, or external attestation and returns typed, redacted data. The Extension receives no unrestricted Wallet, renderer, or network access. |
| Settlement | Protocol acceptance after action, policy, witness, replay, state-root, and delta checks. It is distinct from local preparation and submission. |

## Safety and limits

### Closed actions instead of one VM per application

Z00Z does not create an EVM or unrestricted smart-contract runtime for each
Extension. A policy selects registered `ActionDescriptorV1` entries from an
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
settlement layer even when a proposal originated from an untrusted Extension.

### Required production controls

- Reject executable code, scripts, native libraries, remote resources, arbitrary
  URLs, callbacks, and unknown package fields. A v1 package is data-only.
- Give Extensions no App API client, Tauri/native bridge, key store, wallet
  session, inventory enumeration, filesystem, clipboard, browser storage,
  unrestricted networking, or signing access.
- Let Wallet choose spendable objects only after local policy checks.
- Route external quotes, solvers, and attestations through typed native brokers
  with explicit endpoints, deadlines, response-size limits, redaction, and
  visible trust assumptions.
- Verify a signed, content-addressed manifest before use. Pin installed
  versions and support quarantine, revocation, rollback, and security history.
- Apply CPU, memory, wall-clock, output-size, request-count, and storage quotas.
  A timeout or malformed artifact terminates processing without changing Wallet.
- Render the final review in trusted Wallet UI from canonical typed data.
  Extension presentation must never cover or imitate the confirmation surface.
- Require fresh local confirmation for every value-moving or authority-changing
  action. Do not expose unlimited approvals or reusable `sign(bytes)`.
- Bind accepted proposals to intent version, manifest digest, expiry, replay
  nonce, policy, Action Pool, and reviewed values so they cannot be substituted
  after confirmation.

### What isolation does not guarantee

A malicious Extension can still lie in its own text, waste parser/host
resources, request misleading fields, return poor quotes through a broker, or
disclose information the user voluntarily enters. A compromised publisher key,
parser or host defect, malicious external solver, or compromised operating
system also remains a risk. Publisher verification and package review reduce
risk but do not replace canonical parsing, artifact revalidation, trusted Wallet
review, protocol validation, revocation, and resource isolation.

For the protocol model, see
[Why Z00Z is different](https://www.z00z.io/whitepapers/uniqueness),
[Use Cases](https://www.z00z.io/whitepapers/usecases), and
[Selective Disclosure](https://www.z00z.io/docs/protocol/selective-disclosure).
The closed lifecycle and witness vocabulary is defined in the
[Action descriptor source](https://github.com/z00z-labs/z00z/blob/main/crates/z00z_core/src/actions/action_descriptor.rs),
while Action Pool validation is defined in the
[Action Pool source](https://github.com/z00z-labs/z00z/blob/main/crates/z00z_core/src/actions/action_pool.rs).

<!-- help-sync:source {"page_path":"extensions/security-model.md","route_id":"none","screenshot":"help/assets/en/extensions-discover.png","topic_id":"extensions.security-model"} -->
