---
id: dapps.create-permission
title: dApps: Create Permission
route: dapps.create-permission
scope: context
---

# dApps: Create Permission

[TOC]

## App View {#current-view}

![dApps: Create Permission application view](help/assets/en/dapps-create-permission.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Create Permission proposes bounded authority without sharing wallet keys or
granting generic signing access.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Enter the recipient, exact action and object scope, allowed uses, expiry, and
delegation rule. Wallet reviews issuer authority and every ceiling before
creating the permission.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Recipient | Person, service, or agent commitment receiving the permission. |
| Exact authority scope | Allowed action, object family, service, and value boundary. |
| Uses and expiry | Maximum invocations and hard end time. |
| Delegation | Forbidden or attenuation-only; scope can never expand. |
| Evidence | Permission digest and issuance receipt. |

## Safety and limits

The permission cannot increase its value, uses, expiry, recipients, or object
families. Wallet re-checks current authority before issuance and before any
later value-bearing action.

<!-- help-sync:source {"page_path":"dapps/create-permission.md","route_id":"dapps.create-permission","screenshot":"help/assets/en/dapps-create-permission.png","topic_id":"dapps.create-permission"} -->
