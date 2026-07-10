use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use tempfile::tempdir;
use z00z_simulator::scenario_11::{
    report::{
        CommitSubjectReport, ConsensusStoreReport, EvidenceRegistryReport, FaultMatrixReport,
        LocalDaBindingReport, PackageIngressReport, PlacementMembershipReport,
        QuorumCertificateReport, ReportHonesty, RoutePlanReport, SecondaryReplayVotesReport,
        ValidatorVerdictReport, CLAIM_LEVEL_LIVE, CLAIM_LEVEL_LIVE_CLAIM_REMOVED,
        CLAIM_LEVEL_SIMULATED_FULL, PLANNER_AUTHORITY_MODEL_DETERMINISTIC_REPLICATED,
        TERM_CELESTIA_FINALITY, TERM_DETERMINISTIC_REPLICATED_PLANNER, TERM_PLANNER_HA,
    },
    run,
};
use z00z_utils::io::load_json;

const SCENARIO11_ARTIFACT_ROOT_ENV: &str = "Z00Z_HJMT_SCENARIO11_ARTIFACT_ROOT";

#[test]
fn scenario11_happy_path_consistent() {
    let output = scenario11_output_root("happy-path");
    let run = run(&output.root).expect("scenario_11 run");
    let root = run.artifact_root();

    let ingress: PackageIngressReport =
        load_json(root.join("package_ingress_report.json")).expect("ingress report");
    let route: RoutePlanReport =
        load_json(root.join("route_plan_report.json")).expect("route report");
    let placement: PlacementMembershipReport =
        load_json(root.join("placement_membership.json")).expect("placement report");
    let subject: CommitSubjectReport =
        load_json(root.join("commit_subject.json")).expect("subject report");
    let votes: SecondaryReplayVotesReport =
        load_json(root.join("secondary_replay_votes.json")).expect("votes report");
    let qc: QuorumCertificateReport =
        load_json(root.join("quorum_certificate.json")).expect("qc report");
    let da: LocalDaBindingReport =
        load_json(root.join("local_da_binding.json")).expect("da report");
    let store: ConsensusStoreReport =
        load_json(root.join("consensus_store_report.json")).expect("store report");
    let verdict: ValidatorVerdictReport =
        load_json(root.join("validator_verdict_report.json")).expect("verdict report");

    assert!(root.join("fault_matrix.json").exists());
    assert!(root.join("evidence_registry.json").exists());
    assert!(root.join("report_honesty.json").exists());

    assert!(ingress.ingress_recomputed_digest);
    assert_eq!(ingress.package_digest_hex, ingress.route_key_hex);
    assert_eq!(ingress.batch_id_hex, subject.batch_id_hex);
    assert_eq!(route.happy_path.shard_id, ingress.shard_id);
    assert_eq!(route.planner_mode, "central");
    assert_eq!(
        route.planner_authority_model,
        PLANNER_AUTHORITY_MODEL_DETERMINISTIC_REPLICATED
    );
    assert_eq!(route.planner_ha_claim_level, CLAIM_LEVEL_LIVE_CLAIM_REMOVED);
    assert_eq!(route.planner_config_digest_hex.len(), 64);
    assert_eq!(route.planner_authority_digest_hex.len(), 64);
    assert_eq!(
        route.happy_path.route_table_digest_hex,
        subject.route_table_digest_hex
    );
    assert_eq!(route.happy_path.batch_id_hex, subject.batch_id_hex);
    assert_eq!(
        placement.happy_path.membership_digest_hex,
        subject.membership_digest_hex
    );
    assert_eq!(qc.happy_path.subject_digest_hex, subject.subject_digest_hex);
    assert_eq!(
        qc.happy_path.membership_digest_hex,
        subject.membership_digest_hex
    );
    assert_eq!(
        da.publication_binding_digest_hex,
        subject.publication_binding_digest_hex
    );
    assert_eq!(da.claim_level, CLAIM_LEVEL_SIMULATED_FULL);
    assert_eq!(da.namespace_hex.len(), 16);
    assert_eq!(da.blob_commitment_hex.len(), 64);
    assert!(da
        .inclusion_reference
        .starts_with("celestia-local-inclusion://"));
    assert!(da.retention_until_height >= da.blob_height);
    assert!(!da.degraded_mode);
    assert!(da.payload_available);
    assert!(da.resumed_same_certificate);
    assert_eq!(store.subject_digest_hex, subject.subject_digest_hex);
    assert_eq!(
        store.certificate_digest_hex,
        qc.happy_path.certificate_digest_hex
    );
    assert_eq!(
        store.publication_binding_digest_hex,
        da.publication_binding_digest_hex
    );
    assert_eq!(store.validator_verdict_kind, verdict.verdict_kind);
    assert_eq!(store.resumed_by_secondary_id, da.resumed_by_secondary_id);
    assert_eq!(store.resume_source, "reloaded_from_store");
    assert_eq!(verdict.verdict_kind, "accepted");
    assert_eq!(verdict.subject_digest_hex, subject.subject_digest_hex);
    assert_eq!(
        verdict.certificate_digest_hex,
        qc.happy_path.certificate_digest_hex
    );
    assert!(root.join("consensus_store/batches").exists());
    assert!(root.join("consensus_store/routes").exists());
    assert_eq!(route.all_shard_sweep.len(), 7);
    assert_eq!(placement.all_shard_sweep.len(), 7);
    assert_eq!(route.authority_replicas.len(), 5);
    assert!(route
        .authority_replicas
        .iter()
        .all(|row| row.recomputed_plan_digest_hex == route.happy_path.plan_digest_hex));
    assert_eq!(route.dual_primary_owner.shard_ids.len(), 2);
    assert_eq!(
        route.dual_primary_owner.membership_digests_hex.len(),
        route.dual_primary_owner.certificate_digests_hex.len()
    );
    assert!(
        route.dual_primary_owner.membership_digests_hex[0]
            != route.dual_primary_owner.membership_digests_hex[1]
    );
    assert!(
        route.dual_primary_owner.certificate_digests_hex[0]
            != route.dual_primary_owner.certificate_digests_hex[1]
    );
    assert!(votes.happy_path_votes.iter().all(|vote| {
        vote.verdict == "accept"
            && vote.transport_verdict == "delivered_in_memory"
            && vote.signature_scheme.as_deref() == Some("deterministic_local")
    }));
}

#[test]
fn scenario11_fault_matrix_covers() {
    let output = scenario11_output_root("fault-matrix");
    let run = run(&output.root).expect("scenario_11 run");
    let root = run.artifact_root();

    let faults: FaultMatrixReport =
        load_json(root.join("fault_matrix.json")).expect("fault matrix");
    let registry: EvidenceRegistryReport =
        load_json(root.join("evidence_registry.json")).expect("evidence registry");
    let votes: SecondaryReplayVotesReport =
        load_json(root.join("secondary_replay_votes.json")).expect("votes report");

    let registry_ids = registry
        .entries
        .iter()
        .map(|entry| entry.evidence_id_hex.as_str())
        .collect::<BTreeSet<_>>();
    let registry_kinds = registry
        .entries
        .iter()
        .map(|entry| entry.evidence_kind.as_str())
        .collect::<BTreeSet<_>>();

    for kind in [
        "equivocation",
        "payload_withholding",
        "missing_blob",
        "wrong_root",
        "wrong_route_digest",
        "stale_member",
        "split_brain",
    ] {
        assert!(
            registry_kinds.contains(kind),
            "missing registry kind {kind}"
        );
    }
    assert!(faults
        .entries
        .iter()
        .flat_map(|entry| entry.evidence_refs.iter())
        .all(|raw| is_hex_digest(raw)));
    assert!(faults
        .entries
        .iter()
        .flat_map(|entry| entry.evidence_refs.iter())
        .all(|raw| registry_ids.contains(raw.as_str())));

    let reject_ids = [
        "celestia_missing_blob",
        "wrong_route_digest",
        "wrong_generation",
        "wrong_plan_digest",
        "wrong_state_root",
        "wrong_proof_version",
        "wrong_publication_binding",
        "wrong_theorem_digest",
        "old_primary_restart_after_takeover",
        "restart_reconnect_old_membership",
        "observer_not_ready_before_readiness",
        "removed_member_vote",
        "mixed_generation_certificate",
    ];
    for fault_id in reject_ids {
        assert_fault_status(&faults, fault_id, "rejected_as_expected");
    }

    for (fault_id, expected_status) in [
        ("equivocation_same_voter", "evidence_as_expected"),
        ("observer_ready_after_catchup", "accepted_as_expected"),
        ("one_secondary_offline", "degraded_as_expected"),
        ("one_secondary_stale", "rejected_as_expected"),
        ("transport_duplicate_replay", "ignored_as_expected"),
        ("transport_payload_withholding", "evidence_as_expected"),
        ("same_term_divergent_root_freeze", "frozen_as_expected"),
        ("partition_and_heal", "healed_without_conflict"),
        (
            "rolling_primary_takeover_continuity",
            "continued_as_expected",
        ),
    ] {
        assert_fault_status(&faults, fault_id, expected_status);
    }

    assert_fault_status(
        &faults,
        "primary_crash_before_quorum",
        "rejected_as_expected",
    );
    assert_fault_status(
        &faults,
        "primary_offline_before_dispatch",
        "deferred_as_expected",
    );
    assert_fault_status(
        &faults,
        "primary_crash_after_quorum_before_da",
        "resumed_same_certificate",
    );
    assert_fault_status(
        &faults,
        "old_primary_restart_after_takeover",
        "rejected_as_expected",
    );
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
            .entries
            .iter()
            .find(|entry| entry.fault_id == fault_id)
            .expect("structured evidence fault");
        assert!(
            !entry.evidence_refs.is_empty(),
            "fault {fault_id} must carry evidence ids"
        );
        assert!(
            !entry.artifact_digests_hex.is_empty(),
            "fault {fault_id} must carry artifact digests"
        );
    }

    assert!(votes
        .offline_case_votes
        .iter()
        .any(|vote| vote.verdict == "offline"));
    assert!(votes
        .stale_case_votes
        .iter()
        .any(|vote| vote.reject_code.as_deref() == Some("StaleSecondaryState")));
    assert!(votes
        .drift_case_votes
        .iter()
        .any(|vote| vote.case_id == "equivocation_same_voter"
            && vote.evidence_kind.as_deref() == Some("equivocation")));
    for vote in votes
        .drift_case_votes
        .iter()
        .filter(|vote| vote.evidence_id_hex.is_some())
    {
        let evidence_id = vote
            .evidence_id_hex
            .as_deref()
            .expect("evidence id must exist for structured vote");
        let registry_entry = registry
            .entries
            .iter()
            .find(|entry| entry.evidence_id_hex == evidence_id)
            .expect("structured vote evidence id must resolve in registry");
        assert_eq!(
            vote.evidence_kind.as_deref(),
            Some(registry_entry.evidence_kind.as_str()),
            "vote {} must preserve the canonical evidence kind",
            vote.case_id
        );
        assert_eq!(
            vote.artifact_digests_hex, registry_entry.artifact_digests_hex,
            "vote {} must preserve the canonical evidence artifacts",
            vote.case_id
        );
    }
    assert!(votes
        .drift_case_votes
        .iter()
        .any(|vote| vote.case_id == "transport_duplicate_replay"
            && vote.transport_verdict == "duplicate_message"));
    assert!(votes
        .drift_case_votes
        .iter()
        .any(|vote| vote.case_id == "transport_payload_withholding"
            && vote.transport_verdict == "evidence_emitted"));
    assert!(votes
        .drift_case_votes
        .iter()
        .any(|vote| vote.case_id == "restart_reconnect_old_membership"
            && vote.reject_code.as_deref() == Some("MembershipDrift")));
}

#[test]
fn scenario11_report_honesty_rejects_overclaims() {
    let output = scenario11_output_root("report-honesty");
    let run = run(&output.root).expect("scenario_11 run");
    let honesty: ReportHonesty =
        load_json(run.artifact_root().join("report_honesty.json")).expect("honesty report");

    assert!(honesty
        .supported_claims
        .iter()
        .any(|line| line.contains("local per-shard 2-of-3 CFT quorum")));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains("network BFT")));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains(TERM_CELESTIA_FINALITY)));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains("production HotStuff")));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains(TERM_PLANNER_HA)));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains("devnet")));
    assert!(honesty
        .deferred_claims
        .iter()
        .any(|line| line.contains("067-08")));
    assert!(honesty
        .deferred_claims
        .iter()
        .any(|line| line.contains("real Celestia finality")));
    assert!(honesty
        .deferred_claims
        .iter()
        .any(|line| line.contains("production HotStuff")));
    assert!(honesty.claim_levels.iter().any(|entry| {
        entry.term == TERM_DETERMINISTIC_REPLICATED_PLANNER && entry.claim_level == CLAIM_LEVEL_LIVE
    }));
    assert!(honesty.claim_levels.iter().any(|entry| {
        entry.term == TERM_PLANNER_HA && entry.claim_level == CLAIM_LEVEL_LIVE_CLAIM_REMOVED
    }));
    assert!(honesty.claim_levels.iter().any(|entry| {
        entry.term == "Celestia" && entry.claim_level == CLAIM_LEVEL_SIMULATED_FULL
    }));
    assert!(honesty.claim_levels.iter().any(|entry| {
        entry.term == TERM_CELESTIA_FINALITY && entry.claim_level == CLAIM_LEVEL_LIVE_CLAIM_REMOVED
    }));
}

#[test]
fn scenario11_claim_registry_matches_report() {
    let output = scenario11_output_root("claim-registry");
    let run = run(&output.root).expect("scenario_11 run");
    let honesty: ReportHonesty =
        load_json(run.artifact_root().join("report_honesty.json")).expect("honesty report");
    let registry = load_claim_registry().expect("claim registry");
    let report_terms = honesty
        .claim_levels
        .iter()
        .map(|entry| (entry.term.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(honesty.claim_levels.len(), report_terms.len());
    assert_eq!(registry.len(), report_terms.len());
    for (term, claim_level) in registry {
        let report_entry = report_terms
            .get(&term)
            .unwrap_or_else(|| panic!("missing report term {term}"));
        assert_eq!(report_entry.claim_level, claim_level);
        assert!(
            !report_entry.evidence_refs.is_empty(),
            "term {term} must carry evidence refs"
        );
    }
}

#[test]
fn scenario11_process_devnet_fault_contract() {
    let output = scenario11_output_root("process-devnet-contract");
    let run = run(&output.root).expect("scenario_11 run");
    let root = run.artifact_root();
    let faults: FaultMatrixReport =
        load_json(root.join("fault_matrix.json")).expect("fault matrix");
    let honesty: ReportHonesty =
        load_json(root.join("report_honesty.json")).expect("honesty report");

    for (fault_id, expected_status) in [
        ("primary_crash_before_quorum", "rejected_as_expected"),
        (
            "primary_crash_after_quorum_before_da",
            "resumed_same_certificate",
        ),
        ("old_primary_restart_after_takeover", "rejected_as_expected"),
        ("primary_offline_before_dispatch", "deferred_as_expected"),
        ("one_secondary_offline", "degraded_as_expected"),
        ("one_secondary_stale", "rejected_as_expected"),
        ("transport_duplicate_replay", "ignored_as_expected"),
        ("restart_reconnect_old_membership", "rejected_as_expected"),
        ("partition_and_heal", "healed_without_conflict"),
    ] {
        assert_fault_status(&faults, fault_id, expected_status);
    }
    assert!(honesty
        .simulated_markers
        .iter()
        .any(|line| line.contains("remote process crash or resume is simulated")));
    assert!(honesty
        .supported_claims
        .iter()
        .any(|line| line.contains("local per-shard 2-of-3 CFT quorum")));
}

fn assert_fault_status(faults: &FaultMatrixReport, fault_id: &str, expected_status: &str) {
    let entry = faults
        .entries
        .iter()
        .find(|entry| entry.fault_id == fault_id)
        .expect("fault entry");
    assert_eq!(entry.observed_status, expected_status);
}

fn is_hex_digest(raw: &str) -> bool {
    raw.len() == 64 && raw.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn load_claim_registry() -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(claim_registry_path())?;
    let mut in_table = false;
    let mut rows = BTreeMap::new();
    for line in raw.lines() {
        if line.starts_with("| term | code owner | artifact/API |") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if line.starts_with("| ---") {
            continue;
        }
        if !line.starts_with('|') {
            break;
        }
        let parts = line
            .trim_matches('|')
            .split('|')
            .map(|part| part.trim().to_string())
            .collect::<Vec<_>>();
        if parts.len() != 8 {
            return Err(format!("claim registry row must have 8 columns: {line}").into());
        }
        let prior = rows.insert(parts[0].clone(), parts[5].clone());
        if prior.is_some() {
            return Err(format!("duplicate claim registry term: {}", parts[0]).into());
        }
    }
    Ok(rows)
}

fn claim_registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.planning/phases/000/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md")
}

struct Scenario11OutputRoot {
    root: PathBuf,
    _temp: Option<tempfile::TempDir>,
}

fn scenario11_output_root(label: &str) -> Scenario11OutputRoot {
    if let Ok(raw) = std::env::var(SCENARIO11_ARTIFACT_ROOT_ENV) {
        let root = PathBuf::from(raw).join(label);
        fs::create_dir_all(&root).expect("create scenario11 artifact root");
        return Scenario11OutputRoot { root, _temp: None };
    }
    let temp = tempdir().expect("tempdir");
    let root = temp.path().join(label);
    fs::create_dir_all(&root).expect("create temp scenario11 root");
    Scenario11OutputRoot {
        root,
        _temp: Some(temp),
    }
}
