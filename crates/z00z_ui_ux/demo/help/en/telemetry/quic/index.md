---
id: telemetry.quic.overview
title: Telemetry QUIC: Overview
route: telemetry.quic.overview
scope: context
---

# Telemetry QUIC: Overview

[TOC]

## App View {#current-view}

![Telemetry QUIC: Overview application view](help/assets/en/telemetry-quic-overview.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

This read-only view summarizes whether authoritative QUIC connection, path, stream, recovery, and security evidence is available. It does not configure transport or synthesize live-looking values.

## How to use this view

1. Read the capability banner before interpreting any metric.
2. Use the local tabs to inspect connections, paths, streams, recovery, and security separately.
3. If every metric is **Unavailable**, configure or start the native runtime bridge outside this view.
4. Use Settings only for configuration; Telemetry remains read-only.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Runtime capability | Availability of an authoritative native QUIC status source. |
| Handshake state | Sanitized aggregate state of TLS-integrated QUIC connection establishment. |
| Active streams | Aggregate multiplexed stream count without endpoint or contact details. |
| Path migration | Verified evidence that a connection changed network paths. |

## Safety and limits

- No endpoint addresses, connection identifiers, session tokens, contacts, or payload contents are exposed.
- **Unavailable** is not a transport failure; it means this UI has no authoritative observation.
- QUIC status does not prove OnionNet privacy or Reticulum carrier health.

<!-- help-sync:source {"page_path":"telemetry/quic/index.md","route_id":"telemetry.quic.overview","screenshot":"help/assets/en/telemetry-quic-overview.png","topic_id":"telemetry.quic.overview"} -->
