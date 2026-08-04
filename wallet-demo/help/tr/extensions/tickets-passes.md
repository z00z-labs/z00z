---
id: extensions.tickets-passes
title: "Extensions: Ticket & Pass"
route: extensions.tickets-passes
scope: context
---

# Extensions: Ticket & Pass

[TOC]

## App View {#current-view}

![Extensions: Ticket & Pass application view](help/assets/en/extensions-tickets-passes.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Ticket & Pass issues a private event, transport, membership, or access
right with bounded use and an explicit verification policy.

**Boundary:** a Z00Z Extension does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the event or service, use count, validity window, transferability, and
offline verification policy. Wallet checks issuer identity and every bound
before it builds the pass.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Event or service | Exact admission, transport, membership, or venue scope. |
| Number of uses | Maximum successful presentations or redemptions. |
| Validity | Declared time window for the pass. |
| Transferability | None, one transfer, or transfer until first use. |
| Evidence | Pass digest and bounded redemption receipt. |

## Safety and limits

Offline presentation is valid only inside the declared freshness and verifier
policy. The pass proves the named access right, not identity, universal
admission, service delivery, or settlement.

<!-- help-sync:source {"page_path":"extensions/tickets-passes.md","route_id":"extensions.tickets-passes","screenshot":"help/assets/en/extensions-tickets-passes.png","topic_id":"extensions.tickets-passes"} -->
