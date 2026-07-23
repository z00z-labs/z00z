//! Backend-neutral recursive proof persistence orchestration.

use std::{
    fs::File,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use fs2::FileExt;
use z00z_crypto::sha256_256;
use z00z_utils::io::{SecureDir, Write};
use z00z_utils::time::Instant;

use super::{
    canonical_transition::CanonicalCheckpointTransitionV2,
    nova::{
        resolve_cached_verifier_v2, NovaChainTransitionV2, NovaContinuousSessionV2, NovaRunGuardV2,
        NovaVerifierCacheV2,
    },
    receipt::{recursive_receipt_digest, CryptographicVerificationReceiptV2, PreparedReceiptV2},
    recursive_circuit::RecursiveCircuitProfileV2,
    recursive_context::RecursiveAuthoritySnapshotV2,
    recursive_measurement::{
        NovaCadenceActionV2, NovaCadenceRequestV2, NovaCompressionAuthorityV2,
        NovaCompressionPolicyV2,
    },
    recursive_recovery::{NovaRecoveryStoreMetricsV2, NovaRecoveryStoreV2},
    recursive_statement::RecursiveFinalizedIvcStateV2,
    sidecar::{
        check_shadow_sidecar_binding, recursive_sidecar_digest, RecursiveCheckpointSidecarCodecV2,
        RecursiveCheckpointSidecarV2,
    },
};
use crate::{
    checkpoint::{CheckpointId, CheckpointStore},
    settlement::{SettlementExecHandoff, SettlementStore},
    snapshot::PrepSnapshotStore,
    CheckpointError,
};

const ENVELOPE_READ_CAP_V2: u64 = 512 * 1024 + 32;
const SIDECAR_READ_CAP_V2: u64 = 64 * 1024 + 32;
const LIVE_GATE_DOMAIN_V2: &str = "z00z.storage.checkpoint.live_gate.v2";
const LIVE_GATE_TRACE_LABEL_V2: &str = "gate_trace";
const LIVE_GATE_CONTEXT_LABEL_V2: &str = "gate_context";
const CHECKPOINT_CLAIM_LABEL_V2: &str = "checkpoint_claim";
// Authority generation 2 permits one complete prover run to consume up to one
// hour. Its separate 5-second cancellation budget limits stop responsiveness;
// it is not the wall-clock lifetime of a valid proof attempt.
const LIVE_RUN_MAX_DURATION_V2: Duration = Duration::from_secs(3_600);
const MAX_VERIFIER_ATTEMPTS_V2: u64 = 1_048_576;
const VERIFIER_ATTEMPTS_PER_RECEIPT_V2: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum LiveGateIdV2 {
    AuthorityResolved = 1,
    FamilyCapSelected = 2,
    OuterDecodeBounded = 3,
    InnerDecodeBounded = 4,
    CanonicalCurveValid = 5,
    BundleMatched = 6,
    BackendVerified = 7,
    FinalStateLimbsMatched = 8,
    StatementLinkPredecessorMatched = 9,
    StorageEndpointReloaded = 10,
    PrewriteComplete = 11,
    AtomicWrite = 12,
    ByteReload = 13,
    PostwriteBackendVerified = 14,
    PostwriteEndpointMatched = 15,
    ReceiptIssued = 16,
}

const SUCCESSFUL_GATE_SEQUENCE_V2: [LiveGateIdV2; 16] = [
    LiveGateIdV2::AuthorityResolved,
    LiveGateIdV2::FamilyCapSelected,
    LiveGateIdV2::OuterDecodeBounded,
    LiveGateIdV2::InnerDecodeBounded,
    LiveGateIdV2::CanonicalCurveValid,
    LiveGateIdV2::BundleMatched,
    LiveGateIdV2::BackendVerified,
    LiveGateIdV2::FinalStateLimbsMatched,
    LiveGateIdV2::StatementLinkPredecessorMatched,
    LiveGateIdV2::StorageEndpointReloaded,
    LiveGateIdV2::PrewriteComplete,
    LiveGateIdV2::AtomicWrite,
    LiveGateIdV2::ByteReload,
    LiveGateIdV2::PostwriteBackendVerified,
    LiveGateIdV2::PostwriteEndpointMatched,
    LiveGateIdV2::ReceiptIssued,
];

/// Fixed-cap private trace. Callers cannot supply, reorder, or deserialize it.
struct LiveGateTraceV2 {
    context_digest: Option<[u8; 32]>,
    ids: [u8; SUCCESSFUL_GATE_SEQUENCE_V2.len()],
    len: usize,
}

impl LiveGateTraceV2 {
    fn start() -> Self {
        Self {
            context_digest: None,
            ids: [0; SUCCESSFUL_GATE_SEQUENCE_V2.len()],
            len: 0,
        }
    }

    #[cfg(test)]
    fn push(mut self, gate: LiveGateIdV2) -> Result<Self, CheckpointError> {
        if SUCCESSFUL_GATE_SEQUENCE_V2.get(self.len).copied() != Some(gate) {
            return Err(CheckpointError::Invariant);
        }
        self.ids[self.len] = gate as u8;
        self.len += 1;
        Ok(self)
    }

    fn digest(&self) -> Result<[u8; 32], CheckpointError> {
        let context_digest = self.context_digest.ok_or(CheckpointError::Invariant)?;
        Ok(gate_trace_digest(context_digest, &self.ids[..self.len]))
    }

    #[cfg(test)]
    fn ids(&self) -> &[u8] {
        &self.ids[..self.len]
    }
}

/// A failed transition owns the exact successful prefix and one terminal
/// reason. No later gate can be appended because the trace has been consumed.
struct LiveGateFailureV2 {
    trace: LiveGateTraceV2,
    terminal: CheckpointError,
}

impl LiveGateFailureV2 {
    fn new(trace: LiveGateTraceV2, terminal: CheckpointError) -> Self {
        Self { trace, terminal }
    }

    fn into_error(self) -> CheckpointError {
        let Self { trace, terminal } = self;
        let valid_prefix = trace.len <= SUCCESSFUL_GATE_SEQUENCE_V2.len()
            && trace.ids[..trace.len]
                .iter()
                .copied()
                .eq(SUCCESSFUL_GATE_SEQUENCE_V2[..trace.len]
                    .iter()
                    .map(|gate| *gate as u8));
        if valid_prefix {
            terminal
        } else {
            CheckpointError::Invariant
        }
    }

    #[cfg(test)]
    fn ids(&self) -> &[u8] {
        self.trace.ids()
    }

    #[cfg(test)]
    fn terminal(&self) -> &CheckpointError {
        &self.terminal
    }
}

/// Linear private capability for one exact acceptance gate. Neither this
/// wrapper nor any marker implements Clone, Copy, Serialize, or Default.
struct LiveGateStageV2<S> {
    trace: LiveGateTraceV2,
    marker: PhantomData<S>,
}

struct GateStartV2;

impl LiveGateStageV2<GateStartV2> {
    fn start() -> Self {
        Self {
            trace: LiveGateTraceV2::start(),
            marker: PhantomData,
        }
    }
}

impl<S> LiveGateStageV2<S> {
    fn retain<T>(self, result: Result<T, CheckpointError>) -> Result<(Self, T), LiveGateFailureV2> {
        match result {
            Ok(value) => Ok((self, value)),
            Err(terminal) => Err(LiveGateFailureV2::new(self.trace, terminal)),
        }
    }

    fn bind_context(self, context_digest: [u8; 32]) -> Result<Self, LiveGateFailureV2> {
        if context_digest == [0; 32] || self.trace.context_digest.is_some() {
            return Err(LiveGateFailureV2::new(
                self.trace,
                CheckpointError::Invariant,
            ));
        }
        let mut trace = self.trace;
        trace.context_digest = Some(context_digest);
        Ok(Self {
            trace,
            marker: PhantomData,
        })
    }
}

macro_rules! live_gate_edge {
    ($from:ident, $method:ident, $gate:ident, $to:ident) => {
        struct $to;

        impl LiveGateStageV2<$from> {
            fn $method<T>(
                self,
                result: Result<T, CheckpointError>,
            ) -> Result<(LiveGateStageV2<$to>, T), LiveGateFailureV2> {
                match result {
                    Ok(value) => {
                        let mut trace = self.trace;
                        if SUCCESSFUL_GATE_SEQUENCE_V2.get(trace.len).copied()
                            != Some(LiveGateIdV2::$gate)
                        {
                            return Err(LiveGateFailureV2::new(trace, CheckpointError::Invariant));
                        }
                        trace.ids[trace.len] = LiveGateIdV2::$gate as u8;
                        trace.len += 1;
                        Ok((
                            LiveGateStageV2 {
                                trace,
                                marker: PhantomData,
                            },
                            value,
                        ))
                    }
                    Err(terminal) => Err(LiveGateFailureV2::new(self.trace, terminal)),
                }
            }
        }
    };
}

live_gate_edge!(
    GateStartV2,
    authority_resolved,
    AuthorityResolved,
    AuthorityReadyV2
);
live_gate_edge!(
    AuthorityReadyV2,
    family_selected,
    FamilyCapSelected,
    FamilyReadyV2
);
live_gate_edge!(
    FamilyReadyV2,
    outer_bounded,
    OuterDecodeBounded,
    OuterReadyV2
);
live_gate_edge!(
    OuterReadyV2,
    inner_bounded,
    InnerDecodeBounded,
    InnerReadyV2
);
live_gate_edge!(InnerReadyV2, curve_valid, CanonicalCurveValid, CurveReadyV2);
live_gate_edge!(CurveReadyV2, bundle_matched, BundleMatched, BundleReadyV2);
live_gate_edge!(
    BundleReadyV2,
    backend_verified,
    BackendVerified,
    BackendReadyV2
);
live_gate_edge!(
    BackendReadyV2,
    limbs_matched,
    FinalStateLimbsMatched,
    LimbsReadyV2
);
live_gate_edge!(
    LimbsReadyV2,
    bindings_matched,
    StatementLinkPredecessorMatched,
    BindingsReadyV2
);
live_gate_edge!(
    BindingsReadyV2,
    endpoint_reloaded,
    StorageEndpointReloaded,
    EndpointReadyV2
);
live_gate_edge!(
    EndpointReadyV2,
    prewrite_complete,
    PrewriteComplete,
    PrewriteReadyV2
);
live_gate_edge!(PrewriteReadyV2, atomic_write, AtomicWrite, WrittenV2);
live_gate_edge!(WrittenV2, bytes_reloaded, ByteReload, ReloadedV2);
live_gate_edge!(
    ReloadedV2,
    post_backend_verified,
    PostwriteBackendVerified,
    PostBackendReadyV2
);
live_gate_edge!(
    PostBackendReadyV2,
    post_endpoint_matched,
    PostwriteEndpointMatched,
    PostEndpointReadyV2
);
struct ReceiptIssuedV2;

impl LiveGateStageV2<PostEndpointReadyV2> {
    /// The caller validates this exact stage before any receipt preparation.
    /// The type is private and can only be reached through all prior edges.
    fn issue_receipt(mut self) -> Result<LiveGateStageV2<ReceiptIssuedV2>, CheckpointError> {
        if SUCCESSFUL_GATE_SEQUENCE_V2.get(self.trace.len).copied()
            != Some(LiveGateIdV2::ReceiptIssued)
            || self.trace.len >= self.trace.ids.len()
        {
            return Err(CheckpointError::Invariant);
        }
        self.trace.ids[self.trace.len] = LiveGateIdV2::ReceiptIssued as u8;
        self.trace.len += 1;
        Ok(LiveGateStageV2 {
            trace: self.trace,
            marker: PhantomData,
        })
    }
}

fn gate_trace_digest(context_digest: [u8; 32], ids: &[u8]) -> [u8; 32] {
    let ids_len = u64::try_from(ids.len())
        .expect("the fixed live-gate trace length always fits u64")
        .to_le_bytes();
    sha256_256(
        LIVE_GATE_DOMAIN_V2,
        LIVE_GATE_TRACE_LABEL_V2,
        &[&ids_len, &context_digest, ids],
    )
}

fn live_gate_context_digest(
    storage_generation: u64,
    envelope_digest: [u8; 32],
    bindings: super::nova::NovaVerificationBindingsV2,
) -> [u8; 32] {
    let authority_generation = bindings.authority_generation.to_le_bytes();
    let storage_generation = storage_generation.to_le_bytes();
    let height = bindings.height.to_le_bytes();
    let steps = bindings.steps.to_le_bytes();
    let final_state_limbs = bindings.final_state_limbs.to_le_bytes();
    let predecessor_tag = [u8::from(bindings.predecessor.is_some())];
    let predecessor = bindings.predecessor.unwrap_or([0; 32]);
    sha256_256(
        LIVE_GATE_DOMAIN_V2,
        LIVE_GATE_CONTEXT_LABEL_V2,
        &[
            &authority_generation,
            &storage_generation,
            &height,
            &steps,
            &final_state_limbs,
            &bindings.config_digest,
            &bindings.bundle_digest,
            &bindings.public_input_digest,
            &bindings.initial_state_digest,
            &bindings.final_state_digest,
            &bindings.successor_finalized_state_digest,
            &bindings.statement_digest,
            &bindings.checkpoint_link_digest,
            &bindings.prior_output_root,
            &bindings.output_root,
            &bindings.trace_digest,
            &bindings.checkpoint_id,
            &bindings.backend_revision_result_digest,
            &envelope_digest,
            &predecessor_tag,
            &predecessor,
        ],
    )
}

fn checkpoint_claim_bytes(
    storage_generation: u64,
    envelope_digest: [u8; 32],
    sidecar_digest: [u8; 32],
    bindings: super::nova::NovaVerificationBindingsV2,
) -> Vec<u8> {
    let mut body = Vec::with_capacity(2 + 4 * 8 + 9 * 32);
    body.extend_from_slice(&2_u16.to_le_bytes());
    body.extend_from_slice(&storage_generation.to_le_bytes());
    body.extend_from_slice(&bindings.authority_generation.to_le_bytes());
    body.extend_from_slice(&bindings.height.to_le_bytes());
    body.extend_from_slice(&bindings.steps.to_le_bytes());
    for digest in [
        bindings.checkpoint_id,
        bindings.config_digest,
        bindings.bundle_digest,
        bindings.public_input_digest,
        bindings.successor_finalized_state_digest,
        bindings.statement_digest,
        bindings.checkpoint_link_digest,
        envelope_digest,
        sidecar_digest,
    ] {
        body.extend_from_slice(&digest);
    }
    let checksum = sha256_256(LIVE_GATE_DOMAIN_V2, CHECKPOINT_CLAIM_LABEL_V2, &[&body]);
    body.extend_from_slice(&checksum);
    body
}

fn is_checkpoint_claim_canonical(bytes: &[u8]) -> bool {
    const BODY_BYTES: usize = 2 + 4 * 8 + 9 * 32;
    if bytes.len() != BODY_BYTES + 32 {
        return false;
    }
    let (body, checksum) = bytes.split_at(BODY_BYTES);
    checksum == sha256_256(LIVE_GATE_DOMAIN_V2, CHECKPOINT_CLAIM_LABEL_V2, &[body]).as_slice()
}

/// One-shot capability created only after exact postwrite backend and endpoint
/// verification. Receipt code can consume it but cannot construct or replay it.
pub(super) struct PostwriteVerifiedV2 {
    stage: LiveGateStageV2<PostEndpointReadyV2>,
    storage_generation: u64,
    envelope_digest: [u8; 32],
    sidecar_digest: [u8; 32],
    bindings: super::nova::NovaVerificationBindingsV2,
}

/// Private capability produced only by consuming the final receipt gate.
pub(super) struct ReceiptIssuedPartsV2 {
    storage_generation: u64,
    envelope_digest: [u8; 32],
    sidecar_digest: [u8; 32],
    bindings: super::nova::NovaVerificationBindingsV2,
}

/// Private capability created after persisted evidence is reloaded and reverified.
pub(super) struct ReloadedEvidenceV2 {
    marker: PhantomData<()>,
}

struct ReceiptReadyToIssueV2 {
    postwrite: PostwriteVerifiedV2,
    prepared: PreparedReceiptV2,
}

impl ReloadedEvidenceV2 {
    fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl ReceiptIssuedPartsV2 {
    pub(super) fn into_parts(
        self,
    ) -> (
        u64,
        [u8; 32],
        [u8; 32],
        super::nova::NovaVerificationBindingsV2,
    ) {
        (
            self.storage_generation,
            self.envelope_digest,
            self.sidecar_digest,
            self.bindings,
        )
    }
}

impl PostwriteVerifiedV2 {
    fn new(
        stage: LiveGateStageV2<PostEndpointReadyV2>,
        storage_generation: u64,
        envelope_digest: [u8; 32],
        sidecar_digest: [u8; 32],
        bindings: super::nova::NovaVerificationBindingsV2,
    ) -> Result<Self, LiveGateFailureV2> {
        // HJMT generation zero is the canonical empty/genesis store. It is an
        // exact authority-bound value, not an "unset" sentinel.
        if envelope_digest == [0; 32] || sidecar_digest == [0; 32] {
            return Err(LiveGateFailureV2::new(
                stage.trace,
                CheckpointError::Invariant,
            ));
        }
        Ok(Self {
            stage,
            storage_generation,
            envelope_digest,
            sidecar_digest,
            bindings,
        })
    }

    pub(super) fn issue_receipt(self) -> Result<ReceiptIssuedPartsV2, CheckpointError> {
        let stage = self.stage.issue_receipt()?;
        let receipt_trace_digest = stage.trace.digest()?;
        let mut bindings = self.bindings;
        bindings.gate_trace_digest = receipt_trace_digest;
        Ok(ReceiptIssuedPartsV2 {
            storage_generation: self.storage_generation,
            envelope_digest: self.envelope_digest,
            sidecar_digest: self.sidecar_digest,
            bindings,
        })
    }

    fn prepare_receipt(self) -> Result<ReceiptReadyToIssueV2, CheckpointError> {
        let prepared = CryptographicVerificationReceiptV2::prepare(
            self.storage_generation,
            self.envelope_digest,
            self.sidecar_digest,
            self.bindings,
        )?;
        Ok(ReceiptReadyToIssueV2 {
            postwrite: self,
            prepared,
        })
    }

    #[cfg(test)]
    fn gate_ids(&self) -> &[u8] {
        self.stage.trace.ids()
    }
}

impl ReceiptReadyToIssueV2 {
    fn issue(self) -> Result<CryptographicVerificationReceiptV2, CheckpointError> {
        let issued = self.postwrite.issue_receipt()?;
        Ok(self.prepared.issue(issued, ReloadedEvidenceV2::new()))
    }
}

/// Digest-bound evidence returned only after exact persistence reload and
/// unchanged-verifier acceptance.
#[derive(Debug)]
pub struct RecursiveCheckpointEvidenceV2 {
    pub sidecar: RecursiveCheckpointSidecarV2,
    pub receipt: CryptographicVerificationReceiptV2,
    pub envelope_digest: [u8; 32],
    pub sidecar_digest: [u8; 32],
    pub receipt_digest: [u8; 32],
    pub successor: RecursiveFinalizedIvcStateV2,
    pub verifier_attempts: u64,
}

/// A real, verified local accumulator recovery image committed under the hot-set cap.
#[derive(Debug)]
pub struct RecursiveCheckpointRecoveryV2 {
    pub snapshot_digest: [u8; 32],
    pub snapshot_height: u64,
    pub snapshot_bytes: u64,
    pub successor: RecursiveFinalizedIvcStateV2,
    pub metrics: NovaRecoveryStoreMetricsV2,
}

/// Explicit caller intent for the sole fold/evidence ingress. Folding never
/// implies compression, persistence, publication, or receipt issuance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecursiveEvidenceRequestV2 {
    FoldOnly,
    Snapshot {
        authority: NovaCompressionAuthorityV2,
        cadence: NovaCadenceRequestV2,
    },
    RecoverySnapshot {
        authority: NovaCompressionAuthorityV2,
    },
}

/// One canonical production result: either the advanced accumulator endpoint
/// or fully reloaded and reverified snapshot evidence.
#[derive(Debug)]
pub enum RecursiveEvidenceOutcomeV2 {
    Folded(Box<RecursiveFinalizedIvcStateV2>),
    Snapshot(Box<RecursiveCheckpointEvidenceV2>),
    Recovery(Box<RecursiveCheckpointRecoveryV2>),
}

/// Cooperative cancellation for one production evidence attempt. It cannot
/// extend the authority-owned deadline or any resource limit.
#[derive(Clone, Default)]
pub struct RecursiveEvidenceCancellationV2 {
    cancelled: Arc<AtomicBool>,
}

impl RecursiveEvidenceCancellationV2 {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }
}

/// One storage-owned finalized-block construction request supplied to the sole
/// evidence ingress. It contains no proof bytes, verifier choice, acceptance
/// flag, prebuilt trace, or backend object. The ingress resolves one strict
/// bundle authority first and only then constructs the canonical transition.
pub struct RecursiveCheckpointChainBlockV2<'a> {
    transition_dir: PathBuf,
    profile: RecursiveCircuitProfileV2,
    checkpoint_store: &'a dyn CheckpointStore,
    prep_snapshot_store: &'a dyn PrepSnapshotStore,
    checkpoint_id: CheckpointId,
    handoff: SettlementExecHandoff,
}

impl<'a> RecursiveCheckpointChainBlockV2<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transition_dir: impl Into<PathBuf>,
        profile: RecursiveCircuitProfileV2,
        checkpoint_store: &'a dyn CheckpointStore,
        prep_snapshot_store: &'a dyn PrepSnapshotStore,
        checkpoint_id: CheckpointId,
        handoff: SettlementExecHandoff,
    ) -> Self {
        Self {
            transition_dir: transition_dir.into(),
            profile,
            checkpoint_store,
            prep_snapshot_store,
            checkpoint_id,
            handoff,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LiveAuthorityAttemptV2 {
    snapshot: RecursiveAuthoritySnapshotV2,
    profile: RecursiveCircuitProfileV2,
    cutover: RecursiveFinalizedIvcStateV2,
}

struct EvidenceProcessLockV2<'a> {
    file: &'a File,
}

impl<'a> EvidenceProcessLockV2<'a> {
    fn acquire(file: &'a File) -> Result<Self, CheckpointError> {
        file.lock_exclusive()
            .map_err(|_| CheckpointError::Storage)?;
        Ok(Self { file })
    }
}

impl Drop for EvidenceProcessLockV2<'_> {
    fn drop(&mut self) {
        let _ = FileExt::unlock(self.file);
    }
}

/// One fixed-layout local evidence store. It has no scheduler, retry queue,
/// provider SDK, or authority promotion API.
pub struct RecursiveCheckpointEvidenceStoreV2 {
    envelopes: SecureDir,
    sidecars: SecureDir,
    claims: SecureDir,
    quarantine: SecureDir,
    process_lock: File,
    recovery: NovaRecoveryStoreV2,
    compression_policy: NovaCompressionPolicyV2,
    session: Mutex<Option<NovaContinuousSessionV2>>,
    verifier: Mutex<Option<NovaVerifierCacheV2>>,
    verifier_attempts: AtomicU64,
}

impl RecursiveCheckpointEvidenceStoreV2 {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, CheckpointError> {
        let root = root.as_ref().to_path_buf();
        let root_dir = SecureDir::ensure_private(&root).map_err(|_| CheckpointError::Storage)?;
        let envelopes = root_dir
            .ensure_dir("envelopes")
            .map_err(|_| CheckpointError::Storage)?;
        let sidecars = root_dir
            .ensure_dir("sidecars")
            .map_err(|_| CheckpointError::Storage)?;
        let claims = root_dir
            .ensure_dir("claims")
            .map_err(|_| CheckpointError::Storage)?;
        let quarantine = root_dir
            .ensure_dir("quarantine")
            .map_err(|_| CheckpointError::Storage)?;
        let process_lock = root_dir
            .open_lock(".evidence-session.lock")
            .map_err(|_| CheckpointError::Storage)?;
        let recovery = NovaRecoveryStoreV2::open(root.join("recovery"))?;
        let compression_policy = NovaCompressionPolicyV2::authority_pinned()?;
        scavenge_temporary_files(&envelopes)?;
        scavenge_temporary_files(&sidecars)?;
        scavenge_temporary_files(&claims)?;
        Ok(Self {
            envelopes,
            sidecars,
            claims,
            quarantine,
            process_lock,
            recovery,
            compression_policy,
            session: Mutex::new(None),
            verifier: Mutex::new(None),
            verifier_attempts: AtomicU64::new(0),
        })
    }

    /// Execute the mandatory gate sequence: native evaluation, continuous
    /// fold, local verification, exact envelope persistence/reload/reverify,
    /// reference-only sidecar persistence/reload, final authority/snapshot
    /// check, then write-only receipt creation.
    pub fn produce(
        &self,
        blocks: &mut [RecursiveCheckpointChainBlockV2<'_>],
        settlement_store: &mut SettlementStore,
        prover_material_bytes: &[u8],
        verifier_bundle_bytes: &[u8],
        cancellation: &RecursiveEvidenceCancellationV2,
        request: RecursiveEvidenceRequestV2,
    ) -> Result<RecursiveEvidenceOutcomeV2, CheckpointError> {
        if blocks.is_empty() {
            return Err(CheckpointError::RecursiveRejected(
                super::recursive_reject::RecursiveCheckpointRejectReasonV2::ChainTooShort,
            ));
        }
        let _process_lock = EvidenceProcessLockV2::acquire(&self.process_lock)?;
        require_secret_process_hardening()?;

        let stage = LiveGateStageV2::<GateStartV2>::start();
        let (stage, authority) = stage
            .authority_resolved(capture_live_authority(blocks, settlement_store))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, config_attempt) = stage
            .retain(authority.snapshot.begin_evidence_attempt())
            .map_err(LiveGateFailureV2::into_error)?;
        let _config_attempt = config_attempt;
        let verifier_result = self
            .verifier
            .lock()
            .map_err(|_| CheckpointError::Storage)
            .and_then(|mut cache| {
                resolve_cached_verifier_v2(
                    &mut cache,
                    prover_material_bytes,
                    verifier_bundle_bytes,
                    authority.snapshot,
                    &authority.profile,
                )
            });
        let (stage, verifier) = stage
            .family_selected(verifier_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let deadline = Instant::now()
            .checked_add(LIVE_RUN_MAX_DURATION_V2)
            .ok_or(CheckpointError::Resource)?;
        let guard =
            NovaRunGuardV2::from_profile(deadline, cancellation.flag(), &authority.profile)?;

        let mut transitions_result = Ok(Vec::with_capacity(blocks.len()));
        for block in blocks.iter() {
            let transition = CanonicalCheckpointTransitionV2::from_exec_with_verifier(
                &block.transition_dir,
                authority.profile,
                block.checkpoint_store,
                block.prep_snapshot_store,
                block.checkpoint_id,
                &mut *settlement_store,
                block.handoff.clone(),
                verifier,
            );
            match (&mut transitions_result, transition) {
                (Ok(transitions), Ok(transition)) => transitions.push(transition),
                (_, Err(error)) => {
                    transitions_result = Err(error);
                    break;
                }
                (Err(_), Ok(_)) => unreachable!("failed transition loop must stop"),
            }
        }
        let (stage, mut transitions) = stage
            .retain(transitions_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(revalidate_attempt(
                &authority,
                &transitions,
                blocks,
                settlement_store,
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let target_height = transitions
            .last()
            .map(CanonicalCheckpointTransitionV2::checkpoint_height)
            .ok_or(CheckpointError::Invariant)?;
        let cadence_action = match request {
            RecursiveEvidenceRequestV2::FoldOnly => NovaCadenceActionV2 {
                fold: true,
                recovery_snapshot: false,
                compress: false,
                publish: false,
            },
            RecursiveEvidenceRequestV2::Snapshot { authority, cadence } => {
                let action = self
                    .compression_policy
                    .action(target_height, authority, cadence)?;
                if !action.compress {
                    return Err(CheckpointError::Authority);
                }
                action
            }
            RecursiveEvidenceRequestV2::RecoverySnapshot { authority } => {
                let action = self.compression_policy.action(
                    target_height,
                    authority,
                    NovaCadenceRequestV2::RecoverySnapshot,
                )?;
                if !action.recovery_snapshot {
                    return Err(CheckpointError::Authority);
                }
                action
            }
        };
        let nova_blocks = transitions
            .iter_mut()
            .map(|transition| NovaChainTransitionV2 {
                transition,
                store: &*settlement_store,
            })
            .collect::<Vec<_>>();

        // Keep the slot locked through receipt issuance. A concurrent caller
        // cannot observe an empty slot and fork a second accumulator lineage.
        let mut session_slot = self.session.lock().map_err(|_| CheckpointError::Storage)?;
        let mut iterator = nova_blocks.into_iter();
        let mut new_session = None;
        if let Some(existing) = session_slot.as_mut() {
            contain_backend(|| {
                existing.renew_guard(guard)?;
                if existing.bundle_digest()? != verifier.verifier_bundle_digest() {
                    return Err(CheckpointError::Authority);
                }
                let mut pending = iterator.collect::<Vec<_>>();
                let _ = existing.fold_blocks(&mut pending)?;
                Ok(())
            })?;
        } else {
            new_session = Some(contain_backend(|| {
                if let Some((snapshot, image)) = self.recovery.latest()? {
                    let first = iterator.next().ok_or(CheckpointError::Invariant)?;
                    let resumed = NovaContinuousSessionV2::resume_and_fold_block(
                        snapshot,
                        image,
                        NovaChainTransitionV2 {
                            transition: &mut *first.transition,
                            store: first.store,
                        },
                        prover_material_bytes,
                        verifier_bundle_bytes,
                        guard,
                    )?;
                    let mut resumed = resumed;
                    let mut remaining = iterator.collect::<Vec<_>>();
                    if !remaining.is_empty() {
                        let _ = resumed.fold_blocks(&mut remaining)?;
                    }
                    Ok(resumed)
                } else {
                    let first = iterator.next().ok_or(CheckpointError::Invariant)?;
                    let mut created = NovaContinuousSessionV2::start(
                        first,
                        prover_material_bytes,
                        verifier_bundle_bytes,
                        authority.cutover,
                        guard,
                    )?;
                    let mut remaining = iterator.collect::<Vec<_>>();
                    if !remaining.is_empty() {
                        let _ = created.fold_blocks(&mut remaining)?;
                    }
                    Ok(created)
                }
            })?);
        }
        let session = new_session
            .as_ref()
            .or_else(|| session_slot.as_ref())
            .ok_or(CheckpointError::Invariant)?;
        if session.bundle_digest()? != verifier.verifier_bundle_digest() {
            return Err(CheckpointError::Authority);
        }
        let successor = session.successor()?;
        if successor.height() != target_height {
            return Err(CheckpointError::Invariant);
        }
        // The fold is already authoritative local progress. Restore the valid
        // lineage before recovery, compression, persistence, or publication
        // can fail so a non-authoritative artifact error cannot drop it.
        if let Some(session) = new_session {
            *session_slot = Some(session);
        }
        let recovery_snapshot = if cadence_action.recovery_snapshot {
            let snapshot = contain_backend(|| {
                session_slot
                    .as_ref()
                    .ok_or(CheckpointError::Invariant)?
                    .recovery_snapshot()
            })?;
            Some(snapshot)
        } else {
            None
        };
        if matches!(request, RecursiveEvidenceRequestV2::RecoverySnapshot { .. }) {
            revalidate_or_drop_session(
                &mut session_slot,
                &authority,
                &transitions,
                blocks,
                settlement_store,
            )?;
            let snapshot = recovery_snapshot.ok_or(CheckpointError::Invariant)?;
            let metrics = self.recovery.commit(&snapshot)?;
            let recovery = RecursiveCheckpointRecoveryV2 {
                snapshot_digest: snapshot.digest(),
                snapshot_height: snapshot.height(),
                snapshot_bytes: snapshot.encoded_len() as u64,
                successor,
                metrics,
            };
            return Ok(RecursiveEvidenceOutcomeV2::Recovery(Box::new(recovery)));
        }
        if request == RecursiveEvidenceRequestV2::FoldOnly {
            revalidate_or_drop_session(
                &mut session_slot,
                &authority,
                &transitions,
                blocks,
                settlement_store,
            )?;
            if let Some(snapshot) = recovery_snapshot {
                let _ = self.recovery.commit(&snapshot)?;
            }
            return Ok(RecursiveEvidenceOutcomeV2::Folded(Box::new(successor)));
        }
        if let Some(snapshot) = recovery_snapshot {
            revalidate_or_drop_session(
                &mut session_slot,
                &authority,
                &transitions,
                blocks,
                settlement_store,
            )?;
            let _ = self.recovery.commit(&snapshot)?;
        }
        let (stage, verifier_attempts) = stage
            .retain(self.reserve_verifier_attempts(VERIFIER_ATTEMPTS_PER_RECEIPT_V2))
            .map_err(LiveGateFailureV2::into_error)?;

        let (stage, outer) = stage
            .outer_bounded(contain_backend(|| {
                session_slot
                    .as_ref()
                    .ok_or(CheckpointError::Invariant)?
                    .snapshot()
                    .and_then(|candidate| candidate.check_outer())
            }))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, inner) = stage
            .inner_bounded(contain_backend(|| outer.check_inner()))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, curve) = stage
            .curve_valid(contain_backend(|| inner.check_curve()))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, bundle) = stage
            .bundle_matched(contain_backend(|| curve.check_bundle()))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, backend) = stage
            .backend_verified(contain_backend(|| bundle.check_backend()))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, verified) = stage
            .limbs_matched(contain_backend(|| backend.check_limbs()))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, bindings) = stage
            .retain(verified.bindings())
            .map_err(LiveGateFailureV2::into_error)?;
        let binding_revalidation = revalidate_or_drop_session(
            &mut session_slot,
            &authority,
            &transitions,
            blocks,
            settlement_store,
        );
        let (stage, ()) = stage
            .bindings_matched(binding_revalidation)
            .map_err(LiveGateFailureV2::into_error)?;
        let storage_generation_result = transitions
            .last()
            .map(CanonicalCheckpointTransitionV2::recursive_storage_generation)
            .ok_or(CheckpointError::Invariant);
        let (stage, storage_generation) = stage
            .retain(storage_generation_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let trace_context =
            live_gate_context_digest(storage_generation, verified.envelope_digest(), bindings);
        let stage = stage
            .bind_context(trace_context)
            .map_err(LiveGateFailureV2::into_error)?;
        let endpoint_revalidation = revalidate_or_drop_session(
            &mut session_slot,
            &authority,
            &transitions,
            blocks,
            settlement_store,
        );
        let (stage, ()) = stage
            .endpoint_reloaded(endpoint_revalidation)
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .prewrite_complete(contain_backend(|| {
                verified.verify_exact_bytes(verified.framed_envelope())
            }))
            .map_err(LiveGateFailureV2::into_error)?;

        let envelope_name = object_name(verified.envelope_digest());
        let mut written = Vec::<(&str, String, bool)>::new();
        let persisted = (|| {
            let envelope_created =
                self.persist_attempt("envelopes", &envelope_name, verified.framed_envelope())?;
            written.push(("envelopes", envelope_name.clone(), envelope_created));
            revalidate_or_drop_session(
                &mut session_slot,
                &authority,
                &transitions,
                blocks,
                settlement_store,
            )?;
            let reloaded_envelope =
                read_exact_bounded(&self.envelopes, &envelope_name, ENVELOPE_READ_CAP_V2)?;
            contain_backend(|| verified.verify_exact_bytes(&reloaded_envelope))?;
            revalidate_or_drop_session(
                &mut session_slot,
                &authority,
                &transitions,
                blocks,
                settlement_store,
            )?;

            let sidecar = RecursiveCheckpointSidecarV2::new(
                storage_generation,
                verified.envelope_digest(),
                verified.framed_envelope().len(),
                bindings,
            )?;
            check_shadow_sidecar_binding(
                &sidecar,
                storage_generation,
                verified.envelope_digest(),
                verified.framed_envelope().len(),
                bindings,
            )?;
            let sidecar_bytes = RecursiveCheckpointSidecarCodecV2::encode_bin(&sidecar)?;
            let sidecar_digest = recursive_sidecar_digest(&sidecar_bytes);
            let sidecar_name = object_name(sidecar_digest);
            let (stage, sidecar_created) = stage
                .atomic_write(self.persist_attempt("sidecars", &sidecar_name, &sidecar_bytes))
                .map_err(LiveGateFailureV2::into_error)?;
            written.push(("sidecars", sidecar_name.clone(), sidecar_created));
            let (stage, reloaded_sidecar_bytes) = stage
                .bytes_reloaded(read_exact_bounded(
                    &self.sidecars,
                    &sidecar_name,
                    SIDECAR_READ_CAP_V2,
                ))
                .map_err(LiveGateFailureV2::into_error)?;
            let reloaded_sidecar =
                RecursiveCheckpointSidecarCodecV2::decode_bin(&reloaded_sidecar_bytes)?;
            check_shadow_sidecar_binding(
                &reloaded_sidecar,
                storage_generation,
                verified.envelope_digest(),
                verified.framed_envelope().len(),
                bindings,
            )?;
            if reloaded_sidecar != sidecar {
                return Err(CheckpointError::Canonical);
            }
            revalidate_or_drop_session(
                &mut session_slot,
                &authority,
                &transitions,
                blocks,
                settlement_store,
            )?;
            let (stage, ()) = stage
                .post_backend_verified(contain_backend(|| {
                    verified.verify_exact_bytes(&reloaded_envelope)
                }))
                .map_err(LiveGateFailureV2::into_error)?;
            let post_endpoint = (|| {
                check_shadow_sidecar_binding(
                    &reloaded_sidecar,
                    storage_generation,
                    verified.envelope_digest(),
                    verified.framed_envelope().len(),
                    bindings,
                )?;
                revalidate_or_drop_session(
                    &mut session_slot,
                    &authority,
                    &transitions,
                    blocks,
                    settlement_store,
                )
            })();
            let (stage, ()) = stage
                .post_endpoint_matched(post_endpoint)
                .map_err(LiveGateFailureV2::into_error)?;
            let claim_name = format!("{}.claim", lowercase_hex(bindings.checkpoint_id));
            let claim_bytes = checkpoint_claim_bytes(
                storage_generation,
                verified.envelope_digest(),
                sidecar_digest,
                bindings,
            );
            let claim_created = self.persist_claim(&claim_name, &claim_bytes)?;
            written.push(("claims", claim_name, claim_created));
            let postwrite = PostwriteVerifiedV2::new(
                stage,
                storage_generation,
                verified.envelope_digest(),
                sidecar_digest,
                bindings,
            )
            .map_err(LiveGateFailureV2::into_error)?;
            // All registry, field and size checks finish before gate 16. Once
            // that gate succeeds, final receipt construction is infallible and
            // no later I/O or authority operation remains.
            let ready = postwrite.prepare_receipt()?;
            let receipt = ready.issue()?;
            let receipt_digest = recursive_receipt_digest(receipt.canonical_bytes());
            Ok((reloaded_sidecar, sidecar_digest, receipt, receipt_digest))
        })();

        let (sidecar, sidecar_digest, receipt, receipt_digest) = match persisted {
            Ok(output) => output,
            Err(error) => {
                self.quarantine_attempt(&written)?;
                return Err(error);
            }
        };
        Ok(RecursiveEvidenceOutcomeV2::Snapshot(Box::new(
            RecursiveCheckpointEvidenceV2 {
                sidecar,
                receipt,
                envelope_digest: verified.envelope_digest(),
                sidecar_digest,
                receipt_digest,
                successor,
                verifier_attempts,
            },
        )))
    }

    fn object_dir(&self, class: &str) -> Result<&SecureDir, CheckpointError> {
        match class {
            "envelopes" => Ok(&self.envelopes),
            "sidecars" => Ok(&self.sidecars),
            "claims" => Ok(&self.claims),
            _ => Err(CheckpointError::Invariant),
        }
    }

    fn reserve_verifier_attempts(&self, count: u64) -> Result<u64, CheckpointError> {
        self.verifier_attempts
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current
                    .checked_add(count)
                    .filter(|next| *next <= MAX_VERIFIER_ATTEMPTS_V2)
            })
            .map(|previous| previous + count)
            .map_err(|_| CheckpointError::Resource)
    }

    fn persist_attempt(
        &self,
        class: &'static str,
        name: &str,
        bytes: &[u8],
    ) -> Result<bool, CheckpointError> {
        let directory = self.object_dir(class)?;
        match persist_content_addressed(directory, name, bytes) {
            Ok(created) => Ok(created),
            Err(CheckpointError::Canonical) => {
                let cap = u64::try_from(bytes.len())
                    .map_err(|_| CheckpointError::Limit)?
                    .checked_add(1)
                    .ok_or(CheckpointError::Overflow)?;
                if directory.read_file_bounded(name, cap).is_err() {
                    return Err(CheckpointError::Canonical);
                }
                quarantine_written(directory, class, name, &self.quarantine)?;
                persist_content_addressed(directory, name, bytes)
            }
            Err(error) => Err(error),
        }
    }

    fn quarantine_attempt(&self, written: &[(&str, String, bool)]) -> Result<(), CheckpointError> {
        const MAX_QUARANTINE_OBJECTS_V2: usize = 64;
        let created = written.iter().filter(|(_, _, created)| *created).count();
        let existing = self
            .quarantine
            .read_dir_bounded(MAX_QUARANTINE_OBJECTS_V2)
            .map_err(|_| CheckpointError::Limit)?
            .len();
        if existing
            .checked_add(created)
            .filter(|total| *total <= MAX_QUARANTINE_OBJECTS_V2)
            .is_none()
        {
            return Err(CheckpointError::Limit);
        }
        for (class, name, created) in written {
            if *created {
                quarantine_written(self.object_dir(class)?, class, name, &self.quarantine)?;
            }
        }
        Ok(())
    }

    fn persist_claim(&self, name: &str, bytes: &[u8]) -> Result<bool, CheckpointError> {
        const CLAIM_READ_CAP_V2: u64 = 1024;
        if let Ok(existing) = self.claims.read_file_bounded(name, CLAIM_READ_CAP_V2) {
            if existing == bytes {
                return Ok(false);
            }
            if is_checkpoint_claim_canonical(&existing) {
                return Err(CheckpointError::Canonical);
            }
            quarantine_written(&self.claims, "claims", name, &self.quarantine)?;
        }
        persist_content_addressed(&self.claims, name, bytes)
    }
}

fn capture_live_authority(
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
    settlement_store: &SettlementStore,
) -> Result<LiveAuthorityAttemptV2, CheckpointError> {
    let first = blocks.first().ok_or(CheckpointError::Invariant)?;
    let snapshot = RecursiveAuthoritySnapshotV2::resolve_active_authority(settlement_store)?;
    let profile = first.profile;
    let cutover = RecursiveFinalizedIvcStateV2::from_cutover_store(settlement_store)?;
    let attempt = LiveAuthorityAttemptV2 {
        snapshot,
        profile,
        cutover,
    };
    revalidate_live_authority(&attempt, blocks, settlement_store)?;
    Ok(attempt)
}

fn require_secret_process_hardening() -> Result<(), CheckpointError> {
    let report = z00z_utils::os_hardening::apply_best_effort();
    #[cfg(all(unix, not(target_os = "ios")))]
    if !report.core_dumps_disabled {
        return Err(CheckpointError::Resource);
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    if !report.non_dumpable {
        return Err(CheckpointError::Resource);
    }
    Ok(())
}

fn contain_backend<T>(
    operation: impl FnOnce() -> Result<T, CheckpointError>,
) -> Result<T, CheckpointError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
        .map_err(|_| CheckpointError::BackendVerificationFailed)?
}

fn revalidate_live_authority(
    attempt: &LiveAuthorityAttemptV2,
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
    settlement_store: &SettlementStore,
) -> Result<(), CheckpointError> {
    attempt.snapshot.revalidate_config()?;
    for block in blocks {
        let live = RecursiveAuthoritySnapshotV2::resolve_active_authority(settlement_store)?;
        let cutover = RecursiveFinalizedIvcStateV2::from_cutover_store(settlement_store)?;
        if live.authority() != attempt.snapshot.authority()
            || block.profile != attempt.profile
            || cutover != attempt.cutover
        {
            return Err(CheckpointError::Authority);
        }
    }
    Ok(())
}

fn revalidate_attempt(
    attempt: &LiveAuthorityAttemptV2,
    transitions: &[CanonicalCheckpointTransitionV2],
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
    settlement_store: &SettlementStore,
) -> Result<(), CheckpointError> {
    revalidate_live_authority(attempt, blocks, settlement_store)?;
    revalidate_chain(transitions, blocks, settlement_store)
}

fn revalidate_or_drop_session(
    session: &mut Option<NovaContinuousSessionV2>,
    attempt: &LiveAuthorityAttemptV2,
    transitions: &[CanonicalCheckpointTransitionV2],
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
    settlement_store: &SettlementStore,
) -> Result<(), CheckpointError> {
    let result = revalidate_attempt(attempt, transitions, blocks, settlement_store);
    if result.is_err() {
        *session = None;
    }
    result
}

fn revalidate_chain(
    transitions: &[CanonicalCheckpointTransitionV2],
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
    settlement_store: &SettlementStore,
) -> Result<(), CheckpointError> {
    if transitions.len() != blocks.len() {
        return Err(CheckpointError::Invariant);
    }
    for transition in transitions {
        transition.revalidate_evidence_authority(settlement_store)?;
    }
    Ok(())
}

fn quarantine_written(
    source: &SecureDir,
    class: &str,
    name: &str,
    quarantine: &SecureDir,
) -> Result<(), CheckpointError> {
    const MAX_QUARANTINE_OBJECTS_V2: usize = 64;
    let entries = quarantine
        .read_dir_bounded(MAX_QUARANTINE_OBJECTS_V2)
        .map_err(|_| CheckpointError::Limit)?;
    let destination = format!("{class}-{name}");
    if entries.iter().any(|entry| entry == destination.as_str()) {
        source
            .remove_file(name)
            .map_err(|_| CheckpointError::Storage)?;
        source.sync().map_err(|_| CheckpointError::Storage)?;
        return Ok(());
    }
    if entries.len() >= MAX_QUARANTINE_OBJECTS_V2 {
        return Err(CheckpointError::Limit);
    }
    source
        .rename_to_no_clobber(name, quarantine, &destination)
        .map_err(|_| CheckpointError::Storage)?;
    source.sync().map_err(|_| CheckpointError::Storage)?;
    quarantine.sync().map_err(|_| CheckpointError::Storage)
}

fn scavenge_temporary_files(directory: &SecureDir) -> Result<(), CheckpointError> {
    const MAX_SCAVENGE_ENTRIES_V2: usize = 256;
    let entries = directory
        .read_dir_bounded(MAX_SCAVENGE_ENTRIES_V2)
        .map_err(|_| CheckpointError::Limit)?;
    let mut changed = false;
    for entry in entries {
        let Some(name) = entry.to_str() else {
            return Err(CheckpointError::Canonical);
        };
        if name.starts_with(".tmp-") {
            directory
                .remove_file(name)
                .map_err(|_| CheckpointError::Storage)?;
            changed = true;
        }
    }
    if changed {
        directory.sync().map_err(|_| CheckpointError::Storage)?;
    }
    Ok(())
}

fn persist_content_addressed(
    directory: &SecureDir,
    name: &str,
    bytes: &[u8],
) -> Result<bool, CheckpointError> {
    let byte_cap = u64::try_from(bytes.len()).map_err(|_| CheckpointError::Limit)?;
    if let Ok(existing) = directory.read_file_bounded(name, byte_cap) {
        return if existing == bytes {
            Ok(false)
        } else {
            Err(CheckpointError::Canonical)
        };
    }

    static TEMP_SEQUENCE_V2: AtomicU64 = AtomicU64::new(0);
    let mut temporary = None;
    for _ in 0..8 {
        let sequence = TEMP_SEQUENCE_V2.fetch_add(1, Ordering::Relaxed);
        let candidate = format!(".tmp-{}-{sequence}-{name}", std::process::id());
        if let Ok(file) = directory.create_file(&candidate) {
            temporary = Some((candidate, file));
            break;
        }
    }
    let (temporary_name, mut temporary_file) = temporary.ok_or(CheckpointError::Storage)?;
    if temporary_file
        .write_all(bytes)
        .and_then(|()| temporary_file.sync_all())
        .is_err()
    {
        drop(temporary_file);
        let _ = directory.remove_file(&temporary_name);
        let _ = directory.sync();
        return Err(CheckpointError::Storage);
    }
    drop(temporary_file);
    if directory.rename_no_clobber(&temporary_name, name).is_err() {
        let existing = directory.read_file_bounded(name, byte_cap);
        let _ = directory.remove_file(&temporary_name);
        let _ = directory.sync();
        return match existing {
            Ok(existing) if existing == bytes => Ok(false),
            _ => Err(CheckpointError::Canonical),
        };
    }
    directory.sync().map_err(|_| CheckpointError::Storage)?;
    Ok(true)
}

fn read_exact_bounded(
    directory: &SecureDir,
    name: &str,
    cap: u64,
) -> Result<Vec<u8>, CheckpointError> {
    directory
        .read_file_bounded(name, cap)
        .map_err(|_| CheckpointError::Storage)
}

fn object_name(digest: [u8; 32]) -> String {
    format!("{}.bin", lowercase_hex(digest))
}

fn lowercase_hex(digest: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        checkpoint::{
            CheckpointDraft, CheckpointExecInput, CheckpointFsStore, CheckpointStore,
            CheckpointVersion,
        },
        fixture_support::checkpoint_fixtures,
        settlement::SettlementStateRoot,
        snapshot::{build_snapshot_v2, PrepFsStore, PrepSnapshotStore},
    };
    use z00z_utils::io::{create_dir_all, path_exists_no_follow, read_to_string};
    use z00z_utils::logger::{Logger, StdoutLogger};

    fn seal_noop_checkpoint(
        root: &Path,
        height: u64,
        settlement_root: SettlementStateRoot,
        prior: RecursiveFinalizedIvcStateV2,
    ) -> (CheckpointFsStore, PrepFsStore, CheckpointId) {
        let draft = CheckpointDraft::new_settlement(
            CheckpointVersion::CURRENT,
            height,
            settlement_root,
            settlement_root,
            Vec::new(),
            Vec::new(),
        );
        let (snapshot, snapshot_id) =
            build_snapshot_v2(settlement_root, Vec::new()).expect("T3 no-op prep snapshot");
        let mut prep_store = PrepFsStore::new(root);
        assert_eq!(
            prep_store
                .save_snapshot(&snapshot)
                .expect("persist T3 no-op prep snapshot"),
            snapshot_id
        );
        let exec = CheckpointExecInput::new_recursive_v2_noop(snapshot_id, settlement_root)
            .expect("typed T3 no-op execution input");
        let mut checkpoint_store = CheckpointFsStore::new(root);
        let exec_id = checkpoint_store
            .save_exec_input(&exec)
            .expect("persist T3 no-op execution input");
        let statement_core = checkpoint_fixtures::statement_core(&exec)
            .with_prior_recursive_output_root(prior.digest());
        let manifest =
            checkpoint_fixtures::archive_manifest_with_core(&draft, &exec, exec_id, statement_core);
        let da_reference = checkpoint_fixtures::da_reference(&manifest);
        checkpoint_store
            .stage_publication_contract(exec_id, &statement_core, &manifest, &da_reference)
            .expect("stage T3 no-op publication contract");
        let link = checkpoint_store
            .seal_artifact(
                &draft,
                draft
                    .attest_proof(snapshot_id, exec_id)
                    .expect("T3 no-op attested proof"),
                snapshot_id,
                exec_id,
            )
            .unwrap_or_else(|error| {
                panic!("seal T3 no-op checkpoint at height {height}: {error:?}")
            });
        (checkpoint_store, prep_store, link.checkpoint_id())
    }

    fn retained_t3_artifact(name: &str, cap: u64) -> Vec<u8> {
        let root = PathBuf::from(
            std::env::var_os("Z00Z_NOVA_T3_ARTIFACT_DIR_V2")
                .expect("retained T3 artifact directory"),
        );
        z00z_utils::io::read_file_bounded(root.join(name), cap)
            .expect("read bounded retained T3 artifact")
    }

    fn process_peak_rss_bytes() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = read_to_string("/proc/self/status") {
                if let Some(kibibytes) = status.lines().find_map(|line| {
                    line.strip_prefix("VmHWM:")
                        .and_then(|value| value.split_whitespace().next())
                        .and_then(|value| value.parse::<u64>().ok())
                }) {
                    return kibibytes.saturating_mul(1024).max(1);
                }
            }
        }
        1
    }

    #[test]
    fn test_noop_links_stay_linear() {
        let chain = tempfile::tempdir().expect("no-op chain root");
        let store = SettlementStore::new();
        let settlement_root = store.settlement_root_v2(7).expect("no-op chain root");
        let prior = RecursiveFinalizedIvcStateV2::cutover_fixture(
            CheckpointId::new([0x31; 32]),
            *settlement_root.as_bytes(),
            store.recursive_v2_definition_root(),
            [0x32; 32],
        )
        .expect("no-op chain cutover");
        let mut predecessor = None;

        for height in 1_u64..=3 {
            let (checkpoint_store, _, checkpoint_id) =
                seal_noop_checkpoint(chain.path(), height, settlement_root, prior);
            let link = checkpoint_store
                .load_link(&checkpoint_id)
                .expect("load linear no-op link");
            assert_eq!(link.prev_checkpoint_id(), predecessor);
            predecessor = Some(checkpoint_id);
        }
    }

    #[test]
    fn test_content_write_rejects_collision() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("evidence");
        let store = RecursiveCheckpointEvidenceStoreV2::open(&root).unwrap();
        let path = root.join("sidecars").join(object_name([7; 32]));
        let name = object_name([7; 32]);
        persist_content_addressed(&store.sidecars, &name, b"canonical").unwrap();
        persist_content_addressed(&store.sidecars, &name, b"canonical").unwrap();
        assert_eq!(
            read_exact_bounded(&store.sidecars, &name, 64).unwrap(),
            b"canonical"
        );
        assert!(persist_content_addressed(&store.sidecars, &name, b"different").is_err());
        assert!(path_exists_no_follow(path).unwrap());
        assert!(store
            .sidecars
            .read_dir_bounded(64)
            .unwrap()
            .into_iter()
            .all(|name| !name.to_string_lossy().starts_with(".tmp-")));
    }

    fn stage_through_reload(context: [u8; 32]) -> LiveGateStageV2<ReloadedV2> {
        let stage = LiveGateStageV2::<GateStartV2>::start();
        let (stage, ()) = stage
            .authority_resolved(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .family_selected(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .outer_bounded(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .inner_bounded(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .curve_valid(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .bundle_matched(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .backend_verified(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .limbs_matched(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .bindings_matched(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let stage = stage
            .bind_context(context)
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .endpoint_reloaded(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .prewrite_complete(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .atomic_write(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .bytes_reloaded(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        stage
    }

    fn receipt_bindings() -> super::super::nova::NovaVerificationBindingsV2 {
        super::super::nova::NovaVerificationBindingsV2 {
            authority_generation: 2,
            config_digest: [1; 32],
            bundle_digest: [2; 32],
            public_input_digest: [3; 32],
            initial_state_digest: [4; 32],
            final_state_digest: [5; 32],
            final_state_limbs: 4096,
            successor_finalized_state_digest: [6; 32],
            statement_digest: [7; 32],
            checkpoint_link_digest: [8; 32],
            prior_output_root: [15; 32],
            output_root: [16; 32],
            trace_digest: [9; 32],
            gate_trace_digest: [0; 32],
            checkpoint_id: [10; 32],
            predecessor: Some([11; 32]),
            height: 5,
            steps: 1727,
            backend_revision_result_digest: [12; 32],
        }
    }

    fn postwrite_token_with(
        context: [u8; 32],
        bindings: super::super::nova::NovaVerificationBindingsV2,
    ) -> PostwriteVerifiedV2 {
        let stage = stage_through_reload(context);
        let (stage, ()) = stage
            .post_backend_verified(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let (stage, ()) = stage
            .post_endpoint_matched(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        PostwriteVerifiedV2::new(stage, 0, [13; 32], [14; 32], bindings)
            .map_err(LiveGateFailureV2::into_error)
            .unwrap()
    }

    fn postwrite_token(context: [u8; 32]) -> PostwriteVerifiedV2 {
        postwrite_token_with(context, receipt_bindings())
    }

    #[test]
    fn test_gate_typestate_failure_prefix() {
        let context = [9; 32];
        let stage = stage_through_reload(context);
        let expected = SUCCESSFUL_GATE_SEQUENCE_V2[..13]
            .iter()
            .map(|gate| *gate as u8)
            .collect::<Vec<_>>();
        let failure = match stage
            .post_backend_verified::<()>(Err(CheckpointError::BackendVerificationFailed))
        {
            Ok(_) => panic!("failed backend gate advanced"),
            Err(failure) => failure,
        };
        assert_eq!(failure.ids(), expected);
        assert_eq!(
            std::mem::discriminant(failure.terminal()),
            std::mem::discriminant(&CheckpointError::BackendVerificationFailed)
        );

        let reordered = LiveGateTraceV2::start();
        assert!(reordered.push(LiveGateIdV2::BundleMatched).is_err());
    }

    #[test]
    fn test_bytes_follow_receipt_gate() {
        let context = [7; 32];
        let postwrite = postwrite_token(context);
        assert_eq!(
            postwrite.gate_ids().last(),
            Some(&(LiveGateIdV2::PostwriteEndpointMatched as u8))
        );
        assert_eq!(postwrite.gate_ids().len(), 15);

        let ready = postwrite.prepare_receipt().unwrap();
        assert_eq!(ready.postwrite.gate_ids().len(), 15);
        let receipt = ready.issue().unwrap();
        assert!(!receipt.canonical_bytes().is_empty());
    }

    #[test]
    fn test_receipt_stays_opaque() {
        let context = [8; 32];
        let temp = tempfile::tempdir().unwrap();
        let evidence_root = temp.path().join("evidence");
        let store = RecursiveCheckpointEvidenceStoreV2::open(&evidence_root).unwrap();
        let receipt = postwrite_token(context)
            .prepare_receipt()
            .unwrap()
            .issue()
            .unwrap();
        let receipt_bytes = receipt.canonical_bytes().to_vec();
        assert_eq!(
            receipt_bytes.len(),
            super::super::version_registry::RECURSIVE_OBJECT_PREHEADER_BYTES_V2
                + super::super::receipt::RECURSIVE_RECEIPT_PAYLOAD_BYTES_V2
        );
        assert!(!evidence_root.join("receipts").exists());
        assert!(store
            .quarantine
            .read_dir_bounded(1)
            .expect("bounded quarantine read")
            .is_empty());

        assert_eq!(receipt.canonical_bytes(), receipt_bytes);
        assert_eq!(
            receipt.result(),
            super::super::receipt::RecursiveVerificationResultV2::VerifiedExactReload
        );
    }

    #[test]
    fn test_invalid_receipt_parts_reject() {
        let mut bindings = receipt_bindings();
        bindings.steps = 0;
        let error = match postwrite_token_with([15; 32], bindings).prepare_receipt() {
            Ok(_) => panic!("invalid receipt must not prepare"),
            Err(error) => error,
        };
        assert!(matches!(error, CheckpointError::Invariant));
    }

    #[test]
    #[ignore = "real continuous 1/3/5-block Nova ingress is milestone-only; run nova_milestone_tests.sh t3-chain"]
    fn test_real_chain_public_receipt() {
        use crate::{
            checkpoint::canonical_transition::SettlementRootGenerationCutoverV2,
            fixture_support::settlement_corpus::{asset_item, load_fixture, redb_store},
        };
        use z00z_crypto::{sha256_256_role, CheckpointShaRole};

        crate::fixture_support::genesis_chain_identity::ensure_test_process_chain_identity()
            .expect("validated canonical devnet genesis identity");
        let material = retained_t3_artifact("prover-material.bin", 1024 * 1024 * 1024);
        // Match the sole format-4 bundle admission ceiling before this
        // milestone helper reads the artifact into memory. The production
        // Nova loader performs the same fail-closed check again.
        let bundle = retained_t3_artifact("verifier-bundle.bin", 64 * 1024 * 1024);
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let chain = tempfile::tempdir().expect("continuous T3 chain root");
        let evidence_root = chain.path().join("evidence");
        let checkpoint_root = chain.path().join("checkpoints");
        create_dir_all(&checkpoint_root).expect("create checkpoint root");

        // The retained prover/VK material is bound to the canonical active
        // authority policy.  A test-only HJMT bucket override would select a
        // distinct authority and must be covered by the negative corpus, not
        // by this positive continuous-ingress path.
        let (_hjmt_guard, _settlement_dir, mut settlement_store) =
            redb_store().expect("durable T3 settlement store");
        let fixture = load_fixture();
        settlement_store
            .put_settlement_item(asset_item(&fixture.assets[0]))
            .expect("persist T3 settlement root");
        let authority = RecursiveAuthoritySnapshotV2::resolve_active_authority(&settlement_store)
            .expect("resolve cutover authority");
        let settlement_root = settlement_store
            .settlement_root_v2(7)
            .expect("T3 settlement root");
        let opaque = [0x42; 32];
        let pinned_opaque = sha256_256_role(
            CheckpointShaRole::Link,
            &[b"z00z.recursive.v2.opaque-last-root-record", &opaque],
        );
        let mut cutover = SettlementRootGenerationCutoverV2::active_authority(
            authority,
            &settlement_store,
            1,
            opaque,
            pinned_opaque,
            settlement_root,
            11,
        )
        .expect("construct active T3 cutover");
        cutover
            .install_active_authority(&mut settlement_store, 11)
            .expect("install durable T3 cutover");
        let mut prior = RecursiveFinalizedIvcStateV2::from_cutover_store(&settlement_store)
            .expect("derive T3 z0 from durable cutover");

        let evidence_store = RecursiveCheckpointEvidenceStoreV2::open(&evidence_root)
            .expect("open public T3 evidence store");
        let cancellation = RecursiveEvidenceCancellationV2::new();
        let mut chain_steps = Vec::new();

        // Compression is non-authoritative. A too-short snapshot request must
        // report its typed error while retaining the successfully folded
        // height-one accumulator for the next canonical block.
        let (checkpoint_store, prep_store, checkpoint_id) =
            seal_noop_checkpoint(&checkpoint_root, 1, settlement_root, prior);
        let transition_dir = chain.path().join("transition-1");
        create_dir_all(&transition_dir).expect("create first transition directory");
        let mut blocks = [RecursiveCheckpointChainBlockV2::new(
            transition_dir,
            profile,
            &checkpoint_store,
            &prep_store,
            checkpoint_id,
            SettlementExecHandoff::recursive_v2_noop(),
        )];
        assert!(matches!(
            evidence_store.produce(
                &mut blocks,
                &mut settlement_store,
                &material,
                &bundle,
                &cancellation,
                RecursiveEvidenceRequestV2::Snapshot {
                    authority: NovaCompressionAuthorityV2::LocalOperator,
                    cadence: NovaCadenceRequestV2::Compress,
                },
            ),
            Err(CheckpointError::RecursiveRejected(
                super::super::recursive_reject::RecursiveCheckpointRejectReasonV2::ChainTooShort
            ))
        ));
        prior = evidence_store
            .session
            .lock()
            .expect("retained session lock")
            .as_ref()
            .expect("retained post-fold session")
            .successor()
            .expect("retained post-fold successor");
        assert_eq!(prior.height(), 1);

        for height in 2_u64..=7 {
            let (checkpoint_store, prep_store, checkpoint_id) =
                seal_noop_checkpoint(&checkpoint_root, height, settlement_root, prior);
            let transition_dir = chain.path().join(format!("transition-{height}"));
            create_dir_all(&transition_dir).expect("create T3 transition directory");
            let mut blocks = [RecursiveCheckpointChainBlockV2::new(
                transition_dir,
                profile,
                &checkpoint_store,
                &prep_store,
                checkpoint_id,
                SettlementExecHandoff::recursive_v2_noop(),
            )];
            let request = if height >= 3 {
                RecursiveEvidenceRequestV2::Snapshot {
                    authority: NovaCompressionAuthorityV2::LocalOperator,
                    cadence: NovaCadenceRequestV2::Compress,
                }
            } else {
                RecursiveEvidenceRequestV2::FoldOnly
            };
            let started = Instant::now();
            let outcome = evidence_store
                .produce(
                    &mut blocks,
                    &mut settlement_store,
                    &material,
                    &bundle,
                    &cancellation,
                    request,
                )
                .expect("sole public ingress advances or issues verified evidence");
            match outcome {
                RecursiveEvidenceOutcomeV2::Folded(successor) => {
                    assert_eq!(request, RecursiveEvidenceRequestV2::FoldOnly);
                    prior = *successor;
                }
                RecursiveEvidenceOutcomeV2::Snapshot(evidence) => {
                    assert!(matches!(
                        request,
                        RecursiveEvidenceRequestV2::Snapshot { .. }
                    ));
                    assert_eq!(evidence.receipt.height(), height);
                    assert_eq!(evidence.successor.height(), height);
                    assert_eq!(
                        evidence.verifier_attempts,
                        u64::try_from(chain_steps.len()).unwrap() * 4 + 8
                    );
                    assert!(evidence_root
                        .join("envelopes")
                        .join(object_name(evidence.envelope_digest))
                        .is_file());
                    assert!(evidence_root
                        .join("sidecars")
                        .join(object_name(evidence.sidecar_digest))
                        .is_file());
                    assert!(!evidence_root.join("receipts").exists());
                    let statement =
                        super::super::recursive_chain::NovaChainStatementV2::from_sidecar(
                            &evidence.sidecar,
                        );
                    let verification_micros = u64::try_from(started.elapsed().as_micros())
                        .unwrap_or(u64::MAX)
                        .max(1);
                    let measurement =
                        super::super::recursive_chain::NovaChainMeasurementV2::for_verified_receipt(
                            u8::try_from(chain_steps.len()).unwrap(),
                            &evidence.receipt,
                            verification_micros,
                            process_peak_rss_bytes(),
                        );
                    chain_steps.push(super::super::recursive_chain::NovaChainEvidenceStepV2::new(
                        &evidence.receipt,
                        statement,
                        Some(measurement),
                    ));
                    StdoutLogger.info(&format!(
                        "nova_chain_step height={height} verify_us={verification_micros} peak_rss_bytes={}",
                        process_peak_rss_bytes()
                    ));
                    prior = evidence.successor;
                }
                RecursiveEvidenceOutcomeV2::Recovery(_) => {
                    panic!("compression-only fixture returned a recovery outcome")
                }
            }
        }
        assert_eq!(chain_steps.len(), 5);
        let three_root = super::super::recursive_chain::VerifiedNovaChainV2::derive_chain_root(
            &chain_steps[..3],
        );
        let three = super::super::recursive_chain::VerifiedNovaChainV2::verify(
            &chain_steps[..3],
            three_root,
        )
        .expect("real three-receipt Nova chain");
        let five_root =
            super::super::recursive_chain::VerifiedNovaChainV2::derive_chain_root(&chain_steps);
        let five =
            super::super::recursive_chain::VerifiedNovaChainV2::verify(&chain_steps, five_root)
                .expect("real five-receipt Nova chain");
        assert_ne!(three.chain_root(), five.chain_root());
        assert_ne!(
            five.retention_input_facts(0, [0x91; 32])
                .expect("immutable retention input facts")
                .digest(),
            [0; 32]
        );

        let (checkpoint_store, prep_store, checkpoint_id) =
            seal_noop_checkpoint(&checkpoint_root, 8, settlement_root, prior);
        let transition_dir = chain.path().join("transition-8");
        create_dir_all(&transition_dir).expect("create recovery transition directory");
        let mut blocks = [RecursiveCheckpointChainBlockV2::new(
            transition_dir,
            profile,
            &checkpoint_store,
            &prep_store,
            checkpoint_id,
            SettlementExecHandoff::recursive_v2_noop(),
        )];
        let recovery_started = Instant::now();
        let recovery = match evidence_store
            .produce(
                &mut blocks,
                &mut settlement_store,
                &material,
                &bundle,
                &cancellation,
                RecursiveEvidenceRequestV2::RecoverySnapshot {
                    authority: NovaCompressionAuthorityV2::RecoveryWorkflow,
                },
            )
            .expect("commit real recovery snapshot")
        {
            RecursiveEvidenceOutcomeV2::Recovery(recovery) => recovery,
            _ => panic!("explicit recovery request returned another outcome"),
        };
        assert_eq!(recovery.snapshot_height, 8);
        assert_eq!(recovery.metrics.live_snapshot_count, 1);
        assert!(
            recovery.metrics.total_hot_bytes
                <= super::super::recursive_measurement::NovaCadenceManifestV2::authority_pinned()
                    .max_hot_recovery_bytes()
        );
        StdoutLogger.info(&format!(
            "nova_recovery height=8 snapshot_bytes={} elapsed_ms={} peak_rss_bytes={} hot_bytes={}",
            recovery.snapshot_bytes,
            recovery_started.elapsed().as_millis(),
            process_peak_rss_bytes(),
            recovery.metrics.total_hot_bytes
        ));
        prior = recovery.successor;
        drop(evidence_store);

        let resumed_store = RecursiveCheckpointEvidenceStoreV2::open(&evidence_root)
            .expect("reopen evidence store from committed recovery");
        let fork_root = chain.path().join("fork-checkpoints");
        create_dir_all(&fork_root).expect("create fork checkpoint root");
        let (fork_checkpoint_store, fork_prep_store, fork_checkpoint_id) =
            seal_noop_checkpoint(&fork_root, 9, settlement_root, prior);
        let fork_transition_dir = chain.path().join("fork-transition-9");
        create_dir_all(&fork_transition_dir).expect("create fork transition directory");
        let mut fork_blocks = [RecursiveCheckpointChainBlockV2::new(
            fork_transition_dir,
            profile,
            &fork_checkpoint_store,
            &fork_prep_store,
            fork_checkpoint_id,
            SettlementExecHandoff::recursive_v2_noop(),
        )];
        assert!(resumed_store
            .produce(
                &mut fork_blocks,
                &mut settlement_store,
                &material,
                &bundle,
                &cancellation,
                RecursiveEvidenceRequestV2::FoldOnly,
            )
            .is_err());

        let (checkpoint_store, prep_store, checkpoint_id) =
            seal_noop_checkpoint(&checkpoint_root, 9, settlement_root, prior);
        let transition_dir = chain.path().join("transition-9");
        create_dir_all(&transition_dir).expect("create resumed transition directory");
        let mut blocks = [RecursiveCheckpointChainBlockV2::new(
            transition_dir,
            profile,
            &checkpoint_store,
            &prep_store,
            checkpoint_id,
            SettlementExecHandoff::recursive_v2_noop(),
        )];
        let resumed = resumed_store
            .produce(
                &mut blocks,
                &mut settlement_store,
                &material,
                &bundle,
                &cancellation,
                RecursiveEvidenceRequestV2::FoldOnly,
            )
            .expect("resume exact accumulator and fold canonical successor");
        match resumed {
            RecursiveEvidenceOutcomeV2::Folded(successor) => assert_eq!(successor.height(), 9),
            _ => panic!("resumed fold returned another outcome"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_store_rejects_symlink_root() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        create_dir_all(&target).unwrap();
        let link = temp.path().join("link");
        std::os::unix::fs::symlink(target, &link).unwrap();
        assert!(RecursiveCheckpointEvidenceStoreV2::open(link).is_err());
    }
}
