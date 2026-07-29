---
id: dapps.create-voucher
title: "dApps: Create Voucher"
route: dapps.create-voucher
scope: context
---

# dApps: Create Voucher

[TOC]

## App View {#current-view}

![dApps: Create Voucher application view](help/assets/en/dapps-create-voucher.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Create Voucher proposes conditional value using fixed, inspectable policy
primitives instead of arbitrary executable rules.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Define the voucher class, backing asset, positive face value, merchant or
service scope, maximum redemptions, partial-redemption rule, expiry,
transferability, and unused-value fallback. Wallet verifies issuer authority
and backing before it can build an issuance package.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Voucher class | Human-readable purpose such as travel or event credit. |
| Backing asset and face value | Exact asset and positive value Wallet must verify before issuance. |
| Scope | Exact merchant, provider, or service class allowed at redemption. |
| Uses and expiry | Hard redemption count and validity ceiling. |
| Partial redemption | Whether a redemption must consume the voucher or may leave a remainder. |
| Transferability | No transfer, one transfer, or transfer until first use. |
| Unused-value fallback | Explicit refund or no-refund behavior after expiry. |
| Evidence | Voucher-definition digest and issuance/redemption receipts. |

## Safety and limits

The dApp cannot mint value or become issuer authority. Wallet checks backing,
refund path, policy bounds, and fees. A local voucher handoff is not evidence
of redemption or checkpoint settlement.

<!-- help-sync:source {"page_path":"dapps/create-voucher.md","route_id":"dapps.create-voucher","screenshot":"help/assets/en/dapps-create-voucher.png","topic_id":"dapps.create-voucher"} -->
