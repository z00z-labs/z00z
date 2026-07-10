use std::path::Path;

use serde_json::Value;
use z00z_simulator::config::Stage6ProofMode;
use z00z_storage::checkpoint::{audit::decode_audit_bin, decode_link_bin};
use z00z_utils::io::read_file;

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use scenario_support::make_cfg_in;

fn good_cfg(cfg: &mut z00z_simulator::config::ScenarioCfg) {
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 10;
    stage4.transaction.outputs.bob_outputs_count = 4;
    stage4.transaction.class = "Coin".to_string();
    stage4.transaction.symbol = "Z00Z".to_string();
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.1);
    stage4.transaction.amount = None;
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::OpaqueTest;
}

fn load_json(path: &Path) -> Value {
    serde_json::from_slice(&read_file(path).expect("read json")).expect("decode json")
}

fn only_bin(dir: &Path) -> std::path::PathBuf {
    let items = std::fs::read_dir(dir)
        .expect("read dir")
        .map(|row| row.expect("dir entry").path())
        .filter(|path| path.extension().and_then(|item| item.to_str()) == Some("bin"))
        .collect::<Vec<_>>();
    assert_eq!(
        items.len(),
        1,
        "expected one .bin file in {}",
        dir.display()
    );
    items.into_iter().next().expect("bin path")
}

struct OutCase {
    out: std::path::PathBuf,
}

fn ok_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root =
            fixture_cache::ensure_shared_case("scenario1_opaque_stage12_shared_v1", |base| {
                let (cfg_path, design_path, out) = make_cfg_in(base, good_cfg);
                let _ctx = stage_runner_support::run_stage_setup(
                    &cfg_path,
                    &design_path,
                    &[1_u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
                );
                assert!(out.join("transactions/checkpoint_s8.json").exists());
            });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

#[test]
fn test_unified_gate_rs() {
    if cfg!(debug_assertions) {
        return;
    }

    let out = &ok_case().out;

    let s7 = load_json(&out.join("transactions/checkpoint_s7.json"));
    let s8 = load_json(&out.join("transactions/checkpoint_s8.json"));
    let scan = load_json(&out.join("wallet_scan.json"));
    let stage1 = load_json(&out.join("stage_1_snapshot.json"));
    let claim_publish_audit = load_json(&out.join("claim_publish/audit_log.json"));
    let manifest = load_json(&out.join("genesis/genesis_settlement_manifest.json"));
    let claim_post = load_json(&out.join("storage/claim_post/summary.json"));
    let diff = load_json(&out.join("transactions/wallets_state_diff.json"));
    let ledger = load_json(&out.join("storage/ledger_path.json"));
    let link = decode_link_bin(
        &read_file(only_bin(
            &out.join("transactions/artifacts/checkpoints/links"),
        ))
        .expect("read link"),
    )
    .expect("decode link");
    let audit = decode_audit_bin(
        &read_file(only_bin(
            &out.join("transactions/artifacts/checkpoints/audit"),
        ))
        .expect("read audit"),
    )
    .expect("decode audit");

    assert_eq!(s7["stage"].as_u64(), Some(11));
    assert_eq!(s7["status"].as_str(), Some("ok"));
    assert_eq!(s7["wallet_invariant_ok"].as_bool(), Some(true));
    assert_eq!(s7["wallet_scan_file"].as_str(), Some("wallet_scan.json"));
    assert!(s7["charlie_detected_count"].as_u64().unwrap_or(0) > 0);
    assert!(s7["charlie_detected_amount"].as_u64().unwrap_or(0) > 0);
    assert_eq!(s8["stage"].as_u64(), Some(12));
    assert_eq!(s8["status"].as_str(), Some("ok"));
    assert_eq!(s8["checkpoint_id_hex"].as_str().map(str::len), Some(64));
    assert_eq!(
        s8["artifact_path"].as_str(),
        Some("transactions/artifacts/checkpoints/final")
    );
    assert_eq!(
        s8["link_path"].as_str(),
        Some("transactions/artifacts/checkpoints/links")
    );
    assert_eq!(
        s8["audit_path"].as_str(),
        Some("transactions/artifacts/checkpoints/audit")
    );
    assert_eq!(scan["actor"].as_str(), Some("charlie"));
    assert_eq!(scan["scan_path"].as_str(), Some("jmt_scan"));
    assert_eq!(scan["status"].as_str(), Some("ok"));
    assert_eq!(
        scan["proof_step"].as_str(),
        Some("proof_blob+chk_blob_settlement before runtime ownership detection")
    );
    assert!(scan["skipped_non_asset_count"].as_u64().unwrap_or(0) > 0);
    assert!(scan["distinction"]
        .as_str()
        .is_some_and(|text| text.contains("not equivalent to detached Stage 5 leaf scan")));
    assert!(scan["detected_count"].as_u64().unwrap_or(0) > 0);
    assert!(scan["total_detected_amount"].as_u64().unwrap_or(0) > 0);
    assert!(stage1["rights_count"].as_u64().unwrap_or(0) > 0);
    assert_eq!(
        stage1["rights_artifact_file"].as_str(),
        Some("genesis_rights.json")
    );
    assert_eq!(
        stage1["settlement_manifest_file"].as_str(),
        Some("genesis_settlement_manifest.json")
    );
    assert_eq!(
        claim_publish_audit["genesis_rights_included"].as_bool(),
        Some(true)
    );
    assert_eq!(
        claim_publish_audit["genesis_rights_count"].as_u64(),
        stage1["rights_count"].as_u64()
    );
    assert_eq!(
        manifest["rights_artifact"].as_str(),
        Some("genesis_rights.json")
    );
    assert_eq!(
        manifest["right_count"].as_u64(),
        stage1["rights_count"].as_u64()
    );
    assert!(manifest["deterministic_replay_digest"]
        .as_str()
        .is_some_and(|value| value.len() == 64));
    assert!(manifest["rights_output_roundtrip_digest"]
        .as_str()
        .is_some_and(|value| value.len() == 64));
    assert!(manifest["manifest_hash"]
        .as_str()
        .is_some_and(|value| value.len() == 64));
    assert_eq!(claim_post["root_match"].as_bool(), Some(true));
    assert_eq!(
        claim_post["source_check_root_hex"].as_str(),
        claim_post["view_check_root_hex"].as_str()
    );
    assert_eq!(
        ledger["claim_root_hex"].as_str(),
        claim_post["source_check_root_hex"].as_str()
    );
    assert!(diff["rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| {
            row["actor"].as_str() == Some("charlie")
                && row["status"].as_str() == Some("new")
                && row["lifecycle_status"].as_str() == Some("confirmed_receive")
        })));
    assert_eq!(ledger["checkpoint_id_hex"], s8["checkpoint_id_hex"]);
    assert_eq!(
        s8["checkpoint_id_hex"].as_str(),
        Some(hex::encode(link.checkpoint_id().as_bytes()).as_str())
    );
    assert_eq!(audit.checkpoint_id(), link.checkpoint_id());
    assert!(Path::new(&out.join("transactions/checkpoint_bridge_s6.json")).exists());
    assert!(Path::new(&out.join("transactions/checkpoint_s7.json")).exists());
    assert!(Path::new(&out.join("transactions/checkpoint_s8.json")).exists());
    assert!(Path::new(&out.join("wallet_scan.json")).exists());
    assert!(Path::new(&out.join("transactions/wallets_state_diff.json")).exists());
    assert!(Path::new(&out.join("transactions/artifacts/checkpoints/final")).exists());
    assert!(Path::new(&out.join("transactions/artifacts/checkpoints/links")).exists());
    assert!(Path::new(&out.join("transactions/artifacts/checkpoints/audit")).exists());
    assert!(Path::new(&out.join("genesis/genesis_rights.json")).exists());
    assert!(Path::new(&out.join("genesis/genesis_settlement_manifest.json")).exists());
    assert!(Path::new(&out.join("storage/claim_post/settlement_state.redb")).exists());
    assert!(Path::new(&out.join("storage/pre_tx/settlement_state.redb")).exists());
    assert!(Path::new(&out.join("storage/pre_tx/artifacts/checkpoints/prep_snapshot")).exists());
    assert!(Path::new(&out.join("storage/post_tx/settlement_state.redb")).exists());
    assert!(Path::new(&out.join("storage/post_tx/artifacts/checkpoints/prep_snapshot")).exists());
    assert!(Path::new(&out.join("storage/post_tx/artifacts/checkpoints/draft")).exists());
    assert!(Path::new(&out.join("storage/post_tx/artifacts/checkpoints/exec_input")).exists());
    assert!(Path::new(&out.join("storage/post_tx/artifacts/checkpoints/final")).exists());
    assert!(Path::new(&out.join("storage/post_tx/artifacts/checkpoints/links")).exists());
    assert!(Path::new(&out.join("storage/post_tx/artifacts/checkpoints/audit")).exists());
}
