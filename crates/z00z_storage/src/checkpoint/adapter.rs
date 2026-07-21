//! Backend-neutral recursive proof persistence orchestration.

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use z00z_crypto::sha256_256;
use z00z_utils::io::{
    create_dir_all, path_exists_no_follow, read_dir, rename_file, set_file_mode,
    set_permissions_mode, symlink_metadata, sync_directory, File, Read, Write,
};

use super::{
    canonical_transition::CanonicalCheckpointTransitionV2,
    nova::{prove_continuous_chain_v2, resolve_verifier_authority_v2, NovaChainTransitionV2},
    receipt::{recursive_receipt_digest, CryptographicVerificationReceiptV2},
    recursive_circuit::RecursiveCircuitProfileV2,
    recursive_context::RecursiveAuthoritySnapshotV2,
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
const RECEIPT_READ_CAP_V2: u64 = 16 * 1024 + 32;
const LIVE_GATE_DOMAIN_V2: &str = "z00z.storage.checkpoint.live_gate.v2";
const LIVE_GATE_TRACE_LABEL_V2: &str = "gate_trace";
const LIVE_GATE_CONTEXT_LABEL_V2: &str = "gate_context";

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

    fn receipt_digest(&self) -> Result<[u8; 32], CheckpointError> {
        if self.len + 1 != SUCCESSFUL_GATE_SEQUENCE_V2.len()
            || SUCCESSFUL_GATE_SEQUENCE_V2.get(self.len).copied()
                != Some(LiveGateIdV2::ReceiptIssued)
        {
            return Err(CheckpointError::Invariant);
        }
        let context_digest = self.context_digest.ok_or(CheckpointError::Invariant)?;
        let mut ids = self.ids;
        ids[self.len] = LiveGateIdV2::ReceiptIssued as u8;
        Ok(gate_trace_digest(context_digest, &ids))
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

/// One-shot capability created only after exact postwrite backend and endpoint
/// verification. Receipt code can consume it but cannot construct or replay it.
pub(super) struct PostwriteVerifiedV2 {
    stage: LiveGateStageV2<PostEndpointReadyV2>,
    storage_generation: u64,
    envelope_digest: [u8; 32],
    sidecar_digest: [u8; 32],
    bindings: super::nova::NovaVerificationBindingsV2,
    receipt_trace_digest: [u8; 32],
}

pub(super) struct PostwriteReceiptPartsV2 {
    pub(super) storage_generation: u64,
    pub(super) envelope_digest: [u8; 32],
    pub(super) sidecar_digest: [u8; 32],
    pub(super) bindings: super::nova::NovaVerificationBindingsV2,
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
        let receipt_trace_digest = match stage.trace.receipt_digest() {
            Ok(digest) => digest,
            Err(terminal) => return Err(LiveGateFailureV2::new(stage.trace, terminal)),
        };
        Ok(Self {
            stage,
            storage_generation,
            envelope_digest,
            sidecar_digest,
            bindings,
            receipt_trace_digest,
        })
    }

    fn retain<T>(self, result: Result<T, CheckpointError>) -> Result<(Self, T), LiveGateFailureV2> {
        let Self {
            stage,
            storage_generation,
            envelope_digest,
            sidecar_digest,
            bindings,
            receipt_trace_digest,
        } = self;
        let (stage, value) = stage.retain(result)?;
        Ok((
            Self {
                stage,
                storage_generation,
                envelope_digest,
                sidecar_digest,
                bindings,
                receipt_trace_digest,
            },
            value,
        ))
    }

    pub(super) fn issue_receipt(self) -> Result<PostwriteReceiptPartsV2, CheckpointError> {
        let stage = self.stage.issue_receipt()?;
        if stage.trace.digest()? != self.receipt_trace_digest {
            return Err(CheckpointError::Invariant);
        }
        let mut bindings = self.bindings;
        bindings.gate_trace_digest = self.receipt_trace_digest;
        Ok(PostwriteReceiptPartsV2 {
            storage_generation: self.storage_generation,
            envelope_digest: self.envelope_digest,
            sidecar_digest: self.sidecar_digest,
            bindings,
        })
    }

    pub(super) fn receipt_preview(&self) -> PostwriteReceiptPartsV2 {
        let mut bindings = self.bindings;
        bindings.gate_trace_digest = self.receipt_trace_digest;
        PostwriteReceiptPartsV2 {
            storage_generation: self.storage_generation,
            envelope_digest: self.envelope_digest,
            sidecar_digest: self.sidecar_digest,
            bindings,
        }
    }

    #[cfg(test)]
    fn gate_ids(&self) -> &[u8] {
        self.stage.trace.ids()
    }
}

/// Paths and typed evidence returned only after exact persistence reload and
/// unchanged-verifier acceptance.
#[derive(Debug)]
pub struct RecursiveCheckpointEvidenceV2 {
    pub sidecar: RecursiveCheckpointSidecarV2,
    pub receipt: CryptographicVerificationReceiptV2,
    pub envelope_path: PathBuf,
    pub sidecar_path: PathBuf,
    pub receipt_path: PathBuf,
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
    settlement_store: &'a mut SettlementStore,
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
        settlement_store: &'a mut SettlementStore,
        handoff: SettlementExecHandoff,
    ) -> Self {
        Self {
            transition_dir: transition_dir.into(),
            profile,
            checkpoint_store,
            prep_snapshot_store,
            checkpoint_id,
            settlement_store,
            handoff,
        }
    }
}

/// One fixed-layout local evidence store. It has no scheduler, retry queue,
/// provider SDK, or authority promotion API.
pub struct RecursiveCheckpointEvidenceStoreV2 {
    root: PathBuf,
}

impl RecursiveCheckpointEvidenceStoreV2 {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, CheckpointError> {
        let root = root.as_ref().to_path_buf();
        ensure_private_directory(&root)?;
        for child in ["envelopes", "sidecars", "receipts", "quarantine"] {
            ensure_private_directory(&root.join(child))?;
        }
        Ok(Self { root })
    }

    /// Execute the mandatory gate sequence: native evaluation, continuous
    /// fold, local verification, exact envelope persistence/reload/reverify,
    /// reference-only sidecar persistence/reload, final authority/snapshot
    /// check, then write-only receipt creation.
    pub fn produce(
        &self,
        blocks: &mut [RecursiveCheckpointChainBlockV2<'_>],
        prover_material_bytes: &[u8],
        verifier_bundle_bytes: &[u8],
        prior: RecursiveFinalizedIvcStateV2,
    ) -> Result<RecursiveCheckpointEvidenceV2, CheckpointError> {
        if blocks.len() < 3 {
            return Err(CheckpointError::RecursiveRejected(
                super::recursive_reject::RecursiveCheckpointRejectReasonV2::ChainTooShort,
            ));
        }
        if blocks.len() > 5 {
            return Err(CheckpointError::RecursiveRejected(
                super::recursive_reject::RecursiveCheckpointRejectReasonV2::ChainTooLong,
            ));
        }
        let stage = LiveGateStageV2::<GateStartV2>::start();
        let authority_result = (|| {
            let first = blocks.first().ok_or(CheckpointError::Invariant)?;
            let authority =
                RecursiveAuthoritySnapshotV2::resolve_active_authority(&*first.settlement_store)?;
            let authority_context = authority.authority();
            let profile = first.profile;
            for block in blocks.iter() {
                let candidate = RecursiveAuthoritySnapshotV2::resolve_active_authority(
                    &*block.settlement_store,
                )?;
                if candidate.authority() != authority_context || block.profile != profile {
                    return Err(CheckpointError::Authority);
                }
            }
            Ok((authority, profile))
        })();
        let (stage, (authority, profile)) = stage
            .authority_resolved(authority_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, verifier) = stage
            .family_selected(resolve_verifier_authority_v2(
                prover_material_bytes,
                verifier_bundle_bytes,
                authority,
                &profile,
            ))
            .map_err(LiveGateFailureV2::into_error)?;

        let transitions_result = blocks
            .iter_mut()
            .map(|block| {
                CanonicalCheckpointTransitionV2::from_exec_with_verifier(
                    &block.transition_dir,
                    profile,
                    block.checkpoint_store,
                    block.prep_snapshot_store,
                    block.checkpoint_id,
                    &mut *block.settlement_store,
                    block.handoff.clone(),
                    verifier,
                )
            })
            .collect::<Result<Vec<_>, _>>();
        let (stage, mut transitions) = stage
            .retain(transitions_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(revalidate_chain(&transitions, blocks))
            .map_err(LiveGateFailureV2::into_error)?;
        let mut nova_blocks = transitions
            .iter_mut()
            .zip(blocks.iter())
            .map(|(transition, block)| NovaChainTransitionV2 {
                transition,
                store: &*block.settlement_store,
            })
            .collect::<Vec<_>>();
        let (stage, verified) = stage
            .outer_bounded(prove_continuous_chain_v2(
                &mut nova_blocks,
                prover_material_bytes,
                verifier_bundle_bytes,
                prior,
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(verified.portable_input().validate_required_local_chain())
            .map_err(LiveGateFailureV2::into_error)?;
        drop(nova_blocks);
        let inner_result = if verified.framed_envelope().is_empty() {
            Err(CheckpointError::Canonical)
        } else {
            Ok(verified)
        };
        let (stage, verified) = stage
            .inner_bounded(inner_result)
            .map_err(LiveGateFailureV2::into_error)?;
        // The sealed verified value is created only after the Nova owner's
        // canonical curve decoder accepts the exact envelope.
        let (stage, verified) = stage
            .curve_valid(Ok(verified))
            .map_err(LiveGateFailureV2::into_error)?;
        let bundle_result = if verified.bundle_digest() == verifier.verifier_bundle_digest() {
            Ok(verified)
        } else {
            Err(CheckpointError::Authority)
        };
        let (stage, verified) = stage
            .bundle_matched(bundle_result)
            .map_err(LiveGateFailureV2::into_error)?;
        // Construction of `VerifiedNovaCheckpointV2` includes the unchanged
        // backend verifier call. Consuming it prevents replay across stages.
        let (stage, verified) = stage
            .backend_verified(Ok(verified))
            .map_err(LiveGateFailureV2::into_error)?;
        let bindings_result = verified.bindings().and_then(|bindings| {
            if bindings.final_state_limbs == 0 {
                Err(CheckpointError::Invariant)
            } else {
                Ok(bindings)
            }
        });
        let (stage, bindings) = stage
            .limbs_matched(bindings_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .bindings_matched(revalidate_chain(&transitions, blocks))
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
        let (stage, ()) = stage
            .endpoint_reloaded(revalidate_chain(&transitions, blocks))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .prewrite_complete(verified.verify_exact_bytes(verified.framed_envelope()))
            .map_err(LiveGateFailureV2::into_error)?;

        let envelope_path = self.object_path("envelopes", verified.envelope_digest());
        let (stage, envelope_created) = stage
            .retain(persist_content_addressed(
                &envelope_path,
                verified.framed_envelope(),
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(revalidate_or_quarantine(
                &transitions,
                blocks,
                &self.root,
                &[(&envelope_path, envelope_created)],
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, reloaded_envelope) = stage
            .retain(read_exact_bounded(&envelope_path, ENVELOPE_READ_CAP_V2))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(verified.verify_exact_bytes(&reloaded_envelope))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(revalidate_chain(&transitions, blocks))
            .map_err(LiveGateFailureV2::into_error)?;

        let (stage, sidecar) = stage
            .retain(RecursiveCheckpointSidecarV2::new(
                storage_generation,
                verified.envelope_digest(),
                verified.framed_envelope().len(),
                bindings,
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .retain(check_shadow_sidecar_binding(
                &sidecar,
                storage_generation,
                verified.envelope_digest(),
                verified.framed_envelope().len(),
                bindings,
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, sidecar_bytes) = stage
            .retain(RecursiveCheckpointSidecarCodecV2::encode_bin(&sidecar))
            .map_err(LiveGateFailureV2::into_error)?;
        let sidecar_digest = recursive_sidecar_digest(&sidecar_bytes);
        let sidecar_path = self.object_path("sidecars", sidecar_digest);
        let (stage, sidecar_created) = stage
            .atomic_write(persist_content_addressed(&sidecar_path, &sidecar_bytes))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, reloaded_sidecar_bytes) = stage
            .bytes_reloaded(read_exact_bounded(&sidecar_path, SIDECAR_READ_CAP_V2))
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, reloaded_sidecar) = stage
            .retain(RecursiveCheckpointSidecarCodecV2::decode_bin(
                &reloaded_sidecar_bytes,
            ))
            .map_err(LiveGateFailureV2::into_error)?;
        let reloaded_binding = (|| {
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
            revalidate_or_quarantine(
                &transitions,
                blocks,
                &self.root,
                &[
                    (&envelope_path, envelope_created),
                    (&sidecar_path, sidecar_created),
                ],
            )
        })();
        let (stage, ()) = stage
            .retain(reloaded_binding)
            .map_err(LiveGateFailureV2::into_error)?;
        let (stage, ()) = stage
            .post_backend_verified(verified.verify_exact_bytes(&reloaded_envelope))
            .map_err(LiveGateFailureV2::into_error)?;
        let post_endpoint = (|| {
            check_shadow_sidecar_binding(
                &reloaded_sidecar,
                storage_generation,
                verified.envelope_digest(),
                verified.framed_envelope().len(),
                bindings,
            )?;
            revalidate_chain(&transitions, blocks)
        })();
        let (stage, ()) = stage
            .post_endpoint_matched(post_endpoint)
            .map_err(LiveGateFailureV2::into_error)?;

        let postwrite = PostwriteVerifiedV2::new(
            stage,
            storage_generation,
            verified.envelope_digest(),
            sidecar_digest,
            bindings,
        )
        .map_err(LiveGateFailureV2::into_error)?;
        let prepared_result = CryptographicVerificationReceiptV2::prepare_postwrite(&postwrite);
        let (postwrite, prepared) = postwrite
            .retain(prepared_result)
            .map_err(LiveGateFailureV2::into_error)?;
        let receipt_bytes = prepared.canonical_bytes().to_vec();
        let receipt_digest = recursive_receipt_digest(&receipt_bytes);
        let receipt_path = self.object_path("receipts", receipt_digest);
        let (postwrite, receipt_created) = postwrite
            .retain(persist_content_addressed(&receipt_path, &receipt_bytes))
            .map_err(LiveGateFailureV2::into_error)?;
        // Receipts have no decoder. Reload is byte equality only and can never
        // promote evidence or select a proof/config path.
        let (postwrite, reloaded_receipt) = postwrite
            .retain(read_exact_bounded(&receipt_path, RECEIPT_READ_CAP_V2))
            .map_err(LiveGateFailureV2::into_error)?;
        let exact_reload = if reloaded_receipt == receipt_bytes {
            Ok(())
        } else {
            Err(CheckpointError::Canonical)
        };
        let (postwrite, ()) = postwrite
            .retain(exact_reload)
            .map_err(LiveGateFailureV2::into_error)?;
        let (postwrite, ()) = postwrite
            .retain(revalidate_or_quarantine(
                &transitions,
                blocks,
                &self.root,
                &[
                    (&envelope_path, envelope_created),
                    (&sidecar_path, sidecar_created),
                    (&receipt_path, receipt_created),
                ],
            ))
            .map_err(LiveGateFailureV2::into_error)?;

        // This is deliberately the final operation and the only edge that
        // appends `ReceiptIssued` or constructs a public receipt. The call
        // release-checks the complete prepared/postwrite binding pair.
        let receipt = CryptographicVerificationReceiptV2::issue_postwrite(postwrite, prepared)?;

        Ok(RecursiveCheckpointEvidenceV2 {
            sidecar,
            receipt,
            envelope_path,
            sidecar_path,
            receipt_path,
        })
    }

    fn object_path(&self, class: &str, digest: [u8; 32]) -> PathBuf {
        self.root
            .join(class)
            .join(format!("{}.bin", lowercase_hex(digest)))
    }
}

fn revalidate_chain(
    transitions: &[CanonicalCheckpointTransitionV2],
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
) -> Result<(), CheckpointError> {
    if transitions.len() != blocks.len() {
        return Err(CheckpointError::Invariant);
    }
    for (transition, block) in transitions.iter().zip(blocks) {
        transition.revalidate_evidence_authority(&*block.settlement_store)?;
    }
    Ok(())
}

fn revalidate_or_quarantine(
    transitions: &[CanonicalCheckpointTransitionV2],
    blocks: &[RecursiveCheckpointChainBlockV2<'_>],
    root: &Path,
    written: &[(&Path, bool)],
) -> Result<(), CheckpointError> {
    if let Err(error) = revalidate_chain(transitions, blocks) {
        for (path, created) in written {
            if *created {
                quarantine_written(root, path)?;
            }
        }
        return Err(error);
    }
    Ok(())
}

fn quarantine_written(root: &Path, path: &Path) -> Result<(), CheckpointError> {
    const MAX_QUARANTINE_OBJECTS_V2: usize = 64;
    let quarantine = root.join("quarantine");
    let count = read_dir(&quarantine)
        .map_err(|_| CheckpointError::Storage)?
        .into_iter()
        .take(MAX_QUARANTINE_OBJECTS_V2 + 1)
        .count();
    if count >= MAX_QUARANTINE_OBJECTS_V2 {
        return Err(CheckpointError::Limit);
    }
    let class = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .ok_or(CheckpointError::Storage)?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(CheckpointError::Storage)?;
    let destination = quarantine.join(format!("{class}-{name}"));
    if path_exists_no_follow(&destination).map_err(|_| CheckpointError::Storage)? {
        return Err(CheckpointError::Canonical);
    }
    rename_file(path, &destination).map_err(|_| CheckpointError::Storage)?;
    sync_directory(&quarantine).map_err(|_| CheckpointError::Storage)
}

fn ensure_private_directory(path: &Path) -> Result<(), CheckpointError> {
    if path_exists_no_follow(path).map_err(|_| CheckpointError::Storage)? {
        let metadata = symlink_metadata(path).map_err(|_| CheckpointError::Storage)?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return Err(CheckpointError::Storage);
        }
    } else {
        create_dir_all(path).map_err(|_| CheckpointError::Storage)?;
    }
    #[cfg(unix)]
    set_permissions_mode(path, 0o700).map_err(|_| CheckpointError::Storage)?;
    Ok(())
}

fn persist_content_addressed(path: &Path, bytes: &[u8]) -> Result<bool, CheckpointError> {
    let parent = path.parent().ok_or(CheckpointError::Storage)?;
    if path_exists_no_follow(path).map_err(|_| CheckpointError::Storage)? {
        if read_exact_bounded(path, bytes.len() as u64)? == bytes {
            return Ok(false);
        }
        return Err(CheckpointError::Canonical);
    }
    let mut temporary = tempfile::Builder::new()
        .prefix(".z00z-recursive-evidence-")
        .tempfile_in(parent)
        .map_err(|_| CheckpointError::Storage)?;
    set_file_mode(temporary.as_file_mut(), 0o600).map_err(|_| CheckpointError::Storage)?;
    temporary
        .as_file_mut()
        .write_all(bytes)
        .and_then(|()| temporary.as_file_mut().sync_all())
        .map_err(|_| CheckpointError::Storage)?;
    if temporary.persist_noclobber(path).is_err() {
        if path_exists_no_follow(path).map_err(|_| CheckpointError::Storage)?
            && read_exact_bounded(path, bytes.len() as u64)? == bytes
        {
            return Ok(false);
        }
        return Err(CheckpointError::Canonical);
    }
    sync_directory(parent).map_err(|_| CheckpointError::Storage)?;
    Ok(true)
}

fn read_exact_bounded(path: &Path, cap: u64) -> Result<Vec<u8>, CheckpointError> {
    let metadata = symlink_metadata(path).map_err(|_| CheckpointError::Storage)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() > cap
    {
        return Err(CheckpointError::Storage);
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| CheckpointError::Storage)?
        .take(cap.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| CheckpointError::Storage)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > cap {
        return Err(CheckpointError::Storage);
    }
    Ok(bytes)
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
            nova::{CheckpointNovaRunnerV2, NovaRunnerFoldingV2, NovaRunnerReadyV2},
            CheckpointDraft, CheckpointExecInput, CheckpointFsStore, CheckpointStore,
            CheckpointVersion,
        },
        fixture_support::checkpoint_fixtures,
        settlement::SettlementStateRoot,
        snapshot::{build_snapshot_v2, PrepFsStore, PrepSnapshotStore},
    };

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
        read_exact_bounded(&root.join(name), cap).expect("read bounded retained T3 artifact")
    }

    #[test]
    fn test_noop_links_stay_linear() {
        let chain = tempfile::tempdir().expect("no-op chain root");
        let store = SettlementStore::new();
        let settlement_root = store.settlement_root_v2(7).expect("no-op chain root");
        let prior = RecursiveFinalizedIvcStateV2::cutover(
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
        let path = store.object_path("sidecars", [7; 32]);
        persist_content_addressed(&path, b"canonical").unwrap();
        persist_content_addressed(&path, b"canonical").unwrap();
        assert_eq!(read_exact_bounded(&path, 64).unwrap(), b"canonical");
        assert!(persist_content_addressed(&path, b"different").is_err());
        assert!(read_dir(&root).unwrap().into_iter().all(|entry| entry
            .file_name()
            .map(|name| !name.to_string_lossy().starts_with(".z00z"))
            .unwrap_or(true)));
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
    fn test_receipt_gate_is_last() {
        let postwrite = postwrite_token([7; 32]);
        assert_eq!(
            postwrite.gate_ids().last(),
            Some(&(LiveGateIdV2::PostwriteEndpointMatched as u8))
        );
        assert_eq!(postwrite.gate_ids().len(), 15);
        let prepared = CryptographicVerificationReceiptV2::prepare_postwrite(&postwrite).unwrap();
        let prepared_bytes = prepared.canonical_bytes().to_vec();
        let (postwrite, ()) = postwrite
            .retain(Ok(()))
            .map_err(LiveGateFailureV2::into_error)
            .unwrap();
        let receipt =
            CryptographicVerificationReceiptV2::issue_postwrite(postwrite, prepared).unwrap();
        assert_eq!(receipt.canonical_bytes(), prepared_bytes);
        assert_eq!(
            receipt.result(),
            super::super::receipt::RecursiveVerificationResultV2::VerifiedExactReload
        );
    }

    #[test]
    fn test_receipt_issuance_rejects_mismatched_prepared_pair() {
        let postwrite = postwrite_token([7; 32]);
        let mut prepared =
            CryptographicVerificationReceiptV2::prepare_postwrite(&postwrite).unwrap();
        prepared.corrupt_wire_for_test();
        assert!(matches!(
            CryptographicVerificationReceiptV2::issue_postwrite(postwrite, prepared),
            Err(CheckpointError::Invariant)
        ));

        let postwrite = postwrite_token([7; 32]);
        let mut prepared =
            CryptographicVerificationReceiptV2::prepare_postwrite(&postwrite).unwrap();
        prepared.corrupt_bytes_for_test();
        assert!(matches!(
            CryptographicVerificationReceiptV2::issue_postwrite(postwrite, prepared),
            Err(CheckpointError::Invariant)
        ));
    }

    #[test]
    fn test_receipt_failpoints_keep_prefix() {
        #[derive(Clone, Copy)]
        enum ReceiptFailpointV2 {
            Encode,
            Write,
            Reload,
            Quarantine,
            AuthorityDrift,
        }
        let failpoints = [
            ReceiptFailpointV2::Encode,
            ReceiptFailpointV2::Write,
            ReceiptFailpointV2::Reload,
            ReceiptFailpointV2::Quarantine,
            ReceiptFailpointV2::AuthorityDrift,
        ];
        let expected = SUCCESSFUL_GATE_SEQUENCE_V2[..15]
            .iter()
            .map(|gate| *gate as u8)
            .collect::<Vec<_>>();

        for failpoint in failpoints {
            let name = match failpoint {
                ReceiptFailpointV2::Encode => "encode",
                ReceiptFailpointV2::Write => "write",
                ReceiptFailpointV2::Reload => "reload",
                ReceiptFailpointV2::Quarantine => "quarantine",
                ReceiptFailpointV2::AuthorityDrift => "authority-drift",
            };
            let mut bindings = receipt_bindings();
            if matches!(failpoint, ReceiptFailpointV2::Encode) {
                bindings.steps = 0;
            }
            let postwrite = postwrite_token_with([15; 32], bindings);
            let temp = tempfile::tempdir().unwrap();
            let result = match failpoint {
                ReceiptFailpointV2::Encode => {
                    CryptographicVerificationReceiptV2::prepare_postwrite(&postwrite).map(|_| ())
                }
                ReceiptFailpointV2::Write => persist_content_addressed(
                    &temp.path().join("missing").join("receipt.bin"),
                    b"prepared-receipt",
                )
                .map(|_| ()),
                ReceiptFailpointV2::Reload => {
                    read_exact_bounded(&temp.path().join("missing.bin"), 64).map(|_| ())
                }
                ReceiptFailpointV2::Quarantine => {
                    let root = temp.path().join("evidence");
                    let _store = RecursiveCheckpointEvidenceStoreV2::open(&root).unwrap();
                    quarantine_written(&root, &root.join("receipts").join("missing.bin"))
                }
                ReceiptFailpointV2::AuthorityDrift => Err(CheckpointError::Authority),
            };
            let failure = match postwrite.retain(result) {
                Ok(_) => panic!("receipt failpoint advanced: {name}"),
                Err(failure) => failure,
            };
            assert_eq!(failure.ids(), expected, "wrong prefix at {name}");
            assert!(!failure.ids().contains(&(LiveGateIdV2::ReceiptIssued as u8)));
            let terminal_matches = match failpoint {
                ReceiptFailpointV2::Encode => {
                    matches!(failure.terminal(), CheckpointError::Invariant)
                }
                ReceiptFailpointV2::Write
                | ReceiptFailpointV2::Reload
                | ReceiptFailpointV2::Quarantine => {
                    matches!(failure.terminal(), CheckpointError::Storage)
                }
                ReceiptFailpointV2::AuthorityDrift => {
                    matches!(failure.terminal(), CheckpointError::Authority)
                }
            };
            assert!(terminal_matches, "wrong terminal at {name}");
        }
    }

    #[test]
    #[ignore = "real continuous 1/3/5-block Nova ingress is milestone-only; run nova_milestone_tests.sh t3-chain"]
    fn test_real_chain_public_receipt() {
        crate::fixture_support::genesis_chain_identity::ensure_test_process_chain_identity()
            .expect("validated canonical devnet genesis identity");
        let material = retained_t3_artifact("prover-material.bin", 1024 * 1024 * 1024);
        // Mirror the production Nova verifier-bundle safety ceiling. This
        // test-only file intake stays bounded while accommodating the current
        // canonical 83 MB compressed VK artifact.
        let bundle = retained_t3_artifact("verifier-bundle.bin", 384 * 1024 * 1024 + 1024);
        let profile = RecursiveCircuitProfileV2::authority_pinned();
        let chain = tempfile::tempdir().expect("continuous T3 chain root");
        let mut store = SettlementStore::new();
        let authority = RecursiveAuthoritySnapshotV2::resolve_active_authority(&store)
            .expect("T3 repository authority");
        let verifier = resolve_verifier_authority_v2(&material, &bundle, authority, &profile)
            .expect("T3 material and bundle authority");
        let settlement_root = store.settlement_root_v2(7).expect("T3 settlement root");
        let mut prior = RecursiveFinalizedIvcStateV2::cutover(
            CheckpointId::new([0x41; 32]),
            *settlement_root.as_bytes(),
            store.recursive_v2_definition_root(),
            [0x42; 32],
        )
        .expect("T3 cutover z0");
        let mut runner: Option<CheckpointNovaRunnerV2<NovaRunnerFoldingV2>> = None;
        let mut snapshots = Vec::new();
        let mut initial_state_digest = None;

        for height in 1_u64..=5 {
            let (checkpoint_store, prep_store, checkpoint_id) =
                seal_noop_checkpoint(chain.path(), height, settlement_root, prior);
            let transition_dir = chain.path().join(format!("transition-{height}"));
            create_dir_all(&transition_dir).expect("create isolated T3 transition directory");
            let mut transition = CanonicalCheckpointTransitionV2::from_exec_with_verifier(
                &transition_dir,
                profile,
                &checkpoint_store,
                &prep_store,
                checkpoint_id,
                &mut store,
                SettlementExecHandoff::recursive_v2_noop(),
                verifier,
            )
            .expect("construct bundle-bound T3 transition");
            let evaluated = transition.evaluate(&store).expect("evaluate T3 transition");
            if let Some(active) = runner.as_mut() {
                active
                    .advance_transition(&transition, evaluated)
                    .expect("advance the same T3 accumulator");
                transition
                    .replay_nova_events(&store, |event| active.push_event(event))
                    .expect("replay next T3 block into the same accumulator");
            } else {
                let ready = CheckpointNovaRunnerV2::<NovaRunnerReadyV2>::load_for_transition(
                    &material,
                    &bundle,
                    &transition,
                    evaluated,
                    prior,
                )
                .expect("load the sole T3 runner from z0");
                let mut ready = Some(ready);
                let mut folding: Option<CheckpointNovaRunnerV2<NovaRunnerFoldingV2>> = None;
                transition
                    .replay_nova_events(&store, |event| {
                        if let Some(active) = folding.as_mut() {
                            active.push_event(event)
                        } else {
                            folding = Some(
                                ready
                                    .take()
                                    .ok_or(CheckpointError::TraceState)?
                                    .first_event(event)?,
                            );
                            Ok(())
                        }
                    })
                    .expect("replay first T3 block from z0");
                runner = folding;
            }
            let active = runner.as_mut().expect("T3 runner entered folding state");
            prior = active
                .finish_block(false)
                .expect("finish one nonterminal T3 block");
            transition.finish(&store).expect("finish T3 trace source");

            if matches!(height, 1 | 3 | 5) {
                let snapshot = active.snapshot().expect("real non-consuming T3 snapshot");
                snapshot
                    .verify_exact_bytes(snapshot.framed_envelope())
                    .expect("unchanged verifier accepts exact T3 snapshot bytes");
                let bindings = snapshot.bindings().expect("T3 snapshot bindings");
                assert_eq!(bindings.height, height);
                assert_eq!(bindings.steps, prior.cumulative_steps());
                match initial_state_digest {
                    Some(expected) => assert_eq!(bindings.initial_state_digest, expected),
                    None => initial_state_digest = Some(bindings.initial_state_digest),
                }
                if height == 1 {
                    let repeated = active
                        .snapshot()
                        .expect("second consecutive snapshot is non-consuming");
                    let repeated_bindings = repeated.bindings().expect("repeated bindings");
                    assert_eq!(
                        bindings.initial_state_digest,
                        repeated_bindings.initial_state_digest
                    );
                    assert_eq!(
                        bindings.final_state_digest,
                        repeated_bindings.final_state_digest
                    );
                    assert_eq!(bindings.steps, repeated_bindings.steps);
                }
                snapshots.push((height, bindings.steps, snapshot.framed_envelope().len()));
            }
        }
        assert_eq!(snapshots.len(), 3);
        assert!(initial_state_digest.is_some());
        eprintln!("real continuous same-z0 T3 snapshots: {snapshots:?}");

        // Exercise the public production ingress and write-only receipt on a
        // fresh one-block chain. The raw block carries no prebuilt transition
        // and therefore cannot inject a transition-only verifier digest.
        let ingress = tempfile::tempdir().expect("T3 public ingress root");
        let evidence_root = ingress.path().join("evidence");
        let checkpoint_root = ingress.path().join("checkpoint");
        create_dir_all(&checkpoint_root).expect("create public ingress checkpoint root");
        let mut ingress_store = SettlementStore::new();
        let ingress_root = ingress_store
            .settlement_root_v2(7)
            .expect("public ingress settlement root");
        let ingress_prior = RecursiveFinalizedIvcStateV2::cutover(
            CheckpointId::new([0x51; 32]),
            *ingress_root.as_bytes(),
            ingress_store.recursive_v2_definition_root(),
            [0x52; 32],
        )
        .expect("public ingress cutover");
        let (checkpoint_store, prep_store, checkpoint_id) =
            seal_noop_checkpoint(&checkpoint_root, 1, ingress_root, ingress_prior);
        let transition_dir = ingress.path().join("transition");
        create_dir_all(&transition_dir).expect("create public ingress transition root");
        let mut blocks = [RecursiveCheckpointChainBlockV2::new(
            transition_dir,
            profile,
            &checkpoint_store,
            &prep_store,
            checkpoint_id,
            &mut ingress_store,
            SettlementExecHandoff::recursive_v2_noop(),
        )];
        let evidence_store = RecursiveCheckpointEvidenceStoreV2::open(&evidence_root)
            .expect("open public T3 evidence store");
        let evidence = evidence_store
            .produce(&mut blocks, &material, &bundle, ingress_prior)
            .expect("public ingress issues receipt only after exact reload verification");
        assert_eq!(evidence.receipt.height(), 1);
        assert!(evidence.envelope_path.is_file());
        assert!(evidence.sidecar_path.is_file());
        assert!(evidence.receipt_path.is_file());
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
