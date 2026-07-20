use crate::{
    checkpoint::audit::{CheckpointAudit, CheckpointAuditVersion},
    checkpoint::{
        ArchiveManifestVersion, CheckpointArchiveEncodingKindV1, CheckpointArchiveEntryKindV1,
        CheckpointArchiveEntryV1, CheckpointArchiveEntryVersion, CheckpointArchiveManifestV1,
        CheckpointArchiveRetentionClassV1, CheckpointArtifact, CheckpointDaLocatorKind,
        CheckpointDaProviderFamily, CheckpointDaReferenceV1, CheckpointDaReferenceVersion,
        CheckpointDraft, CheckpointExecInput, CheckpointExecInputId, CheckpointExecOut,
        CheckpointExecTx, CheckpointExecVersion, CheckpointId, CheckpointInRef, CheckpointLink,
        CheckpointLinkVersion, CheckpointProof, CheckpointTransitionStatementCoreV1,
        CheckpointTransitionStatementFinalV1, CheckpointTransitionStatementV1, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::{CheckRoot, DefinitionId, SerialId, SettlementStore, TerminalLeaf},
    snapshot::PrepSnapshotId,
};
use serde::Serialize;
use z00z_core::assets::AssetLeaf;
use z00z_utils::codec::{Codec, JsonCodec};

#[derive(Serialize)]
struct PriorMadeEnt {
    asset_id_hex: String,
    leaf_hash_hex: String,
}

#[derive(Serialize)]
struct PriorStage6 {
    prev_root_hex: String,
    new_root_hex: String,
    spent_delta: Vec<String>,
    created_delta: Vec<PriorMadeEnt>,
    fragment_ids: Vec<String>,
}

pub fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        41,
        empty_check_root(),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    )
}

fn empty_check_root() -> CheckRoot {
    CheckRoot::from(
        SettlementStore::new()
            .settlement_root()
            .expect("empty settlement root"),
    )
}

pub fn proof(draft: &CheckpointDraft, _byte: u8) -> CheckpointProof {
    draft
        .attest_proof(
            PrepSnapshotId::new([7u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
        .expect("proof")
}

pub fn artifact() -> CheckpointArtifact {
    let draft = draft();
    let exec = CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        PrepSnapshotId::new([7u8; 32]),
        draft.prev_root(),
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([2u8; 32], SerialId::new(7))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([3u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(7)),
            )
            .expect("exec out")],
            vec![3u8],
        )
        .expect("exec tx")],
    )
    .expect("artifact exec");
    canonical_artifact(
        &draft,
        &exec,
        CheckpointExecInputId::new([8u8; 32]),
        proof(&draft, 9),
    )
}

pub fn exec() -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        PrepSnapshotId::new([9u8; 32]),
        empty_check_root(),
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([2u8; 32], SerialId::new(7))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([3u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(7)),
            )
            .expect("exec out")],
            vec![3u8],
        )
        .expect("exec tx")],
    )
    .expect("exec")
}

pub fn statement_core(exec: &CheckpointExecInput) -> CheckpointTransitionStatementCoreV1 {
    CheckpointTransitionStatementCoreV1::from_exec(exec, [0x61; 32], [0x62; 32], [0x63; 32])
}

fn archive_entries() -> Vec<CheckpointArchiveEntryV1> {
    vec![
        CheckpointArchiveEntryV1::new(
            CheckpointArchiveEntryVersion::CURRENT,
            CheckpointArchiveEntryKindV1::RawTxPackage,
            0,
            [0x71; 32],
            512,
            CheckpointArchiveRetentionClassV1::ArchiveRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )
        .expect("entry 0"),
        CheckpointArchiveEntryV1::new(
            CheckpointArchiveEntryVersion::CURRENT,
            CheckpointArchiveEntryKindV1::ExactTxProofBytes,
            1,
            [0x72; 32],
            256,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )
        .expect("entry 1"),
        CheckpointArchiveEntryV1::new(
            CheckpointArchiveEntryVersion::CURRENT,
            CheckpointArchiveEntryKindV1::WitnessArchive,
            2,
            [0x73; 32],
            384,
            CheckpointArchiveRetentionClassV1::AuditRequired,
            CheckpointArchiveEncodingKindV1::CanonicalBinV1,
        )
        .expect("entry 2"),
        CheckpointArchiveEntryV1::new(
            CheckpointArchiveEntryVersion::CURRENT,
            CheckpointArchiveEntryKindV1::DeltaJournal,
            3,
            [0x74; 32],
            192,
            CheckpointArchiveRetentionClassV1::DisputeRequired,
            CheckpointArchiveEncodingKindV1::CanonicalJsonV1,
        )
        .expect("entry 3"),
    ]
}

pub fn archive_manifest(
    draft: &CheckpointDraft,
    exec: &CheckpointExecInput,
    exec_id: CheckpointExecInputId,
) -> CheckpointArchiveManifestV1 {
    let core = statement_core(exec);
    archive_manifest_with_core(draft, exec, exec_id, core)
}

pub fn archive_manifest_with_core(
    draft: &CheckpointDraft,
    exec: &CheckpointExecInput,
    exec_id: CheckpointExecInputId,
    core: CheckpointTransitionStatementCoreV1,
) -> CheckpointArchiveManifestV1 {
    let statement =
        CheckpointTransitionStatementV1::from_draft(draft, exec.prep_snapshot_id(), exec_id);
    let statement_core_digest = statement.statement_core_digest_v1(&core);
    CheckpointArchiveManifestV1::new(
        ArchiveManifestVersion::CURRENT,
        statement_core_digest,
        exec_id,
        exec.prep_snapshot_id(),
        core.tx_data_root(),
        core.delta_root(),
        core.witness_root(),
        core.journal_digest(),
        [0x11; 32],
        [0x12; 32],
        [0x13; 32],
        [0x14; 32],
        [0x15; 32],
        [0x16; 32],
        [0x17; 32],
        [0x18; 32],
        [0x19; 32],
        archive_entries(),
        3,
    )
    .expect("archive manifest")
}

pub fn da_reference(manifest: &CheckpointArchiveManifestV1) -> CheckpointDaReferenceV1 {
    CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::LocalArchive,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        "checkpoint-da://local_archive/fixture/1",
        manifest.da_payload_commitment(),
        manifest.statement_core_digest(),
        manifest.archive_manifest_root(),
        41,
    )
    .expect("da reference")
}

pub fn canonical_artifact(
    draft: &CheckpointDraft,
    exec: &CheckpointExecInput,
    exec_id: CheckpointExecInputId,
    proof: CheckpointProof,
) -> CheckpointArtifact {
    let manifest = archive_manifest(draft, exec, exec_id);
    let da_reference = da_reference(&manifest);
    let core = statement_core(exec);
    draft
        .finalize(proof)
        .expect("artifact")
        .bind_canonical_v1(
            core,
            CheckpointTransitionStatementFinalV1::new(da_reference.da_ref()),
        )
        .expect("canonical artifact")
}

pub fn link(checkpoint_id: CheckpointId, exec_id: CheckpointExecInputId) -> CheckpointLink {
    CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        PrepSnapshotId::new([7u8; 32]),
        exec_id,
    )
    .expect("link")
}

pub fn audit(checkpoint_id: CheckpointId) -> CheckpointAudit {
    CheckpointAudit::new(
        CheckpointAuditVersion::CURRENT,
        checkpoint_id,
        vec![String::from("frag-1")],
    )
    .expect("audit")
}

pub fn prior_stage6_json() -> Vec<u8> {
    JsonCodec
        .serialize(&PriorStage6 {
            prev_root_hex: "11".repeat(32),
            new_root_hex: "22".repeat(32),
            spent_delta: vec!["33".repeat(32)],
            created_delta: vec![PriorMadeEnt {
                asset_id_hex: "44".repeat(32),
                leaf_hash_hex: "55".repeat(32),
            }],
            fragment_ids: vec![String::from("frag_1"), String::from("frag_2")],
        })
        .expect("prior stage6 json")
}
