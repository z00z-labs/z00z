# 069-051 T2-T4 mutation ledger

Status: **T2 CLOSED; T3/T4 TERMINAL SOURCE-BOUND REGENERATION IN PROGRESS.**
Historical sections remain explicitly labelled as such. The active candidate
source revision is
`1da05771ae22d8da4b8e8693954540f468708be47f25f7dc654a0f7f9df4c4e3`;
authority generation is `2`; its role-framed verifier bundle pin is pending
current-source artifact regeneration.

The preceding-source generation-2 artifact packet was generated once with a zero authority pin
and again after installing the active role pin; PP/PK/VK bytes are identical.
All three exact artifact milestones passed, including the 15,372,615-byte
format-4 verifier bundle, strict key/compact-grammar mutation corpus, and real
source-binding worker.
The preceding-source T3 public-ingress/storage-reload chain passed from one `z_0` at blocks
`1/3/5`, cumulative steps `316/948/1580`; block 1 is fold-only, while the
explicit snapshots at blocks 3 and 5 each emitted a `346,859 B` envelope. Its
mutation results remain architectural evidence but are not active-candidate
acceptance until the current artifact/proof/T3 corpus reproduces them.

| Boundary | Mutation preserving unrelated framing | Intended gate/evidence | Status |
| --- | --- | --- | --- |
| Source window | byte, source/chunk ordinal, count, byte count, cursor, zero tail | direct writer/reader equality in `test_source_window_binding` | killed |
| Source/global SHA | role, stage, block ordinal, padding, length, chaining word | FIPS/control binding corpus | killed |
| Replay grammar | output→input reorder and invalid op kind | replay phase/parser relation | killed |
| Replay→original row | definition/serial/terminal/value hash independently | adjacent full semantic-row equality | killed |
| Precommit | version, counts, each digest family | streamed precommit parser/count binding | killed |
| Challenge | committed-precommit byte and derived challenge byte | SHA-derived challenge equality | killed |
| Sorted uniqueness | version, set/pass/list tag, limb, order, count | exact row parser, strict order, products | killed |
| Net | case tag, path field, old/new hash, pending side, Close field/count | semantic Net relation | killed |
| Net→JMT | product factor, mutation count, terminal operation row | two products plus exact count | killed |
| JMT | header version/kind/count/digest and each micro-op key/value/sibling/order | JMT fixed machine | killed |
| Hierarchy | role, ordinal, child root, parent key/value, definition root | hierarchy induction | killed |
| Settlement roots | policy/layout/definition/finalize post-root byte | exact SHA/root binding | killed |
| Typed commitments | kind/order/digest/progress | four-commitment relation | killed |
| Final successor | declared opcode count, done bit, endpoint limb | unique finalization and independent expected state | killed |
| Verifier bundle identity | selected bundle digest, PP/authority/profile/spec/source/lock/VK digest, width/generation/bounds/shape and payload lengths | authority selection/header validation before key/proof decode | killed by fresh real-key run |
| Verifier bundle key codec | noncanonical scalar/curve encodings; canonical identity/default blinder; primary/secondary generator swap | noncanonical point/scalar reach strict dependency decode under a recomputed selected bundle; `validate_pinned_verifier_key_wire` walks all pinned Pedersen/IPA key ranges and rejects identity, default, order and derandomization-key drift before proof decode | killed by fresh real-key run; Pallas/Vesta have cofactor one, so no separate nonidentity wrong-subgroup point exists |
| Prover material | cap+1, lengths, digests, trailing fields, identity mutations and canonical PP/PK mismatch | private recovery loader plus deterministic PK regeneration | killed by generation-2 fresh strict roundtrip; PP 958,329,152 B, PK 208 B, framed material 958,329,882 B |
| Proof envelope | body byte, endpoint limb, bundle/public-input/prior/successor digest | strict envelope then unchanged verifier | killed |
| Model A | malformed canonical transition before circuit | independent evaluator | killed by semantic corpus |
| Model B | satisfying unrelated framing with changed intermediate relation state | TestCS constraint name and full state differential | killed by per-family R1CS corpus |
| Model C | independently recomputed complete mixed proof under the same PP/VK with a changed typed predecessor/statement | unchanged compressed VK-only verification accepts the candidate for its own state; the sole all-limb target comparator rejects it | killed on a transient generation-2 `1..=1` bundle sharing the active authority/PP/VK/shape/source; recomputation `1,166.614 s`, proof `123,688 B`, target comparator rejected; active `1..=5` acceptance is separately covered by the artifact/T3 corpus |

## Live acceptance-gate mutations

| Gate | Valid control | Minimally changed negative | Actual deepest result | Status |
| --- | --- | --- | --- | --- |
| `AuthorityResolved` | one immutable generation-2 authority for all blocks | generation/config/profile/snapshot rotation | exact G01 failure prefix; no proof/write | killed |
| `FamilyCapSelected` | selected Nova proof family and active bundle | wrong family/cap/bundle/PP/profile/spec/source | exact G02/G06 failure prefix; no proof-selected authority | killed |
| bounded outer/inner decode | canonical nonempty envelope | cap+1, empty, truncated, nested length, trailing bytes | G03/G04; unchanged verifier not claimed reached | killed |
| canonical curve wire | strict Pallas/Vesta scalar/point encodings | identity/default/order/key swap/noncanonical encoding | G05 before proof use | killed |
| initial verifier | active GEN2 proof/bundle | proof byte, key, count, `z_0`, statement or source change | G07 verifier or G08/G09 endpoint rejection | killed by active artifact/T3 corpus |
| storage endpoint reload | canonical checkpoint/link/predecessor/state | stale/reorg/reordered/skipped predecessor or store rotation | G09/G10 exact prefix | killed |
| atomic object write | new content-addressed path or byte-identical idempotent object | collision, symlink, permission, partial write, authority rotation | G11/G12 exact prefix and quarantine | killed |
| sidecar byte reload | canonical reference-only V2 sidecar | cap/version/type/trailing/binding/byte mismatch | G13 or retained binding failure | killed |
| postwrite verifier | byte-identical reloaded envelope | changed persisted envelope/proof/bundle | G14 verifier rejection; no receipt | killed |
| postwrite endpoint | reloaded sidecar + canonical chain + exact final limbs | changed sidecar/link/root/endpoint/generation | G15 rejection/quarantine; no receipt | killed |
| receipt issuance | private terminal typestate after G15 | early/replayed/deserialized/default/clone capability | G16 unreachable from mutation | killed |

Checksum-only or native-decoder rejection does not substitute for R1CS or
Model C. The fresh generation-2 Model C now passes. Final closure still requires
the active generation-2 36-test semantic corpus, full TestCS, terminal release
suite, and two consecutive clean review passes. Until those facts are appended,
this ledger must not be reported as T4 closed.
