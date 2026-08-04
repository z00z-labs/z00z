---
id: about
title: "App: About"
route: about
scope: context
---

# App: About

[TOC]

## App View {#current-view}

![App: About application view](help/assets/en/about.png)

This image is captured from the live Demo view. Review the current interface before publishing explanatory guidance.

## Overview

About identifies the current Z00Z Wallet Demo version, explains its purpose,
links to the legal pages and source repository, and provides a local update
status check.

The JavaScript Demo defines the desktop and mobile UX target for the future
native Rust and Tauri application. A demonstrated screen is not automatically
evidence that its production service is implemented.

## How to use this view

1. Confirm the displayed product version when reporting or reproducing a UI
   issue.
2. Open Privacy Policy or Terms of Use to review the current published legal
   documents.
3. Open the Z00Z GitHub repository to inspect the public source and project
   history.
4. Select **Check for updates** to refresh the Demo's local status message for
   this session.

## Terms and controls

| Term or control | Explanation |
| --- | --- |
| Z00Z Wallet v0.1.0 | Version declared by the current Demo runtime contract. |
| Privacy Policy | Opens the published Z00Z privacy document in a separate tab. |
| Terms of Use | Opens the published Z00Z terms document in a separate tab. |
| Visit Z00Z GitHub repository | Opens the public `z00z-labs/z00z` repository in a separate tab. |
| Check for updates | Shows the version known to this Demo session; it does not install software. |

## Safety and limits

- The Demo does not download or install an update.
- A packaged application must verify a signed release manifest before offering
  an update.
- External links use a separate tab and do not receive wallet secrets from
  Help.
- Check the displayed version again after changing builds; Help does not claim
  that a local or deployed copy is the newest release.

<!-- help-sync:source {"page_path":"about/index.md","route_id":"about","screenshot":"help/assets/en/about.png","topic_id":"about"} -->
