---
id: dapps.request
title: "dApps: Request"
route: dapps.request
scope: context
---

# dApps: Request

[TOC]

## App View {#current-view}

![dApps: Request application view](help/assets/en/dapps-request.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Request creates a portable private invoice without exposing a reusable public
account graph. A request is information for a payer; it is never authority to
pull funds.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose the requested asset and positive amount, whether it is an exact or
minimum total, and whether one payment, partial payments, or multiple payers
are allowed. Set expiry and optional private business reference, attachment
digest, and memo. Wallet can then review and sign the request for QR or file
handoff.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Requested asset and amount | Exact or minimum total requested from a future payer. |
| Amount rule | Distinguishes a fixed total from a minimum acceptable total. |
| Payment mode | Controls single payment, partial payment, or multiple-payer collection. |
| Expiry | Time after which Wallet must reject the request. |
| Business reference and attachment digest | Optional private references; the attachment is represented by a digest, never a local path. |
| Private memo | Optional text disclosed only to the intended payer. |
| Evidence | Request digest and proof-of-payment return reference. |

## Safety and limits

Creating a request does not reserve or receive value. The payer reviews a
separate payment package, and both parties must distinguish request delivery,
local acceptance, publication, and settlement.

<!-- help-sync:source {"page_path":"dapps/request.md","route_id":"dapps.request","screenshot":"help/assets/en/dapps-request.png","topic_id":"dapps.request"} -->
