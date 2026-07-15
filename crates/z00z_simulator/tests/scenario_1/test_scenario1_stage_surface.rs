use std::{
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::tempdir;
use z00z_simulator::{
    scenario_1::runner, DesignDoc, DesignStage, ScenarioResult, StageResult, StageState,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_to_string, remove_file, write_file},
};

use z00z_simulator::scenario_1::stage_13::shared_cases;
use z00z_simulator::scenario_1::support::{scenario_support, stage_runner_support};

struct StageSurfaceGuard {
    _process_guard: stage_runner_support::ProcessLock,
    _thread_guard: MutexGuard<'static, ()>,
}

fn stage_surface_lock() -> StageSurfaceGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    StageSurfaceGuard {
        _process_guard: stage_runner_support::acquire_process_lock(),
        _thread_guard: LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()),
    }
}

fn live_stage_surface_out() -> std::path::PathBuf {
    shared_cases::full_stage13_out()
}

fn stage_surface_cfg_path(out_dir: &Path) -> PathBuf {
    out_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("stage surface base")
        .join("scenario_config.yaml")
}

fn live_stage_surface_cfg() -> PathBuf {
    stage_surface_cfg_path(&live_stage_surface_out())
}

fn stage_surface_mutation_paths() -> (PathBuf, PathBuf) {
    let out_dir = shared_cases::stage13_out("stage_surface_mutations");
    (stage_surface_cfg_path(&out_dir), out_dir)
}

fn design_doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml")
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

fn run_scenario_bin(cfg_path: &Path, design_path: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_scenario_1"))
        .arg("--config")
        .arg(cfg_path)
        .arg("--design")
        .arg(design_path)
        .output()
        .expect("launch scenario_1 binary");
    assert!(
        output.status.success(),
        "scenario_1 binary failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn synthetic_live_run() -> ScenarioResult {
    ScenarioResult {
        scenario_id: 1,
        is_aborted: false,
        stages: expected_stage_names()
            .into_iter()
            .map(|(stage, name)| StageState {
                stage,
                name,
                result: StageResult::Ok,
            })
            .collect(),
    }
}

fn step_ids(stage: &DesignStage) -> Vec<&str> {
    stage.steps.iter().map(|step| step.id.as_str()).collect()
}
fn step_actions(stage: &DesignStage) -> Vec<&str> {
    stage
        .steps
        .iter()
        .map(|step| step.action.as_str())
        .collect()
}
fn expected_stage_names() -> Vec<(u32, String)> {
    vec![
        (1, "genesis_init".to_string()),
        (2, "wallet_create".to_string()),
        (3, "claim_prepare".to_string()),
        (4, "claim_publish".to_string()),
        (5, "tx_plan".to_string()),
        (6, "tx_prepare".to_string()),
        (7, "transfer_receive".to_string()),
        (8, "transfer_claim".to_string()),
        (9, "bundle_build".to_string()),
        (10, "bundle_publish".to_string()),
        (11, "checkpoint_apply_storage".to_string()),
        (12, "checkpoint_finalize".to_string()),
        (13, "hjmt_settlement_examples".to_string()),
    ]
}

fn expected_stage_descs() -> Vec<&'static str> {
    vec![
        "Generate deterministic genesis settlement corpus through the z00z_core genesis pipeline,\npersist per-definition binary packs plus policy/right/voucher settlement artifacts,\nand snapshot the canonical stage-1 state.\n",
        "Create the three simulator wallets, derive public receiver material, prove\nbackup/restart/lifecycle behavior, and snapshot the actor runtime for the\nlater claim and transfer lanes.\n",
        "Load the stage-1 genesis packs, distribute claimable assets across the three\nactors, import them into wallet runtime state, and snapshot the prepared\nclaim lane without anchoring it as a checkpoint yet.\n",
        "Publish the prepared claim package into a claim-store surface, inject genesis rights into storage,\nmaterialize a dedicated claim-publish snapshot, and emit claim-publish audit artifacts.\n",
        "Validate the tx-lane planning surface without mutating wallet state or writing\ntransaction-history rows. The canonical transaction artifact and wallet history\nproducer is stage 6.\n",
        "Execute the same shared tx-lane core behind the explicit stage-6 prepare facade.\nThis remains the canonical handoff producer for the transfer and bundle lanes.\n",
        "Drive the report-only receive bridge for Bob's selected stage-4 output and\npersist the receive handoff needed by the explicit claim step.\n",
        "Consume the receive handoff, perform the explicit claim transition, and persist\nthe stage-5 receive/claim artifacts for the downstream bundle lane.\n",
        "Build the checkpoint bridge from the transfer artifacts, persist fragment files,\nand save the exec_input handoff needed by storage-backed apply.\n",
        "Reuse the built bridge and fragments, publish the report surface for the bundle lane,\nand keep the storage-apply handoff explicit.\n",
        "Load the published checkpoint bridge, apply the checkpoint draft through storage,\nrefresh Charlie's committed-state view while skipping right leaves before ownership detection,\nand persist the shared stage-7 checkpoint summary.\n",
        "Finalize checkpoint publication from the shared stage-7 checkpoint summary,\nseal the final checkpoint artifact, and export the final post-tx surfaces.\n",
        "Demonstrate live generalized settlement proofs and debug artifacts through\nproduction storage APIs, including generated asset/right examples, fee support,\ndeletion and absence proofs, adaptive policy evidence, cache metrics, and\nreload-debug verification against the persisted settlement root.\n",
    ]
}

fn step_post(stage: &DesignStage) -> Vec<Vec<&str>> {
    stage
        .steps
        .iter()
        .map(|step| {
            step.post_conditions
                .iter()
                .map(|text| text.as_str())
                .collect()
        })
        .collect()
}

fn assert_step_contracts(design: &DesignDoc) {
    assert!(design.stages.iter().all(|stage| stage
        .description
        .as_deref()
        .is_some_and(|text| !text.trim().is_empty())));
    assert!(design.stages.iter().all(|stage| !stage.steps.is_empty()));
    assert_eq!(
        step_ids(&design.stages[0]),
        vec!["S1-1", "S1-2", "S1-3", "S1-4", "S1-5", "S1-6", "S1-7", "S1-8"]
    );
    assert_eq!(
        step_ids(&design.stages[1]),
        vec![
            "S2-1", "S2-2", "S2-5", "S2-6", "S2-8", "S2-9", "S2-10", "S2-11", "S2-12", "S2-13",
            "S2-14", "S2-15", "S2-16", "S2-17", "S2-18", "S2-19"
        ]
    );
    assert_eq!(
        step_ids(&design.stages[2]),
        vec!["S3-1", "S3-2", "S3-3", "S3-4"]
    );
    assert_eq!(
        step_ids(&design.stages[3]),
        vec!["P4-1", "P4-2", "P4-3", "P4-4"]
    );
    assert_eq!(step_ids(&design.stages[4]), vec!["P5-1", "P5-2", "P5-3"]);
    assert_eq!(
        step_ids(&design.stages[5]),
        vec![
            "S4-1", "S4-2", "S4-12", "S4-3", "S4-4", "S4-5", "S4-9", "S4-10", "S4-6", "S4-7",
            "S4-8", "S4-C1", "S4-11", "S4-13"
        ]
    );
    assert_eq!(
        step_ids(&design.stages[6]),
        vec!["S5-1", "S5-2", "S5-3", "S5-4", "S5-5", "S5-6"]
    );
    assert_eq!(
        step_ids(&design.stages[7]),
        vec!["S5-1", "S5-2", "S5-3", "S5-4", "S5-6", "S5-7", "S5-8"]
    );
    assert_eq!(
        step_ids(&design.stages[8]),
        vec!["S6-1", "S6-2", "S6-3", "S6-4", "S6-5", "S6-6", "S6-7", "S6-8"]
    );
    assert_eq!(
        step_ids(&design.stages[9]),
        vec!["S6-1", "S6-2", "S6-3", "S6-4", "S6-5", "S6-6", "S6-7", "S6-8"]
    );
    assert_eq!(
        step_ids(&design.stages[10]),
        vec!["S7-1", "S7-2", "S7-3", "S7-4"]
    );
    assert_eq!(
        step_ids(&design.stages[11]),
        vec!["S8-1", "S8-2", "S8-3", "S8-4"]
    );
    assert_eq!(
        step_ids(&design.stages[12]),
        vec!["S13-1", "S13-2", "S13-3", "S13-4", "S13-5", "S13-6", "S13-7", "S13-8"]
    );
}
fn assert_stage_descs(design: &DesignDoc) {
    let actual: Vec<&str> = design
        .stages
        .iter()
        .map(|stage| stage.description.as_deref().expect("description"))
        .collect();
    assert_eq!(actual, expected_stage_descs());
}

fn assert_rust_entries(design: &DesignDoc) {
    assert_eq!(
        design.stages[2].rust_entry.as_deref(),
        Some("stage_3::run_claim_prepare(ctx, stage)")
    );
    assert_eq!(
        design.stages[3].rust_entry.as_deref(),
        Some("stage_4::run_claim_publish(ctx, stage)")
    );
    assert_eq!(
        design.stages[4].rust_entry.as_deref(),
        Some("stage_5::run_tx_plan(ctx, stage)")
    );
    assert_eq!(
        design.stages[5].rust_entry.as_deref(),
        Some("stage_6::run_tx_prepare(ctx, stage)")
    );
    assert_eq!(
        design.stages[6].rust_entry.as_deref(),
        Some("stage_7::run_transfer_receive(ctx, stage)")
    );
    assert_eq!(
        design.stages[7].rust_entry.as_deref(),
        Some("stage_8::run_transfer_claim(ctx, stage)")
    );
    assert_eq!(
        design.stages[8].rust_entry.as_deref(),
        Some("stage_9::run_bundle_build(ctx, stage)")
    );
    assert_eq!(
        design.stages[9].rust_entry.as_deref(),
        Some("stage_10::run_bundle_publish(ctx, stage)")
    );
    assert_eq!(
        design.stages[10].rust_entry.as_deref(),
        Some("stage_11::run_apply(ctx, stage)")
    );
    assert_eq!(
        design.stages[11].rust_entry.as_deref(),
        Some("stage_12::run_finalize(ctx, stage)")
    );
    assert_eq!(
        design.stages[12].rust_entry.as_deref(),
        Some("stage_13::run_hjmt_examples(ctx, stage)")
    );
}
fn assert_config_sources(design: &DesignDoc) {
    assert_eq!(
        design.stages[0].config_source.as_deref(),
        Some("scenario_config.yaml::stage1_paths + stage1_genesis_config")
    );
    assert_eq!(
        design.stages[1].config_source.as_deref(),
        Some("scenario_config.yaml::stage2_wallet_create")
    );
    assert_eq!(
        design.stages[2].config_source.as_deref(),
        Some("scenario_config.yaml::stage3_claim")
    );
    assert_eq!(
        design.stages[3].config_source.as_deref(),
        Some("scenario_config.yaml::stage4_claim_publish")
    );
    assert_eq!(
        design.stages[4].config_source.as_deref(),
        Some("scenario_config.yaml::stage4_tx_prepare")
    );
    assert_eq!(
        design.stages[5].config_source.as_deref(),
        Some("scenario_config.yaml::stage4_tx_prepare")
    );
    assert_eq!(
        design.stages[6].config_source.as_deref(),
        Some("scenario_config.yaml::stage5_transfer")
    );
    assert_eq!(
        design.stages[7].config_source.as_deref(),
        Some("scenario_config.yaml::stage5_transfer")
    );
    assert_eq!(
        design.stages[8].config_source.as_deref(),
        Some("scenario_config.yaml::stage6_bundle")
    );
    assert_eq!(
        design.stages[9].config_source.as_deref(),
        Some("scenario_config.yaml::stage6_bundle")
    );
    assert_eq!(
        design.stages[10].config_source.as_deref(),
        Some("scenario_config.yaml::stage7_paths")
    );
    assert_eq!(
        design.stages[11].config_source.as_deref(),
        Some("scenario_config.yaml::stage8_paths")
    );
    assert_eq!(
        design.stages[12].config_source.as_deref(),
        Some("scenario_config.yaml::stage13_hjmt_settlement_examples")
    );
}

fn assert_step_actions(design: &DesignDoc) {
    assert_eq!(
        step_actions(&design.stages[0]),
        vec![
            "Load the canonical genesis config and validate the derived seed",
            "Prepare the stage-1 output directories",
            "Create every AssetDefinition and register it in ctx.registry",
            "Generate the full genesis settlement corpus with the canonical genesis accumulator",
            "Verify all generated assets and proofs",
            "Compute the deterministic genesis state hash",
            "Persist one genesis binary pack per definition",
            "Write the stage-1 snapshot and supporting settlement artifacts",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[1]),
        vec![
            "Prepare stage-2 dirs, env vars, and the logged RPC stack",
            "Create the Alice, Bob, and Charlie wallets through RPC",
            "List wallets and verify the expected actor set",
            "Derive stealth receiver keys and export receiver cards",
            "Export public key JSON artifacts without leaking secrets",
            "Verify .wlt files and emit the wallet mapping artifact",
            "Validate backup creation and listing for every wallet",
            "Write the stage-2 snapshot last after all checks pass",
            "Verify show-seed integrity against the wallet creation response",
            "Derive and list receivers for each actor",
            "Keep wallet-secret debug output behind an explicit debug-only private lane",
            "Record stage-3 readiness for the actor runtime",
            "Check lifecycle locking by denying receiver listing after backgrounding",
            "Validate export/import persistence roundtrip for Bob",
            "Prove restart determinism for key derivation",
            "Check RPC log privacy and risk labeling",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[2]),
        vec![
            "Load the genesis binary packs and reconcile them with stage-1 snapshot counts",
            "Distribute assets across Alice, Bob, and Charlie according to the configured mode",
            "Persist claim artifacts, import assets into wallets, and write the prepared snapshot",
            "Optionally consume the genesis bins and always write the audit log",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[3]),
        vec![
            "Prepare the claim-publish directories and log surface",
            "Load the stage-3 snapshot and extract wallet import statistics",
            "Publish the claim package, inject genesis rights into the claim store, and export the post-claim storage view",
            "Write the claim-publish snapshot, logger, and audit artifacts",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[6]),
        vec![
            "Prepare transfer logs and transaction directories",
            "Pick Bob's recipient output from the stage-4 tx package",
            "Build the canonical asset leaf and decrypted asset view",
            "Check canonical scan and runtime scanner parity",
            "Run the report-only receive RPC flow without mutating claimed state",
            "Persist the receive handoff for the explicit claim stage",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[7]),
        vec![
            "Prepare transfer logs and transaction directories",
            "Pick Bob's recipient output from the stage-4 tx package",
            "Build the canonical asset leaf and decrypted asset view",
            "Check canonical scan and runtime scanner parity",
            "Run the explicit claim transition through recv_route",
            "Write the stage-5 leaf artifact",
            "Write the stage-5 snapshot",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[8]),
        vec![
            "Prepare bundle logs and transaction directories",
            "Write checkpoint fragment artifacts from the transfer lane",
            "Record that wallet mutation is represented by checkpoint artifacts",
            "Write the stage-6 bridge and exec_input handoff",
            "Confirm the ordered handoff pipeline toward storage apply",
            "Persist the targeted fragment pair",
            "Aggregate the fragment output amount",
            "Check replay-safe checkpoint invariants",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[11]),
        vec![
            "Prepare checkpoint-finalize logs and directories",
            "Load the shared stage-7 checkpoint summary",
            "Finalize and optionally seal the checkpoint publication",
            "Write the stage-8 checkpoint summary",
        ]
    );
    assert_eq!(
        step_actions(&design.stages[12]),
        vec![
            "Prepare the deterministic HJMT output directory and link the Stage-1 settlement manifest",
            "Seed generated asset and rights into the live settlement store",
            "Verify asset/right inclusion and fee-supported right transition proofs",
            "Verify right deletion and absent-right non-existence proofs",
            "Emit deterministic adaptive split and policy-transition evidence",
            "Emit bounded cache and scheduler metrics from warmed proof work",
            "Reopen the settlement store and re-verify every emitted example artifact",
            "Write settlement example, tamper, proof-size, cache, replay, and log artifacts",
        ]
    );
}

fn assert_post_conds(design: &DesignDoc) {
    assert_eq!(
        step_post(&design.stages[0]),
        vec![
            vec!["Genesis config is readable from the configured or fallback path", "GenesisSeed::from_config succeeds"],
            vec!["outputs/ exists", "outputs/genesis exists"],
            vec!["Each definition id is non-zero", "ctx.registry contains every generated definition"],
            vec!["Generated asset count matches the accumulator total", "ctx.assets and ctx.genesis_rights plus policy/voucher artifacts are populated"],
            vec!["verify_assets_all passes"],
            vec!["Repeated hash computation is stable"],
            vec!["genesis_Z00Z.bin, genesis_zUSD.bin, genesis_zNFT.bin, and genesis_zBurnSink.bin exist"],
            vec!["stage_1_snapshot.json exists", "genesis state hash, policy/right/voucher artifacts, and settlement manifest exist"],
        ]
    );
    assert_eq!(
        step_post(&design.stages[1]),
        vec![
            vec![
                "outputs/wallets, outputs/keys, and outputs/logs exist",
                "Logged RPC transport is ready"
            ],
            vec!["Three wallet ids are issued"],
            vec!["Wallet list contains alice, bob, and charlie"],
            vec!["Each actor has a non-degenerate owner handle"],
            vec!["alice_keys.json, bob_keys.json, and charlie_keys.json exist"],
            vec!["Every created .wlt opens successfully", "wlt_map.md exists"],
            vec!["Each wallet reports at least one backup"],
            vec!["stage_2_snapshot.json exists"],
            vec!["Recovered seed phrases match the create_wallet response"],
            vec!["Every actor exposes at least one receiver id"],
            vec!["Default stage-2 outputs contain no plaintext wallet secret artifact"],
            vec!["ctx.actors is ready for the claim lane"],
            vec!["list_receivers is rejected after the lifecycle event"],
            vec![
                "export_wallet_encrypted_payload.json exists",
                "Imported wallet id matches Bob's original wallet id"
            ],
            vec!["The same derived public key is produced before and after restart"],
            vec!["The RPC log contains no leaked passwords or seed phrases"],
        ]
    );
    assert_eq!(
        step_post(&design.stages[2]),
        vec![
            vec!["Loaded asset count matches stage_1_snapshot.assets_count"],
            vec!["All loaded assets are assigned exactly once"],
            vec![
                "stage_3_snapshot.json exists",
                "wallet_import_stats is present for all three actors"
            ],
            vec!["claim/audit_log.json exists"],
        ]
    );
    assert_eq!(
        step_post(&design.stages[3]),
        vec![
            vec!["claim_publish and logs_publish directories exist"],
            vec!["wallet_import_stats is present and non-empty"],
            vec![
                "claim_store_pub.json exists",
                "claim_post storage view is exported with right rows"
            ],
            vec![
                "stage_4_snapshot.json exists",
                "claim_publish/audit_log.json exists"
            ],
        ]
    );
}

fn assert_stage_results(run: &z00z_simulator::ScenarioResult) {
    for stage_id in 3..=12 {
        assert!(matches!(
            scenario_support::stage_res(run, stage_id),
            StageResult::Ok
        ));
    }
    assert!(matches!(
        scenario_support::stage_res(run, 13),
        StageResult::Ok
    ));
}

fn assert_stage13_redacted_text(text: &str, label: &str) {
    let lowered = text.to_ascii_lowercase();
    for forbidden in [
        "private key",
        "private_key",
        "private wallet key",
        "wallet private key",
        "owner_sk",
        "seed phrase",
        "seed_phrase",
        "mnemonic",
        "proof witness",
        "witness bytes",
        "witness_bytes",
        "payload bytes",
        "payload_bytes",
        "payload contents",
        "row key",
        "row_key",
        "redb row",
    ] {
        assert!(
            !lowered.contains(forbidden),
            "{label} leaked forbidden marker {forbidden}"
        );
    }

    let mut run = 0usize;
    for ch in text.chars() {
        if ch.is_ascii_hexdigit() {
            run += 1;
            assert!(run < 32, "{label} leaked long hex run");
        } else {
            run = 0;
        }
    }
}

fn assert_stage13_live_report(out_dir: &Path) {
    let report_path = out_dir.join("hjmt/hjmt_settlement_examples.json");
    let report: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(&report_path)
                .expect("read stage13 report")
                .as_bytes(),
        )
        .expect("parse stage13 report");

    assert_eq!(report["status"].as_str(), Some("hjmt_examples_complete"));
    assert_eq!(
        report["boundary_mode"].as_str(),
        Some("generalized_settlement")
    );
    assert_eq!(
        report["manifest_file"].as_str(),
        Some("hjmt/genesis_settlement_manifest.json")
    );
    assert_eq!(
        report["artifact_names"],
        serde_json::json!([
            "hjmt/hjmt_settlement_examples.json",
            "hjmt/hjmt_tamper_report.json",
            "hjmt/hjmt_proof_size_report.json",
            "hjmt/hjmt_cache_scheduler_metrics.json",
            "hjmt/hjmt_replay_roots.json",
            "hjmt/genesis_settlement_manifest.json"
        ])
    );
    assert!(report["settlement_state_root_hex"]
        .as_str()
        .is_some_and(|v| !v.is_empty()));
    assert_eq!(
        report["backend_modes"],
        serde_json::json!(["generalized", "adaptive"])
    );
    assert_eq!(
        report["artifact"]["verifier_status"].as_str(),
        Some("verified")
    );
    assert!(report["artifact"]["example_id"]
        .as_str()
        .is_some_and(|v| !v.is_empty()));
    assert!(report["artifact"]["backend_mode"]
        .as_str()
        .is_some_and(|v| !v.is_empty()));
    assert!(report["artifact"]["api_surface"]
        .as_str()
        .is_some_and(|v| !v.is_empty()));
    assert!(report["artifact"]["typed_error"].is_null());
    let examples = report["examples"]
        .as_array()
        .expect("stage13 examples array");
    assert_eq!(examples.len(), 8);
    for example in examples {
        assert_eq!(example["verifier_status"].as_str(), Some("verified"));
        assert_eq!(
            example["settlement_state_root_hex"].as_str(),
            report["settlement_state_root_hex"].as_str()
        );
        assert!(example["api_surface"]
            .as_str()
            .is_some_and(|v| !v.is_empty()));
        assert!(example["typed_error"].is_null());
        assert!(example["settlement_path"]
            .as_str()
            .is_some_and(|v| !v.is_empty()));
        if let Some(reject) = example["present_key_rejection"].as_str() {
            assert_stage13_redacted_text(reject, "stage13 present_key_rejection");
        }
    }
    let comparison_rows = report["comparison_rows"]
        .as_array()
        .expect("stage13 comparison rows");
    let surfaces = comparison_rows
        .iter()
        .filter_map(|row| row["proof_surface"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        surfaces,
        std::collections::BTreeSet::from(
            ["proof_blob_single", "proof_blob_vec", "batch_proof_v1",]
        )
    );
    let batch_rows = comparison_rows
        .iter()
        .filter(|row| row["proof_surface"].as_str() == Some("batch_proof_v1"))
        .collect::<Vec<_>>();
    assert!(!batch_rows.is_empty());
    let batch_shapes = batch_rows
        .iter()
        .filter_map(|row| row["path_shape"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        batch_shapes,
        std::collections::BTreeSet::from(["clustered", "scattered"])
    );
    let batch_counts = batch_rows
        .iter()
        .filter_map(|row| row["path_count"].as_u64())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [2u64, 8u64, 32u64] {
        assert!(batch_counts.contains(&required));
    }
    let batch_families = batch_rows
        .iter()
        .filter_map(|row| row["proof_family"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["inclusion", "deletion", "nonexistence"] {
        assert!(batch_families.contains(required));
    }
    for row in comparison_rows {
        assert_eq!(row["verifier_status"].as_str(), Some("verified"));
        assert!(row["typed_error"].is_null());
        assert_eq!(
            row["settlement_state_root_hex"].as_str(),
            report["settlement_state_root_hex"].as_str()
        );
        assert!(row["proof_size_bytes"]
            .as_u64()
            .is_some_and(|value| value > 0));
        assert!(row["verify_time_us"]
            .as_u64()
            .is_some_and(|value| value > 0));
        let paths = row["settlement_paths"]
            .as_array()
            .expect("settlement_paths");
        assert_eq!(
            u64::try_from(paths.len()).expect("path len"),
            row["path_count"].as_u64().expect("path_count")
        );
        if row["proof_surface"].as_str() == Some("batch_proof_v1") {
            assert_eq!(row["atomic_verdict"].as_str(), Some("accepted"));
            assert_eq!(row["shard_context_mode"].as_str(), Some("none"));
        }
    }
    let report_text = String::from_utf8(
        JsonCodec
            .serialize(&report)
            .expect("serialize stage13 report"),
    )
    .expect("stage13 report utf8");
    assert_no_legacy_send(&report_text, "stage13 report");
    let metrics_path = out_dir.join("hjmt/hjmt_cache_scheduler_metrics.json");
    let metrics: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(&metrics_path)
                .expect("read stage13 metrics report")
                .as_bytes(),
        )
        .expect("parse stage13 metrics report");
    assert_eq!(metrics["verifier_status"].as_str(), Some("verified"));
    assert_eq!(metrics["example_id"].as_str(), Some("E8_cache_scheduler"));
    assert_eq!(
        metrics["deterministic_parent_ordering"].as_bool(),
        Some(true)
    );
    assert!(metrics["typed_error"].is_null());
    let proof_path = out_dir.join("hjmt/hjmt_proof_size_report.json");
    let proof: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(&proof_path)
                .expect("read stage13 proof report")
                .as_bytes(),
        )
        .expect("parse stage13 proof report");
    assert_eq!(
        proof["artifact"]["verifier_status"].as_str(),
        Some("verified")
    );
    assert_eq!(
        proof["root_generation"].as_u64(),
        report["root_generation"].as_u64()
    );
    for entry in proof["entries"].as_array().expect("proof entries") {
        assert_eq!(entry["verifier_status"].as_str(), Some("verified"));
        assert!(entry["api_surface"]
            .as_str()
            .is_some_and(|value| !value.is_empty()));
        assert!(entry["proof_size_bytes"]
            .as_u64()
            .is_some_and(|value| value > 0));
        assert!(entry["verify_time_us"]
            .as_u64()
            .is_some_and(|value| value > 0));
        assert!(entry["typed_error"].is_null());
    }
    let proof_comparison_rows = proof["comparison_rows"]
        .as_array()
        .expect("proof comparison rows");
    assert_eq!(proof_comparison_rows.len(), comparison_rows.len());
    let replay_path = out_dir.join("hjmt/hjmt_replay_roots.json");
    let replay: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(&replay_path)
                .expect("read stage13 replay report")
                .as_bytes(),
        )
        .expect("parse stage13 replay report");
    assert_eq!(
        replay["artifact"]["verifier_status"].as_str(),
        Some("verified")
    );
    assert_eq!(
        replay["root_generation"].as_u64(),
        report["root_generation"].as_u64()
    );
    for entry in replay["replay_entries"].as_array().expect("replay entries") {
        assert_eq!(entry["verifier_status"].as_str(), Some("verified"));
        assert!(entry["typed_error"].is_null());
        assert_eq!(
            entry["settlement_state_root_hex"].as_str(),
            report["settlement_state_root_hex"].as_str()
        );
        assert_eq!(
            entry["reloaded_settlement_state_root_hex"].as_str(),
            report["settlement_state_root_hex"].as_str()
        );
    }
    let tamper_path = out_dir.join("hjmt/hjmt_tamper_report.json");
    let tamper: serde_json::Value = JsonCodec
        .deserialize(
            read_to_string(&tamper_path)
                .expect("read stage13 tamper report")
                .as_bytes(),
        )
        .expect("parse stage13 tamper report");
    assert_eq!(
        tamper["artifact"]["verifier_status"].as_str(),
        Some("verified")
    );
    assert_eq!(
        tamper["root_generation"].as_u64(),
        report["root_generation"].as_u64()
    );
    let tamper_case_ids = tamper["cases"]
        .as_array()
        .expect("tamper cases")
        .iter()
        .filter_map(|case| case["case_id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for required in [
        "batch_wrong_root_generation",
        "batch_reordered_paths",
        "batch_duplicate_path",
        "batch_mixed_proof_family",
        "batch_opening_kind_mismatch",
        "batch_leaf_family_mismatch",
        "batch_witness_ref_out_of_range",
        "batch_wrong_default_commitment",
        "batch_wrong_witness_domain",
        "batch_hash_material_count",
    ] {
        assert!(tamper_case_ids.contains(required));
    }
    for case in tamper["cases"].as_array().expect("tamper cases") {
        assert_eq!(case["verifier_status"].as_str(), Some("rejected"));
        assert!(case["api_surface"].as_str().is_some_and(|v| !v.is_empty()));
        assert!(case["proof_surface"]
            .as_str()
            .is_some_and(|v| !v.is_empty()));
        if case["case_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("batch_"))
        {
            assert_eq!(case["proof_surface"].as_str(), Some("batch_proof_v1"));
            assert!(case["path_count"].as_u64().is_some_and(|value| value > 1));
            assert!(matches!(
                case["path_shape"].as_str(),
                Some("clustered") | Some("scattered")
            ));
        }
        let class = case["typed_error"]["class"]
            .as_str()
            .expect("tamper typed_error class");
        let message = case["typed_error"]["message"]
            .as_str()
            .expect("tamper typed_error message");
        assert_stage13_redacted_text(class, "stage13 tamper typed_error class");
        assert_stage13_redacted_text(message, "stage13 tamper typed_error message");
    }
    assert!(out_dir.join("hjmt/store/settlement_state.redb").is_file());
    assert!(out_dir
        .join("hjmt/genesis_settlement_manifest.json")
        .is_file());
}

fn assert_stage13_live_log(out_dir: &Path) {
    let log_path = out_dir.join("hjmt/stage13_hjmt_examples.log");
    let rows: Vec<serde_json::Value> = read_to_string(&log_path)
        .expect("read stage13 log")
        .lines()
        .map(|line| {
            JsonCodec
                .deserialize(line.as_bytes())
                .expect("parse stage13 log row")
        })
        .collect();

    fn detail<'a>(rows: &'a [serde_json::Value], step: &str) -> &'a str {
        let row = rows
            .iter()
            .find(|row| row["step"].as_str() == Some(step))
            .unwrap_or_else(|| panic!("missing stage13 log row {step}"));
        assert_eq!(row["status"].as_str(), Some("ok"));
        row["detail"]
            .as_str()
            .unwrap_or_else(|| panic!("missing detail for {step}"))
    }

    assert!(detail(&rows, "S13-1").contains("hjmt output prepared"));
    assert!(detail(&rows, "S13-2").contains("yaml-generated genesis asset and rights were seeded"));
    assert!(detail(&rows, "S13-3").contains("fee-supported right transition"));
    assert!(detail(&rows, "S13-4").contains("present-key rejection stayed fail-closed"));
    assert!(detail(&rows, "S13-5").contains("policy-transition proofs"));
    assert!(detail(&rows, "S13-6").contains("cache and scheduler metrics"));
    assert!(detail(&rows, "S13-7").contains("reload-debug reopened"));
    assert!(detail(&rows, "S13-8").contains("wrote artifacts:"));

    let log_text = read_to_string(&log_path).expect("read stage13 log text");
    assert_no_snapshot_auth(&log_text, "stage13 log");
    assert_no_legacy_send(&log_text, "stage13 log");
}

fn read_trace(out_dir: &Path, name: &str) -> serde_json::Value {
    JsonCodec
        .deserialize(
            read_to_string(out_dir.join(name))
                .unwrap_or_else(|err| panic!("read {name}: {err}"))
                .as_bytes(),
        )
        .unwrap_or_else(|err| panic!("parse {name}: {err}"))
}

fn write_trace(out_dir: &Path, name: &str, value: &serde_json::Value) {
    let bytes = JsonCodec
        .serialize(value)
        .unwrap_or_else(|err| panic!("serialize {name}: {err}"));
    write_file(out_dir.join(name), &bytes).unwrap_or_else(|err| panic!("write {name}: {err}"));
}

fn expect_trace_tamper_reject(
    trace_name: &str,
    expected_field: &str,
    mutate: impl FnOnce(&mut serde_json::Value),
) {
    let _guard = stage_surface_lock();
    let (cfg_path, out_dir) = stage_surface_mutation_paths();
    let design_path = design_doc_path();

    let mut trace = read_trace(&out_dir, trace_name);
    mutate(&mut trace);
    write_trace(&out_dir, trace_name, &trace);

    let err = runner::validate_runtime_observability_artifacts(&cfg_path, &design_path, &out_dir)
        .expect_err("tampered runtime trace must reject");
    assert!(
        err.to_string().contains(expected_field),
        "tamper reject must mention {expected_field}, got {err}"
    );
}

fn assert_runtime_trace_pack(out_dir: &Path, cfg_path: &Path) {
    runner::validate_runtime_observability_artifacts(cfg_path, design_doc_path(), out_dir)
        .expect("runtime trace pack validates");

    let trace_names = [
        "cfg_flow.json",
        "tx_flow.json",
        "route_flow.json",
        "plan_flow.json",
        "journal_flow.json",
        "scope_flow.json",
        "proc_flow.json",
        "recovery_flow.json",
        "leaf_flow.json",
        "proof_flow.json",
        "pub_flow.json",
        "val_flow.json",
        "watch_flow.json",
    ];
    let traces = trace_names
        .iter()
        .map(|name| (*name, read_trace(out_dir, name)))
        .collect::<Vec<_>>();
    let run_meta = read_trace(out_dir, "run_meta.json");
    let wallet_scan = read_trace(out_dir, "wallet_scan.json");
    let hist_flow = read_trace(out_dir, "hist_flow.json");
    let occ_flow = read_trace(out_dir, "occ_flow.json");
    let stage7_summary = read_trace(out_dir, "transactions/checkpoint_s7.json");
    let sim_summary =
        read_to_string(out_dir.join("sim_summary.md")).expect("read simulator summary");

    let cfg_flow = &traces[0].1;
    assert_eq!(cfg_flow["active_profile"].as_str(), Some("SIM-SMALL"));
    assert_eq!(
        cfg_flow["supported_profiles"].as_array().map(Vec::len),
        Some(4)
    );
    assert_eq!(
        cfg_flow["heavy_only_profiles"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        cfg_flow["heavy_only_profiles"][0].as_str(),
        Some("SIM-BATCH-1000")
    );
    assert_eq!(
        cfg_flow["trace_files"]["cfg_flow_file"].as_str(),
        Some("cfg_flow.json")
    );
    assert_eq!(
        cfg_flow["trace_files"]["leaf_flow_file"].as_str(),
        Some("leaf_flow.json")
    );
    assert_eq!(
        cfg_flow["trace_files"]["val_flow_file"].as_str(),
        Some("val_flow.json")
    );
    assert_eq!(
        cfg_flow["trace_files"]["watch_flow_file"].as_str(),
        Some("watch_flow.json")
    );

    let semantic = cfg_flow["semantic_digest_hex"]
        .as_str()
        .expect("cfg_flow semantic digest");
    let route_digest = cfg_flow["route_table_digest"]
        .as_str()
        .expect("cfg_flow route digest");
    let config_set = cfg_flow["config_digest_set_hex"]
        .as_str()
        .expect("cfg_flow config digest set");
    let journal_lineage = cfg_flow["journal_lineage_digest_hex"]
        .as_str()
        .expect("cfg_flow journal digest");
    let process_topology = cfg_flow["process_topology_digest_hex"]
        .as_str()
        .expect("cfg_flow process digest");

    for (name, trace) in &traces {
        let expected_version = match *name {
            "leaf_flow.json" | "proof_flow.json" | "pub_flow.json" | "val_flow.json"
            | "watch_flow.json" => "phase057_publication_trace_v1",
            _ => "phase056_runtime_trace_v1",
        };
        assert_eq!(trace["trace_version"].as_str(), Some(expected_version));
        assert_eq!(
            trace["semantic_digest_hex"].as_str(),
            Some(semantic),
            "{name} semantic digest drift"
        );
        assert_eq!(
            trace["route_table_digest"].as_str(),
            Some(route_digest),
            "{name} route digest drift"
        );
        assert_eq!(
            trace["config_digest_set_hex"].as_str(),
            Some(config_set),
            "{name} config set drift"
        );
        assert_eq!(
            trace["journal_lineage_digest_hex"].as_str(),
            Some(journal_lineage),
            "{name} journal lineage drift"
        );
        assert_eq!(
            trace["process_topology_digest_hex"].as_str(),
            Some(process_topology),
            "{name} process topology drift"
        );
    }

    let journal_flow = &traces[4].1;
    assert!(journal_flow["journal_contract"]["cache_capacity"]
        .as_u64()
        .is_some_and(|value| value > 0));
    assert_eq!(
        journal_flow["cache_edge_samples"].as_array().map(Vec::len),
        Some(0)
    );

    let tx_flow = &traces[1].1;
    let plan_flow = &traces[3].1;
    let normalized_out = normalize_path(out_dir);
    let expected_tx_pkg = normalized_out
        .join("transactions/tx_alice_to_bob_pkg.json")
        .display()
        .to_string();
    let expected_stage13_report = normalized_out
        .join("hjmt/hjmt_settlement_examples.json")
        .display()
        .to_string();
    assert_eq!(
        tx_flow["tx_package_path"].as_str(),
        Some(expected_tx_pkg.as_str())
    );
    assert_eq!(
        tx_flow["hjmt_examples_report_path"].as_str(),
        Some(expected_stage13_report.as_str())
    );
    assert_eq!(plan_flow["planner_mode"].as_str(), Some("central"));
    assert!(plan_flow["planner_config_path"].as_str().is_some_and(
        |path| path.ends_with("config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml")
    ));

    let proc_flow = &traces[6].1;
    assert_eq!(
        proc_flow["process_topology"]["process_model"].as_str(),
        Some("os_process")
    );
    assert_eq!(
        proc_flow["process_topology"]["shard_mapping"].as_str(),
        Some("aggregator_owned")
    );
    assert_eq!(proc_flow["process_topology"]["agg_count"].as_u64(), Some(5));
    assert_eq!(
        proc_flow["process_topology"]["shard_count"].as_u64(),
        Some(7)
    );
    assert_eq!(
        proc_flow["process_topology"]["aggregators"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert!(proc_flow["process_topology"]["aggregators"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .enumerate()
            .all(|(idx, row)| row["process_id"].as_str() == Some(format!("agg-{idx}").as_str()))));

    let scope_flow = &traces[5].1;
    assert_eq!(
        scope_flow["trace_owner_home"].as_str(),
        Some("crates/z00z_storage/tests/test_hjmt_scope_birth.rs")
    );
    assert_eq!(scope_flow["private_tree_id_exposed"].as_bool(), Some(false));
    assert!(scope_flow["owner_contract_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["contract_id"].as_str()
            == Some("first_seen_terminal_birth")
            && row["first_seen_definition"].as_bool() == Some(true)
            && row["first_seen_serial"].as_bool() == Some(true))));
    assert_eq!(
        scope_flow["wallet_promotion_rows"].as_array().map(Vec::len),
        Some(2)
    );
    assert!(scope_flow["wallet_promotion_rows"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .all(|row| row["proof_validated"].as_bool() == Some(true)
                && row["pending_lifecycle_status"].as_str() == Some("pending_receive")
                && row["confirmed_lifecycle_status"].as_str() == Some("confirmed_receive"))));
    assert!(scope_flow["wallet_negative_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["receive_status"].as_str()
            == Some("RUNTIME_ASSET_MISS")
            && row["count"].as_u64() == Some(56))));
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

    let recovery_flow = &traces[7].1;
    assert_eq!(
        recovery_flow["recovery_owner_homes"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let val_flow = &traces[11].1;
    let watch_flow = &traces[12].1;
    let pub_flow = &traces[10].1;
    assert_eq!(
        val_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        watch_flow["publication_digest_hex"].as_str(),
        pub_flow["publication_digest_hex"].as_str()
    );
    assert_eq!(
        val_flow["binding_digest_hex"].as_str(),
        watch_flow["binding_digest_hex"].as_str()
    );
    assert_eq!(
        val_flow["checkpoint_id_hex"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(
        watch_flow["checkpoint_id_hex"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(
        val_flow["draft_id_hex"].as_str(),
        watch_flow["draft_id_hex"].as_str()
    );
    assert_eq!(val_flow["verdict_kind"].as_str(), Some("accepted"));
    assert_eq!(watch_flow["verdict_kind"].as_str(), Some("accepted"));
    assert_eq!(watch_flow["publication_state"].as_str(), Some("accepted"));
    assert_eq!(run_meta["execution_mode"].as_str(), Some("release"));
    assert_eq!(
        run_meta["process_map_file"].as_str(),
        Some("proc_flow.json")
    );
    assert_eq!(
        run_meta["wallet_scan_file"].as_str(),
        Some("wallet_scan.json")
    );
    assert_eq!(
        run_meta["artifact_inventory"].as_array().map(Vec::len),
        Some(21)
    );
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("asset_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("voucher_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("right_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert!(run_meta["artifact_inventory"]
        .as_array()
        .is_some_and(|rows| rows
            .iter()
            .any(|row| row["file"].as_str() == Some("occ_flow.json")
                && row["status"].as_str() == Some("emitted"))));
    assert_eq!(wallet_scan["actor"].as_str(), Some("charlie"));
    assert_eq!(
        wallet_scan["store_root_hex"].as_str(),
        stage7_summary["new_root_hex"].as_str()
    );
    assert_eq!(
        stage7_summary["wallet_scan_file"].as_str(),
        Some("wallet_scan.json")
    );
    assert!(sim_summary.contains("## Release Packet"));
    assert!(sim_summary.contains("- emitted: wallet_scan.json"));
    assert!(sim_summary.contains("- emitted: occ_flow.json"));
    assert!(sim_summary.contains("- emitted: asset_flow.json"));
    assert!(sim_summary.contains("- emitted: voucher_flow.json"));
    assert!(sim_summary.contains("- emitted: right_flow.json"));
    assert_eq!(hist_flow["trace_kind"].as_str(), Some("hist_flow"));
    assert_eq!(occ_flow["trace_kind"].as_str(), Some("occ_flow"));
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
    assert!(occ_flow["privacy_owner_contract_rows"]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row["proof_point"].as_str()
            == Some("test_metric_stays_private")
            && row["disclosure_guard"].as_str() == Some("coarse_only"))));
}

fn assert_no_snapshot_auth(text: &str, label: &str) {
    fn phrase(parts: &[&str]) -> String {
        parts.concat()
    }

    let lowered = text.to_ascii_lowercase();
    let forbidden = [
        phrase(&[
            "claimed-asset persistence remains in encrypted .wlt",
            " snapshot payloads",
        ]),
        phrase(&["snapshot is the live wallet", " asset authority"]),
        phrase(&[
            "snapshot-owned assets are the live wallet",
            " asset authority",
        ]),
        phrase(&["claimed_assets is the live wallet", " asset authority"]),
        phrase(&["snapshot remains the target", " live authority"]),
    ];
    for forbidden in &forbidden {
        assert!(
            !lowered.contains(forbidden),
            "{label} must not contain stale authority phrase: {forbidden}"
        );
    }
}

fn assert_no_legacy_send(text: &str, label: &str) {
    let lowered = text.to_ascii_lowercase();
    for forbidden in ["wallet.asset.send_asset", "send_asset_method"] {
        assert!(
            !lowered.contains(forbidden),
            "{label} must not contain legacy compatibility phrase: {forbidden}"
        );
    }
}

fn source_text(rel_path: &str) -> String {
    read_to_string(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel_path))
        .unwrap_or_else(|err| panic!("read {rel_path}: {err}"))
}

fn assert_s4_rpc_path() {
    let targets = [
        (
            "config",
            source_text("src/config.rs"),
            [
                "pub build_transaction_method: String",
                "default_build_transaction_method",
            ],
        ),
        (
            "config_defaults",
            source_text("src/config_defaults.rs"),
            [
                "fn default_build_transaction_method() -> String",
                "\"wallet.tx.build_transaction\"",
            ],
        ),
        (
            "scenario_cfg",
            source_text("src/scenario_1/scenario_config.yaml"),
            [
                "build_transaction_method: \"wallet.tx.build_transaction\"",
                "",
            ],
        ),
        (
            "stage4_persistence",
            source_text("src/scenario_1/stage_6/persistence.rs"),
            [
                "cfg.rpc.build_transaction_method",
                "\"asset_id\": hex::encode(send_asset_definition_id)",
            ],
        ),
    ];

    let forbidden = ["send_asset_method", "wallet.asset.send_asset"];

    for (name, source, required) in targets {
        for needle in required {
            if needle.is_empty() {
                continue;
            }
            assert!(
                source.contains(needle),
                "{name} must contain canonical Stage 4 path marker: {needle}"
            );
        }
        for needle in &forbidden {
            assert!(
                !source.contains(needle),
                "{name} must not contain legacy Stage 4 path marker: {needle}"
            );
        }
    }
}

#[test]
fn test_scenario1_stage_surface() {
    let _guard = stage_surface_lock();
    let out_dir = live_stage_surface_out();
    let run = synthetic_live_run();
    let design_path = design_doc_path();

    let design = DesignDoc::from_file(&design_path).expect("load design");
    let names: Vec<(u32, String)> = design
        .stages
        .iter()
        .map(|stage| (stage.stage, stage.name.clone()))
        .collect();
    assert_eq!(names, expected_stage_names());
    assert_stage_descs(&design);
    assert_rust_entries(&design);
    assert_config_sources(&design);
    assert_step_contracts(&design);
    assert_step_actions(&design);
    assert_post_conds(&design);
    assert_s4_rpc_path();

    let stage_names: Vec<(u32, String)> = run
        .stages
        .iter()
        .map(|stage| (stage.stage, stage.name.clone()))
        .collect();
    assert_eq!(stage_names, expected_stage_names());
    assert_stage_results(&run);
    assert_stage13_live_report(&out_dir);
    assert_stage13_live_log(&out_dir);
    assert_runtime_trace_pack(&out_dir, &live_stage_surface_cfg());
    assert_no_snapshot_auth(
        &read_to_string(out_dir.join("hjmt/hjmt_settlement_examples.json"))
            .expect("read stage13 report text"),
        "stage13 report",
    );

    let design_source = source_text("src/scenario_1/scenario_design.yaml");
    for needle in [
        "SIM-SMALL",
        "SIM-MEDIUM",
        "SIM-CACHE-EDGE",
        "SIM-BATCH-1000",
        "run_meta.json",
        "cfg_flow.json",
        "tx_flow.json",
        "route_flow.json",
        "plan_flow.json",
        "journal_flow.json",
        "scope_flow.json",
        "proc_flow.json",
        "recovery_flow.json",
        "leaf_flow.json",
        "proof_flow.json",
        "pub_flow.json",
        "val_flow.json",
        "watch_flow.json",
        "wallet_scan.json",
        "asset_flow.json",
        "voucher_flow.json",
        "right_flow.json",
        "hist_flow.json",
        "occ_flow.json",
        "sim_summary.md",
    ] {
        assert!(
            design_source.contains(needle),
            "scenario_design.yaml must keep runtime-observability string: {needle}"
        );
    }
}

#[test]
fn test_rejects_invalid_design_yaml() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let bad_design_path = temp.path().join("design_scenario_invalid.yaml");
    write_file(
        &bad_design_path,
        b"stages:\n  - stage: 1\n    name: broken_stage\n    steps: [\n",
    )
    .expect("write bad design yaml");

    let err =
        runner::validate_design_file(&bad_design_path).expect_err("invalid design yaml must fail");
    assert!(matches!(err, runner::Scenario1Err::Design(_)));
}

#[test]
fn test_rejects_blank_stage_name() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let bad_design_path = temp.path().join("design_scenario_blank_name.yaml");
    write_file(
        &bad_design_path,
        br#"stages:
  - stage: 1
    name: ""
    description: "broken"
    rust_entry: "stage_1::run(ctx)"
    steps:
      - id: "S1-1"
        action: "broken"
"#,
    )
    .expect("write blank-name design yaml");

    let err =
        runner::validate_design_file(&bad_design_path).expect_err("blank stage name must fail");
    assert!(matches!(err, runner::Scenario1Err::Design(_)));
}

#[test]
fn test_rejects_narrowed_stage_contract() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let bad_design_path = temp.path().join("design_scenario_narrowed.yaml");
    write_file(
        &bad_design_path,
        br#"stages:
  - stage: 1
    name: "genesis_init"
    description: "narrowed"
    rust_entry: "stage_1::run(ctx)"
    steps:
      - id: "S1-1"
        action: "narrowed"
"#,
    )
    .expect("write narrowed design yaml");

    let err = runner::validate_design_file(&bad_design_path)
        .expect_err("narrowed design contract must fail");
    assert!(matches!(err, runner::Scenario1Err::Design(_)));
}

#[test]
fn test_rejects_action_drift() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let bad_design_path = temp.path().join("design_scenario_action_drift.yaml");
    write_file(
        &bad_design_path,
        br#"stages:
    - stage: 1
        name: "genesis_init"
        description: |
            Generate deterministic genesis assets through the z00z_core genesis pipeline,
            persist per-definition binary packs, and snapshot the canonical stage-1 state.
        rust_entry: "stage_1::run(ctx)"
        config_source: "scenario_config.yaml::stage1_paths + stage1_genesis_config"
        steps:
            - id: "S1-1"
                action: "drifted action"
                post_conditions:
                    - "Genesis config is readable from the configured or fallback path"
                    - "GenesisSeed::from_config succeeds"
            - id: "S1-2"
                action: "Prepare the stage-1 output directories"
            - id: "S1-3"
                action: "Create every AssetDefinition and register it in ctx.registry"
            - id: "S1-4"
                action: "Generate all genesis assets with the canonical genesis accumulator"
            - id: "S1-5"
                action: "Verify all generated assets and proofs"
            - id: "S1-6"
                action: "Compute the deterministic genesis state hash"
            - id: "S1-7"
                action: "Persist one genesis binary pack per definition"
            - id: "S1-8"
                action: "Write the stage-1 snapshot and supporting hash artifacts"
"#,
    )
    .expect("write action-drift design yaml");

    let err = runner::validate_design_file(&bad_design_path).expect_err("action drift must fail");
    assert!(matches!(err, runner::Scenario1Err::Design(_)));
}

#[test]
fn test_rejects_desc_drift() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let bad_design_path = temp.path().join("design_scenario_desc_drift.yaml");
    write_file(
        &bad_design_path,
        br#"stages:
    - stage: 1
        name: "genesis_init"
        description: "drifted description"
        rust_entry: "stage_1::run(ctx)"
        config_source: "scenario_config.yaml::stage1_paths + stage1_genesis_config"
        steps:
            - id: "S1-1"
                action: "Load the canonical genesis config and validate the derived seed"
                post_conditions:
                    - "Genesis config is readable from the configured or fallback path"
                    - "GenesisSeed::from_config succeeds"
            - id: "S1-2"
                action: "Prepare the stage-1 output directories"
            - id: "S1-3"
                action: "Create every AssetDefinition and register it in ctx.registry"
            - id: "S1-4"
                action: "Generate all genesis assets with the canonical genesis accumulator"
            - id: "S1-5"
                action: "Verify all generated assets and proofs"
            - id: "S1-6"
                action: "Compute the deterministic genesis state hash"
            - id: "S1-7"
                action: "Persist one genesis binary pack per definition"
            - id: "S1-8"
                action: "Write the stage-1 snapshot and supporting hash artifacts"
"#,
    )
    .expect("write desc-drift design yaml");

    let err =
        runner::validate_design_file(&bad_design_path).expect_err("description drift must fail");
    assert!(matches!(err, runner::Scenario1Err::Design(_)));
}

#[test]
fn test_rejects_post_drift() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let bad_design_path = temp.path().join("design_scenario_post_drift.yaml");
    write_file(
        &bad_design_path,
        br#"stages:
    - stage: 1
        name: "genesis_init"
        description: |
            Generate deterministic genesis assets through the z00z_core genesis pipeline,
            persist per-definition binary packs, and snapshot the canonical stage-1 state.
        rust_entry: "stage_1::run(ctx)"
        config_source: "scenario_config.yaml::stage1_paths + stage1_genesis_config"
        steps:
            - id: "S1-1"
                action: "Load the canonical genesis config and validate the derived seed"
                post_conditions:
                    - "drifted post condition"
            - id: "S1-2"
                action: "Prepare the stage-1 output directories"
            - id: "S1-3"
                action: "Create every AssetDefinition and register it in ctx.registry"
            - id: "S1-4"
                action: "Generate all genesis assets with the canonical genesis accumulator"
            - id: "S1-5"
                action: "Verify all generated assets and proofs"
            - id: "S1-6"
                action: "Compute the deterministic genesis state hash"
            - id: "S1-7"
                action: "Persist one genesis binary pack per definition"
            - id: "S1-8"
                action: "Write the stage-1 snapshot and supporting hash artifacts"
"#,
    )
    .expect("write post-drift design yaml");

    let err = runner::validate_design_file(&bad_design_path).expect_err("post drift must fail");
    assert!(matches!(err, runner::Scenario1Err::Design(_)));
}

#[test]
fn test_boundary_wording_stays_narrow() {
    let spend_verification =
        source_text("../z00z_wallets/src/tx/spend_verification.rs").to_lowercase();

    assert!(
        spend_verification.contains("verify the current public spend statement only."),
        "public spend verifier must stay explicitly scoped to the current statement"
    );
    assert!(
        spend_verification.contains("delivered")
            && spend_verification.contains("persisted public spend contract")
            && spend_verification.contains("deterministic nullifier semantics surface")
            && spend_verification.contains("current proof/auth seam")
            && spend_verification.contains("already live")
            && spend_verification.contains("receiver-secret")
            && spend_verification.contains("`s_out`")
            && spend_verification.contains("wallet-local post-scan exclusivity gate")
            && spend_verification.contains("validator-facing")
            && spend_verification.contains("public trustless theorem"),
        "public spend verifier wording must keep the boundary narrow while reflecting shipped nullifier closure"
    );
}

#[test]
fn test_prohibited_shortcuts_stay_explicit() {
    let context =
        source_text("../../.planning/phases/000/040-spend-proof/040-CONTEXT.md").to_lowercase();
    let todo = source_text("../../.planning/phases/000/040-spend-proof/040-TODO.md").to_lowercase();
    let closeout = source_text("../../.planning/phases/000/040-spend-proof/040-CLOSEOUT-GATES.md")
        .to_lowercase();

    assert!(
        context.contains("prohibited shortcut mirror")
            && context.contains("do not claim stark proof support until `txproofwire` is non-empty and wired")
            && context.contains("do not introduce `receiver_cards` into the regular persisted package")
            && context.contains("do not replace fee-as-output semantics with a separate `c_fee` contract")
            && context.contains("do not mix wallet `compute_leaf_ad()` with crypto `derive_leaf_ad()` in the same runtime path without a documented migration plan"),
        "phase 040 context mirror must keep every shortcut prohibition explicit"
    );
    assert!(
        todo.contains("040-14 prohibited shortcut checklist")
            && todo.contains("re-check this shortcut checklist before marking `040-07` complete")
            && todo.contains("- [x] do not claim stark proof support until `txproofwire` is non-empty and wired through `txproofverifier`.")
            && todo.contains("- [x] do not introduce `receiver_cards` into the regular persisted package.")
            && todo.contains("- [x] do not replace fee-as-output semantics with a separate `c_fee` contract unless verifier logic, tests, and spec are migrated together.")
            && todo.contains("- [x] do not mix wallet `compute_leaf_ad()` with crypto `derive_leaf_ad()` in the same runtime path without a documented migration plan.")
            && todo.contains("- [x] re-check the stark shortcut against the stronger live boundary from `040-context.md`: non-empty carrier wiring is necessary, but standalone-proof closure remains prohibited until the remaining checkpoint replay hooks are closed."),
        "phase 040 TODO must keep the shortcut checklist and its validation hook explicit"
    );
    assert!(
        closeout.contains("## 040-14 prohibited shortcut checklist")
            && closeout.contains("stark proof support | still prohibited")
            && closeout.contains("one canonical internal")
            && closeout.contains("theorem-relation path")
            && closeout.contains("`receiver_cards` in the regular persisted package | not introduced; the live tx package carries compact receiver authorization material")
            && closeout.contains("separate `c_fee` contract | not introduced; fee-as-output semantics remain the live contract on both the verifier path and the persisted tx package surface.")
            && closeout.contains("mixed `compute_leaf_ad()` / `derive_leaf_ad()` runtime path | not introduced; the wallet-local and crypto-boundary formulas stay separate")
            && closeout.contains("phase 040 completion gate re-check")
            && closeout.contains("`040-01` through `040-07` complete | pass")
            && closeout.contains("all mandatory tests listed above exist and are green | pass")
            && closeout.contains("still aligned | pass")
            && closeout.contains("retired superseded draft | pass")
            && closeout.contains("shortcut checklist re-run before marking `040-07` complete | pass"),
        "phase 040 closeout ledger must record both shortcut results and the completion-gate recheck explicitly"
    );
}

#[test]
fn test_shortcuts_tx_package_surface() {
    let _guard = stage_surface_lock();
    let out_dir = live_stage_surface_out();
    assert_stage_results(&synthetic_live_run());

    let tx_pkg_path = out_dir
        .join("transactions")
        .join("tx_alice_to_bob_pkg.json");
    let tx_pkg_text =
        read_to_string(&tx_pkg_path).expect("read live tx package for shortcut guard");
    let tx_pkg: serde_json::Value = JsonCodec
        .deserialize(tx_pkg_text.as_bytes())
        .expect("parse live tx package json");
    let tx_pkg_text = tx_pkg_text.to_lowercase();

    assert!(
        !tx_pkg_text.contains("receiver_cards"),
        "live persisted tx package must not grow a receiver_cards shortcut surface"
    );
    assert!(
        !tx_pkg_text.contains("\"c_fee\""),
        "live persisted tx package must not grow a separate c_fee shortcut surface"
    );

    let proof_suite = tx_pkg["tx"]["proof"]["spend"]["proof_suite"]
        .as_str()
        .expect("live proof suite missing");
    assert_eq!(
        proof_suite, "regular_spend_theorem_bpplus",
        "live tx package must carry the canonical theorem suite on the shipped runtime path"
    );

    let receiver_card_compact = tx_pkg["tx"]["auth"]["spend"]["receiver_card_compact"]
        .as_str()
        .expect("live receiver_card_compact missing");
    assert!(
        !receiver_card_compact.is_empty(),
        "live tx package must keep compact receiver auth material explicit"
    );
}

#[test]
fn test_boundary_closure() {
    let integrity =
        source_text("../../.planning/phases/000/040-spend-proof/040-INTEGRITY-GATES.md")
            .to_lowercase();
    let summary =
        source_text("../../.planning/phases/000/040-spend-proof/040-05-SUMMARY.md").to_lowercase();
    let closeout = source_text("../../.planning/phases/000/040-spend-proof/040-CLOSEOUT-GATES.md")
        .to_lowercase();
    let uat = source_text("../../.planning/phases/000/040-spend-proof/040-UAT.md").to_lowercase();
    let validation =
        source_text("../../.planning/phases/000/040-spend-proof/040-VALIDATION.md").to_lowercase();
    let historical_boundary = ["statement-bound", "envelope"].join(" ");

    assert!(
        integrity.contains("040-09")
            && integrity.contains("040-10")
            && integrity.contains("canonical theorem preservation")
            && integrity.contains("verify_spend_rules(...)")
            && integrity.contains("canonical suite id")
            && integrity.contains("regular_spend_theorem_bpplus")
            && integrity.contains("no version-suffixed proof branches")
            && integrity.contains("no bridge shadow layer")
            && integrity.contains("wallet, checkpoint, and rollup"),
        "phase 040 integrity gates must record the active canonical theorem closure truth explicitly"
    );
    assert!(
        summary.contains("040-09")
            && summary.contains("then-current status")
            && summary.contains(&historical_boundary)
            && summary.contains("passed for the workspace state captured at the time this")
            && summary.contains("summary was written")
            && summary.contains("attempted to close `040-09`")
            && summary.contains("the 2026-04-27 repository-backed audit reopened that claim")
            && summary.contains("historical summary alone, for live repository truth"),
        "phase 040 historical summary must stay historical instead of masquerading as the live theorem status"
    );
    assert!(
        closeout.contains("concrete regular-tx prover | closed for the internal canonical theorem carrier and suite")
            && closeout.contains("concrete regular-tx verifier | still public-artifact bounded")
            && closeout.contains("package admission can succeed while checkpoint apply still rejects a tampered exec row")
            && closeout.contains("040-10` retires alternate suite names and lands the canonical internal theorem relation")
            && closeout.contains("internal canonical theorem package path"),
        "phase 040 closeout matrix must stay aligned with the active internal theorem-relation boundary"
    );
    assert!(
        uat.contains("status: complete")
            && uat.contains("internal theorem-relation closure readiness")
            && uat.contains("regular_spend_theorem_bpplus")
            && uat.contains("040-10")
            && uat.contains("result: [passed for internal relation; public proof closure remains open]")
            && uat.contains("public/trustless proof-of-knowledge")
            && uat.contains("checkpoint theorem finality")
            && uat.contains("rollup settlement closure"),
        "phase 040 UAT ledger must keep the active internal relation closeout explicit without public proof overclaim"
    );
    assert!(
        validation.contains("status: in_progress")
            && validation.contains("040-09 | 10 | t1")
            && validation.contains("040-10 | 10 | t2")
            && validation.contains("canonical internal theorem carrier and backend")
            && validation.contains("wallet proof-generation path")
            && validation.contains("040-cg | 10 | t6")
            && validation.contains("public/trustless proof-of-knowledge")
            && validation.contains(
                "approval: passed for internal theorem-relation closure on 2026-04-28"
            ),
        "phase 040 validation ledger must keep the active internal theorem-relation closeout explicit"
    );
}

#[test]
fn test_risk_categories_stay_separate() {
    let witness_gate = source_text("../z00z_wallets/src/tx/witness_gate.rs").to_lowercase();
    let ownership_security =
        source_text("../z00z_wallets/tests/test_asset_ownership_security.rs").to_lowercase();

    assert!(
        witness_gate.contains("does not close withholding risk before")
            && witness_gate.contains("prove public anti-theft closure"),
        "wallet-local spend witness wording must stay separate from withholding and public closure"
    );
    assert!(
        ownership_security.contains("withholding before publication")
            && ownership_security.contains("validator-facing anti-theft")
            && ownership_security.contains("remain separate open risks"),
        "asset ownership tests must keep the theft windows distinct"
    );
}

#[test]
fn test_continuity_stays_package_coupled() {
    let checkpoint_state = source_text("../z00z_wallets/src/tx/state_checkpoint.rs").to_lowercase();
    let bundle_lane =
        source_text("../z00z_simulator/src/scenario_1/stage_9/bundle_lane_impl.rs").to_lowercase();
    let stage12 = source_text("../z00z_simulator/src/scenario_1/stage_12/mod.rs").to_lowercase();

    assert!(
        checkpoint_state.contains("not standalone checkpoint-authorization")
            && checkpoint_state.contains("carriers"),
        "checkpoint public input wording must deny standalone authorization semantics"
    );
    assert!(
        bundle_lane.contains("current-stack package-coupled verifier")
            && bundle_lane.contains("accepted package path")
            && bundle_lane.contains("standalone authorization carriers"),
        "bundle lane verifier wording must keep exec continuity package-coupled"
    );
    assert!(
        stage12.contains("does not")
            && stage12.contains("upgrade the accepted")
            && stage12.contains("package-coupled continuity path")
            && stage12.contains("standalone checkpoint authority"),
        "stage 12 wording must not imply a standalone checkpoint backend"
    );
}

#[test]
fn test_rejects_detached_proof_bytes() {
    let checkpoint_state = source_text("../z00z_wallets/src/tx/state_checkpoint.rs").to_lowercase();
    let stage11 = source_text("../z00z_simulator/src/scenario_1/stage_11/mod.rs").to_lowercase();
    let stage12 = source_text("../z00z_simulator/src/scenario_1/stage_12/mod.rs").to_lowercase();

    assert!(
        checkpoint_state.contains("package-coupled checkpoint acceptance")
            && checkpoint_state.contains("package-coupled checkpoint integrity exists")
            && checkpoint_state.contains("detached proof bytes remain non-authoritative"),
        "checkpoint public input wording must freeze package-coupled continuity against proof-byte-only interpretations"
    );
    assert!(
        stage11.contains("package-coupled continuity")
            && stage11.contains("stage4 proof")
            && stage11.contains("input refs")
            && stage11.contains("stage6 bridge outputs")
            && stage11.contains("detached proof bytes")
            && stage11.contains("insufficient by themselves"),
        "stage 11 wording must pin continuity to the stage4 package plus stage6 bridge outputs"
    );
    assert!(
        stage12.contains("package-coupled continuity")
            && stage12.contains("detached proof bytes")
            && stage12.contains("insufficient by themselves"),
        "stage 12 wording must keep proof-byte-only interpretations below standalone authority"
    );
}

#[test]
fn test_rejects_missing_runtime_trace() {
    let _guard = stage_surface_lock();
    let (cfg_path, out_dir) = stage_surface_mutation_paths();
    let design_path = design_doc_path();

    remove_file(out_dir.join("proc_flow.json")).expect("remove proc_flow");
    let err = runner::validate_runtime_observability_artifacts(&cfg_path, &design_path, &out_dir)
        .expect_err("missing proc_flow must reject");
    assert!(err.to_string().contains("proc_flow.json"));
}

#[test]
fn rejects_tampered_tx_path() {
    expect_trace_tamper_reject("tx_flow.json", "tx_package_path", |trace| {
        trace["tx_package_path"] = serde_json::Value::String("/tmp/tampered_pkg.json".to_string());
    });
}

#[test]
fn rejects_tampered_plan_dir() {
    expect_trace_tamper_reject("plan_flow.json", "planner_evidence_dir", |trace| {
        trace["planner_evidence_dir"] =
            serde_json::Value::String("/tmp/tampered_evidence".to_string());
    });
}

#[test]
fn rejects_tampered_scope_flag() {
    expect_trace_tamper_reject("scope_flow.json", "private_tree_id_exposed", |trace| {
        trace["private_tree_id_exposed"] = serde_json::Value::Bool(true);
    });
}

#[test]
fn rejects_tampered_hist_migration() {
    expect_trace_tamper_reject("hist_flow.json", "old_public_root_hex", |trace| {
        trace["route_migration_rows"][0]["old_public_root_hex"] =
            serde_json::Value::String("deadbeef".repeat(8));
    });
}

#[test]
fn rejects_tampered_occ_guard() {
    expect_trace_tamper_reject("occ_flow.json", "disclosure_guard", |trace| {
        trace["occupancy_disclosure_verdicts"][0]["disclosure_guard"] =
            serde_json::Value::String("exact_count".to_string());
    });
}

#[test]
fn rejects_tampered_recovery_checks() {
    expect_trace_tamper_reject("recovery_flow.json", "startup_checks_required", |trace| {
        trace["startup_checks_required"] = serde_json::Value::Array(vec![
            serde_json::Value::String("route_codec".to_string()),
            serde_json::Value::String("tampered_check".to_string()),
        ]);
    });
}

#[test]
fn accepts_heavy_runtime_profile() {
    let _guard = stage_surface_lock();
    let temp = tempdir().expect("temp dir");
    let (cfg_path, design_path, out_dir) = scenario_support::make_cfg_in(temp.path(), |cfg| {
        cfg.runtime_observability
            .as_mut()
            .expect("runtime observability")
            .active_profile = "SIM-BATCH-1000".to_string();
    });

    let run = runner::run_with_paths(&cfg_path, &design_path).expect("heavy-only profile must run");
    assert!(run.is_ok());
    let cfg_flow = read_trace(&out_dir, "cfg_flow.json");
    assert_eq!(cfg_flow["active_profile"].as_str(), Some("SIM-BATCH-1000"));
    assert_eq!(
        cfg_flow["heavy_only_profiles"][0].as_str(),
        Some("SIM-BATCH-1000")
    );
}

#[test]
fn small_medium_stay_deterministic() {
    let _guard = stage_surface_lock();
    let small_temp = tempdir().expect("small temp dir");
    let (small_cfg_path, small_design_path, small_out_dir) =
        scenario_support::make_cfg_in(small_temp.path(), |cfg| {
            cfg.runtime_observability
                .as_mut()
                .expect("runtime observability")
                .active_profile = "SIM-SMALL".to_string();
        });
    run_scenario_bin(&small_cfg_path, &small_design_path);
    let before_small = read_trace(&small_out_dir, "cfg_flow.json")["semantic_digest_hex"]
        .as_str()
        .expect("first small semantic")
        .to_string();
    run_scenario_bin(&small_cfg_path, &small_design_path);
    let after_small = read_trace(&small_out_dir, "cfg_flow.json")["semantic_digest_hex"]
        .as_str()
        .expect("rerun small semantic")
        .to_string();
    assert_eq!(before_small, after_small);
    assert_eq!(
        read_trace(&small_out_dir, "cfg_flow.json")["active_profile"].as_str(),
        Some("SIM-SMALL")
    );

    let medium_temp = tempdir().expect("medium temp dir");
    let (cfg_path, design_path, out_dir) =
        scenario_support::make_cfg_in(medium_temp.path(), |cfg| {
            cfg.runtime_observability
                .as_mut()
                .expect("runtime observability")
                .active_profile = "SIM-MEDIUM".to_string();
        });
    run_scenario_bin(&cfg_path, &design_path);
    let first_digest = read_trace(&out_dir, "cfg_flow.json")["semantic_digest_hex"]
        .as_str()
        .expect("first medium digest")
        .to_string();
    run_scenario_bin(&cfg_path, &design_path);
    let second_digest = read_trace(&out_dir, "cfg_flow.json")["semantic_digest_hex"]
        .as_str()
        .expect("second medium digest")
        .to_string();
    assert_eq!(first_digest, second_digest);
    assert_eq!(
        read_trace(&out_dir, "cfg_flow.json")["active_profile"].as_str(),
        Some("SIM-MEDIUM")
    );
}

#[test]
fn cache_edge_uses_runtime_capacity() {
    let _guard = stage_surface_lock();
    let temp = tempdir().expect("temp dir");
    let (cfg_path, design_path, out_dir) = scenario_support::make_cfg_in(temp.path(), |cfg| {
        cfg.runtime_observability
            .as_mut()
            .expect("runtime observability")
            .active_profile = "SIM-CACHE-EDGE".to_string();
    });
    let run = runner::run_with_paths(&cfg_path, &design_path).expect("SIM-CACHE-EDGE run");
    assert!(run.is_ok());

    let journal_flow = read_trace(&out_dir, "journal_flow.json");
    let cap = journal_flow["journal_contract"]["cache_capacity"]
        .as_u64()
        .expect("cache capacity");
    let samples = journal_flow["cache_edge_samples"]
        .as_array()
        .expect("cache edge samples")
        .iter()
        .map(|value| value.as_u64().expect("cache sample"))
        .collect::<Vec<_>>();
    assert_eq!(samples, vec![cap.saturating_sub(1), cap, cap + 1, cap * 2]);
}
