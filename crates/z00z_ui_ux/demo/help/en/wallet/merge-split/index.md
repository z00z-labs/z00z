---
id: wallet.merge
title: "Wallet: Merge"
route: wallet.merge-split
scope: context
---

# Wallet: Merge

[TOC]

## App View {#current-view}

![Wallet: Merge application view](help/assets/en/wallet-merge.png)

This image is captured from the live Demo Merge view.

## Overview

Merge combines two or more compatible confidential asset fragments into one output. The output keeps the same `definition_id` and base `serial_id`, and its amount equals the sum of the selected inputs. Merge changes output arrangement; it does not change the asset definition or create supply.

Candidates are grouped by both definition and serial. Fragments from different groups cannot be combined, even when they use the same display symbol.

## How to use this view

1. Confirm the active wallet and chain in the application header.
2. Select **Merge**.
3. Choose at least two available fragments from one compatibility group.
4. Check the selected input count, total output amount, definition, and serial.
5. Select **Preview merge** and review every input and the proposed single output.
6. Continue only in a native wallet that can re-check authorization, fees, submission, and reconciliation.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Definition ID | Immutable identifier of the asset type and policy. Every selected input must share it. |
| Serial ID | Base issuance series. Every selected input and the merged output keep the same serial. |
| Asset ID | Identifier of one concrete confidential output. Compatible fragments can have different asset IDs. |
| Compatible group | Available fragments with the same definition ID and serial ID. |
| Locked | The fragment is visible for context but cannot be selected. |
| Total output | Exact sum of the selected inputs before any separate native fee policy is applied. |
| Preview merge | Review-only intent showing the inputs and proposed output; it does not sign or submit. |

## Safety and limits

- Merge never crosses definitions or base serials in this interface.
- Locked, spent, frozen, burned, slashed, or otherwise unavailable inputs must be rejected by the native wallet even if a stale screen once displayed them.
- Combining fragments can make related inputs easier to correlate. Review the privacy impact before repeated or highly patterned operations.
- The JavaScript Demo uses public fixtures and stops at preview. It does not hold keys, prove ownership, build signatures, charge fees, submit a package, or reconcile an uncertain outcome.
- The current `wallet.asset.merge_assets` helper is a compatibility surface and does not claim canonical ledger reconciliation authority. Native integration must route confirmation through the authoritative wallet transaction path.

<!-- help-sync:source {"page_path":"wallet/merge-split/index.md","route_id":"wallet.merge-split","screenshot":"help/assets/en/wallet-merge.png","topic_id":"wallet.merge"} -->
