use std::sync::{Mutex, MutexGuard, OnceLock};

use z00z_simulator::config::{ObjectFlowCaseCfg, ObjectFlowMatrixCfg, ScenarioCfg};
use z00z_simulator::scenario_1::{stage_13::shared_cases, support::stage_runner_support};
use z00z_storage::settlement::{
    DefinitionId, RightAction, RightActionCtx, RightClass, RightErr, RightLeaf, SerialId,
    SettlementPath, TerminalId,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_to_string,
};

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

struct HjmtE2eGuard {
    _process_guard: stage_runner_support::ProcessLock,
    _thread_guard: MutexGuard<'static, ()>,
}

fn hjmt_e2e_lock() -> HjmtE2eGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    HjmtE2eGuard {
        _process_guard: stage_runner_support::acquire_process_lock(),
        _thread_guard: LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()),
    }
}

fn live_out(case_suffix: &str) -> std::path::PathBuf {
    shared_cases::stage13_out(case_suffix)
}

fn live_cfg(out_dir: &std::path::Path) -> std::path::PathBuf {
    out_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("live case base")
        .join("scenario_config.yaml")
}

fn read_json(out_dir: &std::path::Path, name: &str) -> serde_json::Value {
    JsonCodec
        .deserialize(
            read_to_string(out_dir.join(name))
                .expect("read json")
                .as_bytes(),
        )
        .expect("parse json")
}

fn wallet_lifecycle_row<'a>(
    hist_flow: &'a serde_json::Value,
    case_id: &str,
) -> &'a serde_json::Value {
    hist_flow["wallet_lifecycle_rows"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|row| row["case_id"].as_str() == Some(case_id))
        })
        .unwrap_or_else(|| panic!("missing wallet_lifecycle_rows case {}", case_id))
}

fn object_flow_matrix(out_dir: &std::path::Path) -> ObjectFlowMatrixCfg {
    ScenarioCfg::from_file(live_cfg(out_dir))
        .expect("load localized scenario cfg")
        .object_flow_matrix
        .expect("object flow matrix")
}

fn matrix_row<'a>(rows: &'a [ObjectFlowCaseCfg], id: &str) -> &'a ObjectFlowCaseCfg {
    rows.iter()
        .find(|row| row.id == id)
        .unwrap_or_else(|| panic!("missing object flow row {id}"))
}

fn right_fixture(class: RightClass) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: TerminalId::new([0x41; 32]),
        right_class: class,
        issuer_scope: [0x42; 32],
        provider_scope: [0x43; 32],
        holder_commitment: [0x44; 32],
        control_commitment: [0x45; 32],
        beneficiary_commitment: [0x46; 32],
        payload_commitment: [0x47; 32],
        valid_from: 1,
        valid_until: 100,
        challenge_from: 10,
        challenge_until: 80,
        use_nonce: [0x48; 32],
        revocation_policy_id: [0x49; 32],
        transition_policy_id: [0x4A; 32],
        challenge_policy_id: [0x4B; 32],
        disclosure_policy_id: [0x4C; 32],
        retention_policy_id: [0x4D; 32],
    }
}

fn wrong_path() -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([0x51; 32]),
        SerialId::new(7),
        TerminalId::new([0x52; 32]),
    )
}

#[test]
fn test_e2e_acceptance_homes_live() {
    let _guard = hjmt_e2e_lock();
    let out_dir = live_out("test_hjmt_e2e_acceptance_homes_live");
    z00z_simulator::scenario_1::runner::validate_runtime_observability_artifacts(
        live_cfg(&out_dir),
        "src/scenario_1/scenario_design.yaml",
        &out_dir,
    )
    .expect("runtime observability packet");

    let run_meta = read_json(&out_dir, "run_meta.json");
    let scope_flow = read_json(&out_dir, "scope_flow.json");
    let wallet_scan = read_json(&out_dir, "wallet_scan.json");
    let hist_flow = read_json(&out_dir, "hist_flow.json");
    let occ_flow = read_json(&out_dir, "occ_flow.json");
    let sim_summary = read_to_string(out_dir.join("sim_summary.md")).expect("sim summary");

    assert_eq!(run_meta["execution_mode"].as_str(), Some("release"));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("wallet_scan.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("hist_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("occ_flow.json")
                && row["status"].as_str() == Some("emitted"))));

    assert!(scope_flow["acceptance_homes"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["home"].as_str()
            == Some("crates/z00z_storage/tests/test_hjmt_transition_proofs.rs")
            && row["status"].as_str() == Some("live_home"))));
    assert!(scope_flow["acceptance_homes"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["home"].as_str()
            == Some("crates/z00z_storage/tests/test_hjmt_privacy_regression.rs")
            && row["status"].as_str() == Some("live_home"))));
    assert!(scope_flow["acceptance_homes"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["home"].as_str()
            == Some("crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs")
            && row["status"].as_str() == Some("live_home"))));
    assert!(scope_flow["wallet_promotion_rows"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .all(|row| row["proof_validated"].as_bool() == Some(true)
                && row["pending_lifecycle_status"].as_str() == Some("pending_receive")
                && row["confirmed_lifecycle_status"].as_str() == Some("confirmed_receive"))));

    assert_eq!(wallet_scan["status"].as_str(), Some("ok"));
    assert!(hist_flow["source_artifacts"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row.as_str() == Some("wallet_scan.json"))));
    assert_eq!(
        hist_flow["wallet_lifecycle_rows"]
            .as_array()
            .map(|rows| rows.len()),
        Some(REQUIRED_WALLET_LIFECYCLE_CASES.len())
    );
    for case_id in REQUIRED_WALLET_LIFECYCLE_CASES {
        let row = wallet_lifecycle_row(&hist_flow, case_id);
        assert_eq!(row["restart_verification_passed"].as_bool(), Some(true));
        assert_eq!(
            row["wallet_scan_digest_hex"].as_str(),
            hist_flow["wallet_scan_digest_hex"].as_str()
        );
        assert!(!row["tx_id"].as_str().unwrap_or_default().is_empty());
        assert!(!row["tx_history_digest_hex"]
            .as_str()
            .unwrap_or_default()
            .is_empty());
        assert!(!row["publication_digest_hex"]
            .as_str()
            .unwrap_or_default()
            .is_empty());
    }
    assert!(hist_flow["imported_artifact_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["verdict_id"].as_str()
            == Some("wallet_scan_digest_binding")
            && row["status"].as_str() == Some("verified"))));
    assert!(hist_flow["imported_artifact_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["verdict_id"].as_str()
            == Some("wallet_restart_projection")
            && row["status"].as_str() == Some("verified"))));
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "submitted")["lifecycle"].as_str(),
        Some("submitted")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "admitted")["lifecycle"].as_str(),
        Some("admitted")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "imported")["lifecycle"].as_str(),
        Some("imported")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "confirmed")["coarse_status"].as_str(),
        Some("confirmed")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "duplicate_import")["wallet_asset_rows_changed"].as_bool(),
        Some(false)
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "duplicate_import")["tx_history_row_count_changed"]
            .as_bool(),
        Some(false)
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "conflicted")["error_code"].as_str(),
        Some("duplicate_conflict")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "already_spent")["error_code"].as_str(),
        Some("already_spent")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "no_owned_output")["error_code"].as_str(),
        Some("no_owned_outputs")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "wrong_chain")["error_code"].as_str(),
        Some("wrong_chain")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "invalid_digest")["error_code"].as_str(),
        Some("invalid_digest")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "unsupported_package_version")["error_code"].as_str(),
        Some("unsupported_package_version")
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "wrong_chain")["tx_history_row_count_changed"].as_bool(),
        Some(false)
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "invalid_digest")["tx_history_row_count_changed"]
            .as_bool(),
        Some(false)
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "unsupported_package_version")
            ["tx_history_row_count_changed"]
            .as_bool(),
        Some(false)
    );
    assert!(hist_flow["historical_proof_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["example_id"].as_str()
            == Some("E2_right_inclusion")
            && row["verifier_status"].as_str() == Some("verified"))));
    assert!(occ_flow["occupancy_disclosure_verdicts"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["example_id"].as_str()
            == Some("E6_adaptive_split")
            && row["disclosure_guard"].as_str() == Some("coarse_only"))));
    assert!(sim_summary.contains("- emitted: wallet_scan.json"));
}

#[test]
fn test_e2e_digest_story_single() {
    let _guard = hjmt_e2e_lock();
    let out_dir = live_out("test_hjmt_e2e_digest_story_single");
    let leaf_flow = read_json(&out_dir, "leaf_flow.json");
    let proof_flow = read_json(&out_dir, "proof_flow.json");
    let pub_flow = read_json(&out_dir, "pub_flow.json");
    let val_flow = read_json(&out_dir, "val_flow.json");
    let watch_flow = read_json(&out_dir, "watch_flow.json");
    let hist_flow = read_json(&out_dir, "hist_flow.json");
    let occ_flow = read_json(&out_dir, "occ_flow.json");

    assert_eq!(
        leaf_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        proof_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        val_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        watch_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["old_public_root_hex"].as_str(),
        pub_flow["prior_public_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["new_public_root_hex"].as_str(),
        pub_flow["public_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["old_settlement_root_hex"].as_str(),
        val_flow["prev_settlement_root_hex"].as_str()
    );
    assert_eq!(
        hist_flow["route_migration_rows"][0]["new_settlement_root_hex"].as_str(),
        val_flow["new_settlement_root_hex"].as_str()
    );
    assert!(hist_flow["live_reject_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["case_id"].as_str()
            == Some("wrong_root_generation")
            && row["typed_error_class"].as_str() == Some("RootGenerationMix"))));
    assert!(occ_flow["live_reject_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["case_id"].as_str()
            == Some("stale_policy_transition_id")
            && row["typed_error_class"].as_str() == Some("StalePolicyId"))));
    assert_eq!(
        hist_flow["wallet_scan_digest_hex"].as_str(),
        wallet_lifecycle_row(&hist_flow, "imported")["wallet_scan_digest_hex"].as_str()
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "imported")["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        wallet_lifecycle_row(&hist_flow, "duplicate_import")["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
}

#[test]
fn test_rights_business_entitlement_lifecycle_local() {
    let _guard = hjmt_e2e_lock();
    let out_dir = live_out("test_rights_business_entitlement_lifecycle_local");
    let matrix = object_flow_matrix(&out_dir);

    let grant = matrix_row(&matrix.positive, "right_grant");
    let gated = matrix_row(&matrix.positive, "right_gated_voucher_action");
    let fee = matrix_row(&matrix.positive, "fee_supported_transition");
    let missing = matrix_row(&matrix.negative, "right_missing_for_voucher_action");
    let expired = matrix_row(&matrix.negative, "right_expired_for_voucher_action");

    assert_eq!(grant.policy_label, "right_delegate_policy_v1");
    assert_eq!(
        gated.required_rights,
        vec!["service_entitlement", "validator_mandate"]
    );
    assert_eq!(missing.required_rights, vec!["service_entitlement"]);
    assert_eq!(expired.required_rights, vec!["service_entitlement"]);
    assert!(fee
        .evidence_files
        .iter()
        .any(|file| file == "hjmt/hjmt_settlement_examples.json"));
    assert!([grant, gated, missing, expired].into_iter().all(|row| row
        .evidence_files
        .iter()
        .any(|file| file == "right_flow.json")));
}

#[test]
fn test_agentic_right_lifecycle_local() {
    let _guard = hjmt_e2e_lock();
    let out_dir = live_out("test_agentic_right_lifecycle_local");
    let matrix = object_flow_matrix(&out_dir);
    for id in [
        "right_grant",
        "right_delegate",
        "right_consume",
        "right_expiry",
        "right_replay_reject",
    ] {
        let row = matrix_row(
            if id == "right_replay_reject" {
                &matrix.negative
            } else {
                &matrix.positive
            },
            id,
        );
        assert!(row
            .evidence_files
            .iter()
            .any(|file| file == "right_flow.json"));
    }

    let original = right_fixture(RightClass::ServiceEntitlement);
    let mut delegated = original;
    delegated.holder_commitment = [0x54; 32];
    delegated.control_commitment = [0x55; 32];
    delegated
        .validate_action(
            RightAction::Transfer,
            RightActionCtx {
                now: 20,
                ..RightActionCtx::default()
            },
            Some(&original),
        )
        .expect("delegation must stay valid locally");
    delegated
        .validate_action(
            RightAction::Consume,
            RightActionCtx {
                now: 21,
                ..RightActionCtx::default()
            },
            Some(&delegated),
        )
        .expect("consumption must stay valid locally");
    assert_eq!(
        delegated
            .validate_action(
                RightAction::Consume,
                RightActionCtx {
                    now: 21,
                    seen_use_nonce: Some(delegated.use_nonce),
                    ..RightActionCtx::default()
                },
                Some(&delegated),
            )
            .expect_err("replay must reject"),
        RightErr::OneTimeReplay
    );

    let mut wrong_action = delegated;
    wrong_action.challenge_policy_id = [0u8; 32];
    assert_eq!(
        wrong_action
            .validate_action(
                RightAction::Challenge,
                RightActionCtx {
                    now: 30,
                    ..RightActionCtx::default()
                },
                Some(&delegated),
            )
            .expect_err("wrong action must reject"),
        RightErr::ChallengePolicyMix
    );
}

#[test]
fn test_machine_capability_lifecycle_local() {
    let _guard = hjmt_e2e_lock();
    let out_dir = live_out("test_machine_capability_lifecycle_local");
    let cfg = ScenarioCfg::from_file(live_cfg(&out_dir)).expect("load localized scenario cfg");
    assert!(cfg
        .stage13_hjmt_settlement_examples
        .as_ref()
        .is_some_and(|cfg| cfg
            .expected_right_classes
            .iter()
            .any(|class| class == "machine_capability")));

    let issued = right_fixture(RightClass::MachineCapability);
    issued
        .validate_action(RightAction::Create, RightActionCtx::default(), None)
        .expect("issuance must stay valid locally");
    issued
        .validate_action(
            RightAction::Consume,
            RightActionCtx {
                now: 20,
                ..RightActionCtx::default()
            },
            Some(&issued),
        )
        .expect("one-time use must stay valid locally");
    issued
        .validate_action(
            RightAction::Expire,
            RightActionCtx {
                now: 101,
                ..RightActionCtx::default()
            },
            None,
        )
        .expect("expiry must stay valid locally");
    assert_eq!(
        issued
            .check_path(wrong_path())
            .expect_err("wrong object binding must reject"),
        RightErr::PathTerminalMix
    );
    assert_eq!(
        issued
            .validate_action(
                RightAction::Challenge,
                RightActionCtx {
                    now: 20,
                    ..RightActionCtx::default()
                },
                None,
            )
            .expect_err("wrong action must reject"),
        RightErr::TransitionMix
    );
    assert_eq!(
        issued
            .validate_action(
                RightAction::Consume,
                RightActionCtx {
                    now: 20,
                    seen_use_nonce: Some(issued.use_nonce),
                    ..RightActionCtx::default()
                },
                Some(&issued),
            )
            .expect_err("reuse must reject"),
        RightErr::OneTimeReplay
    );
}
