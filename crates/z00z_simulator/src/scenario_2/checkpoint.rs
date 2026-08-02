use std::path::Path;

use sha2::{Digest, Sha256};
use z00z_storage::{
    checkpoint::{
        encode_exec_bin, ArchiveManifestVersion, CheckpointArchiveEncodingKindV1,
        CheckpointArchiveEntryKindV1, CheckpointArchiveEntryV1, CheckpointArchiveEntryVersion,
        CheckpointArchiveManifestV1, CheckpointArchiveRetentionClassV1, CheckpointDaLocatorKind,
        CheckpointDaProviderFamily, CheckpointDaReferenceV1, CheckpointDaReferenceVersion,
        CheckpointDraft, CheckpointExecInput, CheckpointExecInputId, CheckpointExecVersion,
        CheckpointFsStore, CheckpointId, CheckpointStore, CheckpointTransitionStatementCoreV1,
        CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{SettlementExecHandoff, SettlementPath, SettlementStateRoot, SettlementStore},
    snapshot::{build_snapshot_v2, PrepFsStore, PrepSnapshotId, PrepSnapshotStore},
};
use z00z_utils::codec::{Codec, JsonCodec};

use super::{da::DaCommit, runner::Scenario2Err};

const DELTA_LABEL: &[u8] = b"z00z.simulator.scenario-2.delta-root.v1";
const WITNESS_LABEL: &[u8] = b"z00z.simulator.scenario-2.witness-root.v1";
const JOURNAL_LABEL: &[u8] = b"z00z.simulator.scenario-2.journal-root.v1";

pub(super) struct Projection {
    pub post_root: SettlementStateRoot,
    pub created: Vec<CreatedEnt>,
}

pub(super) struct SealedCheckpoint {
    pub checkpoint_id: CheckpointId,
    pub snapshot_id: PrepSnapshotId,
    pub exec_id: CheckpointExecInputId,
    pub archive_manifest_root: [u8; 32],
}

pub(super) fn project_block(
    preview: &mut SettlementStore,
    handoff: &SettlementExecHandoff,
    output_paths: &[SettlementPath],
    layout: u32,
) -> Result<Projection, Scenario2Err> {
    preview
        .apply_exec_handoff(handoff.clone())
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let post_root = preview
        .settlement_root_v2(layout)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let proofs = preview
        .settlement_proof_blobs(output_paths)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    if proofs.len() != output_paths.len() {
        return Err(Scenario2Err::Invariant(
            "post-state output proof cardinality drift".to_string(),
        ));
    }
    let created = output_paths
        .iter()
        .zip(proofs)
        .map(|(path, proof)| CreatedEnt::new(path.terminal_id(), proof.terminal_leaf_hash()))
        .collect();
    Ok(Projection { post_root, created })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn seal_checkpoint(
    height: u64,
    pre_root: SettlementStateRoot,
    projection: &Projection,
    prior_recursive_digest: [u8; 32],
    handoff: &SettlementExecHandoff,
    spent_paths: &[SettlementPath],
    da: &DaCommit,
    checkpoint_store: &mut CheckpointFsStore,
    prep_store: &mut PrepFsStore,
) -> Result<SealedCheckpoint, Scenario2Err> {
    let (snapshot, snapshot_id) = build_snapshot_v2(pre_root, Vec::new())
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let saved_snapshot = prep_store
        .save_snapshot(&snapshot)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    if saved_snapshot != snapshot_id {
        return Err(Scenario2Err::Invariant(
            "prep snapshot id changed on save".to_string(),
        ));
    }
    let exec = CheckpointExecInput::new_settlement(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        pre_root,
        handoff.txs().to_vec(),
    )
    .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let exec_id = checkpoint_store
        .save_exec_input(&exec)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let spent = spent_paths
        .iter()
        .map(|path| SpentEnt::new(path.terminal_id()))
        .collect();
    let draft = CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        height,
        pre_root,
        projection.post_root,
        spent,
        projection.created.clone(),
    );
    let draft_id = checkpoint_store
        .save_draft(&draft)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let handoff_bytes = JsonCodec
        .serialize(handoff)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let exec_bytes =
        encode_exec_bin(&exec).map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let delta_root = hash_parts(DELTA_LABEL, &[&handoff_bytes, pre_root.as_bytes()]);
    let witness_root = hash_parts(WITNESS_LABEL, &[&exec_bytes, &da.exact_proof_root]);
    let journal_digest = hash_parts(
        JOURNAL_LABEL,
        &[
            &height.to_le_bytes(),
            &handoff.route().batch_id(),
            &handoff.route().route_table_digest(),
        ],
    );
    let statement_core = CheckpointTransitionStatementCoreV1::from_exec(
        &exec,
        delta_root,
        witness_root,
        journal_digest,
    )
    .with_prior_recursive_output_root(prior_recursive_digest);
    let statement_core_digest =
        CheckpointTransitionStatementV1::from_draft(&draft, snapshot_id, exec_id)
            .statement_core_digest_v1(&statement_core);
    let manifest = build_manifest(
        &exec,
        exec_id,
        statement_core_digest,
        statement_core,
        da,
        &exec_bytes,
        &handoff_bytes,
    )?;
    let da_reference = CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::NamespaceBlob,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        da.path.to_string_lossy().into_owned(),
        da.payload_commitment,
        statement_core_digest,
        manifest.archive_manifest_root(),
        height,
    )
    .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    checkpoint_store
        .stage_publication_contract(exec_id, &statement_core, &manifest, &da_reference)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let link = checkpoint_store
        .seal_artifact(
            &draft,
            draft
                .attest_proof(snapshot_id, exec_id)
                .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?,
            snapshot_id,
            exec_id,
        )
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    let checkpoint_id = link.checkpoint_id();

    if checkpoint_store
        .load_draft(&draft_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?
        != draft
        || checkpoint_store
            .load_exec_input(&exec_id)
            .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?
            != exec
        || prep_store
            .load_snapshot(&snapshot_id)
            .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?
            != snapshot
        || checkpoint_store
            .load_link(&checkpoint_id)
            .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?
            != link
        || checkpoint_store
            .load_archive_manifest(&checkpoint_id)
            .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?
            != manifest
        || checkpoint_store
            .load_da_reference(&checkpoint_id)
            .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?
            != da_reference
    {
        return Err(Scenario2Err::Invariant(
            "checkpoint persistence reload mismatch".to_string(),
        ));
    }
    checkpoint_store
        .load_artifact(&checkpoint_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    Ok(SealedCheckpoint {
        checkpoint_id,
        snapshot_id,
        exec_id,
        archive_manifest_root: manifest.archive_manifest_root(),
    })
}

#[allow(clippy::too_many_arguments)]
fn build_manifest(
    exec: &CheckpointExecInput,
    exec_id: CheckpointExecInputId,
    statement_core_digest: [u8; 32],
    statement_core: CheckpointTransitionStatementCoreV1,
    da: &DaCommit,
    exec_bytes: &[u8],
    handoff_bytes: &[u8],
) -> Result<CheckpointArchiveManifestV1, Scenario2Err> {
    let epoch_manifest_root = hash_parts(
        b"z00z.simulator.scenario-2.epoch-manifest.v1",
        &[&da.payload_commitment, exec_id.as_bytes()],
    );
    let witness_archive_root = hash_parts(
        b"z00z.simulator.scenario-2.witness-archive.v1",
        &[exec_bytes, &statement_core.witness_root()],
    );
    let delta_journal_root = hash_parts(
        b"z00z.simulator.scenario-2.delta-journal.v1",
        &[handoff_bytes, &statement_core.delta_root()],
    );
    let archive_provider_receipt_root = hash_parts(
        b"z00z.simulator.scenario-2.archive-receipt.v1",
        &[&da.payload_commitment, &da.artifact_bytes.to_le_bytes()],
    );
    let retrieval_audit_root = hash_parts(
        b"z00z.simulator.scenario-2.retrieval-audit.v1",
        &[&da.raw_tx_root, &da.package_count.to_le_bytes()],
    );
    let content_address_root = hash_parts(
        b"z00z.simulator.scenario-2.content-address.v1",
        &[&da.payload_commitment, &da.raw_tx_root, exec_bytes],
    );
    let entries = vec![
        archive_entry(
            CheckpointArchiveEntryKindV1::RawTxPackage,
            0,
            da.raw_tx_root,
            da.payload_bytes,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ExactTxProofBytes,
            1,
            da.exact_proof_root,
            da.proof_bytes,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::WitnessArchive,
            2,
            witness_archive_root,
            bytes_len(exec_bytes, "checkpoint exec")?,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::DeltaJournal,
            3,
            delta_journal_root,
            bytes_len(handoff_bytes, "checkpoint handoff")?,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ArchiveProviderReceipt,
            4,
            archive_provider_receipt_root,
            32,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::RetrievalAudit,
            5,
            retrieval_audit_root,
            32,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
        archive_entry(
            CheckpointArchiveEntryKindV1::ContentAddressIndex,
            6,
            content_address_root,
            32,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::ProviderPayloadV1,
        )?,
    ];
    CheckpointArchiveManifestV1::new(
        ArchiveManifestVersion::CURRENT,
        statement_core_digest,
        exec_id,
        exec.prep_snapshot_id(),
        statement_core.tx_data_root(),
        statement_core.delta_root(),
        statement_core.witness_root(),
        statement_core.journal_digest(),
        epoch_manifest_root,
        da.raw_tx_root,
        da.exact_proof_root,
        witness_archive_root,
        delta_journal_root,
        da.payload_commitment,
        archive_provider_receipt_root,
        retrieval_audit_root,
        content_address_root,
        entries,
        3,
    )
    .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))
}

fn archive_entry(
    kind: CheckpointArchiveEntryKindV1,
    ordinal: u32,
    digest: [u8; 32],
    bytes: u64,
    retention: CheckpointArchiveRetentionClassV1,
    encoding: CheckpointArchiveEncodingKindV1,
) -> Result<CheckpointArchiveEntryV1, Scenario2Err> {
    CheckpointArchiveEntryV1::new(
        CheckpointArchiveEntryVersion::CURRENT,
        kind,
        ordinal,
        digest,
        bytes,
        retention,
        encoding,
    )
    .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))
}

pub(super) fn verify_reopened(root: &Path, sealed: &SealedCheckpoint) -> Result<(), Scenario2Err> {
    let checkpoint_store = CheckpointFsStore::new(root);
    let prep_store = PrepFsStore::new(root);
    checkpoint_store
        .load_artifact(&sealed.checkpoint_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    checkpoint_store
        .load_exec_input(&sealed.exec_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    checkpoint_store
        .load_archive_manifest(&sealed.checkpoint_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    checkpoint_store
        .load_da_reference(&sealed.checkpoint_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    prep_store
        .load_snapshot(&sealed.snapshot_id)
        .map_err(|error| Scenario2Err::Checkpoint(error.to_string()))?;
    Ok(())
}

fn bytes_len(bytes: &[u8], label: &str) -> Result<u64, Scenario2Err> {
    u64::try_from(bytes.len())
        .map_err(|_| Scenario2Err::Checkpoint(format!("{label} length overflow")))
}

fn hash_parts(label: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update((label.len() as u64).to_le_bytes());
    hasher.update(label);
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}

pub(super) fn checkpoint_root(run_dir: &Path) -> std::path::PathBuf {
    run_dir.join("checkpoint")
}
