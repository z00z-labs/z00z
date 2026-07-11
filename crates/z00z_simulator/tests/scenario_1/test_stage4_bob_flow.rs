use crate::output_roots;
use tempfile::tempdir;
use z00z_simulator::scenario_1::stage_6::sim_pkg_support;
use z00z_simulator::scenario_1::support::scenario_support;

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use z00z_networks_rpc::{RpcError, RpcTransport};
use z00z_simulator::StageResult;
use z00z_utils::{
    codec::json,
    io::{create_dir_all, load_json, write_file},
};
use z00z_wallets::{
    receiver::{ScanResult, StealthOutputScanner},
    rpc::types::common::PersistWalletId,
    tx::TxPackage,
};

use crate::stage4_bob;
use crate::stage4_bob::{import_wire, setup_env, unlock};
use scenario_support::read_rpc_req_rows;
use sim_pkg_support::load_pkg_bundle;
use z00z_simulator::scenario_1::stage_6::shared_cases;
use z00z_simulator::scenario_1::support::fixture_cache;

struct RunCase {
    out: PathBuf,
    stage4: StageResult,
}

static E2E18_RUN: OnceLock<RunCase> = OnceLock::new();

fn run_case() -> &'static RunCase {
    E2E18_RUN.get_or_init(|| RunCase {
        out: shared_cases::e2e18_stage6_out(),
        stage4: StageResult::Ok,
    })
}

fn load_json_value(path: &Path) -> serde_json::Value {
    load_json(path).unwrap_or_else(|_| panic!("load json value: {}", path.display()))
}

fn find_wallet<'a>(dump: &'a serde_json::Value, actor: &str) -> &'a serde_json::Value {
    dump["wallets"]
        .as_array()
        .expect("wallets array")
        .iter()
        .find(|row| row["actor"].as_str() == Some(actor))
        .unwrap_or_else(|| panic!("wallet row missing for actor={actor}"))
}

fn bob_id(run: &RunCase) -> PersistWalletId {
    let after = load_json_value(
        &run.out
            .join("transactions")
            .join("wallets_state_after.json"),
    );
    PersistWalletId(
        find_wallet(&after, "bob")["wallet_id"]
            .as_str()
            .expect("bob wallet_id")
            .to_string(),
    )
}

fn pick_wires(
    pkg: &TxPackage,
    bob_keys: &z00z_wallets::key::ReceiverKeys,
) -> (Vec<z00z_core::AssetWire>, z00z_core::AssetWire) {
    let scan = StealthOutputScanner::from_keys(bob_keys);
    let mut bob_wires = Vec::new();
    let mut alice_wire = None;

    for row in &pkg.tx.outputs {
        let asset = row.asset_wire.clone().to_asset().expect("tx output asset");
        let wire = row.asset_wire.clone().to_wire().expect("tx output wire");
        match scan.scan_leaf(&asset) {
            ScanResult::Mine { .. } => bob_wires.push(wire),
            ScanResult::NotMine | ScanResult::MaybeMine { .. } => alice_wire = Some(wire),
        }
    }

    (bob_wires, alice_wire.expect("alice change output"))
}

fn is_bad_owner(err: &RpcError) -> bool {
    matches!(err, RpcError::InvalidParams(msg) if msg == "IMPORT_STEALTH_INCONSISTENT")
}

fn is_bad_session(err: &RpcError) -> bool {
    matches!(err, RpcError::InvalidParams(msg) if msg == "IMPORT_SESSION_INVALID")
}

async fn has_bob_asset(
    env: &stage4_bob::TestEnv,
    bob_id: &PersistWalletId,
    asset_id: [u8; 32],
) -> bool {
    env.wallet_service
        .list_claimed_assets(bob_id)
        .await
        .is_ok_and(|items| items.into_iter().any(|item| item.asset_id() == asset_id))
}

async fn import_bob(
    env: &stage4_bob::TestEnv,
    bob_id: &PersistWalletId,
    bob: &z00z_wallets::rpc::types::wallet::SessionToken,
    bob_wires: &[z00z_core::AssetWire],
) -> Vec<z00z_core::Asset> {
    let mut assets = Vec::new();
    for wire in bob_wires {
        let asset = wire.clone().to_asset().expect("bob asset");
        let import = import_wire(env, bob, wire)
            .await
            .expect("bob import must succeed");
        assert_eq!(import["success"].as_bool(), Some(true));
        assert!(
            import["is_inserted"].as_bool() == Some(true)
                || import["asset_already_exists"].as_bool() == Some(true),
            "bob import must either insert or report the existing stage-4 asset"
        );
        assert!(
            has_bob_asset(env, bob_id, asset.asset_id()).await,
            "bob-owned stage-4 output must be stored"
        );
        assets.push(asset);
    }
    assets
}

fn req_rows(out: &Path) -> Vec<serde_json::Value> {
    read_rpc_req_rows(out)
}

fn has_wallet(log_id: &str, wallet_id: &str) -> bool {
    let prefix = &wallet_id[..10.min(wallet_id.len())];
    let suffix_start = wallet_id.len().saturating_sub(4);
    let suffix = &wallet_id[suffix_start..];
    log_id.contains(prefix) && log_id.contains(suffix)
}

fn is_bob_req(row: &serde_json::Value, bob_id: &PersistWalletId) -> bool {
    row["wallet_id"]
        .as_str()
        .map(|log_id| has_wallet(log_id, &bob_id.0))
        .unwrap_or(false)
}

fn has_bob_rpc_flow(
    rows: &[serde_json::Value],
    bob_id: &PersistWalletId,
    bob_count: usize,
) -> bool {
    let mut import_count = 0usize;
    let mut in_bob = false;

    for row in rows {
        let method = row["method"].as_str().unwrap_or("");
        match (in_bob, method, is_bob_req(row, bob_id)) {
            (_, "wallet.session.unlock_wallet", true) => {
                in_bob = true;
                import_count = 0;
            }
            (true, "wallet.asset.import_asset", _) => import_count += 1,
            (true, "wallet.session.lock_wallet", true) if import_count >= bob_count => {
                return true;
            }
            (true, "wallet.session.lock_wallet", true) => in_bob = false,
            _ => {}
        }
    }

    false
}

fn check_bob_rpc(out: &Path, bob_id: &PersistWalletId, bob_count: usize) {
    let rows = req_rows(out);
    assert!(
        has_bob_rpc_flow(&rows, bob_id, bob_count),
        "stage-4 rpc log must show bob unlock/import/lock flow with enough imports"
    );
    assert!(
        req_rows(out)
            .iter()
            .filter(|row| row["method"].as_str() == Some("wallet.asset.import_asset"))
            .count()
            >= bob_count,
        "stage-4 rpc log must contain at least one Bob import per Bob-owned output"
    );
}

async fn lock_bob(env: &stage4_bob::TestEnv, bob_id: &PersistWalletId) {
    let session = unlock(env, bob_id, "Bob_Pass_Z00Z_43!").await;
    env.transport
        .call("wallet.session.lock_wallet", json!({"session": session}))
        .await
        .expect("bob lock must succeed");
}

fn write_log(
    bob_id: &PersistWalletId,
    bob_asset: &z00z_core::Asset,
    owner_bad: bool,
    session_bad: bool,
) {
    let out_dir = output_roots::stage4_output_root();
    create_dir_all(&out_dir).expect("mkdir outputs/e2e18");
    let bob_log = format!(
        "bob_wallet_id={}\nimport_ok_asset={}\nchange_reject={}\nlock_reject={}\n",
        bob_id.0,
        hex::encode(bob_asset.asset_id()),
        owner_bad,
        session_bad,
    );
    write_file(out_dir.join("bob_flow_log.txt"), bob_log.as_bytes()).expect("write bob log");
}

#[test]
fn test_stage4_bob_flow() {
    if cfg!(debug_assertions) {
        return;
    }

    let run = run_case();
    assert!(
        matches!(run.stage4, StageResult::Ok),
        "stage 4 must succeed"
    );

    let (_, _, pkg): (PathBuf, Vec<u8>, TxPackage) = load_pkg_bundle(&run.out);
    let bob_id = bob_id(run);
    let wallet_temp = tempdir().expect("wallet temp dir");
    let wallet_root = wallet_temp.path().join("wallets");
    fixture_cache::copy_tree(&run.out.join("wallets"), &wallet_root);
    let env = setup_env(wallet_root);
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let bob = unlock(&env, &bob_id, "Bob_Pass_Z00Z_43!").await;
        let bob_keys = env
            .wallet_service
            .receiver_keys(&bob_id)
            .await
            .expect("bob receiver keys");
        let (bob_wires, alice_wire) = pick_wires(&pkg, &bob_keys);
        assert_eq!(
            bob_wires.len(),
            4,
            "all configured bob outputs must stay Bob-owned"
        );
        let bob_assets = import_bob(&env, &bob_id, &bob, &bob_wires).await;

        let alice_asset = alice_wire.clone().to_asset().expect("alice asset");
        let alice_err = import_wire(&env, &bob, &alice_wire)
            .await
            .expect_err("alice change must not import into bob wallet");
        assert!(is_bad_owner(&alice_err));
        assert!(
            !has_bob_asset(&env, &bob_id, alice_asset.asset_id()).await,
            "alice change must not be stored under bob"
        );

        check_bob_rpc(&run.out, &bob_id, bob_wires.len());

        lock_bob(&env, &bob_id).await;
        let lock_err = import_wire(&env, &bob, &bob_wires[0])
            .await
            .expect_err("locked bob session must be rejected");
        assert!(is_bad_session(&lock_err));
        write_log(
            &bob_id,
            &bob_assets[0],
            is_bad_owner(&alice_err),
            is_bad_session(&lock_err),
        );
    });
}
