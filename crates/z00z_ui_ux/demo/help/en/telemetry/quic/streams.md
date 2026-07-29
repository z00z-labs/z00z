---
id: telemetry.quic.streams
title: Telemetry QUIC: Streams
route: telemetry.quic.streams
scope: context
---

# Telemetry QUIC: Streams

[TOC]

## App View {#current-view}

![Telemetry QUIC: Streams application view](help/assets/en/telemetry-quic-streams.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

This read-only view summarizes multiplexed QUIC stream activity and flow-control pressure without exposing stream payloads or mapping a stream to a wallet action.

## How to use this view

1. Compare aggregate bidirectional and unidirectional stream counts.
2. Check for connection-level or stream-level flow-control blocking.
3. Review reset and stop events as aggregate termination evidence.
4. Use **Recovery** when delays are caused by packet loss or congestion rather than flow control.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Bidirectional streams | Streams on which both endpoints can send application data. |
| Unidirectional streams | Streams on which only the initiating endpoint sends application data. |
| Flow-control pressure | Aggregate evidence that connection or stream limits blocked new data. |
| Reset / stop events | Counts of abrupt send or receive termination events, grouped without stream contents. |

## Safety and limits

- No stream ID, contact, asset, message, request, or payload content is shown.
- A blocked stream is not necessarily a congested network path; inspect **Recovery** separately.
- **Unavailable** means the runtime did not supply an authoritative stream snapshot.

<!-- help-sync:source {"page_path":"telemetry/quic/streams.md","route_id":"telemetry.quic.streams","screenshot":"help/assets/en/telemetry-quic-streams.png","topic_id":"telemetry.quic.streams"} -->
