use std::path::{Path, PathBuf};

use z00z_storage::checkpoint::{
    build_cp_draft, decode_exec_bin, derive_exec_id, CheckpointFsStore, CheckpointStore,
};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_dir, read_file},
};
use z00z_wallets::tx::TxPackage;

use crate::SimContext;

use super::charlie::refresh_charlie;
use super::{Stage11Apply, Stage11Cfg, Stage11Checkpoint, Stage11Load};
use crate::scenario_1::stage_4::export_post_tx_view;
use crate::scenario_1::stage_9::bundle_lane_impl::{
    load_frag, load_stage9_bridge, load_tx_pkg, Checkpoint, CheckpointPackageProofVerifier,
    CheckpointReplaySpentIndex,
};
use crate::scenario_1::stage_9::exec_input_builder::{
    checkpoint_from_draft, draft_link, exec_out_from_wire, in_ref_from_input,
};
use crate::scenario_1::stage_9::prep_snapshot_loader::{load_prep, resolve_input_path};

pub(super) fn apply_stage11(
    ctx: &mut SimContext,
    stage_id: u32,
    cfg: &Stage11Cfg,
    load: Stage11Load,
) -> Result<Stage11Apply, String> {
    verify_stage11_handoff(&load)?;
    let link = draft_link(load.snap_id, load.exec_id)?;
    let proof_verifier =
        CheckpointPackageProofVerifier::from_stage11(&load.pkg, &load.bridge.bridge_outputs)?;
    let spent_index = CheckpointReplaySpentIndex::from_exec(&load.exec)?;
    let draft = build_cp_draft(
        stage_id as u64,
        load.snap_id,
        &load.prep,
        &load.replay,
        &link,
        &load.exec,
        &proof_verifier,
        &spent_index,
    )
    .map_err(|e| e.to_string())?;
    let mut store = CheckpointFsStore::new(&cfg.tx_dir);
    let draft_id = store.save_draft(&draft).map_err(|e| e.to_string())?;
    export_post_tx_view(&cfg.out, load.snap_id, &load.prep, &load.exec, &draft)?;
    let cp: Checkpoint = checkpoint_from_draft(
        &draft,
        &load.bridge.bridge_outputs,
        &[load.frag_a, load.frag_b],
    )?;
    let charlie = refresh_charlie(
        ctx,
        cfg,
        &load.bridge.bridge_outputs,
        &load.pkg.tx_digest_hex,
    )?;

    Ok(Stage11Apply {
        summary: Stage11Checkpoint {
            stage: stage_id,
            prev_root_hex: cp.prev_root_hex,
            new_root_hex: cp.new_root_hex,
            draft_id_hex: hex::encode(draft_id.as_bytes()),
            exec_input_id_hex: hex::encode(load.exec_id.as_bytes()),
            snapshot_id_hex: hex::encode(load.snap_id.as_bytes()),
            spent_delta: cp.spent_delta,
            created_delta: cp.created_delta,
            fragment_ids: load.bridge.fragment_ids,
            charlie_detected_count: charlie.artifact.detected_count,
            charlie_detected_amount: charlie.artifact.total_detected_amount,
            wallet_invariant_ok: charlie.invariant_ok,
            wallet_scan_file: cfg.wallet_scan_file.clone(),
            status: "ok".to_string(),
        },
        draft_id_hex: hex::encode(draft_id.as_bytes()),
    })
}

fn verify_stage11_handoff(load: &Stage11Load) -> Result<(), String> {
    let [exec_tx] = load.exec.txs() else {
        return Err("stage7: expected exactly one checkpoint exec tx".to_string());
    };

    CheckpointPackageProofVerifier::verify_pkg_contract(&load.pkg)?;
    let expected_prev_root = CheckpointPackageProofVerifier::expected_prev_root(&load.pkg)?;
    if load.exec.prev_root() != expected_prev_root {
        return Err("stage7: exec prev_root mismatch with stage4 spend proof".to_string());
    }

    let expected_proof = JsonCodec
        .serialize(&load.pkg.tx.proof)
        .map_err(|e| format!("stage7: tx proof encode failed: {e}"))?;
    if exec_tx.tx_proof() != expected_proof {
        return Err("stage7: exec tx proof mismatch with stage4 package".to_string());
    }

    let expected_refs = load
        .pkg
        .tx
        .inputs
        .iter()
        .map(in_ref_from_input)
        .collect::<Result<Vec<_>, _>>()?;
    if exec_tx.input_refs() != expected_refs.as_slice() {
        return Err("stage7: exec input refs mismatch with stage4 package".to_string());
    }

    let expected_outputs = load
        .bridge
        .bridge_outputs
        .iter()
        .map(exec_out_from_wire)
        .collect::<Result<Vec<_>, _>>()?;
    if exec_tx.outputs() != expected_outputs.as_slice() {
        return Err("stage7: exec outputs mismatch with stage6 bridge outputs".to_string());
    }

    Ok(())
}

pub(super) fn load_stage11_checked(
    out: &Path,
    tx_dir: &Path,
    p6: &crate::config::Stage6PathsCfg,
    s4: &crate::config::Stage4TxPrepareCfg,
) -> Result<(Stage11Load, PathBuf, PathBuf), String> {
    let (load, bridge_path, exec_path) = load_stage11(out, tx_dir, &p6.checkpoint_file, s4, p6)?;
    if load.bridge.exec_input_id_hex != hex::encode(load.exec_id.as_bytes()) {
        return Err("stage6 bridge exec_input_id mismatch".to_string());
    }
    Ok((load, bridge_path, exec_path))
}

fn load_stage11(
    out: &Path,
    tx_dir: &Path,
    bridge_file: &str,
    s4: &crate::config::Stage4TxPrepareCfg,
    p6: &crate::config::Stage6PathsCfg,
) -> Result<(Stage11Load, PathBuf, PathBuf), String> {
    let bridge_path = tx_dir.join(bridge_file);
    let bridge = load_stage9_bridge(&bridge_path)?;
    let frag_a = load_frag(&tx_dir.join(&p6.frag1_file), "frag1")?;
    let frag_b = load_frag(&tx_dir.join(&p6.frag2_file), "frag2")?;
    let (pkg, snap_id, prep, replay) = load_stage4(out, tx_dir, s4)?;
    let (exec, exec_id, exec_path) = load_exec(tx_dir)?;

    Ok((
        Stage11Load {
            bridge,
            frag_a,
            frag_b,
            pkg,
            snap_id,
            prep,
            replay,
            exec,
            exec_id,
        },
        bridge_path,
        exec_path,
    ))
}

fn load_stage4(
    out: &Path,
    tx_dir: &Path,
    s4: &crate::config::Stage4TxPrepareCfg,
) -> Result<
    (
        TxPackage,
        z00z_storage::snapshot::PrepSnapshotId,
        z00z_storage::snapshot::PrepSnapshot,
        Vec<z00z_storage::snapshot::PrepReplayEntry>,
    ),
    String,
> {
    let s4_tx_path = resolve_input_path(out, &s4.paths.tx_pkg_file)?;
    let s4_prep_path = s4_tx_path
        .parent()
        .unwrap_or(tx_dir)
        .join("checkpoint_prep.json");
    let pkg = load_tx_pkg(&s4_tx_path)?;
    let (snap_id, prep, replay) = load_prep(&s4_prep_path)?;
    Ok((pkg, snap_id, prep, replay))
}

fn load_exec(
    tx_dir: &Path,
) -> Result<
    (
        z00z_storage::checkpoint::CheckpointExecInput,
        z00z_storage::checkpoint::CheckpointExecInputId,
        PathBuf,
    ),
    String,
> {
    let exec_path = only_bin(&CheckpointFsStore::new(tx_dir).exec_dir())?;
    let exec_bytes = read_file(&exec_path).map_err(|e| e.to_string())?;
    let exec = decode_exec_bin(&exec_bytes).map_err(|e| e.to_string())?;
    let exec_id = derive_exec_id(&exec_bytes);
    Ok((exec, exec_id, exec_path))
}

fn only_bin(dir: &Path) -> Result<PathBuf, String> {
    let mut files = read_dir(dir).map_err(|e| format!("failed reading {}: {e}", dir.display()))?;
    files.retain(|path| path.extension().and_then(|item| item.to_str()) == Some("bin"));
    if files.len() != 1 {
        return Err(format!("expected one .bin file in {}", dir.display()));
    }
    Ok(files.remove(0))
}
