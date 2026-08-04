---
id: wallet.import
title: "Wallet: Import"
route: wallet.import
scope: context
---

# Wallet: Import

[TOC]

## App View {#current-view}

![Wallet: Import application view](help/assets/en/wallet-import.png)

This image is captured from the live Demo view.

## Overview

Import claims a public `AssetPkgWire` JSON package from local disk into the active wallet. The file is read locally and its JSON content—not its filesystem path—is prepared for `wallet.asset.import_asset`.

## How to use this view

1. Confirm the active wallet and chain in the application header.
2. Choose a public asset package no larger than 64 KiB.
3. Review the asset definition, amount, serial, domain, state flags, owner-binding mode, and cryptographic field presence.
4. Select **Import asset** to continue to native wallet verification.
5. Treat an `asset_already_exists` result as an idempotent outcome; do not retry a rejected claim without reviewing its `IMPORT_*` reason.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Definition ID | Public 32-byte identifier from the asset definition. It is not a wallet secret. |
| Serial ID | Instance serial inside the definition's declared serial range. |
| Amount and decimals | Public atomic amount and the decimal display rule from the definition. |
| Domain | Declared definition domain. It is metadata, not a trust verdict. |
| Declared serials and nominal amount | Definition-level issuance limits. They are reviewed separately from this instance's serial and amount. |
| Lock height | Optional height before which the asset cannot be spent. |
| Burned, Frozen, Slashed | Protocol-state flags that can make an otherwise valid package unusable. |
| Owner binding | Either a direct owner signature or a complete stealth receiver tuple. |
| Range proof | Optional public proof material; the native wallet verifies it when present. |
| Metadata and Tag 16 | Optional public definition metadata and compact leaf tag. The view shows their presence without treating them as authority. |
| Import result | `asset_id`, `serial_id`, `symbol`, `class`, status, `is_inserted`, and `asset_already_exists`. |

## Safety and limits

- A top-level `secret` field is forbidden.
- The JavaScript demo checks the public shape only. The native Rust wallet owns cryptographic verification, active-wallet ownership, claim/nullifier reservation, persistence, finalization, replay protection, and quarantine.
- Unknown fields fail closed because the canonical DTO uses `deny_unknown_fields`.
- The view does not store or transmit an absolute local file path.

<!-- help-sync:source {"page_path":"wallet/import.md","route_id":"wallet.import","screenshot":"help/assets/en/wallet-import.png","topic_id":"wallet.import"} -->
