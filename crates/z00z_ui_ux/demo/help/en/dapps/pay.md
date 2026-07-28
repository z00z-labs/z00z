---
id: dapps.pay
title: dApps: Pay
route: dapps.pay
scope: context
---

# dApps: Pay

[TOC]

## App View {#current-view}

![dApps: Pay application view](help/assets/en/dapps-pay.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Pay prepares a typed private-payment proposal. It names the recipient, asset,
amount, and connectivity mode; it does not choose inputs, sign, publish, or
claim settlement.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Enter a receiver card or request ID, select the asset and positive amount, set
the proposal expiry, then choose the connectivity mode. Select **Review payment
in Wallet** to create a local proposal. Wallet review is a separate step and no
value moves in this view.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Recipient or payment request | Exact receiver commitment or typed payment request. |
| Asset and amount | Proposed value; Wallet independently checks spendable inputs and fees. |
| Proposal expiry | Hard lifetime after which the prepared proposal must be rebuilt. |
| Connectivity | Online, delayed-connectivity, or prepared offline handoff. |
| Wallet checks | Recipient scope, spendable inputs, value, fee, and connectivity risk. |
| Evidence | Payment receipt plus a checkpoint reference after settlement. |

## Safety and limits

Local acceptance is not settlement. An exported package remains inspectable,
but only Wallet may confirm value, fees, signatures, publication, and the final
checkpoint state.

<!-- help-sync:source {"page_path":"dapps/pay.md","route_id":"dapps.pay","screenshot":"help/assets/en/dapps-pay.png","topic_id":"dapps.pay"} -->
