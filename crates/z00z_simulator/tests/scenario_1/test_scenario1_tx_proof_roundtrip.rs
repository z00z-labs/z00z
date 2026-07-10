use std::path::{Path, PathBuf};

use z00z_simulator::{
    config::{ScenarioCfg, Stage6ProofMode},
    scenario_1::{stage_10, stage_7, stage_8, stage_9},
    StageResult,
};
use z00z_utils::io::save_json;
use z00z_wallets::tx::{build_tx_package_digest, verify_full_tx_package, TxPackage};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::stage_runner_support;

use stage_runner_support::{make_cfg_in, run_stage_plan_subset, stage_by_id};

fn opaque_cfg(cfg: &mut ScenarioCfg) {
    cfg.stage6_bundle
        .get_or_insert_with(Default::default)
        .proof_mode = Stage6ProofMode::OpaqueTest;
}

fn tx_pkg_path(out: &Path) -> PathBuf {
    out.join("transactions").join("tx_alice_to_bob_pkg.json")
}

fn stage4_pkg_path(out: &Path) -> PathBuf {
    out.join("transactions")
        .join("tx_alice_to_bob_pkg_stage4.json")
}

fn post_tx_store_root(out: &Path) -> PathBuf {
    out.join("storage").join("post_tx")
}

fn run_stage7_to_stage10(ctx: &mut z00z_simulator::SimContext, design_path: &Path) {
    for stage_id in [7_u32, 8, 9, 10] {
        let stage = stage_by_id(design_path, stage_id);
        let result = match stage_id {
            7 => stage_7::run_transfer_receive(ctx, &stage),
            8 => stage_8::run_transfer_claim(ctx, &stage),
            9 => stage_9::run_bundle_build(ctx, &stage),
            10 => stage_10::run_bundle_publish(ctx, &stage),
            _ => unreachable!(),
        };
        assert!(
            matches!(result, StageResult::Ok),
            "stage {stage_id} failed: {result:?}"
        );
    }
}

fn baseline_out() -> &'static PathBuf {
    static OUT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    OUT.get_or_init(|| {
        let root = fixture_cache::ensure_case("scenario1_tx_proof_roundtrip_v1", |base| {
            let (cfg_path, design_path, out) = make_cfg_in(base, opaque_cfg);
            let mut ctx = stage_runner_support::run_stage_setup_session(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4, 5, 6],
            );
            let pkg_path = tx_pkg_path(&out);

            let stage4_pkg: TxPackage =
                z00z_utils::io::load_json(&pkg_path).expect("stage4 tx package");
            save_json(stage4_pkg_path(&out), &stage4_pkg).expect("persist stage4 tx package");
            run_stage7_to_stage10(&mut ctx, &design_path);
            assert!(pkg_path.exists());
        });
        root.join("outputs/scenario_1")
    })
}

#[test]
fn test_scenario1_roundtrip_through_stage10() {
    let out = baseline_out();
    let stage4_pkg: TxPackage =
        z00z_utils::io::load_json(stage4_pkg_path(out)).expect("stage4 tx package");
    let stage4_bytes = std::fs::read(stage4_pkg_path(out)).expect("stage4 tx package bytes");
    let stage4_verify = verify_full_tx_package(&stage4_bytes).expect("full verifier result");
    assert!(
        stage4_verify.valid,
        "stage4 package must pass the full verifier: {:?}",
        stage4_verify.errors
    );

    let pkg_path = tx_pkg_path(out);
    let stage10_pkg: TxPackage = z00z_utils::io::load_json(&pkg_path).expect("stage10 tx package");
    let stage10_bytes = std::fs::read(&pkg_path).expect("stage10 tx package bytes");
    let stage10_verify = verify_full_tx_package(&stage10_bytes).expect("full verifier result");

    assert!(
        stage10_verify.valid,
        "stage10 package must still pass the full verifier: {:?}",
        stage10_verify.errors
    );
    assert_eq!(stage10_pkg.tx_digest_hex, stage4_pkg.tx_digest_hex);
    assert_eq!(stage10_pkg.tx.proof, stage4_pkg.tx.proof);
    assert_eq!(stage10_pkg.tx.auth, stage4_pkg.tx.auth);
}

#[test]
fn test_scenario1_canonical_chain_tamper() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.keep();
    let (cfg_path, design_path, out) = make_cfg_in(&base, opaque_cfg);
    let pkg_path = tx_pkg_path(&out);
    fixture_cache::copy_tree(baseline_out(), &out);

    let mut pkg: TxPackage = z00z_utils::io::load_json(&pkg_path).expect("tx package");
    pkg.chain_id += 1;
    pkg.tx_digest_hex = build_tx_package_digest(
        &pkg.kind,
        &pkg.package_type,
        pkg.version,
        pkg.chain_id,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .expect("recompute digest after chain-scope tamper");
    save_json(&pkg_path, &pkg).expect("rewrite tx package");

    let result = run_stage_plan_subset(&cfg_path, &design_path, &[11_u32]);
    let stage11 = stage_runner_support::stage_res(&result, 11);
    let msg = match stage11 {
        StageResult::Fail(msg) => msg.clone(),
        other => {
            panic!("stage 11 must fail after canonical chain-scope tamper, got {other:?}")
        }
    };

    assert!(
        msg.contains("public spend contract failed")
            || msg.contains("authorization verification failed"),
        "unexpected stage 11 failure: {msg}"
    );
    assert!(
        !out.join("transactions/checkpoint_s7.json").exists(),
        "canonical chain-scope tamper must block authoritative checkpoint summary emission"
    );
    assert!(
        !post_tx_store_root(&out)
            .join("artifacts/checkpoints/draft")
            .exists(),
        "canonical chain-scope tamper must block post-tx checkpoint draft persistence"
    );
}
