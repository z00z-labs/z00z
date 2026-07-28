---
id: dapps.bounties
title: dApps: Bounties
route: dapps.bounties
scope: context
---

# dApps: Bounties

[TOC]

## App View {#current-view}

![dApps: Bounties application view](help/assets/en/dapps-bounties.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Bounties proposes a reward claim whose payout depends on a named verifier and
explicit evidence. Publishing the bounty does not validate a submission.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Define the deliverable, verifier, reward asset and positive amount, deadline,
and accepted evidence family. Wallet checks backing and verifier authority
before creating the bounty; payout needs a separate verifier decision and
Wallet confirmation.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Task or result scope | Exact deliverable eligible for the reward. |
| Verifier | Independent commitment allowed to accept evidence. |
| Reward asset, amount, and deadline | Explicit backed value and final submission time. |
| Evidence requirement | Digest, receipt, or proof family the verifier evaluates. |
| Evidence | Bounty definition, verifier decision, and payout receipt. |

## Safety and limits

A carried submission digest is not verification. The dApp cannot self-approve,
change verifier scope, or authorize payout; Wallet reviews reward, fees, and
the verifier decision.

<!-- help-sync:source {"page_path":"dapps/bounties.md","route_id":"dapps.bounties","screenshot":"help/assets/en/dapps-bounties.png","topic_id":"dapps.bounties"} -->
