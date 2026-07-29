---
id: telemetry.quic.paths
title: "Telemetry QUIC: Paths"
route: telemetry.quic.paths
scope: context
---

# Telemetry QUIC: Paths

[TOC]

## App View {#current-view}

![Telemetry QUIC: Paths application view](help/assets/en/telemetry-quic-paths.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

This read-only view reports aggregate path validation and migration evidence. It distinguishes a validated network path from a migration event and does not display either endpoint address.

## How to use this view

1. Check whether an authoritative runtime can report validated paths.
2. Review migration and NAT-rebinding counts as separate causes of address change.
3. Use the validated maximum UDP payload size to diagnose path-size limitations.
4. Return to **Connections** for lifecycle state or **Recovery** for the post-migration RTT and congestion state.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Validated paths | Paths for which the peer completed QUIC reachability validation. |
| Migration events | Verified changes from one usable network path to another. |
| NAT rebinding | Peer-address changes attributed to a middlebox rather than an intentional migration. |
| Path MTU | Validated maximum UDP payload size supported by the active path. |

## Safety and limits

- Path validation establishes reachability for the direction tested by an endpoint; the peer validates the return direction independently. It does not prove identity, privacy, or OnionNet route quality.
- The view exposes counts and states only; local and remote addresses remain hidden.
- **Unavailable** is not a failed validation. It means no authoritative path snapshot reached the UI.

<!-- help-sync:source {"page_path":"telemetry/quic/paths.md","route_id":"telemetry.quic.paths","screenshot":"help/assets/en/telemetry-quic-paths.png","topic_id":"telemetry.quic.paths"} -->
