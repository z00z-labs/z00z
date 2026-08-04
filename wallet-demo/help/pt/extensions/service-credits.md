---
id: extensions.service-credits
title: "Extensions: Service Credit"
route: extensions.service-credits
scope: context
---

# Extensions: Service Credit

[TOC]

## App View {#current-view}

![Extensions: Service Credit application view](help/assets/en/extensions-service-credits.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Service Credit models bounded API, data, compute, storage, or access rights as
explicit service entitlements rather than money.

**Boundary:** a Z00Z Extension does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Select the exact provider and service class, positive quota, metering unit,
expiry, and delegation rule. The form proposes issuance of a bounded service
right. Later presentation and provider redemption are separate actions.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Provider and service | Exact authority and API, compute, storage, or access class. |
| Quota and unit | Bounded calls, minutes, GiB, or requests. |
| Expiry | Hard validity ceiling. |
| Delegation | Forbidden or attenuation-only. |
| Evidence | Issued credit definition; a later presentation can produce a separate redemption receipt. |

## Safety and limits

A credit is not currency, a universal entitlement, or proof that service was
delivered. Provider availability and delivery are external facts; Wallet only
authorizes the declared issuance proposal here.

<!-- help-sync:source {"page_path":"extensions/service-credits.md","route_id":"extensions.service-credits","screenshot":"help/assets/en/extensions-service-credits.png","topic_id":"extensions.service-credits"} -->
