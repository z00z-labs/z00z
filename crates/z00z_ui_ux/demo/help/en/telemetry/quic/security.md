---
id: telemetry.quic.security
title: Telemetry QUIC: Security
route: telemetry.quic.security
scope: context
---

# Telemetry QUIC: Security

[TOC]

## App View {#current-view}

![Telemetry QUIC: Security application view](help/assets/en/telemetry-quic-security.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

This read-only view exposes sanitized TLS-integrated QUIC security state. It reports handshake and key-phase evidence without exposing keys, tickets, tokens, certificates, or peer identity material.

## How to use this view

1. Confirm that the handshake state came from an authoritative QUIC runtime.
2. Read the negotiated cipher suite as metadata, not as proof of peer authorization.
3. Review key-phase updates only as counts and state transitions.
4. Keep state-changing wallet actions out of 0-RTT early data even if the transport runtime supports it.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Handshake confirmation | Whether the TLS-integrated QUIC handshake reached confirmed state. |
| Cipher suite | Sanitized negotiated TLS cipher-suite name. |
| Key phase updates | Aggregate evidence of packet-protection key phase changes and failures. |
| 0-RTT early data | Data sent before handshake completion; wallet policy excludes state-changing actions from it. |

## Safety and limits

- Never expose traffic secrets, packet-protection keys, session tickets, address-validation tokens, reset tokens, raw certificates, or Connection IDs.
- A confirmed transport handshake does not prove that a wallet action was authorized or settled.
- **Unavailable** means the runtime did not provide authoritative security metadata.

<!-- help-sync:source {"page_path":"telemetry/quic/security.md","route_id":"telemetry.quic.security","screenshot":"help/assets/en/telemetry-quic-security.png","topic_id":"telemetry.quic.security"} -->
