---
id: settings.onionnet
title: Settings: OnionNet
route: settings.onionnet
scope: context
---

# Settings: OnionNet

[TOC]

## App View {#current-view}

![Settings: OnionNet application view](help/assets/en/settings-onionnet.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

The **Network** view groups Reticulum, OnionNet, and QUIC without mixing their responsibilities. OnionNet controls the intended privacy and fallback floor independently from Reticulum carriers and QUIC transport sessions.

## How to use this view

1. Choose whether direct-path fallback is forbidden, requires a prompt, or is allowed only with a warning.
2. Set when the wallet should request a fresh privacy route.
3. Choose the background cover-traffic policy.
4. Keep **Fail closed** enabled when actions must stop below the required privacy floor.
5. Open **Telemetry: OnionNet** for read-only evidence.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Privacy mode | Chooses the intended direct-path fallback posture. |
| Route rotation | Chooses when a fresh privacy path should be requested. |
| Cover traffic | Chooses whether background cover packets are adaptive, continuous, or disabled. |
| Fail closed | Blocks an action when the required privacy floor cannot be met. |
| Admission & replay checks | Runtime evidence expected from the privacy boundary; currently unavailable. |

## Safety and limits

- The current wallet Demo stores these controls as target policy; it has no authoritative OnionNet configuration or status bridge.
- Disabling cover traffic or fail-closed behavior can reduce privacy and must remain explicit.
- These choices do not prove that a user is anonymous or untraceable.
- Runtime route health, topology, admission, and replay evidence must remain unavailable until supplied authoritatively.

<!-- help-sync:source {"page_path":"settings/onionnet.md","route_id":"settings.onionnet","screenshot":"help/assets/en/settings-onionnet.png","topic_id":"settings.onionnet"} -->
