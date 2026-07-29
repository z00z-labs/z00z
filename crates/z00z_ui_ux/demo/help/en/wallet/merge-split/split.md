---
id: wallet.split
title: "Wallet: Split"
route: wallet.merge-split
scope: context
---

# Wallet: Split

[TOC]

## App View {#current-view}

![Wallet: Split application view](help/assets/en/wallet-split.png)

This image is captured from the live Demo Split view.

## Overview

Split consumes one confidential asset fragment and prepares two or more outputs. Every output keeps the source `definition_id` and base `serial_id`, and all positive output amounts must add up exactly to the input amount. Split changes output arrangement; it does not change the asset definition or create supply.

Each resulting fragment receives its own concrete output identity while remaining part of the same issuance series.

## How to use this view

1. Confirm the active wallet and chain in the application header.
2. Select **Split**.
3. Choose one available source fragment.
4. Enter between two and eight positive output amounts.
5. Confirm that **Conservation** reads **Exact**.
6. Select **Preview split** and review the source and every proposed output.
7. Continue only in a native wallet that can re-check authorization, fees, submission, and reconciliation.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Source asset | The single available fragment consumed by the proposed Split. |
| Definition ID | Immutable identifier of the asset type and policy. Every output keeps the source definition. |
| Serial ID | Base issuance series. Every output keeps the source serial. |
| Output allocation | Between two and eight positive amounts assigned to the proposed outputs. |
| Conservation | Exact equality between the input amount and the sum of all output amounts. |
| Add output | Adds another positive allocation field, up to the interface limit. |
| Preview split | Review-only intent showing the source and proposed outputs; it does not sign or submit. |

## Safety and limits

- Split never changes the source definition or base serial.
- Zero, negative, excessive, or non-conserving allocations must be rejected.
- A source that became locked, spent, frozen, burned, slashed, or otherwise unavailable must be rejected by the native wallet.
- Repeated or unusually patterned allocations can make related outputs easier to correlate.
- The JavaScript Demo uses public fixtures and stops at preview. It does not hold keys, prove ownership, build signatures, charge fees, submit a package, or reconcile an uncertain outcome.
- The current `wallet.asset.split_asset` helper is a compatibility surface and does not claim canonical ledger reconciliation authority. Native integration must route confirmation through the authoritative wallet transaction path.

<!-- help-sync:source {"page_path":"wallet/merge-split/split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-split.png","topic_id":"wallet.split"} -->
