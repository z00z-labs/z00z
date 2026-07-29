---
id: settings.reticulum
title: Settings: Reticulum
route: settings.reticulum
scope: context
---

# Settings: Reticulum

[TOC]

## App View {#current-view}

![Settings: Reticulum application view](help/assets/en/settings-reticulum.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

The **Network** view groups Reticulum, OnionNet, and QUIC without mixing their responsibilities. Reticulum controls the local carrier boundary. Wallet keys, OnionNet privacy policy, and QUIC session behavior remain separate.

## How to use this view

1. Choose an **Interface profile** appropriate for the device and carrier.
2. Limit **Discovery scope** to the smallest environment that needs peer discovery.
3. Enable or disable local **Peer discovery**.
4. Choose whether links are retained for ingress, created on demand, or managed manually.
5. Open **Telemetry: Reticulum** for read-only runtime evidence.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Interface profile | Preferred interface set: automatic selection, local AutoInterface, Backbone/TCP client, or RNode/LoRa. |
| Discovery scope | Boundary for automatic discovery. Link local is the conservative default. |
| Peer discovery | Allows compatible interfaces and peers to be found within the selected scope. |
| Link strategy | Controls whether ingress links are retained, opened on demand, or managed manually. |
| Network identity | A Reticulum transport identity kept separate from wallet signing material. |

## Safety and limits

- The current wallet Demo stores these controls as a local target draft; it has no authoritative Reticulum configuration bridge.
- Selecting a value does not prove that a local service accepted or applied it.
- Wider discovery can require explicit network and firewall configuration.
- Transport configuration must never expose wallet seeds, signing keys, contacts, or destinations.

<!-- help-sync:source {"page_path":"settings/reticulum.md","route_id":"settings.reticulum","screenshot":"help/assets/en/settings-reticulum.png","topic_id":"settings.reticulum"} -->
