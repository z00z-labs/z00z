---
id: dapps.payroll
title: "dApps: Payroll"
route: dapps.payroll
scope: context
---

# dApps: Payroll

[TOC]

## App View {#current-view}

![dApps: Payroll application view](help/assets/en/dapps-payroll.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Payroll proposes a private batch payout with aggregate totals and narrow
employee receipts. It avoids exposing the payroll graph or treasury keys.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the private batch, payout asset, typed recipient-set reference, expected
recipient count, positive aggregate ceiling, schedule, and audit output.
Wallet revalidates the actual recipient set, aggregate value, and fees before
creating per-recipient packages.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Payroll batch | Private identifier for one reviewed payout set. |
| Recipient set reference | Imported encrypted batch or typed local batch ID, never a filesystem path. |
| Expected recipient count | Draft expectation; Wallet verifies it against the imported recipient set. |
| Payout asset and aggregate ceiling | Explicit asset and maximum batch value. |
| Audit output | Aggregate-only, employee receipts, or a scoped auditor package. |
| Evidence | Narrow employee receipts and an aggregate batch proof. |

## Safety and limits

A sanitized local draft cannot access treasury keys or sign a batch. Recipient
details, totals, authority, fees, disclosure scope, publication, and settlement
are rechecked by Wallet.

<!-- help-sync:source {"page_path":"dapps/payroll.md","route_id":"dapps.payroll","screenshot":"help/assets/en/dapps-payroll.png","topic_id":"dapps.payroll"} -->
