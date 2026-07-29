---
id: telemetry.quic.connections
title: Telemetry QUIC: Connections
route: telemetry.quic.connections
scope: context
---

# Telemetry QUIC: Connections

[TOC]

## App View {#current-view}

![Telemetry QUIC: Connections application view](help/assets/en/telemetry-quic-connections.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

This read-only view groups sanitized connection-lifecycle and negotiation evidence. It intentionally avoids endpoint addresses, raw connection identifiers, tokens, and per-contact traffic.

## How to use this view

1. Read the capability banner before interpreting any value.
2. Check whether established, closing, or draining connections can be counted authoritatively.
3. Review the negotiated QUIC version and application protocol only as sanitized metadata.
4. Use **Paths** for migration evidence and **Recovery** for RTT, loss, and congestion evidence.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Connection lifecycle | Aggregate counts for active, closing, and draining QUIC connections. |
| Negotiated version / ALPN | Sanitized protocol-version and application-protocol negotiation result. |
| Connection ID rotation | Aggregate evidence that connection identifiers were issued, retired, or rotated without displaying their values. |
| Traffic totals | Aggregate bytes sent and received across observed QUIC connections. |

## Safety and limits

- The view must not expose IP addresses, UDP ports, raw Connection IDs, reset tokens, session tickets, contacts, or payload contents.
- **Unavailable** means no authoritative QUIC bridge supplied the value; it is not proof that no connection exists.
- Aggregate transport evidence does not identify the wallet action carried by a stream.

<!-- help-sync:source {"page_path":"telemetry/quic/connections.md","route_id":"telemetry.quic.connections","screenshot":"help/assets/en/telemetry-quic-connections.png","topic_id":"telemetry.quic.connections"} -->
