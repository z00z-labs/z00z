use z00z_utils::codec::{BincodeCodec, Codec};

use super::{
    helpers, state::StateMeta, CHECK_TABLE, DRAFT_TABLE, EXEC_TABLE, LINK_TABLE, SNAP_TABLE,
};
use crate::{
    backend::error::StoreBackendError, checkpoint::CheckpointStatement, snapshot::PrepSnapshotId,
};

pub(super) fn validate_checkpoint_meta(
    read: &redb::ReadTransaction,
    meta: &StateMeta,
) -> Result<(), StoreBackendError> {
    // The raw checkpoint artifact surface is weaker because checkpoint identity
    // does not bind proof payload bytes.
    // The persisted RedB path is the live backend-defined acceptance contract:
    // it performs fail-closed revalidation over proof-system typing, statement
    // shape, exec identity, the persisted snapshot or link tuple, and bound
    // root or payload invariants before reload metadata is accepted.
    // Package-coupled checkpoint integrity exists here without claiming a
    // generic standalone proof backend or finished trustless publish theorem.
    // Compatibility-looking proof bytes remain non-authoritative fallback
    // inputs rather than the live closure story. This is only the
    // package-coupled checkpoint leg of the broader replay/stale story;
    // storage-backed claim continuity and current-stack spend boundaries remain
    // separate closure surfaces.
    if meta.draft_id == [0u8; 32] && meta.check_id == [0u8; 32] && meta.exec_id == [0u8; 32] {
        return Ok(());
    }

    let exec_table = read
        .open_table(EXEC_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let exec_bytes = exec_table
        .get(&meta.exec_id[..])
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        .ok_or_else(|| {
            StoreBackendError::Tx("missing canonical exec row for checkpoint metadata".to_owned())
        })?;
    let exec =
        crate::checkpoint::decode_exec_bin(exec_bytes.value()).map_err(helpers::map_check)?;

    let snap_table = read
        .open_table(SNAP_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let snap_bytes = snap_table
        .get(&meta.snap_id[..])
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        .ok_or_else(|| {
            StoreBackendError::Tx(
                "missing canonical snapshot row for checkpoint metadata".to_owned(),
            )
        })?;
    let snapshot_id = sha256_row_hash(snap_bytes.value());
    if snapshot_id != meta.snap_id {
        return Err(StoreBackendError::Tx(
            "checkpoint snapshot row does not match persisted snapshot id".to_owned(),
        ));
    }
    let snapshot: crate::snapshot::PrepSnapshot = BincodeCodec.deserialize(snap_bytes.value())?;
    if snapshot.version != crate::snapshot::PrepSnapshotVersion::CURRENT {
        return Err(StoreBackendError::Tx(
            "checkpoint snapshot row has unsupported version".to_owned(),
        ));
    }

    let draft_table = read
        .open_table(DRAFT_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let draft_bytes = draft_table
        .get(&meta.draft_id[..])
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        .ok_or_else(|| {
            StoreBackendError::Tx("missing canonical draft row for checkpoint metadata".to_owned())
        })?;
    let draft =
        crate::checkpoint::decode_draft_bin(draft_bytes.value()).map_err(helpers::map_check)?;

    let check_table = read
        .open_table(CHECK_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let check_bytes = check_table
        .get(&meta.check_id[..])
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        .ok_or_else(|| {
            StoreBackendError::Tx(
                "missing canonical checkpoint row for checkpoint metadata".to_owned(),
            )
        })?;
    let checkpoint: crate::checkpoint::CheckpointArtifact =
        BincodeCodec.deserialize(check_bytes.value())?;

    let link_table = read
        .open_table(LINK_TABLE)
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?;
    let link_bytes = link_table
        .get(&meta.check_id[..])
        .map_err(|err| StoreBackendError::Tx(err.to_string()))?
        .ok_or_else(|| {
            StoreBackendError::Tx(
                "missing canonical checkpoint link row for checkpoint metadata".to_owned(),
            )
        })?;
    let link =
        crate::checkpoint::decode_link_bin(link_bytes.value()).map_err(helpers::map_check)?;
    if !matches!(
        checkpoint.statement(),
        crate::checkpoint::CheckpointStatement::V1(_)
    ) {
        return Err(StoreBackendError::Tx(
            "checkpoint artifacts missing statement ids cannot carry persisted link metadata"
                .to_owned(),
        ));
    }

    let new_exec_id = crate::checkpoint::derive_exec_id(exec_bytes.value()).into_bytes();
    let new_draft_id = crate::checkpoint::derive_draft_id(&draft)
        .map_err(helpers::map_check)?
        .into_bytes();
    let new_check_id = crate::checkpoint::derive_checkpoint_id(&checkpoint)
        .map_err(helpers::map_check)?
        .into_bytes();
    let is_current_id_set = new_exec_id == meta.exec_id
        && new_draft_id == meta.draft_id
        && new_check_id == meta.check_id;
    if !is_current_id_set {
        return Err(StoreBackendError::Tx(
            "checkpoint metadata ids do not match current checkpoint artifacts".to_owned(),
        ));
    }

    if link.checkpoint_id().into_bytes() != meta.check_id
        || link.prep_snapshot_id().into_bytes() != meta.snap_id
        || link.exec_input_id().into_bytes() != meta.exec_id
    {
        return Err(StoreBackendError::Tx(
            "checkpoint link tuple does not match persisted checkpoint metadata".to_owned(),
        ));
    }
    if exec.prep_snapshot_id().into_bytes() != meta.snap_id {
        return Err(StoreBackendError::Tx(
            "checkpoint exec artifact does not match persisted snapshot id".to_owned(),
        ));
    }
    let expect_stmt = draft.attest_stmt(
        PrepSnapshotId::new(meta.snap_id),
        crate::checkpoint::CheckpointExecInputId::new(meta.exec_id),
    );
    match checkpoint.statement() {
        CheckpointStatement::V1(stmt) => {
            if *stmt != expect_stmt {
                return Err(StoreBackendError::Tx(
                    "checkpoint artifact statement does not match persisted draft boundary"
                        .to_owned(),
                ));
            }

            if snapshot.prev_root != exec.prev_root()
                || draft.prev_root() != exec.prev_root()
                || checkpoint.prev_root() != exec.prev_root()
            {
                return Err(StoreBackendError::Tx(
                    "checkpoint prior root metadata does not match persisted artifacts".to_owned(),
                ));
            }
            if draft.new_root().as_bytes() != &meta.state_root
                || checkpoint.new_root().as_bytes() != &meta.state_root
            {
                return Err(StoreBackendError::Tx(
                    "checkpoint next root metadata does not match persisted artifacts".to_owned(),
                ));
            }

            if checkpoint.cp_proof() != stmt.backend_payload().as_slice()
                || stmt.new_root().as_bytes() != &meta.state_root
                || stmt.prev_root() != exec.prev_root()
                || stmt.exec_input_id().into_bytes() != meta.exec_id
            {
                return Err(StoreBackendError::Tx(
                    "checkpoint proof bytes do not match persisted exec id and state root"
                        .to_owned(),
                ));
            }
        }
        CheckpointStatement::Detached => {
            return Err(StoreBackendError::Tx(
                "checkpoint artifacts missing statement ids cannot carry persisted link metadata"
                    .to_owned(),
            ));
        }
    }

    Ok(())
}

fn sha256_row_hash(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    Sha256::digest(bytes).into()
}
