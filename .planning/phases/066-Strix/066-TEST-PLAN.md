---
phase: 066-Strix
artifact: test-plan
status: compatibility-alias
source: 066-TESTS-TASKS.md
updated: 2026-07-03
---

# Phase 066 Test Plan

**Phase:** `066`
**Status:** Implemented compatibility alias for the live test packet

## Purpose

This file is a compatibility index for the live Phase 066 test packet.
Use it as the shortest entrypoint into the implemented and verification-backed
test artifacts generated for the phase.

## Canonical Test Artifacts

- `066-TEST-SPEC.md` - detailed unit, contract, integration, and end-to-end
  coverage requirements
- `066-TESTS-TASKS.md` - implementation task breakdown for another engineer or
  agent

## Classification Summary

- Unit and contract tests are authoritative for validators, manifests, report
  classification, prompt contracts, profile routing, and Docker scope guards.
- CLI integration and end-to-end tests are authoritative for runner behavior,
  artifact generation, pack portability, Docker check-only behavior, and host
  report export.
- Browser E2E is not relevant to Phase 066.

## Required Test Outcomes

- Local-only scope enforcement must be proven.
- Missing tools must be recorded truthfully.
- Confirmed findings must require evidence mapping.
- `.codex` surfaces must stay symlink-based and canonical.
- Pentest Docker must remain archive-driven and isolated from formal
  verification tooling.
- Z00Z proof-boundary findings must route to existing audit surfaces without
  vendor-code edits.

## Use

Reviewers and implementers should read `066-TEST-SPEC.md` first, then use
`066-TESTS-TASKS.md` as the execution and validation breakdown for the live
test surface.
