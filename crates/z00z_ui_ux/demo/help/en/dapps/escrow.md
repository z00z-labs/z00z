---
id: dapps.escrow
title: dApps: Escrow
route: dapps.escrow
scope: context
---

# dApps: Escrow

[TOC]

## App View {#current-view}

![dApps: Escrow application view](help/assets/en/dapps-escrow.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Escrow proposes bounded conditional release terms. Z00Z supplies an
inspectable conditional object; it does not become the arbitrator.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Enter the counterparty, amount, release condition, timeout, fallback recipient,
and optional independent arbitration authority. Wallet checks every authority
and failure path before locking value.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Release condition | Exact milestone or delivery evidence required. |
| Timeout | Deadline that activates the declared fallback path. |
| Fallback recipient | Return or alternate recipient after timeout. |
| Arbitration service | Optional independent authority with narrow decision scope. |
| Evidence | Escrow terms, evidence digest, and release receipt. |

## Safety and limits

Terms may be inspected offline, but release, dispute, timeout, and arbitrator
authority require current evidence. Publishing escrow terms does not prove
delivery, resolve a dispute, or settle value.

<!-- help-sync:source {"page_path":"dapps/escrow.md","route_id":"dapps.escrow","screenshot":"help/assets/en/dapps-escrow.png","topic_id":"dapps.escrow"} -->
