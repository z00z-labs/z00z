---
id: extensions.assets-locker
title: "Extensions: Assets Locker"
route: extensions.assets-locker
scope: context
---

# Extensions: Assets Locker

[TOC]

## App View {#current-view}

![Extensions: Assets Locker application view](help/assets/en/extensions-assets-locker.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Assets Locker represents a bounded private right over one explicit external
custody route. Z00Z does not become the custodian.

**Boundary:** a Z00Z Extension does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the exact external asset and network, lock or redeem action, positive
amount, named locker operator and route, trust tier, and maximum fee. A redeem
action also requires an external recipient on the selected network. Wallet
checks route evidence before building the internal or external package.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| External asset | Exact asset identifier and network. |
| Action | Lock and import a right, or consume the right and redeem. |
| Operator and route | One explicit custody authority and adapter path. |
| External recipient | Mandatory external-network destination when consuming a right to redeem. |
| Trust tier | External-asset boundary shown without implying native security. |
| Evidence | Custody event reference, internal receipt, and redemption status. |

## Safety and limits

Offline inspection cannot establish current custody, reserves, pause state, or
redemption. Z00Z cannot guarantee the external operator; Wallet must show that
trust boundary and reject stale or replayed evidence.

<!-- help-sync:source {"page_path":"extensions/assets-locker.md","route_id":"extensions.assets-locker","screenshot":"help/assets/en/extensions-assets-locker.png","topic_id":"extensions.assets-locker"} -->
