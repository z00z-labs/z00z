---
id: settings.quic
title: "Settings: QUIC"
route: settings.quic
scope: context
---

# Settings: QUIC

[TOC]

## App View {#current-view}

![Settings: QUIC application view](help/assets/en/settings-quic.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

The **Network** view groups Reticulum, OnionNet, and QUIC without mixing their responsibilities. QUIC controls secure transport-session behavior and remains separate from OnionNet privacy routing and Reticulum carrier configuration.

## How to use this view

1. Choose how configured QUIC endpoints are selected.
2. Decide whether an established connection may migrate when the local network path changes.
3. Set the idle timeout and keep-alive interval together.
4. Keep wallet actions out of 0-RTT early data.
5. Open **Telemetry: QUIC** for read-only runtime evidence.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Endpoint policy | Chooses automatic selection, a preferred configured endpoint, or direct-only behavior. |
| Path migration | Allows an established connection to continue after the local network path changes. |
| Idle timeout | Closes a connection after the negotiated period without activity. |
| Keep-alive | Sends activity often enough to prevent an otherwise idle connection from closing. |
| 0-RTT wallet actions | Disabled because early data can be replayed and is unsuitable for state-changing wallet actions. |

## Safety and limits

- The current wallet Demo stores these controls as a local target draft; it has no authoritative QUIC configuration bridge.
- The Demo does not infer QUIC state from unrelated traffic counters.
- QUIC transport security is not an OnionNet privacy guarantee.
- An endpoint policy is not live endpoint or session evidence.
- No connection identifier, session token, or per-contact route is shown.

<!-- help-sync:source {"page_path":"settings/network/quic.md","route_id":"settings.quic","screenshot":"help/assets/en/settings-quic.png","topic_id":"settings.quic"} -->
