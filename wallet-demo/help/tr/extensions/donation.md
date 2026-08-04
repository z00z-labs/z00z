---
id: extensions.donation
title: "Extensions: Donation"
route: extensions.donation
scope: context
---

# Extensions: Donation

[TOC]

## App View {#current-view}

![Extensions: Donation application view](help/assets/en/extensions-donation.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Donation proposes a private one-time or bounded recurring contribution with an
explicit beneficiary and selective receipt disclosure.

**Boundary:** a Z00Z Extension does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Select the beneficiary, amount, bounded schedule, and donor-receipt policy.
Wallet confirms beneficiary identity, value, recurrence, disclosure purpose,
fees, and cancellation boundary.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Beneficiary | Verified receiver or project commitment. |
| Schedule | One-time or a fixed number of monthly periods. |
| Donor receipt | Private, amount disclosed to beneficiary, or aggregate-only. |
| Wallet checks | Identity, amount, recurrence, disclosure, and cancellation. |
| Evidence | Private donor receipt and optional aggregate reference. |

## Safety and limits

The Extension cannot debit the wallet and cannot attest how a beneficiary uses
funds. A prepared package is not proof of beneficiary receipt or settlement.

<!-- help-sync:source {"page_path":"extensions/donation.md","route_id":"extensions.donation","screenshot":"help/assets/en/extensions-donation.png","topic_id":"extensions.donation"} -->
