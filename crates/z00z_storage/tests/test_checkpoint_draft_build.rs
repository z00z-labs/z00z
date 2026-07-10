use std::cell::RefCell;

use z00z_storage::fixture_support::snapshot_fix;

use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        build_cp_draft, derive_exec_id, encode_draft_bin, encode_exec_bin, CheckpointExecInput,
        CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion, CheckpointInRef,
        CheckpointLink, CheckpointLinkVersion, CheckpointTransitionStatementCoreV1, SpentIndex,
        SpentIndexError, StateError, TxPkgSum, TxProofError, TxProofVerifier,
    },
    settlement::{DefinitionId, SerialId, TerminalLeaf},
    snapshot::PrepSnapshotStore,
};

struct NoSpent;
impl SpentIndex for NoSpent {
    fn is_spent(
        &self,
        _prev: z00z_storage::settlement::CheckRoot,
        _curr: z00z_storage::settlement::CheckRoot,
        _id: &z00z_storage::settlement::TerminalId,
    ) -> Result<bool, SpentIndexError> {
        Ok(false)
    }
}

struct PassProof;
impl TxProofVerifier for PassProof {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError> {
        if tx.tx_proof.is_empty() {
            return Err(TxProofError::Invalid);
        }
        Ok(())
    }
}

struct CaptureProof {
    seen: RefCell<Vec<Vec<u8>>>,
}

impl CaptureProof {
    fn new() -> Self {
        Self {
            seen: RefCell::new(Vec::new()),
        }
    }
}

impl TxProofVerifier for CaptureProof {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError> {
        self.seen.borrow_mut().push(tx.tx_proof.clone());
        Ok(())
    }
}

fn exec_with_proof(
    snapshot_id: z00z_storage::snapshot::PrepSnapshotId,
    prev_root: z00z_storage::settlement::CheckRoot,
    tx_proof: Vec<u8>,
) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([7u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(11)),
            )
            .expect("exec out")],
            tx_proof,
        )
        .expect("exec tx")],
    )
    .expect("exec input")
}

fn exec(
    snapshot_id: z00z_storage::snapshot::PrepSnapshotId,
    prev_root: z00z_storage::settlement::CheckRoot,
) -> CheckpointExecInput {
    exec_with_proof(snapshot_id, prev_root, vec![9u8, 7u8, 5u8])
}

#[test]
fn test_pass_draft_is_deterministic() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let exec = exec(snap_id, loaded.prev_root);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let first = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("first draft");
    let second = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect("second draft");

    assert_eq!(
        encode_draft_bin(&first).expect("bytes"),
        encode_draft_bin(&second).expect("bytes")
    );
}

#[test]
fn test_dup_out_rejects() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let leaf = TerminalLeaf::from(AssetLeaf::dummy_for_scan(11));
    let exec = CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snap_id,
        loaded.prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([1u8; 32], SerialId::new(1))],
            vec![
                CheckpointExecOut::new(DefinitionId::new([7u8; 32]), leaf.clone()).expect("out a"),
                CheckpointExecOut::new(DefinitionId::new([7u8; 32]), leaf).expect("out b"),
            ],
            vec![9u8],
        )
        .expect("exec tx")],
    )
    .expect("exec input");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");

    let err = build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &PassProof, &NoSpent,
    )
    .expect_err("dup out must reject");

    assert!(matches!(err, StateError::DupOut));
}

#[test]
fn test_build_proof_bytes_verifier() {
    let snapshot = snapshot_fix::snap(&[(1, 1, 1)]);
    let (_dir, store, snap_id) = snapshot_fix::save(&snapshot);
    let loaded = store.load_snapshot(&snap_id).expect("snapshot");
    let replay = store.replay_entries(&loaded).expect("replay");
    let want_proof = b"verified-proof-v1".to_vec();
    let exec = exec_with_proof(snap_id, loaded.prev_root, want_proof.clone());
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        z00z_storage::checkpoint::CheckpointId::new([4u8; 32]),
        snap_id,
        derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes")),
    )
    .expect("link");
    let capture = CaptureProof::new();

    build_cp_draft(
        7, snap_id, &loaded, &replay, &link, &exec, &capture, &NoSpent,
    )
    .expect("draft");

    assert_eq!(capture.seen.into_inner(), vec![want_proof]);
}

#[test]
fn test_statement_core_uses_exec_root() {
    let exec = exec_with_proof(
        z00z_storage::snapshot::PrepSnapshotId::new([2u8; 32]),
        z00z_storage::settlement::CheckRoot::new([3u8; 32]),
        b"verified-proof-v1".to_vec(),
    );

    let core =
        CheckpointTransitionStatementCoreV1::from_exec(&exec, [0x11; 32], [0x22; 32], [0x33; 32]);

    assert_eq!(core.tx_data_root(), exec.tx_data_root());
}
