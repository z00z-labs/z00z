# Article Review Ideas for Z00Z

## 🎯 Executive Synthesis

This review is based on a fresh pass over the extracted `docs/articles` corpus and the current Z00Z code/docs surface. It does not treat earlier repository notes as evidence. The strongest conclusion is that the article corpus does not argue for replacing Z00Z's architecture. It argues for sharpening architectural hooks Z00Z already has: request-aware receiving, policy-ready settlement rights, wallet-local privacy decisions, advisory remote scan evidence, and checkpoint-bound public settlement.

The highest-value implementation work is not a new mixer, not a general scripting VM, and not a trace backdoor. The best fit is a set of narrow, auditable protocol and wallet upgrades:

- wallet-local privacy linting before package creation and after receive scan;
- typed selective-disclosure and retention policies using the existing `RightLeaf` policy fields;
- typed remote scan proof hints and helper accountability without making helpers receive authorities;
- checkpoint data-availability evidence tied to `CheckpointArtifact`, `CheckpointLink`, and `SettlementStateRoot`;
- bounded right-policy constraints inspired by TariScript, but expressed as settlement rights rather than a general VM;
- research-only offline transferable e-cash that remains explicitly below the checkpoint settlement boundary until solved.

The ideological fit is strongest where papers preserve privacy by default but allow explicit, scoped accountability. The fit is weakest where papers assume permanent global tracing, a trusted coordinator as a core authority, or offline bearer finality without an online double-spend boundary.

## 🔑 Current Z00Z Anchors

The current repository already contains several surfaces that map directly to the papers.

| Z00Z surface | Current role | Article-derived opportunity |
| --- | --- | --- |
| [crates/z00z_storage/src/settlement/record.rs](../../crates/z00z_storage/src/settlement/record.rs) | `SettlementLeaf::Right(RightLeaf)` with `RightClass`, holder/control/payload commitments, validity windows, `use_nonce`, and revocation/transition/challenge/disclosure/retention policy IDs. | Make accountable privacy and regulated rights first-class without adding a tracing backdoor. |
| [crates/z00z_storage/src/settlement/identity.rs](../../crates/z00z_storage/src/settlement/identity.rs) | `SettlementStateRoot` is semantic; `CheckRoot` is checkpoint-facing; `TxDigest::to_check()` rejects digest/root mixing. | Keep DA, proof, and package ideas rooted in typed root boundaries. |
| [crates/z00z_rollup_node/src/lib.rs](../../crates/z00z_rollup_node/src/lib.rs) | `verify_settlement_theorem()` binds `TxPackage`, `CheckpointArtifact`, `CheckpointExecInput`, and `CheckpointLink`; range proofs alone are not settlement closure. | Data availability and audit ideas must attach to the checkpoint theorem, not bypass it. |
| [crates/z00z_wallets/src/receiver/payment_request_types.rs](../../crates/z00z_wallets/src/receiver/payment_request_types.rs) | Signed one-time `PaymentRequest` with `req_id`, chain ID, amount, expiry, metadata, and identity signature. | WabiSabi/CoinJoin usability lessons fit as request freshness, identity, and privacy warnings. |
| [crates/z00z_wallets/src/receiver/request.rs](../../crates/z00z_wallets/src/receiver/request.rs) | Request validation enforces version, chain, expiry, public key validity, signature, and TOFU/pinning. | Make wallet approval and first-contact decisions more explicit and testable. |
| [crates/z00z_wallets/src/chain/scan_engine.rs](../../crates/z00z_wallets/src/chain/scan_engine.rs) | `RemoteScanEvidence` returns chunks, proof hints, and resume hints as advisory inputs only. | Turn opaque proof hints into typed DA/witness envelopes while keeping wallet-local authority. |
| [crates/z00z_wallets/src/tx/tx_wire.rs](../../crates/z00z_wallets/src/tx/tx_wire.rs) | `TxPackage`, input refs, output roles, public spend proof, and spend authorization are already separated. | Add package-phase transcripts and privacy lint outputs without changing settlement semantics. |
| [crates/z00z_wallets/src/tx/state_witness.rs](../../crates/z00z_wallets/src/tx/state_witness.rs) | `MemberWit` wraps canonical storage proof bytes and verifies inclusion under `SettlementStateRoot`. | Formalize wallet witness freshness and helper evidence around this seam. |

## 🧭 Ideological Fit Ranking

| Fit | Source group | Why it is close to Z00Z | What to take | What to avoid |
| --- | --- | --- | --- | --- |
| Very high | `UTT - Decentralized Ecash with Accountable Privacy`, `accountable_privacy_for_decentralized_anonymous_payments`, `PARScoin`, `PRCash`, `Privacy-preserving auditable token payments`, `Auditable, Anonymous Electronic Cash` | These papers share Z00Z's central tension: private payments with explicit accountability hooks. | Scoped disclosure receipts, policy-bound audit proofs, privacy budgets, double-spend accountability. | Always-on tracing, master deanonymization keys, regulator-first design. |
| Very high | `WabiSabi`, `Adoption and Actual Privacy of Decentralized CoinJoin`, `User-Perceived Privacy in Blockchain`, `Usability of Cryptocurrency Wallets` | They show that privacy fails in user behavior and wallet defaults even when cryptography is sound. | Privacy linting, post-spend merge warnings, request identity warnings, user-visible privacy state. | Copying a mixer/coordinator model as consensus infrastructure. |
| High | `Transferable E-cash - A Cleaner Model`, `Anonymous Transferable E-Cash`, `Fully Anonymous Transferable Ecash`, `Zef` | They are ideologically close to Z00Z's digital cash ambition and offline/local ownership theme. | Liability chains, one-time use rights, offline handoff manifests, low-latency client flows. | Claiming offline bearer settlement is final before checkpoint reconciliation. |
| High | `TariScript`, MimbleWimble formal and Grin transaction-file articles | They are close to Z00Z's existing Tari-backed crypto and confidential package model. | Bounded constraints, unilateral receive improvements, local package-construction transcript discipline. | General scripting VM, MimbleWimble cut-through as a replacement for Z00Z object settlement. |
| High | `Coded Merkle Tree - Solving Data Availability`, `MSMPT`, `Optimization of Merkle Tree Structures` | They map cleanly to checkpoint proof hints, light clients, HJMT locality, and witness availability. | DA sampling, typed proof-hint envelopes, subtree/page-local proof caches. | Replacing semantic `SettlementStateRoot` with backend-root or storage-layout commitments. |
| Medium | `PGC`, `Hiding Transaction Amounts and Balances in Bitcoin` | The primitives are compatible but mostly already represented by Z00Z commitments and range proofs. | Better proof aggregation UX, verifier docs, public spend theorem hardening. | Treating amount-hiding proofs as complete settlement proofs. |
| Medium | `Functional Encryption - Definitions and Challenges`, `Enabling Regulatory Compliance`, `UCoin`, `PCH-based privacy-preserving with reusability` | Useful for future compliance credentials and scoped views, but heavier than current implementation needs. | View keys with purpose limits, reusable compliance credentials, formal disclosure vocabulary. | Bringing in complex crypto before the policy registry and audit-proof interface exist. |
| Low | `SendingNetwork - Advancing the Future`, generic Verkle/PQ survey material | Some infrastructure themes fit, but the actionable Z00Z mapping is less direct today. | Keep as background for future network/PQ work. | Diverting active settlement work into broad infrastructure rewrites. |

## 🧾 Compliance-Profile Wallets Through RightLeaf

The strongest compliance idea from the article corpus is not a public trace mode. It is a wallet-selected compliance profile that can prove specific policy obedience while preserving ordinary Z00Z privacy outside the disclosed scope. `RightLeaf` is the right Z00Z primitive for that model because it is already a narrow, checkpointed settlement object for bounded rights rather than a public account row or a wallet-history transcript.

The correct claim is precise:

> Z00Z can support compliance-verifiable private wallets. Corporate or regulated users may opt into strict policy profiles and produce cryptographic evidence that selected actions followed those profiles, while unrelated transactions and wallet inventory remain private.

This is stronger than a generic view-key promise and safer than a regulator backdoor. A view key often risks exposing a broad wallet slice. A `RightLeaf` proof can instead be path-local: it proves a bounded right, policy, transition, or disclosure condition without revealing unrelated rights, assets, counterparties, or historical wallet behavior.

### ✅ Why This Fits Z00Z

Z00Z's base architecture separates public settlement evidence from wallet-local meaning. The public layer notarizes narrow roots, paths, package commitments, and checkpoint relations; it does not become a reusable public account ledger. A compliance-profile wallet should preserve that same shape.

`RightLeaf` fits because it carries compliance-relevant commitments without turning them into plaintext public state:

- `right_class` tells the verifier what kind of bounded right is being settled, such as data access, service entitlement, machine capability, validator mandate, or one-time use.
- `issuer_scope` and `provider_scope` bind the right to the relevant issuer or service boundary without making Z00Z core endorse that issuer.
- `holder_commitment`, `control_commitment`, and `beneficiary_commitment` bind authority and benefit to commitments rather than public identities.
- `payload_commitment` commits to the off-chain or encrypted business meaning without publishing the payload.
- `valid_from`, `valid_until`, `challenge_from`, and `challenge_until` make time and dispute windows explicit.
- `use_nonce` gives one-time or anti-replay rights a protocol-visible marker.
- `revocation_policy_id`, `transition_policy_id`, `challenge_policy_id`, `disclosure_policy_id`, and `retention_policy_id` bind the right to policy commitments that can later be verified by a wallet, auditor, counterparty, or corporate archive.

That field set matches the accountability papers well: prove the rule, prove the action was inside the rule, and reveal only the minimum evidence needed for the reviewer.

### ⚙️ How The Flow Should Work

A corporate client would not ask the base protocol to know its jurisdiction, customer identity, accounting system, or full business record. Instead, the corporate wallet would run a strict local profile and retain scoped evidence above the protocol line.

Recommended flow:

1. A policy profile is defined outside consensus, such as `corporate_eu_transfer_v1`, `enterprise_retention_7y_v1`, or `sanctions_screened_counterparty_v1`.
2. The profile's canonical bytes are hashed into one or more `RightLeaf` policy IDs.
3. The wallet refuses to create, transfer, consume, revoke, challenge, or disclose a right unless the action satisfies the selected profile.
4. The resulting `RightLeaf` transition is committed under the normal settlement path and root discipline.
5. The wallet stores an `Evidence Package` or `DisclosureReceipt` containing the selected fields, purpose, expiry, policy ID, retained document hash, and checkpoint anchor.
6. An auditor receives an `AuditViewRequest` response only for the approved scope and only after holder approval or wallet-policy approval.
7. The auditor verifies the proof against the relevant `RightLeaf`, policy ID, settlement path, root, checkpoint anchor, and retained evidence hash.

The important property is that the auditor verifies one bounded claim. They do not receive a universal key to the wallet, and they do not become a consensus authority.

### 🧾 Corporate Memo And Evidence Package Shape

For corporate clients, the encrypted transaction memo should be treated as a compact business reference, not as the complete compliance record. A 512-byte encrypted memo is enough for short structured fields such as payment purpose, invoice reference, counterparty reference, due date, internal cost center, or a hash pointer to retained evidence. It is also enough for wallet-readable context that helps finance, treasury, or procurement staff recognize why a transaction exists.

It is not enough for a full SWIFT MT/MX-style message. A complete corporate payment record may include addresses, banks, BIC/IBAN data, regulatory reporting fields, remittance information, sanctions-screening metadata, invoice details, tax fields, and longer prose. Trying to fit that entire record into 512 bytes would either truncate important data or pressure the wallet to leak too much business context directly in the transaction package.

The corporate-friendly Z00Z shape should therefore be:

```text
512B encrypted memo + hash/reference to EvidencePackage + optional DisclosureReceipt
```

This matches the privacy model. The chain and ordinary observers learn no extra corporate context. The corporate wallet retains the full `EvidencePackage` off-chain or in a controlled archive. A reviewer receives the full package only through scoped disclosure, with a `DisclosureReceipt` recording what was revealed, for which purpose, under which profile, and until which expiry. The memo is the private transaction note; the `EvidencePackage` is the durable compliance record.

### 🔍 What Can Be Proven

This model can support several practical proof families.

| Proof family | What the wallet can prove | What the proof should avoid revealing |
| --- | --- | --- |
| Policy membership proof | The right was created or used under an allowed policy profile. | Full policy text if the reviewer only needs the profile hash and registry entry. |
| Right-class proof | The object belongs to an allowed `RightClass`. | Other right classes held by the same wallet. |
| Holder/control proof | The action was authorized by the expected holder or control commitment. | The holder's real-world identity unless that identity is part of the requested disclosure. |
| Validity-window proof | The right was active at action time or properly expired afterward. | Other actions before or after the reviewed event. |
| One-time-use proof | A bounded right was consumed once and cannot be reused under the same nonce. | Unrelated nonces or wallet inventory. |
| Retention proof | The disclosed evidence is bound to a retention policy and expiry semantics. | Future transactions or unrelated retained documents. |
| Checkpoint-anchor proof | The evidence package refers to a real settlement boundary. | The underlying business record, unless explicitly disclosed. |

This is the compliance sweet spot for Z00Z: the corporation can show that it operated under a rule, while the public chain still sees only verifier-minimal settlement evidence.

### 👁️‍🗨️ What Remains Private

The compliance proof should be deliberately smaller than the wallet. By default, these remain private:

- unrelated assets and rights;
- wallet balance and full inventory;
- counterparties outside the requested scope;
- historical transactions not covered by the request;
- private business documents whose hashes are enough for the audit purpose;
- internal wallet selection logic;
- helper or remote-scan metadata unless it is part of the retained evidence package.

This distinction is essential. A compliance-profile wallet is not less private because it can disclose. It is more legally usable because it can disclose narrowly, with receipts, purpose limits, and retention boundaries.

### 🚫 What This Must Not Claim

The model should not be described as "full compliance for any jurisdiction" or "absolute anonymity." Those phrases overstate both law and privacy.

The defensible claim is narrower:

- Z00Z can make policy obedience cryptographically verifiable for a declared profile.
- Z00Z can let corporate users prove selected compliance facts without revealing their whole wallet.
- Z00Z can preserve pseudonymous honest-path behavior for ordinary settlement observers.
- Z00Z cannot make every jurisdiction accept a policy profile automatically.
- Z00Z cannot guarantee anonymity against all wallet, network, service, ingress, egress, or voluntary-disclosure leaks.
- `RightLeaf` cannot replace legal counsel, issuer duties, regulated-service obligations, or corporate recordkeeping.

This boundary is not a weakness. It is what keeps the idea credible.

### 🧪 Evidence Already Present In The Repository

The current code and docs already support the direction, but not the full production workflow.

| Evidence | What it proves today | Remaining gap |
| --- | --- | --- |
| [crates/z00z_storage/src/settlement/record.rs](../../crates/z00z_storage/src/settlement/record.rs) | `RightLeaf` is a real settlement object with commitments, windows, nonce, and policy ID fields. | No full policy registry or disclosure-proof API yet. |
| [crates/z00z_storage/src/settlement/query.rs](../../crates/z00z_storage/src/settlement/query.rs) | Right actions call `validate_action()` for create, transfer, consume, revoke, expire, and challenge paths. | Compliance profiles are not yet expressed as end-user wallet modes. |
| [crates/z00z_core/src/rights/config.rs](../../crates/z00z_core/src/rights/config.rs) | Rights config carries policy IDs and rejects fee/processing semantics inside `RightLeaf`. | Need profile-level policy grammar and verifier-facing schemas. |
| [docs/tech-papers/done/Z00Z-HJMT-Design.md](done/Z00Z-HJMT-Design.md) | `RightLeaf` is documented as a bounded evidence-bearing rights object that improves selective disclosure. | The same doc warns it does not create legal sufficiency by itself. |
| [docs/Z00Z-Legal-Architecture-Whitepaper.md](../Z00Z-Legal-Architecture-Whitepaper.md) | The corpus already defines compliance-profile wallets, evidence packages, and corporate archives above the base protocol. | Needs concrete formats and wallet enforcement rules. |
| [docs/Z00Z-Privacy-Threat-Model-Whitepaper.md](../Z00Z-Privacy-Threat-Model-Whitepaper.md) | Privacy is layered and selective disclosure is compatible with private movement. | Need avoid marketing language that promises universal anonymity. |
| [docs/Z00Z-Multi-DA-and-Checkpoint-Architecture.md](Z00Z-Multi-DA-and-Checkpoint-Architecture.md) | Audit receipts and disclosure packages belong above consensus and can anchor to checkpoints. | Need verifier libraries and retained evidence package format. |

### 🛠️ Implementation Implications

To turn the concept into a usable Z00Z feature, implement the compliance-profile wallet as an overlay on lanes 2 and 3 rather than as a separate protocol personality.

Required pieces:

1. `ComplianceProfileId`: stable identifier for a jurisdiction, enterprise, or counterparty-selected policy bundle.
2. `RightPolicyKind`: bounded enum for transition, revocation, challenge, disclosure, retention, cap, category, and one-time-use rules.
3. `PolicyRegistry`: canonical mapping from policy ID bytes to deterministic policy bytes and human-readable profile metadata.
4. `WalletPolicyMode`: local wallet setting that can require specific policy IDs before package creation or right transition.
5. `EvidencePackage`: retained object that binds policy ID, action, field set, purpose, expiry, document hash, and checkpoint anchor.
6. `DisclosureReceipt`: holder-controlled proof that a specific disclosure happened under a specific profile and retention policy.
7. `AuditViewProof`: bounded proof over commitments, paths, roots, and policy IDs, not a wallet-history export.
8. Tests proving unrelated `RightLeaf` and `AssetLeaf` inventory is not revealed by a disclosure proof.

The first implementation should be conservative: prove policy-profile obedience for one or two right classes, such as `DataAccess` and `ServiceEntitlement`, before attempting broad corporate finance language.

### ⭐ Strategic Takeaway

`RightLeaf` can make Z00Z unusually strong for enterprise privacy. A corporate client can opt into a strict policy profile and later prove "we followed these rules" without turning every transaction into public compliance telemetry. That is exactly the middle path the article corpus points toward: privacy by default, disclosure by rule, receipts by consent or explicit wallet policy, and final settlement through the normal checkpoint theorem.

## ⚙️ Implementation Lanes

### 🔒 Lane 1: Wallet Privacy Linting

This is the most immediately useful implementation. The CoinJoin/WabiSabi and wallet-usability papers show that privacy loss usually comes from transaction construction, merge behavior, stale requests, or confusing wallet UX rather than from primitive failure.

Implement a wallet-local privacy linter that runs before a regular `TxPackage` is emitted and after receive scans import assets.

| Check | Existing hook | Desired outcome |
| --- | --- | --- |
| Input merge risk | `TxInputWire`, spend selection, owned asset rows | Warn when inputs from unrelated receive contexts are merged. |
| Change linkability | `TxOutRole::Change` and `TxOutRole::Recipient` | Mark change outputs and warn when change is likely linkable. |
| Request freshness | `PaymentRequest::check_validity()` | Reject expired requests and warn for near-expiry requests before payment. |
| First-contact risk | `ValidationOutcome::RequiresUserConfirmation` | Present a clear receiver identity confirmation gate. |
| Identity drift | `ValidationOutcome::IdentityMismatch` | Fail closed unless the user explicitly rotates the receiver identity. |
| Scan provenance | `ScanRangeStat`, `RemoteScanEvidence` | Show whether an output came from local scan or helper-fetched evidence. |

Deliverable shape:

- Add a `PrivacyReport` DTO under the wallet transaction or receiver layer.
- Add a `PrivacyWarning` enum with stable machine-readable variants.
- Keep all scores local to the wallet. Do not publish privacy telemetry by default.
- Add tests for input merge, change output, stale request, first-contact, identity mismatch, and remote-evidence provenance.

### 👁️‍🗨️ Lane 2: Scoped Accountable Privacy

The accountable privacy papers are a strong ideological fit if Z00Z treats accountability as explicit, scoped, and user-visible. The current `RightLeaf` already has `disclosure_policy_id` and `retention_policy_id`, so the next step should be a policy registry and proof envelope, not new tracing powers.

Implement a narrow disclosure model:

- `DisclosurePolicyId`: typed wrapper over the current policy ID bytes.
- `RetentionPolicyId`: typed wrapper over the current policy ID bytes.
- `DisclosureReceipt`: evidence that holder-controlled disclosure occurred for a specific field set, purpose, and expiry.
- `AuditViewRequest`: request object from a policy-recognized requester; it is not authority by itself and should require holder or wallet policy approval before any disclosure response.
- `AuditViewProof`: bounded proof over commitments, not a full wallet-history reveal.

Good first policy families:

| Policy | What it proves | What stays hidden |
| --- | --- | --- |
| Amount-limit proof | A transfer is below or within a configured policy cap. | Full wallet balance and unrelated outputs. |
| Right-class proof | A `RightLeaf` belongs to an allowed `RightClass`. | Holder identity and unrelated rights. |
| Expiry proof | A right or request was valid at action time. | Other wallet activity. |
| Retention proof | A disclosed view expires and has limited retention semantics. | Future or unrelated transactions. |

Non-goal: do not introduce a universal disclosure key or hidden surveillance authority.

### 🧾 Lane 3: RightLeaf Policy Runtime

`RightLeaf` is the best current bridge between article ideas and Z00Z code. It already models non-coin settlement objects with holder/control commitments, policy IDs, `use_nonce`, validity windows, and challenge/revocation paths.

Next implementation should make these fields operational:

- define a small policy registry for known transition, revocation, challenge, disclosure, and retention policies;
- add fixtures for `RightClass::DataAccess`, `ServiceEntitlement`, `MachineCapability`, and `OneTimeUse`;
- add tests for `RightAction::Transfer`, `Consume`, `Revoke`, and `Challenge` using non-zero policy IDs;
- expose policy validation through the storage facade, not raw HJMT internals;
- document that policy IDs are commitments to rules, not mutable off-chain metadata.

This would turn PARScoin/PRCash/UTT-style accountable rights into Z00Z-native settlement rights without pretending the policy registry already exists.

### 📡 Lane 4: Typed Remote Scan Evidence and DA Hints

The data-availability and light-client papers fit the current remote scan seam almost exactly. `RemoteScanEvidence` is intentionally advisory. That is correct. The upgrade is to make the advisory evidence typed and verifiable.

Replace opaque proof hints with a typed envelope such as:

```text
RemoteScanProofHintV1
  checkpoint_height
  chunk_hash
  checkpoint_id
  prev_root
  post_root
  proof_kind
  proof_bytes
  worker_provenance_attestation
```

Rules:

- proof hints must bind to returned chunks;
- chunks must remain strictly increasing and contiguous;
- helper evidence must not mutate `ScanStatePayload` directly;
- wallet-local scanning remains the only ownership detector;
- final spend remains checkpoint-theorem-bound.

This is the clean place to import Coded Merkle Tree and compact delta log ideas: make missing witnesses detectable, retryable, and attributable without trusting the worker.

### 🧮 Lane 5: Storage Proof Locality and Witness Caches

The MSMPT and Merkle subtree articles are implementation-infrastructure gold. They should not change protocol semantics, but they can improve proof generation and scan performance.

Implement as storage-internal improvements:

- measure HJMT proof generation hot paths;
- add page/subtree locality counters for `ProofBlob` generation;
- cache safe intermediate proof material keyed by semantic `SettlementPath` plus root generation;
- preserve `SettlementStateRoot` as the public semantic commitment;
- keep `backend_root` proof-local and private.

Useful tests:

- proof cache must invalidate on root generation change;
- `TxDigest` must never become a `CheckRoot`;
- proof-local `backend_root` must not leak as public API authority.

### 🧩 Lane 6: Non-Interactive Package Build Transcripts

The Grin transaction-file articles are useful only as a cautionary source on phase discipline. Z00Z must not adopt Grin's sender-receiver finalization round, synchronous receiver contribution, or partial-signature exchange. In Z00Z, receiver-side material should already exist as an asynchronous signed payment request, receiver card, or other approved receiver descriptor before the sender builds a package.

Add optional local package-build transcripts for debugging and auditability:

- sender input-selection hash;
- referenced payment request or receiver-card hash, when used;
- output binding hash;
- spend-proof statement hash;
- final package digest;
- construction warnings from the wallet privacy report;
- explicit absent/private fields.

This is not an exchange protocol and not a public settlement artifact. It must not require the receiver to be online, add receiver partial signatures, or introduce a synchronous transaction-construction handshake. The public settlement path still verifies `TxPackage` plus checkpoint theorem.

### ⚙️ Lane 7: Bounded Script-Like Rights

TariScript is close to Z00Z because it tries to add expressive constraints without abandoning confidential transaction structure. Z00Z should use this as inspiration for right policies, not as a reason to embed a general VM.

Good bounded constraints:

- spend only after a validity window;
- consume only once via `use_nonce`;
- transfer only if transition policy matches;
- challenge only during challenge window;
- disclose only under an explicit disclosure policy;
- pay only to an approved receiver descriptor or request-bound receiver identity.

Implementation direction:

- define a small `RightPolicyKind` enum;
- hash canonical policy bytes into existing policy ID fields;
- verify policy commitments at action time;
- keep the evaluator deterministic, bounded, and testable.

Avoid:

- general scripting language;
- unbounded loops;
- dynamic network calls during validation;
- public API exposure of dependency-specific script types.

### 💸 Lane 8: Offline and Transferable Cash Research

The transferable e-cash papers are highly aligned with Z00Z's digital-cash story, but they should stay research-gated until double-spend accountability and checkpoint reconciliation are fully specified.

Near-term safe subset:

- one-time offline claim manifest;
- signed payment request plus right/asset handoff record;
- local liability chain attached to a wallet object;
- mandatory online reconciliation before final settlement language;
- double-spend or replay marker based on `use_nonce` or nullifier semantics.

Do not call this final offline bearer settlement yet. The honest Z00Z phrasing is: offline handoff can be prepared and locally accepted, but public settlement finality remains checkpoint-bound.

## ✅ Recommended Phase Order

### ✅ Phase A: Wallet Safety and Privacy UX

- Add wallet-local `PrivacyReport` and `PrivacyWarning`.
- Gate payment execution on request validity, identity pin state, and merge/change risk.
- Add tests using existing request and tx package types.

Why first: it uses live wallet surfaces and directly addresses the largest privacy failures described by the usability and CoinJoin papers.

### ✅ Phase B: Right Policy Registry

- Add typed policy ID wrappers.
- Add registry entries for transition, revocation, challenge, disclosure, and retention.
- Add `RightLeaf` action tests using non-zero policy IDs.

Why second: it activates already-present storage fields and gives accountable privacy a Z00Z-native home.

### ✅ Phase C: Typed Remote Proof Hints

- Convert opaque `RemoteScanProofHint` into a versioned proof-hint envelope.
- Bind proof hints to checkpoint height, chunk hash, root, and checkpoint ID.
- Keep worker evidence advisory.

Why third: it improves light-client and remote scan safety without making infrastructure trusted.

### ✅ Phase D: Disclosure Receipts

- Add `AuditViewRequest`, `DisclosureReceipt`, and minimal `AuditViewProof` skeleton.
- Bind every receipt to purpose, field set, expiry, policy ID, and retention ID.

Why fourth: it builds on the policy registry rather than creating standalone compliance logic.

### ✅ Phase E: Storage Locality Benchmarks

- Benchmark proof generation.
- Add proof-local cache and subtree/page metrics if measurements justify it.

Why fifth: the optimization papers are compelling, but performance work should be measurement-led.

### ✅ Phase F: Offline Transfer Research Spike

- Specify offline handoff objects and double-spend accountability.
- Prove how they reconcile with checkpoint theorem semantics.
- Keep this behind explicit research status until validated.

Why sixth: it has high strategic value but the largest semantic risk.

## 🚫 Explicit Non-Goals

- Do not add a master tracing key.
- Do not make regulators, auditors, helpers, or coordinators consensus authorities.
- Do not copy CoinJoin as a core transaction model.
- Do not treat range proofs as final settlement closure.
- Do not conflate `TxDigest`, `CheckRoot`, `SettlementStateRoot`, or proof-local `backend_root`.
- Do not expose raw HJMT/JMT internals in public wallet or rollup APIs.
- Do not claim offline bearer finality until online checkpoint reconciliation and double-spend accountability are complete.
- Do not modify `crates/z00z_crypto/tari` vendor code.

## 📌 Source Coverage Map

The following extracted article groups informed this synthesis.

| Group | Titles reviewed | Main extraction |
| --- | --- | --- |
| Accountable private payments | `UTT - Decentralized Ecash with Accountable Privacy`, `accountable_privacy_for_decentralized_anonymous_payments`, `PARScoin`, `PRCash`, `Privacy-preserving auditable token payments`, `Auditable, Anonymous Electronic Cash`, `Enabling Regulatory Compliance`, `UCoin` | Use scoped accountability and policy proofs, not global tracing. |
| Transferable e-cash | `Transferable E-cash - A Cleaner Model`, `Anonymous Transferable E-Cash`, `Fully Anonymous Transferable Ecash`, `Zef` | Offline handoff is strategically important but must stay reconciliation-bound for now. |
| Wallet privacy and usability | `WabiSabi`, `Adoption and Actual Privacy of Decentralized CoinJoin`, `User-Perceived Privacy in Blockchain`, `Usability of Cryptocurrency Wallets` | Privacy must be enforced and explained at wallet decision points. |
| Confidential transactions and MW | `Hiding Transaction Amounts and Balances in Bitcoin`, `PGC`, MimbleWimble formal papers, Grin transaction internals, `TariScript` | Preserve confidential proofs but add package transcripts and bounded right constraints. |
| Data availability and storage | `Coded Merkle Tree - Solving Data Availability`, `MSMPT`, `Optimization of Merkle Tree Structures`, Verkle/PQ survey material | Improve witness availability and proof locality without changing semantic roots. |
| Advanced crypto background | `Functional Encryption - Definitions and Challenges`, `PCH-based privacy-preserving with reusability` | Useful for future scoped disclosure and reusable credentials after policy APIs exist. |

## ⭐ Bottom Line

The best article-driven Z00Z upgrade is a privacy-and-accountability layer that is explicit, typed, and local-first:

1. wallet decides safely before building packages;
2. helpers provide evidence but never become authorities;
3. rights carry policy commitments;
4. disclosure is scoped, receipted, and expiring;
5. final public settlement remains checkpoint-theorem-bound.

That direction absorbs the strongest ideas from the corpus while staying faithful to Z00Z's current architecture.
