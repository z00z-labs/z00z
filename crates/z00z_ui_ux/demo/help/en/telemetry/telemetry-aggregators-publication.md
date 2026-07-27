---
id: telemetry.aggregators.publication
title: Aggregator publication
summary: Publication explains how an ordered batch is bound to checkpoint, quorum, data-availability, and lifecycle evidence.
scope: context
---
## Use this view {#current-view}
- Follow `PublicationRequest` to `PublishedBatch` and `PublicationRecord`.
- Unavailable means no verified publication or readiness bundle is connected.

## Fail-closed boundary
- Partial or mismatched provider, height, manifest, payload, statement, or evidence data is rejected.
- Storage owns checkpoint roots, proofs, and lifecycle truth.
