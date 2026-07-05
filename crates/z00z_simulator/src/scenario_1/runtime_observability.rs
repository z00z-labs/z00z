use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use crate::{DesignDoc, ScenarioCfg, ScenarioResult, StageResult};
use rand::{rngs::StdRng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use z00z_aggregators::{bind_publication_contract, BatchId, PublicationBinding, ShardRouteTable};
use z00z_core::assets::AssetClass;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_rollup_node::{ConfigDigestRecord, NodeConfig};
use z00z_storage::checkpoint::{CheckpointFsStore, CheckpointId, CheckpointStore};
use z00z_storage::settlement::{
    check_public_checkpoint_route_v1, check_public_checkpoint_v1, check_publication_route_v1,
    CheckpointPublicationProofV1, CheckpointPublicationV1, DefinitionId, HjmtProofFamily,
    PublicationModeTagV1, PublicationRouteSnapshotV1, RootGenerationTagV1, SerialId,
    SettlementLeafFamily, SettlementPath, SettlementStateRoot, SettlementStore,
    ShardProofContextV1, ShardRootLeafV1, TerminalId,
};
use z00z_utils::{
    codec::{Codec, JsonCodec, Value},
    io,
    rng::SecureRngProvider,
    time::MockTimeProvider,
};
use z00z_wallets::{
    backup::{decode_tx_history_rows, WalletTxHistoryEntryKind, WalletTxHistoryJsonlEntry},
    chain::ReceiverCardRecord,
    domains::compute_wallet_file_id,
    persistence::{TxRecord, TxStatus as WalletTxStatus, TxStorage, TxStorageImpl},
    rpc::{
        methods::{AssetRpcImpl, AssetRpcServer, TxRpcImpl, TxRpcServer},
        types::{
            common::{PersistTxId, PersistWalletId},
            tx::{PortableWalletTxPackage, RuntimeTxErrorCode, RuntimeTxLifecycle},
            wallet::{SessionToken, WalletSource},
        },
    },
    services::{AppService, WalletService},
    stealth::{build_seeded_output_bundle, SenderWallet},
    tx::TxPackage,
};

use crate::scenario_1::{
    stage_11::jmt_wallet_scan::{load_post_tx_candidate_set, JmtScanArtifact},
    stage_11::Stage11Checkpoint,
};

use super::runner::Scenario1Err;

const RUNTIME_TRACE_VERSION: &str = "phase056_runtime_trace_v1";
const PUBLICATION_TRACE_VERSION: &str = "phase057_publication_trace_v1";
const RELEASE_PACKET_VERSION: &str = "phase058_release_packet_v1";
const RUNTIME_CONTRACT: &str = "runtime_contract";
const LINKED_OWNER_CONTRACT: &str = "linked_owner_contract";
const IMPORTED_ARTIFACT_CONTRACT: &str = "imported_artifact_contract";
const PUBLICATION_CONTRACT: &str = "publication_contract";
const SCOPE_OWNER_HOME: &str = "crates/z00z_storage/tests/test_hjmt_scope_birth.rs";
const HISTORICAL_OWNER_HOME: &str = "crates/z00z_storage/tests/test_hjmt_historical_proofs.rs";
const ADAPTIVE_POLICY_OWNER_HOME: &str =
    "crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs";
const OCCUPANCY_PRIVACY_OWNER_HOME: &str = "crates/z00z_storage/tests/test_occupancy_privacy.rs";
const OCCUPANCY_EVIDENCE_OWNER_HOME: &str = "crates/z00z_storage/tests/test_occupancy_evidence.rs";
const WALLET_SCAN_OWNER_HOME: &str =
    "crates/z00z_simulator/tests/scenario_1/test_stage7_jmt_wallet_scan.rs";
const RECOVERY_OWNER_HOME_RUNTIME: &str =
    "crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs";
const RECOVERY_OWNER_HOME_STORAGE: &str =
    "crates/z00z_storage/src/settlement/test_live_recovery.rs";
const SCENARIO_OUTPUT_MARKER: &str = "crates/z00z_simulator/outputs/scenario_1";
const REQUIRED_PROFILES: &[&str] = &["SIM-SMALL", "SIM-MEDIUM", "SIM-CACHE-EDGE"];
const HEAVY_ONLY_PROFILE: &str = "SIM-BATCH-1000";
const REQUIRED_EMITTED_PUBLIC_FILES: &[&str] = &[
    "hist_flow.json",
    "occ_flow.json",
    "asset_flow.json",
    "voucher_flow.json",
    "right_flow.json",
];
const TRANSITION_OWNER_HOME: &str = "crates/z00z_storage/tests/test_hjmt_transition_proofs.rs";
const PRIVACY_OWNER_HOME: &str = "crates/z00z_storage/tests/test_hjmt_privacy_regression.rs";
const E2E_OWNER_HOME: &str = "crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs";
const WALLET_SIM_PASSWORD: &str = "StrongPassw0rd!";
const WALLET_SIM_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const WALLET_SIM_ALT_SEED_24: &str = "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title";
const WALLET_SIM_BASE_TIME_SECS: u64 = 1_703_260_800;
const REQUIRED_WALLET_LIFECYCLE_CASES: &[&str] = &[
    "imported",
    "submitted",
    "admitted",
    "confirmed",
    "duplicate_import",
    "conflicted",
    "already_spent",
    "no_owned_output",
    "wrong_chain",
    "invalid_digest",
    "unsupported_package_version",
];

pub(super) struct RuntimeTraceSpec {
    scenario_id: u32,
    scenario_name: String,
    active_profile: String,
    deterministic: bool,
    hjmt_home: PathBuf,
    route_table_path: PathBuf,
    route_table_digest: String,
    routing_generation: u64,
    config_digests: Vec<ConfigDigestRecord>,
    config_digest_set_hex: String,
    design_digest_hex: String,
    process_view: ProcessTopologyView,
    process_topology_digest_hex: String,
    journal_view: JournalContractView,
    journal_lineage_digest_hex: String,
    traces: crate::config::RuntimeTraceCfg,
}

#[derive(Clone, Serialize)]
struct FlowCommon {
    trace_version: &'static str,
    trace_kind: &'static str,
    trace_mode: &'static str,
    scenario_id: u32,
    scenario_name: String,
    active_profile: String,
    deterministic: bool,
    semantic_digest_hex: String,
    config_digest_set_hex: String,
    design_digest_hex: String,
    route_table_digest: String,
    process_topology_digest_hex: String,
    journal_lineage_digest_hex: String,
    scenario_config_path: PathBuf,
    design_path: PathBuf,
    hjmt_home: PathBuf,
    config_digests: Vec<ConfigDigestRecord>,
}

#[derive(Clone, Serialize)]
struct RuntimeProfileView {
    id: String,
    deterministic: bool,
    purpose: String,
}

#[derive(Clone, Serialize)]
struct TraceFileView {
    cfg_flow_file: String,
    tx_flow_file: String,
    route_flow_file: String,
    plan_flow_file: String,
    journal_flow_file: String,
    scope_flow_file: String,
    proc_flow_file: String,
    recovery_flow_file: String,
    leaf_flow_file: String,
    proof_flow_file: String,
    pub_flow_file: String,
    val_flow_file: String,
    watch_flow_file: String,
}

#[derive(Clone, Serialize)]
struct PlacementRowView {
    shard_id: u16,
    primary_aggregator_id: u16,
    secondary_ids: Vec<u16>,
    expected_journal_lineage_hex: String,
}

#[derive(Clone, Serialize)]
struct PlannerPolicyView {
    shard_local_only: bool,
    reject_cross_shard: bool,
    cadence_ms: u64,
}

#[derive(Clone, Serialize)]
struct PlannerLimitsView {
    max_batch_ops: usize,
    max_batch_bytes: usize,
}

#[derive(Clone, Serialize)]
struct StoragePathsView {
    data_dir: PathBuf,
    journal_dir: PathBuf,
    export_dir: PathBuf,
    import_dir: PathBuf,
    lock_path: PathBuf,
}

#[derive(Clone, Serialize)]
struct ProcessTopologyView {
    profile: String,
    process_model: &'static str,
    shard_mapping: &'static str,
    agg_count: usize,
    shard_count: usize,
    routing_generation: u64,
    aggregators: Vec<AggregatorProcessView>,
}

#[derive(Clone, Serialize)]
struct AggregatorProcessView {
    aggregator_id: u16,
    process_id: String,
    role: String,
    listen_addr: String,
    data_dir: PathBuf,
    journal_path: PathBuf,
    log_path: PathBuf,
    start_cmd: String,
    restart_cmd: String,
    shard_ids: Vec<u16>,
}

#[derive(Clone, Serialize)]
struct JournalContractView {
    backend: String,
    generation: u64,
    cache_capacity: usize,
    lock_timeout_ms: u64,
    storage_paths: StoragePathsView,
    lineage_rows: Vec<PlacementRowView>,
}

#[derive(Clone, Serialize)]
struct StageResultView {
    stage: u32,
    name: String,
    result: &'static str,
}

#[derive(Serialize)]
struct CfgFlow {
    #[serde(flatten)]
    common: FlowCommon,
    route_table_path: PathBuf,
    routing_generation: u64,
    supported_profiles: Vec<RuntimeProfileView>,
    heavy_only_profiles: Vec<String>,
    trace_files: TraceFileView,
    design_stage_ids: Vec<u32>,
    design_stage_names: Vec<String>,
}

#[derive(Serialize)]
struct TxFlow {
    #[serde(flatten)]
    common: FlowCommon,
    tx_package_path: PathBuf,
    transfer_leaf_path: PathBuf,
    bundle_frag1_path: PathBuf,
    bundle_frag2_path: PathBuf,
    bundle_bridge_path: PathBuf,
    checkpoint_apply_path: PathBuf,
    checkpoint_finalize_path: PathBuf,
    hjmt_examples_report_path: PathBuf,
    stage_results: Vec<StageResultView>,
}

#[derive(Serialize)]
struct RouteFlow {
    #[serde(flatten)]
    common: FlowCommon,
    route_table_path: PathBuf,
    routing_generation: u64,
    placement_rows: Vec<PlacementRowView>,
}

#[derive(Serialize)]
struct PlanFlow {
    #[serde(flatten)]
    common: FlowCommon,
    planner_mode: String,
    planner_config_path: PathBuf,
    planner_policy: PlannerPolicyView,
    planner_limits: PlannerLimitsView,
    planner_plan_dir: PathBuf,
    planner_evidence_dir: PathBuf,
}

#[derive(Serialize)]
struct JournalFlow {
    #[serde(flatten)]
    common: FlowCommon,
    journal_contract: JournalContractView,
    cache_edge_samples: Vec<usize>,
}

#[derive(Serialize)]
struct ScopeFlow {
    #[serde(flatten)]
    common: FlowCommon,
    semantic_owner: &'static str,
    trace_owner_home: &'static str,
    trace_owner_mode: &'static str,
    linked_stage_ids: Vec<u32>,
    private_tree_id_exposed: bool,
    owner_contract_rows: Vec<ScopeOwnerContractView>,
    wallet_promotion_rows: Vec<ScopeWalletPromotionView>,
    wallet_negative_rows: Vec<WalletNegativeSummaryView>,
    proof_boundary: &'static str,
    restart_failover_owner_homes: [&'static str; 2],
    acceptance_homes: Vec<AcceptanceHomeView>,
}

#[derive(Clone, Serialize)]
struct ScopeOwnerContractView {
    contract_id: &'static str,
    owner_home: &'static str,
    proof_point: &'static str,
    leaf_family: &'static str,
    first_seen_definition: bool,
    first_seen_serial: bool,
    first_seen_object: bool,
    status: &'static str,
}

#[derive(Clone, Serialize)]
struct ScopeWalletPromotionView {
    asset_id_hex: String,
    definition_id: String,
    serial_id: u32,
    amount: u64,
    output_role: String,
    scan_receive_status: String,
    scan_receive_next: String,
    pending_lifecycle_status: String,
    confirmed_lifecycle_status: String,
    diff_status: String,
    proof_validated: bool,
    owner_detected: bool,
    tx_digest_hex: String,
}

#[derive(Clone, Serialize)]
struct WalletNegativeSummaryView {
    receive_status: String,
    receive_next: String,
    count: usize,
}

#[derive(Clone, Serialize)]
struct AcceptanceHomeView {
    home: &'static str,
    purpose: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
struct ProcFlow {
    #[serde(flatten)]
    common: FlowCommon,
    process_topology: ProcessTopologyView,
}

#[derive(Serialize)]
struct RecoveryFlow {
    #[serde(flatten)]
    common: FlowCommon,
    recovery_owner_homes: [&'static str; 2],
    failover_rows: Vec<PlacementRowView>,
    startup_checks_required: [&'static str; 7],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicationTopologyStageView {
    stage_id: String,
    topology: String,
    aggregator_count: usize,
    shard_count: usize,
    route_generation: u64,
    owner_aggregator_id: u16,
    secondary_aggregator_ids: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicationTopologyView {
    fixture_id: String,
    shard_id: u16,
    old_topology: String,
    new_topology: String,
    old_aggregator_count: usize,
    old_shard_count: usize,
    new_aggregator_count: usize,
    new_shard_count: usize,
    route_generation_from: u64,
    route_generation_to: u64,
    owner_aggregator_id: u16,
    secondary_aggregator_ids: Vec<u16>,
    join_mode: String,
    transfer_target: String,
    activation_checkpoint: u64,
    transition_stages: Vec<PublicationTopologyStageView>,
    removed_aggregator_ids: Vec<u16>,
    removed_aggregator_absent_from_owner_tables: bool,
    removed_aggregator_absent_from_secondary_tables: bool,
    all_shards_owned_across_stages: bool,
    prior_lineage_preserved: bool,
    publication_continuity_preserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicationLeafView {
    shard_id: u16,
    source_settlement_path: String,
    primary_aggregator_id: u16,
    secondary_ids: Vec<u16>,
    state_root_hex: String,
    leaf_canonical_bytes_hex: String,
    leaf_digest_hex: String,
    journal_checkpoint: u64,
    local_sequence: u64,
    policy_set_digest_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicationProofView {
    shard_id: u16,
    source_settlement_path: String,
    proof_digest_hex: String,
    proof_size_bytes: usize,
    public_root_hex: String,
    proof_family: String,
    leaf_family: String,
    journal_checkpoint: u64,
    policy_generation: u64,
    bucket_policy_digest_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicationProcessVerdictView {
    aggregator_id: u16,
    process_id: String,
    journal_path: PathBuf,
    owned_shard_ids: Vec<u16>,
    secondary_shard_ids: Vec<u16>,
    exit_verdict: String,
    restart_verdict: String,
}

#[derive(Serialize)]
struct LeafFlow {
    #[serde(flatten)]
    common: FlowCommon,
    publication_profile: String,
    inherited_runtime_profile: String,
    topology_status: String,
    public_leaf_count: usize,
    publication_checkpoint: u64,
    prior_public_root_hex: String,
    publication_digest_hex: String,
    leaf_rows: Vec<PublicationLeafView>,
    topology_examples: Vec<PublicationTopologyView>,
    linked_trace_files: TraceFileView,
}

#[derive(Serialize)]
struct ProofFlow {
    #[serde(flatten)]
    common: FlowCommon,
    publication_profile: String,
    publication_digest_hex: String,
    public_root_hex: String,
    proof_rows: Vec<PublicationProofView>,
    linked_trace_files: TraceFileView,
}

#[derive(Serialize)]
struct PubFlow {
    #[serde(flatten)]
    common: FlowCommon,
    publication_profile: String,
    inherited_runtime_profile: String,
    topology_status: String,
    public_leaf_count: usize,
    publication_checkpoint: u64,
    activation_checkpoint: u64,
    route_generation: u64,
    prior_public_root_hex: String,
    public_root_hex: String,
    publication_digest_hex: String,
    canonical_publication_hex: String,
    topology_examples: Vec<PublicationTopologyView>,
    process_verdicts: Vec<PublicationProcessVerdictView>,
    linked_trace_files: TraceFileView,
}

#[derive(Serialize)]
struct ValFlow {
    #[serde(flatten)]
    common: FlowCommon,
    publication_profile: String,
    publication_digest_hex: String,
    binding_digest_hex: String,
    draft_id_hex: String,
    checkpoint_id_hex: String,
    route_generation: u64,
    verdict_kind: String,
    prev_settlement_root_hex: String,
    new_settlement_root_hex: String,
    spent_delta_count: usize,
    created_delta_count: usize,
    topology_examples: Vec<PublicationTopologyView>,
    linked_trace_files: TraceFileView,
}

#[derive(Serialize)]
struct WatchFlow {
    #[serde(flatten)]
    common: FlowCommon,
    publication_profile: String,
    publication_digest_hex: String,
    binding_digest_hex: String,
    draft_id_hex: String,
    checkpoint_id_hex: String,
    verdict_kind: String,
    publication_state: String,
    topology_examples: Vec<PublicationTopologyView>,
    process_verdicts: Vec<PublicationProcessVerdictView>,
    linked_trace_files: TraceFileView,
}

#[derive(Clone, Serialize)]
struct RouteMigrationView {
    fixture_id: String,
    old_topology: String,
    new_topology: String,
    old_route_generation: u64,
    new_route_generation: u64,
    old_public_root_hex: String,
    new_public_root_hex: String,
    old_settlement_root_hex: String,
    new_settlement_root_hex: String,
    activation_checkpoint: u64,
}

#[derive(Clone, Serialize)]
struct HistoricalProofVerdictView {
    example_id: String,
    proof_family: String,
    leaf_family: String,
    verifier_status: String,
    settlement_path: Option<String>,
    settlement_state_root_hex: String,
    reloaded_settlement_state_root_hex: String,
    proof_is_ownership: bool,
}

#[derive(Clone, Serialize)]
struct OccupancyDisclosureVerdictView {
    example_id: String,
    proof_family: String,
    verifier_status: String,
    settlement_path: Option<String>,
    prior_state_root_hex: Option<String>,
    next_state_root_hex: Option<String>,
    disclosure_guard: &'static str,
    binding_hex: Option<String>,
    proof_is_ownership: bool,
}

#[derive(Clone, Serialize)]
struct ImportedArtifactVerdictView {
    verdict_id: &'static str,
    status: &'static str,
    detail: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct WalletLifecycleEvidenceView {
    case_id: String,
    tx_id: String,
    lifecycle: RuntimeTxLifecycle,
    coarse_status: String,
    error_code: Option<RuntimeTxErrorCode>,
    wallet_asset_rows_changed: bool,
    tx_history_row_count_changed: bool,
    restart_verification_passed: bool,
    wallet_scan_digest_hex: String,
    tx_history_digest_hex: String,
    publication_digest_hex: String,
}

#[derive(Clone, Serialize)]
struct LiveRejectVerdictView {
    example_id: String,
    case_id: String,
    proof_surface: String,
    verifier_status: String,
    typed_error_class: String,
}

#[derive(Clone, Serialize)]
struct OwnerRejectHomeView {
    contract_id: &'static str,
    owner_home: &'static str,
    proof_point: &'static str,
    status: &'static str,
}

#[derive(Clone, Serialize)]
struct PrivacyOwnerContractView {
    contract_id: &'static str,
    owner_home: &'static str,
    proof_point: &'static str,
    disclosure_guard: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
struct HistFlow {
    #[serde(flatten)]
    common: FlowCommon,
    route_migration_rows: Vec<RouteMigrationView>,
    source_artifacts: Vec<String>,
    wallet_scan_digest_hex: String,
    wallet_lifecycle_rows: Vec<WalletLifecycleEvidenceView>,
    historical_proof_verdicts: Vec<HistoricalProofVerdictView>,
    live_reject_rows: Vec<LiveRejectVerdictView>,
    owner_reject_rows: Vec<OwnerRejectHomeView>,
    imported_artifact_verdicts: Vec<ImportedArtifactVerdictView>,
}

#[derive(Serialize)]
struct OccFlow {
    #[serde(flatten)]
    common: FlowCommon,
    route_migration_rows: Vec<RouteMigrationView>,
    source_artifacts: Vec<String>,
    occupancy_disclosure_verdicts: Vec<OccupancyDisclosureVerdictView>,
    live_reject_rows: Vec<LiveRejectVerdictView>,
    privacy_owner_contract_rows: Vec<PrivacyOwnerContractView>,
    imported_artifact_verdicts: Vec<ImportedArtifactVerdictView>,
}

#[derive(Clone, Serialize)]
struct ObjectFlowCaseView {
    id: String,
    family: String,
    action: String,
    policy_label: String,
    lane: String,
    actors: Vec<String>,
    required_rights: Vec<String>,
    expected_verdict: String,
    evidence_files: Vec<String>,
}

#[derive(Serialize)]
struct ObjectFlow {
    #[serde(flatten)]
    common: FlowCommon,
    packet_anchor_file: String,
    positive_count: usize,
    negative_count: usize,
    covered_families: Vec<String>,
    covered_lanes: Vec<String>,
    source_artifacts: Vec<String>,
    positive_rows: Vec<ObjectFlowCaseView>,
    negative_rows: Vec<ObjectFlowCaseView>,
}

#[derive(Clone, Serialize)]
struct ArtifactInventoryRow {
    file: String,
    status: &'static str,
}

#[derive(Clone, Serialize)]
struct WalletScanSummaryView {
    actor: String,
    store_root_hex: String,
    detected_count: usize,
    total_detected_amount: u64,
    proof_validated_count: usize,
    skipped_non_asset_count: usize,
}

#[derive(Clone, Serialize)]
struct PublicLaneGuardView {
    wallet_debug_tools_required: bool,
    private_lane_dependency: bool,
    secret_artifacts_excluded: bool,
    redaction_status: &'static str,
}

#[derive(Serialize)]
struct RunMeta {
    schema_version: &'static str,
    scenario_id: u32,
    scenario_name: String,
    binary_name: &'static str,
    execution_mode: &'static str,
    public_lane_status: &'static str,
    stage_sync_status: &'static str,
    active_profile: String,
    deterministic: bool,
    stage_count: usize,
    config_digest_set_hex: String,
    design_digest_hex: String,
    route_table_digest: String,
    process_topology_digest_hex: String,
    journal_lineage_digest_hex: String,
    process_map_file: String,
    recovery_file: String,
    wallet_scan_file: String,
    summary_file: String,
    trace_files: TraceFileView,
    artifact_inventory: Vec<ArtifactInventoryRow>,
    stage_results: Vec<StageResultView>,
    wallet_scan: WalletScanSummaryView,
    public_lane_guards: PublicLaneGuardView,
}

#[derive(Debug, Clone)]
struct PublicationEvidence {
    publication_checkpoint: u64,
    prior_public_root_hex: String,
    public_root_hex: String,
    publication_digest_hex: String,
    canonical_publication_hex: String,
    draft_id_hex: String,
    checkpoint_id_hex: String,
    verdict_kind: String,
    publication_state: String,
    binding: PublicationBinding,
    leaf_rows: Vec<PublicationLeafView>,
    proof_rows: Vec<PublicationProofView>,
    process_verdicts: Vec<PublicationProcessVerdictView>,
}

#[derive(Debug, Deserialize)]
struct Stage13ExamplesRoot {
    settlement_state_root_hex: String,
    #[serde(default)]
    examples: Vec<Stage13ExampleRow>,
    #[serde(default)]
    comparison_rows: Vec<Stage13ComparisonRow>,
}

#[derive(Debug, Deserialize)]
struct Stage13ExampleRow {
    example_id: String,
    proof_family: String,
    leaf_family: String,
    verifier_status: String,
    settlement_state_root_hex: String,
    #[serde(default)]
    settlement_path: Option<String>,
    #[serde(default)]
    prior_state_root_hex: Option<String>,
    #[serde(default)]
    next_state_root_hex: Option<String>,
    #[serde(default)]
    bucket_policy_id: Option<String>,
    #[serde(default)]
    transition_binding: Option<String>,
    #[serde(default)]
    proof_is_ownership: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Stage13ComparisonRow {
    settlement_state_root_hex: String,
    #[serde(default)]
    settlement_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Stage12SummaryView {
    draft_id_hex: String,
    evidence_class: String,
    checkpoint_id_hex: Option<String>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct Stage13ReplayRoots {
    #[serde(default)]
    replay_entries: Vec<Stage13ReplayEntry>,
}

#[derive(Debug, Deserialize)]
struct Stage13ReplayEntry {
    example_id: String,
    verifier_status: String,
    settlement_state_root_hex: String,
    reloaded_settlement_state_root_hex: String,
}

#[derive(Debug, Deserialize)]
struct Stage13TamperReport {
    #[serde(default)]
    cases: Vec<Stage13TamperCase>,
}

#[derive(Debug, Deserialize)]
struct Stage13TamperCase {
    example_id: String,
    case_id: String,
    proof_surface: String,
    verifier_status: String,
    typed_error: Stage13TypedError,
}

#[derive(Debug, Deserialize)]
struct Stage13TypedError {
    class: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct WalletLifecycleRow {
    actor: String,
    wallet_id: String,
    asset_id_hex: String,
    serial_id: u32,
    class: String,
    amount: u64,
    lifecycle_status: String,
    output_role: Option<String>,
    tx_digest_hex: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WalletDiffRoot {
    rows: Vec<WalletDiffRow>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct WalletDiffRow {
    actor: String,
    wallet_id: String,
    asset_id_hex: String,
    serial_id: u32,
    class: String,
    output_role: Option<String>,
    status: String,
    lifecycle_status: String,
    tx_digest_hex: Option<String>,
}

#[derive(Clone, Deserialize)]
struct RuntimeTxErrorPayloadView {
    error_codes: Vec<RuntimeTxErrorCode>,
    #[serde(default)]
    lifecycle: Option<RuntimeTxLifecycle>,
}

#[derive(Clone)]
struct WalletLifecycleSnapshot {
    asset_ids: BTreeSet<String>,
    history_len: usize,
    history_digest_hex: String,
}

struct WalletRestartCheck {
    passed: bool,
}

struct WalletSimEnv {
    output_dir: PathBuf,
    time_provider: Arc<MockTimeProvider>,
    app_service: Arc<AppService>,
    wallet_service: Arc<WalletService>,
    asset_rpc: Arc<AssetRpcImpl>,
    tx_rpc: Arc<TxRpcImpl>,
    wallet_id: PersistWalletId,
    session: SessionToken,
}

struct WalletConfigEnvRestore {
    prev_path: Option<OsString>,
    prev_network: Option<OsString>,
    prev_chain: Option<OsString>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

struct DeterministicSecureRngProvider {
    base_seed: [u8; 32],
    counter: AtomicU64,
}

impl DeterministicSecureRngProvider {
    fn new(base_seed: [u8; 32]) -> Self {
        Self {
            base_seed,
            counter: AtomicU64::new(0),
        }
    }

    fn next_seed(&self) -> [u8; 32] {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut hasher = Sha256::new();
        hasher.update(self.base_seed);
        hasher.update(counter.to_le_bytes());
        hasher.finalize().into()
    }
}

impl SecureRngProvider for DeterministicSecureRngProvider {
    type Rng = StdRng;

    fn rng(&self) -> Self::Rng {
        StdRng::from_seed(self.next_seed())
    }
}

impl WalletConfigEnvRestore {
    fn acquire() -> Self {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let prev_path = std::env::var_os("Z00Z_WALLET_CONFIG_PATH");
        let prev_network = std::env::var_os("Z00Z_WALLET_NETWORK");
        let prev_chain = std::env::var_os("Z00Z_WALLET_CHAIN");
        std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
        Self {
            prev_path,
            prev_network,
            prev_chain,
            _lock: lock,
        }
    }
}

impl Drop for WalletConfigEnvRestore {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }
        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

macro_rules! decode_runtime_tx_error_payload {
    ($err:expr) => {{
        let data = $err.data().ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation missing typed tx error payload: {}",
                $err
            ))
        })?;
        JsonCodec
            .deserialize::<RuntimeTxErrorPayloadView>(data.get().as_bytes())
            .map_err(|decode_err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation failed to decode typed tx error payload: {decode_err}"
            ))
        })?
    }};
}

#[derive(Debug, Deserialize)]
struct LeafFlowSource {
    publication_digest_hex: String,
    leaf_rows: Vec<PublicationLeafView>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ProofFlowSource {
    publication_digest_hex: String,
    public_root_hex: String,
    proof_rows: Vec<PublicationProofView>,
}

#[derive(Debug, Deserialize)]
struct PubFlowSource {
    publication_digest_hex: String,
    route_generation: u64,
    prior_public_root_hex: String,
    public_root_hex: String,
    topology_examples: Vec<PublicationTopologyView>,
}

#[derive(Debug, Deserialize)]
struct ValFlowSource {
    publication_digest_hex: String,
    route_generation: u64,
    prev_settlement_root_hex: String,
    new_settlement_root_hex: String,
}

#[derive(Debug, Deserialize)]
struct WatchFlowSource {
    publication_digest_hex: String,
}

#[derive(Debug)]
struct ImportedFlowSources {
    leaf: LeafFlowSource,
    proof: ProofFlowSource,
    pub_flow: PubFlowSource,
    val: ValFlowSource,
    watch: WatchFlowSource,
    examples: Stage13ExamplesRoot,
    replay: Stage13ReplayRoots,
    tamper: Stage13TamperReport,
}

#[derive(Clone, Copy)]
enum PacketMode {
    Strict,
    Cached,
}

#[derive(Debug, Clone)]
struct TxOutputBindingMeta {
    definition_id: String,
    serial_id: u32,
}

#[derive(Debug, Clone)]
struct PublicationBindingEvidence {
    draft_id_hex: String,
    checkpoint_id_hex: String,
    verdict_kind: String,
    publication_state: String,
    binding: PublicationBinding,
}

fn observability_cfg(
    cfg: &ScenarioCfg,
) -> Result<&crate::config::RuntimeObservabilityCfg, Scenario1Err> {
    cfg.runtime_observability_ref()
        .ok_or_else(|| Scenario1Err::Evidence("runtime_observability config missing".to_string()))
}

fn publication_cfg(
    cfg: &ScenarioCfg,
) -> Result<&crate::config::PublicationObservabilityCfg, Scenario1Err> {
    cfg.publication_observability_ref().ok_or_else(|| {
        Scenario1Err::Evidence("runtime_observability publication missing".to_string())
    })
}

fn hjmt_cfg(home: &Path) -> Result<z00z_rollup_node::HjmtCfg, Scenario1Err> {
    load_node_cfg(home)?
        .hjmt
        .ok_or_else(|| Scenario1Err::Evidence("runtime cfg missing hjmt section".to_string()))
}

fn stage4_cfg(cfg: &ScenarioCfg) -> Result<&crate::config::Stage4TxPrepareCfg, Scenario1Err> {
    cfg.stage4_tx_prepare
        .as_ref()
        .ok_or_else(|| Scenario1Err::Evidence("stage4_tx_prepare config missing".to_string()))
}

pub(super) fn prepare(
    cfg_path: &Path,
    design_path: &Path,
    cfg: &ScenarioCfg,
) -> Result<RuntimeTraceSpec, Scenario1Err> {
    let observability = cfg.runtime_observability_ref().ok_or_else(|| {
        Scenario1Err::Evidence("runtime_observability config missing".to_string())
    })?;
    validate_observability_cfg(observability)?;
    let active_profile = observability
        .supported_profiles
        .iter()
        .find(|profile| profile.id == observability.active_profile)
        .ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "runtime_observability active_profile {} is not supported",
                observability.active_profile
            ))
        })?;

    let hjmt_home =
        repo_root().join(cfg.hjmt_config_root().ok_or_else(|| {
            Scenario1Err::Evidence("hjmt_runtime.config_root missing".to_string())
        })?);
    let node_cfg = NodeConfig::from_hjmt_home(&hjmt_home).map_err(|err| {
        Scenario1Err::Evidence(format!("failed to load hjmt runtime home: {err}"))
    })?;
    let hjmt = node_cfg
        .hjmt
        .as_ref()
        .ok_or_else(|| Scenario1Err::Evidence("hjmt runtime config did not load".to_string()))?;

    let route_table_rel =
        hjmt.planner.route.table_path.as_ref().ok_or_else(|| {
            Scenario1Err::Evidence("planner route table_path missing".to_string())
        })?;
    let route_table_path = if route_table_rel.is_absolute() {
        route_table_rel.clone()
    } else {
        hjmt.home.join(route_table_rel)
    };
    let route_table_digest = hjmt.planner.route.expected_digest.clone().ok_or_else(|| {
        Scenario1Err::Evidence("planner route expected_digest missing".to_string())
    })?;

    let mut config_digests = node_cfg.config_digests().map_err(|err| {
        Scenario1Err::Evidence(format!("failed to hash runtime config files: {err}"))
    })?;
    config_digests.push(file_digest_record("scenario-config", cfg_path)?);
    config_digests.sort_by(|left, right| left.label.cmp(&right.label));

    let process_view = build_process_view(hjmt)?;
    let journal_view = build_journal_view(hjmt);

    Ok(RuntimeTraceSpec {
        scenario_id: cfg.scenario.id,
        scenario_name: cfg.scenario.name.clone(),
        active_profile: active_profile.id.clone(),
        deterministic: active_profile.deterministic,
        hjmt_home,
        route_table_path,
        route_table_digest,
        routing_generation: hjmt.routing_generation(),
        config_digest_set_hex: config_digest_set_hex(&config_digests)?,
        design_digest_hex: ScenarioCfg::config_digest(design_path).map_err(|err| {
            Scenario1Err::Evidence(format!("failed to hash scenario_design.yaml: {err}"))
        })?,
        process_topology_digest_hex: digest_hex(&process_view)?,
        journal_lineage_digest_hex: digest_hex(&journal_view)?,
        config_digests,
        process_view,
        journal_view,
        traces: observability.traces.clone(),
    })
}

pub(super) fn emit(
    spec: &RuntimeTraceSpec,
    cfg_path: &Path,
    design_path: &Path,
    out_dir: &Path,
    cfg: &ScenarioCfg,
    design: &DesignDoc,
    run: &ScenarioResult,
) -> Result<(), Scenario1Err> {
    let observability = observability_cfg(cfg)?;
    let common = flow_common(spec, cfg_path, design_path);
    let supported_profiles = observability
        .supported_profiles
        .iter()
        .map(|profile| RuntimeProfileView {
            id: profile.id.clone(),
            deterministic: profile.deterministic,
            purpose: profile.purpose.clone(),
        })
        .collect::<Vec<_>>();
    let heavy_only_profiles = observability.heavy_only_profiles.clone();
    let trace_files = TraceFileView {
        cfg_flow_file: spec.traces.cfg_flow_file.clone(),
        tx_flow_file: spec.traces.tx_flow_file.clone(),
        route_flow_file: spec.traces.route_flow_file.clone(),
        plan_flow_file: spec.traces.plan_flow_file.clone(),
        journal_flow_file: spec.traces.journal_flow_file.clone(),
        scope_flow_file: spec.traces.scope_flow_file.clone(),
        proc_flow_file: spec.traces.proc_flow_file.clone(),
        recovery_flow_file: spec.traces.recovery_flow_file.clone(),
        leaf_flow_file: spec.traces.leaf_flow_file.clone(),
        proof_flow_file: spec.traces.proof_flow_file.clone(),
        pub_flow_file: spec.traces.pub_flow_file.clone(),
        val_flow_file: spec.traces.val_flow_file.clone(),
        watch_flow_file: spec.traces.watch_flow_file.clone(),
    };
    let wallet_scan = load_wallet_scan_artifact(out_dir, &observability.packet)?;
    let publication_cfg = publication_cfg(cfg)?;
    let publication = build_publication_evidence(spec, cfg, out_dir)?;
    let placement_rows = spec.journal_view.lineage_rows.clone();
    let stage_results = run
        .stages
        .iter()
        .map(|stage| StageResultView {
            stage: stage.stage,
            name: stage.name.clone(),
            result: stage_result_tag(&stage.result),
        })
        .collect::<Vec<_>>();
    let hjmt = hjmt_cfg(&spec.hjmt_home)?;
    let planner = hjmt.planner;
    let storage = hjmt.storage;

    write_trace_file(
        out_dir,
        &spec.traces.cfg_flow_file,
        &CfgFlow {
            common: common.clone_with("cfg_flow", RUNTIME_CONTRACT),
            route_table_path: spec.route_table_path.clone(),
            routing_generation: spec.routing_generation,
            supported_profiles,
            heavy_only_profiles,
            trace_files: trace_files.clone(),
            design_stage_ids: design.stages.iter().map(|stage| stage.stage).collect(),
            design_stage_names: design
                .stages
                .iter()
                .map(|stage| stage.name.clone())
                .collect(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.tx_flow_file,
        &TxFlow {
            common: common.clone_with("tx_flow", RUNTIME_CONTRACT),
            tx_package_path: stage4_tx_pkg_path(cfg, out_dir)?,
            transfer_leaf_path: stage5_leaf_path(cfg, out_dir),
            bundle_frag1_path: stage6_frag_path(cfg, out_dir, true),
            bundle_frag2_path: stage6_frag_path(cfg, out_dir, false),
            bundle_bridge_path: stage6_bridge_path(cfg, out_dir),
            checkpoint_apply_path: stage7_checkpoint_path(cfg, out_dir),
            checkpoint_finalize_path: stage8_checkpoint_path(cfg, out_dir),
            hjmt_examples_report_path: resolve_path(out_dir, "hjmt/hjmt_settlement_examples.json"),
            stage_results: stage_results.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.route_flow_file,
        &RouteFlow {
            common: common.clone_with("route_flow", RUNTIME_CONTRACT),
            route_table_path: spec.route_table_path.clone(),
            routing_generation: spec.routing_generation,
            placement_rows: placement_rows.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.plan_flow_file,
        &PlanFlow {
            common: common.clone_with("plan_flow", RUNTIME_CONTRACT),
            planner_mode: planner.mode.as_str().to_string(),
            planner_config_path: planner.cfg_path,
            planner_policy: PlannerPolicyView {
                shard_local_only: planner.policy.shard_local_only,
                reject_cross_shard: planner.policy.reject_cross_shard,
                cadence_ms: planner.policy.cadence_ms,
            },
            planner_limits: PlannerLimitsView {
                max_batch_ops: planner.limits.max_batch_ops,
                max_batch_bytes: planner.limits.max_batch_bytes,
            },
            planner_plan_dir: planner.paths.plan_dir,
            planner_evidence_dir: planner.paths.evidence_dir,
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.journal_flow_file,
        &JournalFlow {
            common: common.clone_with("journal_flow", RUNTIME_CONTRACT),
            journal_contract: spec.journal_view.clone(),
            cache_edge_samples: cache_edge_samples(spec),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.scope_flow_file,
        &build_scope_flow(
            common.clone_with("scope_flow", LINKED_OWNER_CONTRACT),
            cfg,
            out_dir,
            &wallet_scan,
        )?,
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.proc_flow_file,
        &ProcFlow {
            common: common.clone_with("proc_flow", RUNTIME_CONTRACT),
            process_topology: spec.process_view.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.recovery_flow_file,
        &RecoveryFlow {
            common: common.clone_with("recovery_flow", LINKED_OWNER_CONTRACT),
            recovery_owner_homes: [RECOVERY_OWNER_HOME_RUNTIME, RECOVERY_OWNER_HOME_STORAGE],
            failover_rows: placement_rows,
            startup_checks_required: [
                "route_codec",
                "placement",
                "journal_lineage",
                "backend_generation",
                "proof_codec",
                "handoff_ready",
                "crypto_tags",
            ],
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.leaf_flow_file,
        &LeafFlow {
            common: common.clone_with_version(
                PUBLICATION_TRACE_VERSION,
                "leaf_flow",
                PUBLICATION_CONTRACT,
            ),
            publication_profile: publication_cfg.acceptance_profile.clone(),
            inherited_runtime_profile: publication_cfg.inherited_runtime_profile.clone(),
            topology_status: publication_cfg.topology_status.clone(),
            public_leaf_count: publication_cfg.public_leaf_count,
            publication_checkpoint: publication.publication_checkpoint,
            prior_public_root_hex: publication.prior_public_root_hex.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            leaf_rows: publication.leaf_rows.clone(),
            topology_examples: publication_topology_views(publication_cfg),
            linked_trace_files: trace_files.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.proof_flow_file,
        &ProofFlow {
            common: common.clone_with_version(
                PUBLICATION_TRACE_VERSION,
                "proof_flow",
                PUBLICATION_CONTRACT,
            ),
            publication_profile: publication_cfg.acceptance_profile.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            public_root_hex: publication.public_root_hex.clone(),
            proof_rows: publication.proof_rows.clone(),
            linked_trace_files: trace_files.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.pub_flow_file,
        &PubFlow {
            common: common.clone_with_version(
                PUBLICATION_TRACE_VERSION,
                "pub_flow",
                PUBLICATION_CONTRACT,
            ),
            publication_profile: publication_cfg.acceptance_profile.clone(),
            inherited_runtime_profile: publication_cfg.inherited_runtime_profile.clone(),
            topology_status: publication_cfg.topology_status.clone(),
            public_leaf_count: publication_cfg.public_leaf_count,
            publication_checkpoint: publication.publication_checkpoint,
            activation_checkpoint: publication_cfg.publication_activation_checkpoint,
            route_generation: spec.routing_generation,
            prior_public_root_hex: publication.prior_public_root_hex.clone(),
            public_root_hex: publication.public_root_hex.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            canonical_publication_hex: publication.canonical_publication_hex.clone(),
            topology_examples: publication_topology_views(publication_cfg),
            process_verdicts: publication.process_verdicts.clone(),
            linked_trace_files: trace_files.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.val_flow_file,
        &ValFlow {
            common: common.clone_with_version(
                PUBLICATION_TRACE_VERSION,
                "val_flow",
                PUBLICATION_CONTRACT,
            ),
            publication_profile: publication_cfg.acceptance_profile.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            binding_digest_hex: hex::encode(publication.binding.binding_digest()),
            draft_id_hex: publication.draft_id_hex.clone(),
            checkpoint_id_hex: publication.checkpoint_id_hex.clone(),
            route_generation: spec.routing_generation,
            verdict_kind: publication.verdict_kind.clone(),
            prev_settlement_root_hex: hex::encode(
                publication.binding.prev_settlement_root().into_bytes(),
            ),
            new_settlement_root_hex: hex::encode(
                publication.binding.new_settlement_root().into_bytes(),
            ),
            spent_delta_count: publication.binding.spent_count(),
            created_delta_count: publication.binding.created_count(),
            topology_examples: publication_topology_views(publication_cfg),
            linked_trace_files: trace_files.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        &spec.traces.watch_flow_file,
        &WatchFlow {
            common: common.clone_with_version(
                PUBLICATION_TRACE_VERSION,
                "watch_flow",
                PUBLICATION_CONTRACT,
            ),
            publication_profile: publication_cfg.acceptance_profile.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            binding_digest_hex: hex::encode(publication.binding.binding_digest()),
            draft_id_hex: publication.draft_id_hex.clone(),
            checkpoint_id_hex: publication.checkpoint_id_hex.clone(),
            verdict_kind: publication.verdict_kind.clone(),
            publication_state: publication.publication_state.clone(),
            topology_examples: publication_topology_views(publication_cfg),
            process_verdicts: publication.process_verdicts.clone(),
            linked_trace_files: trace_files.clone(),
        },
    )?;
    write_trace_file(
        out_dir,
        emitted_public_file(&observability.packet, "hist_flow.json")?,
        &build_hist_flow(
            common.clone_with_version(
                RELEASE_PACKET_VERSION,
                "hist_flow",
                IMPORTED_ARTIFACT_CONTRACT,
            ),
            cfg,
            out_dir,
            &trace_files,
        )?,
    )?;
    write_trace_file(
        out_dir,
        emitted_public_file(&observability.packet, "occ_flow.json")?,
        &build_occ_flow(
            common.clone_with_version(
                RELEASE_PACKET_VERSION,
                "occ_flow",
                IMPORTED_ARTIFACT_CONTRACT,
            ),
            cfg,
            out_dir,
            &trace_files,
        )?,
    )?;
    for emitted_file in ["asset_flow.json", "voucher_flow.json", "right_flow.json"] {
        write_trace_file(
            out_dir,
            emitted_public_file(&observability.packet, emitted_file)?,
            &build_object_flow(
                common.clone_with_version(
                    RELEASE_PACKET_VERSION,
                    emitted_file.trim_end_matches(".json"),
                    IMPORTED_ARTIFACT_CONTRACT,
                ),
                cfg,
                emitted_file,
            )?,
        )?;
    }
    let run_meta = build_run_meta(
        spec,
        &observability.packet,
        &trace_files,
        &stage_results,
        &wallet_scan,
    );
    write_trace_file(out_dir, &observability.packet.run_meta_file, &run_meta)?;
    write_text_file(
        out_dir,
        &observability.packet.sim_summary_file,
        &build_sim_summary(&run_meta),
    )?;

    let _ = storage;
    Ok(())
}

pub(super) fn validate(
    cfg_path: &Path,
    design_path: &Path,
    out_dir: &Path,
) -> Result<(), Scenario1Err> {
    validate_with_mode(cfg_path, design_path, out_dir, PacketMode::Cached)
}

pub(super) fn validate_strict(
    cfg_path: &Path,
    design_path: &Path,
    out_dir: &Path,
) -> Result<(), Scenario1Err> {
    validate_with_mode(cfg_path, design_path, out_dir, PacketMode::Strict)
}

fn validate_with_mode(
    cfg_path: &Path,
    design_path: &Path,
    out_dir: &Path,
    mode: PacketMode,
) -> Result<(), Scenario1Err> {
    let cfg = ScenarioCfg::from_file(cfg_path)?;
    let design = DesignDoc::from_file(design_path).map_err(Scenario1Err::Design)?;
    let spec = prepare(cfg_path, design_path, &cfg)?;
    let expected_design_path = trace_design_path(design_path);
    let normalized_out_dir = normalize_path(out_dir);
    let planner = hjmt_cfg(&spec.hjmt_home)?.planner;
    let publication = build_publication_evidence(&spec, &cfg, &normalized_out_dir)?;
    let expected = [
        ("cfg_flow", &spec.traces.cfg_flow_file, RUNTIME_CONTRACT),
        ("tx_flow", &spec.traces.tx_flow_file, RUNTIME_CONTRACT),
        ("route_flow", &spec.traces.route_flow_file, RUNTIME_CONTRACT),
        ("plan_flow", &spec.traces.plan_flow_file, RUNTIME_CONTRACT),
        (
            "journal_flow",
            &spec.traces.journal_flow_file,
            RUNTIME_CONTRACT,
        ),
        (
            "scope_flow",
            &spec.traces.scope_flow_file,
            LINKED_OWNER_CONTRACT,
        ),
        ("proc_flow", &spec.traces.proc_flow_file, RUNTIME_CONTRACT),
        (
            "recovery_flow",
            &spec.traces.recovery_flow_file,
            LINKED_OWNER_CONTRACT,
        ),
        (
            "leaf_flow",
            &spec.traces.leaf_flow_file,
            PUBLICATION_CONTRACT,
        ),
        (
            "proof_flow",
            &spec.traces.proof_flow_file,
            PUBLICATION_CONTRACT,
        ),
        ("pub_flow", &spec.traces.pub_flow_file, PUBLICATION_CONTRACT),
        ("val_flow", &spec.traces.val_flow_file, PUBLICATION_CONTRACT),
        (
            "watch_flow",
            &spec.traces.watch_flow_file,
            PUBLICATION_CONTRACT,
        ),
    ];
    let semantic_digest_hex = semantic_digest_hex(&spec);

    for (trace_kind, rel_path, trace_mode) in expected {
        let path = resolve_trace_file(&normalized_out_dir, rel_path)?;
        let bytes = io::read_file(&path).map_err(|err| {
            Scenario1Err::Evidence(format!(
                "failed to read runtime trace {}: {err}",
                path.display()
            ))
        })?;
        let value: Value = JsonCodec.deserialize(&bytes).map_err(|err| {
            Scenario1Err::Evidence(format!("failed to decode {}: {err}", path.display()))
        })?;

        check_string(
            &value,
            "trace_version",
            expected_trace_version(trace_kind),
            &path,
        )?;
        check_string(&value, "trace_kind", trace_kind, &path)?;
        check_string(&value, "trace_mode", trace_mode, &path)?;
        check_u64(&value, "scenario_id", spec.scenario_id as u64, &path)?;
        check_string(&value, "scenario_name", &spec.scenario_name, &path)?;
        check_string(&value, "active_profile", &spec.active_profile, &path)?;
        check_bool(&value, "deterministic", spec.deterministic, &path)?;
        check_string(&value, "semantic_digest_hex", &semantic_digest_hex, &path)?;
        check_string(
            &value,
            "config_digest_set_hex",
            &spec.config_digest_set_hex,
            &path,
        )?;
        check_string(&value, "design_digest_hex", &spec.design_digest_hex, &path)?;
        check_string(
            &value,
            "route_table_digest",
            &spec.route_table_digest,
            &path,
        )?;
        check_string(
            &value,
            "process_topology_digest_hex",
            &spec.process_topology_digest_hex,
            &path,
        )?;
        check_string(
            &value,
            "journal_lineage_digest_hex",
            &spec.journal_lineage_digest_hex,
            &path,
        )?;
        check_path(&value, "scenario_config_path", cfg_path, &path)?;
        check_path(&value, "design_path", &expected_design_path, &path)?;
        check_path(&value, "hjmt_home", &spec.hjmt_home, &path)?;
        check_trace_payload(
            &value,
            &expected_trace_payload(
                trace_kind,
                trace_mode,
                &spec,
                &cfg,
                &design,
                &planner,
                cfg_path,
                design_path,
                &normalized_out_dir,
                &publication,
            )?,
            &path,
        )?;
    }

    validate_release_packet(
        &spec,
        &cfg,
        &design,
        cfg_path,
        design_path,
        &normalized_out_dir,
        mode,
    )?;

    Ok(())
}

fn expected_trace_payload(
    trace_kind: &str,
    trace_mode: &str,
    spec: &RuntimeTraceSpec,
    cfg: &ScenarioCfg,
    design: &DesignDoc,
    planner: &z00z_rollup_node::PlanCfg,
    cfg_path: &Path,
    design_path: &Path,
    out_dir: &Path,
    publication: &PublicationEvidence,
) -> Result<Value, Scenario1Err> {
    let common = flow_common(spec, cfg_path, design_path).clone_with_version(
        expected_trace_version(trace_kind),
        match trace_kind {
            "cfg_flow" => "cfg_flow",
            "tx_flow" => "tx_flow",
            "route_flow" => "route_flow",
            "plan_flow" => "plan_flow",
            "journal_flow" => "journal_flow",
            "scope_flow" => "scope_flow",
            "proc_flow" => "proc_flow",
            "recovery_flow" => "recovery_flow",
            "leaf_flow" => "leaf_flow",
            "proof_flow" => "proof_flow",
            "pub_flow" => "pub_flow",
            "val_flow" => "val_flow",
            "watch_flow" => "watch_flow",
            other => {
                return Err(Scenario1Err::Evidence(format!(
                    "unsupported runtime trace kind {other}"
                )))
            }
        },
        match trace_mode {
            RUNTIME_CONTRACT => RUNTIME_CONTRACT,
            LINKED_OWNER_CONTRACT => LINKED_OWNER_CONTRACT,
            PUBLICATION_CONTRACT => PUBLICATION_CONTRACT,
            other => {
                return Err(Scenario1Err::Evidence(format!(
                    "unsupported runtime trace mode {other}"
                )))
            }
        },
    );
    let observability = observability_cfg(cfg)?;
    let supported_profiles = observability
        .supported_profiles
        .iter()
        .map(|profile| RuntimeProfileView {
            id: profile.id.clone(),
            deterministic: profile.deterministic,
            purpose: profile.purpose.clone(),
        })
        .collect::<Vec<_>>();
    let heavy_only_profiles = observability.heavy_only_profiles.clone();
    let trace_files = TraceFileView {
        cfg_flow_file: spec.traces.cfg_flow_file.clone(),
        tx_flow_file: spec.traces.tx_flow_file.clone(),
        route_flow_file: spec.traces.route_flow_file.clone(),
        plan_flow_file: spec.traces.plan_flow_file.clone(),
        journal_flow_file: spec.traces.journal_flow_file.clone(),
        scope_flow_file: spec.traces.scope_flow_file.clone(),
        proc_flow_file: spec.traces.proc_flow_file.clone(),
        recovery_flow_file: spec.traces.recovery_flow_file.clone(),
        leaf_flow_file: spec.traces.leaf_flow_file.clone(),
        proof_flow_file: spec.traces.proof_flow_file.clone(),
        pub_flow_file: spec.traces.pub_flow_file.clone(),
        val_flow_file: spec.traces.val_flow_file.clone(),
        watch_flow_file: spec.traces.watch_flow_file.clone(),
    };
    let placement_rows = spec.journal_view.lineage_rows.clone();
    let publication_cfg = publication_cfg(cfg)?;

    match trace_kind {
        "cfg_flow" => json_value(&CfgFlow {
            common,
            route_table_path: spec.route_table_path.clone(),
            routing_generation: spec.routing_generation,
            supported_profiles,
            heavy_only_profiles,
            trace_files: trace_files.clone(),
            design_stage_ids: design.stages.iter().map(|stage| stage.stage).collect(),
            design_stage_names: design
                .stages
                .iter()
                .map(|stage| stage.name.clone())
                .collect(),
        }),
        "tx_flow" => json_value(&TxFlow {
            common,
            tx_package_path: stage4_tx_pkg_path(cfg, out_dir)?,
            transfer_leaf_path: stage5_leaf_path(cfg, out_dir),
            bundle_frag1_path: stage6_frag_path(cfg, out_dir, true),
            bundle_frag2_path: stage6_frag_path(cfg, out_dir, false),
            bundle_bridge_path: stage6_bridge_path(cfg, out_dir),
            checkpoint_apply_path: stage7_checkpoint_path(cfg, out_dir),
            checkpoint_finalize_path: stage8_checkpoint_path(cfg, out_dir),
            hjmt_examples_report_path: resolve_path(out_dir, "hjmt/hjmt_settlement_examples.json"),
            stage_results: design
                .stages
                .iter()
                .map(|stage| StageResultView {
                    stage: stage.stage,
                    name: stage.name.clone(),
                    result: "ok",
                })
                .collect(),
        }),
        "route_flow" => json_value(&RouteFlow {
            common,
            route_table_path: spec.route_table_path.clone(),
            routing_generation: spec.routing_generation,
            placement_rows,
        }),
        "plan_flow" => json_value(&PlanFlow {
            common,
            planner_mode: planner.mode.as_str().to_string(),
            planner_config_path: planner.cfg_path.clone(),
            planner_policy: PlannerPolicyView {
                shard_local_only: planner.policy.shard_local_only,
                reject_cross_shard: planner.policy.reject_cross_shard,
                cadence_ms: planner.policy.cadence_ms,
            },
            planner_limits: PlannerLimitsView {
                max_batch_ops: planner.limits.max_batch_ops,
                max_batch_bytes: planner.limits.max_batch_bytes,
            },
            planner_plan_dir: planner.paths.plan_dir.clone(),
            planner_evidence_dir: planner.paths.evidence_dir.clone(),
        }),
        "journal_flow" => json_value(&JournalFlow {
            common,
            journal_contract: spec.journal_view.clone(),
            cache_edge_samples: cache_edge_samples(spec),
        }),
        "scope_flow" => json_value(&build_scope_flow(
            common,
            cfg,
            out_dir,
            &load_wallet_scan_artifact(out_dir, &observability.packet)?,
        )?),
        "proc_flow" => json_value(&ProcFlow {
            common,
            process_topology: spec.process_view.clone(),
        }),
        "recovery_flow" => json_value(&RecoveryFlow {
            common,
            recovery_owner_homes: [RECOVERY_OWNER_HOME_RUNTIME, RECOVERY_OWNER_HOME_STORAGE],
            failover_rows: spec.journal_view.lineage_rows.clone(),
            startup_checks_required: [
                "route_codec",
                "placement",
                "journal_lineage",
                "backend_generation",
                "proof_codec",
                "handoff_ready",
                "crypto_tags",
            ],
        }),
        "leaf_flow" => json_value(&LeafFlow {
            common,
            publication_profile: publication_cfg.acceptance_profile.clone(),
            inherited_runtime_profile: publication_cfg.inherited_runtime_profile.clone(),
            topology_status: publication_cfg.topology_status.clone(),
            public_leaf_count: publication_cfg.public_leaf_count,
            publication_checkpoint: publication.publication_checkpoint,
            prior_public_root_hex: publication.prior_public_root_hex.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            leaf_rows: publication.leaf_rows.clone(),
            topology_examples: publication_topology_views(publication_cfg),
            linked_trace_files: trace_files,
        }),
        "proof_flow" => json_value(&ProofFlow {
            common,
            publication_profile: publication_cfg.acceptance_profile.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            public_root_hex: publication.public_root_hex.clone(),
            proof_rows: publication.proof_rows.clone(),
            linked_trace_files: trace_files,
        }),
        "pub_flow" => json_value(&PubFlow {
            common,
            publication_profile: publication_cfg.acceptance_profile.clone(),
            inherited_runtime_profile: publication_cfg.inherited_runtime_profile.clone(),
            topology_status: publication_cfg.topology_status.clone(),
            public_leaf_count: publication_cfg.public_leaf_count,
            publication_checkpoint: publication.publication_checkpoint,
            activation_checkpoint: publication_cfg.publication_activation_checkpoint,
            route_generation: spec.routing_generation,
            prior_public_root_hex: publication.prior_public_root_hex.clone(),
            public_root_hex: publication.public_root_hex.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            canonical_publication_hex: publication.canonical_publication_hex.clone(),
            topology_examples: publication_topology_views(publication_cfg),
            process_verdicts: publication.process_verdicts.clone(),
            linked_trace_files: trace_files,
        }),
        "val_flow" => json_value(&ValFlow {
            common,
            publication_profile: publication_cfg.acceptance_profile.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            binding_digest_hex: hex::encode(publication.binding.binding_digest()),
            draft_id_hex: publication.draft_id_hex.clone(),
            checkpoint_id_hex: publication.checkpoint_id_hex.clone(),
            route_generation: spec.routing_generation,
            verdict_kind: publication.verdict_kind.clone(),
            prev_settlement_root_hex: hex::encode(
                publication.binding.prev_settlement_root().into_bytes(),
            ),
            new_settlement_root_hex: hex::encode(
                publication.binding.new_settlement_root().into_bytes(),
            ),
            spent_delta_count: publication.binding.spent_count(),
            created_delta_count: publication.binding.created_count(),
            topology_examples: publication_topology_views(publication_cfg),
            linked_trace_files: trace_files,
        }),
        "watch_flow" => json_value(&WatchFlow {
            common,
            publication_profile: publication_cfg.acceptance_profile.clone(),
            publication_digest_hex: publication.publication_digest_hex.clone(),
            binding_digest_hex: hex::encode(publication.binding.binding_digest()),
            draft_id_hex: publication.draft_id_hex.clone(),
            checkpoint_id_hex: publication.checkpoint_id_hex.clone(),
            verdict_kind: publication.verdict_kind.clone(),
            publication_state: publication.publication_state.clone(),
            topology_examples: publication_topology_views(publication_cfg),
            process_verdicts: publication.process_verdicts.clone(),
            linked_trace_files: trace_files,
        }),
        other => Err(Scenario1Err::Evidence(format!(
            "unsupported runtime trace kind {other}"
        ))),
    }
}

fn build_scope_flow(
    common: FlowCommon,
    cfg: &ScenarioCfg,
    out_dir: &Path,
    wallet_scan: &JmtScanArtifact,
) -> Result<ScopeFlow, Scenario1Err> {
    Ok(ScopeFlow {
        common,
        semantic_owner: "z00z_storage",
        trace_owner_home: SCOPE_OWNER_HOME,
        trace_owner_mode: "semantic_scope_birth_contract",
        linked_stage_ids: vec![11, 13],
        private_tree_id_exposed: false,
        owner_contract_rows: vec![
            ScopeOwnerContractView {
                contract_id: "first_seen_terminal_birth",
                owner_home: SCOPE_OWNER_HOME,
                proof_point: "test_scope_flow_records_first_seen_birth_and_reload",
                leaf_family: "terminal",
                first_seen_definition: true,
                first_seen_serial: true,
                first_seen_object: true,
                status: "linked_owner_contract",
            },
            ScopeOwnerContractView {
                contract_id: "first_right_creation",
                owner_home: SCOPE_OWNER_HOME,
                proof_point: "test_scope_flow_records_first_right_creation",
                leaf_family: "right",
                first_seen_definition: false,
                first_seen_serial: false,
                first_seen_object: true,
                status: "linked_owner_contract",
            },
            ScopeOwnerContractView {
                contract_id: "mixed_family_reject",
                owner_home: SCOPE_OWNER_HOME,
                proof_point: "test_handoff_rejects_mixed_families",
                leaf_family: "mixed_terminal_right",
                first_seen_definition: false,
                first_seen_serial: false,
                first_seen_object: false,
                status: "linked_owner_contract",
            },
            ScopeOwnerContractView {
                contract_id: "duplicate_terminal_reject",
                owner_home: SCOPE_OWNER_HOME,
                proof_point: "test_exec_handoff_rejects_duplicate_terminal_id",
                leaf_family: "right",
                first_seen_definition: false,
                first_seen_serial: false,
                first_seen_object: false,
                status: "linked_owner_contract",
            },
        ],
        wallet_promotion_rows: build_wallet_promotion_rows(cfg, out_dir, wallet_scan)?,
        wallet_negative_rows: build_wallet_negative_rows(wallet_scan),
        proof_boundary: "proof_blob+chk_blob_settlement before ownership detection",
        restart_failover_owner_homes: [RECOVERY_OWNER_HOME_RUNTIME, RECOVERY_OWNER_HOME_STORAGE],
        acceptance_homes: vec![
            AcceptanceHomeView {
                home: WALLET_SCAN_OWNER_HOME,
                purpose: "proof_before_ownership_and_final_wallet_promotion",
                status: "live_home",
            },
            AcceptanceHomeView {
                home: TRANSITION_OWNER_HOME,
                purpose: "historical_transition_acceptance",
                status: "live_home",
            },
            AcceptanceHomeView {
                home: PRIVACY_OWNER_HOME,
                purpose: "occupancy_privacy_regression",
                status: "live_home",
            },
            AcceptanceHomeView {
                home: E2E_OWNER_HOME,
                purpose: "scope_birth_to_wallet_e2e",
                status: "live_home",
            },
        ],
    })
}

fn build_wallet_promotion_rows(
    cfg: &ScenarioCfg,
    out_dir: &Path,
    wallet_scan: &JmtScanArtifact,
) -> Result<Vec<ScopeWalletPromotionView>, Scenario1Err> {
    let output_bindings = load_tx_output_bindings(out_dir)?;
    let pending_rows: Vec<WalletLifecycleRow> =
        load_json_file(&wallet_pending_path(cfg, out_dir)?, "wallet pending rows")?;
    let confirmed_rows: Vec<WalletLifecycleRow> = load_json_file(
        &wallet_confirmed_path(cfg, out_dir)?,
        "wallet confirmed rows",
    )?;
    let diff_root: WalletDiffRoot =
        load_json_file(&wallet_diff_path(cfg, out_dir)?, "wallet diff rows")?;

    let pending = pending_rows
        .into_iter()
        .filter(|row| row.actor == "charlie")
        .map(|row| (row.asset_id_hex.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let confirmed = confirmed_rows
        .into_iter()
        .filter(|row| row.actor == "charlie")
        .map(|row| (row.asset_id_hex.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let diff = diff_root
        .rows
        .into_iter()
        .filter(|row| row.actor == "charlie")
        .map(|row| (row.asset_id_hex.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = pending
        .into_iter()
        .map(|(asset_id_hex, pending_row)| {
            let scan_row = wallet_scan
                .rows
                .iter()
                .find(|row| row.asset_id_hex == asset_id_hex)
                .ok_or_else(|| {
                    Scenario1Err::Evidence(format!(
                        "scope_flow wallet promotion missing wallet_scan row for {asset_id_hex}"
                    ))
                })?;
            let confirmed_row = confirmed.get(&asset_id_hex).ok_or_else(|| {
                Scenario1Err::Evidence(format!(
                    "scope_flow wallet promotion missing confirmed row for {asset_id_hex}"
                ))
            })?;
            let diff_row = diff.get(&asset_id_hex).ok_or_else(|| {
                Scenario1Err::Evidence(format!(
                    "scope_flow wallet promotion missing diff row for {asset_id_hex}"
                ))
            })?;
            let output = output_bindings.get(&asset_id_hex).ok_or_else(|| {
                Scenario1Err::Evidence(format!(
                    "scope_flow wallet promotion missing committed candidate binding for {asset_id_hex}"
                ))
            })?;
            if pending_row.serial_id != output.serial_id || confirmed_row.serial_id != output.serial_id
            {
                return Err(Scenario1Err::Evidence(format!(
                    "scope_flow wallet promotion serial drifted for {asset_id_hex}: pending={} confirmed={} committed={}",
                    pending_row.serial_id, confirmed_row.serial_id, output.serial_id
                )));
            }
            if pending_row.amount != confirmed_row.amount {
                return Err(Scenario1Err::Evidence(format!(
                    "scope_flow wallet promotion amount drifted for {asset_id_hex}: pending={} confirmed={}",
                    pending_row.amount, confirmed_row.amount
                )));
            }

            Ok(ScopeWalletPromotionView {
                asset_id_hex,
                definition_id: output.definition_id.clone(),
                serial_id: pending_row.serial_id,
                amount: pending_row.amount,
                output_role: pending_row
                    .output_role
                    .clone()
                    .or_else(|| confirmed_row.output_role.clone())
                    .unwrap_or_else(|| "recipient".to_string()),
                scan_receive_status: scan_row.receive_status.clone(),
                scan_receive_next: scan_row.receive_next.clone(),
                pending_lifecycle_status: pending_row.lifecycle_status.clone(),
                confirmed_lifecycle_status: confirmed_row.lifecycle_status.clone(),
                diff_status: diff_row.status.clone(),
                proof_validated: scan_row.proof_validated,
                owner_detected: scan_row.owner_detected,
                tx_digest_hex: pending_row
                    .tx_digest_hex
                    .clone()
                    .or_else(|| confirmed_row.tx_digest_hex.clone())
                    .unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, Scenario1Err>>()?;
    rows.sort_by_key(|row| row.serial_id);
    Ok(rows)
}

fn build_wallet_negative_rows(wallet_scan: &JmtScanArtifact) -> Vec<WalletNegativeSummaryView> {
    let mut counts = BTreeMap::<(String, String), usize>::new();
    for row in &wallet_scan.rows {
        if row.owner_detected {
            continue;
        }
        *counts
            .entry((row.receive_status.clone(), row.receive_next.clone()))
            .or_default() += 1;
    }

    counts
        .into_iter()
        .map(
            |((receive_status, receive_next), count)| WalletNegativeSummaryView {
                receive_status,
                receive_next,
                count,
            },
        )
        .collect()
}

fn wallet_status_label(
    lifecycle: RuntimeTxLifecycle,
    error_code: Option<RuntimeTxErrorCode>,
) -> String {
    if error_code.is_some() {
        return "rejected".to_string();
    }

    match lifecycle {
        RuntimeTxLifecycle::Confirmed => "confirmed".to_string(),
        RuntimeTxLifecycle::Failed
        | RuntimeTxLifecycle::Cancelled
        | RuntimeTxLifecycle::Conflicted
        | RuntimeTxLifecycle::AlreadySpent => "rejected".to_string(),
        RuntimeTxLifecycle::Created
        | RuntimeTxLifecycle::Imported
        | RuntimeTxLifecycle::Exported
        | RuntimeTxLifecycle::Submitted
        | RuntimeTxLifecycle::Admitted => "pending".to_string(),
    }
}

fn wallet_history_path(output_dir: &Path, wallet_id: &PersistWalletId) -> PathBuf {
    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_stem = hex::encode(&hash[..8]);
    output_dir.join(format!("wallet_{wallet_stem}_tx_history.jsonl"))
}

fn wallet_file_path(output_dir: &Path, wallet_id: &PersistWalletId) -> PathBuf {
    let hash = compute_wallet_file_id(&wallet_id.0);
    let wallet_stem = hex::encode(&hash[..8]);
    output_dir.join(format!("wallet_{wallet_stem}.wlt"))
}

fn read_wallet_history_rows(
    output_dir: &Path,
    wallet_id: &PersistWalletId,
) -> Result<Vec<WalletTxHistoryJsonlEntry>, Scenario1Err> {
    let history_path = wallet_history_path(output_dir, wallet_id);
    if !history_path.exists() {
        return Ok(Vec::new());
    }
    let bytes = io::read_file(&history_path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to read tx-history {}: {err}",
            history_path.display()
        ))
    })?;
    decode_tx_history_rows(&bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to decode tx-history {}: {err}",
            history_path.display()
        ))
    })
}

fn wallet_history_digest_hex(
    output_dir: &Path,
    wallet_id: &PersistWalletId,
) -> Result<String, Scenario1Err> {
    let history_path = wallet_history_path(output_dir, wallet_id);
    let bytes = if history_path.exists() {
        io::read_file(&history_path).map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation failed to hash tx-history {}: {err}",
                history_path.display()
            ))
        })?
    } else {
        Vec::new()
    };
    Ok(hex::encode(Sha256::digest(&bytes)))
}

fn wallet_lifecycle_from_record(
    record: Option<&TxRecord>,
    rows: &[WalletTxHistoryJsonlEntry],
    tx_hash: &str,
) -> Option<RuntimeTxLifecycle> {
    let record = record?;
    let latest_kind = rows
        .iter()
        .rev()
        .find(|row| row.tx_hash == tx_hash)
        .map(|row| row.entry_kind);

    Some(
        if matches!(latest_kind, Some(WalletTxHistoryEntryKind::Conflicted)) {
            RuntimeTxLifecycle::Conflicted
        } else if matches!(latest_kind, Some(WalletTxHistoryEntryKind::AlreadySpent)) {
            RuntimeTxLifecycle::AlreadySpent
        } else if record.status == WalletTxStatus::Confirmed
            || matches!(latest_kind, Some(WalletTxHistoryEntryKind::Confirmed))
        {
            RuntimeTxLifecycle::Confirmed
        } else if record.status == WalletTxStatus::Failed
            || matches!(latest_kind, Some(WalletTxHistoryEntryKind::Failed))
        {
            RuntimeTxLifecycle::Failed
        } else if record.status == WalletTxStatus::Cancelled
            || matches!(latest_kind, Some(WalletTxHistoryEntryKind::Cancelled))
        {
            RuntimeTxLifecycle::Cancelled
        } else {
            match latest_kind {
                Some(WalletTxHistoryEntryKind::Created) => RuntimeTxLifecycle::Created,
                Some(WalletTxHistoryEntryKind::Imported) => RuntimeTxLifecycle::Imported,
                Some(WalletTxHistoryEntryKind::Exported) => RuntimeTxLifecycle::Exported,
                Some(WalletTxHistoryEntryKind::Submitted) => RuntimeTxLifecycle::Submitted,
                Some(WalletTxHistoryEntryKind::Admitted) => RuntimeTxLifecycle::Admitted,
                Some(WalletTxHistoryEntryKind::Tombstoned) => {
                    if record.imported {
                        RuntimeTxLifecycle::Imported
                    } else {
                        RuntimeTxLifecycle::Created
                    }
                }
                Some(WalletTxHistoryEntryKind::Conflicted) => RuntimeTxLifecycle::Conflicted,
                Some(WalletTxHistoryEntryKind::AlreadySpent) => RuntimeTxLifecycle::AlreadySpent,
                Some(WalletTxHistoryEntryKind::Confirmed) => RuntimeTxLifecycle::Confirmed,
                Some(WalletTxHistoryEntryKind::Failed) => RuntimeTxLifecycle::Failed,
                Some(WalletTxHistoryEntryKind::Cancelled) => RuntimeTxLifecycle::Cancelled,
                None if record.imported => RuntimeTxLifecycle::Imported,
                None => RuntimeTxLifecycle::Created,
            }
        },
    )
}

async fn wallet_snapshot(
    wallet_service: &WalletService,
    output_dir: &Path,
    wallet_id: &PersistWalletId,
) -> Result<WalletLifecycleSnapshot, Scenario1Err> {
    let asset_ids = wallet_service
        .list_claimed_assets(wallet_id)
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation failed to list claimed assets for {}: {err}",
                wallet_id.0
            ))
        })?
        .into_iter()
        .map(|asset| hex::encode(asset.asset_id()))
        .collect::<BTreeSet<_>>();
    let history_rows = read_wallet_history_rows(output_dir, wallet_id)?;
    let history_digest_hex = wallet_history_digest_hex(output_dir, wallet_id)?;

    Ok(WalletLifecycleSnapshot {
        asset_ids,
        history_len: history_rows.len(),
        history_digest_hex,
    })
}

async fn restart_wallet_state_matches(
    output_dir: &Path,
    time_provider: Arc<MockTimeProvider>,
    wallet_id: &PersistWalletId,
    expected: &WalletLifecycleSnapshot,
    tx_id: &PersistTxId,
    expected_lifecycle: Option<RuntimeTxLifecycle>,
) -> Result<WalletRestartCheck, Scenario1Err> {
    let mut seed_hasher = Sha256::new();
    seed_hasher.update(b"wallet-lifecycle-restart");
    seed_hasher.update(output_dir.display().to_string().as_bytes());
    seed_hasher.update(wallet_id.0.as_bytes());
    let restart_wallet_service = Arc::new(WalletService::create_service_custom_output_directory(
        output_dir.to_path_buf(),
        time_provider.clone(),
        DeterministicSecureRngProvider::new(seed_hasher.finalize().into()),
    ));
    let restart_app = Arc::new(AppService::with_dependencies(
        time_provider.clone(),
        Arc::clone(&restart_wallet_service),
    ));
    let wallet_path = wallet_file_path(output_dir, wallet_id);
    restart_app
        .open_wallet_source(WalletSource::Path {
            path: wallet_path.to_string_lossy().to_string(),
        })
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation restart open failed for {}: {err}",
                wallet_path.display()
            ))
        })?;
    restart_wallet_service
        .unlock_wallet_in_memory(wallet_id, &SafePassword::from(WALLET_SIM_PASSWORD))
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation restart unlock failed for {}: {err}",
                wallet_id.0
            ))
        })?;

    let reopened_snapshot = wallet_snapshot(&restart_wallet_service, output_dir, wallet_id).await?;
    let history_path = wallet_history_path(output_dir, wallet_id);
    let store = TxStorageImpl::new(&history_path, time_provider.as_ref().clone());
    let record = match store.get(&tx_id.0) {
        Ok(record) => Some(record),
        Err(z00z_wallets::persistence::TxStorageError::NotFound(_)) => None,
        Err(err) => {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation restart failed to load tx record {}: {err}",
                tx_id.0
            )))
        }
    };
    let history_rows = read_wallet_history_rows(output_dir, wallet_id)?;
    let restarted_lifecycle =
        wallet_lifecycle_from_record(record.as_ref(), &history_rows, &tx_id.0);
    let passed = reopened_snapshot.asset_ids == expected.asset_ids
        && reopened_snapshot.history_len == expected.history_len
        && reopened_snapshot.history_digest_hex == expected.history_digest_hex
        && restarted_lifecycle == expected_lifecycle;
    restart_wallet_service
        .lock_wallet(wallet_id)
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation restart lock failed for {}: {err}",
                wallet_id.0
            ))
        })?;

    Ok(WalletRestartCheck { passed })
}

async fn receiver_card_compact(
    wallet_service: &WalletService,
    wallet_id: &PersistWalletId,
) -> Result<String, Scenario1Err> {
    let recv_keys = wallet_service
        .receiver_keys(wallet_id)
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation receiver key load failed for {}: {err}",
                wallet_id.0
            ))
        })?;
    let card = recv_keys.export_receiver_card().map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation receiver card export failed for {}: {err}",
            wallet_id.0
        ))
    })?;
    ReceiverCardRecord::new(&card, card.canonical_encoding(), 0)
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation receiver card record failed for {}: {err}",
                wallet_id.0
            ))
        })?
        .to_compact()
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation receiver card compact encoding failed for {}: {err}",
                wallet_id.0
            ))
        })
}

async fn seed_spendable_stealth_coins(
    wallet_service: &WalletService,
    wallet_id: &PersistWalletId,
    amount: u64,
    serial_base: u32,
) -> Result<(), Scenario1Err> {
    for offset in 0..3u32 {
        let current_serial = serial_base.saturating_add(offset);
        let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(
            AssetClass::Coin,
            current_serial,
            amount,
        )
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation asset seed failed for serial {}: {err}",
                current_serial
            ))
        })?;
        let recv_keys = wallet_service
            .receiver_keys(wallet_id)
            .await
            .map_err(|err| {
                Scenario1Err::Evidence(format!(
                    "wallet lifecycle simulation receiver key load failed for {}: {err}",
                    wallet_id.0
                ))
            })?;
        let card = recv_keys.export_receiver_card().map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation receiver card export failed for {}: {err}",
                wallet_id.0
            ))
        })?;
        let tx_digest = [current_serial as u8; 32];
        let mut sender_wallet = SenderWallet::new([current_serial.saturating_add(41) as u8; 32]);
        let output = build_seeded_output_bundle(
            wallet_id.0.clone(),
            z00z_wallets::tx::TxOutRole::Recipient,
            AssetClass::Coin,
            &card,
            None,
            &mut sender_wallet,
            &tx_digest,
            asset.serial_id,
            [current_serial.saturating_add(41) as u8; 32],
            asset.amount,
            &asset.definition.id,
            asset.serial_id,
        )
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation stealth output build failed for serial {}: {err}",
                current_serial
            ))
        })?;

        asset.commitment = z00z_crypto::Commitment::from_bytes(&output.leaf.c_amount)
            .map_err(|err| {
                Scenario1Err::Evidence(format!(
                    "wallet lifecycle simulation commitment decode failed for serial {}: {err}",
                    current_serial
                ))
            })?
            .0;
        asset.owner_pub = None;
        asset.owner_signature = None;
        asset.r_pub = Some(output.leaf.r_pub);
        asset.owner_tag = Some(output.leaf.owner_tag);
        asset.enc_pack = Some(output.leaf.enc_pack);
        asset.tag16 = Some(output.leaf.tag16);
        asset.leaf_ad_id = Some(output.leaf.asset_id);
        asset.range_proof = Some(output.leaf.range_proof);

        wallet_service
            .put_claimed_asset(wallet_id, asset)
            .await
            .map_err(|err| {
                Scenario1Err::Evidence(format!(
                    "wallet lifecycle simulation seed asset insert failed for {}: {err}",
                    wallet_id.0
                ))
            })?;
    }

    Ok(())
}

async fn create_wallet_sim_env(
    output_root: &Path,
    seed_root: &str,
    case_id: &str,
    case_index: usize,
) -> Result<WalletSimEnv, Scenario1Err> {
    let mut path_hasher = Sha256::new();
    path_hasher.update(b"wallet-lifecycle-output");
    path_hasher.update(seed_root.as_bytes());
    path_hasher.update(case_id.as_bytes());
    let dir_suffix = hex::encode(&path_hasher.finalize()[..8]);
    let output_dir = output_root.join(dir_suffix);
    if output_dir.exists() {
        io::remove_dir_all(&output_dir)?;
    }
    io::create_dir_all(&output_dir)?;

    let time_provider = Arc::new(MockTimeProvider::from_unix_secs(
        WALLET_SIM_BASE_TIME_SECS + (case_index as u64).saturating_mul(100),
    ));
    let mut seed_hasher = Sha256::new();
    seed_hasher.update(b"wallet-lifecycle-entropy");
    seed_hasher.update(seed_root.as_bytes());
    seed_hasher.update(case_id.as_bytes());
    let wallet_service = Arc::new(WalletService::create_service_custom_output_directory(
        output_dir.clone(),
        time_provider.clone(),
        DeterministicSecureRngProvider::new(seed_hasher.finalize().into()),
    ));
    let app_service = Arc::new(AppService::with_dependencies(
        time_provider.clone(),
        Arc::clone(&wallet_service),
    ));
    let created = app_service
        .create_wallet(
            format!("wallet-{case_id}"),
            WALLET_SIM_PASSWORD.to_string(),
            Some(WALLET_SIM_SEED_PHRASE_24.to_string()),
        )
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation wallet create failed for {case_id}: {err}"
            ))
        })?;
    let session = wallet_service
        .unlock_wallet_in_memory(&created.wallet_id, &SafePassword::from(WALLET_SIM_PASSWORD))
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation wallet unlock failed for {case_id}: {err}"
            ))
        })?;

    Ok(WalletSimEnv {
        output_dir,
        time_provider: time_provider.clone(),
        app_service,
        wallet_service: Arc::clone(&wallet_service),
        asset_rpc: Arc::new(AssetRpcImpl::with_dependencies_and_wallet_service(
            time_provider.clone(),
            Arc::clone(&wallet_service),
        )),
        tx_rpc: Arc::new(TxRpcImpl::with_dependencies(
            Arc::clone(&wallet_service),
            time_provider,
        )),
        wallet_id: created.wallet_id,
        session,
    })
}

fn portable_tx_package_from_export(
    contents: &str,
) -> Result<PortableWalletTxPackage, Scenario1Err> {
    JsonCodec.deserialize(contents.as_bytes()).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to decode exported portable package: {err}"
        ))
    })
}

fn portable_tx_bytes_from_export(contents: &str) -> Result<String, Scenario1Err> {
    let portable = portable_tx_package_from_export(contents)?;
    String::from_utf8(portable.tx_bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation exported tx bytes were not valid utf8: {err}"
        ))
    })
}

fn portable_tx_input_ids(contents: &str) -> Result<Vec<[u8; 32]>, Scenario1Err> {
    let portable = portable_tx_package_from_export(contents)?;
    let package: TxPackage = JsonCodec.deserialize(&portable.tx_bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to decode tx package bytes: {err}"
        ))
    })?;
    package
        .tx
        .inputs
        .iter()
        .map(|input| {
            let bytes = hex::decode(&input.asset_id_hex).map_err(|err| {
                Scenario1Err::Evidence(format!(
                    "wallet lifecycle simulation failed to decode input asset id {}: {err}",
                    input.asset_id_hex
                ))
            })?;
            bytes.try_into().map_err(|_| {
                Scenario1Err::Evidence(format!(
                    "wallet lifecycle simulation input asset id has wrong shape: {}",
                    input.asset_id_hex
                ))
            })
        })
        .collect()
}

fn portable_tx_id(contents: &str) -> Result<PersistTxId, Scenario1Err> {
    let portable = portable_tx_package_from_export(contents)?;
    Ok(PersistTxId::new(format!("tx_{}", portable.tx_hash_hex)))
}

fn file_sha256_hex(path: &Path) -> Result<String, Scenario1Err> {
    let bytes = io::read_file(path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to hash {}: {err}",
            path.display()
        ))
    })?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

fn wallet_lifecycle_seed_root(
    wallet_scan_digest_hex: &str,
    publication_digest_hex: &str,
) -> String {
    let mut seed_hasher = Sha256::new();
    seed_hasher.update(b"wallet-lifecycle-seed-root");
    seed_hasher.update(wallet_scan_digest_hex.as_bytes());
    seed_hasher.update(publication_digest_hex.as_bytes());
    hex::encode(seed_hasher.finalize())
}

fn wallet_lifecycle_output_root(out_dir: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(b"wallet-lifecycle-output-root");
    hasher.update(out_dir.display().to_string().as_bytes());
    let dir_suffix = hex::encode(&hasher.finalize()[..8]);
    std::env::temp_dir()
        .join("z00z_wallet_lifecycle_sim")
        .join(dir_suffix)
}

fn assert_required_wallet_lifecycle_rows(
    rows: &[WalletLifecycleEvidenceView],
) -> Result<(), Scenario1Err> {
    let actual = rows
        .iter()
        .map(|row| row.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let expected = REQUIRED_WALLET_LIFECYCLE_CASES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(Scenario1Err::Evidence(format!(
            "wallet lifecycle evidence cases drifted: expected {:?}, got {:?}",
            expected, actual
        )));
    }
    if rows.len() != REQUIRED_WALLET_LIFECYCLE_CASES.len() {
        return Err(Scenario1Err::Evidence(format!(
            "wallet lifecycle evidence count drifted: expected {}, got {}",
            REQUIRED_WALLET_LIFECYCLE_CASES.len(),
            rows.len()
        )));
    }
    Ok(())
}

fn wallet_sim_timestamp_ms(case_index: usize) -> u64 {
    (WALLET_SIM_BASE_TIME_SECS + (case_index as u64).saturating_mul(100)).saturating_mul(1_000)
}

async fn build_pending_wallet_tx(
    env: &WalletSimEnv,
    case_index: usize,
) -> Result<PersistTxId, Scenario1Err> {
    seed_spendable_stealth_coins(
        &env.wallet_service,
        &env.wallet_id,
        50_000 + case_index as u64,
        1 + case_index as u32,
    )
    .await?;
    let recipient = receiver_card_compact(&env.wallet_service, &env.wallet_id).await?;
    env.tx_rpc
        .build_transaction(env.session.clone(), recipient, 10 + case_index as u64, None)
        .await
        .map(|response| response.tx_id)
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation build_transaction failed for case index {}: {}",
                case_index, err
            ))
        })
}

async fn send_and_export_self(
    env: &WalletSimEnv,
    case_index: usize,
) -> Result<(PersistTxId, String), Scenario1Err> {
    seed_spendable_stealth_coins(
        &env.wallet_service,
        &env.wallet_id,
        50_000 + case_index as u64,
        1 + case_index as u32,
    )
    .await?;
    let recipient = receiver_card_compact(&env.wallet_service, &env.wallet_id).await?;
    let tx_id = env
        .tx_rpc
        .send_transaction(
            env.session.clone(),
            recipient,
            10 + case_index as u64,
            None,
            None,
            None,
            Some(wallet_sim_timestamp_ms(case_index)),
        )
        .await
        .map(|response| response.tx_id)
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation send_transaction failed for case index {}: {}",
                case_index, err
            ))
        })?;
    let export_path = env
        .tx_rpc
        .export_transaction(env.session.clone(), tx_id.clone())
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation export_transaction failed for {}: {}",
                tx_id.0, err
            ))
        })?
        .export_path
        .ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation export path missing for {}",
                tx_id.0
            ))
        })?;
    let contents = io::read_to_string(&export_path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to read export {}: {err}",
            export_path
        ))
    })?;

    Ok((tx_id, contents))
}

async fn create_secondary_wallet(
    env: &WalletSimEnv,
    case_id: &str,
    seed_phrase: &str,
) -> Result<(PersistWalletId, SessionToken), Scenario1Err> {
    let created = env
        .app_service
        .create_wallet(
            format!("wallet-{case_id}-secondary"),
            WALLET_SIM_PASSWORD.to_string(),
            Some(seed_phrase.to_string()),
        )
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation secondary wallet create failed for {case_id}: {err}"
            ))
        })?;
    let session = env
        .wallet_service
        .unlock_wallet_in_memory(&created.wallet_id, &SafePassword::from(WALLET_SIM_PASSWORD))
        .await
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "wallet lifecycle simulation secondary wallet unlock failed for {case_id}: {err}"
            ))
        })?;
    Ok((created.wallet_id, session))
}

fn build_wallet_lifecycle_rows(
    out_dir: &Path,
    wallet_scan_digest_hex: &str,
    publication_digest_hex: &str,
) -> Result<Vec<WalletLifecycleEvidenceView>, Scenario1Err> {
    let _process_guard = crate::scenario_1::acquire_scenario_process_guard();
    let _env_restore = WalletConfigEnvRestore::acquire();
    let rt = tokio::runtime::Runtime::new().map_err(|err| {
        Scenario1Err::Evidence(format!(
            "wallet lifecycle simulation failed to create tokio runtime: {err}"
        ))
    })?;
    let wallet_scan_digest_hex = wallet_scan_digest_hex.to_string();
    let publication_digest_hex = publication_digest_hex.to_string();
    let seed_root = wallet_lifecycle_seed_root(&wallet_scan_digest_hex, &publication_digest_hex);
    let lifecycle_output_root = wallet_lifecycle_output_root(out_dir);

    let rows = rt.block_on(async move {
        let mut rows = Vec::new();

        for (case_index, case_id) in REQUIRED_WALLET_LIFECYCLE_CASES.iter().copied().enumerate() {
            let row = match case_id {
                "submitted" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let tx_id = build_pending_wallet_tx(&env, case_index).await?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let history_path = wallet_history_path(&env.output_dir, &env.wallet_id);
                    let mut store =
                        TxStorageImpl::new(&history_path, env.time_provider.as_ref().clone());
                    store.record_submitted(&tx_id.0).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation record_submitted failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let history_rows = read_wallet_history_rows(&env.output_dir, &env.wallet_id)?;
                    let record = store.get(&tx_id.0).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation load tx record failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let lifecycle =
                        wallet_lifecycle_from_record(Some(&record), &history_rows, &tx_id.0)
                            .ok_or_else(|| {
                                Scenario1Err::Evidence(format!(
                                    "wallet lifecycle simulation missing projected lifecycle for {}",
                                    tx_id.0
                                ))
                            })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, None),
                        error_code: None,
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "admitted" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let tx_id = build_pending_wallet_tx(&env, case_index).await?;
                    let history_path = wallet_history_path(&env.output_dir, &env.wallet_id);
                    let mut store =
                        TxStorageImpl::new(&history_path, env.time_provider.as_ref().clone());
                    store.record_submitted(&tx_id.0).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation record_submitted failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    store.record_admitted(&tx_id.0).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation record_admitted failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let history_rows = read_wallet_history_rows(&env.output_dir, &env.wallet_id)?;
                    let record = store.get(&tx_id.0).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation load tx record failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let lifecycle =
                        wallet_lifecycle_from_record(Some(&record), &history_rows, &tx_id.0)
                            .ok_or_else(|| {
                                Scenario1Err::Evidence(format!(
                                    "wallet lifecycle simulation missing projected lifecycle for {}",
                                    tx_id.0
                                ))
                            })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, None),
                        error_code: None,
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "imported" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let imported = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), contents)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation import_transaction failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    if imported.imported_outputs.is_empty() {
                        return Err(Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation imported case produced no owned outputs for {}",
                            tx_id.0
                        )));
                    }
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(imported.lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: imported.tx_id.0,
                        lifecycle: imported.lifecycle,
                        coarse_status: wallet_status_label(imported.lifecycle, None),
                        error_code: None,
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "confirmed" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    env.tx_rpc
                        .import_transaction(env.session.clone(), contents)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation import_transaction failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let reconciled = env
                        .tx_rpc
                        .reconcile_transaction(env.session.clone(), tx_id.clone())
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation reconcile_transaction failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    if !reconciled.confirmation.verified {
                        return Err(Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation reconcile confirmation was not verified for {}",
                            tx_id.0
                        )));
                    }
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(reconciled.lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: reconciled.tx_id.0,
                        lifecycle: reconciled.lifecycle,
                        coarse_status: wallet_status_label(reconciled.lifecycle, None),
                        error_code: None,
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "duplicate_import" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let first = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), contents.clone())
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation first import failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    let first_output_ids = first
                        .imported_outputs
                        .iter()
                        .map(|output| output.asset_id_hex.clone())
                        .collect::<BTreeSet<_>>();
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let second = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), contents)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation duplicate import failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    let second_output_ids = second
                        .imported_outputs
                        .iter()
                        .map(|output| output.asset_id_hex.clone())
                        .collect::<BTreeSet<_>>();
                    if first_output_ids != second_output_ids {
                        return Err(Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation duplicate import outputs drifted for {}",
                            tx_id.0
                        )));
                    }
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(second.lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: second.tx_id.0,
                        lifecycle: second.lifecycle,
                        coarse_status: wallet_status_label(second.lifecycle, None),
                        error_code: None,
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "conflicted" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let verify = env
                        .tx_rpc
                        .verify_transaction_package(
                            env.session.clone(),
                            portable_tx_bytes_from_export(&contents)?,
                        )
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation verify_transaction_package failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    let owned_output = verify.owned_outputs.first().ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation verify_transaction_package produced no owned outputs for {}",
                            tx_id.0
                        ))
                    })?;
                    let imported = env
                        .asset_rpc
                        .import_asset(env.session.clone(), owned_output.asset_data.clone())
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation import_asset failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    if !imported.is_inserted {
                        return Err(Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation import_asset did not insert conflict seed for {}",
                            tx_id.0
                        )));
                    }
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let err = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), contents)
                        .await
                        .unwrap_err();
                    let payload = decode_runtime_tx_error_payload!(err);
                    let error_code =
                        payload.error_codes.first().copied().ok_or_else(|| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation conflicted case missing error code for {}",
                                tx_id.0
                            ))
                        })?;
                    let lifecycle = payload.lifecycle.ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation conflicted case missing lifecycle for {}",
                            tx_id.0
                        ))
                    })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, Some(error_code)),
                        error_code: Some(error_code),
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "already_spent" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let input_ids = portable_tx_input_ids(&contents)?;
                    let input_id = *input_ids.first().ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation already-spent case missing tx input for {}",
                            tx_id.0
                        ))
                    })?;
                    env.wallet_service
                        .release_claimed_asset_reservation(&env.wallet_id, &tx_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation release reservation failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    env.wallet_service
                        .reserve_claimed_asset_inputs(
                            &env.wallet_id,
                            &PersistTxId::new("tx_foreign_already_spent".to_string()),
                            &[input_id],
                        )
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation reserve conflicting input failed for {}: {}",
                                tx_id.0, err
                            ))
                        })?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let err = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), contents)
                        .await
                        .unwrap_err();
                    let payload = decode_runtime_tx_error_payload!(err);
                    let error_code =
                        payload.error_codes.first().copied().ok_or_else(|| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation already-spent case missing error code for {}",
                                tx_id.0
                            ))
                        })?;
                    let lifecycle = payload.lifecycle.ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation already-spent case missing lifecycle for {}",
                            tx_id.0
                        ))
                    })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(lifecycle),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, Some(error_code)),
                        error_code: Some(error_code),
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "no_owned_output" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (_target_wallet_id, target_session) =
                        create_secondary_wallet(&env, case_id, WALLET_SIM_ALT_SEED_24)
                            .await?;
                    let target_wallet_id = target_session.wallet_id.clone();
                    let (tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &target_wallet_id)
                            .await?;
                    let err = env
                        .tx_rpc
                        .import_transaction(target_session, contents)
                        .await
                        .unwrap_err();
                    let payload = decode_runtime_tx_error_payload!(err);
                    let error_code =
                        payload.error_codes.first().copied().ok_or_else(|| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation no-owned-output case missing error code for {}",
                                tx_id.0
                            ))
                        })?;
                    let lifecycle = payload.lifecycle.ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation no-owned-output case missing lifecycle for {}",
                            tx_id.0
                        ))
                    })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &target_wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&target_wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                target_wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &target_wallet_id,
                        &after,
                        &tx_id,
                        None,
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, Some(error_code)),
                        error_code: Some(error_code),
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "wrong_chain" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (_tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let tx_id = portable_tx_id(&contents)?;
                    let mut portable = portable_tx_package_from_export(&contents)?;
                    portable.chain_id = "999".to_string();
                    let package_version_bytes = portable.package_version.to_le_bytes();
                    portable.metadata_hash_hex = hex::encode(z00z_crypto::blake2b_hash(
                        b"z00z.wallet.portable.metadata.v1",
                        &[
                            &package_version_bytes,
                            portable.chain_id.as_bytes(),
                            portable.tx_hash_hex.as_bytes(),
                        ],
                    ));
                    let tampered = String::from_utf8(JsonCodec.serialize(&portable).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation wrong-chain package encode failed for {}: {err}",
                            tx_id.0
                        ))
                    })?)
                    .map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation wrong-chain package utf8 failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let err = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), tampered)
                        .await
                        .unwrap_err();
                    let payload = decode_runtime_tx_error_payload!(err);
                    let error_code =
                        payload.error_codes.first().copied().ok_or_else(|| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation wrong-chain case missing error code for {}",
                                tx_id.0
                            ))
                        })?;
                    let lifecycle = payload.lifecycle.ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation wrong-chain case missing lifecycle for {}",
                            tx_id.0
                        ))
                    })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(RuntimeTxLifecycle::Exported),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, Some(error_code)),
                        error_code: Some(error_code),
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "invalid_digest" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (_tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let tx_id = portable_tx_id(&contents)?;
                    let mut portable = portable_tx_package_from_export(&contents)?;
                    portable.metadata_hash_hex = hex::encode([1u8; 32]);
                    let tampered = String::from_utf8(JsonCodec.serialize(&portable).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation invalid-digest package encode failed for {}: {err}",
                            tx_id.0
                        ))
                    })?)
                    .map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation invalid-digest package utf8 failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let err = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), tampered)
                        .await
                        .unwrap_err();
                    let payload = decode_runtime_tx_error_payload!(err);
                    let error_code =
                        payload.error_codes.first().copied().ok_or_else(|| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation invalid-digest case missing error code for {}",
                                tx_id.0
                            ))
                        })?;
                    let lifecycle = payload.lifecycle.ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation invalid-digest case missing lifecycle for {}",
                            tx_id.0
                        ))
                    })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(RuntimeTxLifecycle::Exported),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, Some(error_code)),
                        error_code: Some(error_code),
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                "unsupported_package_version" => {
                    let env = create_wallet_sim_env(
                        &lifecycle_output_root,
                        &seed_root,
                        case_id,
                        case_index,
                    )
                    .await?;
                    let (_tx_id, contents) = send_and_export_self(&env, case_index).await?;
                    let tx_id = portable_tx_id(&contents)?;
                    let mut portable = portable_tx_package_from_export(&contents)?;
                    portable.package_version = 2;
                    let package_version_bytes = portable.package_version.to_le_bytes();
                    portable.metadata_hash_hex = hex::encode(z00z_crypto::blake2b_hash(
                        b"z00z.wallet.portable.metadata.v1",
                        &[
                            &package_version_bytes,
                            portable.chain_id.as_bytes(),
                            portable.tx_hash_hex.as_bytes(),
                        ],
                    ));
                    let tampered = String::from_utf8(JsonCodec.serialize(&portable).map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation wrong-version package encode failed for {}: {err}",
                            tx_id.0
                        ))
                    })?)
                    .map_err(|err| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation wrong-version package utf8 failed for {}: {err}",
                            tx_id.0
                        ))
                    })?;
                    let before =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    let err = env
                        .tx_rpc
                        .import_transaction(env.session.clone(), tampered)
                        .await
                        .unwrap_err();
                    let payload = decode_runtime_tx_error_payload!(err);
                    let error_code =
                        payload.error_codes.first().copied().ok_or_else(|| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation wrong-version case missing error code for {}",
                                tx_id.0
                            ))
                        })?;
                    let lifecycle = payload.lifecycle.ok_or_else(|| {
                        Scenario1Err::Evidence(format!(
                            "wallet lifecycle simulation wrong-version case missing lifecycle for {}",
                            tx_id.0
                        ))
                    })?;
                    let after =
                        wallet_snapshot(&env.wallet_service, &env.output_dir, &env.wallet_id)
                            .await?;
                    env.wallet_service
                        .lock_wallet(&env.wallet_id)
                        .await
                        .map_err(|err| {
                            Scenario1Err::Evidence(format!(
                                "wallet lifecycle simulation lock failed for {}: {err}",
                                env.wallet_id.0
                            ))
                        })?;
                    let restart = restart_wallet_state_matches(
                        &env.output_dir,
                        env.time_provider.clone(),
                        &env.wallet_id,
                        &after,
                        &tx_id,
                        Some(RuntimeTxLifecycle::Exported),
                    )
                    .await?;

                    WalletLifecycleEvidenceView {
                        case_id: case_id.to_string(),
                        tx_id: tx_id.0,
                        lifecycle,
                        coarse_status: wallet_status_label(lifecycle, Some(error_code)),
                        error_code: Some(error_code),
                        wallet_asset_rows_changed: after.asset_ids != before.asset_ids,
                        tx_history_row_count_changed: after.history_len != before.history_len,
                        restart_verification_passed: restart.passed,
                        wallet_scan_digest_hex: wallet_scan_digest_hex.clone(),
                        tx_history_digest_hex: after.history_digest_hex,
                        publication_digest_hex: publication_digest_hex.clone(),
                    }
                }
                other => {
                    return Err(Scenario1Err::Evidence(format!(
                        "unsupported wallet lifecycle simulation case {other}"
                    )))
                }
            };
            rows.push(row);
        }

        Ok::<Vec<WalletLifecycleEvidenceView>, Scenario1Err>(rows)
    })?;

    assert_required_wallet_lifecycle_rows(&rows)?;
    Ok(rows)
}

fn load_tx_output_bindings(
    out_dir: &Path,
) -> Result<BTreeMap<String, TxOutputBindingMeta>, Scenario1Err> {
    let loaded = load_post_tx_candidate_set(out_dir).map_err(|err| {
        Scenario1Err::Evidence(format!("scope_flow committed candidate load failed: {err}"))
    })?;
    loaded
        .candidates
        .into_iter()
        .map(|candidate| {
            Ok((
                hex::encode(candidate.path.terminal_id().as_bytes()),
                TxOutputBindingMeta {
                    definition_id: hex::encode(candidate.path.definition_id.into_bytes()),
                    serial_id: candidate.path.serial_id.get(),
                },
            ))
        })
        .collect()
}

fn wallet_pending_path(cfg: &ScenarioCfg, out_dir: &Path) -> Result<PathBuf, Scenario1Err> {
    let stage4 = stage4_cfg(cfg)?;
    Ok(resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(&stage4.paths.transactions_dir)
            .join("wallets_pending.json")
            .to_string_lossy(),
    ))
}

fn wallet_confirmed_path(cfg: &ScenarioCfg, out_dir: &Path) -> Result<PathBuf, Scenario1Err> {
    let stage4 = stage4_cfg(cfg)?;
    Ok(resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(&stage4.paths.transactions_dir)
            .join("wallets_confirmed.json")
            .to_string_lossy(),
    ))
}

fn wallet_diff_path(cfg: &ScenarioCfg, out_dir: &Path) -> Result<PathBuf, Scenario1Err> {
    let stage4 = stage4_cfg(cfg)?;
    let rel = stage4
        .paths
        .wallets_state_diff_file
        .clone()
        .unwrap_or_else(|| {
            PathBuf::from(&stage4.paths.transactions_dir)
                .join("wallets_state_diff.json")
                .to_string_lossy()
                .to_string()
        });
    Ok(resolve_runtime_output_path(out_dir, &rel))
}

fn build_hist_flow(
    common: FlowCommon,
    cfg: &ScenarioCfg,
    out_dir: &Path,
    trace_files: &TraceFileView,
) -> Result<HistFlow, Scenario1Err> {
    let observability = observability_cfg(cfg)?;
    let wallet_scan_path = resolve_trace_file(out_dir, &observability.packet.wallet_scan_file)?;
    let wallet_scan_digest_hex = file_sha256_hex(&wallet_scan_path)?;
    let sources = load_imported_flow_sources(out_dir, trace_files)?;
    let wallet_lifecycle_rows = build_wallet_lifecycle_rows(
        out_dir,
        &wallet_scan_digest_hex,
        &sources.pub_flow.publication_digest_hex,
    )?;
    build_hist_flow_with_rows(
        common,
        cfg,
        trace_files,
        &sources,
        wallet_scan_digest_hex,
        wallet_lifecycle_rows,
    )
}

fn build_hist_flow_cached(
    common: FlowCommon,
    cfg: &ScenarioCfg,
    out_dir: &Path,
    trace_files: &TraceFileView,
    actual_hist_flow: &Value,
) -> Result<HistFlow, Scenario1Err> {
    let observability = observability_cfg(cfg)?;
    let wallet_scan_path = resolve_trace_file(out_dir, &observability.packet.wallet_scan_file)?;
    let wallet_scan_digest_hex = file_sha256_hex(&wallet_scan_path)?;
    let sources = load_imported_flow_sources(out_dir, trace_files)?;
    let wallet_lifecycle_rows = decode_wallet_rows(actual_hist_flow)?;
    build_hist_flow_with_rows(
        common,
        cfg,
        trace_files,
        &sources,
        wallet_scan_digest_hex,
        wallet_lifecycle_rows,
    )
}

fn build_hist_flow_with_rows(
    common: FlowCommon,
    cfg: &ScenarioCfg,
    trace_files: &TraceFileView,
    sources: &ImportedFlowSources,
    wallet_scan_digest_hex: String,
    wallet_lifecycle_rows: Vec<WalletLifecycleEvidenceView>,
) -> Result<HistFlow, Scenario1Err> {
    let observability = observability_cfg(cfg)?;
    validate_wallet_rows(
        &wallet_lifecycle_rows,
        &wallet_scan_digest_hex,
        &sources.pub_flow.publication_digest_hex,
    )?;
    let mut imported_artifact_verdicts = build_imported_artifact_verdicts(sources)?;
    imported_artifact_verdicts.push(ImportedArtifactVerdictView {
        verdict_id: "wallet_scan_digest_binding",
        status: "verified",
        detail: wallet_scan_digest_hex.clone(),
    });
    imported_artifact_verdicts.push(ImportedArtifactVerdictView {
        verdict_id: "wallet_lifecycle_matrix",
        status: "verified",
        detail: format!("{} lifecycle cases", wallet_lifecycle_rows.len()),
    });
    imported_artifact_verdicts.push(ImportedArtifactVerdictView {
        verdict_id: "wallet_restart_projection",
        status: "verified",
        detail: format!(
            "{} restart checks matched",
            wallet_lifecycle_rows
                .iter()
                .filter(|row| row.restart_verification_passed)
                .count()
        ),
    });
    let example_map = sources
        .examples
        .examples
        .iter()
        .map(|row| (row.example_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let historical_proof_verdicts = sources
        .replay
        .replay_entries
        .iter()
        .filter_map(|replay| {
            let example = example_map.get(replay.example_id.as_str())?;
            if matches!(
                example.proof_family.as_str(),
                "split" | "policy_transition" | "metrics"
            ) {
                return None;
            }
            Some(HistoricalProofVerdictView {
                example_id: replay.example_id.clone(),
                proof_family: example.proof_family.clone(),
                leaf_family: example.leaf_family.clone(),
                verifier_status: replay.verifier_status.clone(),
                settlement_path: example.settlement_path.clone(),
                settlement_state_root_hex: replay.settlement_state_root_hex.clone(),
                reloaded_settlement_state_root_hex: replay
                    .reloaded_settlement_state_root_hex
                    .clone(),
                proof_is_ownership: example.proof_is_ownership.unwrap_or(false),
            })
        })
        .collect::<Vec<_>>();
    if historical_proof_verdicts.is_empty() {
        return Err(Scenario1Err::Evidence(
            "hist_flow missing historical proof verdict rows".to_string(),
        ));
    }
    let live_reject_rows = sources
        .tamper
        .cases
        .iter()
        .filter_map(|case| {
            let example = example_map.get(case.example_id.as_str())?;
            if matches!(
                example.proof_family.as_str(),
                "split" | "policy_transition" | "metrics"
            ) {
                return None;
            }
            Some(LiveRejectVerdictView {
                example_id: case.example_id.clone(),
                case_id: case.case_id.clone(),
                proof_surface: case.proof_surface.clone(),
                verifier_status: case.verifier_status.clone(),
                typed_error_class: case.typed_error.class.clone(),
            })
        })
        .collect::<Vec<_>>();
    if live_reject_rows.is_empty() {
        return Err(Scenario1Err::Evidence(
            "hist_flow missing live reject rows".to_string(),
        ));
    }

    Ok(HistFlow {
        common,
        route_migration_rows: build_route_migration_rows(&sources.pub_flow, &sources.val),
        source_artifacts: vec![
            observability.packet.wallet_scan_file.clone(),
            trace_files.leaf_flow_file.clone(),
            trace_files.proof_flow_file.clone(),
            trace_files.pub_flow_file.clone(),
            trace_files.val_flow_file.clone(),
            trace_files.watch_flow_file.clone(),
            "hjmt/hjmt_replay_roots.json".to_string(),
            "hjmt/hjmt_tamper_report.json".to_string(),
        ],
        wallet_scan_digest_hex,
        wallet_lifecycle_rows,
        historical_proof_verdicts,
        live_reject_rows,
        owner_reject_rows: vec![
            OwnerRejectHomeView {
                contract_id: "route_generation_drift_reject",
                owner_home: HISTORICAL_OWNER_HOME,
                proof_point: "test_historical_public_proof_stays_valid_after_later_publication",
                status: "linked_owner_contract",
            },
            OwnerRejectHomeView {
                contract_id: "cross_shard_lineage_reject",
                owner_home: HISTORICAL_OWNER_HOME,
                proof_point: "test_cross_shard_counterexample_rejects",
                status: "linked_owner_contract",
            },
            OwnerRejectHomeView {
                contract_id: "current_policy_reinterpretation_reject",
                owner_home: ADAPTIVE_POLICY_OWNER_HOME,
                proof_point: "test_rejects_stale_drifted_policy",
                status: "linked_owner_contract",
            },
        ],
        imported_artifact_verdicts,
    })
}

fn decode_wallet_rows(
    actual_hist_flow: &Value,
) -> Result<Vec<WalletLifecycleEvidenceView>, Scenario1Err> {
    let rows = actual_hist_flow
        .get("wallet_lifecycle_rows")
        .ok_or_else(|| {
            Scenario1Err::Evidence("hist_flow missing wallet_lifecycle_rows".to_string())
        })?;
    let bytes = JsonCodec.serialize(rows).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to encode hist_flow wallet_lifecycle_rows: {err}"
        ))
    })?;
    JsonCodec.deserialize(&bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to decode hist_flow wallet_lifecycle_rows: {err}"
        ))
    })
}

fn validate_wallet_rows(
    rows: &[WalletLifecycleEvidenceView],
    wallet_scan_digest_hex: &str,
    publication_digest_hex: &str,
) -> Result<(), Scenario1Err> {
    assert_required_wallet_lifecycle_rows(rows)?;
    for row in rows {
        if row.tx_id.trim().is_empty() {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle evidence tx_id missing for case {}",
                row.case_id
            )));
        }
        if row.tx_history_digest_hex.trim().is_empty() {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle evidence tx_history_digest_hex missing for case {}",
                row.case_id
            )));
        }
        if !row.restart_verification_passed {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle restart verification failed for case {}",
                row.case_id
            )));
        }
        if row.wallet_scan_digest_hex != wallet_scan_digest_hex {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle wallet_scan digest drifted for case {}: expected {}, got {}",
                row.case_id, wallet_scan_digest_hex, row.wallet_scan_digest_hex
            )));
        }
        if row.publication_digest_hex != publication_digest_hex {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle publication digest drifted for case {}: expected {}, got {}",
                row.case_id, publication_digest_hex, row.publication_digest_hex
            )));
        }
        let expected_status = wallet_status_label(row.lifecycle, row.error_code);
        if row.coarse_status != expected_status {
            return Err(Scenario1Err::Evidence(format!(
                "wallet lifecycle coarse_status drifted for case {}: expected {}, got {}",
                row.case_id, expected_status, row.coarse_status
            )));
        }
    }
    Ok(())
}

fn build_occ_flow(
    common: FlowCommon,
    _cfg: &ScenarioCfg,
    out_dir: &Path,
    trace_files: &TraceFileView,
) -> Result<OccFlow, Scenario1Err> {
    let sources = load_imported_flow_sources(out_dir, trace_files)?;
    let imported_artifact_verdicts = build_imported_artifact_verdicts(&sources)?;
    let example_map = sources
        .examples
        .examples
        .iter()
        .map(|row| (row.example_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let occupancy_disclosure_verdicts = sources
        .examples
        .examples
        .iter()
        .filter(|row| matches!(row.proof_family.as_str(), "split" | "policy_transition"))
        .map(|row| OccupancyDisclosureVerdictView {
            example_id: row.example_id.clone(),
            proof_family: row.proof_family.clone(),
            verifier_status: row.verifier_status.clone(),
            settlement_path: row.settlement_path.clone(),
            prior_state_root_hex: row.prior_state_root_hex.clone(),
            next_state_root_hex: row.next_state_root_hex.clone(),
            disclosure_guard: "coarse_only",
            binding_hex: row
                .transition_binding
                .clone()
                .or_else(|| row.bucket_policy_id.clone()),
            proof_is_ownership: row.proof_is_ownership.unwrap_or(false),
        })
        .collect::<Vec<_>>();
    if occupancy_disclosure_verdicts.is_empty() {
        return Err(Scenario1Err::Evidence(
            "occ_flow missing occupancy disclosure verdict rows".to_string(),
        ));
    }
    let live_reject_rows = sources
        .tamper
        .cases
        .iter()
        .filter_map(|case| {
            let example = example_map.get(case.example_id.as_str())?;
            if !matches!(example.proof_family.as_str(), "split" | "policy_transition") {
                return None;
            }
            Some(LiveRejectVerdictView {
                example_id: case.example_id.clone(),
                case_id: case.case_id.clone(),
                proof_surface: case.proof_surface.clone(),
                verifier_status: case.verifier_status.clone(),
                typed_error_class: case.typed_error.class.clone(),
            })
        })
        .collect::<Vec<_>>();
    if live_reject_rows.is_empty() {
        return Err(Scenario1Err::Evidence(
            "occ_flow missing live reject rows".to_string(),
        ));
    }

    Ok(OccFlow {
        common,
        route_migration_rows: build_route_migration_rows(&sources.pub_flow, &sources.val),
        source_artifacts: vec![
            trace_files.leaf_flow_file.clone(),
            trace_files.proof_flow_file.clone(),
            trace_files.pub_flow_file.clone(),
            trace_files.val_flow_file.clone(),
            trace_files.watch_flow_file.clone(),
            "hjmt/hjmt_settlement_examples.json".to_string(),
            "hjmt/hjmt_tamper_report.json".to_string(),
        ],
        occupancy_disclosure_verdicts,
        live_reject_rows,
        privacy_owner_contract_rows: vec![
            PrivacyOwnerContractView {
                contract_id: "metric_redaction",
                owner_home: OCCUPANCY_PRIVACY_OWNER_HOME,
                proof_point: "test_metric_stays_private",
                disclosure_guard: "coarse_only",
                status: "linked_owner_contract",
            },
            PrivacyOwnerContractView {
                contract_id: "noise_blind_split",
                owner_home: OCCUPANCY_PRIVACY_OWNER_HOME,
                proof_point: "test_split_noise_blind",
                disclosure_guard: "coarse_only",
                status: "linked_owner_contract",
            },
            PrivacyOwnerContractView {
                contract_id: "class_bound_split",
                owner_home: OCCUPANCY_PRIVACY_OWNER_HOME,
                proof_point: "test_split_class_bound",
                disclosure_guard: "coarse_only",
                status: "linked_owner_contract",
            },
            PrivacyOwnerContractView {
                contract_id: "split_evidence_tamper_reject",
                owner_home: OCCUPANCY_EVIDENCE_OWNER_HOME,
                proof_point: "test_split_tamper",
                disclosure_guard: "bind_only",
                status: "linked_owner_contract",
            },
            PrivacyOwnerContractView {
                contract_id: "merge_evidence_tamper_reject",
                owner_home: OCCUPANCY_EVIDENCE_OWNER_HOME,
                proof_point: "test_merge_tamper",
                disclosure_guard: "bind_only",
                status: "linked_owner_contract",
            },
            PrivacyOwnerContractView {
                contract_id: "transition_evidence_tamper_reject",
                owner_home: OCCUPANCY_EVIDENCE_OWNER_HOME,
                proof_point: "test_transition_tamper",
                disclosure_guard: "bind_only",
                status: "linked_owner_contract",
            },
        ],
        imported_artifact_verdicts,
    })
}

fn object_flow_case_view(case: &crate::config::ObjectFlowCaseCfg) -> ObjectFlowCaseView {
    ObjectFlowCaseView {
        id: case.id.clone(),
        family: case.family.clone(),
        action: case.action.clone(),
        policy_label: case.policy_label.clone(),
        lane: case.lane.clone(),
        actors: case.actors.clone(),
        required_rights: case.required_rights.clone(),
        expected_verdict: case.expected_verdict.clone(),
        evidence_files: case.evidence_files.clone(),
    }
}

fn build_object_flow(
    common: FlowCommon,
    cfg: &ScenarioCfg,
    anchor_file: &str,
) -> Result<ObjectFlow, Scenario1Err> {
    let matrix = cfg.object_flow_matrix.as_ref().ok_or_else(|| {
        Scenario1Err::Evidence("object_flow_matrix missing from scenario config".to_string())
    })?;
    let positive_rows = matrix
        .positive
        .iter()
        .filter(|case| case.evidence_files.iter().any(|file| file == anchor_file))
        .map(object_flow_case_view)
        .collect::<Vec<_>>();
    let negative_rows = matrix
        .negative
        .iter()
        .filter(|case| case.evidence_files.iter().any(|file| file == anchor_file))
        .map(object_flow_case_view)
        .collect::<Vec<_>>();
    if positive_rows.is_empty() && negative_rows.is_empty() {
        return Err(Scenario1Err::Evidence(format!(
            "{anchor_file} has no anchored object-flow cases"
        )));
    }

    let mut covered_families = positive_rows
        .iter()
        .chain(negative_rows.iter())
        .map(|case| case.family.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    covered_families.sort();

    let mut covered_lanes = positive_rows
        .iter()
        .chain(negative_rows.iter())
        .map(|case| case.lane.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    covered_lanes.sort();

    let mut source_artifacts = positive_rows
        .iter()
        .chain(negative_rows.iter())
        .flat_map(|case| case.evidence_files.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    source_artifacts.sort();

    Ok(ObjectFlow {
        common,
        packet_anchor_file: anchor_file.to_string(),
        positive_count: positive_rows.len(),
        negative_count: negative_rows.len(),
        covered_families,
        covered_lanes,
        source_artifacts,
        positive_rows,
        negative_rows,
    })
}

fn load_imported_flow_sources(
    out_dir: &Path,
    trace_files: &TraceFileView,
) -> Result<ImportedFlowSources, Scenario1Err> {
    Ok(ImportedFlowSources {
        leaf: load_json_file(
            &resolve_trace_file(out_dir, &trace_files.leaf_flow_file)?,
            "leaf_flow",
        )?,
        proof: load_json_file(
            &resolve_trace_file(out_dir, &trace_files.proof_flow_file)?,
            "proof_flow",
        )?,
        pub_flow: load_json_file(
            &resolve_trace_file(out_dir, &trace_files.pub_flow_file)?,
            "pub_flow",
        )?,
        val: load_json_file(
            &resolve_trace_file(out_dir, &trace_files.val_flow_file)?,
            "val_flow",
        )?,
        watch: load_json_file(
            &resolve_trace_file(out_dir, &trace_files.watch_flow_file)?,
            "watch_flow",
        )?,
        examples: load_json_file(
            &resolve_path(out_dir, "hjmt/hjmt_settlement_examples.json"),
            "stage13 examples",
        )?,
        replay: load_json_file(
            &resolve_path(out_dir, "hjmt/hjmt_replay_roots.json"),
            "stage13 replay roots",
        )?,
        tamper: load_json_file(
            &resolve_path(out_dir, "hjmt/hjmt_tamper_report.json"),
            "stage13 tamper report",
        )?,
    })
}

fn build_imported_artifact_verdicts(
    sources: &ImportedFlowSources,
) -> Result<Vec<ImportedArtifactVerdictView>, Scenario1Err> {
    let digest_set = BTreeSet::from([
        sources.leaf.publication_digest_hex.as_str(),
        sources.proof.publication_digest_hex.as_str(),
        sources.pub_flow.publication_digest_hex.as_str(),
        sources.val.publication_digest_hex.as_str(),
        sources.watch.publication_digest_hex.as_str(),
    ]);
    if digest_set.len() != 1 {
        return Err(Scenario1Err::Evidence(
            "historical/occupancy imported publication digests drifted across trace files"
                .to_string(),
        ));
    }

    let leaf_paths = sources
        .leaf
        .leaf_rows
        .iter()
        .map(|row| row.source_settlement_path.as_str())
        .collect::<BTreeSet<_>>();
    let missing_proof_paths = sources
        .proof
        .proof_rows
        .iter()
        .filter(|row| !leaf_paths.contains(row.source_settlement_path.as_str()))
        .map(|row| row.source_settlement_path.clone())
        .collect::<Vec<_>>();
    if !missing_proof_paths.is_empty() {
        return Err(Scenario1Err::Evidence(format!(
            "historical/occupancy proof paths missing from leaf_flow: {}",
            missing_proof_paths.join(", ")
        )));
    }

    let bad_public_roots = sources
        .proof
        .proof_rows
        .iter()
        .filter(|row| row.public_root_hex != sources.pub_flow.public_root_hex)
        .map(|row| row.source_settlement_path.clone())
        .collect::<Vec<_>>();
    if !bad_public_roots.is_empty() {
        return Err(Scenario1Err::Evidence(format!(
            "historical/occupancy proof public_root drifted for: {}",
            bad_public_roots.join(", ")
        )));
    }

    if sources.pub_flow.route_generation != sources.val.route_generation {
        return Err(Scenario1Err::Evidence(format!(
            "historical/occupancy route generation drifted between pub_flow ({}) and val_flow ({})",
            sources.pub_flow.route_generation, sources.val.route_generation
        )));
    }

    Ok(vec![
        ImportedArtifactVerdictView {
            verdict_id: "publication_digest_alignment",
            status: "verified",
            detail: sources.pub_flow.publication_digest_hex.clone(),
        },
        ImportedArtifactVerdictView {
            verdict_id: "proof_path_alignment",
            status: "verified",
            detail: format!(
                "{} proof paths bound to leaf_flow",
                sources.proof.proof_rows.len()
            ),
        },
        ImportedArtifactVerdictView {
            verdict_id: "public_root_alignment",
            status: "verified",
            detail: sources.pub_flow.public_root_hex.clone(),
        },
        ImportedArtifactVerdictView {
            verdict_id: "route_generation_alignment",
            status: "verified",
            detail: sources.pub_flow.route_generation.to_string(),
        },
    ])
}

fn build_route_migration_rows(
    pub_flow: &PubFlowSource,
    val_flow: &ValFlowSource,
) -> Vec<RouteMigrationView> {
    pub_flow
        .topology_examples
        .iter()
        .map(|row| RouteMigrationView {
            fixture_id: row.fixture_id.clone(),
            old_topology: row.old_topology.clone(),
            new_topology: row.new_topology.clone(),
            old_route_generation: row.route_generation_from,
            new_route_generation: row.route_generation_to,
            old_public_root_hex: pub_flow.prior_public_root_hex.clone(),
            new_public_root_hex: pub_flow.public_root_hex.clone(),
            old_settlement_root_hex: val_flow.prev_settlement_root_hex.clone(),
            new_settlement_root_hex: val_flow.new_settlement_root_hex.clone(),
            activation_checkpoint: row.activation_checkpoint,
        })
        .collect()
}

fn emitted_public_file<'a>(
    packet: &'a crate::config::RuntimePacketCfg,
    required: &str,
) -> Result<&'a str, Scenario1Err> {
    packet
        .emitted_public_files
        .iter()
        .find(|file| file.as_str() == required)
        .map(String::as_str)
        .ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "runtime_observability emitted_public_files missing {}",
                required
            ))
        })
}

fn load_json_file<T: DeserializeOwned>(path: &Path, label: &str) -> Result<T, Scenario1Err> {
    let bytes = io::read_file(path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to read {} {}: {err}",
            label,
            path.display()
        ))
    })?;
    JsonCodec.deserialize(&bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to decode {} {}: {err}",
            label,
            path.display()
        ))
    })
}

fn build_artifact_inventory(
    spec: &RuntimeTraceSpec,
    packet: &crate::config::RuntimePacketCfg,
) -> Vec<ArtifactInventoryRow> {
    let mut rows = vec![
        ArtifactInventoryRow {
            file: packet.run_meta_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.cfg_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.tx_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.route_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.plan_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.journal_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.scope_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.proc_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.recovery_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.leaf_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.proof_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.pub_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.val_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: spec.traces.watch_flow_file.clone(),
            status: "emitted",
        },
        ArtifactInventoryRow {
            file: packet.wallet_scan_file.clone(),
            status: "emitted",
        },
    ];
    rows.extend(
        packet
            .emitted_public_files
            .iter()
            .cloned()
            .map(|file| ArtifactInventoryRow {
                file,
                status: "emitted",
            }),
    );
    rows.push(ArtifactInventoryRow {
        file: packet.sim_summary_file.clone(),
        status: "emitted",
    });
    rows
}

fn build_wallet_scan_summary(wallet_scan: &JmtScanArtifact) -> WalletScanSummaryView {
    WalletScanSummaryView {
        actor: wallet_scan.actor.clone(),
        store_root_hex: wallet_scan.store_root_hex.clone(),
        detected_count: wallet_scan.detected_count,
        total_detected_amount: wallet_scan.total_detected_amount,
        proof_validated_count: wallet_scan.proof_validated_count,
        skipped_non_asset_count: wallet_scan.skipped_non_asset_count,
    }
}

fn build_run_meta(
    spec: &RuntimeTraceSpec,
    packet: &crate::config::RuntimePacketCfg,
    trace_files: &TraceFileView,
    stage_results: &[StageResultView],
    wallet_scan: &JmtScanArtifact,
) -> RunMeta {
    RunMeta {
        schema_version: RELEASE_PACKET_VERSION,
        scenario_id: spec.scenario_id,
        scenario_name: spec.scenario_name.clone(),
        binary_name: "scenario_1",
        execution_mode: "release",
        public_lane_status: "canonical_public_lane",
        stage_sync_status: "design_and_runtime_aligned",
        active_profile: spec.active_profile.clone(),
        deterministic: spec.deterministic,
        stage_count: stage_results.len(),
        config_digest_set_hex: spec.config_digest_set_hex.clone(),
        design_digest_hex: spec.design_digest_hex.clone(),
        route_table_digest: spec.route_table_digest.clone(),
        process_topology_digest_hex: spec.process_topology_digest_hex.clone(),
        journal_lineage_digest_hex: spec.journal_lineage_digest_hex.clone(),
        process_map_file: spec.traces.proc_flow_file.clone(),
        recovery_file: spec.traces.recovery_flow_file.clone(),
        wallet_scan_file: packet.wallet_scan_file.clone(),
        summary_file: packet.sim_summary_file.clone(),
        trace_files: trace_files.clone(),
        artifact_inventory: build_artifact_inventory(spec, packet),
        stage_results: stage_results.to_vec(),
        wallet_scan: build_wallet_scan_summary(wallet_scan),
        public_lane_guards: PublicLaneGuardView {
            wallet_debug_tools_required: false,
            private_lane_dependency: false,
            secret_artifacts_excluded: true,
            redaction_status: "public_only",
        },
    }
}

fn build_sim_summary(run_meta: &RunMeta) -> String {
    let mut summary = String::new();
    summary.push_str("## Release Packet\n");
    summary.push_str(&format!("- binary_name: {}\n", run_meta.binary_name));
    summary.push_str(&format!("- execution_mode: {}\n", run_meta.execution_mode));
    summary.push_str(&format!("- active_profile: {}\n", run_meta.active_profile));
    summary.push_str(&format!(
        "- public_lane_status: {}\n",
        run_meta.public_lane_status
    ));
    summary.push_str(&format!(
        "- stage_sync_status: {}\n",
        run_meta.stage_sync_status
    ));
    summary.push_str(&format!("- stage_count: {}\n", run_meta.stage_count));
    summary.push_str(&format!(
        "- wallet_debug_tools_required: {}\n",
        run_meta.public_lane_guards.wallet_debug_tools_required
    ));
    summary.push_str(&format!(
        "- private_lane_dependency: {}\n",
        run_meta.public_lane_guards.private_lane_dependency
    ));
    summary.push_str(&format!(
        "- secret_artifacts_excluded: {}\n",
        run_meta.public_lane_guards.secret_artifacts_excluded
    ));
    summary.push_str(&format!(
        "- redaction_status: {}\n",
        run_meta.public_lane_guards.redaction_status
    ));
    summary.push_str("\n## Digests\n");
    summary.push_str(&format!(
        "- config_digest_set_hex: {}\n",
        run_meta.config_digest_set_hex
    ));
    summary.push_str(&format!(
        "- design_digest_hex: {}\n",
        run_meta.design_digest_hex
    ));
    summary.push_str(&format!(
        "- route_table_digest: {}\n",
        run_meta.route_table_digest
    ));
    summary.push_str(&format!(
        "- process_topology_digest_hex: {}\n",
        run_meta.process_topology_digest_hex
    ));
    summary.push_str(&format!(
        "- journal_lineage_digest_hex: {}\n",
        run_meta.journal_lineage_digest_hex
    ));
    summary.push_str("\n## Inventory\n");
    for row in &run_meta.artifact_inventory {
        summary.push_str(&format!("- {}: {}\n", row.status, row.file));
    }
    summary.push_str("\n## Wallet Scan\n");
    summary.push_str(&format!(
        "- wallet_scan_file: {}\n",
        run_meta.wallet_scan_file
    ));
    summary.push_str(&format!("- actor: {}\n", run_meta.wallet_scan.actor));
    summary.push_str(&format!(
        "- detected_count: {}\n",
        run_meta.wallet_scan.detected_count
    ));
    summary.push_str(&format!(
        "- total_detected_amount: {}\n",
        run_meta.wallet_scan.total_detected_amount
    ));
    summary.push_str(&format!(
        "- proof_validated_count: {}\n",
        run_meta.wallet_scan.proof_validated_count
    ));
    summary.push_str(&format!(
        "- skipped_non_asset_count: {}\n",
        run_meta.wallet_scan.skipped_non_asset_count
    ));
    summary
}

fn expected_stage_results(design: &DesignDoc) -> Vec<StageResultView> {
    design
        .stages
        .iter()
        .map(|stage| StageResultView {
            stage: stage.stage,
            name: stage.name.clone(),
            result: "ok",
        })
        .collect()
}

fn load_wallet_scan_artifact(
    out_dir: &Path,
    packet: &crate::config::RuntimePacketCfg,
) -> Result<JmtScanArtifact, Scenario1Err> {
    let path = resolve_trace_file(out_dir, &packet.wallet_scan_file)?;
    let bytes = io::read_file(&path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to read wallet scan {}: {err}",
            path.display()
        ))
    })?;
    JsonCodec.deserialize(&bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to decode wallet scan {}: {err}",
            path.display()
        ))
    })
}

fn load_stage11_checkpoint(
    cfg: &ScenarioCfg,
    out_dir: &Path,
) -> Result<Stage11Checkpoint, Scenario1Err> {
    let path = stage7_checkpoint_path(cfg, out_dir);
    let bytes = io::read_file(&path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to read stage7 checkpoint {}: {err}",
            path.display()
        ))
    })?;
    JsonCodec.deserialize(&bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to decode stage7 checkpoint {}: {err}",
            path.display()
        ))
    })
}

fn validate_release_packet(
    spec: &RuntimeTraceSpec,
    cfg: &ScenarioCfg,
    design: &DesignDoc,
    cfg_path: &Path,
    design_path: &Path,
    out_dir: &Path,
    mode: PacketMode,
) -> Result<(), Scenario1Err> {
    let observability = observability_cfg(cfg)?;
    let wallet_scan = load_wallet_scan_artifact(out_dir, &observability.packet)?;
    let stage7 = load_stage11_checkpoint(cfg, out_dir)?;
    if stage7.wallet_scan_file != observability.packet.wallet_scan_file {
        return Err(Scenario1Err::Evidence(format!(
            "stage7 checkpoint wallet_scan_file drifted: expected {}, got {}",
            observability.packet.wallet_scan_file, stage7.wallet_scan_file
        )));
    }
    if wallet_scan.status != "ok" {
        return Err(Scenario1Err::Evidence(format!(
            "wallet scan status drifted: expected ok, got {}",
            wallet_scan.status
        )));
    }
    if wallet_scan.scan_path != "jmt_scan" {
        return Err(Scenario1Err::Evidence(format!(
            "wallet scan path drifted: expected jmt_scan, got {}",
            wallet_scan.scan_path
        )));
    }
    if wallet_scan.proof_validated_count != wallet_scan.candidate_count {
        return Err(Scenario1Err::Evidence(format!(
            "wallet scan proof_validated_count drifted: expected {}, got {}",
            wallet_scan.candidate_count, wallet_scan.proof_validated_count
        )));
    }
    if wallet_scan.store_root_hex != stage7.new_root_hex {
        return Err(Scenario1Err::Evidence(format!(
            "wallet scan store_root_hex drifted from stage7 new_root_hex: expected {}, got {}",
            stage7.new_root_hex, wallet_scan.store_root_hex
        )));
    }
    if wallet_scan.detected_count != stage7.charlie_detected_count {
        return Err(Scenario1Err::Evidence(format!(
            "wallet scan detected_count drifted from stage7: expected {}, got {}",
            stage7.charlie_detected_count, wallet_scan.detected_count
        )));
    }
    if wallet_scan.total_detected_amount != stage7.charlie_detected_amount {
        return Err(Scenario1Err::Evidence(format!(
            "wallet scan total_detected_amount drifted from stage7: expected {}, got {}",
            stage7.charlie_detected_amount, wallet_scan.total_detected_amount
        )));
    }

    let trace_files = TraceFileView {
        cfg_flow_file: spec.traces.cfg_flow_file.clone(),
        tx_flow_file: spec.traces.tx_flow_file.clone(),
        route_flow_file: spec.traces.route_flow_file.clone(),
        plan_flow_file: spec.traces.plan_flow_file.clone(),
        journal_flow_file: spec.traces.journal_flow_file.clone(),
        scope_flow_file: spec.traces.scope_flow_file.clone(),
        proc_flow_file: spec.traces.proc_flow_file.clone(),
        recovery_flow_file: spec.traces.recovery_flow_file.clone(),
        leaf_flow_file: spec.traces.leaf_flow_file.clone(),
        proof_flow_file: spec.traces.proof_flow_file.clone(),
        pub_flow_file: spec.traces.pub_flow_file.clone(),
        val_flow_file: spec.traces.val_flow_file.clone(),
        watch_flow_file: spec.traces.watch_flow_file.clone(),
    };
    let expected_run_meta = build_run_meta(
        spec,
        &observability.packet,
        &trace_files,
        &expected_stage_results(design),
        &wallet_scan,
    );
    let run_meta_path = resolve_trace_file(out_dir, &observability.packet.run_meta_file)?;
    let run_meta_bytes = io::read_file(&run_meta_path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to read run meta {}: {err}",
            run_meta_path.display()
        ))
    })?;
    let actual_run_meta: Value = JsonCodec.deserialize(&run_meta_bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to decode run meta {}: {err}",
            run_meta_path.display()
        ))
    })?;
    check_trace_payload(
        &actual_run_meta,
        &json_value(&expected_run_meta)?,
        &run_meta_path,
    )?;

    let hist_path = resolve_trace_file(
        out_dir,
        emitted_public_file(&observability.packet, "hist_flow.json")?,
    )?;
    let actual_hist_flow: Value = load_json_file(&hist_path, "hist_flow")?;
    let expected_hist_flow = match mode {
        PacketMode::Strict => build_hist_flow(
            flow_common(spec, cfg_path, design_path).clone_with_version(
                RELEASE_PACKET_VERSION,
                "hist_flow",
                IMPORTED_ARTIFACT_CONTRACT,
            ),
            cfg,
            out_dir,
            &trace_files,
        )?,
        PacketMode::Cached => build_hist_flow_cached(
            flow_common(spec, cfg_path, design_path).clone_with_version(
                RELEASE_PACKET_VERSION,
                "hist_flow",
                IMPORTED_ARTIFACT_CONTRACT,
            ),
            cfg,
            out_dir,
            &trace_files,
            &actual_hist_flow,
        )?,
    };
    check_trace_payload(
        &actual_hist_flow,
        &json_value(&expected_hist_flow)?,
        &hist_path,
    )?;

    let expected_occ_flow = build_occ_flow(
        flow_common(spec, cfg_path, design_path).clone_with_version(
            RELEASE_PACKET_VERSION,
            "occ_flow",
            IMPORTED_ARTIFACT_CONTRACT,
        ),
        cfg,
        out_dir,
        &trace_files,
    )?;
    let occ_path = resolve_trace_file(
        out_dir,
        emitted_public_file(&observability.packet, "occ_flow.json")?,
    )?;
    let actual_occ_flow: Value = load_json_file(&occ_path, "occ_flow")?;
    check_trace_payload(
        &actual_occ_flow,
        &json_value(&expected_occ_flow)?,
        &occ_path,
    )?;

    for emitted_file in ["asset_flow.json", "voucher_flow.json", "right_flow.json"] {
        let object_path = resolve_trace_file(
            out_dir,
            emitted_public_file(&observability.packet, emitted_file)?,
        )?;
        let actual_object_flow: Value = load_json_file(&object_path, emitted_file)?;
        let expected_object_flow = build_object_flow(
            flow_common(spec, cfg_path, design_path).clone_with_version(
                RELEASE_PACKET_VERSION,
                emitted_file.trim_end_matches(".json"),
                IMPORTED_ARTIFACT_CONTRACT,
            ),
            cfg,
            emitted_file,
        )?;
        check_trace_payload(
            &actual_object_flow,
            &json_value(&expected_object_flow)?,
            &object_path,
        )?;
    }

    let summary_path = resolve_trace_file(out_dir, &observability.packet.sim_summary_file)?;
    let actual_summary = io::read_to_string(&summary_path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to read simulator summary {}: {err}",
            summary_path.display()
        ))
    })?;
    let expected_summary = build_sim_summary(&expected_run_meta);
    if actual_summary != expected_summary {
        return Err(Scenario1Err::Evidence(format!(
            "{} summary drifted from canonical release packet",
            summary_path.display()
        )));
    }

    Ok(())
}

fn validate_observability_cfg(
    observability: &crate::config::RuntimeObservabilityCfg,
) -> Result<(), Scenario1Err> {
    let mut supported = BTreeSet::new();
    for profile in &observability.supported_profiles {
        if profile.id.trim().is_empty() || profile.purpose.trim().is_empty() {
            return Err(Scenario1Err::Evidence(
                "runtime_observability supported profile fields must not be blank".to_string(),
            ));
        }
        if !supported.insert(profile.id.clone()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability duplicate profile {}",
                profile.id
            )));
        }
    }
    for required in REQUIRED_PROFILES {
        if !supported.contains(*required) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability missing required profile {required}"
            )));
        }
    }
    let mut heavy_only = BTreeSet::new();
    for profile in &observability.heavy_only_profiles {
        if profile.trim().is_empty() {
            return Err(Scenario1Err::Evidence(
                "runtime_observability heavy_only_profiles must not contain blanks".to_string(),
            ));
        }
        if !supported.contains(profile.as_str()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability heavy-only profile {profile} must also appear in supported_profiles"
            )));
        }
        if !heavy_only.insert(profile.clone()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability duplicate heavy-only profile {profile}"
            )));
        }
    }
    if !heavy_only.contains(HEAVY_ONLY_PROFILE) {
        return Err(Scenario1Err::Evidence(format!(
            "runtime_observability heavy_only_profiles must include {HEAVY_ONLY_PROFILE}"
        )));
    }

    if observability
        .publication
        .acceptance_profile
        .trim()
        .is_empty()
        || observability
            .publication
            .inherited_runtime_profile
            .trim()
            .is_empty()
        || observability.publication.topology_status.trim().is_empty()
    {
        return Err(Scenario1Err::Evidence(
            "runtime_observability publication fields must not be blank".to_string(),
        ));
    }
    if observability.publication.public_leaf_count == 0 {
        return Err(Scenario1Err::Evidence(
            "runtime_observability publication public_leaf_count must stay positive".to_string(),
        ));
    }
    if observability.publication.publication_activation_checkpoint == 0 {
        return Err(Scenario1Err::Evidence(
            "runtime_observability publication activation checkpoint must stay positive"
                .to_string(),
        ));
    }
    if observability
        .publication
        .positive_topology_examples
        .is_empty()
    {
        return Err(Scenario1Err::Evidence(
            "runtime_observability publication must declare at least one positive topology example"
                .to_string(),
        ));
    }
    for example in &observability.publication.positive_topology_examples {
        if example.fixture_id.trim().is_empty()
            || example.old_topology.trim().is_empty()
            || example.new_topology.trim().is_empty()
            || example.join_mode.trim().is_empty()
            || example.transfer_target.trim().is_empty()
        {
            return Err(Scenario1Err::Evidence(
                "runtime_observability publication topology example fields must not be blank"
                    .to_string(),
            ));
        }
        if example.old_aggregator_count == 0
            || example.old_shard_count == 0
            || example.new_aggregator_count == 0
            || example.new_shard_count == 0
        {
            return Err(Scenario1Err::Evidence(
                "runtime_observability publication topology counts must stay positive".to_string(),
            ));
        }
        if example.route_generation_to < example.route_generation_from {
            return Err(Scenario1Err::Evidence(
                "runtime_observability publication route generations must stay monotonic"
                    .to_string(),
            ));
        }
        if example.activation_checkpoint == 0 {
            return Err(Scenario1Err::Evidence(
                "runtime_observability publication topology activation checkpoint must stay positive"
                    .to_string(),
            ));
        }
        if example.secondary_aggregator_ids.is_empty() {
            return Err(Scenario1Err::Evidence(
                "runtime_observability publication topology secondary set must stay non-empty"
                    .to_string(),
            ));
        }
        let mut top_level_secondary_ids = BTreeSet::new();
        for secondary in &example.secondary_aggregator_ids {
            if *secondary == example.owner_aggregator_id
                || !top_level_secondary_ids.insert(*secondary)
            {
                return Err(Scenario1Err::Evidence(
                    "runtime_observability publication topology owner and secondary set must stay unique"
                        .to_string(),
                ));
            }
        }
        if !example.transition_stages.is_empty() {
            if example.transition_stages.len() < 2 {
                return Err(Scenario1Err::Evidence(
                    "runtime_observability staged topology examples require at least two stages"
                        .to_string(),
                ));
            }
            let first = example.transition_stages.first().ok_or_else(|| {
                Scenario1Err::Evidence(
                    "runtime_observability staged topology examples require a first stage"
                        .to_string(),
                )
            })?;
            let last = example.transition_stages.last().ok_or_else(|| {
                Scenario1Err::Evidence(
                    "runtime_observability staged topology examples require a last stage"
                        .to_string(),
                )
            })?;
            if first.topology != example.old_topology
                || first.aggregator_count != example.old_aggregator_count
                || first.shard_count != example.old_shard_count
                || first.route_generation != example.route_generation_from
            {
                return Err(Scenario1Err::Evidence(
                    "runtime_observability staged topology first stage must match old_* fields"
                        .to_string(),
                ));
            }
            if last.topology != example.new_topology
                || last.aggregator_count != example.new_aggregator_count
                || last.shard_count != example.new_shard_count
                || last.route_generation != example.route_generation_to
            {
                return Err(Scenario1Err::Evidence(
                    "runtime_observability staged topology last stage must match new_* fields"
                        .to_string(),
                ));
            }
            let mut prior_generation = None;
            for stage in &example.transition_stages {
                if stage.stage_id.trim().is_empty()
                    || stage.topology.trim().is_empty()
                    || stage.aggregator_count == 0
                    || stage.shard_count == 0
                {
                    return Err(Scenario1Err::Evidence(
                        "runtime_observability staged topology fields must not be blank or zero"
                            .to_string(),
                    ));
                }
                if let Some(previous) = prior_generation {
                    if stage.route_generation <= previous {
                        return Err(Scenario1Err::Evidence(
                            "runtime_observability staged topology generations must increase strictly"
                                .to_string(),
                        ));
                    }
                }
                if stage.secondary_aggregator_ids.is_empty() {
                    return Err(Scenario1Err::Evidence(
                        "runtime_observability staged topology secondary set must stay non-empty"
                            .to_string(),
                    ));
                }
                let mut secondary_ids = BTreeSet::new();
                for secondary in &stage.secondary_aggregator_ids {
                    if *secondary == stage.owner_aggregator_id || !secondary_ids.insert(*secondary)
                    {
                        return Err(Scenario1Err::Evidence(
                            "runtime_observability staged topology owner and secondary set must stay unique"
                                .to_string(),
                        ));
                    }
                }
                prior_generation = Some(stage.route_generation);
            }
        }
        if !example.removed_aggregator_ids.is_empty() {
            let mut removed = BTreeSet::new();
            for aggregator_id in &example.removed_aggregator_ids {
                if !removed.insert(*aggregator_id) {
                    return Err(Scenario1Err::Evidence(
                        "runtime_observability removed_aggregator_ids must stay unique".to_string(),
                    ));
                }
            }
        }
        if (example.removed_aggregator_absent_from_owner_tables
            || example.removed_aggregator_absent_from_secondary_tables
            || example.all_shards_owned_across_stages
            || example.prior_lineage_preserved
            || example.publication_continuity_preserved)
            && example.transition_stages.is_empty()
        {
            return Err(Scenario1Err::Evidence(
                "runtime_observability staged invariants require transition_stages evidence"
                    .to_string(),
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for (field, value) in [
        ("cfg_flow_file", observability.traces.cfg_flow_file.as_str()),
        ("tx_flow_file", observability.traces.tx_flow_file.as_str()),
        (
            "route_flow_file",
            observability.traces.route_flow_file.as_str(),
        ),
        (
            "plan_flow_file",
            observability.traces.plan_flow_file.as_str(),
        ),
        (
            "journal_flow_file",
            observability.traces.journal_flow_file.as_str(),
        ),
        (
            "scope_flow_file",
            observability.traces.scope_flow_file.as_str(),
        ),
        (
            "proc_flow_file",
            observability.traces.proc_flow_file.as_str(),
        ),
        (
            "recovery_flow_file",
            observability.traces.recovery_flow_file.as_str(),
        ),
        (
            "leaf_flow_file",
            observability.traces.leaf_flow_file.as_str(),
        ),
        (
            "proof_flow_file",
            observability.traces.proof_flow_file.as_str(),
        ),
        ("pub_flow_file", observability.traces.pub_flow_file.as_str()),
        ("val_flow_file", observability.traces.val_flow_file.as_str()),
        (
            "watch_flow_file",
            observability.traces.watch_flow_file.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability {field} must not be empty"
            )));
        }
        if !paths.insert(value.to_string()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability trace path duplicated: {value}"
            )));
        }
    }

    for (field, value) in [
        ("run_meta_file", observability.packet.run_meta_file.as_str()),
        (
            "wallet_scan_file",
            observability.packet.wallet_scan_file.as_str(),
        ),
        (
            "sim_summary_file",
            observability.packet.sim_summary_file.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability packet {field} must not be empty"
            )));
        }
        if !paths.insert(value.to_string()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability packet path duplicated: {value}"
            )));
        }
    }

    let mut emitted = BTreeSet::new();
    for value in &observability.packet.emitted_public_files {
        if value.trim().is_empty() {
            return Err(Scenario1Err::Evidence(
                "runtime_observability packet emitted_public_files must not contain blanks"
                    .to_string(),
            ));
        }
        if paths.contains(value) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability packet emitted file duplicated with core packet: {value}"
            )));
        }
        if !emitted.insert(value.clone()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability packet emitted_public_files contain duplicate {}",
                value
            )));
        }
        paths.insert(value.clone());
    }
    for required in REQUIRED_EMITTED_PUBLIC_FILES {
        if !emitted.contains(*required) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime_observability packet missing emitted_public_file {required}"
            )));
        }
    }

    Ok(())
}

fn build_process_view(
    hjmt: &z00z_rollup_node::HjmtCfg,
) -> Result<ProcessTopologyView, Scenario1Err> {
    for agg in &hjmt.aggs {
        hjmt.check_life_cmd(agg.aggregator_id, &agg.lifecycle.start_cmd, "start")
            .map_err(|err| Scenario1Err::Evidence(err.to_string()))?;
        hjmt.check_life_cmd(agg.aggregator_id, &agg.lifecycle.restart_cmd, "restart")
            .map_err(|err| Scenario1Err::Evidence(err.to_string()))?;
    }
    let mut aggregators = hjmt
        .aggs
        .iter()
        .map(|agg| AggregatorProcessView {
            aggregator_id: agg.aggregator_id.as_u16(),
            process_id: format!("agg-{}", agg.aggregator_id.as_u16()),
            role: agg.role.clone(),
            listen_addr: agg.network.listen_addr.clone(),
            data_dir: agg.paths.data_dir.clone(),
            journal_path: agg.paths.journal_path.clone(),
            log_path: agg.paths.log_path.clone(),
            start_cmd: agg.lifecycle.start_cmd.clone(),
            restart_cmd: agg.lifecycle.restart_cmd.clone(),
            shard_ids: agg
                .shards
                .iter()
                .map(|shard| shard.shard_id.as_u16())
                .collect(),
        })
        .collect::<Vec<_>>();
    aggregators.sort_by_key(|agg| agg.aggregator_id);

    let view = ProcessTopologyView {
        profile: hjmt.profile.clone(),
        process_model: "os_process",
        shard_mapping: hjmt.shard_mapping().as_str(),
        agg_count: hjmt.agg_count(),
        shard_count: hjmt.shard_count(),
        routing_generation: hjmt.routing_generation(),
        aggregators,
    };
    validate_process_view(&view)?;
    Ok(view)
}

fn validate_process_view(view: &ProcessTopologyView) -> Result<(), Scenario1Err> {
    if view.process_model != "os_process" {
        return Err(Scenario1Err::Evidence(format!(
            "runtime process model drifted from os_process: {}",
            view.process_model
        )));
    }
    if view.shard_mapping != "aggregator_owned" && view.shard_mapping != "shard_process" {
        return Err(Scenario1Err::Evidence(format!(
            "runtime shard mapping is unsupported: {}",
            view.shard_mapping
        )));
    }
    if view.agg_count == 0 || view.shard_count == 0 {
        return Err(Scenario1Err::Evidence(
            "runtime process topology counts must stay positive".to_string(),
        ));
    }
    if view.aggregators.len() != view.agg_count {
        return Err(Scenario1Err::Evidence(format!(
            "runtime process topology agg_count {} does not match {} aggregator rows",
            view.agg_count,
            view.aggregators.len()
        )));
    }

    let mut aggregator_ids = BTreeSet::new();
    let mut process_ids = BTreeSet::new();
    let mut listen_addrs = BTreeSet::new();
    let mut data_dirs = BTreeSet::new();
    let mut journal_paths = BTreeSet::new();

    for agg in &view.aggregators {
        if agg.process_id.trim().is_empty()
            || agg.role.trim().is_empty()
            || agg.listen_addr.trim().is_empty()
            || agg.start_cmd.trim().is_empty()
            || agg.restart_cmd.trim().is_empty()
        {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology aggregator {} contains blank required fields",
                agg.aggregator_id
            )));
        }
        if !aggregator_ids.insert(agg.aggregator_id) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology duplicate aggregator_id {}",
                agg.aggregator_id
            )));
        }
        if !process_ids.insert(agg.process_id.clone()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology duplicate process_id {}",
                agg.process_id
            )));
        }
        if !listen_addrs.insert(agg.listen_addr.clone()) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology duplicate listen_addr {}",
                agg.listen_addr
            )));
        }
        if !data_dirs.insert(normalize_path(&agg.data_dir)) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology duplicate data_dir {}",
                agg.data_dir.display()
            )));
        }
        if !journal_paths.insert(normalize_path(&agg.journal_path)) {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology duplicate journal_path {}",
                agg.journal_path.display()
            )));
        }
        if view.shard_mapping == "shard_process" && agg.shard_ids.len() > 1 {
            return Err(Scenario1Err::Evidence(format!(
                "runtime shard_process topology drifted to multiple primary shards on aggregator {}",
                agg.aggregator_id
            )));
        }
        if !agg.start_cmd.contains("--planner-config")
            || !agg.start_cmd.contains("--storage-config")
        {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology aggregator {} start_cmd must stay config-bound",
                agg.aggregator_id
            )));
        }
        if !agg.restart_cmd.contains("--planner-config")
            || !agg.restart_cmd.contains("--storage-config")
        {
            return Err(Scenario1Err::Evidence(format!(
                "runtime process topology aggregator {} restart_cmd must stay config-bound",
                agg.aggregator_id
            )));
        }
    }

    Ok(())
}

fn build_journal_view(hjmt: &z00z_rollup_node::HjmtCfg) -> JournalContractView {
    let mut lineage_rows = hjmt
        .aggs
        .iter()
        .flat_map(|agg| {
            agg.shards.iter().map(move |shard| PlacementRowView {
                shard_id: shard.shard_id.as_u16(),
                primary_aggregator_id: agg.aggregator_id.as_u16(),
                secondary_ids: shard
                    .secondary_ids
                    .iter()
                    .map(|secondary| secondary.as_u16())
                    .collect(),
                expected_journal_lineage_hex: hex::encode(shard.expected_journal_lineage),
            })
        })
        .collect::<Vec<_>>();
    lineage_rows.sort_by_key(|row| row.shard_id);

    JournalContractView {
        backend: hjmt.storage.backend.clone(),
        generation: hjmt.storage.generation,
        cache_capacity: hjmt.storage.settings.cache_capacity,
        lock_timeout_ms: hjmt.storage.settings.lock_timeout_ms,
        storage_paths: StoragePathsView {
            data_dir: hjmt.storage.paths.data_dir.clone(),
            journal_dir: hjmt.storage.paths.journal_dir.clone(),
            export_dir: hjmt.storage.paths.export_dir.clone(),
            import_dir: hjmt.storage.paths.import_dir.clone(),
            lock_path: hjmt.storage.paths.lock_path.clone(),
        },
        lineage_rows,
    }
}

fn flow_common(spec: &RuntimeTraceSpec, cfg_path: &Path, design_path: &Path) -> FlowCommon {
    FlowCommon {
        trace_version: RUNTIME_TRACE_VERSION,
        trace_kind: "cfg_flow",
        trace_mode: RUNTIME_CONTRACT,
        scenario_id: spec.scenario_id,
        scenario_name: spec.scenario_name.clone(),
        active_profile: spec.active_profile.clone(),
        deterministic: spec.deterministic,
        semantic_digest_hex: semantic_digest_hex(spec),
        config_digest_set_hex: spec.config_digest_set_hex.clone(),
        design_digest_hex: spec.design_digest_hex.clone(),
        route_table_digest: spec.route_table_digest.clone(),
        process_topology_digest_hex: spec.process_topology_digest_hex.clone(),
        journal_lineage_digest_hex: spec.journal_lineage_digest_hex.clone(),
        scenario_config_path: cfg_path.to_path_buf(),
        design_path: trace_design_path(design_path),
        hjmt_home: spec.hjmt_home.clone(),
        config_digests: spec.config_digests.clone(),
    }
}

fn trace_design_path(design_path: &Path) -> PathBuf {
    let canonical_relative = PathBuf::from("src/scenario_1/scenario_design.yaml");
    let simulator_root = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let canonical_absolute = normalize_path(&simulator_root.join(&canonical_relative));
    let candidate = if design_path.is_absolute() {
        normalize_path(design_path)
    } else {
        normalize_path(&simulator_root.join(design_path))
    };

    if candidate == canonical_absolute {
        canonical_relative
    } else {
        design_path.to_path_buf()
    }
}

impl FlowCommon {
    fn clone_with(&self, trace_kind: &'static str, trace_mode: &'static str) -> Self {
        self.clone_with_version(self.trace_version, trace_kind, trace_mode)
    }

    fn clone_with_version(
        &self,
        trace_version: &'static str,
        trace_kind: &'static str,
        trace_mode: &'static str,
    ) -> Self {
        let mut cloned = self.clone();
        cloned.trace_version = trace_version;
        cloned.trace_kind = trace_kind;
        cloned.trace_mode = trace_mode;
        cloned
    }
}

fn semantic_digest_hex(spec: &RuntimeTraceSpec) -> String {
    // INVARIANT: the semantic digest payload is a tuple of serde-serializable scalars and strings.
    digest_hex(&(
        spec.scenario_id,
        spec.active_profile.as_str(),
        spec.deterministic,
        spec.config_digest_set_hex.as_str(),
        spec.design_digest_hex.as_str(),
        spec.route_table_digest.as_str(),
        spec.process_topology_digest_hex.as_str(),
        spec.journal_lineage_digest_hex.as_str(),
    ))
    .expect("BUG: semantic trace digest payload must serialize")
}

fn config_digest_set_hex(records: &[ConfigDigestRecord]) -> Result<String, Scenario1Err> {
    let reduced = records
        .iter()
        .map(|record| (record.label.as_str(), record.digest_hex.as_str()))
        .collect::<Vec<_>>();
    digest_hex(&reduced)
}

fn digest_hex(value: &impl Serialize) -> Result<String, Scenario1Err> {
    let bytes = JsonCodec.serialize(value).map_err(|err| {
        Scenario1Err::Evidence(format!("failed to encode runtime trace digest: {err}"))
    })?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn resolve_trace_file(out_dir: &Path, rel: &str) -> Result<PathBuf, Scenario1Err> {
    let candidate = PathBuf::from(rel);
    if candidate.is_absolute() {
        return Err(Scenario1Err::Evidence(format!(
            "runtime trace path must stay relative: {rel}"
        )));
    }
    if candidate
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(Scenario1Err::Evidence(format!(
            "runtime trace path must not use parent segments: {rel}"
        )));
    }
    Ok(out_dir.join(candidate))
}

fn resolve_path(out_dir: &Path, rel: &str) -> PathBuf {
    let candidate = PathBuf::from(rel);
    if candidate.is_absolute() {
        candidate
    } else {
        out_dir.join(candidate)
    }
}

fn write_trace_file(out_dir: &Path, rel: &str, value: &impl Serialize) -> Result<(), Scenario1Err> {
    let path = resolve_trace_file(out_dir, rel)?;
    let bytes = JsonCodec.serialize(value).map_err(|err| {
        Scenario1Err::Evidence(format!("failed to encode {}: {err}", path.display()))
    })?;
    io::write_file(&path, &bytes)?;
    Ok(())
}

fn write_text_file(out_dir: &Path, rel: &str, text: &str) -> Result<(), Scenario1Err> {
    let path = resolve_trace_file(out_dir, rel)?;
    io::write_file(&path, text.as_bytes())?;
    Ok(())
}

fn file_digest_record(label: &str, path: &Path) -> Result<ConfigDigestRecord, Scenario1Err> {
    let bytes = io::read_file(path)?;
    Ok(ConfigDigestRecord {
        label: label.to_string(),
        path: path.to_path_buf(),
        digest_hex: hex::encode(Sha256::digest(&bytes)),
    })
}

fn stage4_tx_pkg_path(cfg: &ScenarioCfg, out_dir: &Path) -> Result<PathBuf, Scenario1Err> {
    let stage4 = stage4_cfg(cfg)?;
    Ok(resolve_runtime_output_path(
        out_dir,
        &stage4.paths.tx_pkg_file,
    ))
}

fn stage5_leaf_path(cfg: &ScenarioCfg, out_dir: &Path) -> PathBuf {
    let paths = cfg.stage5_paths();
    resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(paths.transactions_dir)
            .join(paths.tx_file)
            .to_string_lossy(),
    )
}

fn stage6_frag_path(cfg: &ScenarioCfg, out_dir: &Path, first: bool) -> PathBuf {
    let paths = cfg.stage6_paths();
    let file = if first {
        paths.frag1_file
    } else {
        paths.frag2_file
    };
    resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(paths.transactions_dir)
            .join(file)
            .to_string_lossy(),
    )
}

fn stage6_bridge_path(cfg: &ScenarioCfg, out_dir: &Path) -> PathBuf {
    let paths = cfg.stage6_paths();
    resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(paths.transactions_dir)
            .join(paths.checkpoint_file)
            .to_string_lossy(),
    )
}

fn stage7_checkpoint_path(cfg: &ScenarioCfg, out_dir: &Path) -> PathBuf {
    let paths = cfg.stage7_paths();
    resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(paths.transactions_dir)
            .join(paths.checkpoint_file)
            .to_string_lossy(),
    )
}

fn stage8_checkpoint_path(cfg: &ScenarioCfg, out_dir: &Path) -> PathBuf {
    let paths = cfg.stage8_paths();
    resolve_runtime_output_path(
        out_dir,
        &PathBuf::from(paths.transactions_dir)
            .join(paths.checkpoint_file)
            .to_string_lossy(),
    )
}

fn resolve_runtime_output_path(out_dir: &Path, rel: &str) -> PathBuf {
    let candidate = PathBuf::from(rel);
    if candidate.is_absolute() {
        return candidate;
    }
    let marker = PathBuf::from(SCENARIO_OUTPUT_MARKER);
    if candidate.starts_with(&marker) {
        if let Ok(suffix) = candidate.strip_prefix(&marker) {
            return out_dir.join(suffix);
        }
    }
    out_dir.join(candidate)
}

fn cache_edge_samples(spec: &RuntimeTraceSpec) -> Vec<usize> {
    if spec.active_profile != "SIM-CACHE-EDGE" {
        return Vec::new();
    }
    let cap = spec.journal_view.cache_capacity;
    [
        cap.saturating_sub(1),
        cap,
        cap.saturating_add(1),
        cap.saturating_mul(2),
    ]
    .to_vec()
}

fn stage_result_tag(result: &StageResult) -> &'static str {
    match result {
        StageResult::Ok => "ok",
        StageResult::Warn(_) => "warn",
        StageResult::Fail(_) => "fail",
    }
}

fn load_node_cfg(hjmt_home: &Path) -> Result<NodeConfig, Scenario1Err> {
    NodeConfig::from_hjmt_home(hjmt_home)
        .map_err(|err| Scenario1Err::Evidence(format!("failed to reload hjmt runtime home: {err}")))
}

fn expected_trace_version(trace_kind: &str) -> &'static str {
    match trace_kind {
        "leaf_flow" | "proof_flow" | "pub_flow" | "val_flow" | "watch_flow" => {
            PUBLICATION_TRACE_VERSION
        }
        _ => RUNTIME_TRACE_VERSION,
    }
}

fn publication_topology_views(
    publication: &crate::config::PublicationObservabilityCfg,
) -> Vec<PublicationTopologyView> {
    publication
        .positive_topology_examples
        .iter()
        .map(|example| PublicationTopologyView {
            fixture_id: example.fixture_id.clone(),
            shard_id: example.shard_id,
            old_topology: example.old_topology.clone(),
            new_topology: example.new_topology.clone(),
            old_aggregator_count: example.old_aggregator_count,
            old_shard_count: example.old_shard_count,
            new_aggregator_count: example.new_aggregator_count,
            new_shard_count: example.new_shard_count,
            route_generation_from: example.route_generation_from,
            route_generation_to: example.route_generation_to,
            owner_aggregator_id: example.owner_aggregator_id,
            secondary_aggregator_ids: example.secondary_aggregator_ids.clone(),
            join_mode: example.join_mode.clone(),
            transfer_target: example.transfer_target.clone(),
            activation_checkpoint: example.activation_checkpoint,
            transition_stages: example
                .transition_stages
                .iter()
                .map(|stage| PublicationTopologyStageView {
                    stage_id: stage.stage_id.clone(),
                    topology: stage.topology.clone(),
                    aggregator_count: stage.aggregator_count,
                    shard_count: stage.shard_count,
                    route_generation: stage.route_generation,
                    owner_aggregator_id: stage.owner_aggregator_id,
                    secondary_aggregator_ids: stage.secondary_aggregator_ids.clone(),
                })
                .collect(),
            removed_aggregator_ids: example.removed_aggregator_ids.clone(),
            removed_aggregator_absent_from_owner_tables: example
                .removed_aggregator_absent_from_owner_tables,
            removed_aggregator_absent_from_secondary_tables: example
                .removed_aggregator_absent_from_secondary_tables,
            all_shards_owned_across_stages: example.all_shards_owned_across_stages,
            prior_lineage_preserved: example.prior_lineage_preserved,
            publication_continuity_preserved: example.publication_continuity_preserved,
        })
        .collect()
}

fn build_publication_evidence(
    spec: &RuntimeTraceSpec,
    cfg: &ScenarioCfg,
    out_dir: &Path,
) -> Result<PublicationEvidence, Scenario1Err> {
    let publication_cfg = cfg.publication_observability_ref().ok_or_else(|| {
        Scenario1Err::Evidence("runtime_observability publication missing".to_string())
    })?;
    let source = stage13_publication_source(out_dir)?;
    let route_digest = decode_hex32(&spec.route_table_digest, "route_table_digest")?;
    let route = load_publication_route(spec)?;
    let binding = load_stage12_publication_binding(cfg, out_dir, route_digest)?;
    let publication_checkpoint = publication_cfg.publication_activation_checkpoint;
    let store = SettlementStore::load(resolve_path(out_dir, "hjmt/store"))
        .map_err(storage_err("load persisted stage13 settlement store"))?;
    let mut usable_sources = Vec::with_capacity(spec.journal_view.lineage_rows.len());
    for source_path in &source.source_paths {
        if store.settlement_proof_blob(&source_path.path).is_ok() {
            usable_sources.push(source_path.clone());
            if usable_sources.len() == spec.journal_view.lineage_rows.len() {
                break;
            }
        }
    }
    if usable_sources.len() != spec.journal_view.lineage_rows.len() {
        return Err(Scenario1Err::Evidence(format!(
            "stage13 persisted store exposed {} proof-ready settlement paths, need {}",
            usable_sources.len(),
            spec.journal_view.lineage_rows.len()
        )));
    }

    let mut leaf_rows = Vec::with_capacity(spec.journal_view.lineage_rows.len());
    let mut proof_rows = Vec::with_capacity(spec.journal_view.lineage_rows.len());
    let mut shard_leaves = Vec::with_capacity(spec.journal_view.lineage_rows.len());
    let mut shard_proofs = Vec::with_capacity(spec.journal_view.lineage_rows.len());
    let mut policy_generations = Vec::with_capacity(spec.journal_view.lineage_rows.len());
    let mut bucket_policy_digests = Vec::with_capacity(spec.journal_view.lineage_rows.len());

    for (row, source_path) in spec
        .journal_view
        .lineage_rows
        .iter()
        .zip(usable_sources.iter())
    {
        let (leaf, proof, policy_generation, bucket_policy_digest) =
            build_publication_leaf(spec, &store, source_path, route_digest, row)?;
        leaf_rows.push(PublicationLeafView {
            shard_id: row.shard_id,
            source_settlement_path: source_path.path_text.clone(),
            primary_aggregator_id: row.primary_aggregator_id,
            secondary_ids: row.secondary_ids.clone(),
            state_root_hex: hex::encode(leaf.shard_root),
            leaf_canonical_bytes_hex: hex::encode(
                leaf.canonical_bytes()
                    .map_err(proof_err("encode shard leaf"))?,
            ),
            leaf_digest_hex: hex::encode(leaf.digest().map_err(proof_err("digest shard leaf"))?),
            journal_checkpoint: leaf.journal_checkpoint,
            local_sequence: leaf.local_sequence,
            policy_set_digest_hex: hex::encode(leaf.policy_set_digest),
        });
        shard_leaves.push(leaf);
        shard_proofs.push(proof);
        policy_generations.push(policy_generation);
        bucket_policy_digests.push(bucket_policy_digest);
    }

    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        publication_checkpoint,
        route_digest,
        source.prior_public_root,
        shard_leaves.clone(),
    );
    check_publication_route_v1(&publication, &route)
        .map_err(proof_err("verify checkpoint publication route contract"))?;
    let public_root = publication
        .public_root_v1()
        .map_err(proof_err("compute public root"))?;
    let publication_digest_hex = hex::encode(
        publication
            .digest()
            .map_err(proof_err("digest checkpoint publication"))?,
    );
    let public_root_hex = hex::encode(public_root.into_bytes());

    for (index, row) in spec.journal_view.lineage_rows.iter().enumerate() {
        let proof_blob = &shard_proofs[index];
        let proof_family = proof_blob.hjmt_proof_family().ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "publication shard {} missing proof family",
                row.shard_id
            ))
        })?;
        let leaf_family = proof_blob.hjmt_leaf_family().ok_or_else(|| {
            Scenario1Err::Evidence(format!(
                "publication shard {} missing leaf family",
                row.shard_id
            ))
        })?;
        let policy_set = spec_policy_set(
            shard_leaves[index],
            policy_generations[index],
            bucket_policy_digests[index],
        );
        let public_proof = CheckpointPublicationProofV1::new(
            RootGenerationTagV1::RootGeneration1,
            public_root,
            publication.clone(),
            index as u32,
            ShardProofContextV1::new(
                u32::from(row.shard_id),
                shard_leaves[index].routing_generation,
                route_digest,
                policy_generations[index],
                bucket_policy_digests[index],
                proof_family,
                leaf_family,
            ),
            policy_set,
            proof_blob.clone(),
        );
        check_public_checkpoint_v1(&public_proof)
            .map_err(proof_err("verify checkpoint publication proof contract"))?;
        check_public_checkpoint_route_v1(&public_proof, &route).map_err(proof_err(
            "verify checkpoint publication proof route contract",
        ))?;
        public_proof
            .verify_against_public_root_v1(public_root)
            .map_err(proof_err("verify checkpoint publication proof"))?;
        let proof_bytes = public_proof
            .encode()
            .map_err(proof_err("encode checkpoint publication proof"))?;
        proof_rows.push(PublicationProofView {
            shard_id: row.shard_id,
            source_settlement_path: usable_sources[index].path_text.clone(),
            proof_digest_hex: hex::encode(Sha256::digest(&proof_bytes)),
            proof_size_bytes: proof_bytes.len(),
            public_root_hex: public_root_hex.clone(),
            proof_family: family_tag(proof_family),
            leaf_family: leaf_family_tag(leaf_family),
            journal_checkpoint: shard_leaves[index].journal_checkpoint,
            policy_generation: policy_generations[index],
            bucket_policy_digest_hex: hex::encode(bucket_policy_digests[index]),
        });
    }

    Ok(PublicationEvidence {
        publication_checkpoint,
        prior_public_root_hex: hex::encode(source.prior_public_root.into_bytes()),
        public_root_hex,
        publication_digest_hex,
        canonical_publication_hex: hex::encode(
            publication
                .canonical_bytes()
                .map_err(proof_err("encode checkpoint publication"))?,
        ),
        draft_id_hex: binding.draft_id_hex,
        checkpoint_id_hex: binding.checkpoint_id_hex,
        verdict_kind: binding.verdict_kind,
        publication_state: binding.publication_state,
        binding: binding.binding,
        leaf_rows,
        proof_rows,
        process_verdicts: build_process_verdicts(spec),
    })
}

fn load_publication_route(
    spec: &RuntimeTraceSpec,
) -> Result<PublicationRouteSnapshotV1, Scenario1Err> {
    let bytes = io::read_file(&spec.route_table_path).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "failed to read route table {}: {err}",
            spec.route_table_path.display()
        ))
    })?;
    let text = String::from_utf8(bytes).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "route table {} must be UTF-8 lowercase hex: {err}",
            spec.route_table_path.display()
        ))
    })?;
    let text = text.trim();
    if text.is_empty() {
        return Err(Scenario1Err::Evidence(format!(
            "route table {} must not be empty",
            spec.route_table_path.display()
        )));
    }
    let canon = hex::decode(text).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "route table {} must stay lowercase hex: {err}",
            spec.route_table_path.display()
        ))
    })?;
    if hex::encode(&canon) != text {
        return Err(Scenario1Err::Evidence(format!(
            "route table {} must stay lowercase hex with canonical byte pairs",
            spec.route_table_path.display()
        )));
    }
    let table = ShardRouteTable::from_canon(&canon).map_err(|err| {
        Scenario1Err::Evidence(format!(
            "route table canonical bytes are invalid in {}: {err}",
            spec.route_table_path.display()
        ))
    })?;
    if table.canonical_bytes() != canon {
        return Err(Scenario1Err::Evidence(format!(
            "route table canonical re-encode changed bytes in {}",
            spec.route_table_path.display()
        )));
    }
    let digest_hex = hex::encode(table.digest().as_bytes());
    if digest_hex != spec.route_table_digest {
        return Err(Scenario1Err::Evidence(format!(
            "route table digest drift in {}: expected {}, got {}",
            spec.route_table_path.display(),
            spec.route_table_digest,
            digest_hex
        )));
    }
    Ok(PublicationRouteSnapshotV1::new(
        table.routing_generation,
        table.digest().into_bytes(),
        table.activation_checkpoint,
        table
            .shard_set
            .iter()
            .map(|shard_id| shard_id.as_u32())
            .collect(),
    ))
}

fn load_stage12_publication_binding(
    cfg: &ScenarioCfg,
    out_dir: &Path,
    route_digest: [u8; 32],
) -> Result<PublicationBindingEvidence, Scenario1Err> {
    let summary_path = stage8_checkpoint_path(cfg, out_dir);
    let summary: Stage12SummaryView = JsonCodec
        .deserialize(
            io::read_file(&summary_path)
                .map_err(|err| {
                    Scenario1Err::Evidence(format!(
                        "failed to read stage8 checkpoint summary {}: {err}",
                        summary_path.display()
                    ))
                })?
                .as_slice(),
        )
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "failed to decode stage8 checkpoint summary {}: {err}",
                summary_path.display()
            ))
        })?;
    let batch_id = BatchId::from_bytes(decode_hex32(&summary.draft_id_hex, "stage8 draft_id_hex")?);
    let tx_dir = summary_path.parent().ok_or_else(|| {
        Scenario1Err::Evidence(format!(
            "stage8 checkpoint summary {} missing parent directory",
            summary_path.display()
        ))
    })?;
    let store = CheckpointFsStore::new(tx_dir);

    match summary.status.as_str() {
        "ok" => {
            if summary.evidence_class != crate::config::STAGE12_FINAL_PUBLIC_EVIDENCE_CLASS {
                return Err(Scenario1Err::Evidence(format!(
                    "stage8 checkpoint summary {} must use {} before public publication evidence, got {}",
                    summary_path.display(),
                    crate::config::STAGE12_FINAL_PUBLIC_EVIDENCE_CLASS,
                    summary.evidence_class
                )));
            }
            let checkpoint_id_hex = summary.checkpoint_id_hex.clone().ok_or_else(|| {
                Scenario1Err::Evidence(format!(
                    "stage8 checkpoint summary {} missing checkpoint_id_hex",
                    summary_path.display()
                ))
            })?;
            let checkpoint_id = CheckpointId::new(decode_hex32(
                &checkpoint_id_hex,
                "stage8 checkpoint_id_hex",
            )?);
            let artifact = store.load_artifact(&checkpoint_id).map_err(|err| {
                Scenario1Err::Evidence(format!(
                    "failed to load stage8 artifact {} from {}: {err}",
                    checkpoint_id_hex,
                    tx_dir.display()
                ))
            })?;
            Ok(PublicationBindingEvidence {
                draft_id_hex: summary.draft_id_hex,
                checkpoint_id_hex,
                verdict_kind: "accepted".to_string(),
                publication_state: "accepted".to_string(),
                binding: bind_publication_contract(
                    batch_id,
                    checkpoint_id,
                    route_digest,
                    &artifact.pub_in(),
                ),
            })
        }
        "draft_only" => {
            let _ = store;
            let _ = route_digest;
            Err(Scenario1Err::Evidence(format!(
                "stage8 checkpoint summary {} is {} with {} and must stay off public publication evidence; synthetic checkpoint ids are forbidden",
                summary_path.display(),
                summary.status,
                summary.evidence_class
            )))
        }
        other => Err(Scenario1Err::Evidence(format!(
            "stage8 checkpoint summary {} has unsupported status {}",
            summary_path.display(),
            other
        ))),
    }
}

fn build_publication_leaf(
    spec: &RuntimeTraceSpec,
    store: &SettlementStore,
    source_path: &Stage13SourcePath,
    route_digest: [u8; 32],
    row: &PlacementRowView,
) -> Result<
    (
        ShardRootLeafV1,
        z00z_storage::settlement::ProofBlob,
        u64,
        [u8; 32],
    ),
    Scenario1Err,
> {
    let proof = store
        .settlement_proof_blob(&source_path.path)
        .map_err(storage_err(
            "build publication proof blob from persisted store",
        ))?;
    let recovery = store
        .recovery_state()
        .map_err(storage_err("load persisted shard recovery state"))?;
    let journal_checkpoint = proof.hjmt_journal_checkpoint().ok_or_else(|| {
        Scenario1Err::Evidence(format!(
            "publication shard {} missing journal checkpoint",
            row.shard_id
        ))
    })?;
    let policy_set = recovery.live_policy_set_v1(journal_checkpoint);
    let policy_set_digest = policy_set
        .digest()
        .map_err(proof_err("digest policy set"))?;
    Ok((
        ShardRootLeafV1::new(
            u32::from(row.shard_id),
            source_path.settlement_state_root.into_bytes(),
            recovery.version,
            spec.routing_generation,
            route_digest,
            policy_set_digest,
            journal_checkpoint,
            recovery.version,
            0,
        ),
        proof,
        u64::from(recovery.bucket_policy_generation),
        recovery.bucket_policy_id,
    ))
}

fn build_process_verdicts(spec: &RuntimeTraceSpec) -> Vec<PublicationProcessVerdictView> {
    spec.process_view
        .aggregators
        .iter()
        .map(|agg| {
            let secondary_shard_ids = spec
                .journal_view
                .lineage_rows
                .iter()
                .filter(|row| row.secondary_ids.contains(&agg.aggregator_id))
                .map(|row| row.shard_id)
                .collect::<Vec<_>>();
            PublicationProcessVerdictView {
                aggregator_id: agg.aggregator_id,
                process_id: format!("agg-{}", agg.aggregator_id),
                journal_path: agg.journal_path.clone(),
                owned_shard_ids: agg.shard_ids.clone(),
                secondary_shard_ids,
                exit_verdict: "config_bound_exit_ready".to_string(),
                restart_verdict: if agg.restart_cmd.trim().is_empty() {
                    "restart_cmd_missing".to_string()
                } else {
                    "config_bound_restart_ready".to_string()
                },
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct Stage13PublicationSource {
    prior_public_root: SettlementStateRoot,
    source_paths: Vec<Stage13SourcePath>,
}

#[derive(Debug, Clone)]
struct Stage13SourcePath {
    path_text: String,
    path: SettlementPath,
    settlement_state_root: SettlementStateRoot,
}

fn stage13_publication_source(out_dir: &Path) -> Result<Stage13PublicationSource, Scenario1Err> {
    let report = load_stage13_examples_report(out_dir)?;
    let mut seen = BTreeSet::new();
    let mut source_paths = Vec::new();

    for row in &report.examples {
        let Some(path_text) = row.settlement_path.as_deref() else {
            continue;
        };
        let normalized = path_text.trim();
        if normalized.is_empty() || normalized == "none" || !seen.insert(normalized.to_string()) {
            continue;
        }
        source_paths.push(Stage13SourcePath {
            path_text: normalized.to_string(),
            path: parse_settlement_path(normalized)?,
            settlement_state_root: SettlementStateRoot::settlement_v1(decode_hex32(
                &row.settlement_state_root_hex,
                "stage13 example settlement_state_root_hex",
            )?),
        });
    }

    for row in &report.comparison_rows {
        let settlement_state_root = SettlementStateRoot::settlement_v1(decode_hex32(
            &row.settlement_state_root_hex,
            "stage13 comparison settlement_state_root_hex",
        )?);
        for path_text in &row.settlement_paths {
            let normalized = path_text.trim();
            if normalized.is_empty() || normalized == "none" || !seen.insert(normalized.to_string())
            {
                continue;
            }
            source_paths.push(Stage13SourcePath {
                path_text: normalized.to_string(),
                path: parse_settlement_path(normalized)?,
                settlement_state_root,
            });
        }
    }

    Ok(Stage13PublicationSource {
        prior_public_root: SettlementStateRoot::settlement_v1(decode_hex32(
            &report.settlement_state_root_hex,
            "settlement_state_root_hex",
        )?),
        source_paths,
    })
}

fn load_stage13_examples_report(out_dir: &Path) -> Result<Stage13ExamplesRoot, Scenario1Err> {
    let path = resolve_path(out_dir, "hjmt/hjmt_settlement_examples.json");
    JsonCodec
        .deserialize(&io::read_file(&path).map_err(|err| {
            Scenario1Err::Evidence(format!(
                "failed to read stage13 examples report {}: {err}",
                path.display()
            ))
        })?)
        .map_err(|err| {
            Scenario1Err::Evidence(format!(
                "failed to decode stage13 examples report {}: {err}",
                path.display()
            ))
        })
}

fn parse_settlement_path(value: &str) -> Result<SettlementPath, Scenario1Err> {
    let mut parts = value.split('/');
    let definition_hex = parts.next().ok_or_else(|| {
        Scenario1Err::Evidence(format!(
            "stage13 settlement_path missing definition id: {value}"
        ))
    })?;
    let serial_text = parts.next().ok_or_else(|| {
        Scenario1Err::Evidence(format!(
            "stage13 settlement_path missing serial id: {value}"
        ))
    })?;
    let terminal_hex = parts.next().ok_or_else(|| {
        Scenario1Err::Evidence(format!(
            "stage13 settlement_path missing terminal id: {value}"
        ))
    })?;
    if parts.next().is_some() {
        return Err(Scenario1Err::Evidence(format!(
            "stage13 settlement_path has extra segments: {value}"
        )));
    }

    let path = SettlementPath::new(
        DefinitionId::new(decode_hex32(
            definition_hex,
            "settlement_path definition_id",
        )?),
        SerialId::new(serial_text.parse().map_err(|err| {
            Scenario1Err::Evidence(format!(
                "stage13 settlement_path serial id is invalid in {value}: {err}"
            ))
        })?),
        TerminalId::new(decode_hex32(terminal_hex, "settlement_path terminal_id")?),
    );
    path.check().map_err(|err| {
        Scenario1Err::Evidence(format!(
            "stage13 settlement_path failed validation in {value}: {err}"
        ))
    })?;
    Ok(path)
}

fn spec_policy_set(
    leaf: ShardRootLeafV1,
    policy_generation: u64,
    bucket_policy_digest: [u8; 32],
) -> z00z_storage::settlement::PolicySetCommitmentV1 {
    z00z_storage::settlement::PolicySetCommitmentV1::new(vec![
        z00z_storage::settlement::PolicySetMemberV1::new(
            policy_generation,
            bucket_policy_digest,
            leaf.journal_checkpoint,
            None,
        ),
    ])
}

fn family_tag(family: HjmtProofFamily) -> String {
    format!("{family:?}").to_lowercase()
}

fn leaf_family_tag(family: SettlementLeafFamily) -> String {
    format!("{family:?}").to_lowercase()
}

fn decode_hex32(value: &str, field: &str) -> Result<[u8; 32], Scenario1Err> {
    let bytes = hex::decode(value)
        .map_err(|err| Scenario1Err::Evidence(format!("failed to decode {field} as hex: {err}")))?;
    bytes
        .try_into()
        .map_err(|_| Scenario1Err::Evidence(format!("{field} must decode to exactly 32 bytes")))
}

fn storage_err(
    action: &'static str,
) -> impl FnOnce(z00z_storage::settlement::SettlementStoreError) -> Scenario1Err {
    move |err| Scenario1Err::Evidence(format!("{action} failed: {err}"))
}

fn proof_err(
    action: &'static str,
) -> impl FnOnce(z00z_storage::settlement::ProofChkErr) -> Scenario1Err {
    move |err| Scenario1Err::Evidence(format!("{action} failed: {err}"))
}

fn check_string(
    value: &Value,
    field: &str,
    expected: &str,
    path: &Path,
) -> Result<(), Scenario1Err> {
    let actual = value[field].as_str().ok_or_else(|| {
        Scenario1Err::Evidence(format!("{} missing string field {}", path.display(), field))
    })?;
    if actual != expected {
        return Err(Scenario1Err::Evidence(format!(
            "{} field {} drifted: expected {}, got {}",
            path.display(),
            field,
            expected,
            actual
        )));
    }
    Ok(())
}

fn check_u64(value: &Value, field: &str, expected: u64, path: &Path) -> Result<(), Scenario1Err> {
    let actual = value[field].as_u64().ok_or_else(|| {
        Scenario1Err::Evidence(format!("{} missing u64 field {}", path.display(), field))
    })?;
    if actual != expected {
        return Err(Scenario1Err::Evidence(format!(
            "{} field {} drifted: expected {}, got {}",
            path.display(),
            field,
            expected,
            actual
        )));
    }
    Ok(())
}

fn check_bool(value: &Value, field: &str, expected: bool, path: &Path) -> Result<(), Scenario1Err> {
    let actual = value[field].as_bool().ok_or_else(|| {
        Scenario1Err::Evidence(format!("{} missing bool field {}", path.display(), field))
    })?;
    if actual != expected {
        return Err(Scenario1Err::Evidence(format!(
            "{} field {} drifted: expected {}, got {}",
            path.display(),
            field,
            expected,
            actual
        )));
    }
    Ok(())
}

fn check_path(
    value: &Value,
    field: &str,
    expected: &Path,
    path: &Path,
) -> Result<(), Scenario1Err> {
    let actual = value[field].as_str().ok_or_else(|| {
        Scenario1Err::Evidence(format!("{} missing path field {}", path.display(), field))
    })?;
    let expected = expected.to_string_lossy();
    if actual != expected {
        return Err(Scenario1Err::Evidence(format!(
            "{} field {} drifted: expected {}, got {}",
            path.display(),
            field,
            expected,
            actual
        )));
    }
    Ok(())
}

fn json_value(value: &impl Serialize) -> Result<Value, Scenario1Err> {
    let bytes = JsonCodec.serialize(value).map_err(|err| {
        Scenario1Err::Evidence(format!("failed to encode runtime trace payload: {err}"))
    })?;
    JsonCodec.deserialize(&bytes).map_err(|err| {
        Scenario1Err::Evidence(format!("failed to decode runtime trace payload: {err}"))
    })
}

fn check_trace_payload(actual: &Value, expected: &Value, path: &Path) -> Result<(), Scenario1Err> {
    if actual == expected {
        return Ok(());
    }

    let drift = drift_field_paths(actual, expected);
    let detail = if drift.is_empty() {
        "canonical payload drifted".to_string()
    } else {
        let preview = drift.iter().take(8).cloned().collect::<Vec<_>>();
        let suffix = if drift.len() > preview.len() {
            format!(" (+{} more)", drift.len() - preview.len())
        } else {
            String::new()
        };
        format!("drift fields: {}{}", preview.join(", "), suffix)
    };

    Err(Scenario1Err::Evidence(format!(
        "{} runtime trace payload drifted: {}",
        path.display(),
        detail
    )))
}

fn drift_field_paths(actual: &Value, expected: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    collect_drift_field_paths(Some(actual), Some(expected), "", &mut paths);
    paths
}

fn collect_drift_field_paths(
    actual: Option<&Value>,
    expected: Option<&Value>,
    prefix: &str,
    paths: &mut Vec<String>,
) {
    if actual == expected {
        return;
    }

    match (actual, expected) {
        (Some(Value::Object(actual)), Some(Value::Object(expected))) => {
            let mut keys = BTreeSet::new();
            keys.extend(actual.keys().cloned());
            keys.extend(expected.keys().cloned());
            for key in keys {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_drift_field_paths(actual.get(&key), expected.get(&key), &next, paths);
            }
        }
        (Some(Value::Array(actual)), Some(Value::Array(expected))) => {
            let max_len = actual.len().max(expected.len());
            for index in 0..max_len {
                let next = if prefix.is_empty() {
                    format!("[{index}]")
                } else {
                    format!("{prefix}[{index}]")
                };
                collect_drift_field_paths(actual.get(index), expected.get(index), &next, paths);
            }
        }
        _ => paths.push(if prefix.is_empty() {
            "$".to_string()
        } else {
            prefix.to_string()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_process_view, build_wallet_lifecycle_rows, REQUIRED_WALLET_LIFECYCLE_CASES};
    use std::path::Path;
    use z00z_rollup_node::NodeConfig;
    use z00z_utils::codec::{Codec, JsonCodec};

    #[test]
    fn reject_shadow_cfg_paths() {
        let home = super::repo_root().join("config/hjmt_runtime/sim_5a7s");
        let cfg = NodeConfig::from_hjmt_home(home).expect("load repo SIM-5A7S home");
        let mut hjmt = cfg.hjmt.expect("hjmt config");
        hjmt.aggs[0].lifecycle.start_cmd = "cargo run --release -p z00z_rollup_node -- --mode aggregator --aggregator-config shadow/agg-0/aggregator-config.yaml --planner-config shadow/planner/planner-config.yaml --storage-config shadow/storage/storage-config.yaml".to_string();

        match build_process_view(&hjmt) {
            Ok(_) => panic!("shadow lifecycle path must reject"),
            Err(err) => assert!(err
                .to_string()
                .contains("must reference canonical aggregator/planner/storage config paths")),
        }
    }

    #[test]
    fn wallet_rows_cover_cases() {
        let rows = build_wallet_lifecycle_rows(
            Path::new("/tmp/z00z_wallet_rows_cover_cases"),
            "wallet_scan_digest",
            "publication_digest",
        )
        .expect("wallet lifecycle rows");

        assert_eq!(rows.len(), REQUIRED_WALLET_LIFECYCLE_CASES.len());
        let failing = rows
            .iter()
            .filter(|row| !row.restart_verification_passed)
            .map(|row| row.case_id.clone())
            .collect::<Vec<_>>();
        assert!(
            failing.is_empty(),
            "restart verification failed for cases: {:?}",
            failing
        );
    }

    #[test]
    fn wallet_lifecycle_rows_are_deterministic() {
        let left = build_wallet_lifecycle_rows(
            Path::new("/tmp/z00z_wallet_lifecycle_rows_are_deterministic"),
            "wallet_scan_digest",
            "publication_digest",
        )
        .expect("left rows");
        let right = build_wallet_lifecycle_rows(
            Path::new("/tmp/z00z_wallet_lifecycle_rows_are_deterministic"),
            "wallet_scan_digest",
            "publication_digest",
        )
        .expect("right rows");

        let left_json = JsonCodec.serialize(&left).expect("left json");
        let right_json = JsonCodec.serialize(&right).expect("right json");
        assert_eq!(left_json, right_json);
    }
}
