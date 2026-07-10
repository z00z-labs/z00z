use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use z00z_simulator::config::Stage6ProofMode;
use z00z_storage::checkpoint::{
    audit::decode_audit_bin, decode_art_bin, decode_draft_bin, decode_exec_bin, decode_link_bin,
    derive_draft_id, derive_exec_id, CheckpointTransitionStatementV1,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::read_file,
};
use z00z_wallets::tx::TxPackage;

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

fn only_bin(dir: &Path) -> PathBuf {
    let items = fs::read_dir(dir)
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

fn load_tx(path: &Path) -> TxPackage {
    JsonCodec
        .deserialize(&read_file(path).expect("read tx json"))
        .expect("decode tx package")
}

struct OutCase {
    out: PathBuf,
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
                assert!(out.join("transactions/checkpoint_s7.json").exists());
                assert!(out.join("transactions/checkpoint_s8.json").exists());
            });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

#[test]
fn test_proof_path_draft_publication() {
    if cfg!(debug_assertions) {
        return;
    }

    let out = &ok_case().out;

    let tx = load_tx(&out.join("transactions/tx_alice_to_bob_pkg.json"));
    let want_tx_proof = JsonCodec.serialize(&tx.tx.proof).expect("encode tx proof");
    let exec_bytes = read_file(only_bin(
        &out.join("transactions/artifacts/checkpoints/exec_input"),
    ))
    .expect("read exec bytes");
    let draft_bytes = read_file(only_bin(
        &out.join("transactions/artifacts/checkpoints/draft"),
    ))
    .expect("read draft bytes");
    let art = decode_art_bin(
        &read_file(only_bin(
            &out.join("transactions/artifacts/checkpoints/final"),
        ))
        .expect("read artifact"),
    )
    .expect("decode artifact");
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
    let exec = decode_exec_bin(&exec_bytes).expect("decode exec");
    let draft = decode_draft_bin(&draft_bytes).expect("decode draft");
    let s7 = load_json(&out.join("transactions/checkpoint_s7.json"));
    let s8 = load_json(&out.join("transactions/checkpoint_s8.json"));

    let exec_id_hex = hex::encode(derive_exec_id(&exec_bytes).as_bytes());
    let draft_id_hex = hex::encode(derive_draft_id(&draft).expect("derive draft id").as_bytes());
    let checkpoint_id_hex = hex::encode(link.checkpoint_id().as_bytes());

    assert_eq!(s7["exec_input_id_hex"].as_str(), Some(exec_id_hex.as_str()));
    assert_eq!(s7["draft_id_hex"].as_str(), Some(draft_id_hex.as_str()));
    assert_eq!(s8["exec_input_id_hex"], s7["exec_input_id_hex"]);
    assert_eq!(s8["draft_id_hex"], s7["draft_id_hex"]);
    assert_eq!(
        s8["checkpoint_id_hex"].as_str(),
        Some(checkpoint_id_hex.as_str())
    );
    assert_eq!(
        exec.txs().len(),
        1,
        "scenario 1 uses one proof-bearing tx lane"
    );
    assert_eq!(exec.txs()[0].tx_proof(), want_tx_proof.as_slice());
    assert_eq!(
        art.cp_proof(),
        CheckpointTransitionStatementV1::new(
            art.version(),
            art.height(),
            art.pub_in(),
            link.prep_snapshot_id(),
            link.exec_input_id(),
        )
        .backend_payload()
        .as_slice()
    );
    assert_eq!(audit.checkpoint_id(), link.checkpoint_id());
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
}
