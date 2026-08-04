---
id: settings.general
title: "Settings: General"
route: settings.general
scope: context
---

# Settings: General

[TOC]

## App View {#current-view}

![Settings: General application view](help/assets/en/settings-general.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

General settings control how this application presents language-dependent information. They do not change wallet keys, asset denominations, balances, or settlement state.

## How to use this view

1. Choose the application language.
2. Choose the regional format used for dates, numbers, and decimal separators.
3. Choose the currency used to express coin values. USD is the default.
4. Choose the time zone used for displayed timestamps.
5. Open **Wallet: Assets** to confirm that the **Value** column uses the selected ISO currency code.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Language | Selects the translated application catalogue. |
| Regional format | Controls localized number, date, and decimal formatting independently from the application language. |
| Currency | Selects the national currency used to display coin values. The menu uses national flags for the language regions supported by the application. |
| Time zone | Converts stored UTC timestamps for display without changing the stored time. |

## Safety and limits

- Currency is a presentation preference. It does not exchange funds or change the denomination of an asset.
- The Demo has no authoritative market-price feed. A selected currency formats available value fields but never turns an unavailable price into an estimate.
- These controls update the current local concept and its YAML draft. Production persistence requires a revisioned runtime settings capability.

<!-- help-sync:source {"page_path":"settings/index.md","route_id":"settings.general","screenshot":"help/assets/en/settings-general.png","topic_id":"settings.general"} -->
