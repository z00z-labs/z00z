---
id: telemetry.quic.recovery
title: "Telemetry QUIC: Recovery"
route: telemetry.quic.recovery
scope: context
---

# Telemetry QUIC: Recovery

[TOC]

## App View {#current-view}

![Telemetry QUIC: Recovery application view](help/assets/en/telemetry-quic-recovery.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

This read-only view groups round-trip-time, loss-recovery, congestion-control, and ECN evidence. These measurements describe transport behavior and never imply settlement success.

## How to use this view

1. Compare latest, minimum, smoothed RTT, and RTT variance only when they come from the same authoritative snapshot.
2. Review packet-loss and probe-timeout counts together.
3. Check congestion window and bytes in flight before interpreting reduced throughput.
4. Treat ECN status as a negotiated path capability, not as a wallet security signal.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| RTT estimate | Latest, minimum, smoothed round-trip time and round-trip-time variance. |
| Loss recovery | Aggregate packet-loss and Probe Timeout (PTO) evidence. |
| Congestion state | Congestion window and bytes currently considered in flight. |
| ECN validation | Whether Explicit Congestion Notification counters were validated for the path. |

## Safety and limits

- Transport RTT, loss, and congestion do not prove application delivery, wallet confirmation, or settlement.
- A path migration can reset RTT and congestion state; compare evidence timestamps before drawing conclusions.
- **Unavailable** means no authoritative recovery snapshot was registered.

<!-- help-sync:source {"page_path":"telemetry/quic/recovery.md","route_id":"telemetry.quic.recovery","screenshot":"help/assets/en/telemetry-quic-recovery.png","topic_id":"telemetry.quic.recovery"} -->
