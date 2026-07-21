//! Canonical checkpoint storage surface.
//!
//! Consensus-facing callers should use the top-level types re-exported from this module:
//! drafts, final artifacts, links, typed execution inputs, ids, and the storage facade.
//! Replay and audit-only data stays under the narrower [`audit`] submodule so wrapper-local
//! fields do not leak into the main checkpoint contract by default.
//!
//! Authority boundaries for the current Phase 069 candidate source:
//! - `z00z_storage::checkpoint` owns the inherited canonical statement/final-artifact bytes,
//!   replay links, path resolution, reject taxonomy, and the sole recursive V2 facade.
//! - Recursive sidecars and write-only receipts remain shadow evidence; PQ anchors, DA
//!   references, and publication evidence may bind the statement but cannot replace canonical
//!   checkpoint admission.
//! - Validators consume storage-owned checkpoint artifacts, while watchers remain advisory.
//! - Provider and transport integrations stay behind adapter boundaries and cannot redefine the
//!   storage-owned theorem or select a proof decoder.
//!
//! # Examples
//!
//! ```
//! use z00z_storage::checkpoint::{CheckpointId, CheckpointLinkVersion};
//!
//! let checkpoint_id = CheckpointId::new([7u8; 32]);
//! assert_eq!(checkpoint_id.as_bytes(), &[7u8; 32]);
//! assert_eq!(CheckpointLinkVersion::CURRENT.as_u8(), 1);
//! ```

mod adapter;
mod archive_manifest;
mod archive_receipt;
mod artifact_final;
mod artifact_proof_draft;
mod artifact_stmt;
mod artifact_types;
pub mod audit;
mod authority_artifacts;
mod build;
mod build_prepare;
mod build_state;
mod canonical_transition;
mod codec;
mod contract_config;
mod contract_config_v3;
mod da_reference;
mod exec_input;
mod ids;
mod lifecycle;
mod link;
pub(crate) mod nova;
mod pq_anchor;
mod pruning;
mod publication_evidence;
mod receipt;
mod recursive_circuit;
mod recursive_context;
mod recursive_encoding;
mod recursive_predicate;
pub(crate) mod recursive_reject;
mod recursive_semantics;
mod recursive_statement;
mod recursive_trace;
pub mod recursive_v2;
mod retrieval_audit;
mod sidecar;
mod state_snapshot;
mod store;
mod store_fs;
#[cfg(test)]
mod test_checkpoint;
#[cfg(test)]
mod test_store;
mod version_registry;

pub(crate) use recursive_circuit::RECURSIVE_HJMT_SEGMENT_BYTES_V2;

pub use self::{
    archive_manifest::{
        ArchiveManifestVersion, CheckpointArchiveEncodingKindV1, CheckpointArchiveEntryKindV1,
        CheckpointArchiveEntryV1, CheckpointArchiveEntryVersion, CheckpointArchiveManifestV1,
        CheckpointArchiveRetentionClassV1,
    },
    archive_receipt::{ArchiveBackend, ArchiveProviderReceiptV1, ArchiveProviderReceiptVersion},
    artifact_final::CheckpointArtifact,
    artifact_proof_draft::{CheckpointDraft, CheckpointProof},
    artifact_stmt::{
        CheckpointStatement, CheckpointTransitionStatementCoreV1,
        CheckpointTransitionStatementFinalV1, CheckpointTransitionStatementV1, WalletDraft,
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN,
    },
    artifact_types::{
        CheckpointProofSystem, CheckpointPubIn, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    build::{
        apply_batch_checkpoint, build_cp_draft, build_stmt_core_v1, derive_delta_root_v1,
        InputResolver, MemberIndex, MemberWit, ResolvedInput, SettlementState, SpentIndex,
        SpentIndexError, StateError, TxPkgSum, TxProofError, TxProofVerifier,
    },
    build_prepare::prepare_tx_sum,
    codec::{
        decode_archive_manifest_bin, decode_archive_manifest_json, decode_archive_receipt_bin,
        decode_archive_receipt_json, decode_art_bin, decode_art_json, decode_da_reference_bin,
        decode_da_reference_json, decode_draft_bin, decode_draft_json, decode_exec_bin,
        decode_exec_json, decode_link_bin, decode_link_json, decode_pq_anchor_bin,
        decode_pq_anchor_json, decode_pruning_decision_bin, decode_pruning_decision_json,
        decode_publication_evidence_bin, decode_publication_evidence_json,
        decode_retrieval_audit_bin, decode_retrieval_audit_json, decode_state_snapshot_bin,
        decode_state_snapshot_json, encode_archive_manifest_bin, encode_archive_manifest_json,
        encode_archive_receipt_bin, encode_archive_receipt_json, encode_art_bin, encode_art_json,
        encode_da_reference_bin, encode_da_reference_json, encode_draft_bin, encode_draft_json,
        encode_exec_bin, encode_exec_json, encode_link_bin, encode_link_json, encode_pq_anchor_bin,
        encode_pq_anchor_json, encode_pruning_decision_bin, encode_pruning_decision_json,
        encode_publication_evidence_bin, encode_publication_evidence_json,
        encode_retrieval_audit_bin, encode_retrieval_audit_json, encode_state_snapshot_bin,
        encode_state_snapshot_json, guard_verified_backend_codec_support,
    },
    contract_config::{
        repo_default_path, ArchiveRetentionCfg, AuthorityPromotionCfg, CheckpointContractConfigV2,
        CheckpointContractLimits, CheckpointContractPaths, CheckpointResolvedPaths, PruningCfg,
        SnapshotsCfg, VerifiedBackendCfg, VerifiedBackendChainEvidenceCfg,
        VerifiedBackendRollbackCfg, VerifiedBackendSecurityReviewCfg,
        AUTHORITY_PROMOTION_STAGE_CONFIG_GATE, AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT,
        AUTHORITY_PROMOTION_STAGE_SPEC_ONLY, CHECKPOINT_CONTRACT_CONFIG_PATH,
        POST_QUANTUM_ENFORCEMENT_STAGE, POST_QUANTUM_MODE, POST_QUANTUM_REQUIRED_ARTIFACTS_V2,
        VERIFIED_BACKEND_ADAPTER_TRAIT, VERIFIED_BACKEND_CANDIDATE_STAGE,
        VERIFIED_BACKEND_CHAIN_EVIDENCE_OBJECT, VERIFIED_BACKEND_CODEC_SUPPORT,
        VERIFIED_BACKEND_ENABLED_STAGE, VERIFIED_BACKEND_PROOF_OBJECT,
        VERIFIED_BACKEND_REQUIRED_BENCHMARKS, VERIFIED_BACKEND_REQUIRED_NEGATIVE_TESTS,
        VERIFIED_BACKEND_REVIEW_APPROVED, VERIFIED_BACKEND_REVIEW_PENDING,
        VERIFIED_BACKEND_ROLLBACK_PROCEDURE, VERIFIED_BACKEND_STATEMENT_STABILITY,
        VERIFIED_BACKEND_VERIFIER_API,
    },
    contract_config_v3::{
        ActiveCheckpointConfigIdentityV3, ActiveCheckpointConfigV3, BranchesCfgV3,
        CheckpointConfigHeadV3, CheckpointConfigResolverV3, CheckpointContractConfigV3,
        ConfigFieldTransformV3, ConfigMigrationRecordV3, ConfigV3ActivationStore,
        ConfigV3RenameEntry, ConfigV3RenameLedger, NovaBranchCfgV3, OfflineReceiptMailboxCfgV3,
        Plonky3EpochBranchCfgV3, PostQuantumCfgV3, RuntimeProfileCfgV3, VersionAuthorityCfgV3,
        CONFIG_V3_NEWLINE_POLICY, CONFIG_V3_PQ_MODE, CONFIG_V3_PROFILE,
        CONFIG_V3_TRANSFORM_VERSION, POST_QUANTUM_REQUIRED_ARTIFACTS_V3,
    },
    da_reference::{
        CheckpointDaLocatorKind, CheckpointDaProviderFamily, CheckpointDaReferenceV1,
        CheckpointDaReferenceVersion,
    },
    exec_input::{
        derive_exec_tx_root, CheckpointExecInput, CheckpointExecOut, CheckpointExecTx,
        CheckpointExecVersion, CheckpointInRef,
    },
    ids::{
        derive_checkpoint_id, derive_draft_id, derive_exec_id, reject_draft_for_checkpoint_id,
        CheckpointDraftId, CheckpointExecInputId, CheckpointId,
    },
    lifecycle::{CheckpointLifecycleStatus, CheckpointLifecycleV1, CheckpointLifecycleVersion},
    link::{CheckpointLink, CheckpointLinkVersion},
    pq_anchor::{
        PostQuantumCheckpointAnchorModeV1, PostQuantumCheckpointAnchorRejectReasonV1,
        PostQuantumCheckpointAnchorV1, PostQuantumCheckpointAnchorVersion,
        PostQuantumCheckpointEnforcementStageV1,
    },
    pruning::{PruningDecisionV1, PruningDecisionVersion, PruningNodeClass},
    publication_evidence::{
        CheckpointPublicationEvidenceV1, CheckpointPublicationEvidenceVersion,
        CheckpointPublicationState,
    },
    retrieval_audit::{RetrievalAuditV1, RetrievalAuditVersion},
    state_snapshot::{StateSnapshotV1, StateSnapshotVersion},
    store::{
        check_art_key, check_draft_key, check_exec_key, check_exec_root, check_link_ids,
        load_artifact, load_draft, CheckpointFsStore, CheckpointStore,
    },
};
