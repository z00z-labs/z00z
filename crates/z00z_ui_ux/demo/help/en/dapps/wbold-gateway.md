---
id: dapps.wbold-gateway
title: dApps: wBOLD Gateway
route: dapps.wbold-gateway
scope: context
---

# dApps: wBOLD Gateway

[TOC]

## App View {#current-view}

![dApps: wBOLD Gateway application view](help/assets/en/dapps-wbold-gateway.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

wBOLD Gateway proposes one explicit deposit or redemption route between an
external BOLD event and a private wBOLD right.

**Boundary:** a Z00Z dApp does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Select deposit or redemption, enter the amount and any external recipient,
choose the named locker route, and cap the route fee. Wallet reviews both
external and internal evidence before building a package.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Route | Deposit BOLD for wBOLD or redeem wBOLD for BOLD. |
| External address | Required destination on the external network when redeeming. |
| Locker route | One explicit operator and network, never a hidden route. |
| Maximum route fee | Hard ceiling checked by Wallet before confirmation. |
| Evidence | External event reference, internal receipt, and route status. |

## Safety and limits

Z00Z can verify the internal package boundary but cannot guarantee external
custody, solvency, finality, pause state, or redemption. Offline inspection is
possible; current external status requires reconnection.

<!-- help-sync:source {"page_path":"dapps/wbold-gateway.md","route_id":"dapps.wbold-gateway","screenshot":"help/assets/en/dapps-wbold-gateway.png","topic_id":"dapps.wbold-gateway"} -->
