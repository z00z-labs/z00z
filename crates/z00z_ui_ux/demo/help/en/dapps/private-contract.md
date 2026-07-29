---
id: dapps.private-contract
title: dApps: Private Agreement
route: dapps.private-contract
scope: context
---

# dApps: Private Agreement

[TOC]

## App View {#current-view}

![dApps: Private Agreement application view](help/assets/en/dapps-private-contract.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Private Agreement proposes a typed private agreement with explicit parties,
obligations, validity, disclosure, and decision boundaries. It is an
inspectable agreement object, not an arbitrary smart-contract runtime.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the agreement template and identify the counterparty commitment. State
the subject and bounded obligations, then commit the canonical terms digest.
Declare when the agreement becomes effective, when it expires, who may receive
selective evidence, and which named decision path applies.

Wallet checks both-party scope, digest consistency, validity, disclosure, and
decision authority before it can prepare an acceptance package.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Agreement template | Declares the typed agreement family; it does not install executable code. |
| Counterparty | Commitment identifying the other proposed party. |
| Subject | Exact private relationship or deliverable covered by the agreement. |
| Obligations | Bounded duties or deliverables whose wording is committed by the terms digest. |
| Terms digest | Canonical commitment to the reviewed terms; it is not a local file or executable payload. |
| Effective rule | Event after which both accepted copies treat the agreement as active. |
| Expiry | Declared validity boundary for future actions. |
| Disclosure | Parties or named reviewers permitted to receive selective evidence. |
| Decision path | Explicit mutual, mediator, or external-authority scope for later evidence. |

## Safety and limits

The dApp cannot bind either party, determine legal enforceability, interpret
terms, adjudicate a dispute, or transfer value. Acceptance receipts prove only
the reviewed typed proposal and declared commitments. Any later payment,
escrow, voucher, permission, or settlement action requires its own Wallet
review and authoritative evidence.

<!-- help-sync:source {"page_path":"dapps/private-contract.md","route_id":"dapps.private-contract","screenshot":"help/assets/en/dapps-private-contract.png","topic_id":"dapps.private-contract"} -->
