---
id: dapps.subscription
title: "dApps: Subscription"
route: dapps.subscription
scope: context
---

# dApps: Subscription

[TOC]

## App View {#current-view}

![dApps: Subscription application view](help/assets/en/dapps-subscription.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Subscription proposes bounded claims per period. It deliberately avoids an
unlimited merchant allowance or generic debit permission.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the exact provider and service plan, billing asset, positive amount per
period, period length, maximum number of periods, and policy expiry. Wallet
creates only the bounded slices allowed by the reviewed policy.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Provider and plan | Exact service authority and service class. |
| Billing asset and amount per period | Explicit asset and maximum claim value for one period. |
| Period | Weekly, monthly, or yearly slice. |
| Maximum periods | Hard count after which a fresh proposal is required. |
| Policy expiry | Independent time ceiling that cannot outlive the reviewed policy. |
| Evidence | Subscription policy and individual period receipts. |

## Safety and limits

The provider cannot pull funds, extend duration, or raise the amount. Only
already issued period slices may travel offline; cancellation, expiry, fees,
and each new period stay within Wallet policy.

<!-- help-sync:source {"page_path":"dapps/subscription.md","route_id":"dapps.subscription","screenshot":"help/assets/en/dapps-subscription.png","topic_id":"dapps.subscription"} -->
