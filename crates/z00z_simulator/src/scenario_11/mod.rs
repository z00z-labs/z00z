#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
};

use sha2::{Digest, Sha256};
use thiserror::Error;
use z00z_aggregators::{
    bind_publication_contract, membership_digest_for_voters, persist_consensus_commit,
    persist_consensus_publication, persist_validator_decision, publication_record_for_published,
    validator_decision_snapshot, AggregatorId, BatchId, BatchPlanner, BatchRoute, CommitSubject,
    ConsensusAdapter, ConsensusStore, DispatchStage, DistDispatch, DistSim, EvidenceRecord,
    FrameStage, InMemoryVoteTransport, JournalCandidate, JournalFrame, MissingBlobEvidence,
    OrderedBatch, PlanDigest, PlannerAuthority, PublicationBinding, PublicationRequest,
    PublicationState, RecoveryBoundary, RecoveryIntent, ReplayVerifiedVoteService,
    SecondaryReplayRejectCode, SecondaryReplayRequest, SecondaryState, ShardExecState,
    ShardExecTicket, ShardExecutor, ShardPlacement, ShardPlacementTable, ShardQuorumCertificate,
    ShardRecoveryRecord, ShardRouteTable, ShardVote, ShardVoteKind, ShardVoteRole,
    SplitBrainEvidence, StaleMemberEvidence, TransportDeliveryPlan, VoteEvidenceTracker,
    VoteExchangeContext, VoteExchangeOutcome, VoteTransport, VoteTransportEnvelope, WorkItem,
    WorkPayload, WrongRootEvidence, WrongRouteDigestEvidence, CONSENSUS_STORE_BACKEND,
    CONSENSUS_STORE_SCHEMA_VERSION,
};
use z00z_core::assets::{Asset, AssetClass, AssetDefinition, AssetPkgWire, AssetWire};
use z00z_crypto::ZkPackEncrypted;
use z00z_rollup_node::{
    preview_publication_contract_parts, CelestiaLocalAdapter, DaAdapter, DaError, NodeCfgErr,
    NodeConfig,
};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, CheckpointArtifact,
        CheckpointDaProviderFamily, CheckpointDraft, CheckpointExecInput, CheckpointExecInputId,
        CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion, CheckpointInRef,
        CheckpointLink, CheckpointLinkVersion, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{
        CheckRoot, ClaimNullifier, PublicationRouteSnapshotV1, SettlementRecoveryState,
        SettlementRouteCtx, SettlementStateRoot, SettlementStore, StoreItem, StoreOp,
    },
    snapshot::PrepSnapshotId,
};
use z00z_utils::{
    codec::Codec,
    io::{create_dir_all, read_file, remove_dir_all, save_json},
};
use z00z_validators::{
    ObjectPolicyRegistryV1, ResolvedBatch, SettlementError, SettlementTheoremBundle,
    ValidatorBoundary, Verdict, VerdictKind,
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::{bind_stealth_output_wire, build_card_stealth_leaf},
    tx::{
        asset_wire_to_leaf, build_public_spend_contract, build_tx_package_digest,
        prepare_spend_membership_witnesses, prepare_spend_public_inputs, resolve_input_pack,
        verify_package_public_spend_contract, ClaimAuthWire, ClaimContextWire, ClaimProofWire,
        ClaimTxPackage, ClaimTxWire, SpendProofWitness, TxAuthWire, TxContextWire, TxInputWire,
        TxOutRole, TxOutputWire, TxPackage, TxProofWire, TxVerifierImpl, TxWire,
    },
};

pub mod report;

pub use report::{
    ClaimLevelReport, CommitSubjectReport, ConsensusStoreReport, DualPrimaryIsolationReport,
    EvidenceEntryReport, EvidenceRegistryReport, FaultMatrixEntry, FaultMatrixReport,
    LocalDaBindingReport, PackageIngressReport, PlacementMembershipCaseReport,
    PlacementMembershipReport, PlannerAuthorityReplicaReport, QuorumCertificateCaseReport,
    QuorumCertificateReport, ReportHonesty, RoutePlanCaseReport, RoutePlanReport,
    SecondaryReplayVoteReport, SecondaryReplayVotesReport, ValidatorVerdictReport,
    CLAIM_LEVEL_LIVE, CLAIM_LEVEL_LIVE_CLAIM_REMOVED, CLAIM_LEVEL_SIMULATED_FULL,
    PLANNER_AUTHORITY_MODEL_DETERMINISTIC_REPLICATED, TERM_CELESTIA_FINALITY,
    TERM_DETERMINISTIC_REPLICATED_PLANNER, TERM_PLANNER_HA,
};

const SIM_5A7S_HOME: &str = "config/hjmt_runtime/sim_5a7s";
const SNAPSHOT_ID: PrepSnapshotId = PrepSnapshotId::new([0x44; 32]);
const RECEIVER_SECRET: [u8; 32] = [0x11; 32];

#[derive(Debug, Error)]
pub enum Scenario11Error {
    #[error(transparent)]
    Io(#[from] z00z_utils::io::IoError),
    #[error(transparent)]
    Config(#[from] NodeCfgErr),
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    #[error(transparent)]
    Da(#[from] DaError),
    #[error(transparent)]
    Theorem(#[from] SettlementError),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone)]
pub struct Scenario11Run {
    artifact_root: PathBuf,
}

impl Scenario11Run {
    #[must_use]
    pub fn artifact_root(&self) -> &Path {
        &self.artifact_root
    }
}

fn claim_level(term: &str, claim_level: &str, evidence_refs: &[&str]) -> ClaimLevelReport {
    ClaimLevelReport {
        term: term.to_string(),
        claim_level: claim_level.to_string(),
        evidence_refs: evidence_refs
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
    }
}

fn report_claim_levels() -> Vec<ClaimLevelReport> {
    vec![
        claim_level(
            "Active member",
            CLAIM_LEVEL_LIVE,
            &["quorum_certificate.json", "secondary_replay_votes.json"],
        ),
        claim_level(
            "Aggregator",
            CLAIM_LEVEL_LIVE,
            &["route_plan_report.json", "commit_subject.json"],
        ),
        claim_level(
            "Anti-equivocation evidence",
            CLAIM_LEVEL_LIVE,
            &["evidence_registry.json", "fault_matrix.json"],
        ),
        claim_level(
            "Availability certificate",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "Batch",
            CLAIM_LEVEL_LIVE,
            &["package_ingress_report.json", "commit_subject.json"],
        ),
        claim_level(
            "`BatchPayload`",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "BFT",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["quorum_certificate.json", "report_honesty.json"],
        ),
        claim_level(
            "CFT",
            CLAIM_LEVEL_LIVE,
            &["quorum_certificate.json", "report_honesty.json"],
        ),
        claim_level(
            "Celestia",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["local_da_binding.json", "report_honesty.json"],
        ),
        claim_level(
            TERM_CELESTIA_FINALITY,
            CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
            &["local_da_binding.json", "report_honesty.json"],
        ),
        claim_level(
            "Commit certificate",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["quorum_certificate.json", "report_honesty.json"],
        ),
        claim_level(
            "`CommitSubject`",
            CLAIM_LEVEL_LIVE,
            &["commit_subject.json", "quorum_certificate.json"],
        ),
        claim_level(
            "Consensus adapter",
            CLAIM_LEVEL_LIVE,
            &["commit_subject.json", "quorum_certificate.json"],
        ),
        claim_level(
            "DA",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "DA adapter",
            CLAIM_LEVEL_LIVE,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "DA-before-ordering",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "Deterministic simulator signature",
            CLAIM_LEVEL_LIVE,
            &["secondary_replay_votes.json", "evidence_registry.json"],
        ),
        claim_level(
            "Epoch",
            CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
            &["route_plan_report.json", "report_honesty.json"],
        ),
        claim_level(
            "Evidence",
            CLAIM_LEVEL_LIVE,
            &["evidence_registry.json", "fault_matrix.json"],
        ),
        claim_level(
            "Failover",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["consensus_store_report.json", "fault_matrix.json"],
        ),
        claim_level(
            "HotStuff",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["fault_matrix.json", "report_honesty.json"],
        ),
        claim_level(
            "Ingress",
            CLAIM_LEVEL_LIVE,
            &["package_ingress_report.json", "commit_subject.json"],
        ),
        claim_level(
            "`JournalCandidate`",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["commit_subject.json", "consensus_store_report.json"],
        ),
        claim_level(
            "`LocalDaAdapter`",
            CLAIM_LEVEL_LIVE,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "Local quorum certificate",
            CLAIM_LEVEL_LIVE,
            &["quorum_certificate.json", "commit_subject.json"],
        ),
        claim_level(
            "Membership digest",
            CLAIM_LEVEL_LIVE,
            &["commit_subject.json", "quorum_certificate.json"],
        ),
        claim_level(
            "Mixed-generation commit",
            CLAIM_LEVEL_LIVE,
            &["fault_matrix.json", "secondary_replay_votes.json"],
        ),
        claim_level(
            "Placement generation",
            CLAIM_LEVEL_LIVE,
            &["placement_membership.json", "route_plan_report.json"],
        ),
        claim_level(
            "Primary aggregator",
            CLAIM_LEVEL_LIVE,
            &["route_plan_report.json", "quorum_certificate.json"],
        ),
        claim_level(
            "Production signature",
            CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
            &["secondary_replay_votes.json", "report_honesty.json"],
        ),
        claim_level(
            "Publication binding",
            CLAIM_LEVEL_LIVE,
            &["local_da_binding.json", "validator_verdict_report.json"],
        ),
        claim_level(
            "Quorum",
            CLAIM_LEVEL_LIVE,
            &["quorum_certificate.json", "report_honesty.json"],
        ),
        claim_level(
            "Quorum group",
            CLAIM_LEVEL_LIVE,
            &["placement_membership.json", "quorum_certificate.json"],
        ),
        claim_level(
            "Recovery boundary",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["consensus_store_report.json", "fault_matrix.json"],
        ),
        claim_level(
            "Route table digest",
            CLAIM_LEVEL_LIVE,
            &["route_plan_report.json", "commit_subject.json"],
        ),
        claim_level(
            "Routing generation",
            CLAIM_LEVEL_LIVE,
            &["route_plan_report.json", "commit_subject.json"],
        ),
        claim_level(
            "Runtime validator",
            CLAIM_LEVEL_LIVE,
            &["validator_verdict_report.json", "local_da_binding.json"],
        ),
        claim_level(
            "Secondary replay",
            CLAIM_LEVEL_LIVE,
            &["secondary_replay_votes.json", "fault_matrix.json"],
        ),
        claim_level(
            "Secondary aggregator",
            CLAIM_LEVEL_LIVE,
            &["secondary_replay_votes.json", "fault_matrix.json"],
        ),
        claim_level(
            "Shard",
            CLAIM_LEVEL_LIVE,
            &["commit_subject.json", "quorum_certificate.json"],
        ),
        claim_level(
            "`ShardBatchHeader`",
            CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
            &["commit_subject.json", "report_honesty.json"],
        ),
        claim_level(
            "`ShardQuorumCertificate`",
            CLAIM_LEVEL_LIVE,
            &["quorum_certificate.json", "commit_subject.json"],
        ),
        claim_level(
            "`ShardVote`",
            CLAIM_LEVEL_LIVE,
            &["secondary_replay_votes.json", "quorum_certificate.json"],
        ),
        claim_level(
            "`sim_5a7s`",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["route_plan_report.json", "fault_matrix.json"],
        ),
        claim_level(
            "Split brain",
            CLAIM_LEVEL_LIVE,
            &["fault_matrix.json", "evidence_registry.json"],
        ),
        claim_level(
            "Stale secondary aggregator",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["secondary_replay_votes.json", "fault_matrix.json"],
        ),
        claim_level(
            "Term",
            CLAIM_LEVEL_LIVE,
            &["commit_subject.json", "quorum_certificate.json"],
        ),
        claim_level(
            "Theorem bundle",
            CLAIM_LEVEL_LIVE,
            &["validator_verdict_report.json", "local_da_binding.json"],
        ),
        claim_level(
            "Validator boundary",
            CLAIM_LEVEL_LIVE,
            &["validator_verdict_report.json", "local_da_binding.json"],
        ),
        claim_level(
            "Vote",
            CLAIM_LEVEL_LIVE,
            &["secondary_replay_votes.json", "quorum_certificate.json"],
        ),
        claim_level(
            "Watcher boundary",
            CLAIM_LEVEL_LIVE,
            &[
                "crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs::evidence_keeps_publication_story",
                "crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs::watcher_rejects_binding_drift",
            ],
        ),
        claim_level(
            TERM_DETERMINISTIC_REPLICATED_PLANNER,
            CLAIM_LEVEL_LIVE,
            &["route_plan_report.json", "report_honesty.json"],
        ),
        claim_level(
            TERM_PLANNER_HA,
            CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
            &["route_plan_report.json", "report_honesty.json"],
        ),
        claim_level(
            "devnet",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["fault_matrix.json", "report_honesty.json"],
        ),
        claim_level(
            "Transport",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["fault_matrix.json", "secondary_replay_votes.json"],
        ),
        claim_level(
            "Validator certificate gate",
            CLAIM_LEVEL_LIVE,
            &["validator_verdict_report.json", "report_honesty.json"],
        ),
        claim_level(
            "Production signature seam",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["secondary_replay_votes.json", "report_honesty.json"],
        ),
        claim_level(
            "Slashing/economics",
            CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
            &["evidence_registry.json", "report_honesty.json"],
        ),
        claim_level(
            "Glossary terms",
            CLAIM_LEVEL_LIVE,
            &["067-GLOSSARY-CLAIMS.md", "report_honesty.json"],
        ),
        claim_level(
            "BFT committee",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["quorum_certificate.json", "report_honesty.json"],
        ),
        claim_level(
            "Failover/takeover",
            CLAIM_LEVEL_SIMULATED_FULL,
            &["consensus_store_report.json", "fault_matrix.json"],
        ),
    ]
}

pub fn run(output_root: &Path) -> Result<Scenario11Run, Scenario11Error> {
    let artifact_root = output_root.join("scenario_11").join("quorum");
    create_dir_all(&artifact_root)?;

    let topology = LiveTopology::load()?;
    let happy = run_happy_path(&topology, &artifact_root)?;
    let sweep = run_all_shard_sweep(&topology)?;
    let dual_primary = run_dual_primary_isolation(&topology, happy.theorem_digest)?;
    let fault_outcome = run_fault_matrix(&topology, &happy)?;
    let planner_mode = topology.planner_mode()?;
    let planner_cfg_digest = topology.planner_cfg_digest()?;
    let planner_authority =
        PlannerAuthority::bind(planner_mode, &topology.route_table, planner_cfg_digest);
    let authority_replicas =
        planner_authority_replicas(&topology, &happy.ordered, planner_authority)?;

    save_json(
        artifact_root.join("package_ingress_report.json"),
        &PackageIngressReport {
            package_kind: "TxPackage".to_string(),
            package_digest_hex: happy.package_digest_hex.clone(),
            route_key_hex: hex::encode(route_key(&happy.ordered.items[0])?),
            batch_id_hex: hex::encode(happy.batch_id.into_bytes()),
            shard_id: happy.ordered.planned.route.shard_id.as_u16(),
            routing_generation: happy.ordered.planned.route.routing_generation,
            planner_route_table_digest_hex: hex::encode(
                happy.ordered.planned.route_table_digest.as_bytes(),
            ),
            ingress_recomputed_digest: true,
        },
    )?;

    save_json(
        artifact_root.join("route_plan_report.json"),
        &RoutePlanReport {
            planner_mode: planner_mode.as_str().to_string(),
            planner_authority_model: PLANNER_AUTHORITY_MODEL_DETERMINISTIC_REPLICATED.to_string(),
            planner_config_digest_hex: hex::encode(planner_cfg_digest.as_bytes()),
            planner_authority_digest_hex: hex::encode(planner_authority.digest().as_bytes()),
            planner_ha_claim_level: CLAIM_LEVEL_LIVE_CLAIM_REMOVED.to_string(),
            route_table_digest_hex: hex::encode(topology.route_table.digest().as_bytes()),
            authority_replicas,
            happy_path: RoutePlanCaseReport {
                case_id: "happy_path".to_string(),
                batch_id_hex: hex::encode(happy.batch_id.into_bytes()),
                shard_id: happy.ordered.planned.route.shard_id.as_u16(),
                routing_generation: happy.ordered.planned.route.routing_generation,
                route_table_digest_hex: hex::encode(
                    happy.ordered.planned.route_table_digest.as_bytes(),
                ),
                plan_digest_hex: hex::encode(happy.ordered.planned.plan_digest.as_bytes()),
                dispatch_owner_id: happy.dispatch_owner_id.as_u16(),
                dispatch_stage: dispatch_stage_name(happy.dispatch_stage).to_string(),
            },
            all_shard_sweep: sweep
                .iter()
                .map(|row| RoutePlanCaseReport {
                    case_id: format!("shard_{}", row.shard_id),
                    batch_id_hex: hex::encode(row.batch_id.into_bytes()),
                    shard_id: row.shard_id,
                    routing_generation: row.routing_generation,
                    route_table_digest_hex: row.route_table_digest_hex.clone(),
                    plan_digest_hex: row.plan_digest_hex.clone(),
                    dispatch_owner_id: row.dispatch_owner_id,
                    dispatch_stage: row.dispatch_stage.clone(),
                })
                .collect(),
            dual_primary_owner: DualPrimaryIsolationReport {
                owner_id: dual_primary.owner_id,
                shard_ids: dual_primary.shard_ids.clone(),
                membership_digests_hex: dual_primary.membership_digests_hex.clone(),
                certificate_digests_hex: dual_primary.certificate_digests_hex.clone(),
            },
        },
    )?;

    save_json(
        artifact_root.join("placement_membership.json"),
        &PlacementMembershipReport {
            happy_path: PlacementMembershipCaseReport {
                shard_id: happy.placement.route.shard_id.as_u16(),
                routing_generation: happy.placement.route.routing_generation,
                primary_id: happy.placement.primary_id.as_u16(),
                secondary_ids: secondary_ids(&happy.placement.secondaries),
                ready_secondary_ids: ready_secondary_ids(&happy.placement.secondaries),
                quorum_threshold: quorum_threshold(&happy.placement),
                membership_digest_hex: hex::encode(happy.subject.membership_digest),
                expected_journal_lineage_hex: hex::encode(happy.placement.expected_journal_lineage),
            },
            all_shard_sweep: sweep
                .iter()
                .map(|row| PlacementMembershipCaseReport {
                    shard_id: row.shard_id,
                    routing_generation: row.routing_generation,
                    primary_id: row.dispatch_owner_id,
                    secondary_ids: row.secondary_ids.clone(),
                    ready_secondary_ids: row.secondary_ids.clone(),
                    quorum_threshold: 2,
                    membership_digest_hex: row.membership_digest_hex.clone(),
                    expected_journal_lineage_hex: row.expected_journal_lineage_hex.clone(),
                })
                .collect(),
        },
    )?;

    save_json(
        artifact_root.join("commit_subject.json"),
        &CommitSubjectReport {
            subject_digest_hex: hex::encode(happy.subject.digest()),
            term: happy.subject.term,
            batch_id_hex: hex::encode(happy.subject.batch_id.into_bytes()),
            shard_id: happy.subject.shard_id.as_u16(),
            routing_generation: happy.subject.routing_generation,
            plan_digest_hex: hex::encode(happy.subject.plan_digest),
            route_table_digest_hex: hex::encode(happy.subject.route_table_digest),
            membership_digest_hex: hex::encode(happy.subject.membership_digest),
            previous_state_root_hex: hex::encode(happy.subject.previous_state_root.into_bytes()),
            new_state_root_hex: hex::encode(happy.subject.new_state_root.into_bytes()),
            journal_lineage_hex: hex::encode(happy.subject.journal_lineage),
            proof_version: happy.subject.proof_version,
            theorem_digest_hex: hex::encode(happy.subject.theorem_or_settlement_digest),
            publication_binding_digest_hex: hex::encode(happy.subject.publication_binding_digest),
        },
    )?;

    save_json(
        artifact_root.join("secondary_replay_votes.json"),
        &SecondaryReplayVotesReport {
            happy_path_votes: happy.happy_votes.clone(),
            offline_case_votes: happy.offline_votes.clone(),
            stale_case_votes: happy.stale_votes.clone(),
            drift_case_votes: fault_outcome
                .faults
                .iter()
                .filter_map(|entry| entry.vote.as_ref().cloned())
                .collect(),
        },
    )?;

    save_json(
        artifact_root.join("quorum_certificate.json"),
        &QuorumCertificateReport {
            happy_path: QuorumCertificateCaseReport {
                case_id: "happy_path".to_string(),
                shard_id: happy.commit.certificate.shard_id.as_u16(),
                routing_generation: happy.commit.certificate.routing_generation,
                quorum_threshold: quorum_threshold(&happy.placement),
                membership_digest_hex: hex::encode(happy.commit.certificate.membership_digest),
                subject_digest_hex: hex::encode(happy.commit.certificate.subject_digest),
                certificate_digest_hex: hex::encode(happy.commit.certificate.digest()),
                voter_ids: happy
                    .commit
                    .certificate
                    .votes
                    .iter()
                    .map(|vote| vote.voter_id.as_u16())
                    .collect(),
            },
            dual_primary_cases: dual_primary
                .cases
                .iter()
                .map(|case| QuorumCertificateCaseReport {
                    case_id: case.case_id.clone(),
                    shard_id: case.shard_id,
                    routing_generation: case.routing_generation,
                    quorum_threshold: 2,
                    membership_digest_hex: case.membership_digest_hex.clone(),
                    subject_digest_hex: case.subject_digest_hex.clone(),
                    certificate_digest_hex: case.certificate_digest_hex.clone(),
                    voter_ids: case.voter_ids.clone(),
                })
                .collect(),
        },
    )?;

    save_json(
        artifact_root.join("local_da_binding.json"),
        &LocalDaBindingReport {
            batch_id_hex: hex::encode(happy.published.batch_id.into_bytes()),
            checkpoint_id_hex: hex::encode(happy.published.checkpoint_id.into_bytes()),
            publication_checkpoint: happy.published.publication_checkpoint,
            publication_route_digest_hex: hex::encode(
                happy.published.publication_route.route_table_digest,
            ),
            publication_shard_ids: happy.published.publication_route.shard_ids.clone(),
            publication_binding_digest_hex: hex::encode(happy.publication_binding.binding_digest()),
            blob_ref: happy.published.blob_ref.clone(),
            provider: happy.published.da_provider.clone(),
            claim_level: CLAIM_LEVEL_SIMULATED_FULL.to_string(),
            namespace_hex: happy.celestia_record.namespace_hex.clone(),
            blob_commitment_hex: hex::encode(happy.celestia_record.blob_commitment),
            inclusion_reference: happy.celestia_record.inclusion_reference.clone(),
            blob_height: happy.celestia_record.blob_height,
            retention_until_height: happy.celestia_record.retention_until_height,
            degraded_mode: happy.celestia_record.degraded_mode,
            payload_available: happy.celestia_record.payload_available,
            certificate_digest_hex: hex::encode(happy.commit.certificate.digest()),
            resumed_by_secondary_id: happy.resumed_by_secondary_id.as_u16(),
            resumed_same_certificate: happy.resumed_same_certificate,
        },
    )?;

    save_json(
        artifact_root.join("consensus_store_report.json"),
        &happy.consensus_store,
    )?;

    save_json(
        artifact_root.join("validator_verdict_report.json"),
        &ValidatorVerdictReport {
            verdict_kind: verdict_kind_name(&happy.verdict.kind).to_string(),
            reject_class: happy
                .verdict
                .reject
                .as_ref()
                .map(|reject| format!("{reject:?}")),
            checkpoint_id_hex: happy
                .verdict
                .checkpoint_id
                .map(|id| hex::encode(id.into_bytes())),
            publication_binding_digest_hex: happy
                .verdict
                .publication
                .as_ref()
                .map(|binding| hex::encode(binding.binding_digest())),
            theorem_digest_hex: hex::encode(happy.theorem_digest),
            batch_id_hex: hex::encode(happy.batch_id.into_bytes()),
            subject_digest_hex: hex::encode(happy.subject.digest()),
            certificate_digest_hex: hex::encode(happy.commit.certificate.digest()),
        },
    )?;

    save_json(
        artifact_root.join("fault_matrix.json"),
        &FaultMatrixReport {
            entries: fault_outcome
                .faults
                .iter()
                .cloned()
                .map(|entry| entry.entry)
                .collect(),
        },
    )?;

    save_json(
        artifact_root.join("evidence_registry.json"),
        &fault_outcome.evidence_registry.report(),
    )?;

    save_json(
        artifact_root.join("report_honesty.json"),
        &ReportHonesty {
            supported_claims: vec![
                "local per-shard 2-of-3 CFT quorum is proven".to_string(),
                "planner truth is deterministic replicated local computation over canonical planner config and route-table digest".to_string(),
                "secondary replay uses live ingress, planner, placement, recovery, and subject builders".to_string(),
                "local DA publish and resolve preserve the live route snapshot carried by PublicationRequest".to_string(),
                "Celestia-local artifact contract is simulated-full over live publication and validator bindings".to_string(),
                "validator verdict is produced from the live theorem and publication path".to_string(),
            ],
            forbidden_claims: vec![
                "network BFT".to_string(),
                TERM_CELESTIA_FINALITY.to_string(),
                "production HotStuff".to_string(),
                TERM_PLANNER_HA.to_string(),
                "unqualified devnet".to_string(),
                "production signatures".to_string(),
                "slashing".to_string(),
                "public finality".to_string(),
            ],
            deferred_claims: vec![
                "network, production-signature, and evidence expansion beyond deterministic local signers stays planned for 067-08 and later".to_string(),
                "a separate planner primary/secondary HA service with durable plan state remains out of scope for 067-12".to_string(),
                "real Celestia finality stays unclaimed until real provider dependencies are installed and exercised".to_string(),
                "production HotStuff stays unclaimed until a real backend crate is installed and exercised".to_string(),
            ],
            simulated_markers: vec![
                "external transport is simulated".to_string(),
                "remote process crash or resume is simulated through DistSim and DistDispatch".to_string(),
                "cryptography, theorem bundle, route table, placement, recovery state, and checkpoint artifacts are live project primitives".to_string(),
                "local process/devnet orchestration is simulated-full over the manifest-driven harness".to_string(),
            ],
            claim_levels: report_claim_levels(),
        },
    )?;

    Ok(Scenario11Run { artifact_root })
}

#[derive(Debug, Clone)]
struct LiveTopology {
    cfg: NodeConfig,
    route_table: ShardRouteTable,
    placement_table: ShardPlacementTable,
}

impl LiveTopology {
    fn load() -> Result<Self, Scenario11Error> {
        let cfg = NodeConfig::from_hjmt_home(sim_5a7s_home())?;
        let hjmt = cfg
            .hjmt
            .as_ref()
            .ok_or_else(|| Scenario11Error::Message("missing sim_5a7s hjmt config".to_string()))?;
        let route_path = cfg
            .hjmt
            .as_ref()
            .and_then(|hjmt| hjmt.planner.route.table_path.clone())
            .ok_or_else(|| {
                Scenario11Error::Message("missing sim_5a7s route-table path".to_string())
            })?;
        let raw_hex = String::from_utf8(
            read_file(resolve_hjmt_path(&hjmt.home, &route_path)).map_err(Scenario11Error::Io)?,
        )
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
        let route_table = ShardRouteTable::from_canon(&hex::decode(raw_hex.trim())?)
            .map_err(|err| Scenario11Error::Message(err.to_string()))?;
        let placement_table = cfg
            .placement_table()
            .ok_or_else(|| Scenario11Error::Message("missing placement table".to_string()))?;
        Ok(Self {
            cfg,
            route_table,
            placement_table,
        })
    }

    fn placement(&self, route: BatchRoute) -> Result<ShardPlacement, Scenario11Error> {
        self.placement_table
            .placement(route)
            .cloned()
            .ok_or_else(|| Scenario11Error::Message("missing placement route".to_string()))
    }

    fn lock_path_for(&self, shard_id: u16) -> Result<String, Scenario11Error> {
        let base = self
            .cfg
            .hjmt
            .as_ref()
            .map(|hjmt| resolve_hjmt_path(&hjmt.home, &hjmt.storage.paths.lock_path))
            .and_then(|path| path.to_str().map(str::to_string))
            .ok_or_else(|| Scenario11Error::Message("missing storage lock path".to_string()))?;
        Ok(format!("{base}.scenario11.shard-{shard_id}"))
    }

    fn planner_mode(&self) -> Result<z00z_rollup_node::PlannerMode, Scenario11Error> {
        self.cfg
            .hjmt
            .as_ref()
            .map(|hjmt| hjmt.planner.mode)
            .ok_or_else(|| Scenario11Error::Message("missing sim_5a7s planner mode".to_string()))
    }

    fn planner_cfg_digest(&self) -> Result<PlanDigest, Scenario11Error> {
        let record = self
            .cfg
            .config_digests()?
            .into_iter()
            .find(|record| record.label == "planner-config")
            .ok_or_else(|| {
                Scenario11Error::Message("missing planner-config digest evidence".to_string())
            })?;
        parse_plan_digest(&record.digest_hex)
    }

    fn aggregator_ids(&self) -> Result<Vec<AggregatorId>, Scenario11Error> {
        let mut ids = self
            .cfg
            .hjmt
            .as_ref()
            .map(|hjmt| {
                hjmt.aggs
                    .iter()
                    .map(|agg| agg.aggregator_id)
                    .collect::<Vec<_>>()
            })
            .ok_or_else(|| {
                Scenario11Error::Message("missing sim_5a7s aggregator roster".to_string())
            })?;
        ids.sort_by_key(|id| id.as_u16());
        Ok(ids)
    }
}

fn sim_5a7s_home() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SIM_5A7S_HOME)
}

fn resolve_hjmt_path(home: &Path, raw: &Path) -> PathBuf {
    if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        home.join(raw)
    }
}

fn parse_plan_digest(raw: &str) -> Result<PlanDigest, Scenario11Error> {
    let bytes = hex::decode(raw.trim())?;
    if bytes.len() != 32 {
        return Err(Scenario11Error::Message(format!(
            "planner digest must decode to 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(PlanDigest::new(out))
}

fn planner_authority_replicas(
    topology: &LiveTopology,
    ordered: &OrderedBatch,
    authority: PlannerAuthority,
) -> Result<Vec<PlannerAuthorityReplicaReport>, Scenario11Error> {
    let planner_mode = topology.planner_mode()?;
    let planner_cfg_digest = topology.planner_cfg_digest()?;
    topology
        .aggregator_ids()?
        .into_iter()
        .map(|aggregator_id| {
            let planned = authority
                .verify_batch(
                    planner_mode,
                    &topology.route_table,
                    planner_cfg_digest,
                    ordered.batch_id,
                    &ordered.items,
                    &ordered.planned,
                )
                .map_err(reject_record_to_error)?;
            Ok(PlannerAuthorityReplicaReport {
                aggregator_id: aggregator_id.as_u16(),
                recomputed_plan_digest_hex: hex::encode(planned.plan_digest.as_bytes()),
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
struct HappyPathOutcome {
    package_digest_hex: String,
    batch_id: BatchId,
    ordered: OrderedBatch,
    placement: ShardPlacement,
    subject: CommitSubject,
    theorem_digest: [u8; 32],
    publication_binding: PublicationBinding,
    publication_request: PublicationRequest,
    commit: z00z_aggregators::ConsensusCommit,
    published: z00z_aggregators::PublishedBatch,
    celestia_record: z00z_rollup_node::CelestiaLocalRecord,
    verdict: Verdict,
    dispatch_owner_id: AggregatorId,
    dispatch_stage: DispatchStage,
    consensus_store: ConsensusStoreReport,
    resumed_by_secondary_id: AggregatorId,
    resumed_same_certificate: bool,
    happy_votes: Vec<SecondaryReplayVoteReport>,
    offline_votes: Vec<SecondaryReplayVoteReport>,
    stale_votes: Vec<SecondaryReplayVoteReport>,
}

#[derive(Debug, Clone)]
struct SweepRow {
    batch_id: BatchId,
    shard_id: u16,
    routing_generation: u64,
    route_table_digest_hex: String,
    plan_digest_hex: String,
    dispatch_owner_id: u16,
    dispatch_stage: String,
    secondary_ids: Vec<u16>,
    membership_digest_hex: String,
    expected_journal_lineage_hex: String,
}

#[derive(Debug, Clone)]
struct DualPrimaryCase {
    case_id: String,
    shard_id: u16,
    routing_generation: u64,
    membership_digest_hex: String,
    subject_digest_hex: String,
    certificate_digest_hex: String,
    voter_ids: Vec<u16>,
}

#[derive(Debug, Clone)]
struct DualPrimaryOutcome {
    owner_id: u16,
    shard_ids: Vec<u16>,
    membership_digests_hex: Vec<String>,
    certificate_digests_hex: Vec<String>,
    cases: Vec<DualPrimaryCase>,
}

#[derive(Debug, Clone)]
struct VoteReplayBatchOutcome {
    reports: Vec<SecondaryReplayVoteReport>,
    votes: Vec<ShardVote>,
}

#[derive(Debug, Clone)]
struct DriftFault {
    entry: FaultMatrixEntry,
    vote: Option<SecondaryReplayVoteReport>,
}

#[derive(Debug, Default, Clone)]
struct EvidenceRegistry {
    records: BTreeMap<[u8; 32], EvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct EvidenceRefs {
    ids: Vec<String>,
    artifact_digests_hex: Vec<String>,
}

#[derive(Debug, Clone)]
struct FaultMatrixOutcome {
    faults: Vec<DriftFault>,
    evidence_registry: EvidenceRegistry,
}

impl EvidenceRegistry {
    fn insert(&mut self, record: EvidenceRecord) -> EvidenceRefs {
        let digest = record.digest();
        let artifact_digests_hex = artifact_digests_hex(&record);
        self.records.entry(digest).or_insert(record);
        EvidenceRefs {
            ids: vec![hex::encode(digest)],
            artifact_digests_hex,
        }
    }

    fn insert_many(&mut self, records: impl IntoIterator<Item = EvidenceRecord>) -> EvidenceRefs {
        let mut ids = Vec::new();
        let mut artifact_digests = BTreeSet::new();
        for record in records {
            let refs = self.insert(record);
            ids.extend(refs.ids);
            artifact_digests.extend(refs.artifact_digests_hex);
        }
        ids.sort();
        ids.dedup();
        EvidenceRefs {
            ids,
            artifact_digests_hex: artifact_digests.into_iter().collect(),
        }
    }

    fn contains_id(&self, raw: &str) -> bool {
        let Ok(bytes) = hex::decode(raw) else {
            return false;
        };
        if bytes.len() != 32 {
            return false;
        }
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&bytes);
        self.records.contains_key(&digest)
    }

    fn has_kind(&self, kind: &str) -> bool {
        self.records
            .values()
            .any(|record| record.kind().as_str() == kind)
    }

    fn report(&self) -> EvidenceRegistryReport {
        let entries = self
            .records
            .values()
            .map(|record| EvidenceEntryReport {
                evidence_id_hex: hex::encode(record.digest()),
                evidence_kind: record.kind().as_str().to_string(),
                artifact_digests_hex: artifact_digests_hex(record),
                detail: record
                    .detail()
                    .unwrap_or(record.kind().as_str())
                    .to_string(),
            })
            .collect();
        EvidenceRegistryReport { entries }
    }
}

fn artifact_digests_hex(record: &EvidenceRecord) -> Vec<String> {
    let mut digests = record
        .artifact_refs()
        .into_iter()
        .map(|artifact| hex::encode(artifact.digest))
        .collect::<Vec<_>>();
    digests.sort();
    digests.dedup();
    digests
}

fn validate_fault_evidence(
    faults: &[DriftFault],
    evidence_registry: &EvidenceRegistry,
) -> Result<(), Scenario11Error> {
    for kind in [
        "equivocation",
        "payload_withholding",
        "missing_blob",
        "wrong_root",
        "wrong_route_digest",
        "stale_member",
        "split_brain",
    ] {
        if !evidence_registry.has_kind(kind) {
            return Err(Scenario11Error::Message(format!(
                "structured evidence registry missing required kind {kind}"
            )));
        }
    }

    for fault_id in [
        "equivocation_same_voter",
        "transport_payload_withholding",
        "celestia_missing_blob",
        "wrong_route_digest",
        "wrong_state_root",
        "removed_member_vote",
        "restart_reconnect_old_membership",
        "same_term_divergent_root_freeze",
    ] {
        let entry = faults
            .iter()
            .find(|fault| fault.entry.fault_id == fault_id)
            .ok_or_else(|| {
                Scenario11Error::Message(format!("missing fault-matrix entry {fault_id}"))
            })?;
        if entry.entry.evidence_refs.is_empty() {
            return Err(Scenario11Error::Message(format!(
                "fault {fault_id} must reference structured evidence ids"
            )));
        }
        if entry.entry.artifact_digests_hex.is_empty() {
            return Err(Scenario11Error::Message(format!(
                "fault {fault_id} must reference artifact digests"
            )));
        }
        if entry
            .entry
            .evidence_refs
            .iter()
            .any(|raw| !evidence_registry.contains_id(raw))
        {
            return Err(Scenario11Error::Message(format!(
                "fault {fault_id} referenced a non-registry evidence id"
            )));
        }
    }

    for fault in faults {
        let Some(vote) = &fault.vote else {
            continue;
        };
        let Some(evidence_id_hex) = vote.evidence_id_hex.as_deref() else {
            continue;
        };
        if !evidence_registry.contains_id(evidence_id_hex) {
            return Err(Scenario11Error::Message(format!(
                "vote report {} referenced a non-registry evidence id",
                vote.case_id
            )));
        }
        if vote.evidence_kind.is_none() {
            return Err(Scenario11Error::Message(format!(
                "vote report {} emitted evidence without kind metadata",
                vote.case_id
            )));
        }
        if vote.artifact_digests_hex.is_empty() {
            return Err(Scenario11Error::Message(format!(
                "vote report {} emitted evidence without artifact digests",
                vote.case_id
            )));
        }
    }

    Ok(())
}

fn decode_namespace_hex(raw: &str) -> Result<[u8; 8], Scenario11Error> {
    let bytes = hex::decode(raw)?;
    if bytes.len() != 8 {
        return Err(Scenario11Error::Message(format!(
            "invalid Celestia-local namespace length: expected 8 bytes, got {}",
            bytes.len()
        )));
    }
    let mut namespace = [0u8; 8];
    namespace.copy_from_slice(&bytes);
    Ok(namespace)
}

fn require_error<T, E>(result: Result<T, E>, message: &str) -> Result<E, Scenario11Error> {
    match result {
        Ok(_) => Err(Scenario11Error::Message(message.to_string())),
        Err(err) => Ok(err),
    }
}

fn run_happy_path(
    topology: &LiveTopology,
    artifact_root: &Path,
) -> Result<HappyPathOutcome, Scenario11Error> {
    let (package, prev_root) = valid_tx_package("scenario11-happy")?;
    let batch_id = batch_id("scenario11-happy");
    let item = z00z_aggregators::IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(package.clone())))
        .map_err(reject_record_to_error)?;
    let planner = BatchPlanner::new(topology.route_table.clone());
    let ordered = planner
        .make_batch(batch_id, std::slice::from_ref(&item))
        .map_err(reject_record_to_error)?;
    let placement = topology.placement(ordered.planned.route)?;
    let recovery = route_bound_recovery_state(
        0x91,
        batch_id,
        ordered.planned.route,
        ordered.planned.route_table_digest.into_bytes(),
        placement.expected_journal_lineage,
    )?;
    let record = recovery_record(
        batch_id,
        ordered.planned.route,
        placement.primary_id,
        placement.secondaries.clone(),
        recovery.clone(),
    )?;
    let candidate = JournalCandidate::from_record(&record).map_err(reject_record_to_error)?;
    let publication_route = PublicationRouteSnapshotV1::new(
        ordered.planned.route.routing_generation,
        ordered.planned.route_table_digest.into_bytes(),
        topology.route_table.activation_checkpoint,
        vec![u32::from(ordered.planned.route.shard_id.as_u16())],
    );
    let replay_id = "scenario11-happy-replay";
    let prepared_publication = prepare_publication_for_package(
        batch_id,
        package.clone(),
        prev_root,
        recovery.state_root,
        publication_route.clone(),
        replay_id,
    )?;
    let theorem_digest = theorem_digest(&prepared_publication)?;
    let publication_binding = publication_binding_for_prepared(&prepared_publication)?;
    let subject = CommitSubject::from_runtime(
        7,
        membership_digest_for_voters(
            ordered.planned.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        &ordered,
        &candidate,
        &publication_binding,
        theorem_digest,
        None,
    )
    .map_err(reject_record_to_error)?;

    let happy_vote_outcome = replay_votes_for_subject(
        "happy_path",
        &subject,
        &planner,
        topology,
        &record,
        &recovery,
        &publication_binding,
        theorem_digest,
        &placement,
        std::slice::from_ref(&item),
    )?;
    let takeover_id = placement
        .secondaries
        .iter()
        .find(|secondary| secondary.is_ready)
        .map(|secondary| secondary.aggregator_id)
        .ok_or_else(|| Scenario11Error::Message("missing ready secondary".to_string()))?;
    let happy_votes = happy_vote_outcome.reports;
    let primary_vote = ShardVote::new_local(
        placement.primary_id,
        ShardVoteRole::Primary,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let quorum_vote = happy_vote_outcome
        .votes
        .into_iter()
        .find(|vote| vote.voter_id == takeover_id)
        .ok_or_else(|| Scenario11Error::Message("missing accepting secondary vote".to_string()))?;
    let mut adapter =
        ConsensusAdapter::from_placement(&placement).map_err(reject_record_to_error)?;
    let commit = adapter
        .commit(&subject, &[primary_vote, quorum_vote])
        .map_err(reject_record_to_error)?;
    let request = publication_request_for_package(
        prepared_publication,
        ordered.clone(),
        commit.subject.clone(),
        commit.certificate.clone(),
        replay_id,
    );

    let mut dispatch = DistDispatch::new(
        topology.route_table.clone(),
        topology.placement_table.clone(),
    )
    .map_err(reject_record_to_error)?;
    let lock_path = topology.lock_path_for(ordered.planned.route.shard_id.as_u16())?;
    let dispatch_verdict = dispatch
        .dispatch_batch(
            batch_id,
            std::slice::from_ref(&item),
            placement.primary_id,
            1,
            1,
            lock_path,
        )
        .map_err(reject_record_to_error)?;

    let mut da = CelestiaLocalAdapter::new("scenario_11_local_da");
    let published = da.publish(request.clone())?;
    let celestia_record = da.record(published.batch_id).cloned().ok_or_else(|| {
        Scenario11Error::Message(
            "missing Celestia-local publication record for happy-path batch".to_string(),
        )
    })?;
    let resolved = da.resolve(&published)?;
    let executor = ShardExecutor::new(topology.placement_table.clone());
    let ticket = executor.mark_running(
        &executor
            .route(&ordered.planned)
            .map_err(reject_record_to_error)?,
    );
    let resolved_batch = ResolvedBatch::new(
        published.clone(),
        Some(publication_record_for_published(
            &published,
            PublicationState::Posted,
        )),
        ordered.clone(),
        resolved.theorem.clone(),
        resolved.subject.clone(),
        resolved.certificate.clone(),
        resolved.nullifiers.clone(),
        Some(placement.view()),
        Some(ticket),
    );
    let verdict =
        ValidatorBoundary.verdict_for_batch(&resolved_batch, &ObjectPolicyRegistryV1::default());
    if verdict.kind != VerdictKind::Accepted {
        return Err(Scenario11Error::Message(format!(
            "happy-path validator verdict was not accepted: {:?}",
            verdict.kind
        )));
    }

    let store_root = artifact_root.join("consensus_store");
    let store = ConsensusStore::open(&store_root).map_err(consensus_store_error_to_error)?;
    persist_consensus_commit(
        &store,
        &record,
        &subject,
        &commit.certificate.votes,
        &commit.certificate,
    )
    .map_err(reject_record_to_error)?;
    let publication_record = publication_record_for_published(&published, PublicationState::Posted);
    persist_consensus_publication(
        &store,
        batch_id,
        publication_record,
        &publication_binding,
        &published,
    )
    .map_err(reject_record_to_error)?;
    let validator_decision = validator_decision_snapshot(
        verdict_kind_name(&verdict.kind),
        verdict.reject.as_ref().map(|reject| format!("{reject:?}")),
        batch_id,
        &subject,
        &commit.certificate,
        theorem_digest,
        verdict.checkpoint_id,
        verdict.publication.as_ref(),
    );
    persist_validator_decision(&store, batch_id, validator_decision)
        .map_err(reject_record_to_error)?;
    let reloaded_store =
        ConsensusStore::open(&store_root).map_err(consensus_store_error_to_error)?;
    let resumed = RecoveryBoundary
        .resume_from_store(
            takeover_id,
            &topology.placement_table,
            &recovery,
            &reloaded_store,
            ordered.planned.route,
            RecoveryIntent::TakeoverSecondary,
        )
        .map_err(reject_record_to_error)?;
    let resumed_same_certificate = resumed.ticket.batch_id == batch_id
        && resumed.ticket.placement.route == placement.view().route
        && resumed.ticket.placement.primary_id == takeover_id
        && resumed.record.header.digest() == subject.digest()
        && resumed.record.certificate.digest() == commit.certificate.digest()
        && resumed
            .record
            .votes
            .iter()
            .any(|vote| vote.voter_id == takeover_id);
    let resumed_publication = resumed.record.publication.as_ref().ok_or_else(|| {
        Scenario11Error::Message(
            "reloaded consensus store is missing publication evidence for happy-path batch"
                .to_string(),
        )
    })?;
    let resumed_validator = resumed.record.validator_decision.as_ref().ok_or_else(|| {
        Scenario11Error::Message(
            "reloaded consensus store is missing validator decision for happy-path batch"
                .to_string(),
        )
    })?;
    let resumed_checkpoint = resumed_validator.checkpoint_id.ok_or_else(|| {
        Scenario11Error::Message(
            "reloaded validator decision is missing checkpoint id for happy-path batch".to_string(),
        )
    })?;
    let consensus_store = ConsensusStoreReport {
        backend: CONSENSUS_STORE_BACKEND.to_string(),
        schema_version: CONSENSUS_STORE_SCHEMA_VERSION,
        route_key_hex: resumed.record.route_key_hex.clone(),
        batch_id_hex: hex::encode(batch_id.into_bytes()),
        subject_digest_hex: hex::encode(resumed.record.header.digest()),
        certificate_digest_hex: hex::encode(resumed.record.certificate.digest()),
        vote_digests_hex: resumed
            .record
            .votes
            .iter()
            .map(|vote| hex::encode(vote.digest()))
            .collect(),
        publication_binding_digest_hex: hex::encode(resumed_publication.binding.binding_digest),
        validator_verdict_kind: resumed_validator.verdict_kind.clone(),
        checkpoint_id_hex: hex::encode(resumed_checkpoint.into_bytes()),
        resumed_by_secondary_id: takeover_id.as_u16(),
        resume_source: "reloaded_from_store".to_string(),
    };

    let offline_votes = offline_secondary_case(
        &subject,
        &planner,
        topology,
        &record,
        &recovery,
        &publication_binding,
        theorem_digest,
        &placement,
        std::slice::from_ref(&item),
    )?;
    let stale_votes = stale_secondary_case(
        &subject,
        &planner,
        topology,
        &record,
        &publication_binding,
        theorem_digest,
        &placement,
        std::slice::from_ref(&item),
    )?;

    Ok(HappyPathOutcome {
        package_digest_hex: package.tx_digest_hex,
        batch_id,
        ordered,
        placement,
        subject,
        theorem_digest,
        publication_binding,
        publication_request: request,
        commit,
        published,
        celestia_record,
        verdict,
        dispatch_owner_id: dispatch_verdict.owner_id,
        dispatch_stage: dispatch_verdict.stage,
        consensus_store,
        resumed_by_secondary_id: takeover_id,
        resumed_same_certificate,
        happy_votes,
        offline_votes,
        stale_votes,
    })
}

fn run_all_shard_sweep(topology: &LiveTopology) -> Result<Vec<SweepRow>, Scenario11Error> {
    let mut dispatch = DistDispatch::new(
        topology.route_table.clone(),
        topology.placement_table.clone(),
    )
    .map_err(reject_record_to_error)?;
    let mut owner_seq = BTreeMap::<AggregatorId, u64>::new();
    let mut rows = Vec::new();
    for shard_id in topology
        .route_table
        .shard_set
        .iter()
        .map(|shard| shard.as_u16())
    {
        let item = find_simple_item_for_shard(&topology.route_table, shard_id, "scenario11-sweep")?;
        let batch_id = batch_id(&format!("scenario11-sweep-{shard_id}"));
        let planner = BatchPlanner::new(topology.route_table.clone());
        let ordered = planner
            .make_batch(batch_id, std::slice::from_ref(&item))
            .map_err(reject_record_to_error)?;
        let placement = topology.placement(ordered.planned.route)?;
        let delivery_seq = owner_seq.entry(placement.primary_id).or_insert(1);
        let verdict = dispatch
            .dispatch_batch(
                batch_id,
                std::slice::from_ref(&item),
                placement.primary_id,
                *delivery_seq,
                1,
                topology.lock_path_for(shard_id)?,
            )
            .map_err(reject_record_to_error)?;
        *delivery_seq += 1;
        rows.push(SweepRow {
            batch_id,
            shard_id,
            routing_generation: ordered.planned.route.routing_generation,
            route_table_digest_hex: hex::encode(ordered.planned.route_table_digest.as_bytes()),
            plan_digest_hex: hex::encode(ordered.planned.plan_digest.as_bytes()),
            dispatch_owner_id: verdict.owner_id.as_u16(),
            dispatch_stage: dispatch_stage_name(verdict.stage).to_string(),
            secondary_ids: secondary_ids(&placement.secondaries),
            membership_digest_hex: hex::encode(membership_digest_for_voters(
                ordered.planned.route,
                placement.primary_id,
                placement
                    .secondaries
                    .iter()
                    .filter(|secondary| secondary.is_ready)
                    .map(|secondary| secondary.aggregator_id),
            )),
            expected_journal_lineage_hex: hex::encode(placement.expected_journal_lineage),
        });
    }
    Ok(rows)
}

fn run_dual_primary_isolation(
    topology: &LiveTopology,
    theorem_digest: [u8; 32],
) -> Result<DualPrimaryOutcome, Scenario11Error> {
    let hjmt = topology
        .cfg
        .hjmt
        .as_ref()
        .ok_or_else(|| Scenario11Error::Message("missing hjmt config".to_string()))?;
    let (owner_id, _) = hjmt
        .primary_counts()
        .into_iter()
        .find(|(_, count)| *count >= 2)
        .ok_or_else(|| Scenario11Error::Message("missing dual-primary owner".to_string()))?;
    let shard_ids = hjmt
        .proc(owner_id)
        .ok_or_else(|| Scenario11Error::Message("missing dual-primary proc".to_string()))?
        .shards
        .iter()
        .map(|shard| shard.shard_id.as_u16())
        .collect::<Vec<_>>();
    let mut cases = Vec::new();
    for (index, shard_id) in shard_ids.iter().copied().enumerate() {
        let item = find_simple_item_for_shard(&topology.route_table, shard_id, "scenario11-dual")?;
        let batch_id = batch_id(&format!("scenario11-dual-{shard_id}"));
        let planner = BatchPlanner::new(topology.route_table.clone());
        let ordered = planner
            .make_batch(batch_id, std::slice::from_ref(&item))
            .map_err(reject_record_to_error)?;
        let placement = topology.placement(ordered.planned.route)?;
        let quorum_case = build_quorum_only_case(
            &ordered,
            &placement,
            theorem_digest,
            0xA0u8.wrapping_add(index as u8),
        )?;
        cases.push(DualPrimaryCase {
            case_id: format!("dual_primary_shard_{shard_id}"),
            shard_id,
            routing_generation: ordered.planned.route.routing_generation,
            membership_digest_hex: hex::encode(quorum_case.certificate.membership_digest),
            subject_digest_hex: hex::encode(quorum_case.subject.digest()),
            certificate_digest_hex: hex::encode(quorum_case.certificate.digest()),
            voter_ids: quorum_case
                .certificate
                .votes
                .iter()
                .map(|vote| vote.voter_id.as_u16())
                .collect(),
        });
    }
    Ok(DualPrimaryOutcome {
        owner_id: owner_id.as_u16(),
        shard_ids: shard_ids.clone(),
        membership_digests_hex: cases
            .iter()
            .map(|case| case.membership_digest_hex.clone())
            .collect(),
        certificate_digests_hex: cases
            .iter()
            .map(|case| case.certificate_digest_hex.clone())
            .collect(),
        cases,
    })
}

fn run_fault_matrix(
    topology: &LiveTopology,
    happy: &HappyPathOutcome,
) -> Result<FaultMatrixOutcome, Scenario11Error> {
    let planner = BatchPlanner::new(topology.route_table.clone());
    let ready_secondary = happy
        .placement
        .secondaries
        .iter()
        .find(|secondary| secondary.is_ready)
        .map(|secondary| secondary.aggregator_id)
        .ok_or_else(|| Scenario11Error::Message("missing ready secondary".to_string()))?;
    let recovery = route_bound_recovery_state(
        0x91,
        happy.batch_id,
        happy.ordered.planned.route,
        happy.ordered.planned.route_table_digest.into_bytes(),
        happy.placement.expected_journal_lineage,
    )?;
    let record = recovery_record(
        happy.batch_id,
        happy.ordered.planned.route,
        happy.placement.primary_id,
        happy.placement.secondaries.clone(),
        recovery.clone(),
    )?;
    let candidate = JournalCandidate::from_record(&record).map_err(reject_record_to_error)?;
    let items = happy.ordered.items.clone();
    let request = SecondaryReplayRequest {
        voter_id: ready_secondary,
        term: happy.subject.term,
        items: &items,
        planner: &planner,
        placement_table: &topology.placement_table,
        recovery_record: &record,
        local_recovery: &recovery,
        publication_binding: &happy.publication_binding,
        theorem_or_settlement_digest: happy.theorem_digest,
        da_availability_digest: None,
    };

    let mut faults = Vec::new();
    let mut evidence_registry = EvidenceRegistry::default();

    let mut dispatch = DistDispatch::new(
        topology.route_table.clone(),
        topology.placement_table.clone(),
    )
    .map_err(reject_record_to_error)?;
    dispatch
        .partition(happy.placement.primary_id)
        .map_err(reject_record_to_error)?;
    let unavailable = dispatch
        .dispatch_batch(
            batch_id("scenario11-primary-offline"),
            &items,
            happy.placement.primary_id,
            1,
            1,
            topology.lock_path_for(happy.subject.shard_id.as_u16())?,
        )
        .map_err(reject_record_to_error)?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "primary_offline_before_dispatch".to_string(),
            expected_status: "deferred_before_dispatch".to_string(),
            observed_status: if unavailable.stage == DispatchStage::Deferred
                && unavailable.detail.contains("owner unavailable")
            {
                "deferred_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "dispatch deferred while the shard owner stayed offline before execution began"
                .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "one_secondary_offline".to_string(),
            expected_status: "degraded_without_synthetic_vote".to_string(),
            observed_status: if happy
                .offline_votes
                .iter()
                .any(|vote| vote.case_id == "one_secondary_offline" && vote.verdict == "offline")
            {
                "degraded_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "one ready secondary stayed offline, quorum preserved with no synthetic vote"
                .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "one_secondary_stale".to_string(),
            expected_status: "stale_secondary_rejects".to_string(),
            observed_status: if happy.stale_votes.iter().any(|vote| {
                vote.case_id == "stale_secondary"
                    && vote.reject_code.as_deref() == Some("StaleSecondaryState")
            }) {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: Some("StaleSecondaryState".to_string()),
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "stale secondary replay failed closed before any vote was created".to_string(),
            degraded_mode: false,
        },
        vote: None,
    });

    let observer_id = AggregatorId::new(610);
    let mut observer_pending = happy.placement.clone();
    observer_pending
        .secondaries
        .push(SecondaryState::pending(observer_id));
    let observer_pending_subject = subject_for_placement(
        happy.subject.term + 1,
        &happy.ordered,
        &observer_pending,
        &candidate,
        &happy.publication_binding,
        happy.theorem_digest,
    )?;
    let observer_pending_err = require_error(
        ConsensusAdapter::from_placement(&observer_pending)
            .map_err(reject_record_to_error)?
            .commit(
                &observer_pending_subject,
                &[
                    vote_for_subject(
                        &observer_pending_subject,
                        observer_pending.primary_id,
                        ShardVoteRole::Primary,
                    ),
                    vote_for_subject(
                        &observer_pending_subject,
                        observer_id,
                        ShardVoteRole::Secondary,
                    ),
                ],
            ),
        "observer unexpectedly voted before readiness",
    )?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "observer_not_ready_before_readiness".to_string(),
            expected_status: "pending_observer_rejects".to_string(),
            observed_status: if observer_pending_err.detail.contains("inactive voter ids") {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "observer stayed pending and could not contribute before readiness".to_string(),
            degraded_mode: false,
        },
        vote: None,
    });

    let mut observer_ready = observer_pending.clone();
    if let Some(observer) = observer_ready
        .secondaries
        .iter_mut()
        .find(|secondary| secondary.aggregator_id == observer_id)
    {
        observer.is_ready = true;
    }
    let observer_ready_subject = subject_for_placement(
        happy.subject.term + 2,
        &happy.ordered,
        &observer_ready,
        &candidate,
        &happy.publication_binding,
        happy.theorem_digest,
    )?;
    let observer_ready_commit = ConsensusAdapter::from_placement(&observer_ready)
        .map_err(reject_record_to_error)?
        .commit(
            &observer_ready_subject,
            &[
                vote_for_subject(
                    &observer_ready_subject,
                    observer_ready.primary_id,
                    ShardVoteRole::Primary,
                ),
                vote_for_subject(
                    &observer_ready_subject,
                    ready_secondary,
                    ShardVoteRole::Secondary,
                ),
                vote_for_subject(
                    &observer_ready_subject,
                    observer_id,
                    ShardVoteRole::Secondary,
                ),
            ],
        )
        .map_err(reject_record_to_error)?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "observer_ready_after_catchup".to_string(),
            expected_status: "ready_observer_commits_lawfully".to_string(),
            observed_status: if observer_ready_commit
                .certificate
                .votes
                .iter()
                .any(|vote| vote.voter_id == observer_id)
            {
                "accepted_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail:
                "observer became ready, joined the active membership digest, and voted lawfully"
                    .to_string(),
            degraded_mode: false,
        },
        vote: None,
    });

    let removed_secondary = happy
        .placement
        .secondaries
        .iter()
        .find(|secondary| secondary.is_ready && secondary.aggregator_id != ready_secondary)
        .copied()
        .or_else(|| {
            happy
                .placement
                .secondaries
                .iter()
                .find(|secondary| secondary.is_ready)
                .copied()
        })
        .ok_or_else(|| Scenario11Error::Message("missing removable secondary".to_string()))?;
    let mut removed_member_placement = happy.placement.clone();
    removed_member_placement
        .secondaries
        .retain(|secondary| secondary.aggregator_id != removed_secondary.aggregator_id);
    let removed_member_subject = subject_for_placement(
        happy.subject.term + 3,
        &happy.ordered,
        &removed_member_placement,
        &candidate,
        &happy.publication_binding,
        happy.theorem_digest,
    )?;
    let removed_member_err = require_error(
        ConsensusAdapter::from_placement(&removed_member_placement)
            .map_err(reject_record_to_error)?
            .commit(
                &removed_member_subject,
                &[
                    vote_for_subject(
                        &removed_member_subject,
                        removed_member_placement.primary_id,
                        ShardVoteRole::Primary,
                    ),
                    vote_for_subject(
                        &removed_member_subject,
                        removed_secondary.aggregator_id,
                        ShardVoteRole::Secondary,
                    ),
                ],
            ),
        "removed member unexpectedly voted",
    )?;
    let removed_member_refs = evidence_registry.insert(EvidenceRecord::StaleMember(
        StaleMemberEvidence::new(
            removed_secondary.aggregator_id,
            removed_member_subject.shard_id,
            removed_member_subject.term,
            removed_member_subject.membership_digest,
            happy.subject.membership_digest,
            removed_member_err.detail.clone(),
        )
        .map_err(reject_record_to_error)?,
    ));
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "removed_member_vote".to_string(),
            expected_status: "removed_member_rejects".to_string(),
            observed_status: if removed_member_err.detail.contains("inactive voter ids") {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: removed_member_refs.ids,
            artifact_digests_hex: removed_member_refs.artifact_digests_hex,
            detail: "removed committee member could not reappear in the next shard commit"
                .to_string(),
            degraded_mode: false,
        },
        vote: None,
    });

    let mut mixed_generation_subject = happy.subject.clone();
    mixed_generation_subject.routing_generation = mixed_generation_subject
        .routing_generation
        .saturating_add(1);
    mixed_generation_subject.membership_digest = membership_digest_for_voters(
        mixed_generation_subject.route(),
        happy.placement.primary_id,
        happy
            .placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.is_ready)
            .map(|secondary| secondary.aggregator_id),
    );
    let mixed_generation_err = require_error(
        ConsensusAdapter::from_placement(&happy.placement)
            .map_err(reject_record_to_error)?
            .commit(
                &mixed_generation_subject,
                &[
                    vote_for_subject(
                        &mixed_generation_subject,
                        happy.placement.primary_id,
                        ShardVoteRole::Primary,
                    ),
                    vote_for_subject(
                        &mixed_generation_subject,
                        ready_secondary,
                        ShardVoteRole::Secondary,
                    ),
                ],
            ),
        "mixed-generation certificate unexpectedly succeeded",
    )?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "mixed_generation_certificate".to_string(),
            expected_status: "mixed_generation_rejects".to_string(),
            observed_status: if mixed_generation_err.detail.contains("wrong generation") {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "certificate formation rejected when the subject generation drifted"
                .to_string(),
            degraded_mode: false,
        },
        vote: None,
    });

    let mut divergent_adapter =
        ConsensusAdapter::from_placement(&happy.placement).map_err(reject_record_to_error)?;
    divergent_adapter
        .commit(
            &happy.subject,
            &[
                vote_for_subject(
                    &happy.subject,
                    happy.placement.primary_id,
                    ShardVoteRole::Primary,
                ),
                vote_for_subject(&happy.subject, ready_secondary, ShardVoteRole::Secondary),
            ],
        )
        .map_err(reject_record_to_error)?;
    let mut divergent_subject = happy.subject.clone();
    mutate_state_root(&mut divergent_subject);
    let divergent_err = require_error(
        divergent_adapter.commit(
            &divergent_subject,
            &[
                vote_for_subject(
                    &divergent_subject,
                    happy.placement.primary_id,
                    ShardVoteRole::Primary,
                ),
                vote_for_subject(
                    &divergent_subject,
                    ready_secondary,
                    ShardVoteRole::Secondary,
                ),
            ],
        ),
        "same-term divergent root unexpectedly committed",
    )?;
    let frozen_err = require_error(
        divergent_adapter.commit(
            &happy.subject,
            &[
                vote_for_subject(
                    &happy.subject,
                    happy.placement.primary_id,
                    ShardVoteRole::Primary,
                ),
                vote_for_subject(&happy.subject, ready_secondary, ShardVoteRole::Secondary),
            ],
        ),
        "frozen same-term branch unexpectedly accepted",
    )?;
    let split_brain_refs = evidence_registry.insert(EvidenceRecord::SplitBrain(
        SplitBrainEvidence::new(
            happy.placement.primary_id,
            divergent_subject.shard_id,
            divergent_subject.term,
            divergent_subject.membership_digest,
            happy.subject.digest(),
            divergent_subject.digest(),
            divergent_err.detail.clone(),
        )
        .map_err(reject_record_to_error)?,
    ));
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "same_term_divergent_root_freeze".to_string(),
            expected_status: "divergence_freezes_term".to_string(),
            observed_status: if divergent_err.detail.contains("split-brain")
                && frozen_err.detail.contains("frozen")
            {
                "frozen_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: split_brain_refs.ids,
            artifact_digests_hex: split_brain_refs.artifact_digests_hex,
            detail:
                "same-term divergent root froze the quorum term before any conflicting certificate"
                    .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    let mut lifecycle_sim = DistSim::new(
        happy.ordered.planned.route,
        std::iter::once(happy.placement.primary_id).chain(
            happy
                .placement
                .secondaries
                .iter()
                .map(|secondary| secondary.aggregator_id),
        ),
    )
    .map_err(reject_record_to_error)?;
    lifecycle_sim
        .seed(happy.placement.primary_id, record.clone())
        .map_err(reject_record_to_error)?;
    lifecycle_sim
        .partition(ready_secondary)
        .map_err(reject_record_to_error)?;
    let lifecycle_frame = JournalFrame::new(
        happy.placement.primary_id,
        ready_secondary,
        happy.subject.term,
        record.clone(),
    );
    lifecycle_sim.enqueue(lifecycle_frame.clone());
    let deferred = lifecycle_sim.step();
    lifecycle_sim
        .heal(ready_secondary)
        .map_err(reject_record_to_error)?;
    let applied = lifecycle_sim.step();
    lifecycle_sim.enqueue(lifecycle_frame);
    let replay_ignored = lifecycle_sim.step();
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "partition_and_heal".to_string(),
            expected_status: "defer_then_apply_without_conflict".to_string(),
            observed_status: if deferred
                .iter()
                .any(|verdict| verdict.stage == FrameStage::Deferred)
                && applied
                    .iter()
                    .any(|verdict| verdict.stage == FrameStage::Applied)
                && replay_ignored
                    .iter()
                    .any(|verdict| verdict.stage == FrameStage::ReplayIgnored)
            {
                "healed_without_conflict".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail:
                "partitioned replication deferred, healed cleanly, and ignored replay instead of forking"
                    .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    let mut takeover_sim = DistSim::new(
        happy.ordered.planned.route,
        std::iter::once(happy.placement.primary_id).chain(
            happy
                .placement
                .secondaries
                .iter()
                .map(|secondary| secondary.aggregator_id),
        ),
    )
    .map_err(reject_record_to_error)?;
    takeover_sim
        .seed(happy.placement.primary_id, record.clone())
        .map_err(reject_record_to_error)?;
    for secondary in &happy.placement.secondaries {
        if secondary.is_ready {
            takeover_sim
                .seed(secondary.aggregator_id, record.clone())
                .map_err(reject_record_to_error)?;
        }
    }
    let takeover_ticket = takeover_sim
        .resume(
            ready_secondary,
            &topology.placement_table,
            &record,
            z00z_aggregators::RecoveryIntent::TakeoverSecondary,
        )
        .map_err(reject_record_to_error)?;
    let unrelated_shard_id = topology
        .route_table
        .shard_set
        .iter()
        .map(|shard| shard.as_u16())
        .find(|shard_id| *shard_id != happy.subject.shard_id.as_u16())
        .ok_or_else(|| Scenario11Error::Message("missing unrelated shard".to_string()))?;
    let unrelated_item = find_simple_item_for_shard(
        &topology.route_table,
        unrelated_shard_id,
        "scenario11-takeover-continuity",
    )?;
    let unrelated_planner = BatchPlanner::new(topology.route_table.clone());
    let unrelated_ordered = unrelated_planner
        .make_batch(
            batch_id(&format!(
                "scenario11-takeover-continuity-{unrelated_shard_id}"
            )),
            std::slice::from_ref(&unrelated_item),
        )
        .map_err(reject_record_to_error)?;
    let unrelated_placement = topology.placement(unrelated_ordered.planned.route)?;
    let unrelated_case = build_quorum_only_case(
        &unrelated_ordered,
        &unrelated_placement,
        happy.theorem_digest,
        0xC4,
    )?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "rolling_primary_takeover_continuity".to_string(),
            expected_status: "affected_and_unrelated_shards_continue_lawfully".to_string(),
            observed_status: if takeover_ticket.placement.primary_id == ready_secondary
                && unrelated_case.subject.shard_id.as_u16() != happy.subject.shard_id.as_u16()
                && unrelated_case.certificate.votes.len() >= 2
            {
                "continued_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail:
                "one shard failed over to a lawful new primary while an unrelated shard kept producing a lawful certificate"
                    .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    let first_equivocation_vote =
        vote_for_subject(&happy.subject, ready_secondary, ShardVoteRole::Secondary);
    let mut conflicting_equivocation_subject = happy.subject.clone();
    mutate_theorem_digest(&mut conflicting_equivocation_subject);
    let second_equivocation_vote = vote_for_subject(
        &conflicting_equivocation_subject,
        ready_secondary,
        ShardVoteRole::Secondary,
    );
    let mut equivocation_tracker = VoteEvidenceTracker::default();
    equivocation_tracker
        .record_vote(first_equivocation_vote)
        .map_err(|_| {
            Scenario11Error::Message(
                "first same-voter vote unexpectedly emitted equivocation evidence".to_string(),
            )
        })?;
    let equivocation_record = match equivocation_tracker.record_vote(second_equivocation_vote) {
        Err(record) => record,
        Ok(()) => {
            return Err(Scenario11Error::Message(
                "conflicting same-voter votes failed to emit structured equivocation evidence"
                    .to_string(),
            ))
        }
    };
    let equivocation_evidence = EvidenceRecord::from(equivocation_record);
    let equivocation_id_hex = hex::encode(equivocation_evidence.digest());
    let equivocation_kind = equivocation_evidence.kind().as_str().to_string();
    let equivocation_artifacts = artifact_digests_hex(&equivocation_evidence);
    let equivocation_refs = evidence_registry.insert(equivocation_evidence);
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "equivocation_same_voter".to_string(),
            expected_status: "same_voter_conflict_emits_evidence".to_string(),
            observed_status: if equivocation_tracker.records().len() == 1 {
                "evidence_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: equivocation_refs.ids,
            artifact_digests_hex: equivocation_refs.artifact_digests_hex,
            detail: "conflicting same-voter votes emitted structured equivocation evidence"
                .to_string(),
            degraded_mode: false,
        },
        vote: Some(SecondaryReplayVoteReport {
            case_id: "equivocation_same_voter".to_string(),
            voter_id: ready_secondary.as_u16(),
            voter_role: "secondary".to_string(),
            verdict: "degraded".to_string(),
            transport_verdict: "evidence_emitted".to_string(),
            signature_scheme: None,
            vote_digest_hex: None,
            evidence_id_hex: Some(equivocation_id_hex),
            evidence_kind: Some(equivocation_kind),
            artifact_digests_hex: equivocation_artifacts,
            reject_code: None,
            detail: "same-voter conflicting replay emitted structured equivocation evidence"
                .to_string(),
        }),
    });

    let mut duplicate_transport = InMemoryVoteTransport::default();
    duplicate_transport.enqueue_planned(
        VoteTransportEnvelope::available(
            happy.placement.primary_id,
            ready_secondary,
            happy.subject.clone(),
            ShardVoteKind::LocalCommit,
        ),
        TransportDeliveryPlan::default()
            .with_duplicate_after(1)
            .with_replay_after(2),
    );
    let mut duplicate_service = ReplayVerifiedVoteService::local();
    let duplicate_first = duplicate_transport
        .step()
        .into_iter()
        .next()
        .ok_or_else(|| Scenario11Error::Message("missing first duplicate delivery".to_string()))?;
    let (duplicate_accept, _) = replay_vote_report(
        "transport_duplicate_replay",
        ready_secondary,
        duplicate_service.process_envelope(
            &duplicate_first,
            VoteExchangeContext {
                voter_role: ShardVoteRole::Secondary,
                replay_request: request,
            },
        ),
    );
    let duplicate_second = duplicate_transport
        .step()
        .into_iter()
        .next()
        .ok_or_else(|| Scenario11Error::Message("missing duplicate delivery".to_string()))?;
    let (duplicate_report, _) = replay_vote_report(
        "transport_duplicate_replay",
        ready_secondary,
        duplicate_service.process_envelope(
            &duplicate_second,
            VoteExchangeContext {
                voter_role: ShardVoteRole::Secondary,
                replay_request: request,
            },
        ),
    );
    let duplicate_third = duplicate_transport
        .step()
        .into_iter()
        .next()
        .ok_or_else(|| Scenario11Error::Message("missing replay delivery".to_string()))?;
    let (replay_report, _) = replay_vote_report(
        "transport_duplicate_replay",
        ready_secondary,
        duplicate_service.process_envelope(
            &duplicate_third,
            VoteExchangeContext {
                voter_role: ShardVoteRole::Secondary,
                replay_request: request,
            },
        ),
    );
    let duplicate_refs = evidence_registry.insert_many(
        duplicate_transport
            .fault_records()
            .iter()
            .cloned()
            .map(EvidenceRecord::from),
    );
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "transport_duplicate_replay".to_string(),
            expected_status: "duplicate_and_replay_do_not_double_count".to_string(),
            observed_status: if duplicate_accept.verdict == "accept"
                && duplicate_report.transport_verdict == "duplicate_message"
                && replay_report.transport_verdict == "duplicate_message"
            {
                "ignored_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: duplicate_refs.ids,
            artifact_digests_hex: duplicate_refs.artifact_digests_hex,
            detail: "duplicate and replayed envelopes stayed bounded to one replay-verified vote"
                .to_string(),
            degraded_mode: false,
        },
        vote: Some(duplicate_report),
    });

    let mut evidence_service = ReplayVerifiedVoteService::local();
    let payload_envelope = VoteTransportEnvelope::missing_payload(
        happy.placement.primary_id,
        ready_secondary,
        happy.subject.clone(),
        ShardVoteKind::LocalCommit,
        "payload missing before replay",
    );
    let (payload_report, _) = replay_vote_report(
        "transport_payload_withholding",
        ready_secondary,
        evidence_service.process_envelope(
            &payload_envelope,
            VoteExchangeContext {
                voter_role: ShardVoteRole::Secondary,
                replay_request: request,
            },
        ),
    );
    let payload_refs = evidence_registry.insert_many(
        evidence_service
            .evidence_records()
            .iter()
            .cloned()
            .map(EvidenceRecord::from),
    );
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "transport_payload_withholding".to_string(),
            expected_status: "evidence_without_vote".to_string(),
            observed_status: if payload_report.transport_verdict == "evidence_emitted" {
                "evidence_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: payload_refs.ids,
            artifact_digests_hex: payload_refs.artifact_digests_hex,
            detail: "transport emitted structured withholding evidence instead of a vote"
                .to_string(),
            degraded_mode: true,
        },
        vote: Some(payload_report),
    });

    let mut missing_payload_da = CelestiaLocalAdapter::new("scenario_11_local_da_missing");
    let missing_payload_publish = missing_payload_da.publish(happy.publication_request.clone())?;
    if !missing_payload_da.mark_payload_missing(missing_payload_publish.batch_id) {
        return Err(Scenario11Error::Message(
            "failed to mark Celestia-local payload missing".to_string(),
        ));
    }
    let missing_payload_err = require_error(
        missing_payload_da.resolve(&missing_payload_publish),
        "missing Celestia-local payload unexpectedly resolved",
    )?;
    let missing_blob_refs = evidence_registry.insert(EvidenceRecord::MissingBlob(
        MissingBlobEvidence::new(
            ready_secondary,
            happy.subject.shard_id,
            happy.subject.term,
            happy.subject.membership_digest,
            happy.subject.digest(),
            decode_namespace_hex(&happy.celestia_record.namespace_hex)?,
            happy.celestia_record.blob_commitment,
            happy.commit.certificate.digest(),
            format!(
                "celestia-local payload resolution failed with {:?}",
                missing_payload_err
            ),
        )
        .map_err(reject_record_to_error)?,
    ));
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "celestia_missing_blob".to_string(),
            expected_status: "missing_payload_rejects".to_string(),
            observed_status: if missing_payload_err == DaError::MissingPayload {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: Some(format!("{missing_payload_err:?}")),
            evidence_refs: missing_blob_refs.ids,
            artifact_digests_hex: missing_blob_refs.artifact_digests_hex,
            detail:
                "celestia-local resolve rejected when blob bytes were unavailable inside the challenge window"
                    .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    let mut reconnect_transport = InMemoryVoteTransport::default();
    reconnect_transport.restart_peer(ready_secondary);
    reconnect_transport.reconnect_peer(ready_secondary);
    reconnect_transport.enqueue(VoteTransportEnvelope::available(
        happy.placement.primary_id,
        ready_secondary,
        happy.subject.clone(),
        ShardVoteKind::LocalCommit,
    ));
    let reconnect_envelope = reconnect_transport
        .step()
        .into_iter()
        .next()
        .ok_or_else(|| Scenario11Error::Message("missing reconnect delivery".to_string()))?;
    let mut drifted_placement = happy.placement.clone();
    drifted_placement
        .secondaries
        .retain(|secondary| secondary.aggregator_id == ready_secondary);
    let mut drifted_table = ShardPlacementTable::default();
    drifted_table.insert(drifted_placement.clone());
    let mut reconnect_service = ReplayVerifiedVoteService::local();
    let (reconnect_report, _) = replay_vote_report(
        "restart_reconnect_old_membership",
        ready_secondary,
        reconnect_service.process_envelope(
            &reconnect_envelope,
            VoteExchangeContext {
                voter_role: ShardVoteRole::Secondary,
                replay_request: SecondaryReplayRequest {
                    placement_table: &drifted_table,
                    ..request
                },
            },
        ),
    );
    let reconnect_membership = membership_digest_for_voters(
        drifted_placement.route,
        drifted_placement.primary_id,
        drifted_placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.is_ready)
            .map(|secondary| secondary.aggregator_id),
    );
    let reconnect_refs = evidence_registry.insert_many(
        reconnect_transport
            .fault_records()
            .iter()
            .cloned()
            .map(EvidenceRecord::from)
            .chain(std::iter::once(EvidenceRecord::StaleMember(
                StaleMemberEvidence::new(
                    ready_secondary,
                    happy.subject.shard_id,
                    happy.subject.term,
                    reconnect_membership,
                    happy.subject.membership_digest,
                    reconnect_report.detail.clone(),
                )
                .map_err(reject_record_to_error)?,
            ))),
    );
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "restart_reconnect_old_membership".to_string(),
            expected_status: "old_membership_generation_rejects".to_string(),
            observed_status: if reconnect_report.reject_code.as_deref() == Some("MembershipDrift") {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: reconnect_report.reject_code.clone(),
            evidence_refs: reconnect_refs.ids,
            artifact_digests_hex: reconnect_refs.artifact_digests_hex,
            detail:
                "restart or reconnect could not replay an envelope against a drifted membership"
                    .to_string(),
            degraded_mode: false,
        },
        vote: Some(reconnect_report),
    });

    for (fault_id, mutate, expected_code, detail) in [
        (
            "wrong_route_digest",
            mutate_route_digest as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongRoute,
            "wrong route",
        ),
        (
            "wrong_generation",
            mutate_generation as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongRoute,
            "wrong generation",
        ),
        (
            "wrong_plan_digest",
            mutate_plan_digest as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongPlanDigest,
            "planner digest",
        ),
        (
            "wrong_state_root",
            mutate_state_root as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongRoot,
            "wrong root",
        ),
        (
            "wrong_proof_version",
            mutate_proof_version as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongProofVersion,
            "wrong proof version",
        ),
        (
            "wrong_publication_binding",
            mutate_publication_binding as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongPublicationBinding,
            "wrong publication binding",
        ),
        (
            "wrong_theorem_digest",
            mutate_theorem_digest as fn(&mut CommitSubject),
            SecondaryReplayRejectCode::WrongTheoremDigest,
            "wrong theorem digest",
        ),
    ] {
        let mut claimed = happy.subject.clone();
        mutate(&mut claimed);
        let envelope = VoteTransportEnvelope::available(
            happy.placement.primary_id,
            ready_secondary,
            claimed.clone(),
            ShardVoteKind::LocalCommit,
        );
        let mut service = ReplayVerifiedVoteService::local();
        let (vote, _) = replay_vote_report(
            fault_id,
            ready_secondary,
            service.process_envelope(
                &envelope,
                VoteExchangeContext {
                    voter_role: ShardVoteRole::Secondary,
                    replay_request: request,
                },
            ),
        );
        let observed = vote
            .reject_code
            .clone()
            .unwrap_or_else(|| "accept".to_string());
        let status = if observed == format!("{expected_code:?}") {
            "rejected_as_expected"
        } else {
            "unexpected_result"
        };
        let evidence_refs = match fault_id {
            "wrong_route_digest" => evidence_registry.insert(EvidenceRecord::WrongRouteDigest(
                WrongRouteDigestEvidence::new(
                    ready_secondary,
                    claimed.shard_id,
                    claimed.term,
                    happy.subject.digest(),
                    happy.subject.route_table_digest,
                    claimed.route_table_digest,
                    vote.detail.clone(),
                )
                .map_err(reject_record_to_error)?,
            )),
            "wrong_state_root" => evidence_registry.insert(EvidenceRecord::WrongRoot(
                WrongRootEvidence::new(
                    ready_secondary,
                    claimed.shard_id,
                    claimed.term,
                    happy.subject.digest(),
                    happy.subject.new_state_root.into_bytes(),
                    claimed.new_state_root.into_bytes(),
                    vote.detail.clone(),
                )
                .map_err(reject_record_to_error)?,
            )),
            _ => EvidenceRefs::default(),
        };
        faults.push(DriftFault {
            entry: FaultMatrixEntry {
                scenario_id: "scenario_11".to_string(),
                fault_id: fault_id.to_string(),
                expected_status: "rejected".to_string(),
                observed_status: status.to_string(),
                reject_code: vote.reject_code.clone(),
                evidence_refs: evidence_refs.ids,
                artifact_digests_hex: evidence_refs.artifact_digests_hex,
                detail: detail.to_string(),
                degraded_mode: false,
            },
            vote: Some(vote),
        });
    }

    let below_quorum_err = require_error(
        ConsensusAdapter::from_placement(&happy.placement)
            .map_err(reject_record_to_error)?
            .commit(
                &happy.subject,
                &[ShardVote::new_local(
                    happy.placement.primary_id,
                    ShardVoteRole::Primary,
                    happy.subject.shard_id,
                    happy.subject.term,
                    happy.subject.membership_digest,
                    happy.subject.digest(),
                    ShardVoteKind::LocalCommit,
                )],
            ),
        "below-quorum certificate unexpectedly formed",
    )?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "primary_crash_before_quorum".to_string(),
            expected_status: "no_certificate_no_publication".to_string(),
            observed_status: if below_quorum_err.detail.contains("below quorum") {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "primary crash before quorum produced no certificate".to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "primary_crash_after_quorum_before_da".to_string(),
            expected_status: "resume_same_certificate".to_string(),
            observed_status: if happy.resumed_same_certificate {
                "resumed_same_certificate".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail: "ready secondary resumed publication using the same certificate digest"
                .to_string(),
            degraded_mode: true,
        },
        vote: None,
    });

    let mut takeover_secondaries = vec![SecondaryState::ready(happy.placement.primary_id)];
    takeover_secondaries.extend(
        happy
            .placement
            .secondaries
            .iter()
            .filter(|secondary| secondary.aggregator_id != ready_secondary)
            .cloned(),
    );
    let mut taken_over = ShardPlacementTable::default();
    taken_over.insert(ShardPlacement::new(
        happy.placement.route,
        ready_secondary,
        takeover_secondaries,
        recovery.journal_lineage,
    ));
    let old_primary_reject = require_error(
        RecoveryBoundary.resume(
            happy.placement.primary_id,
            &taken_over,
            &record,
            &recovery,
            RecoveryIntent::RestartPrimary,
        ),
        "old primary unexpectedly restarted after lawful takeover",
    )?;
    faults.push(DriftFault {
        entry: FaultMatrixEntry {
            scenario_id: "scenario_11".to_string(),
            fault_id: "old_primary_restart_after_takeover".to_string(),
            expected_status: "reject_until_lawful_rejoin".to_string(),
            observed_status: if old_primary_reject.detail.contains("live primary owner") {
                "rejected_as_expected".to_string()
            } else {
                "unexpected_result".to_string()
            },
            reject_code: None,
            evidence_refs: vec![],
            artifact_digests_hex: vec![],
            detail:
                "restarted old primary could not reclaim the role after lawful secondary takeover"
                    .to_string(),
            degraded_mode: false,
        },
        vote: None,
    });

    validate_fault_evidence(&faults, &evidence_registry)?;

    Ok(FaultMatrixOutcome {
        faults,
        evidence_registry,
    })
}

fn subject_for_placement(
    term: u64,
    ordered: &OrderedBatch,
    placement: &ShardPlacement,
    candidate: &JournalCandidate,
    publication_binding: &PublicationBinding,
    theorem_digest: [u8; 32],
) -> Result<CommitSubject, Scenario11Error> {
    CommitSubject::from_runtime(
        term,
        membership_digest_for_voters(
            ordered.planned.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        ordered,
        candidate,
        publication_binding,
        theorem_digest,
        None,
    )
    .map_err(reject_record_to_error)
}

fn vote_for_subject(
    subject: &CommitSubject,
    voter_id: AggregatorId,
    voter_role: ShardVoteRole,
) -> ShardVote {
    ShardVote::new_local(
        voter_id,
        voter_role,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    )
}

#[derive(Debug, Clone)]
struct QuorumOnlyCase {
    subject: CommitSubject,
    certificate: ShardQuorumCertificate,
}

fn build_quorum_only_case(
    ordered: &OrderedBatch,
    placement: &ShardPlacement,
    theorem_digest: [u8; 32],
    seed: u8,
) -> Result<QuorumOnlyCase, Scenario11Error> {
    let recovery = route_bound_recovery_state(
        seed,
        ordered.batch_id,
        ordered.planned.route,
        ordered.planned.route_table_digest.into_bytes(),
        placement.expected_journal_lineage,
    )?;
    let record = recovery_record(
        ordered.batch_id,
        ordered.planned.route,
        placement.primary_id,
        placement.secondaries.clone(),
        recovery.clone(),
    )?;
    let candidate = JournalCandidate::from_record(&record).map_err(reject_record_to_error)?;
    let binding = publication_binding_from_roots(
        ordered.batch_id,
        ordered.planned.route_table_digest.into_bytes(),
        SettlementStateRoot::settlement_v1([seed; 32]),
        recovery.state_root,
    )?;
    let subject = CommitSubject::from_runtime(
        9,
        membership_digest_for_voters(
            ordered.planned.route,
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        ),
        ordered,
        &candidate,
        &binding,
        theorem_digest,
        None,
    )
    .map_err(reject_record_to_error)?;
    let primary_vote = ShardVote::new_local(
        placement.primary_id,
        ShardVoteRole::Primary,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let secondary_id = placement
        .secondaries
        .iter()
        .find(|secondary| secondary.is_ready)
        .map(|secondary| secondary.aggregator_id)
        .ok_or_else(|| Scenario11Error::Message("missing ready secondary".to_string()))?;
    let secondary_vote = ShardVote::new_local(
        secondary_id,
        ShardVoteRole::Secondary,
        subject.shard_id,
        subject.term,
        subject.membership_digest,
        subject.digest(),
        ShardVoteKind::LocalCommit,
    );
    let mut adapter =
        ConsensusAdapter::from_placement(placement).map_err(reject_record_to_error)?;
    let commit = adapter
        .commit(&subject, &[primary_vote, secondary_vote])
        .map_err(reject_record_to_error)?;
    Ok(QuorumOnlyCase {
        subject,
        certificate: commit.certificate,
    })
}

fn replay_votes_for_subject(
    case_id: &str,
    subject: &CommitSubject,
    planner: &BatchPlanner,
    topology: &LiveTopology,
    record: &ShardRecoveryRecord,
    recovery: &SettlementRecoveryState,
    publication_binding: &PublicationBinding,
    theorem_digest: [u8; 32],
    placement: &ShardPlacement,
    items: &[WorkItem],
) -> Result<VoteReplayBatchOutcome, Scenario11Error> {
    let ready_secondaries = placement
        .secondaries
        .iter()
        .filter(|secondary| secondary.is_ready)
        .copied()
        .collect::<Vec<_>>();
    let mut transport = InMemoryVoteTransport::default();
    for (index, secondary) in ready_secondaries.iter().enumerate() {
        let envelope = VoteTransportEnvelope::available(
            placement.primary_id,
            secondary.aggregator_id,
            subject.clone(),
            ShardVoteKind::LocalCommit,
        );
        if index == 0 {
            transport.enqueue_delayed(envelope, 1);
        } else {
            transport.enqueue_front(envelope);
        }
    }

    let mut service = ReplayVerifiedVoteService::local();
    let mut reports = Vec::new();
    let mut votes = Vec::new();
    while reports.len() < ready_secondaries.len() {
        for envelope in transport.step() {
            let context = vote_exchange_context(
                envelope.to_id,
                subject,
                planner,
                topology,
                record,
                recovery,
                publication_binding,
                theorem_digest,
                items,
            );
            let result = service.process_envelope(&envelope, context);
            let (report, vote) = replay_vote_report(case_id, envelope.to_id, result);
            if let Some(vote) = vote {
                votes.push(vote);
            }
            reports.push(report);
        }
    }

    Ok(VoteReplayBatchOutcome { reports, votes })
}

fn offline_secondary_case(
    subject: &CommitSubject,
    planner: &BatchPlanner,
    topology: &LiveTopology,
    record: &ShardRecoveryRecord,
    recovery: &SettlementRecoveryState,
    publication_binding: &PublicationBinding,
    theorem_digest: [u8; 32],
    placement: &ShardPlacement,
    items: &[WorkItem],
) -> Result<Vec<SecondaryReplayVoteReport>, Scenario11Error> {
    let mut transport = InMemoryVoteTransport::default();
    let mut service = ReplayVerifiedVoteService::local();
    let online = placement
        .secondaries
        .iter()
        .find(|secondary| secondary.is_ready)
        .ok_or_else(|| Scenario11Error::Message("missing online secondary".to_string()))?;
    transport.enqueue(VoteTransportEnvelope::available(
        placement.primary_id,
        online.aggregator_id,
        subject.clone(),
        ShardVoteKind::LocalCommit,
    ));
    let envelope = transport.step().into_iter().next().ok_or_else(|| {
        Scenario11Error::Message("missing in-memory transport delivery".to_string())
    })?;
    let context = vote_exchange_context(
        online.aggregator_id,
        subject,
        planner,
        topology,
        record,
        recovery,
        publication_binding,
        theorem_digest,
        items,
    );
    let (online_report, _) = replay_vote_report(
        "one_secondary_offline",
        online.aggregator_id,
        service.process_envelope(&envelope, context),
    );
    let mut reports = vec![online_report];
    if let Some(offline) = placement
        .secondaries
        .iter()
        .filter(|secondary| secondary.is_ready)
        .nth(1)
    {
        reports.push(SecondaryReplayVoteReport {
            case_id: "one_secondary_offline".to_string(),
            voter_id: offline.aggregator_id.as_u16(),
            voter_role: "secondary".to_string(),
            verdict: "offline".to_string(),
            transport_verdict: "offline_no_delivery".to_string(),
            signature_scheme: None,
            vote_digest_hex: None,
            evidence_id_hex: None,
            evidence_kind: None,
            artifact_digests_hex: Vec::new(),
            reject_code: None,
            detail: "secondary remained offline and produced no synthetic vote".to_string(),
        });
    }
    Ok(reports)
}

fn stale_secondary_case(
    subject: &CommitSubject,
    planner: &BatchPlanner,
    topology: &LiveTopology,
    record: &ShardRecoveryRecord,
    publication_binding: &PublicationBinding,
    theorem_digest: [u8; 32],
    placement: &ShardPlacement,
    items: &[WorkItem],
) -> Result<Vec<SecondaryReplayVoteReport>, Scenario11Error> {
    let stale_secondary = placement
        .secondaries
        .iter()
        .find(|secondary| secondary.is_ready)
        .ok_or_else(|| Scenario11Error::Message("missing stale secondary".to_string()))?;
    let recovery_route = record.recovery.route.as_ref().ok_or_else(|| {
        Scenario11Error::Message("stale secondary case is missing recovery route".to_string())
    })?;
    let stale_recovery = route_bound_recovery_state(
        0xE1,
        record.batch_id,
        record.placement.route,
        recovery_route.route_table_digest(),
        placement.expected_journal_lineage,
    )?;
    let mut transport = InMemoryVoteTransport::default();
    transport.enqueue(VoteTransportEnvelope::available(
        placement.primary_id,
        stale_secondary.aggregator_id,
        subject.clone(),
        ShardVoteKind::LocalCommit,
    ));
    let envelope =
        transport.step().into_iter().next().ok_or_else(|| {
            Scenario11Error::Message("missing stale transport delivery".to_string())
        })?;
    let context = vote_exchange_context(
        stale_secondary.aggregator_id,
        subject,
        planner,
        topology,
        record,
        &stale_recovery,
        publication_binding,
        theorem_digest,
        items,
    );
    let mut service = ReplayVerifiedVoteService::local();
    let (report, _) = replay_vote_report(
        "stale_secondary",
        stale_secondary.aggregator_id,
        service.process_envelope(&envelope, context),
    );
    Ok(vec![report])
}

fn replay_vote_report(
    case_id: &str,
    voter_id: AggregatorId,
    result: z00z_aggregators::VoteExchangeResult,
) -> (SecondaryReplayVoteReport, Option<ShardVote>) {
    match result.outcome {
        VoteExchangeOutcome::Vote(vote) => (
            SecondaryReplayVoteReport {
                case_id: case_id.to_string(),
                voter_id: voter_id.as_u16(),
                voter_role: "secondary".to_string(),
                verdict: "accept".to_string(),
                transport_verdict: "delivered_in_memory".to_string(),
                signature_scheme: Some(vote.signature_scheme().as_str().to_string()),
                vote_digest_hex: Some(hex::encode(vote.digest())),
                evidence_id_hex: None,
                evidence_kind: None,
                artifact_digests_hex: Vec::new(),
                reject_code: None,
                detail: "secondary replay recomputed the exact primary subject through in-memory transport".to_string(),
            },
            Some(vote),
        ),
        VoteExchangeOutcome::ReplayRejected(reject) => (
            SecondaryReplayVoteReport {
                case_id: case_id.to_string(),
                voter_id: voter_id.as_u16(),
                voter_role: "secondary".to_string(),
                verdict: "reject".to_string(),
                transport_verdict: if reject.class == z00z_aggregators::RejectClass::DeferredRetry {
                    "deferred_retry".to_string()
                } else {
                    "replay_rejected".to_string()
                },
                signature_scheme: None,
                vote_digest_hex: None,
                evidence_id_hex: None,
                evidence_kind: None,
                artifact_digests_hex: Vec::new(),
                reject_code: Some(format!("{:?}", reject.code)),
                detail: reject.detail,
            },
            None,
        ),
        VoteExchangeOutcome::Evidence(evidence) => {
            let record = EvidenceRecord::from(evidence.clone());
            (
                SecondaryReplayVoteReport {
                case_id: case_id.to_string(),
                voter_id: voter_id.as_u16(),
                voter_role: "secondary".to_string(),
                verdict: "degraded".to_string(),
                transport_verdict: "evidence_emitted".to_string(),
                signature_scheme: None,
                vote_digest_hex: None,
                evidence_id_hex: Some(hex::encode(record.digest())),
                evidence_kind: Some(record.kind().as_str().to_string()),
                artifact_digests_hex: artifact_digests_hex(&record),
                reject_code: None,
                detail: format!(
                    "structured evidence emitted instead of vote: {}",
                    evidence_kind_name(&evidence)
                ),
            },
                None,
            )
        }
        VoteExchangeOutcome::DuplicateMessage => (
            SecondaryReplayVoteReport {
                case_id: case_id.to_string(),
                voter_id: voter_id.as_u16(),
                voter_role: "secondary".to_string(),
                verdict: "duplicate".to_string(),
                transport_verdict: "duplicate_message".to_string(),
                signature_scheme: None,
                vote_digest_hex: None,
                evidence_id_hex: None,
                evidence_kind: None,
                artifact_digests_hex: Vec::new(),
                reject_code: None,
                detail: "in-memory transport replayed an already-processed envelope".to_string(),
            },
            None,
        ),
    }
}

fn vote_exchange_context<'a>(
    voter_id: AggregatorId,
    subject: &'a CommitSubject,
    planner: &'a BatchPlanner,
    topology: &'a LiveTopology,
    record: &'a ShardRecoveryRecord,
    recovery: &'a SettlementRecoveryState,
    publication_binding: &'a PublicationBinding,
    theorem_digest: [u8; 32],
    items: &'a [WorkItem],
) -> VoteExchangeContext<'a> {
    VoteExchangeContext {
        voter_role: ShardVoteRole::Secondary,
        replay_request: SecondaryReplayRequest {
            voter_id,
            term: subject.term,
            items,
            planner,
            placement_table: &topology.placement_table,
            recovery_record: record,
            local_recovery: recovery,
            publication_binding,
            theorem_or_settlement_digest: theorem_digest,
            da_availability_digest: None,
        },
    }
}

fn valid_tx_package(tag: &str) -> Result<(TxPackage, CheckRoot), Scenario11Error> {
    let _guard = range_proof_guard();
    let keys = receiver_keys()?;
    let card = keys
        .export_receiver_card()
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let input_asset = asset_fixture(7, 55)?;
    let input_leaf = build_card_stealth_leaf(&card, input_asset.amount, input_asset.serial_id)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let input_wire = bind_stealth_output_wire(AssetWire::from_asset(&input_asset), &input_leaf)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let mut output_wire = input_wire.clone();
    output_wire.nonce[0] ^= 0x55;
    output_wire.leaf_ad_id = Some([0x77; 32]);

    let tx_input = tx_inputs_for_wires(std::slice::from_ref(&input_wire))?
        .pop()
        .ok_or_else(|| Scenario11Error::Message("missing tx input".to_string()))?;
    let tx_output = TxOutputWire {
        role: TxOutRole::Recipient,
        asset_wire: AssetPkgWire::from_wire(&output_wire),
    };
    let proof_inputs = prepare_spend_public_inputs(
        3,
        RECEIVER_SECRET,
        std::slice::from_ref(&input_wire),
        std::slice::from_ref(&tx_input),
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let (prev_root, membership) = prepare_spend_membership_witnesses(
        std::slice::from_ref(&input_wire),
        std::slice::from_ref(&tx_input),
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![tx_input],
        outputs: vec![tx_output],
        fee: 0,
        nonce: 0,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    };
    let (proof, auth) = build_public_spend_contract(
        &keys,
        3,
        1,
        "rollup_settlement",
        &format!("rollup-settlement-{tag}"),
        &tx,
        prev_root,
        proof_inputs,
        SpendProofWitness {
            receiver_secret: ReceiverSecret::from_bytes(RECEIVER_SECRET)
                .map_err(|err| Scenario11Error::Message(err.to_string()))?,
            input_s_in: vec![
                resolve_input_pack(RECEIVER_SECRET, &input_wire)
                    .map_err(|err| Scenario11Error::Message(err.to_string()))?
                    .s_out,
            ],
            membership,
        },
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    tx.proof = proof;
    tx.auth = auth;
    let package = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "rollup_settlement".to_string(),
        chain_name: format!("rollup-settlement-{tag}"),
        tx,
        tx_digest_hex: String::new(),
        status: "prepared".to_string(),
    };
    let mut package = package;
    package.tx_digest_hex = build_tx_package_digest(
        &package.kind,
        &package.package_type,
        package.version,
        package.chain_id,
        &package.chain_type,
        &package.chain_name,
        &package.tx,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    verify_package_public_spend_contract(&package)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let _verifier = TxVerifierImpl;
    Ok((package, prev_root))
}

#[derive(Debug, Clone)]
struct PreparedPublication {
    batch_id: BatchId,
    publication_route: PublicationRouteSnapshotV1,
    draft: CheckpointDraft,
    artifact: CheckpointArtifact,
    tx_package: TxPackage,
    exec_input: CheckpointExecInput,
    link: CheckpointLink,
}

fn prepare_publication_for_package(
    batch_id: BatchId,
    tx_package: TxPackage,
    prev_root: CheckRoot,
    new_root: SettlementStateRoot,
    publication_route: PublicationRouteSnapshotV1,
    replay_id: &str,
) -> Result<PreparedPublication, Scenario11Error> {
    let exec_input = exec_input_from_package(&tx_package, prev_root)?;
    let exec_bytes =
        encode_exec_bin(&exec_input).map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let exec_id = derive_exec_id(&exec_bytes);
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        publication_route.activation_checkpoint.max(11),
        exec_input.prev_root(),
        CheckRoot::new(new_root.into_bytes()),
        Vec::new(),
        Vec::new(),
    );
    let nullifiers = vec![ClaimNullifier::new(
        [batch_id.into_bytes()[0].wrapping_add(0x40); 32],
    )];
    let preview = preview_publication_contract_parts(
        batch_id,
        replay_id,
        &publication_route,
        &draft,
        &tx_package,
        &exec_input,
        &nullifiers,
        CheckpointDaProviderFamily::NamespaceBlob,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let artifact = preview.artifact;
    let checkpoint_id = preview.checkpoint_id;
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        exec_input.prep_snapshot_id(),
        exec_id,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    Ok(PreparedPublication {
        batch_id,
        publication_route,
        draft,
        artifact,
        tx_package,
        exec_input,
        link,
    })
}

fn publication_request_for_package(
    prepared: PreparedPublication,
    ordered_batch: OrderedBatch,
    subject: CommitSubject,
    certificate: ShardQuorumCertificate,
    replay_id: &str,
) -> PublicationRequest {
    PublicationRequest {
        batch_id: prepared.batch_id,
        ordered_batch,
        publication_route: prepared.publication_route,
        draft: prepared.draft,
        subject,
        certificate,
        tx_package: prepared.tx_package,
        exec_input: prepared.exec_input,
        link: prepared.link,
        nullifiers: vec![ClaimNullifier::new(
            [prepared.batch_id.into_bytes()[0].wrapping_add(0x40); 32],
        )],
        idempotency_key: replay_id.to_string(),
    }
}

fn publication_binding_for_prepared(
    prepared: &PreparedPublication,
) -> Result<PublicationBinding, Scenario11Error> {
    let checkpoint_id = derive_checkpoint_id(&prepared.artifact)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    Ok(bind_publication_contract(
        prepared.batch_id,
        checkpoint_id,
        prepared.publication_route.route_table_digest,
        &prepared.artifact.pub_in(),
    ))
}

fn publication_binding_from_roots(
    batch_id: BatchId,
    route_table_digest: [u8; 32],
    prev_root: SettlementStateRoot,
    new_root: SettlementStateRoot,
) -> Result<PublicationBinding, Scenario11Error> {
    let draft = CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        52,
        CheckRoot::new(prev_root.into_bytes()),
        CheckRoot::new(new_root.into_bytes()),
        vec![SpentEnt::new([0x31; 32]), SpentEnt::new([0x32; 32])],
        vec![CreatedEnt::new([0x41; 32], [0x51; 32])],
    );
    let proof = draft
        .attest_proof(SNAPSHOT_ID, CheckpointExecInputId::new([0x71; 32]))
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let artifact = draft
        .finalize(proof)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let checkpoint_id =
        derive_checkpoint_id(&artifact).map_err(|err| Scenario11Error::Message(err.to_string()))?;
    Ok(bind_publication_contract(
        batch_id,
        checkpoint_id,
        route_table_digest,
        &artifact.pub_in(),
    ))
}

fn theorem_digest(prepared: &PreparedPublication) -> Result<[u8; 32], Scenario11Error> {
    let theorem = SettlementTheoremBundle::new(
        prepared.tx_package.clone(),
        prepared.artifact.clone(),
        prepared.exec_input.clone(),
        prepared.link.clone(),
    )?;
    Ok(theorem.theorem_digest())
}

fn receiver_keys() -> Result<ReceiverKeys, Scenario11Error> {
    ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(RECEIVER_SECRET)
            .map_err(|err| Scenario11Error::Message(err.to_string()))?,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))
}

fn asset_fixture(serial_id: u32, amount: u64) -> Result<Asset, Scenario11Error> {
    let definition = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Rollup Settlement Coin".to_string(),
        "RSC".to_string(),
        8,
        1024,
        100_000_000,
        "rollup.settlement.test".to_string(),
        1,
        1,
        0,
        None,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    Ok(Asset::new_confidential(
        std::sync::Arc::new(definition),
        serial_id,
        amount,
        [serial_id as u8; 32],
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?
    .0)
}

fn tx_inputs_for_wires(inputs: &[AssetWire]) -> Result<Vec<TxInputWire>, Scenario11Error> {
    inputs
        .iter()
        .map(|wire| {
            let leaf = asset_wire_to_leaf(wire)
                .map_err(|err| Scenario11Error::Message(err.to_string()))?;
            Ok(TxInputWire {
                asset_id_hex: hex::encode(leaf.asset_id),
                serial_id: wire.serial_id,
            })
        })
        .collect()
}

fn exec_input_from_package(
    package: &TxPackage,
    prev_root: CheckRoot,
) -> Result<CheckpointExecInput, Scenario11Error> {
    let input_refs = package
        .tx
        .inputs
        .iter()
        .map(|input| {
            let asset_id_vec = hex::decode(&input.asset_id_hex)
                .map_err(|err| Scenario11Error::Message(err.to_string()))?;
            let asset_id: [u8; 32] = asset_id_vec.try_into().map_err(|_| {
                Scenario11Error::Message(format!(
                    "asset id must decode to 32 bytes, got {}",
                    input.asset_id_hex
                ))
            })?;
            Ok(CheckpointInRef::new(
                asset_id,
                z00z_storage::settlement::SerialId::new(input.serial_id),
            ))
        })
        .collect::<Result<Vec<_>, Scenario11Error>>()?;
    let outputs = package
        .tx
        .outputs
        .iter()
        .map(|output| {
            let wire = output
                .asset_wire
                .clone()
                .to_wire()
                .map_err(|err| Scenario11Error::Message(err.to_string()))?;
            let leaf = asset_wire_to_leaf(&wire)
                .map_err(|err| Scenario11Error::Message(err.to_string()))?;
            CheckpointExecOut::new(
                z00z_storage::settlement::DefinitionId::new(wire.definition.id),
                leaf,
            )
            .map_err(|err| Scenario11Error::Message(err.to_string()))
        })
        .collect::<Result<Vec<_>, Scenario11Error>>()?;
    let tx_proof = z00z_utils::codec::JsonCodec
        .serialize(&package.tx.proof)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let tx = CheckpointExecTx::new(input_refs, outputs, tx_proof)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        SNAPSHOT_ID,
        prev_root,
        vec![tx],
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))
}

struct RangeProofGuard {
    prev: Option<std::ffi::OsString>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Drop for RangeProofGuard {
    fn drop(&mut self) {
        match &self.prev {
            Some(value) => std::env::set_var("Z00Z_ALLOW_DEBUG_RANGE_PROOF", value),
            None => std::env::remove_var("Z00Z_ALLOW_DEBUG_RANGE_PROOF"),
        }
    }
}

fn range_proof_guard() -> RangeProofGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());
    let prev = std::env::var_os("Z00Z_ALLOW_DEBUG_RANGE_PROOF");
    if prev.as_deref() != Some(std::ffi::OsStr::new("1")) {
        std::env::set_var("Z00Z_ALLOW_DEBUG_RANGE_PROOF", "1");
    }
    RangeProofGuard { prev, _lock: guard }
}

fn simple_tx_item(seed: &str) -> Result<WorkItem, Scenario11Error> {
    let mut pkg = TxPackage {
        kind: "TxPackage".to_string(),
        package_type: "regular_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: format!("z00z-{seed}"),
        tx: TxWire {
            tx_type: "regular_tx".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee: 0,
            nonce: 0,
            context: TxContextWire::default(),
            proof: TxProofWire::default(),
            auth: TxAuthWire::default(),
        },
        tx_digest_hex: String::new(),
        status: "received".to_string(),
    };
    pkg.tx_digest_hex = build_tx_package_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    z00z_aggregators::IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(pkg)))
        .map_err(reject_record_to_error)
}

fn simple_claim_item(seed: &str) -> Result<WorkItem, Scenario11Error> {
    let mut pkg = ClaimTxPackage {
        kind: "ClaimTxPackage".to_string(),
        package_type: "claim_tx".to_string(),
        version: 1,
        chain_id: 3,
        chain_type: "devnet".to_string(),
        chain_name: format!("z00z-{seed}"),
        tx: ClaimTxWire {
            tx_type: "claim_tx".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee: 0,
            nonce: 0,
            context: ClaimContextWire {
                recipient_wallet_id: "wallet".to_string(),
                recipient_owner_hex: "00".repeat(32),
                claim_scope_hash_hex: "11".repeat(32),
                recipient_card_hex: None,
                nullifier_hex: "22".repeat(32),
            },
            proof: ClaimProofWire {
                proof_type: "genesis_claim".to_string(),
                proof_hex: "33".repeat(32),
            },
            auth: ClaimAuthWire {
                claim_authority_sig_hex: "44".repeat(64),
            },
        },
        tx_digest_hex: String::new(),
        status: "received".to_string(),
    };
    pkg.tx_digest_hex = z00z_wallets::tx::build_claim_tx_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    z00z_aggregators::IngressBoundary
        .normalize(WorkPayload::Claim(Box::new(pkg)))
        .map_err(reject_record_to_error)
}

fn find_simple_item_for_shard(
    table: &ShardRouteTable,
    shard_id: u16,
    prefix: &str,
) -> Result<WorkItem, Scenario11Error> {
    let wanted = z00z_aggregators::ShardId::new(shard_id);
    for index in 0..20_000u32 {
        let label = format!("{prefix}-{shard_id}-{index}");
        let item = if index % 2 == 0 {
            simple_tx_item(&label)?
        } else {
            simple_claim_item(&label)?
        };
        if table
            .lookup(route_key(&item)?)
            .map_err(|err| Scenario11Error::Message(err.to_string()))?
            == wanted
        {
            return Ok(item);
        }
    }
    Err(Scenario11Error::Message(format!(
        "failed to synthesize routeable work item for shard {shard_id}"
    )))
}

fn route_bound_recovery_state(
    seed: u8,
    batch_id: BatchId,
    route: BatchRoute,
    route_table_digest: [u8; 32],
    expected_journal_lineage: [u8; 32],
) -> Result<SettlementRecoveryState, Scenario11Error> {
    let temp = ScratchDir::new("scenario11-recovery")?;
    let mut store = SettlementStore::load(temp.path())
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let spent_path = settlement_path(seed);
    let output_path = settlement_path(seed.wrapping_add(0x20));
    let output = settlement_item(output_path, 9_100 + u64::from(seed))?;
    store
        .apply_settlement_ops(vec![StoreOp::Put(Box::new(settlement_item(
            spent_path,
            9_000 + u64::from(seed),
        )?))])
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let terminal_output = output
        .terminal_leaf()
        .cloned()
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let exec_out = CheckpointExecOut::new(output.path().definition_id, terminal_output)
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let exec_tx = CheckpointExecTx::new(
        vec![CheckpointInRef::new(
            spent_path.terminal_id().into_bytes(),
            spent_path.serial_id,
        )],
        vec![exec_out],
        b"route-bound-durable-recovery".to_vec(),
    )
    .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    store
        .apply_exec_handoff(z00z_storage::settlement::SettlementExecHandoff::new(
            SettlementRouteCtx::new(
                batch_id.into_bytes(),
                route.shard_id.as_u32(),
                route.routing_generation,
                route_table_digest,
            ),
            vec![
                StoreOp::Delete(spent_path),
                StoreOp::Put(Box::new(output.clone())),
            ],
            vec![exec_tx],
        ))
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    let mut recovery = store
        .recovery_state()
        .map_err(|err| Scenario11Error::Message(err.to_string()))?;
    recovery.journal_lineage = expected_journal_lineage;
    Ok(recovery)
}

fn recovery_record(
    batch_id: BatchId,
    route: BatchRoute,
    primary: AggregatorId,
    secondaries: Vec<SecondaryState>,
    recovery: SettlementRecoveryState,
) -> Result<ShardRecoveryRecord, Scenario11Error> {
    let placement = ShardPlacement::new(route, primary, secondaries, recovery.journal_lineage);
    let ticket = ShardExecTicket {
        batch_id,
        placement: placement.view(),
        state: ShardExecState::Routed,
    };
    let boundary = z00z_aggregators::RecoveryBoundary;
    let publication = boundary.mark_handed_off(ticket.batch_id);
    boundary
        .capture(&ticket, &publication, recovery)
        .map_err(reject_record_to_error)
}

#[derive(Debug)]
struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    fn new(prefix: &str) -> Result<Self, Scenario11Error> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let seq = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{seq}", std::process::id()));
        if path.exists() {
            remove_dir_all(&path)?;
        }
        create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = remove_dir_all(&self.path);
        }
    }
}

fn settlement_path(seed: u8) -> z00z_storage::settlement::SettlementPath {
    z00z_storage::settlement::SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new([seed; 32]),
        z00z_storage::settlement::SerialId::new(u32::from(seed) + 1),
        z00z_storage::settlement::TerminalId::new([seed.wrapping_add(1); 32]),
    )
}

fn settlement_item(
    path: z00z_storage::settlement::SettlementPath,
    value: u64,
) -> Result<StoreItem, Scenario11Error> {
    let payload = z00z_core::assets::AssetPackPlain {
        value,
        blinding: [3u8; 32],
        s_out: [4u8; 32],
    }
    .to_bytes();
    let leaf: z00z_storage::settlement::TerminalLeaf = z00z_core::assets::AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1u8; 32],
        owner_tag: [2u8; 32],
        c_amount: [5u8; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into();
    StoreItem::new(path, leaf).map_err(|err| Scenario11Error::Message(err.to_string()))
}

fn batch_id(label: &str) -> BatchId {
    let digest: [u8; 32] = Sha256::digest(label.as_bytes()).into();
    BatchId::from_bytes(digest)
}

fn route_key(item: &WorkItem) -> Result<[u8; 32], Scenario11Error> {
    let raw =
        hex::decode(item.digest_hex()).map_err(|err| Scenario11Error::Message(err.to_string()))?;
    raw.try_into().map_err(|_| {
        Scenario11Error::Message(format!(
            "route key digest must decode to 32 bytes, got {}",
            item.digest_hex()
        ))
    })
}

fn secondary_ids(secondaries: &[SecondaryState]) -> Vec<u16> {
    secondaries
        .iter()
        .map(|secondary| secondary.aggregator_id.as_u16())
        .collect()
}

fn ready_secondary_ids(secondaries: &[SecondaryState]) -> Vec<u16> {
    secondaries
        .iter()
        .filter(|secondary| secondary.is_ready)
        .map(|secondary| secondary.aggregator_id.as_u16())
        .collect()
}

fn quorum_threshold(placement: &ShardPlacement) -> usize {
    let members = 1 + placement
        .secondaries
        .iter()
        .filter(|secondary| secondary.is_ready)
        .count();
    (members / 2) + 1
}

fn dispatch_stage_name(stage: DispatchStage) -> &'static str {
    match stage {
        DispatchStage::Delivered => "delivered",
        DispatchStage::Deferred => "deferred",
        DispatchStage::Duplicate => "duplicate",
    }
}

fn evidence_kind_name(evidence: &z00z_aggregators::VoteEvidence) -> &'static str {
    match evidence.kind() {
        z00z_aggregators::VoteEvidenceKind::Equivocation => "equivocation",
        z00z_aggregators::VoteEvidenceKind::PayloadWithholding => "payload_withholding",
    }
}

fn verdict_kind_name(kind: &VerdictKind) -> &'static str {
    match kind {
        VerdictKind::Accepted => "accepted",
        VerdictKind::Rejected => "rejected",
        VerdictKind::Incomplete => "incomplete",
    }
}

fn mutate_route_digest(subject: &mut CommitSubject) {
    subject.route_table_digest[0] ^= 0x55;
}

fn mutate_generation(subject: &mut CommitSubject) {
    subject.routing_generation = subject.routing_generation.saturating_add(1);
}

fn mutate_plan_digest(subject: &mut CommitSubject) {
    subject.plan_digest[0] ^= 0x22;
}

fn mutate_state_root(subject: &mut CommitSubject) {
    subject.new_state_root = SettlementStateRoot::settlement_v1([0xAA; 32]);
}

fn mutate_proof_version(subject: &mut CommitSubject) {
    subject.proof_version = subject.proof_version.saturating_add(1);
}

fn mutate_publication_binding(subject: &mut CommitSubject) {
    subject.publication_binding_digest[0] ^= 0x33;
}

fn mutate_theorem_digest(subject: &mut CommitSubject) {
    subject.theorem_or_settlement_digest[0] ^= 0x44;
}

fn reject_record_to_error(err: z00z_aggregators::RejectRecord) -> Scenario11Error {
    Scenario11Error::Message(format!("{:?}: {}", err.class, err.detail))
}

fn consensus_store_error_to_error(err: z00z_aggregators::ConsensusStoreError) -> Scenario11Error {
    Scenario11Error::Message(err.to_string())
}
