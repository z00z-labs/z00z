---
id: dapps.agents-budget
title: dApps: Agent Budget
route: dapps.agents-budget
scope: context
---

# dApps: Agent Budget

[TOC]

## App View {#current-view}

![dApps: Agent Budget application view](help/assets/en/dapps-agents-budget.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Agent Budget proposes a tightly bounded spending right for a named local
agent. The agent receives policy-limited authority, never a private key or
generic signing interface.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the agent identity, provider allowlist, and budget asset. Set the daily
ceiling, a no-larger per-action ceiling, maximum action count, a no-larger
human-approval threshold, and expiry. Each later expense still creates its own
typed proposal for Wallet review.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Agent identity | Local commitment identifying the constrained agent. |
| Service allowlist | Exact providers or service classes the agent may use. |
| Daily, action, and count limits | The per-action ceiling cannot exceed the daily budget; the action count is independently bounded. |
| Human threshold | Value above which explicit confirmation is mandatory. |
| Evidence | Budget grant plus a receipt for every accepted or rejected action. |

## Safety and limits

No hidden provider, unlimited spend, private key, or generic signature is
allowed. Wallet checks fee and value ceilings per action and must expose an
immediate revoke path.

<!-- help-sync:source {"page_path":"dapps/agents-budget.md","route_id":"dapps.agents-budget","screenshot":"help/assets/en/dapps-agents-budget.png","topic_id":"dapps.agents-budget"} -->
