---
id: extensions.create-asset
title: "Extensions: Create Asset"
route: extensions.create-asset
scope: context
---

# Extensions: Create Asset

[TOC]

## App View {#current-view}

![Extensions: Create Asset application view](help/assets/en/extensions-create-asset.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

Create Asset proposes an immutable asset definition and bounded initial
issuance. It exposes the definition fields used by the asset model without
granting the Extension registry, signing, issuance, or settlement authority.

**Boundary:** a Z00Z Extension does not control the wallet. It proposes a typed
action. Wallet checks scope, builds the package, requests confirmation, and
only then passes it to the settlement path.

## How to use this view

Choose Coin, Token, or NFT, then enter the immutable name and display symbol.
Set decimals, declared serial count, nominal atomic units per serial, issuer
namespace, and supply policy. An NFT definition must use zero decimals.
Optional metadata is represented by a canonical digest, never a local file.

Wallet checks class constraints, issuer authority, definition conflicts, and
supply and policy bounds before it can propose registration or issuance.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Asset class | Coin, Token, or NFT; the class determines definition constraints. |
| Name and symbol | Immutable human-readable identity fields used to identify the definition. |
| Decimals | Display precision; NFT definitions require zero. |
| Declared serials | Number of serial instances declared by the definition. |
| Nominal units per serial | Atomic units assigned to each declared serial. |
| Issuer namespace | Declared issuer or organization namespace reviewed with the definition. |
| Supply policy | Fixed, burnable, or bounded additional issuance policy proposed for Wallet review. |
| Metadata digest | Optional canonical commitment to metadata; it is not a filesystem path or remote program. |
| Evidence | Definition digest, registry decision, and issuance receipt remain separate evidence stages. |

## Safety and limits

The Extension cannot register a definition, issue value, select Wallet objects, or
sign a package. A locally prepared definition is not an accepted registry
entry, an issued asset, or checkpoint settlement. Wallet and protocol checks
remain authoritative at every later stage.

<!-- help-sync:source {"page_path":"extensions/create-asset.md","route_id":"extensions.create-asset","screenshot":"help/assets/en/extensions-create-asset.png","topic_id":"extensions.create-asset"} -->
