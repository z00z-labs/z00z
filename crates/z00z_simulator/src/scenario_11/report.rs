#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const CLAIM_LEVEL_LIVE: &str = "live";
pub const CLAIM_LEVEL_LIVE_CLAIM_REMOVED: &str = "live-claim-removed";
pub const PLANNER_AUTHORITY_MODEL_DETERMINISTIC_REPLICATED: &str = "deterministic_replicated";
pub const TERM_DETERMINISTIC_REPLICATED_PLANNER: &str = "deterministic replicated planner";
pub const TERM_PLANNER_HA: &str = "planner HA";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageIngressReport {
    pub package_kind: String,
    pub package_digest_hex: String,
    pub route_key_hex: String,
    pub batch_id_hex: String,
    pub shard_id: u16,
    pub routing_generation: u64,
    pub planner_route_table_digest_hex: String,
    pub ingress_recomputed_digest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePlanCaseReport {
    pub case_id: String,
    pub batch_id_hex: String,
    pub shard_id: u16,
    pub routing_generation: u64,
    pub route_table_digest_hex: String,
    pub plan_digest_hex: String,
    pub dispatch_owner_id: u16,
    pub dispatch_stage: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualPrimaryIsolationReport {
    pub owner_id: u16,
    pub shard_ids: Vec<u16>,
    pub membership_digests_hex: Vec<String>,
    pub certificate_digests_hex: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerAuthorityReplicaReport {
    pub aggregator_id: u16,
    pub recomputed_plan_digest_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePlanReport {
    pub planner_mode: String,
    pub planner_authority_model: String,
    pub planner_config_digest_hex: String,
    pub planner_authority_digest_hex: String,
    pub planner_ha_claim_level: String,
    pub route_table_digest_hex: String,
    pub authority_replicas: Vec<PlannerAuthorityReplicaReport>,
    pub happy_path: RoutePlanCaseReport,
    pub all_shard_sweep: Vec<RoutePlanCaseReport>,
    pub dual_primary_owner: DualPrimaryIsolationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementMembershipCaseReport {
    pub shard_id: u16,
    pub routing_generation: u64,
    pub primary_id: u16,
    pub secondary_ids: Vec<u16>,
    pub ready_secondary_ids: Vec<u16>,
    pub quorum_threshold: usize,
    pub membership_digest_hex: String,
    pub expected_journal_lineage_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementMembershipReport {
    pub happy_path: PlacementMembershipCaseReport,
    pub all_shard_sweep: Vec<PlacementMembershipCaseReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSubjectReport {
    pub subject_digest_hex: String,
    pub term: u64,
    pub batch_id_hex: String,
    pub shard_id: u16,
    pub routing_generation: u64,
    pub plan_digest_hex: String,
    pub route_table_digest_hex: String,
    pub membership_digest_hex: String,
    pub previous_state_root_hex: String,
    pub new_state_root_hex: String,
    pub journal_lineage_hex: String,
    pub proof_version: u16,
    pub theorem_digest_hex: String,
    pub publication_binding_digest_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondaryReplayVoteReport {
    pub case_id: String,
    pub voter_id: u16,
    pub voter_role: String,
    pub verdict: String,
    pub transport_verdict: String,
    pub signature_scheme: Option<String>,
    pub vote_digest_hex: Option<String>,
    pub reject_code: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondaryReplayVotesReport {
    pub happy_path_votes: Vec<SecondaryReplayVoteReport>,
    pub offline_case_votes: Vec<SecondaryReplayVoteReport>,
    pub stale_case_votes: Vec<SecondaryReplayVoteReport>,
    pub drift_case_votes: Vec<SecondaryReplayVoteReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumCertificateCaseReport {
    pub case_id: String,
    pub shard_id: u16,
    pub routing_generation: u64,
    pub quorum_threshold: usize,
    pub membership_digest_hex: String,
    pub subject_digest_hex: String,
    pub certificate_digest_hex: String,
    pub voter_ids: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumCertificateReport {
    pub happy_path: QuorumCertificateCaseReport,
    pub dual_primary_cases: Vec<QuorumCertificateCaseReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalDaBindingReport {
    pub batch_id_hex: String,
    pub checkpoint_id_hex: String,
    pub publication_checkpoint: u64,
    pub publication_route_digest_hex: String,
    pub publication_shard_ids: Vec<u32>,
    pub publication_binding_digest_hex: String,
    pub blob_ref: String,
    pub provider: String,
    pub certificate_digest_hex: String,
    pub resumed_by_secondary_id: u16,
    pub resumed_same_certificate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusStoreReport {
    pub backend: String,
    pub schema_version: u32,
    pub route_key_hex: String,
    pub batch_id_hex: String,
    pub subject_digest_hex: String,
    pub certificate_digest_hex: String,
    pub vote_digests_hex: Vec<String>,
    pub publication_binding_digest_hex: String,
    pub validator_verdict_kind: String,
    pub checkpoint_id_hex: String,
    pub resumed_by_secondary_id: u16,
    pub resume_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorVerdictReport {
    pub verdict_kind: String,
    pub reject_class: Option<String>,
    pub checkpoint_id_hex: Option<String>,
    pub publication_binding_digest_hex: Option<String>,
    pub theorem_digest_hex: String,
    pub batch_id_hex: String,
    pub subject_digest_hex: String,
    pub certificate_digest_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultMatrixEntry {
    pub scenario_id: String,
    pub fault_id: String,
    pub expected_status: String,
    pub observed_status: String,
    pub reject_code: Option<String>,
    pub evidence_refs: Vec<String>,
    pub detail: String,
    pub degraded_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultMatrixReport {
    pub entries: Vec<FaultMatrixEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimLevelReport {
    pub term: String,
    pub claim_level: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportHonesty {
    pub supported_claims: Vec<String>,
    pub forbidden_claims: Vec<String>,
    pub deferred_claims: Vec<String>,
    pub simulated_markers: Vec<String>,
    pub claim_levels: Vec<ClaimLevelReport>,
}
