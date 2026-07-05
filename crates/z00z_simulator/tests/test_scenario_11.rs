use std::{fs, path::PathBuf};

use tempfile::tempdir;
use z00z_simulator::scenario_11::{
    report::{
        CommitSubjectReport, ConsensusStoreReport, FaultMatrixReport, LocalDaBindingReport,
        PackageIngressReport, PlacementMembershipReport, QuorumCertificateReport, ReportHonesty,
        RoutePlanReport, SecondaryReplayVotesReport, ValidatorVerdictReport, CLAIM_LEVEL_LIVE,
        CLAIM_LEVEL_LIVE_CLAIM_REMOVED, PLANNER_AUTHORITY_MODEL_DETERMINISTIC_REPLICATED,
        TERM_DETERMINISTIC_REPLICATED_PLANNER, TERM_PLANNER_HA,
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
    let votes: SecondaryReplayVotesReport =
        load_json(root.join("secondary_replay_votes.json")).expect("votes report");

    let reject_ids = [
        "wrong_route_digest",
        "wrong_generation",
        "wrong_plan_digest",
        "wrong_state_root",
        "wrong_proof_version",
        "wrong_publication_binding",
        "wrong_theorem_digest",
        "observer_not_ready_before_readiness",
        "removed_member_vote",
        "mixed_generation_certificate",
    ];
    for fault_id in reject_ids {
        assert_fault_status(&faults, fault_id, "rejected_as_expected");
    }

    for (fault_id, expected_status) in [
        ("observer_ready_after_catchup", "accepted_as_expected"),
        ("one_secondary_offline", "degraded_as_expected"),
        ("one_secondary_stale", "rejected_as_expected"),
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

    assert!(votes
        .offline_case_votes
        .iter()
        .any(|vote| vote.verdict == "offline"));
    assert!(votes
        .stale_case_votes
        .iter()
        .any(|vote| vote.reject_code.as_deref() == Some("StaleSecondaryState")));
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
        .any(|line| line.contains("Celestia finality")));
    assert!(honesty
        .forbidden_claims
        .iter()
        .any(|line| line.contains(TERM_PLANNER_HA)));
    assert!(honesty
        .deferred_claims
        .iter()
        .any(|line| line.contains("067-08")));
    assert!(honesty.claim_levels.iter().any(|entry| {
        entry.term == TERM_DETERMINISTIC_REPLICATED_PLANNER && entry.claim_level == CLAIM_LEVEL_LIVE
    }));
    assert!(honesty.claim_levels.iter().any(|entry| {
        entry.term == TERM_PLANNER_HA && entry.claim_level == CLAIM_LEVEL_LIVE_CLAIM_REMOVED
    }));
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
        ("primary_crash_after_quorum_before_da", "resumed_same_certificate"),
        ("primary_offline_before_dispatch", "deferred_as_expected"),
        ("one_secondary_offline", "degraded_as_expected"),
        ("one_secondary_stale", "rejected_as_expected"),
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
