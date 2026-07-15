use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use z00z_utils::io::load_yaml_bounded;

use crate::CheckpointError;

use super::pq_anchor::{
    PostQuantumCheckpointAnchorModeV1, PostQuantumCheckpointAnchorV1,
    PostQuantumCheckpointAnchorVersion, PostQuantumCheckpointEnforcementStageV1,
};

pub const CHECKPOINT_CONTRACT_CONFIG_PATH: &str =
    "crates/z00z_storage/src/checkpoint/checkpoint_contract.yaml";
pub const AUTHORITY_PROMOTION_STAGE_SPEC_ONLY: &str = "spec_only";
pub const AUTHORITY_PROMOTION_STAGE_CONFIG_GATE: &str = "config_gate";
pub const AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT: &str = "canonical_extended_statement";
pub const POST_QUANTUM_MODE: &str = "plonky3_epoch_proof";
pub const POST_QUANTUM_ENFORCEMENT_STAGE: &str = "pq_anchor_writer";
pub const VERIFIED_BACKEND_CANDIDATE_STAGE: &str = "verified_backend_candidate";
pub const VERIFIED_BACKEND_ENABLED_STAGE: &str = "verified_backend_enabled";
pub const VERIFIED_BACKEND_PROOF_OBJECT: &str = "VerifiedCheckpointProofV2";
pub const VERIFIED_BACKEND_VERIFIER_API: &str = "VerifiedCheckpointVerifierV2";
pub const VERIFIED_BACKEND_CODEC_SUPPORT: &str = "VerifiedCheckpointArtifactV2";
pub const VERIFIED_BACKEND_ADAPTER_TRAIT: &str = "VerifiedCheckpointBackendAdapterV2";
pub const VERIFIED_BACKEND_CHAIN_EVIDENCE_OBJECT: &str = "RecursiveCheckpointChainEvidenceV2";
pub const VERIFIED_BACKEND_ROLLBACK_PROCEDURE: &str =
    "disable_verified_backend_without_statement_change";
pub const VERIFIED_BACKEND_STATEMENT_STABILITY: &str = "CheckpointTransitionStatementV1";
pub const VERIFIED_BACKEND_REVIEW_PENDING: &str = "pending";
pub const VERIFIED_BACKEND_REVIEW_APPROVED: &str = "approved";
/// V2 writes only the non-authenticating evidence-commitment field name.
pub const POST_QUANTUM_REQUIRED_ARTIFACTS: [&str; 9] = [
    "pq_statement_digest",
    "pq_delta_root",
    "pq_witness_root",
    "pq_archive_manifest_root",
    "plonky3_epoch_statement_digest",
    "plonky3_epoch_proof_digest",
    "plonky3_public_inputs_digest",
    "nova_chain_root",
    "epoch_evidence_commitment",
];
pub const VERIFIED_BACKEND_REQUIRED_NEGATIVE_TESTS: [&str; 7] = [
    "wrong_root",
    "wrong_delta",
    "wrong_witness",
    "wrong_proof",
    "wrong_link",
    "unsupported_backend",
    "mixed_era",
];
pub const VERIFIED_BACKEND_REQUIRED_BENCHMARKS: [&str; 5] = [
    "proof_size",
    "prover_time",
    "verifier_time",
    "memory",
    "witness_size",
];
const CHECKPOINT_CONTRACT_CONFIG_MAX_BYTES: u64 = 256 * 1024;
const CHECKPOINT_CONTRACT_MAX_OBJECT_BYTES: usize = 256 * 1024 * 1024;

const fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContractConfigV1 {
    pub version: u32,
    pub profile: String,
    pub architecture_mode: String,
    pub statement: StatementCfg,
    pub branches: BranchesCfg,
    pub authority_promotion: AuthorityPromotionCfg,
    pub verified_backend: VerifiedBackendCfg,
    pub gates: GatesCfg,
    pub da: DaCfg,
    pub archive_retention: ArchiveRetentionCfg,
    pub post_quantum: PostQuantumCfg,
    pub snapshots: SnapshotsCfg,
    pub pruning: PruningCfg,
    pub retention: RetentionCfg,
    pub paths: CheckpointContractPaths,
    pub limits: CheckpointContractLimits,
    pub documentation: DocumentationCfg,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatementCfg {
    pub version: u32,
    pub domain: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BranchesCfg {
    pub canonical: CanonicalBranchCfg,
    pub recursive: RecursiveBranchCfg,
    pub nova: NovaBranchCfg,
    pub plonky3_epoch: Plonky3EpochBranchCfg,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalBranchCfg {
    pub is_enabled: bool,
    pub is_authoritative: bool,
    pub proof_system: String,
    pub has_exact_tx_proof_bytes: bool,
    pub has_checkpoint_link: bool,
    pub has_replay_ids: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveBranchCfg {
    pub is_enabled: bool,
    pub is_authoritative: bool,
    pub mode: String,
    pub proof_system: String,
    pub has_prior_output_binding: bool,
    pub min_chain_steps: u32,
    pub target_chain_steps: u32,
    pub no_op: RecursiveNoopCfg,
}

/// Authority-pinned schema for the sole legal recursive V2 empty transition.
///
/// It is intentionally not a generic execution switch: the exact input
/// version, empty-handoff mode, and root-preservation rule are all part of the
/// checkpoint-contract digest consumed by the recursive authority.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveNoopCfg {
    pub is_enabled: bool,
    pub execution_input_version: u8,
    pub mode: String,
    pub requires_empty_handoff: bool,
    pub preserves_settlement_root: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NovaBranchCfg {
    pub is_enabled: bool,
    pub is_authoritative: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_pq_authoritative: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub is_available: bool,
    #[serde(default)]
    pub fold_cadence_blocks: u64,
    #[serde(default)]
    pub compressed_proof_snapshot_cadence_blocks: u64,
    pub mode: String,
    pub proof_system: String,
    pub has_prior_output_binding: bool,
    pub must_bind_statement_digest: bool,
    pub must_bind_checkpoint_link: bool,
    pub retain_until_pq_epoch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plonky3EpochBranchCfg {
    pub is_enabled: bool,
    pub cadence_blocks: u64,
    pub is_authoritative: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_pq_authoritative: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub is_available: bool,
    #[serde(default)]
    pub provides_pq_epoch_evidence: bool,
    pub mode: String,
    pub proof_system: String,
    pub must_prove_canonical_transition_range: bool,
    pub may_bind_nova_chain_root: bool,
    pub must_not_depend_only_on_nova: bool,
    pub field: String,
    pub hash: String,
    pub security_bits: u16,
    pub recursion_library: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPromotionCfg {
    pub stage: String,
    pub recursive_authority_allowed: bool,
    pub verified_backend_allowed: bool,
    pub allowed_next_stages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedBackendCfg {
    pub proof_object: String,
    pub verifier_api: String,
    pub codec_support: String,
    pub adapter_trait: String,
    pub chain_evidence: VerifiedBackendChainEvidenceCfg,
    pub negative_tests: Vec<String>,
    pub benchmarks: Vec<String>,
    pub rollback: VerifiedBackendRollbackCfg,
    pub security_review: VerifiedBackendSecurityReviewCfg,
    pub statement_stability: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedBackendChainEvidenceCfg {
    pub object: String,
    pub min_steps: u32,
    pub max_steps: u32,
    pub requires_prior_output_binding: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedBackendRollbackCfg {
    pub procedure: String,
    pub preserves_statement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedBackendSecurityReviewCfg {
    pub status: String,
    pub requires_third_party_equivalent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatesCfg {
    pub inputs: InputGatesCfg,
    pub outputs: OutputGatesCfg,
    pub artifacts: ArtifactGatesCfg,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputGatesCfg {
    pub has_statement_fields: bool,
    pub has_exec_input_id: bool,
    pub has_prep_snapshot_id: bool,
    pub has_da_ref: bool,
    pub has_exact_tx_proof_bytes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutputGatesCfg {
    pub has_checkpoint_artifact: bool,
    pub has_checkpoint_link: bool,
    pub has_da_export: bool,
    pub has_archive_manifest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactGatesCfg {
    pub has_pq_anchor_on_cadence: bool,
    pub has_mixed_era_fail_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DaCfg {
    pub provider_sdk_boundary: String,
    pub publication_readiness_gate: String,
    pub challenge_window_start: String,
    pub allowed_sync_modes: Vec<String>,
    pub provider_families: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveRetentionCfg {
    pub celestia_is_da_only: bool,
    pub long_term_retrieval_required: bool,
    pub content_addressing_required: bool,
    pub ipfs_pinning_required: bool,
    pub provider_receipts_required: bool,
    pub retrieval_audit_required: bool,
    pub retrievability_is_not_validity: bool,
    pub min_archive_replicas: u32,
    pub retrieval_audit_interval_blocks: u64,
    pub allowed_backends: Vec<String>,
    pub required_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostQuantumCfg {
    pub is_enabled: bool,
    pub cadence_blocks: u64,
    pub mode: String,
    pub enforcement_stage: String,
    pub enforce_live_cadence: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub epoch_evidence_commitment: String,
    pub required_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotsCfg {
    pub is_enabled: bool,
    pub cadence_epochs: u64,
    pub cadence_blocks: u64,
    pub object_type: String,
    pub bootstrap_allowed_from_snapshot: bool,
    pub requires_retrieval_audit: bool,
    pub must_bind_state_root: bool,
    pub must_bind_settlement_root: bool,
    pub must_bind_last_plonky3_epoch_proof: bool,
    pub must_bind_last_epoch_manifest_root: bool,
    pub must_bind_archive_manifest_root: bool,
    pub must_bind_snapshot_chunk_root: bool,
    pub must_bind_pq_anchor_root: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PruningCfg {
    pub full_node_pruning_allowed: bool,
    pub archive_node_pruning_allowed: bool,
    pub prune_scope: String,
    pub min_retain_recent_epochs: u64,
    pub requires_dispute_window_elapsed: bool,
    pub requires_plonky3_epoch_finalized: bool,
    pub requires_epoch_manifest_finalized: bool,
    pub requires_archive_replication_threshold_met: bool,
    pub requires_retrieval_audit_passed: bool,
    pub must_keep_compact_metadata: bool,
    pub must_keep_epoch_manifest: bool,
    pub must_keep_state_snapshot: bool,
    pub must_not_prune_archive_replicas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionCfg {
    pub dispute_window_blocks: u64,
    pub challenge_window_start: String,
    pub raw_tx_packages: String,
    pub witness_data: String,
    pub tx_proof_bytes: String,
    pub nova_block_proofs: String,
    pub plonky3_epoch_proofs: String,
    pub epoch_manifests: String,
    pub compact_metadata: String,
    pub da_blobs: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContractPaths {
    pub checkpoint_artifacts: PathBuf,
    pub checkpoint_links: PathBuf,
    pub exec_inputs: PathBuf,
    pub prep_snapshots: PathBuf,
    pub delta_journals: PathBuf,
    pub witness_archives: PathBuf,
    pub nova_block_proofs: PathBuf,
    pub pq_checkpoints: PathBuf,
    pub plonky3_epoch_proofs: PathBuf,
    pub epoch_manifests: PathBuf,
    pub archive_manifests: PathBuf,
    pub da_references: PathBuf,
    pub publication_evidence: PathBuf,
    pub checkpoint_lifecycles: PathBuf,
    pub state_snapshots: PathBuf,
    pub retrieval_audits: PathBuf,
    pub archive_receipts: PathBuf,
    pub da_exports: PathBuf,
    pub documentation_packets: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointResolvedPaths {
    pub checkpoint_artifacts: PathBuf,
    pub checkpoint_links: PathBuf,
    pub exec_inputs: PathBuf,
    pub prep_snapshots: PathBuf,
    pub delta_journals: PathBuf,
    pub witness_archives: PathBuf,
    pub nova_block_proofs: PathBuf,
    pub pq_checkpoints: PathBuf,
    pub plonky3_epoch_proofs: PathBuf,
    pub epoch_manifests: PathBuf,
    pub archive_manifests: PathBuf,
    pub da_references: PathBuf,
    pub publication_evidence: PathBuf,
    pub checkpoint_lifecycles: PathBuf,
    pub state_snapshots: PathBuf,
    pub retrieval_audits: PathBuf,
    pub archive_receipts: PathBuf,
    pub da_exports: PathBuf,
    pub documentation_packets: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContractLimits {
    pub max_batch_ops: usize,
    pub max_batch_bytes: usize,
    pub max_witness_bytes: usize,
    pub max_nova_block_proof_bytes: usize,
    pub max_epoch_nova_archive_bytes: usize,
    pub max_plonky3_epoch_proof_bytes: usize,
    pub max_plonky3_epoch_sidecar_bytes: usize,
    pub max_pq_anchor_bytes: usize,
    pub max_archive_manifest_bytes: usize,
    pub max_state_snapshot_manifest_bytes: usize,
    pub max_retrieval_audit_bytes: usize,
    pub max_documentation_packet_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentationCfg {
    pub include_source_disposition: bool,
    pub include_object_schemas: bool,
    pub include_golden_vectors: bool,
    pub include_chain_evidence_ids: bool,
    pub include_measurements: bool,
    pub include_pq_cadence_evidence: bool,
    pub include_backend_manifest: bool,
    pub include_rejected_claim_register: bool,
}

/// Return the repository-owned checkpoint-contract YAML path anchored to the
/// `z00z_storage` crate root rather than the caller's current working
/// directory.
pub fn repo_default_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(CHECKPOINT_CONTRACT_CONFIG_PATH)
}

impl CheckpointContractConfigV1 {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CheckpointError> {
        let cfg: Self = load_yaml_bounded(path.as_ref(), CHECKPOINT_CONTRACT_CONFIG_MAX_BYTES)
            .map_err(|err| CheckpointError::ContractConfig(err.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load_repo_default() -> Result<Self, CheckpointError> {
        Self::load(repo_default_path())
    }

    #[must_use]
    pub fn resolve_paths(&self, root: impl Into<PathBuf>) -> CheckpointResolvedPaths {
        let root = root.into();
        CheckpointResolvedPaths {
            checkpoint_artifacts: root.join(&self.paths.checkpoint_artifacts),
            checkpoint_links: root.join(&self.paths.checkpoint_links),
            exec_inputs: root.join(&self.paths.exec_inputs),
            prep_snapshots: root.join(&self.paths.prep_snapshots),
            delta_journals: root.join(&self.paths.delta_journals),
            witness_archives: root.join(&self.paths.witness_archives),
            nova_block_proofs: root.join(&self.paths.nova_block_proofs),
            pq_checkpoints: root.join(&self.paths.pq_checkpoints),
            plonky3_epoch_proofs: root.join(&self.paths.plonky3_epoch_proofs),
            epoch_manifests: root.join(&self.paths.epoch_manifests),
            archive_manifests: root.join(&self.paths.archive_manifests),
            da_references: root.join(&self.paths.da_references),
            publication_evidence: root.join(&self.paths.publication_evidence),
            checkpoint_lifecycles: root.join(&self.paths.checkpoint_lifecycles),
            state_snapshots: root.join(&self.paths.state_snapshots),
            retrieval_audits: root.join(&self.paths.retrieval_audits),
            archive_receipts: root.join(&self.paths.archive_receipts),
            da_exports: root.join(&self.paths.da_exports),
            documentation_packets: root.join(&self.paths.documentation_packets),
        }
    }

    pub fn validate(&self) -> Result<(), CheckpointError> {
        if self.version != 2 {
            return invalid("version must be 2");
        }
        require_eq("profile", &self.profile, "checkpoint-contract-streaming-v2")?;
        require_eq(
            "architecture_mode",
            &self.architecture_mode,
            "checkpoint_contract_first",
        )?;
        self.validate_statement()?;
        self.validate_branches()?;
        self.validate_authority_promotion()?;
        self.validate_verified_backend()?;
        self.validate_gates()?;
        self.validate_da()?;
        self.validate_archive_retention()?;
        self.validate_post_quantum()?;
        self.validate_snapshots()?;
        self.validate_pruning()?;
        self.validate_retention()?;
        self.validate_paths()?;
        self.validate_limits()?;
        self.validate_documentation()?;
        Ok(())
    }

    #[must_use]
    pub const fn is_v2(&self) -> bool {
        self.version == 2
    }

    pub fn has_pq_checkpoint(&self, height: u64) -> bool {
        self.post_quantum.cadence_blocks > 0
            && height > 0
            && height.is_multiple_of(self.post_quantum.cadence_blocks)
    }

    #[must_use]
    pub fn is_pq_cadence_height(&self, height: u64) -> bool {
        self.has_pq_checkpoint(height)
    }

    pub fn is_verified_backend_enabled(&self) -> Result<bool, CheckpointError> {
        self.validate()?;
        Ok(false)
    }

    pub fn verified_backend_codec_ready(&self) -> Result<bool, CheckpointError> {
        self.validate()?;
        Ok(false)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_pq_anchor(
        &self,
        height: u64,
        statement_digest: [u8; 32],
        pq_delta_root: [u8; 32],
        pq_witness_root: [u8; 32],
        pq_archive_manifest_root: [u8; 32],
        plonky3_epoch_statement_digest: [u8; 32],
        plonky3_epoch_proof_digest: [u8; 32],
        plonky3_public_inputs_digest: [u8; 32],
        nova_chain_root: [u8; 32],
        pq_signature_or_commitment: [u8; 32],
    ) -> Result<Option<PostQuantumCheckpointAnchorV1>, CheckpointError> {
        self.ensure_post_quantum_surface()?;
        if !self.has_pq_checkpoint(height) || !is_pq_anchor_ready(&self.authority_promotion.stage)?
        {
            return Ok(None);
        }
        Ok(Some(PostQuantumCheckpointAnchorV1::new(
            PostQuantumCheckpointAnchorVersion::CURRENT,
            height,
            self.post_quantum.cadence_blocks,
            statement_digest,
            pq_delta_root,
            pq_witness_root,
            pq_archive_manifest_root,
            plonky3_epoch_statement_digest,
            plonky3_epoch_proof_digest,
            plonky3_public_inputs_digest,
            nova_chain_root,
            pq_signature_or_commitment,
            PostQuantumCheckpointAnchorModeV1::Plonky3EpochProof,
            PostQuantumCheckpointEnforcementStageV1::PqAnchorWriter,
        )?))
    }

    pub fn validate_pq_anchor(
        &self,
        height: u64,
        statement_digest: [u8; 32],
        pq_delta_root: [u8; 32],
        pq_witness_root: [u8; 32],
        pq_archive_manifest_root: [u8; 32],
        pq_anchor: Option<&PostQuantumCheckpointAnchorV1>,
    ) -> Result<(), CheckpointError> {
        self.ensure_post_quantum_surface()?;
        let live_anchor_required =
            is_pq_anchor_ready(&self.authority_promotion.stage)? && self.has_pq_checkpoint(height);
        let Some(pq_anchor) = pq_anchor else {
            if live_anchor_required {
                return Err(CheckpointError::Backend(
                    "pq anchor reject: pq_anchor_missing".to_string(),
                ));
            }
            return Ok(());
        };
        if pq_anchor.height() != height {
            return Err(CheckpointError::Backend(
                "pq anchor reject: height_mismatch".to_string(),
            ));
        }
        if !self.has_pq_checkpoint(pq_anchor.height()) {
            return Err(CheckpointError::Backend(
                "pq anchor reject: non_cadence_height".to_string(),
            ));
        }
        if pq_anchor.cadence_blocks() != self.post_quantum.cadence_blocks {
            return Err(CheckpointError::Backend(
                "pq anchor reject: cadence_mismatch".to_string(),
            ));
        }
        if pq_anchor.mode().as_str() != self.post_quantum.mode {
            return Err(CheckpointError::Backend(
                "pq anchor reject: mode_mismatch".to_string(),
            ));
        }
        if pq_anchor.enforcement_stage().as_str() != self.post_quantum.enforcement_stage {
            return Err(CheckpointError::Backend(
                "pq anchor reject: enforcement_stage_mismatch".to_string(),
            ));
        }
        if pq_anchor.statement_digest() != statement_digest
            || pq_anchor.pq_delta_root() != pq_delta_root
            || pq_anchor.pq_witness_root() != pq_witness_root
            || pq_anchor.pq_archive_manifest_root() != pq_archive_manifest_root
        {
            return Err(CheckpointError::Backend(
                "pq anchor reject: pq_anchor_digest_mismatch".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_statement(&self) -> Result<(), CheckpointError> {
        require_eq_u32("statement.version", self.statement.version, 1)?;
        require_eq(
            "statement.domain",
            &self.statement.domain,
            "z00z.checkpoint.transition.v1",
        )?;
        require_exact_list(
            "statement.required_fields",
            &self.statement.required_fields,
            &[
                "height",
                "prev_root",
                "new_root",
                "prev_settlement_root",
                "new_settlement_root",
                "checkpoint_exec_input_id",
                "prep_snapshot_id",
                "tx_data_root",
                "delta_root",
                "witness_root",
                "journal_digest",
                "da_ref",
            ],
        )?;
        require_exact_list(
            "statement.optional_fields",
            &self.statement.optional_fields,
            &[
                "claim_root",
                "prior_recursive_output_root",
                "pq_anchor_root",
            ],
        )
    }

    fn validate_branches(&self) -> Result<(), CheckpointError> {
        let canonical = &self.branches.canonical;
        require_true("branches.canonical.is_enabled", canonical.is_enabled)?;
        require_true(
            "branches.canonical.is_authoritative",
            canonical.is_authoritative,
        )?;
        require_eq(
            "branches.canonical.proof_system",
            &canonical.proof_system,
            "opaque_attest",
        )?;
        require_true(
            "branches.canonical.has_exact_tx_proof_bytes",
            canonical.has_exact_tx_proof_bytes,
        )?;
        require_true(
            "branches.canonical.has_checkpoint_link",
            canonical.has_checkpoint_link,
        )?;
        require_true(
            "branches.canonical.has_replay_ids",
            canonical.has_replay_ids,
        )?;

        let recursive = &self.branches.recursive;
        require_true("branches.recursive.is_enabled", recursive.is_enabled)?;
        require_false(
            "branches.recursive.is_authoritative",
            recursive.is_authoritative,
        )?;
        require_eq(
            "branches.recursive.mode",
            &recursive.mode,
            "streaming_transition_v2",
        )?;
        require_eq(
            "branches.recursive.proof_system",
            &recursive.proof_system,
            "nova_streaming_compressed_v2",
        )?;
        require_true(
            "branches.recursive.has_prior_output_binding",
            recursive.has_prior_output_binding,
        )?;
        if recursive.min_chain_steps < 3 {
            return invalid("branches.recursive.min_chain_steps must be >= 3");
        }
        if recursive.target_chain_steps < recursive.min_chain_steps {
            return invalid("branches.recursive.target_chain_steps must be >= min_chain_steps");
        }
        if recursive.target_chain_steps > 5 {
            return invalid("branches.recursive.target_chain_steps must be <= 5");
        }
        let noop = &recursive.no_op;
        require_true("branches.recursive.no_op.is_enabled", noop.is_enabled)?;
        if noop.execution_input_version != 2 {
            return invalid("branches.recursive.no_op.execution_input_version must be 2");
        }
        require_eq(
            "branches.recursive.no_op.mode",
            &noop.mode,
            "explicit_empty_handoff_v2",
        )?;
        require_true(
            "branches.recursive.no_op.requires_empty_handoff",
            noop.requires_empty_handoff,
        )?;
        require_true(
            "branches.recursive.no_op.preserves_settlement_root",
            noop.preserves_settlement_root,
        )?;

        let nova = &self.branches.nova;
        require_true("branches.nova.is_enabled", nova.is_enabled)?;
        if self.is_v2() {
            require_true("branches.nova.selected", nova.selected)?;
            require_true("branches.nova.is_available", nova.is_available)?;
            require_eq_u64(
                "branches.nova.fold_cadence_blocks",
                nova.fold_cadence_blocks,
                1,
            )?;
            if nova.compressed_proof_snapshot_cadence_blocks == 0 {
                return invalid(
                    "branches.nova.compressed_proof_snapshot_cadence_blocks must be > 0",
                );
            }
        }
        require_false("branches.nova.is_authoritative", nova.is_authoritative)?;
        if nova.is_pq_authoritative {
            return Err(CheckpointError::NovaPqAuthorityUnsupported);
        }
        require_eq(
            "branches.nova.mode",
            &nova.mode,
            "fast_classical_streaming_v2",
        )?;
        require_eq(
            "branches.nova.proof_system",
            &nova.proof_system,
            "nova_streaming_compressed_v2",
        )?;
        require_true(
            "branches.nova.has_prior_output_binding",
            nova.has_prior_output_binding,
        )?;
        require_true(
            "branches.nova.must_bind_statement_digest",
            nova.must_bind_statement_digest,
        )?;
        require_true(
            "branches.nova.must_bind_checkpoint_link",
            nova.must_bind_checkpoint_link,
        )?;
        require_true(
            "branches.nova.retain_until_pq_epoch",
            nova.retain_until_pq_epoch,
        )?;

        let plonky3 = &self.branches.plonky3_epoch;
        require_false("branches.plonky3_epoch.is_enabled", plonky3.is_enabled)?;
        if plonky3.cadence_blocks != self.post_quantum.cadence_blocks {
            return invalid(
                "branches.plonky3_epoch.cadence_blocks must equal post_quantum.cadence_blocks",
            );
        }
        require_false(
            "branches.plonky3_epoch.is_authoritative",
            plonky3.is_authoritative,
        )?;
        if self.is_v2() {
            require_false(
                "branches.plonky3_epoch.is_pq_authoritative",
                plonky3.is_pq_authoritative,
            )?;
            require_false("branches.plonky3_epoch.selected", plonky3.selected)?;
            require_false("branches.plonky3_epoch.is_available", plonky3.is_available)?;
            require_true(
                "branches.plonky3_epoch.provides_pq_epoch_evidence",
                plonky3.provides_pq_epoch_evidence,
            )?;
            require_eq(
                "branches.plonky3_epoch.mode",
                &plonky3.mode,
                "pq_epoch_evidence",
            )?;
        }
        require_eq(
            "branches.plonky3_epoch.proof_system",
            &plonky3.proof_system,
            "plonky3_stark_epoch_v2",
        )?;
        require_true(
            "branches.plonky3_epoch.must_prove_canonical_transition_range",
            plonky3.must_prove_canonical_transition_range,
        )?;
        require_true(
            "branches.plonky3_epoch.may_bind_nova_chain_root",
            plonky3.may_bind_nova_chain_root,
        )?;
        require_true(
            "branches.plonky3_epoch.must_not_depend_only_on_nova",
            plonky3.must_not_depend_only_on_nova,
        )?;
        require_eq("branches.plonky3_epoch.field", &plonky3.field, "koala_bear")?;
        require_eq("branches.plonky3_epoch.hash", &plonky3.hash, "poseidon2")?;
        if plonky3.security_bits < 124 {
            return invalid("branches.plonky3_epoch.security_bits must be >= 124");
        }
        require_eq(
            "branches.plonky3_epoch.recursion_library",
            &plonky3.recursion_library,
            "p3_recursion",
        )?;
        Ok(())
    }

    fn validate_authority_promotion(&self) -> Result<(), CheckpointError> {
        match authority_promotion_next_stage(&self.authority_promotion.stage)? {
            Some(next) => require_exact_list(
                "authority_promotion.allowed_next_stages",
                &self.authority_promotion.allowed_next_stages,
                &[next],
            ),
            None => require_empty_list(
                "authority_promotion.allowed_next_stages",
                &self.authority_promotion.allowed_next_stages,
            ),
        }
    }

    fn validate_verified_backend(&self) -> Result<(), CheckpointError> {
        let verified = &self.verified_backend;
        require_eq(
            "verified_backend.proof_object",
            &verified.proof_object,
            VERIFIED_BACKEND_PROOF_OBJECT,
        )?;
        require_eq(
            "verified_backend.verifier_api",
            &verified.verifier_api,
            VERIFIED_BACKEND_VERIFIER_API,
        )?;
        require_eq(
            "verified_backend.codec_support",
            &verified.codec_support,
            VERIFIED_BACKEND_CODEC_SUPPORT,
        )?;
        require_eq(
            "verified_backend.adapter_trait",
            &verified.adapter_trait,
            VERIFIED_BACKEND_ADAPTER_TRAIT,
        )?;
        require_eq(
            "verified_backend.chain_evidence.object",
            &verified.chain_evidence.object,
            VERIFIED_BACKEND_CHAIN_EVIDENCE_OBJECT,
        )?;
        require_eq_u32(
            "verified_backend.chain_evidence.min_steps",
            verified.chain_evidence.min_steps,
            self.branches.recursive.min_chain_steps,
        )?;
        require_eq_u32(
            "verified_backend.chain_evidence.max_steps",
            verified.chain_evidence.max_steps,
            self.branches.recursive.target_chain_steps,
        )?;
        require_true(
            "verified_backend.chain_evidence.requires_prior_output_binding",
            verified.chain_evidence.requires_prior_output_binding,
        )?;
        require_exact_list(
            "verified_backend.negative_tests",
            &verified.negative_tests,
            &VERIFIED_BACKEND_REQUIRED_NEGATIVE_TESTS,
        )?;
        require_exact_list(
            "verified_backend.benchmarks",
            &verified.benchmarks,
            &VERIFIED_BACKEND_REQUIRED_BENCHMARKS,
        )?;
        require_eq(
            "verified_backend.rollback.procedure",
            &verified.rollback.procedure,
            VERIFIED_BACKEND_ROLLBACK_PROCEDURE,
        )?;
        require_true(
            "verified_backend.rollback.preserves_statement",
            verified.rollback.preserves_statement,
        )?;
        match verified.security_review.status.as_str() {
            VERIFIED_BACKEND_REVIEW_PENDING | VERIFIED_BACKEND_REVIEW_APPROVED => {}
            other => {
                return invalid(format!(
                    "verified_backend.security_review.status must be {VERIFIED_BACKEND_REVIEW_PENDING} or {VERIFIED_BACKEND_REVIEW_APPROVED}, got {other}"
                ))
            }
        }
        require_true(
            "verified_backend.security_review.requires_third_party_equivalent",
            verified.security_review.requires_third_party_equivalent,
        )?;
        require_eq(
            "verified_backend.statement_stability",
            &verified.statement_stability,
            VERIFIED_BACKEND_STATEMENT_STABILITY,
        )?;

        require_false(
            "authority_promotion.recursive_authority_allowed",
            self.authority_promotion.recursive_authority_allowed,
        )?;
        require_false(
            "authority_promotion.verified_backend_allowed",
            self.authority_promotion.verified_backend_allowed,
        )?;
        require_eq(
            "verified_backend.security_review.status",
            &verified.security_review.status,
            VERIFIED_BACKEND_REVIEW_PENDING,
        )?;
        Ok(())
    }

    fn validate_gates(&self) -> Result<(), CheckpointError> {
        require_true(
            "gates.inputs.has_statement_fields",
            self.gates.inputs.has_statement_fields,
        )?;
        require_true(
            "gates.inputs.has_exec_input_id",
            self.gates.inputs.has_exec_input_id,
        )?;
        require_true(
            "gates.inputs.has_prep_snapshot_id",
            self.gates.inputs.has_prep_snapshot_id,
        )?;
        require_true("gates.inputs.has_da_ref", self.gates.inputs.has_da_ref)?;
        require_true(
            "gates.inputs.has_exact_tx_proof_bytes",
            self.gates.inputs.has_exact_tx_proof_bytes,
        )?;
        require_true(
            "gates.outputs.has_checkpoint_artifact",
            self.gates.outputs.has_checkpoint_artifact,
        )?;
        require_true(
            "gates.outputs.has_checkpoint_link",
            self.gates.outputs.has_checkpoint_link,
        )?;
        require_true(
            "gates.outputs.has_da_export",
            self.gates.outputs.has_da_export,
        )?;
        require_true(
            "gates.outputs.has_archive_manifest",
            self.gates.outputs.has_archive_manifest,
        )?;
        require_true(
            "gates.artifacts.has_pq_anchor_on_cadence",
            self.gates.artifacts.has_pq_anchor_on_cadence,
        )?;
        require_true(
            "gates.artifacts.has_mixed_era_fail_closed",
            self.gates.artifacts.has_mixed_era_fail_closed,
        )
    }

    fn validate_da(&self) -> Result<(), CheckpointError> {
        require_eq(
            "da.provider_sdk_boundary",
            &self.da.provider_sdk_boundary,
            "adapter_only",
        )?;
        require_eq(
            "da.publication_readiness_gate",
            &self.da.publication_readiness_gate,
            "required",
        )?;
        require_eq(
            "da.challenge_window_start",
            &self.da.challenge_window_start,
            "da_publication_ready",
        )?;
        require_exact_list(
            "da.allowed_sync_modes",
            &self.da.allowed_sync_modes,
            &["da_only", "hybrid_p2p_da_verified"],
        )?;
        require_exact_list(
            "da.provider_families",
            &self.da.provider_families,
            &["local_archive", "sovereign_sdk_adapter", "celestia_adapter"],
        )
    }

    fn validate_archive_retention(&self) -> Result<(), CheckpointError> {
        let archive = &self.archive_retention;
        require_true(
            "archive_retention.celestia_is_da_only",
            archive.celestia_is_da_only,
        )?;
        require_true(
            "archive_retention.long_term_retrieval_required",
            archive.long_term_retrieval_required,
        )?;
        require_true(
            "archive_retention.content_addressing_required",
            archive.content_addressing_required,
        )?;
        require_true(
            "archive_retention.ipfs_pinning_required",
            archive.ipfs_pinning_required,
        )?;
        require_true(
            "archive_retention.provider_receipts_required",
            archive.provider_receipts_required,
        )?;
        require_true(
            "archive_retention.retrieval_audit_required",
            archive.retrieval_audit_required,
        )?;
        require_true(
            "archive_retention.retrievability_is_not_validity",
            archive.retrievability_is_not_validity,
        )?;
        if archive.min_archive_replicas < 3 {
            return invalid("archive_retention.min_archive_replicas must be >= 3");
        }
        if archive.retrieval_audit_interval_blocks != self.post_quantum.cadence_blocks {
            return invalid(
                "archive_retention.retrieval_audit_interval_blocks must equal post_quantum.cadence_blocks",
            );
        }
        require_exact_list(
            "archive_retention.allowed_backends",
            &archive.allowed_backends,
            &[
                "z00z_archive_node",
                "ipfs_pinned",
                "paid_archival_provider",
                "filecoin_or_equivalent",
                "cold_object_store",
            ],
        )?;
        require_exact_list(
            "archive_retention.required_artifacts",
            &archive.required_artifacts,
            &[
                "archive_manifest_root",
                "raw_tx_package_root",
                "exact_tx_proof_bytes_root",
                "witness_archive_root",
                "delta_journal_root",
                "da_payload_commitment",
                "retrieval_audit_root",
                "archive_provider_receipt_root",
            ],
        )
    }

    fn validate_post_quantum(&self) -> Result<(), CheckpointError> {
        let pq = &self.post_quantum;
        require_true("post_quantum.is_enabled", pq.is_enabled)?;
        if pq.cadence_blocks == 0 {
            return invalid("post_quantum.cadence_blocks must be > 0");
        }
        require_eq("post_quantum.mode", &pq.mode, POST_QUANTUM_MODE)?;
        require_eq(
            "post_quantum.enforcement_stage",
            &pq.enforcement_stage,
            POST_QUANTUM_ENFORCEMENT_STAGE,
        )?;
        if is_pq_anchor_ready(&self.authority_promotion.stage)? {
            require_true("post_quantum.enforce_live_cadence", pq.enforce_live_cadence)?;
        } else {
            require_false("post_quantum.enforce_live_cadence", pq.enforce_live_cadence)?;
        }
        require_eq(
            "post_quantum.epoch_evidence_commitment",
            &pq.epoch_evidence_commitment,
            "non_authenticating_digest_v2",
        )?;
        require_exact_list(
            "post_quantum.required_artifacts",
            &pq.required_artifacts,
            &POST_QUANTUM_REQUIRED_ARTIFACTS,
        )
    }

    fn ensure_post_quantum_surface(&self) -> Result<(), CheckpointError> {
        // The public builder and validator can be invoked with a copied config;
        // keep the stage-to-cadence gate fail-closed at this boundary too.
        self.validate_post_quantum()
    }

    fn validate_snapshots(&self) -> Result<(), CheckpointError> {
        let snapshots = &self.snapshots;
        require_true("snapshots.is_enabled", snapshots.is_enabled)?;
        if snapshots.cadence_epochs == 0 {
            return invalid("snapshots.cadence_epochs must be > 0");
        }
        let expected_cadence = snapshots
            .cadence_epochs
            .checked_mul(self.post_quantum.cadence_blocks)
            .ok_or_else(|| {
                CheckpointError::ContractConfig(
                    "snapshots.cadence_epochs overflows cadence_blocks".to_string(),
                )
            })?;
        if snapshots.cadence_blocks != expected_cadence {
            return invalid(
                "snapshots.cadence_blocks must equal cadence_epochs * post_quantum.cadence_blocks",
            );
        }
        require_eq(
            "snapshots.object_type",
            &snapshots.object_type,
            "state_snapshot_v1",
        )?;
        require_true(
            "snapshots.bootstrap_allowed_from_snapshot",
            snapshots.bootstrap_allowed_from_snapshot,
        )?;
        require_true(
            "snapshots.requires_retrieval_audit",
            snapshots.requires_retrieval_audit,
        )?;
        require_true(
            "snapshots.must_bind_state_root",
            snapshots.must_bind_state_root,
        )?;
        require_true(
            "snapshots.must_bind_settlement_root",
            snapshots.must_bind_settlement_root,
        )?;
        require_true(
            "snapshots.must_bind_last_plonky3_epoch_proof",
            snapshots.must_bind_last_plonky3_epoch_proof,
        )?;
        require_true(
            "snapshots.must_bind_last_epoch_manifest_root",
            snapshots.must_bind_last_epoch_manifest_root,
        )?;
        require_true(
            "snapshots.must_bind_archive_manifest_root",
            snapshots.must_bind_archive_manifest_root,
        )?;
        require_true(
            "snapshots.must_bind_snapshot_chunk_root",
            snapshots.must_bind_snapshot_chunk_root,
        )?;
        require_true(
            "snapshots.must_bind_pq_anchor_root",
            snapshots.must_bind_pq_anchor_root,
        )
    }

    fn validate_pruning(&self) -> Result<(), CheckpointError> {
        let pruning = &self.pruning;
        require_true(
            "pruning.full_node_pruning_allowed",
            pruning.full_node_pruning_allowed,
        )?;
        require_false(
            "pruning.archive_node_pruning_allowed",
            pruning.archive_node_pruning_allowed,
        )?;
        require_eq(
            "pruning.prune_scope",
            &pruning.prune_scope,
            "local_full_node_only",
        )?;
        if pruning.min_retain_recent_epochs == 0 {
            return invalid("pruning.min_retain_recent_epochs must be > 0");
        }
        require_true(
            "pruning.requires_dispute_window_elapsed",
            pruning.requires_dispute_window_elapsed,
        )?;
        require_true(
            "pruning.requires_plonky3_epoch_finalized",
            pruning.requires_plonky3_epoch_finalized,
        )?;
        require_true(
            "pruning.requires_epoch_manifest_finalized",
            pruning.requires_epoch_manifest_finalized,
        )?;
        require_true(
            "pruning.requires_archive_replication_threshold_met",
            pruning.requires_archive_replication_threshold_met,
        )?;
        require_true(
            "pruning.requires_retrieval_audit_passed",
            pruning.requires_retrieval_audit_passed,
        )?;
        require_true(
            "pruning.must_keep_compact_metadata",
            pruning.must_keep_compact_metadata,
        )?;
        require_true(
            "pruning.must_keep_epoch_manifest",
            pruning.must_keep_epoch_manifest,
        )?;
        require_true(
            "pruning.must_keep_state_snapshot",
            pruning.must_keep_state_snapshot,
        )?;
        require_true(
            "pruning.must_not_prune_archive_replicas",
            pruning.must_not_prune_archive_replicas,
        )
    }

    fn validate_retention(&self) -> Result<(), CheckpointError> {
        if self.retention.dispute_window_blocks == 0 {
            return invalid("retention.dispute_window_blocks must be > 0");
        }
        require_eq(
            "retention.challenge_window_start",
            &self.retention.challenge_window_start,
            "da_publication_ready",
        )?;
        require_eq(
            "retention.raw_tx_packages",
            &self.retention.raw_tx_packages,
            "archive_required",
        )?;
        require_eq(
            "retention.witness_data",
            &self.retention.witness_data,
            "archive_required",
        )?;
        require_eq(
            "retention.tx_proof_bytes",
            &self.retention.tx_proof_bytes,
            "canonical_until_verified_backend",
        )?;
        require_eq(
            "retention.nova_block_proofs",
            &self.retention.nova_block_proofs,
            "archive_until_pq_epoch",
        )?;
        require_eq(
            "retention.plonky3_epoch_proofs",
            &self.retention.plonky3_epoch_proofs,
            "permanent_metadata",
        )?;
        require_eq(
            "retention.epoch_manifests",
            &self.retention.epoch_manifests,
            "permanent_metadata",
        )?;
        require_eq(
            "retention.compact_metadata",
            &self.retention.compact_metadata,
            "permanent_metadata",
        )?;
        require_eq(
            "retention.da_blobs",
            &self.retention.da_blobs,
            "da_required_until_archive_replicated",
        )
    }

    fn validate_paths(&self) -> Result<(), CheckpointError> {
        let paths = [
            (
                "paths.checkpoint_artifacts",
                &self.paths.checkpoint_artifacts,
            ),
            ("paths.checkpoint_links", &self.paths.checkpoint_links),
            ("paths.exec_inputs", &self.paths.exec_inputs),
            ("paths.prep_snapshots", &self.paths.prep_snapshots),
            ("paths.delta_journals", &self.paths.delta_journals),
            ("paths.witness_archives", &self.paths.witness_archives),
            ("paths.nova_block_proofs", &self.paths.nova_block_proofs),
            ("paths.pq_checkpoints", &self.paths.pq_checkpoints),
            (
                "paths.plonky3_epoch_proofs",
                &self.paths.plonky3_epoch_proofs,
            ),
            ("paths.epoch_manifests", &self.paths.epoch_manifests),
            ("paths.archive_manifests", &self.paths.archive_manifests),
            ("paths.da_references", &self.paths.da_references),
            (
                "paths.publication_evidence",
                &self.paths.publication_evidence,
            ),
            (
                "paths.checkpoint_lifecycles",
                &self.paths.checkpoint_lifecycles,
            ),
            ("paths.da_exports", &self.paths.da_exports),
            (
                "paths.documentation_packets",
                &self.paths.documentation_packets,
            ),
            ("paths.state_snapshots", &self.paths.state_snapshots),
            ("paths.retrieval_audits", &self.paths.retrieval_audits),
            ("paths.archive_receipts", &self.paths.archive_receipts),
        ];
        let mut seen = BTreeSet::new();
        for (label, path) in paths {
            let norm = validate_relative_path(label, path)?;
            if !seen.insert(norm.clone()) {
                return invalid(format!("{label} collides after normalization"));
            }
        }
        Ok(())
    }

    fn validate_limits(&self) -> Result<(), CheckpointError> {
        require_positive_usize("limits.max_batch_ops", self.limits.max_batch_ops)?;
        require_positive_usize("limits.max_batch_bytes", self.limits.max_batch_bytes)?;
        require_positive_usize("limits.max_witness_bytes", self.limits.max_witness_bytes)?;
        require_positive_usize(
            "limits.max_nova_block_proof_bytes",
            self.limits.max_nova_block_proof_bytes,
        )?;
        require_positive_usize(
            "limits.max_epoch_nova_archive_bytes",
            self.limits.max_epoch_nova_archive_bytes,
        )?;
        require_positive_usize(
            "limits.max_plonky3_epoch_proof_bytes",
            self.limits.max_plonky3_epoch_proof_bytes,
        )?;
        require_positive_usize(
            "limits.max_plonky3_epoch_sidecar_bytes",
            self.limits.max_plonky3_epoch_sidecar_bytes,
        )?;
        require_positive_usize(
            "limits.max_pq_anchor_bytes",
            self.limits.max_pq_anchor_bytes,
        )?;
        require_positive_usize(
            "limits.max_archive_manifest_bytes",
            self.limits.max_archive_manifest_bytes,
        )?;
        require_positive_usize(
            "limits.max_state_snapshot_manifest_bytes",
            self.limits.max_state_snapshot_manifest_bytes,
        )?;
        require_positive_usize(
            "limits.max_retrieval_audit_bytes",
            self.limits.max_retrieval_audit_bytes,
        )?;
        require_positive_usize(
            "limits.max_documentation_packet_bytes",
            self.limits.max_documentation_packet_bytes,
        )?;
        for (label, value) in [
            ("limits.max_batch_bytes", self.limits.max_batch_bytes),
            ("limits.max_witness_bytes", self.limits.max_witness_bytes),
            (
                "limits.max_nova_block_proof_bytes",
                self.limits.max_nova_block_proof_bytes,
            ),
            (
                "limits.max_epoch_nova_archive_bytes",
                self.limits.max_epoch_nova_archive_bytes,
            ),
            (
                "limits.max_plonky3_epoch_proof_bytes",
                self.limits.max_plonky3_epoch_proof_bytes,
            ),
            (
                "limits.max_plonky3_epoch_sidecar_bytes",
                self.limits.max_plonky3_epoch_sidecar_bytes,
            ),
            (
                "limits.max_pq_anchor_bytes",
                self.limits.max_pq_anchor_bytes,
            ),
            (
                "limits.max_archive_manifest_bytes",
                self.limits.max_archive_manifest_bytes,
            ),
            (
                "limits.max_state_snapshot_manifest_bytes",
                self.limits.max_state_snapshot_manifest_bytes,
            ),
            (
                "limits.max_retrieval_audit_bytes",
                self.limits.max_retrieval_audit_bytes,
            ),
            (
                "limits.max_documentation_packet_bytes",
                self.limits.max_documentation_packet_bytes,
            ),
        ] {
            if value > CHECKPOINT_CONTRACT_MAX_OBJECT_BYTES {
                return invalid(format!(
                    "{label} must be <= {CHECKPOINT_CONTRACT_MAX_OBJECT_BYTES} to avoid size overflow"
                ));
            }
        }
        if self.limits.max_witness_bytes < self.limits.max_batch_bytes {
            return invalid("limits.max_witness_bytes must be >= max_batch_bytes");
        }
        if self.limits.max_nova_block_proof_bytes < 8192 {
            return invalid("limits.max_nova_block_proof_bytes must be >= 8192");
        }
        if self.limits.max_epoch_nova_archive_bytes < self.limits.max_nova_block_proof_bytes {
            return invalid(
                "limits.max_epoch_nova_archive_bytes must be >= max_nova_block_proof_bytes",
            );
        }
        if self.limits.max_plonky3_epoch_proof_bytes < 8 * 1024 * 1024 {
            return invalid("limits.max_plonky3_epoch_proof_bytes must be >= 8 MiB");
        }
        if self.limits.max_plonky3_epoch_sidecar_bytes < self.limits.max_plonky3_epoch_proof_bytes {
            return invalid(
                "limits.max_plonky3_epoch_sidecar_bytes must be >= max_plonky3_epoch_proof_bytes",
            );
        }
        if self.limits.max_pq_anchor_bytes < self.limits.max_plonky3_epoch_proof_bytes {
            return invalid("limits.max_pq_anchor_bytes must be >= max_plonky3_epoch_proof_bytes");
        }
        Ok(())
    }

    fn validate_documentation(&self) -> Result<(), CheckpointError> {
        require_true(
            "documentation.include_source_disposition",
            self.documentation.include_source_disposition,
        )?;
        require_true(
            "documentation.include_object_schemas",
            self.documentation.include_object_schemas,
        )?;
        require_true(
            "documentation.include_golden_vectors",
            self.documentation.include_golden_vectors,
        )?;
        require_true(
            "documentation.include_chain_evidence_ids",
            self.documentation.include_chain_evidence_ids,
        )?;
        require_true(
            "documentation.include_measurements",
            self.documentation.include_measurements,
        )?;
        require_true(
            "documentation.include_pq_cadence_evidence",
            self.documentation.include_pq_cadence_evidence,
        )?;
        require_true(
            "documentation.include_backend_manifest",
            self.documentation.include_backend_manifest,
        )?;
        require_true(
            "documentation.include_rejected_claim_register",
            self.documentation.include_rejected_claim_register,
        )
    }
}

fn require_eq(label: &str, got: &str, want: &str) -> Result<(), CheckpointError> {
    if got == want {
        return Ok(());
    }
    invalid(format!("{label} must be {want}, got {got}"))
}

fn require_eq_u32(label: &str, got: u32, want: u32) -> Result<(), CheckpointError> {
    if got == want {
        return Ok(());
    }
    invalid(format!("{label} must be {want}, got {got}"))
}

fn require_eq_u64(label: &str, got: u64, want: u64) -> Result<(), CheckpointError> {
    if got == want {
        return Ok(());
    }
    invalid(format!("{label} must be {want}, got {got}"))
}

fn require_true(label: &str, got: bool) -> Result<(), CheckpointError> {
    if got {
        return Ok(());
    }
    invalid(format!("{label} must be true"))
}

fn require_false(label: &str, got: bool) -> Result<(), CheckpointError> {
    if !got {
        return Ok(());
    }
    invalid(format!("{label} must be false"))
}

fn require_positive_usize(label: &str, got: usize) -> Result<(), CheckpointError> {
    if got > 0 {
        return Ok(());
    }
    invalid(format!("{label} must be > 0"))
}

fn require_exact_list(label: &str, got: &[String], want: &[&str]) -> Result<(), CheckpointError> {
    if got.len() == want.len() && got.iter().zip(want).all(|(got, want)| got == want) {
        return Ok(());
    }
    invalid(format!("{label} must equal [{}]", want.join(", ")))
}

fn require_empty_list(label: &str, got: &[String]) -> Result<(), CheckpointError> {
    if got.is_empty() {
        return Ok(());
    }
    invalid(format!("{label} must be empty"))
}

fn validate_relative_path(label: &str, path: &Path) -> Result<PathBuf, CheckpointError> {
    if path.as_os_str().is_empty() {
        return invalid(format!("{label} must not be empty"));
    }
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return invalid(format!("{label} must be normalized relative path"));
            }
        }
    }
    if out.as_os_str().is_empty() {
        return invalid(format!(
            "{label} must contain at least one normal component"
        ));
    }
    Ok(out)
}

fn authority_promotion_next_stage(stage: &str) -> Result<Option<&'static str>, CheckpointError> {
    match stage {
        AUTHORITY_PROMOTION_STAGE_SPEC_ONLY => Ok(Some(AUTHORITY_PROMOTION_STAGE_CONFIG_GATE)),
        AUTHORITY_PROMOTION_STAGE_CONFIG_GATE => {
            Ok(Some(AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT))
        }
        // T0 stops before proof-system activation. T1 introduces the V2
        // streaming promotion path together with its live evidence gates.
        AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT => Ok(None),
        other => invalid(format!("unsupported authority_promotion.stage {other}")),
    }
}

fn is_pq_anchor_ready(stage: &str) -> Result<bool, CheckpointError> {
    match stage {
        AUTHORITY_PROMOTION_STAGE_SPEC_ONLY
        | AUTHORITY_PROMOTION_STAGE_CONFIG_GATE
        | AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT => Ok(false),
        other => invalid(format!("unsupported authority_promotion.stage {other}")),
    }
}

fn invalid<T>(detail: impl Into<String>) -> Result<T, CheckpointError> {
    Err(CheckpointError::ContractConfig(detail.into()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        repo_default_path, CheckpointContractConfigV1, POST_QUANTUM_ENFORCEMENT_STAGE,
        POST_QUANTUM_MODE, VERIFIED_BACKEND_CANDIDATE_STAGE, VERIFIED_BACKEND_PROOF_OBJECT,
        VERIFIED_BACKEND_STATEMENT_STABILITY,
    };
    use crate::CheckpointError;

    fn cfg() -> CheckpointContractConfigV1 {
        CheckpointContractConfigV1::load(repo_default_path()).expect("repo checkpoint contract")
    }

    #[test]
    fn test_repo_contract_loads() {
        let cfg = cfg();

        assert_eq!(cfg.version, 2);
        assert!(cfg.branches.recursive.is_enabled);
        assert!(!cfg.branches.recursive.is_authoritative);
        assert_eq!(
            cfg.branches.recursive.proof_system,
            "nova_streaming_compressed_v2"
        );
        assert!(cfg.branches.recursive.no_op.is_enabled);
        assert_eq!(cfg.branches.recursive.no_op.execution_input_version, 2);
        assert_eq!(
            cfg.branches.recursive.no_op.mode,
            "explicit_empty_handoff_v2"
        );
        assert!(cfg.branches.nova.is_enabled);
        assert_eq!(
            cfg.branches.nova.proof_system,
            "nova_streaming_compressed_v2"
        );
        assert!(!cfg.branches.nova.is_pq_authoritative);
        assert_eq!(cfg.branches.nova.fold_cadence_blocks, 1);
        assert_eq!(
            cfg.branches.nova.compressed_proof_snapshot_cadence_blocks,
            1
        );
        assert!(cfg.branches.nova.is_available);
        assert!(cfg.branches.nova.selected);
        assert_eq!(
            cfg.branches.plonky3_epoch.proof_system,
            "plonky3_stark_epoch_v2"
        );
        assert!(!cfg.branches.plonky3_epoch.is_enabled);
        assert!(!cfg.branches.plonky3_epoch.is_pq_authoritative);
        assert!(cfg.branches.plonky3_epoch.provides_pq_epoch_evidence);
        assert_eq!(cfg.branches.plonky3_epoch.mode, "pq_epoch_evidence");
        assert_eq!(cfg.post_quantum.cadence_blocks, 1000);
        assert_eq!(cfg.post_quantum.mode, POST_QUANTUM_MODE);
        assert!(cfg.archive_retention.celestia_is_da_only);
        assert_eq!(cfg.archive_retention.min_archive_replicas, 3);
        assert!(cfg.archive_retention.ipfs_pinning_required);
        assert_eq!(cfg.snapshots.object_type, "state_snapshot_v1");
        assert_eq!(cfg.snapshots.cadence_blocks, 10_000);
        assert!(!cfg.pruning.archive_node_pruning_allowed);
        assert_eq!(
            cfg.verified_backend.proof_object,
            VERIFIED_BACKEND_PROOF_OBJECT
        );
        assert_eq!(
            cfg.verified_backend.statement_stability,
            VERIFIED_BACKEND_STATEMENT_STABILITY
        );
    }

    #[test]
    fn test_default_uses_repo_anchor() {
        let anchored = CheckpointContractConfigV1::load(repo_default_path())
            .expect("repo checkpoint contract via anchored path");
        let default = CheckpointContractConfigV1::load_repo_default()
            .expect("repo checkpoint contract via default helper");

        assert_eq!(default, anchored);
    }

    #[test]
    fn test_pq_cadence_default() {
        let cfg = cfg();

        assert!(!cfg.has_pq_checkpoint(0));
        assert!(!cfg.has_pq_checkpoint(999));
        assert!(cfg.has_pq_checkpoint(1000));
        assert!(cfg.has_pq_checkpoint(2000));
    }

    #[test]
    fn test_recursive_authority_rejects() {
        let mut cfg = cfg();
        cfg.branches.recursive.is_authoritative = true;

        let err = cfg.validate().expect_err("recursive authority must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_recursive_noop_contract_rejects_drift() {
        let mut disabled = cfg();
        disabled.branches.recursive.no_op.is_enabled = false;
        assert!(matches!(
            disabled.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));

        let mut version_drift = cfg();
        version_drift
            .branches
            .recursive
            .no_op
            .execution_input_version = 3;
        assert!(matches!(
            version_drift.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));

        let mut root_drift = cfg();
        root_drift
            .branches
            .recursive
            .no_op
            .preserves_settlement_root = false;
        assert!(matches!(
            root_drift.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));
    }

    #[test]
    fn test_nova_pq_authority_rejects() {
        let mut cfg = cfg();
        cfg.branches.nova.is_pq_authoritative = true;

        let err = cfg.validate().expect_err("nova pq authority must reject");

        assert!(matches!(err, CheckpointError::NovaPqAuthorityUnsupported));
    }

    #[test]
    fn test_live_recursive_config_rejects_disabled_or_unselected_nova() {
        let mut invalid_cfg = cfg();
        invalid_cfg.branches.recursive.is_enabled = false;
        assert!(matches!(
            invalid_cfg.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));

        let mut invalid_cfg = cfg();
        invalid_cfg.branches.nova.is_enabled = false;
        assert!(matches!(
            invalid_cfg.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));

        let mut invalid_cfg = cfg();
        invalid_cfg.branches.nova.selected = false;
        assert!(matches!(
            invalid_cfg.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));
    }

    #[test]
    fn test_live_recursive_config_rejects_bad_cadence_matrix() {
        for fold in [0, 2] {
            let mut invalid_cfg = cfg();
            invalid_cfg.branches.nova.fold_cadence_blocks = fold;
            assert!(matches!(
                invalid_cfg.validate(),
                Err(CheckpointError::ContractConfig(_))
            ));
        }
        let mut invalid_cfg = cfg();
        invalid_cfg
            .branches
            .nova
            .compressed_proof_snapshot_cadence_blocks = 0;
        assert!(matches!(
            invalid_cfg.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));
        let mut invalid_cfg = cfg();
        invalid_cfg.snapshots.cadence_blocks = 0;
        assert!(matches!(
            invalid_cfg.validate(),
            Err(CheckpointError::ContractConfig(_))
        ));
    }

    #[test]
    fn test_plonky3_cadence_rejects() {
        let mut cfg = cfg();
        cfg.branches.plonky3_epoch.cadence_blocks = 500;

        let err = cfg
            .validate()
            .expect_err("plonky3 cadence mismatch rejects");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_plonky3_nova_rejects() {
        let mut cfg = cfg();
        cfg.branches.plonky3_epoch.must_not_depend_only_on_nova = false;

        let err = cfg
            .validate()
            .expect_err("plonky3 proof cannot depend only on nova");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_plonky3_cap_rejects() {
        let mut cfg = cfg();
        cfg.limits.max_plonky3_epoch_proof_bytes = 1024 * 1024;

        let err = cfg
            .validate()
            .expect_err("small plonky3 proof cap must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_celestia_archive_claim() {
        let mut cfg = cfg();
        cfg.archive_retention.celestia_is_da_only = false;

        let err = cfg.validate().expect_err("celestia must remain DA-only");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_archive_replicas_reject() {
        let mut cfg = cfg();
        cfg.archive_retention.min_archive_replicas = 2;

        let err = cfg
            .validate()
            .expect_err("archive replica threshold must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_ipfs_without_pinning_rejects() {
        let mut cfg = cfg();
        cfg.archive_retention.ipfs_pinning_required = false;

        let err = cfg.validate().expect_err("ipfs pinning must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_snapshot_plonky3_missing() {
        let mut cfg = cfg();
        cfg.snapshots.must_bind_last_plonky3_epoch_proof = false;

        let err = cfg
            .validate()
            .expect_err("snapshot without plonky3 binding must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_archive_node_pruning_rejects() {
        let mut cfg = cfg();
        cfg.pruning.archive_node_pruning_allowed = true;

        let err = cfg
            .validate()
            .expect_err("archive node pruning must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_zero_pq_cadence_rejects() {
        let mut cfg = cfg();
        cfg.post_quantum.cadence_blocks = 0;

        assert!(!cfg.has_pq_checkpoint(1000));

        let err = cfg.validate().expect_err("zero cadence must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_early_pq_cadence() {
        let mut cfg = cfg();
        cfg.post_quantum.enforce_live_cadence = true;

        let err = cfg.validate().expect_err("early live cadence must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_unimplemented_pq_writer_stage_rejects() {
        let mut cfg = cfg();
        cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
        cfg.authority_promotion.allowed_next_stages =
            vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];

        let err = cfg
            .validate()
            .expect_err("an unbound PQ writer stage must reject");
        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_absolute_path_rejects() {
        let mut cfg = cfg();
        cfg.paths.pq_checkpoints = PathBuf::from("/tmp/pq_anchor");

        let err = cfg.validate().expect_err("absolute path must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_path_collision_rejects() {
        let mut cfg = cfg();
        cfg.paths.pq_checkpoints = cfg.paths.nova_block_proofs.clone();

        let err = cfg.validate().expect_err("colliding path must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }

    #[test]
    fn test_documentation_gate_rejects() {
        let mut cfg = cfg();
        cfg.documentation.include_rejected_claim_register = false;

        let err = cfg.validate().expect_err("missing docs gate must reject");

        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }
}
