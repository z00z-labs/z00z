//! Schema-3 checkpoint contract and schema-2 rename/migration ledger.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, OnceLock, RwLock,
    },
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use z00z_crypto::sha256_256;
use z00z_utils::{
    codec::{BincodeCodec, Codec, YamlCodec},
    config::YamlValue,
    io::{
        atomic_write_file_private, create_dir_all, open_lock_file, path_exists_no_follow,
        read_file_bounded, symlink_metadata, sync_directory,
    },
};

use crate::CheckpointError;

use super::{
    contract_config::{
        repo_default_path, CanonicalBranchCfg, CheckpointContractConfigV2, CheckpointResolvedPaths,
        DaCfg, CHECKPOINT_CONTRACT_CONFIG_MAX_BYTES,
    },
    pq_anchor::{
        PostQuantumCheckpointAnchorModeV1, PostQuantumCheckpointAnchorV1,
        PostQuantumCheckpointAnchorVersion, PostQuantumCheckpointEnforcementStageV1,
    },
    version_registry::{
        CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, RegistryOperationV2,
        CHECKPOINT_VERSION_REGISTRY_DIGEST_V2, RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
        RECURSIVE_PARAMETER_GENERATION_V2, RECURSIVE_PROFILE_MANIFEST_DIGEST_V2,
        RECURSIVE_RUNTIME_PROFILE_GENERATION_V2, RECURSIVE_RUNTIME_PROFILE_V2,
    },
};

pub const CONFIG_V3_TRANSFORM_VERSION: u16 = 2;
pub const CONFIG_V3_PROFILE: &str = "checkpoint-contract-client-notary-v2";
pub const CONFIG_V3_PQ_MODE: &str = "plonky3_epoch_evidence_async";
pub const CONFIG_V3_NEWLINE_POLICY: &str = "single-lf";
pub const POST_QUANTUM_REQUIRED_ARTIFACTS_V3: [&str; 11] = [
    "pq_statement_digest",
    "pq_delta_root",
    "pq_witness_root",
    "challenge_content_root",
    "da_payload_commitment",
    "archive_availability_manifest_root",
    "plonky3_epoch_statement_digest",
    "plonky3_epoch_proof_digest",
    "plonky3_public_inputs_digest",
    "nova_chain_root",
    "epoch_evidence_commitment",
];
const MIGRATION_RECORD_MAX_BYTES_V3: usize = 128 * 1024;
const CONFIG_V2_MIGRATION_BYTES: &[u8] = include_bytes!("checkpoint_contract_v2_migration.yaml");
const EMBEDDED_CHAIN_CONTEXT_PREIMAGE_V3: &[u8] =
    b"network=z00z-local\nchain=checkpoint-contract\nconfig_schema=3\n";
const EMBEDDED_RELEASE_IDENTITY_V3: &str = "phase-069-051";
// Independently generated from the complete release tuple by the authority
// review tool. Production recomputes that tuple and compares it to this literal;
// it never accepts a candidate's recomputation as its own authorization.
const EMBEDDED_RELEASE_MANIFEST_DIGEST_V3: [u8; 32] = [
    0x5a, 0x5c, 0xd5, 0x65, 0xc4, 0xa9, 0x90, 0xb2, 0x09, 0x00, 0x51, 0x21, 0x4f, 0x65, 0x1f, 0x27,
    0x47, 0x62, 0x13, 0xe5, 0x5c, 0xa4, 0xec, 0xba, 0x1d, 0xcf, 0xcb, 0xc9, 0x9d, 0x0c, 0xe1, 0x71,
];

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileCfgV3 {
    pub identifier: String,
    pub generation: u16,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionAuthorityCfgV3 {
    pub registry_digest: String,
    pub authority_generation: u32,
    pub parameter_generation: u32,
    pub config_generation: u64,
    pub activation_height: u64,
    pub rollback_floor: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentStateShardingCfgV3 {
    pub is_required: bool,
    pub shard_count: u16,
    pub replication_factor: u16,
    pub write_quorum: u16,
    pub read_quorum: u16,
    pub min_failure_domains: u16,
    pub is_full_state_replica_allowed: bool,
    pub route_key_profile: String,
    pub rollout_profile: String,
    pub has_seed_recovery: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OfflineReceiptMailboxCfgV3 {
    pub is_required: bool,
    pub is_runtime_enabled: bool,
    pub semantic_owner_phase: u16,
    pub phase_069_role: String,
    pub admission_stage: String,
    pub max_admission_bytes_per_block: u64,
    pub max_partition_block_admission_bytes: u64,
    pub object_type: String,
    pub retention_blocks: u64,
    pub retention_start: String,
    pub staging_expiry: String,
    pub max_notice_plaintext_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_entries_per_recipient_output: u16,
    pub is_recipient_capability_required: bool,
    pub is_sender_local_policy_required: bool,
    pub has_payment_request_v1_capability: bool,
    pub logical_partition_count: u16,
    pub partitions_per_entry: u16,
    pub is_cross_partition_fanout_allowed: bool,
    pub has_adversarial_uniformity_claim: bool,
    pub replication_factor: u16,
    pub write_quorum: u16,
    pub read_quorum: u16,
    pub min_failure_domains: u16,
    pub is_public_listing_allowed: bool,
    pub is_public_dht_publication_allowed: bool,
    pub has_ack_early_gc: bool,
    pub has_seed_recovery_fallback: bool,
    pub is_sender_ack_retention_required: bool,
}

impl CurrentStateShardingCfgV3 {
    fn authority_pinned() -> Self {
        Self {
            is_required: true,
            shard_count: 16,
            replication_factor: 3,
            write_quorum: 2,
            read_quorum: 1,
            min_failure_domains: 4,
            is_full_state_replica_allowed: false,
            route_key_profile: "hjmt_terminal_hash_range_v1".to_string(),
            rollout_profile: "cow_copy_delta_catchup_cas_v1".to_string(),
            has_seed_recovery: true,
        }
    }

    fn validate(&self) -> Result<(), CheckpointError> {
        if self != &Self::authority_pinned() {
            return config_error(
                "current_state_sharding must match the exact generation-2 authority profile",
            );
        }
        Ok(())
    }
}

impl OfflineReceiptMailboxCfgV3 {
    fn authority_pinned() -> Self {
        Self {
            is_required: true,
            is_runtime_enabled: false,
            semantic_owner_phase: 71,
            phase_069_role: "reserved_unreachable_handoff".to_string(),
            admission_stage: "declared_only".to_string(),
            max_admission_bytes_per_block: 0,
            max_partition_block_admission_bytes: 0,
            object_type: "encrypted_receipt_mailbox_entry_v1".to_string(),
            retention_blocks: 1_555_200,
            retention_start: "canonical_output_finalized".to_string(),
            staging_expiry: "transaction_expiry".to_string(),
            max_notice_plaintext_bytes: 2_048,
            max_entry_bytes: 8_192,
            max_entries_per_recipient_output: 1,
            is_recipient_capability_required: true,
            is_sender_local_policy_required: true,
            has_payment_request_v1_capability: false,
            logical_partition_count: 16,
            partitions_per_entry: 1,
            is_cross_partition_fanout_allowed: false,
            has_adversarial_uniformity_claim: false,
            replication_factor: 3,
            write_quorum: 2,
            read_quorum: 1,
            min_failure_domains: 3,
            is_public_listing_allowed: false,
            is_public_dht_publication_allowed: false,
            has_ack_early_gc: true,
            has_seed_recovery_fallback: true,
            is_sender_ack_retention_required: false,
        }
    }

    fn validate(&self) -> Result<(), CheckpointError> {
        if self != &Self::authority_pinned() {
            return config_error(
                "offline_receipt_mailbox must match the exact Phase-071 declared-only reservation",
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BranchesCfgV3 {
    pub canonical: CanonicalBranchCfg,
    pub recursive: RecursiveBranchCfgV3,
    pub nova: NovaBranchCfgV3,
    pub plonky3_epoch: Plonky3EpochBranchCfgV3,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatementCfgV3 {
    pub version: u32,
    pub domain: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub canonical_admission_forbidden_when_present: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecursiveBranchCfgV3 {
    pub is_enabled: bool,
    pub is_authoritative: bool,
    pub mode: String,
    pub proof_system: String,
    pub has_prior_output_binding: bool,
    pub min_chain_steps: u32,
    pub target_chain_steps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NovaBranchCfgV3 {
    pub is_enabled: bool,
    pub fold_cadence_blocks: u64,
    pub recovery_snapshot_cadence_blocks: u64,
    pub compression_cadence_blocks: u64,
    pub publication_cadence_blocks: u64,
    pub hot_recovery_snapshot_count: u16,
    pub max_retained_bodies_per_epoch: u16,
    pub max_pending_pq_epochs: u16,
    pub post_pq_grace_certified_epochs: u16,
    pub pending_pq_cap_action: String,
    pub is_authoritative: bool,
    pub security_role: String,
    pub mode: String,
    pub proof_system: String,
    pub has_prior_output_binding: bool,
    pub has_statement_digest_bind: bool,
    pub has_checkpoint_link_bind: bool,
    pub proof_body_retention: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plonky3EpochBranchCfgV3 {
    pub is_enabled: bool,
    pub cadence_blocks: u64,
    pub is_authoritative: bool,
    pub has_pq_epoch_evidence: bool,
    pub mode: String,
    pub proof_system: String,
    pub has_transition_range_proof: bool,
    pub has_nova_chain_bind: bool,
    pub has_independent_transition_proof: bool,
    pub field: String,
    pub hash: String,
    pub security_bits: u16,
    pub recursion_library: String,
    pub has_security_budget_manifest: bool,
    pub soundness_composition: String,
    pub has_epoch_count_bind: bool,
    pub minimum_composed_security_bits: u16,
    pub has_exact_epoch_statement: bool,
    pub has_history_successor: bool,
    pub has_rotation_bridge: bool,
    pub proof_body_retention: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPromotionCfgV3 {
    pub stage: String,
    pub is_recursive_authority_allowed: bool,
    pub is_verified_backend_allowed: bool,
    pub allowed_next_stages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatesCfgV3 {
    pub inputs: super::contract_config::InputGatesCfg,
    pub outputs: OutputGatesCfgV3,
    pub artifacts: ArtifactGatesCfgV3,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutputGatesCfgV3 {
    pub has_checkpoint_artifact: bool,
    pub has_checkpoint_link: bool,
    pub has_da_export: bool,
    pub has_challenge_content_commitment: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactGatesCfgV3 {
    pub has_recursive_sidecar_non_authoritative: bool,
    pub has_async_pq_anchor: bool,
    pub has_archive_availability_manifest_async: bool,
    pub has_mixed_era_fail_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveRetentionCfgV3 {
    pub is_celestia_da_only: bool,
    pub challenge_window_blocks: u64,
    pub challenge_window_start: String,
    pub has_permanent_raw_history: bool,
    pub has_compact_notary_history: bool,
    pub has_content_addressing: bool,
    pub has_ipfs_pinning: bool,
    pub has_provider_receipts: bool,
    pub has_retrieval_audit: bool,
    pub is_retrieval_non_authority: bool,
    pub reconstruction_threshold: u16,
    pub total_shards: u16,
    pub max_shards_per_failure_domain: u16,
    pub erasure_coding_profile: String,
    pub is_full_replica_fallback_allowed: bool,
    pub retrieval_audit_interval_blocks: u64,
    pub allowed_backends: Vec<String>,
    pub required_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostQuantumCfgV3 {
    pub is_enabled: bool,
    pub cadence_blocks: u64,
    pub mode: String,
    pub enforcement_stage: String,
    pub is_live_cadence_enforced: bool,
    pub required_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotsCfgV3 {
    pub is_enabled: bool,
    pub cadence_epochs: u64,
    pub cadence_blocks: u64,
    pub object_type: String,
    pub is_snapshot_bootstrap_allowed: bool,
    pub has_retrieval_audit: bool,
    pub has_state_root_bind: bool,
    pub has_settlement_root_bind: bool,
    pub has_epoch_proof_bind: bool,
    pub has_epoch_manifest_bind: bool,
    pub has_archive_manifest_bind: bool,
    pub has_snapshot_chunk_bind: bool,
    pub has_pq_anchor_bind: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PruningCfgV3 {
    pub stage: String,
    pub activation_scope: String,
    pub is_full_node_pruning_allowed: bool,
    pub is_watcher_pruning_allowed: bool,
    pub is_legacy_pruning_allowed: bool,
    pub watcher_mode: String,
    pub prune_scope: String,
    pub min_retain_recent_epochs: u64,
    pub is_window_expired: bool,
    pub has_no_hold_or_dispute: bool,
    pub is_epoch_proof_verified: bool,
    pub has_verified_history_link: bool,
    pub has_retained_anchor_mmr: bool,
    pub has_later_evidence_anchor: bool,
    pub has_verified_bootstrap_snapshot: bool,
    pub has_archive_replication_quorum: bool,
    pub has_passing_retrieval_audit: bool,
    pub has_live_vk_manifest: bool,
    pub has_retention_ledger_cas: bool,
    pub has_compact_metadata_retention: bool,
    pub must_keep_latest_snapshot_generations: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionCfgV3 {
    pub dispute_window_blocks: u64,
    pub challenge_window_start: String,
    pub raw_tx_packages: String,
    pub checkpoint_artifact_bodies: String,
    pub finalized_checkpoint_record_bodies: String,
    pub checkpoint_artifact_and_record_digests: String,
    pub other_header_qc_bodies: String,
    pub witness_data: String,
    pub tx_proof_bytes: String,
    pub nova_block_proofs: String,
    pub plonky3_epoch_proofs: String,
    pub epoch_manifest_bodies: String,
    pub checkpoint_archive_manifest_bodies: String,
    pub pq_anchor_bodies: String,
    pub compact_epoch_history_rotation_anchors: String,
    pub epoch_close_finality_certificates: String,
    pub state_and_recovery_snapshots: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContractPathsV3 {
    pub checkpoint_artifacts: PathBuf,
    pub checkpoint_links: PathBuf,
    pub exec_inputs: PathBuf,
    pub prep_snapshots: PathBuf,
    pub delta_journals: PathBuf,
    pub witness_archives: PathBuf,
    pub recursive_sidecars: PathBuf,
    pub nova_block_proofs: PathBuf,
    pub pq_checkpoints: PathBuf,
    pub plonky3_epoch_proofs: PathBuf,
    pub epoch_manifests: PathBuf,
    pub epoch_close_anchors: PathBuf,
    pub epoch_evidence_anchors: PathBuf,
    pub archive_manifests: PathBuf,
    pub state_snapshots: PathBuf,
    pub retrieval_audits: PathBuf,
    pub archive_receipts: PathBuf,
    pub challenge_packs: PathBuf,
    pub retention_tickets: PathBuf,
    pub retention_ledger: PathBuf,
    pub history_proofs: PathBuf,
    pub history_rotation_bridges: PathBuf,
    pub da_exports: PathBuf,
    pub documentation_packets: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContractLimitsV3 {
    pub max_batch_ops: usize,
    pub max_batch_bytes: usize,
    pub max_witness_bytes: usize,
    pub max_recursive_proof_envelope_bytes: usize,
    pub max_recursive_sidecar_bytes: usize,
    pub max_nova_block_proof_bytes: usize,
    pub max_nova_retained_proof_bodies: usize,
    pub max_nova_retained_body_bytes: usize,
    pub max_nova_hot_recovery_bytes: usize,
    pub max_epoch_nova_archive_bytes: usize,
    pub max_plonky3_epoch_proof_bytes: usize,
    pub max_plonky3_epoch_sidecar_bytes: usize,
    pub max_pq_anchor_bytes: usize,
    pub max_archive_manifest_bytes: usize,
    pub max_state_snapshot_manifest_bytes: usize,
    pub max_retrieval_audit_bytes: usize,
    pub max_epoch_anchor_bytes: usize,
    pub max_epoch_close_certificate_bytes: usize,
    pub max_close_evidence_appends: usize,
    pub max_daily_history_bytes: usize,
    pub max_documentation_packet_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentationCfgV3 {
    pub has_recursive_packet: bool,
    pub has_source_disposition: bool,
    pub has_object_schemas: bool,
    pub has_golden_vectors: bool,
    pub has_chain_evidence_ids: bool,
    pub has_measurements: bool,
    pub has_pq_cadence_evidence: bool,
    pub has_backend_manifest: bool,
    pub has_rejected_claim_register: bool,
    pub has_retention_evidence: bool,
    pub has_nova_retention_evidence: bool,
    pub has_history_rotation_evidence: bool,
    pub has_wallet_backup_evidence: bool,
    pub has_atomicity_evidence: bool,
    pub has_capacity_traffic_evidence: bool,
}

/// Sole active checkpoint configuration after the schema-3 cutover.
///
/// Fields inherited from schema 2 stay explicit; there is no flatten/default/
/// alias path and therefore no fallback decoder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContractConfigV3 {
    pub version: u32,
    pub profile: String,
    pub architecture_mode: String,
    pub runtime_profile: RuntimeProfileCfgV3,
    pub version_authority: VersionAuthorityCfgV3,
    pub statement: StatementCfgV3,
    pub branches: BranchesCfgV3,
    pub authority_promotion: AuthorityPromotionCfgV3,
    pub gates: GatesCfgV3,
    pub da: DaCfg,
    pub current_state_sharding: CurrentStateShardingCfgV3,
    pub offline_receipt_mailbox: OfflineReceiptMailboxCfgV3,
    pub archive_retention: ArchiveRetentionCfgV3,
    pub post_quantum: PostQuantumCfgV3,
    pub snapshots: SnapshotsCfgV3,
    pub pruning: PruningCfgV3,
    pub retention: RetentionCfgV3,
    pub paths: CheckpointContractPathsV3,
    pub limits: CheckpointContractLimitsV3,
    pub documentation: DocumentationCfgV3,
}

impl CheckpointContractConfigV3 {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CheckpointError> {
        let bytes = read_file_bounded(path.as_ref(), CHECKPOINT_CONTRACT_CONFIG_MAX_BYTES)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        CheckpointVersionRegistryV2::authority_pinned()?.decode_config_schema(
            RecursiveBoundedObjectV2::CheckpointContractConfigV3,
            &bytes,
            RegistryOperationV2::Read,
            Self::decode_canonical_bytes,
        )
    }

    fn decode_canonical_bytes(bytes: &[u8]) -> Result<Self, CheckpointError> {
        reject_noncanonical_yaml_features(bytes)?;
        let cfg: Self = YamlCodec
            .deserialize(bytes)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        cfg.validate()?;
        let canonical = YamlCodec
            .serialize(&cfg)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        if canonical.as_slice() != bytes {
            return Err(CheckpointError::ContractConfig(
                "schema-3 YAML is not the exact canonical re-encoding".to_string(),
            ));
        }
        Ok(cfg)
    }

    pub fn load_repo_default() -> Result<Self, CheckpointError> {
        Self::load(repo_default_path())
    }

    pub fn validate(&self) -> Result<(), CheckpointError> {
        if self.version != 3 || self.profile != CONFIG_V3_PROFILE {
            return config_error("schema/profile must be 3/checkpoint-contract-client-notary-v2");
        }
        if self.runtime_profile.identifier != RECURSIVE_RUNTIME_PROFILE_V2
            || self.runtime_profile.generation != RECURSIVE_RUNTIME_PROFILE_GENERATION_V2
            || self.runtime_profile.manifest_digest
                != hex_digest(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2)
        {
            return config_error("runtime profile identity or manifest digest mismatch");
        }
        let registry = CheckpointVersionRegistryV2::authority_pinned()?;
        if self.version_authority.registry_digest
            != hex_digest(CHECKPOINT_VERSION_REGISTRY_DIGEST_V2)
            || self.version_authority.registry_digest != hex_digest(registry.digest())
            || self.version_authority.authority_generation != 2
            || self.version_authority.parameter_generation != RECURSIVE_PARAMETER_GENERATION_V2
            || self.version_authority.config_generation == 0
            || self.version_authority.rollback_floor > self.version_authority.config_generation
        {
            return config_error("version authority tuple mismatch");
        }
        self.current_state_sharding.validate()?;
        self.offline_receipt_mailbox.validate()?;
        let authority = embedded_authority_unvalidated();
        if self.architecture_mode != authority.architecture_mode
            || self.statement != authority.statement
            || self.branches != authority.branches
            || self.gates != authority.gates
            || self.da != authority.da
            || self.archive_retention != authority.archive_retention
            || self.post_quantum != authority.post_quantum
            || self.snapshots != authority.snapshots
            || self.pruning != authority.pruning
            || self.retention != authority.retention
            || self.paths != authority.paths
            || self.limits != authority.limits
            || self.documentation != authority.documentation
        {
            return config_error("schema-3 authority leaf mismatch");
        }
        if self.statement.canonical_admission_forbidden_when_present
            != ["pq_anchor_root".to_string()]
            || self.branches.recursive.mode != "hybrid_nova_plonky3"
            || self.branches.recursive.proof_system != "recursive_hybrid_v2"
            || self.branches.nova.proof_system != "nova_streaming_compressed_v2"
            || self.branches.plonky3_epoch.proof_system != "plonky3_stark_epoch_v2"
            || !self
                .post_quantum
                .required_artifacts
                .iter()
                .map(String::as_str)
                .eq(POST_QUANTUM_REQUIRED_ARTIFACTS_V3)
            || self.archive_retention.erasure_coding_profile != "rs_10_16_v1"
            || self.archive_retention.reconstruction_threshold != 10
            || self.archive_retention.total_shards != 16
            || self.limits.max_pq_anchor_bytes != 4_096
            || self.pruning.stage != "declared_only"
        {
            return config_error("schema-3 frozen semantic profile mismatch");
        }

        self.validate_authority_promotion()
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
            recursive_sidecars: root.join(&self.paths.recursive_sidecars),
            nova_block_proofs: root.join(&self.paths.nova_block_proofs),
            pq_checkpoints: root.join(&self.paths.pq_checkpoints),
            plonky3_epoch_proofs: root.join(&self.paths.plonky3_epoch_proofs),
            epoch_manifests: root.join(&self.paths.epoch_manifests),
            epoch_close_anchors: root.join(&self.paths.epoch_close_anchors),
            epoch_evidence_anchors: root.join(&self.paths.epoch_evidence_anchors),
            archive_manifests: root.join(&self.paths.archive_manifests),
            da_references: root.join("artifacts/checkpoints/da_reference"),
            publication_evidence: root.join("artifacts/checkpoints/publication_evidence"),
            checkpoint_lifecycles: root.join("artifacts/checkpoints/lifecycle"),
            state_snapshots: root.join(&self.paths.state_snapshots),
            retrieval_audits: root.join(&self.paths.retrieval_audits),
            archive_receipts: root.join(&self.paths.archive_receipts),
            challenge_packs: root.join(&self.paths.challenge_packs),
            retention_tickets: root.join(&self.paths.retention_tickets),
            retention_ledger: root.join(&self.paths.retention_ledger),
            history_proofs: root.join(&self.paths.history_proofs),
            history_rotation_bridges: root.join(&self.paths.history_rotation_bridges),
            da_exports: root.join(&self.paths.da_exports),
            documentation_packets: root.join(&self.paths.documentation_packets),
        }
    }

    pub(crate) fn validate_recursive_proof_envelope_ingress(
        &self,
        encoded_len: usize,
    ) -> Result<(), CheckpointError> {
        self.validate()?;
        if encoded_len > self.limits.max_recursive_proof_envelope_bytes {
            return Err(CheckpointError::Limit);
        }
        Ok(())
    }

    pub(crate) fn validate_recursive_sidecar_ingress(
        &self,
        encoded_len: usize,
    ) -> Result<(), CheckpointError> {
        self.validate()?;
        if encoded_len > self.limits.max_recursive_sidecar_bytes {
            return Err(CheckpointError::Limit);
        }
        Ok(())
    }

    #[must_use]
    pub const fn is_v3(&self) -> bool {
        self.version == 3
    }

    pub fn has_pq_checkpoint(&self, height: u64) -> bool {
        self.post_quantum.cadence_blocks > 0
            && height > 0
            && height.is_multiple_of(self.post_quantum.cadence_blocks)
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
        self.validate()?;
        if !self.has_pq_checkpoint(height)
            || !is_pq_anchor_ready_v3(&self.authority_promotion.stage)?
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
        anchor: Option<&PostQuantumCheckpointAnchorV1>,
    ) -> Result<(), CheckpointError> {
        self.validate()?;
        let required = is_pq_anchor_ready_v3(&self.authority_promotion.stage)?
            && self.has_pq_checkpoint(height);
        let Some(anchor) = anchor else {
            if required {
                return Err(CheckpointError::Backend(
                    "pq anchor reject: pq_anchor_missing".to_string(),
                ));
            }
            return Ok(());
        };
        if anchor.height() != height
            || !self.has_pq_checkpoint(anchor.height())
            || anchor.cadence_blocks() != self.post_quantum.cadence_blocks
            || anchor.mode() != PostQuantumCheckpointAnchorModeV1::Plonky3EpochProof
            || anchor.enforcement_stage() != PostQuantumCheckpointEnforcementStageV1::PqAnchorWriter
            || anchor.statement_digest() != statement_digest
            || anchor.pq_delta_root() != pq_delta_root
            || anchor.pq_witness_root() != pq_witness_root
            || anchor.pq_archive_manifest_root() != pq_archive_manifest_root
        {
            return Err(CheckpointError::Backend(
                "pq anchor reject: ConfigV3 binding mismatch".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_authority_promotion(&self) -> Result<(), CheckpointError> {
        let expected_next = authority_promotion_next_stage_v3(&self.authority_promotion.stage)?;
        let exact_next = expected_next
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if self.authority_promotion.allowed_next_stages != exact_next
            || self.authority_promotion.is_recursive_authority_allowed
            || self.authority_promotion.is_verified_backend_allowed
            || self.post_quantum.is_live_cadence_enforced
                != is_pq_anchor_ready_v3(&self.authority_promotion.stage)?
        {
            return config_error("ConfigV3 authority-promotion state mismatch");
        }
        Ok(())
    }

    pub fn canonical_digest(&self) -> Result<[u8; 32], CheckpointError> {
        let _source = decode_config_v2_migration(CONFIG_V2_MIGRATION_BYTES)?;
        let destination_bytes = YamlCodec.serialize(self).map_err(map_codec)?;
        let ledger =
            ConfigV3RenameLedger::from_pair(CONFIG_V2_MIGRATION_BYTES, &destination_bytes)?;
        self.canonical_digest_with_ledger(ledger.digest)
    }

    fn canonical_digest_with_ledger(
        &self,
        rename_ledger_digest: [u8; 32],
    ) -> Result<[u8; 32], CheckpointError> {
        self.validate()?;
        let bytes = YamlCodec
            .serialize(self)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        CheckpointVersionRegistryV2::authority_pinned()?.validate_config_schema(
            RecursiveBoundedObjectV2::CheckpointContractConfigV3,
            3,
            bytes.len() as u64,
            RegistryOperationV2::Write,
        )?;
        if rename_ledger_digest == [0; 32] {
            return config_error("schema-3 config digest requires a rename ledger");
        }
        Ok(sha256_256(
            "z00z.storage.checkpoint.contract-config.v3",
            "config_digest",
            &[
                &self.version.to_le_bytes(),
                &(bytes.len() as u64).to_le_bytes(),
                &bytes,
                &decode_digest_hex(&self.version_authority.registry_digest)?,
                self.runtime_profile.identifier.as_bytes(),
                &self.runtime_profile.generation.to_le_bytes(),
                &decode_digest_hex(&self.runtime_profile.manifest_digest)?,
                &self.version_authority.authority_generation.to_le_bytes(),
                &self.version_authority.parameter_generation.to_le_bytes(),
                &rename_ledger_digest,
            ],
        ))
    }
}

fn authority_promotion_next_stage_v3(stage: &str) -> Result<Option<&'static str>, CheckpointError> {
    match stage {
        "spec_only" => Ok(Some("config_gate")),
        "config_gate" => Ok(Some("canonical_extended_statement")),
        "canonical_extended_statement" => Ok(None),
        other => config_error(format!("unsupported ConfigV3 authority stage {other}")),
    }
}

fn is_pq_anchor_ready_v3(stage: &str) -> Result<bool, CheckpointError> {
    authority_promotion_next_stage_v3(stage).map(|_| false)
}

fn embedded_chain_context_digest_v3() -> [u8; 32] {
    sha256_256(
        "z00z.storage.checkpoint.config-migration-chain-context.v3",
        "chain_context_digest",
        &[EMBEDDED_CHAIN_CONTEXT_PREIMAGE_V3],
    )
}

fn embedded_authority_unvalidated() -> &'static CheckpointContractConfigV3 {
    static AUTHORITY: OnceLock<CheckpointContractConfigV3> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        YamlCodec
            .deserialize(include_bytes!("checkpoint_contract.yaml"))
            .expect("embedded schema-3 checkpoint authority must deserialize")
    })
}

fn decode_config_v2_migration(bytes: &[u8]) -> Result<CheckpointContractConfigV2, CheckpointError> {
    CheckpointVersionRegistryV2::authority_pinned()?.decode_config_schema(
        RecursiveBoundedObjectV2::CheckpointContractConfigV2,
        bytes,
        RegistryOperationV2::Read,
        |source_bytes| {
            // Schema 2 is an immutable migration/audit input. Require the
            // registry callback to receive the exact embedded authority bytes
            // before also proving that those bytes are the canonical typed
            // re-encoding below.
            if source_bytes != CONFIG_V2_MIGRATION_BYTES {
                return config_error("schema-2 migration source bytes are not authority-pinned");
            }
            let source: CheckpointContractConfigV2 =
                YamlCodec.deserialize(source_bytes).map_err(map_codec)?;
            source.validate()?;
            let canonical = YamlCodec.serialize(&source).map_err(map_codec)?;
            if canonical.as_slice() != source_bytes {
                return config_error(
                    "schema-2 migration source is not its exact canonical re-encoding",
                );
            }
            Ok(source)
        },
    )
}

fn put_ledger_string(out: &mut Vec<u8>, value: &str) -> Result<(), CheckpointError> {
    let len = u16::try_from(value.len())
        .map_err(|_| CheckpointError::ContractConfig("ledger string too long".to_string()))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn put_optional_ledger_digest(out: &mut Vec<u8>, value: Option<[u8; 32]>) {
    match value {
        Some(digest) => {
            out.push(1);
            out.extend_from_slice(&digest);
        }
        None => out.push(0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ConfigFieldTransformV3 {
    ByteOwnerOnly = 1,
    SemanticPreservingRename = 2,
    SemanticTransform = 3,
    Split = 4,
    Merge = 5,
    Removal = 6,
    Introduced = 7,
    ReservedUnreachable = 8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigV3RenameEntry {
    pub source: String,
    pub source_type: String,
    pub source_value_digest: Option<[u8; 32]>,
    pub destination: String,
    pub destination_type: String,
    pub destination_value_digest: Option<[u8; 32]>,
    pub transform: ConfigFieldTransformV3,
    pub justification: String,
}

/// Digest-bound exhaustive leaf-level schema-2 to schema-3 transform ledger.
/// Every source leaf and every introduced/reserved destination leaf has one
/// canonical entry; missing, extra, duplicate, colliding, or downgraded rows
/// fail before an activation record can be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigV3RenameLedger {
    pub transform_version: u16,
    pub source_config_digest: [u8; 32],
    pub destination_config_digest: [u8; 32],
    pub entries: Vec<ConfigV3RenameEntry>,
    pub digest: [u8; 32],
}

impl ConfigV3RenameLedger {
    pub fn migrate(
        source: &CheckpointContractConfigV2,
        source_bytes: &[u8],
        runtime_profile_manifest_digest: String,
        registry_digest: String,
        config_generation: u64,
        activation_height: u64,
    ) -> Result<(CheckpointContractConfigV3, Self), CheckpointError> {
        let decoded_source = decode_config_v2_migration(source_bytes)?;
        if decoded_source != *source {
            return config_error("typed schema-2 migration source does not match supplied owner");
        }
        let mut destination = embedded_authority_unvalidated().clone();
        destination.runtime_profile.manifest_digest = runtime_profile_manifest_digest;
        destination.version_authority.registry_digest = registry_digest;
        destination.version_authority.config_generation = config_generation;
        destination.version_authority.activation_height = activation_height;
        destination.version_authority.rollback_floor = config_generation;
        destination.validate()?;
        let destination_bytes = YamlCodec.serialize(&destination).map_err(map_codec)?;
        let ledger = Self::from_pair(source_bytes, &destination_bytes)?;
        Ok((destination, ledger))
    }

    fn from_pair(source_bytes: &[u8], destination_bytes: &[u8]) -> Result<Self, CheckpointError> {
        let source_config_digest = sha256_256(
            "z00z.storage.checkpoint.contract-config.v2",
            "canonical_bytes_digest",
            &[source_bytes],
        );
        let destination_config_digest = sha256_256(
            "z00z.storage.checkpoint.contract-config.v3",
            "canonical_bytes_digest",
            &[destination_bytes],
        );
        let entries = rename_entries(source_bytes, destination_bytes)?;
        let mut canonical_entries = Vec::new();
        for entry in &entries {
            put_ledger_string(&mut canonical_entries, &entry.source)?;
            put_ledger_string(&mut canonical_entries, &entry.source_type)?;
            put_optional_ledger_digest(&mut canonical_entries, entry.source_value_digest);
            put_ledger_string(&mut canonical_entries, &entry.destination)?;
            put_ledger_string(&mut canonical_entries, &entry.destination_type)?;
            put_optional_ledger_digest(&mut canonical_entries, entry.destination_value_digest);
            canonical_entries.push(entry.transform as u8);
            put_ledger_string(&mut canonical_entries, &entry.justification)?;
        }
        let digest = sha256_256(
            "z00z.storage.checkpoint.config-v3-rename-ledger.v2",
            "rename_ledger_digest",
            &[
                &CONFIG_V3_TRANSFORM_VERSION.to_le_bytes(),
                &source_config_digest,
                &destination_config_digest,
                &canonical_entries,
            ],
        );
        Ok(Self {
            transform_version: CONFIG_V3_TRANSFORM_VERSION,
            source_config_digest,
            destination_config_digest,
            entries,
            digest,
        })
    }

    fn validate_pair(
        &self,
        source_bytes: &[u8],
        destination_bytes: &[u8],
    ) -> Result<(), CheckpointError> {
        self.validate_shape()?;
        if self != &Self::from_pair(source_bytes, destination_bytes)? {
            return config_error("config rename ledger is not the exact canonical transform");
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), CheckpointError> {
        if self.transform_version != CONFIG_V3_TRANSFORM_VERSION
            || self.source_config_digest == [0; 32]
            || self.destination_config_digest == [0; 32]
            || self.digest == [0; 32]
            || self.entries.is_empty()
        {
            return config_error("config rename ledger transform version mismatch");
        }
        let mut sources = BTreeSet::new();
        let mut destinations = BTreeSet::new();
        for entry in &self.entries {
            if entry.source.is_empty()
                || entry.source_type.is_empty()
                || entry.destination.is_empty()
                || entry.destination_type.is_empty()
                || entry.justification.is_empty()
                || entry.source_value_digest == Some([0; 32])
                || entry.destination_value_digest == Some([0; 32])
                || !sources.insert(entry.source.as_str())
                || !destinations.insert(entry.destination.as_str())
            {
                return config_error("config rename ledger has an empty or colliding leaf");
            }
            let source_is_new = entry.source.starts_with("<new:") && entry.source.ends_with('>');
            let source_is_reserved =
                entry.source.starts_with("<reserved:") && entry.source.ends_with('>');
            let destination_is_removed =
                entry.destination.starts_with("<removed:") && entry.destination.ends_with('>');
            if (entry.transform == ConfigFieldTransformV3::Introduced) != source_is_new
                || (entry.transform == ConfigFieldTransformV3::ReservedUnreachable)
                    != source_is_reserved
                || (entry.transform == ConfigFieldTransformV3::Removal) != destination_is_removed
                || (source_is_new || source_is_reserved)
                    && &entry.source[entry.source.find(':').unwrap() + 1..entry.source.len() - 1]
                        != entry.destination.as_str()
                || (source_is_new || source_is_reserved) && entry.source_type != "absent"
                || (source_is_new || source_is_reserved) != entry.source_value_digest.is_none()
                || destination_is_removed
                    && &entry.destination
                        [entry.destination.find(':').unwrap() + 1..entry.destination.len() - 1]
                        != entry.source.as_str()
                || destination_is_removed && entry.destination_type != "absent"
                || destination_is_removed != entry.destination_value_digest.is_none()
            {
                return config_error("config rename ledger leaf classification mismatch");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMigrationRecordV3 {
    pub wire_version: u16,
    pub release_identity: String,
    pub source_schema: u16,
    pub destination_schema: u16,
    pub source_config_bytes: Vec<u8>,
    pub source_bytes_digest: [u8; 32],
    pub destination_bytes_digest: [u8; 32],
    pub destination_config_digest: [u8; 32],
    pub rename_ledger_digest: [u8; 32],
    pub rename_ledger: ConfigV3RenameLedger,
    pub registry_digest: [u8; 32],
    pub runtime_profile_identifier: String,
    pub runtime_profile_generation: u16,
    pub runtime_profile_manifest_digest: [u8; 32],
    pub activation_height: u64,
    pub pre_cutover_authority_generation: u64,
    pub activated_authority_generation: u64,
    pub parameter_generation: u32,
    pub config_generation: u64,
    pub rollback_floor: u64,
    pub chain_context_digest: [u8; 32],
    pub release_manifest_digest: [u8; 32],
}

fn release_manifest_digest_v3(record: &ConfigMigrationRecordV3) -> [u8; 32] {
    sha256_256(
        "z00z.storage.checkpoint.config-migration-release-manifest.v3",
        "release_manifest_digest",
        &[
            record.release_identity.as_bytes(),
            &record.wire_version.to_le_bytes(),
            &record.source_schema.to_le_bytes(),
            &record.destination_schema.to_le_bytes(),
            &record.source_bytes_digest,
            &record.destination_bytes_digest,
            &record.destination_config_digest,
            &record.registry_digest,
            &record.rename_ledger_digest,
            record.runtime_profile_identifier.as_bytes(),
            &record.runtime_profile_generation.to_le_bytes(),
            &record.runtime_profile_manifest_digest,
            &record.pre_cutover_authority_generation.to_le_bytes(),
            &record.activated_authority_generation.to_le_bytes(),
            &record.parameter_generation.to_le_bytes(),
            &record.config_generation.to_le_bytes(),
            &record.activation_height.to_le_bytes(),
            &record.chain_context_digest,
            &record.rollback_floor.to_le_bytes(),
        ],
    )
}

fn authorize_release_manifest_v3(
    record: &mut ConfigMigrationRecordV3,
    expected_release_manifest_digest: [u8; 32],
) -> Result<(), CheckpointError> {
    let derived = release_manifest_digest_v3(record);
    if record.release_identity != EMBEDDED_RELEASE_IDENTITY_V3
        || expected_release_manifest_digest == [0; 32]
        || derived != expected_release_manifest_digest
    {
        return config_error(format!(
            "ConfigV3 release manifest tuple is not authority-pinned: derived={}",
            hex_digest(derived)
        ));
    }
    record.release_manifest_digest = derived;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointConfigHeadV3 {
    pub wire_version: u16,
    pub schema: u16,
    pub config_generation: u64,
    pub authority_generation: u64,
    pub parameter_generation: u32,
    pub activation_height: u64,
    pub rollback_floor: u64,
    pub config_digest: [u8; 32],
    pub registry_digest: [u8; 32],
    pub rename_ledger_digest: [u8; 32],
    pub runtime_profile_identifier: String,
    pub runtime_profile_generation: u16,
    pub runtime_profile_manifest_digest: [u8; 32],
    pub migration_record_digest: [u8; 32],
}

/// One coherent, immutable online view of the ConfigV3 authority tuple.
///
/// The configuration and head are resolved together. Callers cannot combine a
/// configuration from one generation with registry, profile, or authority
/// metadata from another generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveCheckpointConfigV3 {
    head: CheckpointConfigHeadV3,
    config: CheckpointContractConfigV3,
    migration_record: Option<ConfigMigrationRecordV3>,
}

/// Copyable identity captured by long-running operations so they can reject
/// config, registry, profile, or authority rotation at every boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveCheckpointConfigIdentityV3 {
    pub config_generation: u64,
    pub authority_generation: u64,
    pub parameter_generation: u32,
    pub activation_height: u64,
    pub rollback_floor: u64,
    pub config_digest: [u8; 32],
    pub registry_digest: [u8; 32],
    pub runtime_profile_generation: u16,
    pub runtime_profile_manifest_digest: [u8; 32],
}

impl ActiveCheckpointConfigV3 {
    #[must_use]
    pub const fn head(&self) -> &CheckpointConfigHeadV3 {
        &self.head
    }

    #[must_use]
    pub const fn config(&self) -> &CheckpointContractConfigV3 {
        &self.config
    }

    #[must_use]
    pub fn identity(&self) -> ActiveCheckpointConfigIdentityV3 {
        ActiveCheckpointConfigIdentityV3 {
            config_generation: self.head.config_generation,
            authority_generation: self.head.authority_generation,
            parameter_generation: self.head.parameter_generation,
            activation_height: self.head.activation_height,
            rollback_floor: self.head.rollback_floor,
            config_digest: self.head.config_digest,
            registry_digest: self.head.registry_digest,
            runtime_profile_generation: self.head.runtime_profile_generation,
            runtime_profile_manifest_digest: self.head.runtime_profile_manifest_digest,
        }
    }
}

/// Sole online ConfigV3 authority resolver.
///
/// The initial generation is embedded into the executable and therefore
/// cannot change underneath a running process when the repository bootstrap
/// YAML is edited. A validated immutable activation-store generation may
/// replace it atomically for the process through `activate_installed`.
pub struct CheckpointConfigResolverV3;

/// Linear process-local guard preventing ConfigV3 activation while one live
/// recursive evidence attempt is using the captured generation.
pub(crate) struct ActiveConfigEvidenceGuardV3 {
    marker: std::marker::PhantomData<()>,
}

impl Drop for ActiveConfigEvidenceGuardV3 {
    fn drop(&mut self) {
        active_evidence_attempts().fetch_sub(1, Ordering::AcqRel);
    }
}

impl CheckpointConfigResolverV3 {
    pub fn resolve_active() -> Result<Arc<ActiveCheckpointConfigV3>, CheckpointError> {
        active_config_state()?
            .read()
            .map(|active| Arc::clone(&active))
            .map_err(|_| CheckpointError::ContractConfig("active config lock poisoned".to_string()))
    }

    pub(crate) fn begin_evidence_attempt(
        captured: ActiveCheckpointConfigIdentityV3,
    ) -> Result<ActiveConfigEvidenceGuardV3, CheckpointError> {
        Self::require_current(captured)?;
        active_evidence_attempts().fetch_add(1, Ordering::AcqRel);
        if let Err(error) = Self::require_current(captured) {
            active_evidence_attempts().fetch_sub(1, Ordering::AcqRel);
            return Err(error);
        }
        Ok(ActiveConfigEvidenceGuardV3 {
            marker: std::marker::PhantomData,
        })
    }

    /// Atomically activate an already-installed, immutable ConfigV3
    /// generation. The expected release and chain digests are supplied by the
    /// existing operator authority and must equal the executable's independent
    /// pins; proof, network, and candidate bytes cannot authorize themselves.
    pub fn activate_installed(
        store: &ConfigV3ActivationStore,
        expected_release_manifest_digest: [u8; 32],
        expected_chain_context_digest: [u8; 32],
    ) -> Result<Arc<ActiveCheckpointConfigV3>, CheckpointError> {
        if expected_release_manifest_digest == [0; 32] || expected_chain_context_digest == [0; 32] {
            return config_error("active config authority pins are incomplete");
        }
        let candidate = Arc::new(store.load_active()?);
        let record = candidate.migration_record.as_ref().ok_or_else(|| {
            CheckpointError::ContractConfig(
                "installed active config is missing its migration record".to_string(),
            )
        })?;
        if record.release_manifest_digest != expected_release_manifest_digest
            || record.chain_context_digest != expected_chain_context_digest
            || release_manifest_digest_v3(record) != expected_release_manifest_digest
        {
            return config_error("active config authority pin mismatch");
        }

        let mut current = active_config_state()?.write().map_err(|_| {
            CheckpointError::ContractConfig("active config lock poisoned".to_string())
        })?;
        if candidate.head.config_generation < current.head.config_generation
            || candidate.head.authority_generation < current.head.authority_generation
        {
            return config_error("active config generation rollback rejected");
        }
        if candidate.head.config_generation == current.head.config_generation {
            if candidate.head != current.head || candidate.config != current.config {
                return config_error("active config generation ABA replacement rejected");
            }
            return Ok(Arc::clone(&current));
        }
        if active_evidence_attempts().load(Ordering::Acquire) != 0 {
            return config_error("active recursive evidence attempt blocks config activation");
        }
        *current = Arc::clone(&candidate);
        Ok(candidate)
    }

    pub(crate) fn require_current(
        captured: ActiveCheckpointConfigIdentityV3,
    ) -> Result<(), CheckpointError> {
        let current = Self::resolve_active()?;
        if current.identity() != captured {
            return config_error("active config rotated during operation");
        }
        Ok(())
    }
}

fn active_evidence_attempts() -> &'static AtomicU64 {
    static ACTIVE_EVIDENCE_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
    &ACTIVE_EVIDENCE_ATTEMPTS
}

/// Immutable-generation ConfigV3 installer. Authorization is a pre-pinned
/// local release-manifest digest; this module intentionally exposes no network
/// certificate parser or candidate-selected key path.
pub struct ConfigV3ActivationStore {
    root: PathBuf,
}

impl ConfigV3ActivationStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, CheckpointError> {
        let root = root.as_ref().to_path_buf();
        ensure_config_directory(&root)?;
        ensure_config_directory(&root.join("generations"))?;
        Ok(Self { root })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn install(
        &self,
        source: &CheckpointContractConfigV2,
        source_bytes: &[u8],
        destination: &CheckpointContractConfigV3,
        ledger: &ConfigV3RenameLedger,
        pre_cutover_authority_generation: u64,
        release_manifest_digest: [u8; 32],
        chain_context_digest: [u8; 32],
        expected_source_generation: Option<u64>,
    ) -> Result<CheckpointConfigHeadV3, CheckpointError> {
        if release_manifest_digest == [0; 32]
            || chain_context_digest == [0; 32]
            || pre_cutover_authority_generation == 0
        {
            return config_error("config activation authorization is incomplete");
        }
        let decoded_source = decode_config_v2_migration(source_bytes)?;
        if decoded_source != *source {
            return config_error("typed schema-2 migration source does not match supplied owner");
        }
        destination.validate()?;
        let canonical_source = YamlCodec.serialize(source).map_err(map_codec)?;
        let canonical_destination = YamlCodec.serialize(destination).map_err(map_codec)?;
        if canonical_source != source_bytes
            || destination.version_authority.config_generation
                <= expected_source_generation.unwrap_or(0)
            || destination.version_authority.rollback_floor
                > destination.version_authority.config_generation
        {
            return config_error("config migration CAS or canonical bytes mismatch");
        }
        ledger.validate_pair(source_bytes, &canonical_destination)?;

        let lock_path = self.root.join("authority-generation.lock");
        reject_symlink_if_present(&lock_path)?;
        let lock = open_lock_file(&lock_path)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        lock.try_lock_exclusive().map_err(|_| {
            CheckpointError::ContractConfig("config activation lock busy".to_string())
        })?;

        let registry_digest = CheckpointVersionRegistryV2::authority_pinned()?.digest();
        let destination_config_digest = destination.canonical_digest_with_ledger(ledger.digest)?;
        let mut record = ConfigMigrationRecordV3 {
            wire_version: 1,
            release_identity: EMBEDDED_RELEASE_IDENTITY_V3.to_string(),
            source_schema: 2,
            destination_schema: 3,
            source_config_bytes: source_bytes.to_vec(),
            source_bytes_digest: ledger.source_config_digest,
            destination_bytes_digest: ledger.destination_config_digest,
            destination_config_digest,
            rename_ledger_digest: ledger.digest,
            rename_ledger: ledger.clone(),
            registry_digest,
            runtime_profile_identifier: destination.runtime_profile.identifier.clone(),
            runtime_profile_generation: destination.runtime_profile.generation,
            runtime_profile_manifest_digest: decode_digest_hex(
                &destination.runtime_profile.manifest_digest,
            )?,
            activation_height: destination.version_authority.activation_height,
            pre_cutover_authority_generation,
            activated_authority_generation: u64::from(
                destination.version_authority.authority_generation,
            ),
            parameter_generation: destination.version_authority.parameter_generation,
            config_generation: destination.version_authority.config_generation,
            rollback_floor: destination.version_authority.rollback_floor,
            chain_context_digest,
            release_manifest_digest: [0; 32],
        };
        authorize_release_manifest_v3(&mut record, release_manifest_digest)?;
        let record_bytes = encode_local_registry_object(
            RecursiveBoundedObjectV2::ConfigMigrationRecord,
            &record,
            MIGRATION_RECORD_MAX_BYTES_V3,
        )?;
        let migration_record_digest = sha256_256(
            "z00z.storage.checkpoint.config-migration-record.v3",
            "migration_record_digest",
            &[&record_bytes],
        );
        let head = CheckpointConfigHeadV3 {
            wire_version: 1,
            schema: 3,
            config_generation: destination.version_authority.config_generation,
            authority_generation: u64::from(destination.version_authority.authority_generation),
            parameter_generation: destination.version_authority.parameter_generation,
            activation_height: destination.version_authority.activation_height,
            rollback_floor: destination.version_authority.rollback_floor,
            config_digest: destination_config_digest,
            registry_digest,
            rename_ledger_digest: ledger.digest,
            runtime_profile_identifier: destination.runtime_profile.identifier.clone(),
            runtime_profile_generation: destination.runtime_profile.generation,
            runtime_profile_manifest_digest: decode_digest_hex(
                &destination.runtime_profile.manifest_digest,
            )?,
            migration_record_digest,
        };
        let head_bytes = encode_local_registry_object(
            RecursiveBoundedObjectV2::CheckpointConfigHead,
            &head,
            8 * 1024,
        )?;

        let existing = self.load_head_optional()?;
        if existing.as_ref() == Some(&head) {
            let active = self.load_active()?;
            if active.config != *destination || active.migration_record.as_ref() != Some(&record) {
                return config_error("idempotent config activation reload mismatch");
            }
            FileExt::unlock(&lock)
                .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
            return Ok(head);
        }
        match (existing.as_ref(), expected_source_generation) {
            (None, None) => {}
            (Some(current), Some(expected)) if current.config_generation == expected => {}
            _ => return config_error("config head compare-and-swap mismatch"),
        }

        let generation = self
            .root
            .join("generations")
            .join(format!("{:020}", head.config_generation));
        if path_exists_no_follow(&generation)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?
        {
            require_regular_config_directory(&generation)?;
        } else {
            create_dir_all(&generation)
                .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        }
        write_or_validate_immutable(
            &generation.join("checkpoint_contract.yaml"),
            &canonical_destination,
        )?;
        write_or_validate_immutable(&generation.join("migration.bin"), &record_bytes)?;
        sync_directory(&generation)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;

        // Reload and validate the complete immutable candidate before the one
        // atomic head switch.
        let reloaded =
            CheckpointContractConfigV3::load(generation.join("checkpoint_contract.yaml"))?;
        if reloaded != *destination
            || read_local_registry_object::<ConfigMigrationRecordV3, MIGRATION_RECORD_MAX_BYTES_V3>(
                &generation.join("migration.bin"),
                RecursiveBoundedObjectV2::ConfigMigrationRecord,
            )? != record
        {
            return config_error("config generation reload mismatch");
        }
        atomic_replace_synced(&self.root.join("head.bin"), &head_bytes)?;
        let installed = self.load_head_optional()?.ok_or_else(|| {
            CheckpointError::ContractConfig("config head missing after install".to_string())
        })?;
        if installed != head {
            return config_error("config head reload mismatch");
        }
        FileExt::unlock(&lock)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        Ok(head)
    }

    pub fn load_head(&self) -> Result<CheckpointConfigHeadV3, CheckpointError> {
        self.load_head_optional()?.ok_or_else(|| {
            CheckpointError::ContractConfig("config head is not installed".to_string())
        })
    }

    /// Resolve the current head and its exact immutable generation as one
    /// coherent snapshot. A concurrent head switch causes rejection rather
    /// than returning mixed-generation fields.
    pub fn load_active(&self) -> Result<ActiveCheckpointConfigV3, CheckpointError> {
        let head = self.load_head()?;
        let generation = self
            .root
            .join("generations")
            .join(format!("{:020}", head.config_generation));
        require_regular_config_directory(&generation)?;

        let config_path = generation.join("checkpoint_contract.yaml");
        let migration_path = generation.join("migration.bin");
        require_regular_config_file(&config_path)?;
        require_regular_config_file(&migration_path)?;

        let config = CheckpointContractConfigV3::load(&config_path)?;
        let migration_bytes = read_file_bounded(
            &migration_path,
            (MIGRATION_RECORD_MAX_BYTES_V3 + RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + 1) as u64,
        )
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        let migration_record = decode_local_registry_object_bytes::<
            ConfigMigrationRecordV3,
            MIGRATION_RECORD_MAX_BYTES_V3,
        >(
            &migration_bytes,
            RecursiveBoundedObjectV2::ConfigMigrationRecord,
        )?;
        validate_active_snapshot(&head, &config, &migration_record, &migration_bytes)?;

        if self.load_head()? != head {
            return config_error("config head rotated while resolving active generation");
        }
        Ok(ActiveCheckpointConfigV3 {
            head,
            config,
            migration_record: Some(migration_record),
        })
    }

    fn load_head_optional(&self) -> Result<Option<CheckpointConfigHeadV3>, CheckpointError> {
        let path = self.root.join("head.bin");
        if !path_exists_no_follow(&path)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?
        {
            return Ok(None);
        }
        let metadata = symlink_metadata(&path)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return config_error("config head must be a regular non-symlink file");
        }
        Ok(Some(read_local_registry_object::<
            CheckpointConfigHeadV3,
            8192,
        >(
            &path,
            RecursiveBoundedObjectV2::CheckpointConfigHead,
        )?))
    }
}

fn active_config_state() -> Result<&'static RwLock<Arc<ActiveCheckpointConfigV3>>, CheckpointError>
{
    static ACTIVE: OnceLock<Result<RwLock<Arc<ActiveCheckpointConfigV3>>, String>> =
        OnceLock::new();
    match ACTIVE.get_or_init(|| {
        build_embedded_active_config()
            .map(|snapshot| RwLock::new(Arc::new(snapshot)))
            .map_err(|error| error.to_string())
    }) {
        Ok(state) => Ok(state),
        Err(message) => Err(CheckpointError::ContractConfig(message.clone())),
    }
}

fn build_embedded_active_config() -> Result<ActiveCheckpointConfigV3, CheckpointError> {
    const BOOTSTRAP_BYTES: &[u8] = include_bytes!("checkpoint_contract.yaml");
    let config = CheckpointVersionRegistryV2::authority_pinned()?.decode_config_schema(
        RecursiveBoundedObjectV2::CheckpointContractConfigV3,
        BOOTSTRAP_BYTES,
        RegistryOperationV2::Read,
        CheckpointContractConfigV3::decode_canonical_bytes,
    )?;

    let registry_digest = CheckpointVersionRegistryV2::authority_pinned()?.digest();
    let runtime_profile_manifest_digest =
        decode_digest_hex(&config.runtime_profile.manifest_digest)?;
    let destination_bytes = YamlCodec.serialize(&config).map_err(map_codec)?;
    let source = decode_config_v2_migration(CONFIG_V2_MIGRATION_BYTES)?;
    if YamlCodec.serialize(&source).map_err(map_codec)?.as_slice() != CONFIG_V2_MIGRATION_BYTES {
        return config_error("embedded schema-2 migration source changed during typed decode");
    }
    let ledger = ConfigV3RenameLedger::from_pair(CONFIG_V2_MIGRATION_BYTES, &destination_bytes)?;
    let config_digest = config.canonical_digest_with_ledger(ledger.digest)?;
    let mut record = ConfigMigrationRecordV3 {
        wire_version: 1,
        release_identity: EMBEDDED_RELEASE_IDENTITY_V3.to_string(),
        source_schema: 2,
        destination_schema: 3,
        source_config_bytes: CONFIG_V2_MIGRATION_BYTES.to_vec(),
        source_bytes_digest: ledger.source_config_digest,
        destination_bytes_digest: ledger.destination_config_digest,
        destination_config_digest: config_digest,
        rename_ledger_digest: ledger.digest,
        rename_ledger: ledger.clone(),
        registry_digest,
        runtime_profile_identifier: config.runtime_profile.identifier.clone(),
        runtime_profile_generation: config.runtime_profile.generation,
        runtime_profile_manifest_digest,
        activation_height: config.version_authority.activation_height,
        pre_cutover_authority_generation: 1,
        activated_authority_generation: u64::from(config.version_authority.authority_generation),
        parameter_generation: config.version_authority.parameter_generation,
        config_generation: config.version_authority.config_generation,
        rollback_floor: config.version_authority.rollback_floor,
        chain_context_digest: embedded_chain_context_digest_v3(),
        release_manifest_digest: [0; 32],
    };
    authorize_release_manifest_v3(&mut record, EMBEDDED_RELEASE_MANIFEST_DIGEST_V3)?;
    let record_bytes = encode_local_registry_object(
        RecursiveBoundedObjectV2::ConfigMigrationRecord,
        &record,
        MIGRATION_RECORD_MAX_BYTES_V3,
    )?;
    let migration_record_digest = sha256_256(
        "z00z.storage.checkpoint.config-migration-record.v3",
        "migration_record_digest",
        &[&record_bytes],
    );
    let head = CheckpointConfigHeadV3 {
        wire_version: 1,
        schema: 3,
        config_generation: config.version_authority.config_generation,
        authority_generation: u64::from(config.version_authority.authority_generation),
        parameter_generation: config.version_authority.parameter_generation,
        activation_height: config.version_authority.activation_height,
        rollback_floor: config.version_authority.rollback_floor,
        config_digest,
        registry_digest,
        rename_ledger_digest: ledger.digest,
        runtime_profile_identifier: config.runtime_profile.identifier.clone(),
        runtime_profile_generation: config.runtime_profile.generation,
        runtime_profile_manifest_digest,
        migration_record_digest,
    };
    validate_active_snapshot(&head, &config, &record, &record_bytes)?;
    Ok(ActiveCheckpointConfigV3 {
        head,
        config,
        migration_record: Some(record),
    })
}

fn validate_active_snapshot(
    head: &CheckpointConfigHeadV3,
    config: &CheckpointContractConfigV3,
    record: &ConfigMigrationRecordV3,
    record_bytes: &[u8],
) -> Result<(), CheckpointError> {
    validate_head_config_tuple(head, config)?;
    let _typed_source = decode_config_v2_migration(&record.source_config_bytes)?;
    let canonical_config = YamlCodec.serialize(config).map_err(map_codec)?;
    record
        .rename_ledger
        .validate_pair(&record.source_config_bytes, &canonical_config)?;
    let source_bytes_digest = sha256_256(
        "z00z.storage.checkpoint.contract-config.v2",
        "canonical_bytes_digest",
        &[&record.source_config_bytes],
    );
    let destination_bytes_digest = sha256_256(
        "z00z.storage.checkpoint.contract-config.v3",
        "canonical_bytes_digest",
        &[&canonical_config],
    );
    let migration_record_digest = sha256_256(
        "z00z.storage.checkpoint.config-migration-record.v3",
        "migration_record_digest",
        &[record_bytes],
    );
    if record.wire_version != 1
        || record.release_identity != EMBEDDED_RELEASE_IDENTITY_V3
        || record.source_schema != 2
        || record.destination_schema != 3
        || record.source_config_bytes.is_empty()
        || record.source_bytes_digest != source_bytes_digest
        || record.destination_bytes_digest != destination_bytes_digest
        || record.destination_config_digest != head.config_digest
        || record.rename_ledger_digest != head.rename_ledger_digest
        || record.rename_ledger.digest != record.rename_ledger_digest
        || record.rename_ledger.source_config_digest != record.source_bytes_digest
        || record.rename_ledger.destination_config_digest != record.destination_bytes_digest
        || record.registry_digest != head.registry_digest
        || record.runtime_profile_identifier != head.runtime_profile_identifier
        || record.runtime_profile_generation != head.runtime_profile_generation
        || record.runtime_profile_manifest_digest != head.runtime_profile_manifest_digest
        || record.activation_height != head.activation_height
        || record.activated_authority_generation != head.authority_generation
        || record.parameter_generation != head.parameter_generation
        || record.config_generation != head.config_generation
        || record.rollback_floor != head.rollback_floor
        || record.pre_cutover_authority_generation == 0
        || record.chain_context_digest == [0; 32]
        || record.release_manifest_digest == [0; 32]
        || release_manifest_digest_v3(record) != record.release_manifest_digest
        || migration_record_digest != head.migration_record_digest
    {
        return config_error("active ConfigV3 migration/head tuple mismatch");
    }
    Ok(())
}

fn validate_head_config_tuple(
    head: &CheckpointConfigHeadV3,
    config: &CheckpointContractConfigV3,
) -> Result<(), CheckpointError> {
    let registry_digest = CheckpointVersionRegistryV2::authority_pinned()?.digest();
    if head.wire_version != 1
        || head.schema != 3
        || head.config_generation != config.version_authority.config_generation
        || head.authority_generation != u64::from(config.version_authority.authority_generation)
        || head.activation_height != config.version_authority.activation_height
        || head.rollback_floor != config.version_authority.rollback_floor
        || head.rollback_floor > head.config_generation
        || head.config_digest != config.canonical_digest_with_ledger(head.rename_ledger_digest)?
        || head.registry_digest != registry_digest
        || head.runtime_profile_identifier != config.runtime_profile.identifier
        || head.runtime_profile_generation != config.runtime_profile.generation
        || head.runtime_profile_manifest_digest
            != decode_digest_hex(&config.runtime_profile.manifest_digest)?
        || head.parameter_generation != config.version_authority.parameter_generation
        || head.rename_ledger_digest == [0; 32]
        || head.migration_record_digest == [0; 32]
    {
        return config_error("active ConfigV3 head/config tuple mismatch");
    }
    Ok(())
}

fn encode_local_registry_object<T: Serialize>(
    object: RecursiveBoundedObjectV2,
    value: &T,
    cap: usize,
) -> Result<Vec<u8>, CheckpointError> {
    let payload = BincodeCodec.serialize(value).map_err(map_codec)?;
    if payload.len() > cap {
        return config_error("local config object exceeds cap");
    }
    let header =
        CheckpointVersionRegistryV2::authority_pinned()?.encode_preheader(object, payload.len())?;
    let mut bytes = Vec::with_capacity(header.len() + payload.len());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

fn read_local_registry_object<T, const CAP: usize>(
    path: &Path,
    object: RecursiveBoundedObjectV2,
) -> Result<T, CheckpointError>
where
    T: serde::de::DeserializeOwned + Serialize,
{
    let bytes = read_file_bounded(path, (CAP + RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + 1) as u64)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
    decode_local_registry_object_bytes::<T, CAP>(&bytes, object)
}

fn decode_local_registry_object_bytes<T, const CAP: usize>(
    bytes: &[u8],
    object: RecursiveBoundedObjectV2,
) -> Result<T, CheckpointError>
where
    T: serde::de::DeserializeOwned + Serialize,
{
    let validated =
        CheckpointVersionRegistryV2::authority_pinned()?.validate_preheader(bytes, object)?;
    let value = BincodeCodec
        .deserialize_bounded_const::<T, CAP>(&bytes[validated.header_len..])
        .map_err(map_codec)?;
    if encode_local_registry_object(object, &value, CAP)?.as_slice() != bytes {
        return config_error("local config object is not canonical");
    }
    Ok(value)
}

fn ensure_config_directory(path: &Path) -> Result<(), CheckpointError> {
    if path_exists_no_follow(path)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?
    {
        let metadata = symlink_metadata(path)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            return config_error("config store path must be a non-symlink directory");
        }
    } else {
        create_dir_all(path).map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
    }
    Ok(())
}

fn require_regular_config_directory(path: &Path) -> Result<(), CheckpointError> {
    let metadata = symlink_metadata(path)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return config_error("active config generation must be a non-symlink directory");
    }
    Ok(())
}

fn require_regular_config_file(path: &Path) -> Result<(), CheckpointError> {
    let metadata = symlink_metadata(path)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return config_error("active config member must be a regular non-symlink file");
    }
    Ok(())
}

fn reject_symlink_if_present(path: &Path) -> Result<(), CheckpointError> {
    if !path_exists_no_follow(path)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?
    {
        return Ok(());
    }
    let metadata = symlink_metadata(path)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return config_error("config store symlink rejected");
    }
    Ok(())
}

fn write_or_validate_immutable(path: &Path, expected: &[u8]) -> Result<(), CheckpointError> {
    if path_exists_no_follow(path)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?
    {
        require_regular_config_file(path)?;
        let max = u64::try_from(expected.len())
            .map_err(|_| CheckpointError::ContractConfig("config member length overflow".into()))?
            .saturating_add(1);
        let existing = read_file_bounded(path, max)
            .map_err(|error| CheckpointError::ContractConfig(error.to_string()))?;
        if existing != expected {
            return config_error("immutable config generation member mismatch");
        }
        return Ok(());
    }
    atomic_replace_synced(path, expected)
}

fn atomic_replace_synced(path: &Path, bytes: &[u8]) -> Result<(), CheckpointError> {
    reject_symlink_if_present(path)?;
    atomic_write_file_private(path, bytes)
        .map_err(|error| CheckpointError::ContractConfig(error.to_string()))
}

fn rename_entries(
    source_bytes: &[u8],
    destination_bytes: &[u8],
) -> Result<Vec<ConfigV3RenameEntry>, CheckpointError> {
    let source_yaml: YamlValue = YamlCodec.deserialize(source_bytes).map_err(map_codec)?;
    let destination_yaml: YamlValue = YamlCodec
        .deserialize(destination_bytes)
        .map_err(map_codec)?;
    let mut source_leaves = BTreeMap::new();
    let mut destination_leaves = BTreeMap::new();
    flatten_yaml_leaves(&source_yaml, "", &mut source_leaves)?;
    flatten_yaml_leaves(&destination_yaml, "", &mut destination_leaves)?;

    let renames: BTreeMap<&str, &str> = [
        (
            "authority_promotion.recursive_authority_allowed",
            "authority_promotion.is_recursive_authority_allowed",
        ),
        (
            "authority_promotion.verified_backend_allowed",
            "authority_promotion.is_verified_backend_allowed",
        ),
        (
            "branches.nova.must_bind_statement_digest",
            "branches.nova.has_statement_digest_bind",
        ),
        (
            "branches.nova.must_bind_checkpoint_link",
            "branches.nova.has_checkpoint_link_bind",
        ),
        (
            "branches.nova.retain_until_pq_epoch",
            "branches.nova.proof_body_retention",
        ),
        (
            "branches.plonky3_epoch.provides_pq_epoch_evidence",
            "branches.plonky3_epoch.has_pq_epoch_evidence",
        ),
        (
            "branches.plonky3_epoch.must_prove_canonical_transition_range",
            "branches.plonky3_epoch.has_transition_range_proof",
        ),
        (
            "branches.plonky3_epoch.may_bind_nova_chain_root",
            "branches.plonky3_epoch.has_nova_chain_bind",
        ),
        (
            "branches.plonky3_epoch.must_not_depend_only_on_nova",
            "branches.plonky3_epoch.has_independent_transition_proof",
        ),
        (
            "gates.outputs.has_archive_manifest",
            "gates.outputs.has_challenge_content_commitment",
        ),
        (
            "gates.artifacts.has_pq_anchor_on_cadence",
            "gates.artifacts.has_async_pq_anchor",
        ),
        (
            "archive_retention.celestia_is_da_only",
            "archive_retention.is_celestia_da_only",
        ),
        (
            "archive_retention.content_addressing_required",
            "archive_retention.has_content_addressing",
        ),
        (
            "archive_retention.ipfs_pinning_required",
            "archive_retention.has_ipfs_pinning",
        ),
        (
            "archive_retention.provider_receipts_required",
            "archive_retention.has_provider_receipts",
        ),
        (
            "archive_retention.retrieval_audit_required",
            "archive_retention.has_retrieval_audit",
        ),
        (
            "archive_retention.retrievability_is_not_validity",
            "archive_retention.is_retrieval_non_authority",
        ),
        (
            "post_quantum.enforce_live_cadence",
            "post_quantum.is_live_cadence_enforced",
        ),
        (
            "snapshots.bootstrap_allowed_from_snapshot",
            "snapshots.is_snapshot_bootstrap_allowed",
        ),
        (
            "snapshots.requires_retrieval_audit",
            "snapshots.has_retrieval_audit",
        ),
        (
            "snapshots.must_bind_state_root",
            "snapshots.has_state_root_bind",
        ),
        (
            "snapshots.must_bind_settlement_root",
            "snapshots.has_settlement_root_bind",
        ),
        (
            "snapshots.must_bind_last_plonky3_epoch_proof",
            "snapshots.has_epoch_proof_bind",
        ),
        (
            "snapshots.must_bind_last_epoch_manifest_root",
            "snapshots.has_epoch_manifest_bind",
        ),
        (
            "snapshots.must_bind_archive_manifest_root",
            "snapshots.has_archive_manifest_bind",
        ),
        (
            "snapshots.must_bind_snapshot_chunk_root",
            "snapshots.has_snapshot_chunk_bind",
        ),
        (
            "snapshots.must_bind_pq_anchor_root",
            "snapshots.has_pq_anchor_bind",
        ),
        (
            "pruning.full_node_pruning_allowed",
            "pruning.is_full_node_pruning_allowed",
        ),
        (
            "pruning.archive_node_pruning_allowed",
            "pruning.is_legacy_pruning_allowed",
        ),
        (
            "pruning.requires_dispute_window_elapsed",
            "pruning.is_window_expired",
        ),
        (
            "pruning.requires_plonky3_epoch_finalized",
            "pruning.is_epoch_proof_verified",
        ),
        (
            "pruning.requires_archive_replication_threshold_met",
            "pruning.has_archive_replication_quorum",
        ),
        (
            "pruning.requires_retrieval_audit_passed",
            "pruning.has_passing_retrieval_audit",
        ),
        (
            "pruning.must_keep_compact_metadata",
            "pruning.has_compact_metadata_retention",
        ),
        (
            "retention.epoch_manifests",
            "retention.epoch_manifest_bodies",
        ),
        (
            "documentation.include_source_disposition",
            "documentation.has_source_disposition",
        ),
        (
            "documentation.include_object_schemas",
            "documentation.has_object_schemas",
        ),
        (
            "documentation.include_golden_vectors",
            "documentation.has_golden_vectors",
        ),
        (
            "documentation.include_chain_evidence_ids",
            "documentation.has_chain_evidence_ids",
        ),
        (
            "documentation.include_measurements",
            "documentation.has_measurements",
        ),
        (
            "documentation.include_pq_cadence_evidence",
            "documentation.has_pq_cadence_evidence",
        ),
        (
            "documentation.include_backend_manifest",
            "documentation.has_backend_manifest",
        ),
        (
            "documentation.include_rejected_claim_register",
            "documentation.has_rejected_claim_register",
        ),
    ]
    .into_iter()
    .collect();

    let mut entries = Vec::new();
    let mut covered_destinations = BTreeSet::new();
    for (source, source_value) in &source_leaves {
        let destination = if destination_leaves.contains_key(source) {
            Some(source.as_str())
        } else {
            renames.get(source.as_str()).copied()
        };
        if let Some(destination) = destination {
            let destination_value = destination_leaves.get(destination).ok_or_else(|| {
                CheckpointError::ContractConfig(format!(
                    "rename ledger destination is absent: {destination}"
                ))
            })?;
            if !covered_destinations.insert(destination.to_string()) {
                return config_error("rename ledger destination collision");
            }
            let transform = if source == destination {
                if source_value == destination_value {
                    ConfigFieldTransformV3::ByteOwnerOnly
                } else {
                    ConfigFieldTransformV3::SemanticTransform
                }
            } else if source_value == destination_value {
                ConfigFieldTransformV3::SemanticPreservingRename
            } else {
                ConfigFieldTransformV3::SemanticTransform
            };
            entries.push(typed_ledger_entry(
                source,
                yaml_leaf_type(source_value),
                Some(source_value),
                destination,
                yaml_leaf_type(destination_value),
                Some(destination_value),
                transform,
            )?);
        } else {
            entries.push(typed_ledger_entry(
                source,
                yaml_leaf_type(source_value),
                Some(source_value),
                &format!("<removed:{source}>"),
                "absent",
                None,
                ConfigFieldTransformV3::Removal,
            )?);
        }
    }
    for (destination, destination_value) in &destination_leaves {
        if covered_destinations.contains(destination) {
            continue;
        }
        let transform = if destination.starts_with("offline_receipt_mailbox.") {
            ConfigFieldTransformV3::ReservedUnreachable
        } else {
            ConfigFieldTransformV3::Introduced
        };
        let source = match transform {
            ConfigFieldTransformV3::ReservedUnreachable => {
                format!("<reserved:{destination}>")
            }
            _ => format!("<new:{destination}>"),
        };
        entries.push(typed_ledger_entry(
            &source,
            "absent",
            None,
            destination,
            yaml_leaf_type(destination_value),
            Some(destination_value),
            transform,
        )?);
    }
    Ok(entries)
}

fn flatten_yaml_leaves(
    value: &YamlValue,
    prefix: &str,
    out: &mut BTreeMap<String, YamlValue>,
) -> Result<(), CheckpointError> {
    match value {
        YamlValue::Mapping(mapping) => {
            for (key, value) in mapping {
                let key = key.as_str().ok_or_else(|| {
                    CheckpointError::ContractConfig(
                        "config ledger encountered a non-string key".to_string(),
                    )
                })?;
                let path = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_yaml_leaves(value, &path, out)?;
            }
            Ok(())
        }
        YamlValue::Tagged(_) | YamlValue::Null => {
            config_error("config ledger encountered a tagged or null authority leaf")
        }
        _ => {
            if prefix.is_empty() || out.insert(prefix.to_string(), value.clone()).is_some() {
                return config_error("config ledger encountered an empty or duplicate leaf");
            }
            Ok(())
        }
    }
}

fn yaml_leaf_type(value: &YamlValue) -> &'static str {
    match value {
        YamlValue::Bool(_) => "bool",
        YamlValue::Number(_) => "integer",
        YamlValue::String(_) => "string",
        YamlValue::Sequence(_) => "string_list",
        YamlValue::Mapping(_) => "mapping",
        YamlValue::Null => "null",
        YamlValue::Tagged(_) => "tagged",
    }
}

fn typed_ledger_entry(
    source: &str,
    source_type: &str,
    source_value: Option<&YamlValue>,
    destination: &str,
    destination_type: &str,
    destination_value: Option<&YamlValue>,
    transform: ConfigFieldTransformV3,
) -> Result<ConfigV3RenameEntry, CheckpointError> {
    let justification = match transform {
        ConfigFieldTransformV3::ByteOwnerOnly => "schema-2 leaf retains its exact byte owner",
        ConfigFieldTransformV3::SemanticPreservingRename => {
            "schema-2 leaf moves to one semantically identical owner"
        }
        ConfigFieldTransformV3::SemanticTransform => {
            "authority-reviewed schema-2 to schema-3 semantic transform"
        }
        ConfigFieldTransformV3::Split => "schema-2 leaf is split into explicit schema-3 owners",
        ConfigFieldTransformV3::Merge => "schema-2 leaves merge into one schema-3 owner",
        ConfigFieldTransformV3::Removal => "schema-2 leaf is explicitly removed",
        ConfigFieldTransformV3::Introduced => "new schema-3 authority leaf",
        ConfigFieldTransformV3::ReservedUnreachable => {
            "Phase-071 leaf reserved unreachable by schema 3"
        }
    };
    Ok(ConfigV3RenameEntry {
        source: source.to_string(),
        source_type: source_type.to_string(),
        source_value_digest: source_value.map(yaml_leaf_digest).transpose()?,
        destination: destination.to_string(),
        destination_type: destination_type.to_string(),
        destination_value_digest: destination_value.map(yaml_leaf_digest).transpose()?,
        transform,
        justification: justification.to_string(),
    })
}

fn yaml_leaf_digest(value: &YamlValue) -> Result<[u8; 32], CheckpointError> {
    let bytes = YamlCodec.serialize(value).map_err(map_codec)?;
    Ok(sha256_256(
        "z00z.storage.checkpoint.config-v3-rename-ledger.v2",
        "leaf_value_digest",
        &[&bytes],
    ))
}

fn reject_noncanonical_yaml_features(bytes: &[u8]) -> Result<(), CheckpointError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| CheckpointError::ContractConfig("schema-3 YAML is not UTF-8".to_string()))?;
    if !text.ends_with('\n') || text.ends_with("\n\n") || text.contains("\r") {
        return config_error("schema-3 YAML must use the single-LF EOF policy");
    }
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("---")
            || trimmed.starts_with("...")
            || trimmed.starts_with('!')
            || trimmed.contains("<<:")
            || trimmed.contains(" &")
            || trimmed.contains(" *")
        {
            return config_error("schema-3 YAML alias/tag/merge/document syntax is forbidden");
        }
    }
    Ok(())
}

fn is_digest_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0')
}

fn decode_digest_hex(value: &str) -> Result<[u8; 32], CheckpointError> {
    if !is_digest_hex(value) {
        return config_error("digest must be non-zero lowercase hex");
    }
    let mut out = [0u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        out[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Ok(out)
}

fn hex_nibble(value: u8) -> Result<u8, CheckpointError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => config_error("invalid lowercase hex digest"),
    }
}

pub(crate) fn hex_digest(digest: [u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn map_codec(error: z00z_utils::codec::CodecError) -> CheckpointError {
    CheckpointError::ContractConfig(error.to_string())
}

fn config_error<T>(message: impl Into<String>) -> Result<T, CheckpointError> {
    Err(CheckpointError::ContractConfig(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_RELEASE_MANIFEST_DIGEST_V3: [u8; 32] = [
        0x3c, 0x18, 0x60, 0x7e, 0xe0, 0x1d, 0x4d, 0x03, 0x5a, 0xce, 0x2c, 0x42, 0x2e, 0x32, 0x09,
        0x5f, 0x9c, 0x9c, 0x67, 0x25, 0x3e, 0xc8, 0x9f, 0xa6, 0x85, 0x21, 0xbc, 0x0b, 0xc0, 0xc1,
        0xc7, 0x8c,
    ];

    fn migration_source_v2() -> (CheckpointContractConfigV2, Vec<u8>) {
        let source_bytes = CONFIG_V2_MIGRATION_BYTES.to_vec();
        let source: CheckpointContractConfigV2 = YamlCodec.deserialize(&source_bytes).unwrap();
        source.validate().unwrap();
        let canonical = YamlCodec.serialize(&source).unwrap();
        if canonical != source_bytes {
            let mismatch = canonical
                .iter()
                .zip(&source_bytes)
                .position(|(left, right)| left != right)
                .unwrap_or(canonical.len().min(source_bytes.len()));
            let start = mismatch.saturating_sub(80);
            let canonical_end = (mismatch + 160).min(canonical.len());
            let source_end = (mismatch + 160).min(source_bytes.len());
            panic!(
                "schema-2 canonical YAML mismatch at byte {mismatch}: canonical={:?}, source={:?}",
                String::from_utf8_lossy(&canonical[start..canonical_end]),
                String::from_utf8_lossy(&source_bytes[start..source_end]),
            );
        }
        (source, source_bytes)
    }

    #[test]
    fn test_v3_default_registry_bound() {
        let cfg = CheckpointContractConfigV3::load_repo_default().unwrap();
        assert!(cfg.is_v3());
        assert_eq!(
            cfg.current_state_sharding,
            CurrentStateShardingCfgV3::authority_pinned()
        );
        assert_eq!(
            cfg.offline_receipt_mailbox,
            OfflineReceiptMailboxCfgV3::authority_pinned()
        );
        assert_eq!(cfg.offline_receipt_mailbox.max_admission_bytes_per_block, 0);
        assert_ne!(cfg.canonical_digest().unwrap(), [0u8; 32]);
    }

    #[test]
    fn test_migration_source_is_bound() {
        let (source, source_bytes) = migration_source_v2();
        assert_eq!(decode_config_v2_migration(&source_bytes).unwrap(), source);
        assert_eq!(source.version, 2);
        assert_eq!(source_bytes, CONFIG_V2_MIGRATION_BYTES);
        assert_eq!(
            sha256_256(
                "z00z.storage.checkpoint.contract-config.v2",
                "canonical_bytes_digest",
                &[&source_bytes],
            ),
            decode_digest_hex("2a7484600c9056fedb6fa850edbbd284b1faec75d8fbfdd1f50d69c860802d08")
                .unwrap(),
        );
        let active = build_embedded_active_config().unwrap();
        let record = active.migration_record.unwrap();
        assert_eq!(record.source_config_bytes, source_bytes);
        assert_eq!(
            record.source_bytes_digest,
            record.rename_ledger.source_config_digest
        );

        let mut noncanonical = source_bytes;
        noncanonical.push(b'\n');
        assert!(decode_config_v2_migration(&noncanonical).is_err());
    }

    #[test]
    fn test_release_manifest_binds_authority() {
        let active = build_embedded_active_config().unwrap();
        let record = active.migration_record.clone().unwrap();
        let baseline = release_manifest_digest_v3(&record);
        assert_eq!(baseline, EMBEDDED_RELEASE_MANIFEST_DIGEST_V3);
        assert_eq!(record.release_manifest_digest, baseline);

        let mutations: [fn(&mut ConfigMigrationRecordV3); 19] = [
            |value| value.release_identity.push_str("-drift"),
            |value| value.wire_version += 1,
            |value| value.source_schema += 1,
            |value| value.destination_schema += 1,
            |value| value.source_bytes_digest[0] ^= 1,
            |value| value.destination_bytes_digest[0] ^= 1,
            |value| value.destination_config_digest[0] ^= 1,
            |value| value.registry_digest[0] ^= 1,
            |value| value.rename_ledger_digest[0] ^= 1,
            |value| value.runtime_profile_identifier.push_str("-drift"),
            |value| value.runtime_profile_generation += 1,
            |value| value.runtime_profile_manifest_digest[0] ^= 1,
            |value| value.pre_cutover_authority_generation += 1,
            |value| value.activated_authority_generation += 1,
            |value| value.parameter_generation += 1,
            |value| value.config_generation += 1,
            |value| value.activation_height += 1,
            |value| value.chain_context_digest[0] ^= 1,
            |value| value.rollback_floor += 1,
        ];
        for mutate in mutations {
            let mut changed = record.clone();
            mutate(&mut changed);
            assert_ne!(release_manifest_digest_v3(&changed), baseline);
            assert!(authorize_release_manifest_v3(
                &mut changed,
                EMBEDDED_RELEASE_MANIFEST_DIGEST_V3,
            )
            .is_err());
        }

        let mut changed_source_yaml = record.clone();
        changed_source_yaml.source_config_bytes.push(b'\n');
        changed_source_yaml.source_bytes_digest = sha256_256(
            "z00z.storage.checkpoint.contract-config.v2",
            "canonical_bytes_digest",
            &[&changed_source_yaml.source_config_bytes],
        );
        assert_ne!(release_manifest_digest_v3(&changed_source_yaml), baseline);
        assert!(decode_config_v2_migration(&changed_source_yaml.source_config_bytes).is_err());

        let mut destination_yaml = YamlCodec.serialize(active.config()).unwrap();
        destination_yaml.push(b'\n');
        assert!(CheckpointContractConfigV3::decode_canonical_bytes(&destination_yaml).is_err());
        let mut changed_destination_yaml = record.clone();
        changed_destination_yaml.destination_bytes_digest = sha256_256(
            "z00z.storage.checkpoint.contract-config.v3",
            "canonical_bytes_digest",
            &[&destination_yaml],
        );
        assert_ne!(
            release_manifest_digest_v3(&changed_destination_yaml),
            baseline
        );

        let mut wrong_expected = baseline;
        wrong_expected[0] ^= 1;
        let mut candidate = record;
        assert!(authorize_release_manifest_v3(&mut candidate, wrong_expected).is_err());
    }

    #[test]
    fn test_sharding_rejects_leaf_mutations() {
        let mutations: &[fn(&mut CurrentStateShardingCfgV3)] = &[
            |value| value.is_required = false,
            |value| value.shard_count += 1,
            |value| value.replication_factor += 1,
            |value| value.write_quorum += 1,
            |value| value.read_quorum += 1,
            |value| value.min_failure_domains += 1,
            |value| value.is_full_state_replica_allowed = true,
            |value| value.route_key_profile.push_str("_drift"),
            |value| value.rollout_profile.push_str("_drift"),
            |value| value.has_seed_recovery = false,
        ];
        for mutate in mutations {
            let mut cfg = CheckpointContractConfigV3::load_repo_default().unwrap();
            mutate(&mut cfg.current_state_sharding);
            assert!(cfg.validate().is_err());
        }
    }

    #[test]
    fn test_mailbox_rejects_leaf_mutations() {
        let mutations: &[fn(&mut OfflineReceiptMailboxCfgV3)] = &[
            |value| value.is_required = false,
            |value| value.is_runtime_enabled = true,
            |value| value.semantic_owner_phase = 69,
            |value| value.phase_069_role.push_str("_drift"),
            |value| value.admission_stage.push_str("_drift"),
            |value| value.max_admission_bytes_per_block = 1,
            |value| value.max_partition_block_admission_bytes = 1,
            |value| value.object_type.push_str("_drift"),
            |value| value.retention_blocks += 1,
            |value| value.retention_start.push_str("_drift"),
            |value| value.staging_expiry.push_str("_drift"),
            |value| value.max_notice_plaintext_bytes += 1,
            |value| value.max_entry_bytes += 1,
            |value| value.max_entries_per_recipient_output += 1,
            |value| value.is_recipient_capability_required = false,
            |value| value.is_sender_local_policy_required = false,
            |value| value.has_payment_request_v1_capability = true,
            |value| value.logical_partition_count += 1,
            |value| value.partitions_per_entry += 1,
            |value| value.is_cross_partition_fanout_allowed = true,
            |value| value.has_adversarial_uniformity_claim = true,
            |value| value.replication_factor += 1,
            |value| value.write_quorum += 1,
            |value| value.read_quorum += 1,
            |value| value.min_failure_domains += 1,
            |value| value.is_public_listing_allowed = true,
            |value| value.is_public_dht_publication_allowed = true,
            |value| value.has_ack_early_gc = false,
            |value| value.has_seed_recovery_fallback = false,
            |value| value.is_sender_ack_retention_required = true,
        ];
        for mutate in mutations {
            let mut cfg = CheckpointContractConfigV3::load_repo_default().unwrap();
            mutate(&mut cfg.offline_receipt_mailbox);
            assert!(cfg.validate().is_err());
        }
    }

    #[test]
    fn test_resolver_returns_coherent_head() {
        let active = CheckpointConfigResolverV3::resolve_active().unwrap();
        let identity = active.identity();
        assert_eq!(
            identity.config_digest,
            active.config().canonical_digest().unwrap()
        );
        assert_eq!(
            identity.config_generation,
            active.config().version_authority.config_generation
        );
        assert_eq!(
            identity.authority_generation,
            u64::from(active.config().version_authority.authority_generation)
        );
        assert_eq!(
            identity.registry_digest,
            CheckpointVersionRegistryV2::authority_pinned()
                .unwrap()
                .digest()
        );
        CheckpointConfigResolverV3::require_current(identity).unwrap();
    }

    #[test]
    fn test_paths_stay_under_root() {
        let cfg = CheckpointContractConfigV3::load_repo_default().unwrap();
        let root = PathBuf::from("crates/z00z_storage/outputs/checkpoint");
        let resolved = cfg.resolve_paths(&root);

        for (actual, locator) in [
            (
                &resolved.recursive_sidecars,
                "artifacts/checkpoints/recursive_shadow",
            ),
            (
                &resolved.epoch_close_anchors,
                "artifacts/checkpoints/epoch_close_anchor",
            ),
            (
                &resolved.epoch_evidence_anchors,
                "artifacts/checkpoints/epoch_evidence_anchor",
            ),
            (
                &resolved.challenge_packs,
                "artifacts/checkpoints/challenge_pack",
            ),
            (
                &resolved.retention_tickets,
                "artifacts/checkpoints/retention_ticket",
            ),
            (
                &resolved.retention_ledger,
                "artifacts/checkpoints/retention_ledger",
            ),
            (
                &resolved.history_proofs,
                "artifacts/checkpoints/plonky3_history",
            ),
            (
                &resolved.history_rotation_bridges,
                "artifacts/checkpoints/history_rotation_bridge",
            ),
        ] {
            assert_eq!(actual, &root.join(locator));
            assert!(actual.starts_with(&root));
        }
    }

    #[test]
    fn test_rename_ledger_binds_owners() {
        let destination = CheckpointContractConfigV3::load_repo_default().unwrap();
        let (source, source_bytes) = migration_source_v2();
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let (migrated, ledger) = ConfigV3RenameLedger::migrate(
            &source,
            &source_bytes,
            destination.runtime_profile.manifest_digest.clone(),
            hex_digest(registry.digest()),
            2,
            1,
        )
        .unwrap();
        assert!(migrated.is_v3());
        assert_ne!(
            ledger.source_config_digest,
            ledger.destination_config_digest
        );
        for owner in [
            "version",
            "profile",
            "architecture_mode",
            "statement.version",
            "branches.recursive.no_op.execution_input_version",
            "authority_promotion.stage",
            "verified_backend.chain_evidence.object",
            "gates.inputs.has_statement_fields",
            "da.provider_sdk_boundary",
            "archive_retention.celestia_is_da_only",
            "post_quantum.mode",
            "snapshots.object_type",
            "pruning.prune_scope",
            "retention.raw_tx_packages",
            "paths.checkpoint_artifacts",
            "limits.max_batch_ops",
            "documentation.include_object_schemas",
        ] {
            assert!(ledger.entries.iter().any(|entry| entry.source == owner));
        }
        let removed_noop = ledger
            .entries
            .iter()
            .find(|entry| entry.source == "branches.recursive.no_op.execution_input_version")
            .unwrap();
        assert_eq!(removed_noop.transform, ConfigFieldTransformV3::Removal);
        assert_eq!(
            removed_noop.destination,
            "<removed:branches.recursive.no_op.execution_input_version>"
        );
        assert!(ledger
            .entries
            .iter()
            .any(|entry| entry.transform == ConfigFieldTransformV3::Introduced));
        assert_eq!(
            ledger
                .entries
                .iter()
                .filter(|entry| entry.transform == ConfigFieldTransformV3::ReservedUnreachable)
                .count(),
            30
        );
        let pq_role = ledger
            .entries
            .iter()
            .find(|entry| entry.destination == "branches.nova.security_role")
            .unwrap();
        assert_eq!(pq_role.source_type, "absent");
        assert_eq!(pq_role.destination_type, "string");
        assert_eq!(pq_role.transform, ConfigFieldTransformV3::Introduced);
        let mailbox_cap = ledger
            .entries
            .iter()
            .find(|entry| entry.destination == "offline_receipt_mailbox.max_entry_bytes")
            .unwrap();
        assert_eq!(mailbox_cap.source_type, "absent");
        assert_eq!(mailbox_cap.destination_type, "integer");
        assert!(mailbox_cap.source_value_digest.is_none());
        assert!(mailbox_cap.destination_value_digest.is_some());
        let destination_bytes = YamlCodec.serialize(&migrated).unwrap();
        ledger
            .validate_pair(&source_bytes, &destination_bytes)
            .unwrap();

        let mut forged = ledger.clone();
        forged.entries[0].justification.push_str("-forged");
        let temp = tempfile::tempdir().unwrap();
        let store = ConfigV3ActivationStore::open(temp.path().join("config-store")).unwrap();
        assert!(store
            .install(
                &source,
                &source_bytes,
                &migrated,
                &forged,
                1,
                TEST_RELEASE_MANIFEST_DIGEST_V3,
                embedded_chain_context_digest_v3(),
                None,
            )
            .is_err());
        assert!(store.load_head().is_err());
    }

    #[test]
    fn test_rename_ledger_rejects_drift() {
        let (_, source_bytes) = migration_source_v2();
        let destination = CheckpointContractConfigV3::load_repo_default().unwrap();
        let destination_bytes = YamlCodec.serialize(&destination).unwrap();
        let baseline = ConfigV3RenameLedger::from_pair(&source_bytes, &destination_bytes).unwrap();
        baseline
            .validate_pair(&source_bytes, &destination_bytes)
            .unwrap();

        let changed_source = String::from_utf8(source_bytes.clone())
            .unwrap()
            .replacen("version: 2\n", "version: 1\n", 1)
            .into_bytes();
        let changed = ConfigV3RenameLedger::from_pair(&changed_source, &destination_bytes).unwrap();
        assert_ne!(changed.digest, baseline.digest);
        let changed_version = changed
            .entries
            .iter()
            .find(|entry| entry.source == "version")
            .unwrap();
        let baseline_version = baseline
            .entries
            .iter()
            .find(|entry| entry.source == "version")
            .unwrap();
        assert_ne!(
            changed_version.source_value_digest,
            baseline_version.source_value_digest
        );

        let mut missing = baseline.clone();
        missing.entries.pop();
        assert!(missing
            .validate_pair(&source_bytes, &destination_bytes)
            .is_err());

        let mut extra = baseline.clone();
        let mut extra_entry = baseline.entries[0].clone();
        extra_entry.source = "<new:extra.field>".to_string();
        extra_entry.source_type = "absent".to_string();
        extra_entry.destination = "extra.field".to_string();
        extra_entry.transform = ConfigFieldTransformV3::Introduced;
        extra.entries.push(extra_entry);
        assert!(extra
            .validate_pair(&source_bytes, &destination_bytes)
            .is_err());

        let mut collision = baseline.clone();
        collision.entries[1].destination = collision.entries[0].destination.clone();
        assert!(collision
            .validate_pair(&source_bytes, &destination_bytes)
            .is_err());

        let mut wrong_type = baseline.clone();
        wrong_type.entries[0].destination_type = "u64".to_string();
        assert!(wrong_type
            .validate_pair(&source_bytes, &destination_bytes)
            .is_err());

        let mut downgrade = baseline;
        downgrade.transform_version -= 1;
        assert!(downgrade
            .validate_pair(&source_bytes, &destination_bytes)
            .is_err());
    }

    #[test]
    fn test_activation_store_orders_head() {
        let active = CheckpointContractConfigV3::load_repo_default().unwrap();
        let (source, source_bytes) = migration_source_v2();
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let (destination, ledger) = ConfigV3RenameLedger::migrate(
            &source,
            &source_bytes,
            active.runtime_profile.manifest_digest.clone(),
            hex_digest(registry.digest()),
            2,
            1,
        )
        .unwrap();
        let temp = tempfile::tempdir().unwrap();
        let store = ConfigV3ActivationStore::open(temp.path().join("config-store")).unwrap();
        let head = store
            .install(
                &source,
                &source_bytes,
                &destination,
                &ledger,
                1,
                TEST_RELEASE_MANIFEST_DIGEST_V3,
                embedded_chain_context_digest_v3(),
                None,
            )
            .unwrap();
        assert_eq!(store.load_head().unwrap(), head);
        let resolved = store.load_active().unwrap();
        assert_eq!(resolved.head(), &head);
        assert_eq!(resolved.config(), &destination);
        assert_eq!(
            resolved.migration_record.as_ref().unwrap().rename_ledger,
            ledger
        );
        assert_eq!(head.config_generation, 2);
        assert_eq!(
            store
                .install(
                    &source,
                    &source_bytes,
                    &destination,
                    &ledger,
                    1,
                    TEST_RELEASE_MANIFEST_DIGEST_V3,
                    embedded_chain_context_digest_v3(),
                    None,
                )
                .unwrap(),
            head
        );
    }

    #[test]
    fn test_activation_resumes_exact_generation() {
        let active = CheckpointContractConfigV3::load_repo_default().unwrap();
        let (source, source_bytes) = migration_source_v2();
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let (destination, ledger) = ConfigV3RenameLedger::migrate(
            &source,
            &source_bytes,
            active.runtime_profile.manifest_digest.clone(),
            hex_digest(registry.digest()),
            2,
            1,
        )
        .unwrap();
        let destination_bytes = YamlCodec.serialize(&destination).unwrap();

        let temp = tempfile::tempdir().unwrap();
        let store = ConfigV3ActivationStore::open(temp.path().join("config-store")).unwrap();
        let generation = store.root.join("generations/00000000000000000002");
        create_dir_all(&generation).unwrap();
        atomic_write_file_private(
            generation.join("checkpoint_contract.yaml"),
            &destination_bytes,
        )
        .unwrap();
        let resumed = store
            .install(
                &source,
                &source_bytes,
                &destination,
                &ledger,
                1,
                TEST_RELEASE_MANIFEST_DIGEST_V3,
                embedded_chain_context_digest_v3(),
                None,
            )
            .unwrap();
        assert_eq!(store.load_active().unwrap().head(), &resumed);

        let mixed_temp = tempfile::tempdir().unwrap();
        let mixed = ConfigV3ActivationStore::open(mixed_temp.path().join("config-store")).unwrap();
        let mixed_generation = mixed.root.join("generations/00000000000000000002");
        create_dir_all(&mixed_generation).unwrap();
        let mut wrong = destination_bytes;
        wrong[0] ^= 1;
        atomic_write_file_private(mixed_generation.join("checkpoint_contract.yaml"), &wrong)
            .unwrap();
        assert!(mixed
            .install(
                &source,
                &source_bytes,
                &destination,
                &ledger,
                1,
                TEST_RELEASE_MANIFEST_DIGEST_V3,
                embedded_chain_context_digest_v3(),
                None,
            )
            .is_err());
        assert!(mixed.load_head().is_err());
    }
}
