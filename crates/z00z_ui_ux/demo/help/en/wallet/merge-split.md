---
id: wallet.merge-split
title: Wallet: Merge/Split
route: wallet.merge-split
scope: context
---

# Wallet: Merge/Split

[TOC]

## App View {#current-view}

![Wallet: Merge/Split application view](help/assets/en/wallet-merge-split.png)

This image is captured from the live Demo view.

## Overview

Merge/Split changes how value from one base asset series is arranged across confidential wallet outputs. It does not change the asset definition or create new supply.

- **Merge** consumes two or more compatible fragments and prepares one output containing their total amount.
- **Split** consumes one fragment and prepares two or more outputs whose positive amounts add up exactly to the input.

The screen groups Merge candidates by both `definition_id` and `serial_id`. This is intentional: the definition identifies the immutable asset type, while the serial identifies its base issuance series. Different output commitments can share that pair and still have distinct `asset_id` values.

## How to use this view

### Merge

1. Confirm the active wallet and chain in the application header.
2. Select **Merge**.
3. Choose at least two available fragments from one compatibility group.
4. Check the selected input count, total output amount, definition, and serial.
5. Select **Preview merge** and review the proposed inputs and single output.
6. Continue only in a native wallet that can re-check authorization, fees, submission, and reconciliation.

### Split

1. Select **Split**.
2. Choose one available source fragment.
3. Enter between two and eight positive output amounts.
4. Confirm that **Conservation** reads **Exact**.
5. Select **Preview split** and review every proposed output.
6. Continue only after native wallet confirmation is available.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Definition ID | Immutable identifier of the asset type and policy. Merge inputs must share it. |
| Serial ID | Base issuance series. Core assets treat it as immutable; every fragment produced by Split keeps the source serial. |
| Asset ID | Identifier of one concrete confidential output. Different fragments can have different asset IDs while sharing one definition and serial. |
| Fragment | One wallet-owned spendable output carrying part or all of a serial's value. |
| Compatible group | Two or more available fragments with the same definition ID and serial ID. |
| Locked | The fragment is shown for context but cannot be selected as an input. |
| Total output | Sum of selected Merge inputs. The preview creates one output with this amount. |
| Output allocation | Positive Split amounts that must sum exactly to the source amount. |
| Conservation | Check that input and output totals are identical before any native fee or settlement policy is applied. |
| Preview | Review-only intent. It does not sign, submit, settle, or mutate the wallet. |

## Safety and limits

- Merge never crosses definitions or base serials in this interface.
- Split never changes the source definition or serial.
- Locked, spent, frozen, burned, slashed, or otherwise unavailable inputs must be rejected by the native wallet even if a stale screen once displayed them.
- Recomposition can make related inputs and outputs easier to correlate. Review privacy impact before repeated or highly patterned Merge/Split operations.
- The JavaScript Demo uses public fixtures and stops at preview. It does not hold keys, prove ownership, build signatures, charge fees, submit a package, or reconcile an uncertain outcome.
- The current `wallet.asset.merge_assets` and `wallet.asset.split_asset` RPC helpers are compatibility surfaces and do not claim canonical ledger reconciliation authority. Native integration must route confirmation through the authoritative wallet transaction path.

<!-- help-sync:source {"page_path":"wallet/merge-split.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge-split.png","topic_id":"wallet.merge-split"} -->
