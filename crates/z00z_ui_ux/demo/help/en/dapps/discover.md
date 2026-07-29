---
id: dapps.discover
title: dApps: Discover dApps
route: dapps.discover
scope: context
---

# dApps: Discover dApps

[TOC]

## App View {#current-view}

![dApps: Discover dApps application view](help/assets/en/dapps-discover.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Discover dApps presents 18 bundled Z00Z typed-action descriptors. Each card explains
the proposal it can prepare, the objects it requests, its disclosure boundary,
offline behavior, and current maturity. These are local interface descriptors,
not remotely downloaded or executed applications.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

[Read how dApps work and why the plugin model is safe](?topic=dapps.security-model).

## How to use this view

Review a card's purpose, requested objects, disclosed data, offline behavior,
and maturity before opening it. **Open interface** opens the matching typed
proposal form; **Help** opens its matching explanation. Preparing a local draft
does not select Wallet objects, sign a package, or prove settlement.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| 18 descriptors | Number of bundled typed-action interfaces currently available in this Demo catalogue. |
| Maturity | Indicates whether the interface is a local concept or has stronger implementation evidence. |
| Requested objects | Object families the proposal may ask Wallet to review; this is not pre-authorized access. |
| Data disclosed | Information the proposed action may reveal if the user later confirms it. |
| Offline behavior | What can be prepared or inspected locally and which later stages still require current evidence. |
| Open interface | Opens the descriptor's typed local proposal form. |
| Help | Opens the Help topic for the same descriptor and route. |

## Safety and limits

Discover dApps never grants a dApp wallet credentials, generic signing access, or
settlement authority. Catalogue text and drafts can be inspected offline, but
Wallet scope checks, user confirmation, authoritative external facts, and
settlement evidence remain separate stages. A descriptor marked as a concept
must not be treated as an available production capability.

<!-- help-sync:source {"page_path":"dapps/discover.md","route_id":"dapps.discover","screenshot":"help/assets/en/dapps-discover.png","topic_id":"dapps.discover"} -->
