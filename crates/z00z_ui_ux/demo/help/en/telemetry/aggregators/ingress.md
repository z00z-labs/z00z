---
id: telemetry.aggregators.ingress
title: Aggregator ingress
summary: Ingress explains how the runtime admits a transaction or claim payload as a digest-bound work item.
scope: context
---
## Use this view {#current-view}
- Check the `WorkPayload` to `WorkItem` or `RejectRecord` contract.
- Unavailable means no fresh admission snapshot exists; it does not mean accepted or rejected.

## Fail-closed boundary
- Object-package binding changes the admission digest and intake identity.
- Raw payloads, receivers, memos, and wallet-local routes never enter Help.
