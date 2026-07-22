# 069-051 acceptance capability and reachability ledger

## Live authority boundary

Status: **GENERATION-2 T3/T4 TERMINAL SOURCE-BOUND REGENERATION IN PROGRESS.**

The sole public recursive V2 module path is
`z00z_storage::checkpoint::recursive_v2`. Duplicate top-level
`checkpoint::*` re-exports of evidence, receipt, proof, and sidecar types were
removed during T4. The sole admission/evidence ingress is:

```text
RecursiveCheckpointEvidenceStoreV2::produce(...)
    -> Result<RecursiveCheckpointEvidenceV2, RecursiveV2Error>
```

The canonical Rust declaration is the authority; callers do not use a copied
signature or a second module path.

Recursive evidence remains shadow/non-authoritative. It does not replace the
canonical checkpoint admission owned by Phase 068 storage.

Active candidate source revision is
`1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3`.
Authority generation is `2`; authority digest is
`9068b833988b340e6879d9345f7dba5dc79ca5f0625a45a53bc7e8118cb53216`;
the current-source role-framed verifier bundle pin is pending artifact
regeneration. The gate topology below is live code; current-source runtime
acceptance evidence must reproduce it before closure.

## Ordered live gate trace

| ID | Private typestate edge | Exact code action | Failure effect |
| ---: | --- | --- | --- |
| 1 | `AuthorityResolved` | `RecursiveAuthoritySnapshotV2::resolve_active_authority` for every block | `RecursiveV2Error::Authority`; no proof work/write |
| 2 | `FamilyCapSelected` | `resolve_verifier_authority_v2` and authority-selected object limits | authority/resource failure; no proof decode/write |
| 3 | `OuterDecodeBounded` | continuous proof returns a nonempty bounded envelope | limit/canonical failure; no write |
| 4 | `InnerDecodeBounded` | exact inner envelope is present and bounded | limit/canonical failure; no write |
| 5 | `CanonicalCurveValid` | Nova owner has accepted canonical curve/scalar encodings | canonical-encoding failure; no write |
| 6 | `BundleMatched` | proof bindings equal the authority-selected verifier bundle | authority failure; no write |
| 7 | `BackendVerified` | unchanged Nova compressed verifier has accepted | verifier failure; no write |
| 8 | `FinalStateLimbsMatched` | every returned final-state limb is present and matched | invariant/endpoint failure; no write |
| 9 | `StatementLinkPredecessorMatched` | `revalidate_chain` rechecks all canonical bindings | authority/lineage failure; no write |
| 10 | `StorageEndpointReloaded` | canonical transition/storage endpoint is reloaded | authority/endpoint failure; no write |
| 11 | `PrewriteComplete` | exact envelope bytes are reverified before persistence | verifier/canonical failure; no write |
| 12 | `AtomicWrite` | content-addressed sidecar write completes | I/O/collision failure; no receipt |
| 13 | `ByteReload` | bounded sidecar bytes are reloaded and strictly decoded | limit/canonical/I/O failure; no receipt |
| 14 | `PostwriteBackendVerified` | reloaded envelope is sent again to the unchanged verifier | verifier failure; objects quarantined/no receipt |
| 15 | `PostwriteEndpointMatched` | sidecar, authority, chain, and every endpoint limb match after reload | authority/endpoint failure; objects quarantined/no receipt |
| 16 | `ReceiptIssued` | private infallible `issue_postwrite` consumes the terminal typestate | only successful terminal; receipt has no decoder |

## Public V2 surface and capability disposition

| Public surface through `checkpoint::recursive_v2` | Constructor/decode reachability | Capability | Required evidence |
| --- | --- | --- | --- |
| `RecursiveCheckpointChainBlockV2::new` | public input bundle; no proof/verdict field | carries canonical store references and handoff into `produce`; cannot issue acceptance | `test_real_chain_public_receipt`; authority/lineage substitutions |
| `RecursiveCheckpointEvidenceStoreV2::open` | public evidence-root opener with private-directory checks | creates storage facade only | symlink/root/permission/collision tests |
| `RecursiveCheckpointEvidenceStoreV2::produce` | sole public proof attempt | can return evidence only after gates 1–16 | T3 1/3/5 chain; failpoint prefixes; postwrite verifier/reload |
| `RecursiveCheckpointEvidenceV2` | returned only by `produce`; fields are read-only evidence handles | no admission boolean or reusable capability | construction/caller search finds only terminal `Ok` in `produce` |
| `RecursiveCheckpointSidecarCodecV2::encode_bin` | public strict encoder | encodes reference-only shadow sidecar | codec roundtrip and canonical binding tests |
| `RecursiveCheckpointSidecarCodecV2::decode_bin` | sole public recursive sidecar decoder | creates non-authoritative sidecar data only | malformed/version/trailing/cap corpus; `check_shadow_sidecar_binding` |
| `RecursiveCheckpointSidecarV2` | public getters; constructor is crate-private | cannot be caller-created as accepted evidence | public-API scan and sidecar binding mutations |
| `RecursiveCheckpointProofV2` | bounded digest/length reference only | contains no proof bytes, PP/PK, verdict, or receipt | wire-size/field scan and cap tests |
| `CryptographicVerificationReceiptV2` | no public constructor, `Deserialize`, `Default`, or decoder | linear terminal evidence returned in memory only after G15 | `test_receipt_gate_is_last`; failpoint corpus; source scan proves `produce` is the only construction path |
| `RecursiveVerificationResultV2` | fixed public result tag inside receipt | data, never authority | no public receipt constructor/decoder |
| `RecursiveFinalizedIvcStateV2` | typed public prior/final endpoint | state input/output, not proof success | continuous-chain and predecessor mutations |
| `RecursiveCheckpointPublicInputV2` / `RecursiveTransitionStatementV2` | typed statement builders from canonical owners | cannot bypass verifier/endpoint gates | one-field statement/endpoint Model-C corpus |
| `RecursiveAuthoritySnapshotV2::resolve_active_authority` | public read-only resolver; source is live settlement/config store | yields immutable attempt authority, not caller-selected parameters | generation/digest/rotation tests |
| `RecursiveCircuitProfileV2` / `RecursiveCircuitSpecV2` | public inspected authority data; proof path uses pinned values | no runtime width/profile selection | constructor exact/cap+1 and digest substitution tests |
| `CanonicalCheckpointTransitionV2::from_exec` | public integration/test constructor through the sole V2 facade | creates canonical transition, never receipt/admission | real HJMT/evaluator/trace tests; only `produce` uses verifier-enabled private constructor |
| `SettlementRootGenerationCutoverV2` | explicit V2 cutover utility | cutover identity only, not recursive proof acceptance | exactly-once/restart/CAS/crash/state-substitution corpus |
| `RecursiveTraceOpcodeV2`, counts and precommit views | read-only trace vocabulary/commitments | no raw witness/event constructor is exported through facade | grammar/count/source-control scans and R1CS mutations |

## Production call path

```text
attacker/prover bytes
  -> RecursiveCheckpointEvidenceStoreV2::produce
  -> resolve active generation-2 authority for every block
  -> CanonicalCheckpointTransitionV2::from_exec_with_verifier (crate-private)
  -> prove_continuous_chain_v2 (crate-private Nova owner)
  -> strict envelope/curve/bundle checks
  -> unchanged compressed verifier
  -> every final-state limb + statement/link/predecessor comparison
  -> canonical storage endpoint reload
  -> prewrite exact-byte verification
  -> content-addressed envelope and sidecar writes
  -> bounded byte reload + strict sidecar decode/binding
  -> unchanged compressed verifier again
  -> endpoint/authority/storage comparison again
  -> claim persistence
  -> private CryptographicVerificationReceiptV2::issue_postwrite
  -> in-memory receipt plus receipt digest returned to the caller
```

There is no branch from deserialized sidecar bytes or caller-supplied
receipt-shaped bytes to receipt creation, canonical checkpoint admission,
verifier-bundle selection, or parameter selection. The Phase-069 path has no
receipt decoder and does not persist or reload receipt bytes; durable receipt
publication remains downstream scope.

## Deleted/banned surface zero-reachability contract

The following names must have zero live declarations, exports, callers,
generated-doc symbols, features, dynamic-dispatch edges, and decoders. Exact
string literals are permitted only inside negative source guards that assert
the live owner does not contain the banned surface:

```text
accepted
verifier_verdict
verify_sidecar
CheckpointNovaCircuitV1
RecursiveCircuitInputV1
NovaCompressedBlockProofV1
RecursiveCheckpointReceiptV1
LibrarySmoke
recursive_v2::nova
recursive_v2/nova.rs
```

`verify_sidecar`, `verifier_verdict`, and the retired nested Nova path occur in
negative guard assertions only; they have no declaration or call edge. The
lowercase word `accepted` may occur only in prose/test descriptions of an
accepted block; it may not be a field, constructor, verdict, decoder result,
or capability. Historical V1 strings remain allowed only in the T0 eradication
manifest and inert content-addressed reject fixtures.

## Reachability evidence

- CodeGraph exploration on 2026-07-20 found `produce` as the sole admission-like
  V2 method and only private receipt construction in `adapter.rs`/`receipt.rs`.
- Direct source reads confirmed the exact gate sequence and that receipt bytes
  have no decoder.
- Repository import scans found the codec integration test as the only caller
  of the former duplicate root sidecar re-export; it now imports the canonical
  `checkpoint::recursive_v2` path.
- Runtime instrumentation in `test_real_chain_public_receipt` exercises every
  production edge; `test_receipt_failpoints_keep_prefix` exercises the exact
  failure prefix and proves no later success edge occurs.

Final closure requires the terminal T4 source scan, release suite, and two
consecutive clean review passes. Any unknown public constructor/caller/dynamic
edge, receipt decoder, alternate facade, or path to G16 that omits G01..G15 is
S1 and reopens this ledger.
