## 1. State Management Current Spec

**Goal:**

- Make the moved `050-state-mgmt` current spec the self-contained source for state-management implementation work.
- Prove every current state-management requirement maps to local phases, crate tests, and simulator evidence without relying on obsolete Phase 049 paths.

**Source:**

- [State management current spec, purpose and authority](../.planning/phases/050-state-mgmt/045-NEW-State-Management-Spec.md#purpose)
- [State management current spec, required execution order](../.planning/phases/050-state-mgmt/045-NEW-State-Management-Spec.md#required-execution-order)
- [State management current spec, consolidated implementation backlog](../.planning/phases/050-state-mgmt/045-NEW-State-Management-Spec.md#consolidated-implementation-backlog)
- [State management current spec, validation and test work that still matters](../.planning/phases/050-state-mgmt/045-NEW-State-Management-Spec.md#validation-and-test-work-that-still-matters)

**Implementation-relevant fragments:**

- Use the purpose and authority fragment to treat the moved `050-state-mgmt` spec as the current source, not obsolete Phase 049 paths.
- Use required execution order and consolidated backlog to map state-management items into phases `0`, `8`, `9`, `10`, `13`, `14`, `16`, and `19` through `21`.
- Use validation and test work for the exact local test classes that still matter after earlier wallet redesign work.
- Do not use this section to add live validator networking or a second scanner/checkpoint authority.

**Locality gate:**

- The spec describes local storage, wallet, validator-facing verification, and simulator work that can be implemented through local crate tests and deterministic simulator runs.
- No old Phase 049 directory, live network, testnet, or external DA source is needed.

**Implementation boundary:**

- In scope: using this spec as the current state-management authority for claim roots, checkpoint verifier convergence, checkpoint flow over storage authority, receive taxonomy, scan orchestration, nullifier reserved-to-spent bridge, regression waves, and the ordering requirement that these slices land on the phase `0` storage facade.
- Out of scope: resurrecting obsolete 049 material, adding a second scanner authority, adding a second checkpoint verifier, or implementing live validator networking as part of this local plan.

**Implementation tasks:**

1. Treat this spec as the source for phases `0`, `8`, `9`, `10`, `13`, `14`, `16`, `19`, `20`, and `21`.
2. Map honest `claim_root` propagation into storage tests and simulator checkpoint evidence.
3. Map shared authoritative checkpoint proof verification into one `z00z_storage` verifier entrypoint.
4. Map validator checkpoint flow into local verdict/checkpoint tests without adding live validator networking.
5. Map explicit unsupported receive taxonomy into wallet receive/report/import APIs.
6. Map real wallet scan-engine orchestration into one cursor model and runtime scan status.
7. Map nullifier reserved-to-spent bridge into wallet/storage transition tests.
8. Keep obsolete ideas dropped unless the current spec explicitly reopens them.
9. Maintain a traceability check from each current spec backlog item to one or more full `Z00Z-LOCAL-PLANS` sections.

**Tests and simulation:**

- Traceability check: every consolidated backlog item has a corresponding local plan section and test category.
- Claim-root and checkpoint tests from phases `8` and `9`.
- Receive taxonomy and scan orchestration tests from phases `13` and `14`.
- Nullifier transition and wallet import-boundary tests from phases `10` and `16`.
- Simulator closure tests from phases `20` and `21`.
- Drift check proving obsolete Phase 049 references are not used as current source truth when the moved `050-state-mgmt` files exist.

**Done when:**

- The state-management current spec is fully represented by local implementation phases and tests, including the phase `0` storage-facade dependency.
- Developers can delete or ignore superseded roadmap fragments without losing actionable state-management work.
- No state-management task depends on a live network or testnet.

**Doublecheck:**

- Local condition: satisfied. The spec maps to local storage, wallet, and simulator work.
- Developer clarity: satisfied. The exact downstream phases and backlog mappings are named.

## 2. State Management Execution Backlog

**Goal:**

- Convert the state-management TODO into an ordered local execution checklist for claim roots, checkpoint verifier convergence, validator-facing verdict tests, receive taxonomy, scan orchestration, nullifier bridge, and simulator closure.
- Prove the backlog is schedulable as local crate and simulator work with explicit dependency order, test waves, and closure gates.

**Source:**

- [Roadmap blueprint, section 10.4: Concrete Implementation Sequence](Z00Z-Roadmap-Blueprint.md#104-concrete-implementation-sequence)
- [Roadmap blueprint, section 12.1: Verification Gates](Z00Z-Roadmap-Blueprint.md#121-verification-gates)
- [State management TODO, decision summary](../.planning/phases/050-state-mgmt/045-TODO.md#decision-summary)
- [State management TODO, concrete execution tasks](../.planning/phases/050-state-mgmt/045-TODO.md#concrete-execution-tasks)
- [State management TODO, mandatory test waves](../.planning/phases/050-state-mgmt/045-TODO.md#mandatory-test-waves)
- [State management TODO, phase closure gate](../.planning/phases/050-state-mgmt/045-TODO.md#phase-045-closure-gate)

**Implementation-relevant fragments:**

- Use roadmap section 10.4 only for high-level implementation sequence and scheduling discipline.
- Use roadmap section 12.1 only for verification-gate posture: targeted crate tests first, simulator/theorem evidence second, broad checks after stable slices.
- Use the TODO decision summary, concrete execution tasks, mandatory test waves, and closure gate as the authoritative 045 backlog.
- Do not schedule node rollout, DA rollout, or external bridge work from this backlog.

**Locality gate:**

- The backlog is a local execution checklist over storage, wallet, simulator, and local validator-facing tests.
- It can be completed with unit tests, integration tests, and simulator stages; no live chain or testnet is required.

**Implementation boundary:**

- In scope: executing and verifying tasks 045-01 through 045-12 as local code/test work, preserving dependency order, and preventing seam duplication.
- Out of scope: adding new protocol concepts not in the backlog, replacing current wallet/import/checkpoint seams, or turning simulator closure into production closure.

**Implementation tasks:**

1. Complete 045-01 claim-source baseline recheck and optional cleanup decision in `z00z_storage`.
2. Complete 045-02 honest `claim_root` propagation in checkpoint batches.
3. Complete 045-03 shared checkpoint proof verifier and seal/reload convergence.
4. Complete 045-04 validator checkpoint flow over storage authority as local verifier/verdict tests.
5. Complete 045-05 explicit unsupported-version receive taxonomy in `z00z_wallets`.
6. Complete 045-06 real wallet scan-engine orchestration and runtime scan status.
7. Complete 045-07 explicit nullifier reserved-to-spent bridge.
8. Complete 045-08 harness and seam-reuse lock-in to prevent duplicate helpers.
9. Complete 045-09 checkpoint claim-root and proof-verifier wave.
10. Complete 045-10 receive taxonomy and import-boundary regression wave.
11. Complete 045-11 nullifier transition regression wave.
12. Complete 045-12 simulator closure and release regression wave.
13. Package phase `0`, phases `9`, `13`, `14`, `15`, `16`, `19`, `20`, and `21` as the mandatory local execution chain, where phase `0` means `0.1` boundary first, `0.2` authority-facade consumption next, and `0.3` forest rollout as early overlapping infrastructure; keep phase `27` as an optional measurement sidecar.
14. If a GSD phase is created from this document, include one context artifact, one phase TODO, one plan per local slice, one summary per completed slice, and one phase-level verification artifact after simulator closure.
15. Preserve the single-team order `0.1 -> 0.2 -> 9 -> 13 -> 14 -> 15 -> 16 -> 19 -> 20 -> 21`, start `0.3` as soon as `0.1` and `0.2` make that safe, and keep phase `27` only when measurement evidence is desired.
16. On a split team, start phase `0.1` first, start `0.2` next, let one engineer or agent carry `0.3` once the facade is stable, keep phase `9` next, keep phases `13 -> 14 -> 15 -> 16` as the wallet critical path on top of the stable facade, then split checkpoint evidence phase `20` and wallet evidence phase `21` after phase `19` is stable.
17. Start broad workspace commands only after targeted crate and simulator evidence exists for the slice under review.

**Tests and simulation:**

- Storage unit tests for claim roots, checkpoint proof verifier, seal/reload convergence, and tamper rejection.
- Wallet unit tests for unsupported receive versions, scan orchestration, import-boundary behavior, and nullifier transition.
- Local validator-facing tests for checkpoint verdict behavior over storage authority.
- Simulator closure wave covering receive, import, checkpoint, theorem, tamper, and restart evidence.
- Harness tests proving existing seams are reused and no parallel claim-source, scanner, or checkpoint verifier is added.
- Phase-level verification artifact proving targeted crate tests ran first, simulator/theorem tests ran second, and broad workspace checks were saved for stable slices.
- Negative scheduling check proving the packet did not create node-process rollout, DA-provider rollout, live OnionNet work, governance work, extension work, or duplicate wallet/storage authority planes.

**Done when:**

- All 045 backlog tasks map to implemented local code or explicit remaining local work.
- Mandatory test waves have deterministic commands and expected pass/fail classes.
- The closure gate can be evaluated from this document, the live source specs, and the moved `050-state-mgmt` files without any external bridge note or legacy Phase 049 path.

**Doublecheck:**

- Local condition: satisfied. Every task is local crate or simulator work.
- Developer clarity: satisfied. Backlog task IDs, crate owners, and test waves are explicit.

## 
