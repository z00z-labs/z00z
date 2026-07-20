use std::path::Path;

use z00z_storage::checkpoint::audit::{CheckpointAudit, CheckpointAuditVersion};
use z00z_storage::checkpoint::{
    check_exec_root, check_link_ids, CheckpointFsStore, CheckpointStore,
};
use z00z_storage::fixture_support::checkpoint_fixtures;
use z00z_storage::snapshot::PrepSnapshotId;
use z00z_wallets::tx::TxPackage;

use super::final_refs::parse_refs;
use super::{Stage12Cfg, Stage12Summary};
use crate::scenario_1::stage_11::Stage11Checkpoint;
use crate::scenario_1::stage_4::export_post_tx_final_view;
use crate::scenario_1::stage_9::bundle_lane_impl::{load_stage9_bridge, load_tx_pkg, Stage9Bridge};
use crate::scenario_1::stage_9::exec_input_builder::build_attest_proof;
use crate::scenario_1::stage_9::prep_snapshot_loader::{load_prep, resolve_input_path};

pub(super) fn finalize_stage12(
    out: &Path,
    tx_dir: &Path,
    cfg: &Stage12Cfg,
    checkpoint: &Stage11Checkpoint,
    summary: &mut Stage12Summary,
) -> Result<(), String> {
    let refs = parse_refs(checkpoint)?;
    let mut store = CheckpointFsStore::new(tx_dir);
    let draft = store
        .load_draft(&refs.draft_id)
        .map_err(|e| e.to_string())?;
    let exec = store
        .load_exec_input(&refs.exec_id)
        .map_err(|e| format!("stage8 exec_input load failed: {e}"))?;
    let (snap_id, snapshot) = load_stage4_snapshot(out, tx_dir, &cfg.s4)?;
    check_stage12_refs(&draft, &exec, refs.snap_id, snap_id, &snapshot)?;
    let bridge = load_stage9_bridge_artifact(out, &cfg.p6)?;
    check_stage12_fragment_ids(checkpoint, &bridge)?;
    let pkg = load_tx_package(out, &cfg.s4)?;
    let proof = build_attest_proof(&draft, &pkg, refs.snap_id, refs.exec_id)?;
    let manifest = checkpoint_fixtures::archive_manifest(&draft, &exec, refs.exec_id);
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    let statement_core = checkpoint_fixtures::statement_core(&exec);
    store
        .stage_publication_contract(refs.exec_id, &statement_core, &manifest, &da_reference)
        .map_err(|e| e.to_string())?;
    let link = store
        .seal_artifact(&draft, proof, refs.snap_id, refs.exec_id)
        .map_err(|e| e.to_string())?;
    check_link_ids(refs.snap_id, &link, &exec)
        .map_err(|e| format!("stage8 link binding failed: {e}"))?;
    let artifact = store
        .load_artifact(&link.checkpoint_id())
        .map_err(|e| format!("stage8 artifact load failed: {e}"))?;
    let audit = CheckpointAudit::new(
        CheckpointAuditVersion::CURRENT,
        link.checkpoint_id(),
        bridge.fragment_ids.clone(),
    )
    .map_err(|e| e.to_string())?;
    store.save_audit(&audit).map_err(|e| e.to_string())?;
    export_post_tx_final_view(out, &artifact, &link, &audit)?;
    summary.fragment_ids = bridge.fragment_ids;
    summary.checkpoint_id_hex = Some(hex::encode(link.checkpoint_id().as_bytes()));
    summary.artifact_path = Some("transactions/artifacts/checkpoints/final".to_string());
    summary.link_path = Some("transactions/artifacts/checkpoints/links".to_string());
    summary.audit_path = Some("transactions/artifacts/checkpoints/audit".to_string());
    summary.status = "ok".to_string();
    Ok(())
}

fn load_stage9_bridge_artifact(
    out: &Path,
    p6: &crate::config::Stage6PathsCfg,
) -> Result<Stage9Bridge, String> {
    let path = out.join(&p6.transactions_dir).join(&p6.checkpoint_file);
    load_stage9_bridge(&path)
}

fn check_stage12_fragment_ids(
    checkpoint: &Stage11Checkpoint,
    bridge: &Stage9Bridge,
) -> Result<(), String> {
    if checkpoint.fragment_ids != bridge.fragment_ids {
        return Err("stage8 fragment_ids mismatch".to_string());
    }
    Ok(())
}

fn load_stage4_snapshot(
    out: &Path,
    tx_dir: &Path,
    s4: &crate::config::Stage4TxPrepareCfg,
) -> Result<(PrepSnapshotId, z00z_storage::snapshot::PrepSnapshot), String> {
    let s4_tx_path = resolve_input_path(out, &s4.paths.tx_pkg_file)?;
    let s4_prep_path = s4_tx_path
        .parent()
        .unwrap_or(tx_dir)
        .join("checkpoint_prep.json");
    let (snap_id, snapshot, _) = load_prep(&s4_prep_path)?;
    Ok((snap_id, snapshot))
}

fn check_stage12_refs(
    draft: &z00z_storage::checkpoint::CheckpointDraft,
    exec: &z00z_storage::checkpoint::CheckpointExecInput,
    want_snap_id: PrepSnapshotId,
    got_snap_id: PrepSnapshotId,
    snapshot: &z00z_storage::snapshot::PrepSnapshot,
) -> Result<(), String> {
    if want_snap_id != got_snap_id {
        return Err("stage8 snapshot_id mismatch".to_string());
    }
    if exec.prep_snapshot_id() != want_snap_id {
        return Err("stage8 exec_input snapshot_id mismatch".to_string());
    }
    if draft.prev_root() != exec.prev_root() {
        return Err("stage8 draft/exec prev_root mismatch".to_string());
    }
    check_exec_root(snapshot, exec).map_err(|e| format!("stage8 exec root mismatch: {e}"))?;
    Ok(())
}

fn load_tx_package(
    out: &Path,
    s4: &crate::config::Stage4TxPrepareCfg,
) -> Result<TxPackage, String> {
    let s4_tx_path = resolve_input_path(out, &s4.paths.tx_pkg_file)?;
    load_tx_pkg(&s4_tx_path)
}
