use tempfile::TempDir;
use z00z_core::assets::AssetLeaf;
use z00z_storage::fixture_support::checkpoint_fixtures;
use z00z_storage::{
    checkpoint::{
        audit::{CheckpointAudit, CheckpointAuditVersion},
        decode_link_bin, derive_checkpoint_id, CheckpointDraft, CheckpointExecInput,
        CheckpointExecInputId, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointFsStore, CheckpointInRef, CheckpointLink, CheckpointLinkVersion, CheckpointProof,
        CheckpointStore, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, DefinitionId, SerialId, SettlementStore, TerminalLeaf},
    snapshot::{build_snapshot, PrepFsStore, PrepSnapshotId, PrepSnapshotStore},
    CheckpointError,
};
use z00z_utils::codec::{BincodeCodec, Codec};

fn temp_dir() -> TempDir {
    TempDir::new().expect("temp dir")
}

fn empty_check_root() -> CheckRoot {
    CheckRoot::from(
        SettlementStore::new()
            .settlement_root()
            .expect("empty settlement root"),
    )
}

fn draft(seed: u8) -> CheckpointDraft {
    draft_with_roots(71, empty_check_root(), seed)
}

fn draft_exact(
    height: u64,
    prev_root: CheckRoot,
    new_root: CheckRoot,
    seed: u8,
) -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        height,
        prev_root,
        new_root,
        vec![SpentEnt::new([seed.wrapping_add(2); 32])],
        vec![CreatedEnt::new(
            [seed.wrapping_add(3); 32],
            [seed.wrapping_add(4); 32],
        )],
    )
}

fn draft_with_roots(height: u64, prev_root: CheckRoot, seed: u8) -> CheckpointDraft {
    draft_exact(
        height,
        prev_root,
        CheckRoot::new([seed.wrapping_add(1); 32]),
        seed,
    )
}

fn proof(
    draft: &CheckpointDraft,
    _byte: u8,
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
) -> CheckpointProof {
    draft.attest_proof(snap_id, exec_id).expect("proof")
}

fn exec(snapshot_id: PrepSnapshotId, prev_root: CheckRoot, seed: u8) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new(
                [seed; 32],
                SerialId::new(u32::from(seed) + 1),
            )],
            vec![CheckpointExecOut::new(
                DefinitionId::new([seed.wrapping_add(1); 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(u32::from(seed) + 17)),
            )
            .expect("exec out")],
            vec![seed.wrapping_add(2)],
        )
        .expect("exec tx")],
    )
    .expect("exec")
}

fn persist_replay_rows(
    root: &std::path::Path,
    store: &mut CheckpointFsStore,
    prev_root: CheckRoot,
    seed: u8,
) -> (PrepSnapshotId, CheckpointExecInputId) {
    let (snapshot, snapshot_id) = build_snapshot(prev_root, Vec::new()).expect("snapshot");
    let mut snap_store = PrepFsStore::new(root);
    let saved_id = snap_store.save_snapshot(&snapshot).expect("save snapshot");
    assert_eq!(saved_id, snapshot_id);
    let exec_id = store
        .save_exec_input(&exec(snapshot_id, prev_root, seed))
        .expect("save exec");
    (snapshot_id, exec_id)
}

fn stage_contract(
    store: &mut CheckpointFsStore,
    draft: &CheckpointDraft,
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
    seed: u8,
) {
    let exec = exec(snap_id, draft.prev_root(), seed);
    let manifest = checkpoint_fixtures::archive_manifest(draft, &exec, exec_id);
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    store
        .stage_publication_contract(exec_id, &manifest, &da_reference)
        .expect("stage publication contract");
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct LinkWire {
    version: CheckpointLinkVersion,
    checkpoint_id: z00z_storage::checkpoint::CheckpointId,
    prev_checkpoint_id: Option<z00z_storage::checkpoint::CheckpointId>,
    prep_snapshot_id: PrepSnapshotId,
    exec_input_id: CheckpointExecInputId,
    link_bind_ver: u8,
    link_bind: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct UnboundLinkWire {
    version: CheckpointLinkVersion,
    checkpoint_id: z00z_storage::checkpoint::CheckpointId,
    prep_snapshot_id: PrepSnapshotId,
    exec_input_id: CheckpointExecInputId,
}

#[test]
fn test_artifact_persists_roundtrip_link() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = draft(1);
    let (snap_id, exec_id) = persist_replay_rows(dir.path(), &mut store, draft.prev_root(), 1);
    stage_contract(&mut store, &draft, snap_id, exec_id, 1);
    let link = store
        .seal_artifact(&draft, proof(&draft, 9, snap_id, exec_id), snap_id, exec_id)
        .expect("seal artifact");
    let got = store.load_link(&link.checkpoint_id()).expect("load link");

    assert_eq!(got, link);
}

#[test]
fn test_seal_infers_prev_checkpoint_id() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let first = draft_exact(71, empty_check_root(), empty_check_root(), 1);
    let (first_snap_id, first_exec_id) =
        persist_replay_rows(dir.path(), &mut store, first.prev_root(), 1);
    stage_contract(&mut store, &first, first_snap_id, first_exec_id, 1);
    let first_link = store
        .seal_artifact(
            &first,
            proof(&first, 9, first_snap_id, first_exec_id),
            first_snap_id,
            first_exec_id,
        )
        .expect("seal first artifact");

    let second = draft_with_roots(72, first.new_root(), 11);
    let (second_snap_id, second_exec_id) =
        persist_replay_rows(dir.path(), &mut store, second.prev_root(), 2);
    stage_contract(&mut store, &second, second_snap_id, second_exec_id, 2);
    let second_link = store
        .seal_artifact(
            &second,
            proof(&second, 8, second_snap_id, second_exec_id),
            second_snap_id,
            second_exec_id,
        )
        .expect("seal second artifact");

    assert_eq!(
        second_link.prev_checkpoint_id(),
        Some(first_link.checkpoint_id())
    );
}

#[test]
fn test_conflicting_successor_reuse_rejects() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let first = draft_exact(71, empty_check_root(), empty_check_root(), 1);
    let (first_snap_id, first_exec_id) =
        persist_replay_rows(dir.path(), &mut store, first.prev_root(), 1);
    stage_contract(&mut store, &first, first_snap_id, first_exec_id, 1);
    store
        .seal_artifact(
            &first,
            proof(&first, 9, first_snap_id, first_exec_id),
            first_snap_id,
            first_exec_id,
        )
        .expect("seal first artifact");

    let second = draft_with_roots(72, first.new_root(), 11);
    let (second_snap_id, second_exec_id) =
        persist_replay_rows(dir.path(), &mut store, second.prev_root(), 2);
    stage_contract(&mut store, &second, second_snap_id, second_exec_id, 2);
    store
        .seal_artifact(
            &second,
            proof(&second, 8, second_snap_id, second_exec_id),
            second_snap_id,
            second_exec_id,
        )
        .expect("seal second artifact");

    let fork = draft_with_roots(73, first.new_root(), 21);
    let (fork_snap_id, fork_exec_id) =
        persist_replay_rows(dir.path(), &mut store, fork.prev_root(), 3);
    stage_contract(&mut store, &fork, fork_snap_id, fork_exec_id, 3);
    let err = store
        .seal_artifact(
            &fork,
            proof(&fork, 7, fork_snap_id, fork_exec_id),
            fork_snap_id,
            fork_exec_id,
        )
        .expect_err("forked successor must reject");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_second_origin_link_rejects() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let first = draft(1);
    let second = draft(11);
    let (first_snap_id, first_exec_id) =
        persist_replay_rows(dir.path(), &mut store, first.prev_root(), 1);
    stage_contract(&mut store, &first, first_snap_id, first_exec_id, 1);
    store
        .seal_artifact(
            &first,
            proof(&first, 9, first_snap_id, first_exec_id),
            first_snap_id,
            first_exec_id,
        )
        .expect("seal first origin artifact");

    let (second_snap_id, second_exec_id) =
        persist_replay_rows(dir.path(), &mut store, second.prev_root(), 2);
    stage_contract(&mut store, &second, second_snap_id, second_exec_id, 2);
    let err = store
        .seal_artifact(
            &second,
            proof(&second, 8, second_snap_id, second_exec_id),
            second_snap_id,
            second_exec_id,
        )
        .expect_err("second origin link must reject");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_conflicting_checkpoint_link_rejects() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = draft(1);
    let (snap_id, exec_id) = persist_replay_rows(dir.path(), &mut store, draft.prev_root(), 1);
    stage_contract(&mut store, &draft, snap_id, exec_id, 1);
    let first = store
        .seal_artifact(&draft, proof(&draft, 9, snap_id, exec_id), snap_id, exec_id)
        .expect("seal artifact");
    let bad = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        first.checkpoint_id(),
        PrepSnapshotId::new([8u8; 32]),
        exec_id,
    )
    .expect("bad link");

    let err = store
        .save_link(&bad)
        .expect_err("conflicting link must reject");

    assert!(matches!(
        err,
        CheckpointError::LinkMix | CheckpointError::Codec(_)
    ));
}

#[test]
fn test_conflicting_exec_reuse_rejects() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let first = draft(1);
    let second = draft(11);
    let (snap_id, exec_id) = persist_replay_rows(dir.path(), &mut store, first.prev_root(), 1);
    stage_contract(&mut store, &first, snap_id, exec_id, 1);

    store
        .seal_artifact(&first, proof(&first, 9, snap_id, exec_id), snap_id, exec_id)
        .expect("first link");
    stage_contract(&mut store, &second, snap_id, exec_id, 1);
    let err = store
        .seal_artifact(
            &second,
            proof(&second, 8, snap_id, exec_id),
            snap_id,
            exec_id,
        )
        .expect_err("exec reuse must reject");

    assert!(matches!(
        err,
        CheckpointError::LinkMix | CheckpointError::Codec(_)
    ));
}

#[test]
fn test_tampered_tuple_rejects_decode() {
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([1u8; 32]),
        PrepSnapshotId::new([2u8; 32]),
        CheckpointExecInputId::new([3u8; 32]),
    )
    .expect("link");
    let tampered = LinkWire {
        version: link.version(),
        checkpoint_id: link.checkpoint_id(),
        prev_checkpoint_id: link.prev_checkpoint_id(),
        prep_snapshot_id: PrepSnapshotId::new([9u8; 32]),
        exec_input_id: link.exec_input_id(),
        link_bind_ver: link.link_bind_ver(),
        link_bind: link.link_bind(),
    };
    let bytes = BincodeCodec
        .serialize(&tampered)
        .expect("encode tampered link");

    let err = decode_link_bin(&bytes).expect_err("tampered link must reject");
    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_unbound_link_reject_decode() {
    let unbound = UnboundLinkWire {
        version: CheckpointLinkVersion::CURRENT,
        checkpoint_id: z00z_storage::checkpoint::CheckpointId::new([1u8; 32]),
        prep_snapshot_id: PrepSnapshotId::new([2u8; 32]),
        exec_input_id: CheckpointExecInputId::new([3u8; 32]),
    };
    let bytes = BincodeCodec
        .serialize(&unbound)
        .expect("encode unbound link");
    let err = decode_link_bin(&bytes).expect_err("unbound link bytes must reject decode");

    assert!(matches!(
        err,
        CheckpointError::LinkMix | CheckpointError::Codec(_)
    ));
}

#[test]
fn test_audit_keeps_id_link() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = draft(31);
    let (snap_id, exec_id) = persist_replay_rows(dir.path(), &mut store, draft.prev_root(), 1);
    stage_contract(&mut store, &draft, snap_id, exec_id, 1);
    let link = store
        .seal_artifact(&draft, proof(&draft, 9, snap_id, exec_id), snap_id, exec_id)
        .expect("seal artifact");
    let before = store.load_link(&link.checkpoint_id()).expect("link before");
    let audit = CheckpointAudit::new(
        CheckpointAuditVersion::CURRENT,
        link.checkpoint_id(),
        vec![String::from("frag-1")],
    )
    .expect("audit");

    store.save_audit(&audit).expect("save audit");

    let art = store
        .load_artifact(&link.checkpoint_id())
        .expect("load artifact");
    let after = store.load_link(&link.checkpoint_id()).expect("link after");
    let audit_got = store.load_audit(&link.checkpoint_id()).expect("load audit");

    assert_eq!(
        derive_checkpoint_id(&art).expect("artifact id"),
        link.checkpoint_id()
    );
    assert_eq!(before, after);
    assert_eq!(audit_got, audit);
}
