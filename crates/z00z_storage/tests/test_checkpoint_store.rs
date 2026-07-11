use z00z_storage::fixture_support::checkpoint_fixtures;

use tempfile::TempDir;
use z00z_core::assets::AssetLeaf;
use z00z_storage::{
    checkpoint::{
        decode_link_bin, derive_exec_id, encode_art_bin, encode_exec_bin,
        encode_recursive_sidecar_bin, load_artifact, repo_default_path, ArchiveManifestVersion,
        CheckpointArchiveManifestV1, CheckpointContractConfigV1, CheckpointExecInput,
        CheckpointExecInputId, CheckpointExecOut, CheckpointExecTx, CheckpointExecVersion,
        CheckpointFsStore, CheckpointId, CheckpointInRef, CheckpointLink, CheckpointLinkVersion,
        CheckpointProof, CheckpointStore, CheckpointTransitionStatementFinalV1,
        RecursiveCheckpointMeasurementV1, RecursiveCheckpointModeV1,
        RecursiveCheckpointProofFamilyV1, RecursiveCheckpointProofV1,
        RecursiveCheckpointPublicInputV1, RecursiveCheckpointSidecarV1,
        RecursiveCheckpointVerifierV1,
    },
    settlement::{CheckRoot, DefinitionId, SerialId, TerminalLeaf},
    snapshot::{build_snapshot, PrepFsStore, PrepSnapshotId, PrepSnapshotStore},
    CheckpointError,
};
use z00z_utils::{
    codec::{BincodeCodec, Codec},
    io::{create_dir_all, write_file},
};

const CHECKPOINT_STORE_SRC: &str = include_str!("../src/checkpoint/store.rs");
const STAGE4_STORAGE_VIEW_SRC: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_4/storage_view.rs");
const STAGE12_FINALIZE_SRC: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_12/finalize_flow.rs");

fn attest_proof(
    draft: &z00z_storage::checkpoint::CheckpointDraft,
    snap_id: PrepSnapshotId,
    exec_id: CheckpointExecInputId,
) -> CheckpointProof {
    draft.attest_proof(snap_id, exec_id).expect("proof")
}

fn temp_dir() -> TempDir {
    TempDir::new().expect("temp dir")
}

fn hex_id(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn exec(snapshot_id: PrepSnapshotId, prev_root: CheckRoot) -> CheckpointExecInput {
    CheckpointExecInput::new(
        CheckpointExecVersion::CURRENT,
        snapshot_id,
        prev_root,
        vec![CheckpointExecTx::new(
            vec![CheckpointInRef::new([0x31u8; 32], SerialId::new(1))],
            vec![CheckpointExecOut::new(
                DefinitionId::new([0x32u8; 32]),
                TerminalLeaf::from(AssetLeaf::dummy_for_scan(13)),
            )
            .expect("exec out")],
            vec![0x33u8],
        )
        .expect("exec tx")],
    )
    .expect("exec")
}

fn save_snapshot(root: &std::path::Path, prev_root: CheckRoot) -> PrepSnapshotId {
    let (snapshot, snapshot_id) = build_snapshot(prev_root, Vec::new()).expect("snapshot");
    let mut store = PrepFsStore::new(root);
    let saved_id = store.save_snapshot(&snapshot).expect("save snapshot");
    assert_eq!(saved_id, snapshot_id);
    saved_id
}

fn save_artifact_file(
    root: &std::path::Path,
    artifact: &z00z_storage::checkpoint::CheckpointArtifact,
) -> CheckpointId {
    let store = CheckpointFsStore::new(root);
    let checkpoint_id =
        z00z_storage::checkpoint::derive_checkpoint_id(artifact).expect("derive checkpoint id");
    let path = store
        .artifact_dir()
        .join(format!("{}.bin", hex_id(checkpoint_id.as_bytes())));
    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("mkdir");
    }
    write_file(&path, &encode_art_bin(artifact).expect("encode artifact")).expect("write artifact");
    checkpoint_id
}

fn stage_contract(
    store: &mut CheckpointFsStore,
    draft: &z00z_storage::checkpoint::CheckpointDraft,
    exec: &CheckpointExecInput,
    exec_id: CheckpointExecInputId,
) {
    let manifest = checkpoint_fixtures::archive_manifest(draft, exec, exec_id);
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    store
        .stage_publication_contract(exec_id, &manifest, &da_reference)
        .expect("stage publication contract");
}

fn with_statement_core(
    manifest: &CheckpointArchiveManifestV1,
    statement_core_digest: [u8; 32],
) -> CheckpointArchiveManifestV1 {
    CheckpointArchiveManifestV1::new(
        ArchiveManifestVersion::CURRENT,
        statement_core_digest,
        manifest.checkpoint_exec_input_id(),
        manifest.prep_snapshot_id(),
        manifest.tx_data_root(),
        manifest.delta_root(),
        manifest.witness_root(),
        manifest.journal_digest(),
        manifest.epoch_manifest_root(),
        manifest.raw_tx_package_root(),
        manifest.exact_tx_proof_bytes_root(),
        manifest.witness_archive_root(),
        manifest.delta_journal_root(),
        manifest.da_payload_commitment(),
        manifest.archive_provider_receipt_root(),
        manifest.retrieval_audit_root(),
        manifest.content_address_root(),
        manifest.entries().to_vec(),
        manifest.min_archive_replicas(),
    )
    .expect("manifest with foreign statement core")
}

fn recursive_sidecar(
    artifact: &z00z_storage::checkpoint::CheckpointArtifact,
    link: &CheckpointLink,
    proof: &CheckpointProof,
    statement_digest_override: Option<[u8; 32]>,
) -> RecursiveCheckpointSidecarV1 {
    let cfg = CheckpointContractConfigV1::load(repo_default_path()).expect("checkpoint config");
    let verifier = RecursiveCheckpointVerifierV1::new(&cfg).expect("recursive verifier");
    let mode = RecursiveCheckpointModeV1::FastClassicalCompressed;
    let expected_public_input = verifier
        .build_public_input(
            proof.statement(),
            &artifact.statement_core().expect("statement core"),
            &CheckpointTransitionStatementFinalV1::new(artifact.da_ref().expect("da ref")),
            link,
            mode,
            "nova_compressed_v1",
            0,
            3,
            [0x75; 32],
        )
        .expect("public input");
    let public_input = RecursiveCheckpointPublicInputV1::new(
        expected_public_input.mode(),
        expected_public_input.backend_label(),
        statement_digest_override.unwrap_or(expected_public_input.statement_digest()),
        expected_public_input.statement_core_digest(),
        expected_public_input.height(),
        expected_public_input.chain_index(),
        expected_public_input.chain_length(),
        expected_public_input.epoch_index(),
        expected_public_input.epoch_start_height(),
        expected_public_input.epoch_end_height(),
        expected_public_input.prev_root(),
        expected_public_input.output_root(),
        expected_public_input.prior_output_root(),
        expected_public_input.delta_root(),
        expected_public_input.witness_root(),
        expected_public_input.checkpoint_link_digest(),
        expected_public_input.verifier_params_digest(),
    )
    .expect("self-consistent sidecar input");
    let proof_bytes = vec![0x76; 96];
    let recursive_proof = RecursiveCheckpointProofV1::new(
        mode,
        "nova_compressed_v1",
        &public_input,
        proof_bytes.clone(),
    )
    .expect("recursive proof");
    let measurement = RecursiveCheckpointMeasurementV1::new(
        "nova_compressed_v1",
        mode,
        3,
        cfg.post_quantum.cadence_blocks,
        2,
        RecursiveCheckpointProofFamilyV1::Nova,
        0,
        proof_bytes.len() as u64,
        4096,
        17,
        9,
        8192,
        proof.statement().canonical_bytes_v1().len() as u64,
        public_input.canonical_bytes().len() as u64,
    )
    .expect("measurement");
    RecursiveCheckpointSidecarV1::accepted(
        public_input,
        Some(link.checkpoint_id()),
        recursive_proof,
        measurement,
    )
    .expect("sidecar")
}

#[test]
fn test_store_keeps_surfaces_separate() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft_val = checkpoint_fixtures::draft();
    let draft_id = store.save_draft(&draft_val).expect("save draft");
    let draft_got = store.load_draft(&draft_id).expect("load draft");
    let snap_id = save_snapshot(dir.path(), draft_val.prev_root());
    let exec = exec(snap_id, draft_val.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let exec_got = store.load_exec_input(&exec_id).expect("load exec");
    let proof = attest_proof(&draft_val, exec.prep_snapshot_id(), exec_id);
    stage_contract(&mut store, &draft_val, &exec, exec_id);
    let link = store
        .seal_artifact(&draft_val, proof.clone(), exec.prep_snapshot_id(), exec_id)
        .expect("seal artifact");
    let art = checkpoint_fixtures::canonical_artifact(&draft_val, &exec, exec_id, proof);
    let checkpoint_id = link.checkpoint_id();
    let art_got = store.load_artifact(&checkpoint_id).expect("load artifact");
    let link_got = store.load_link(&checkpoint_id).expect("load link");
    let audit = checkpoint_fixtures::audit(checkpoint_id);
    store.save_audit(&audit).expect("save audit");
    let audit_got = store.load_audit(&checkpoint_id).expect("load audit");

    assert_eq!(draft_got, checkpoint_fixtures::draft());
    assert_eq!(exec_got, exec);
    assert_eq!(art_got, art);
    assert_eq!(link_got, link);
    assert_eq!(audit_got, audit);
}

#[test]
fn test_seal_rejects_foreign_da_statement_core() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let manifest = with_statement_core(
        &checkpoint_fixtures::archive_manifest(&draft, &exec, exec_id),
        [0x42; 32],
    );
    let da_reference = checkpoint_fixtures::da_reference(&manifest);
    store
        .stage_publication_contract(exec_id, &manifest, &da_reference)
        .expect("stage mutually-bound foreign publication contract");

    let err = store
        .seal_artifact(
            &draft,
            attest_proof(&draft, snap_id, exec_id),
            snap_id,
            exec_id,
        )
        .expect_err("foreign DA statement core must reject during seal");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_store_persists_only_canonically_bound_recursive_sidecars() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    stage_contract(&mut store, &draft, &exec, exec_id);
    let link = store
        .seal_artifact(&draft, proof.clone(), snap_id, exec_id)
        .expect("seal artifact");
    let checkpoint_id = link.checkpoint_id();
    let artifact = store.load_artifact(&checkpoint_id).expect("artifact");
    let sidecar = recursive_sidecar(&artifact, &link, &proof, None);

    store
        .save_recursive_sidecar(checkpoint_id, &sidecar)
        .expect("save sidecar");
    assert_eq!(
        store
            .load_recursive_sidecar(&checkpoint_id)
            .expect("load sidecar"),
        sidecar
    );

    let tampered = recursive_sidecar(&artifact, &link, &proof, Some([0x77; 32]));
    let err = store
        .save_recursive_sidecar(checkpoint_id, &tampered)
        .expect_err("tampered sidecar must reject");
    assert!(matches!(err, CheckpointError::ArchiveMix));

    let path = store
        .recursive_sidecar_dir()
        .join(format!("{}.sidecar.bin", hex_id(checkpoint_id.as_bytes())));
    write_file(
        &path,
        &encode_recursive_sidecar_bin(&tampered).expect("tampered sidecar bytes"),
    )
    .expect("write tampered sidecar");
    let err = store
        .load_recursive_sidecar(&checkpoint_id)
        .expect_err("tampered stored sidecar must reject");
    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_seal_path_stays_canonical() {
    assert!(!CHECKPOINT_STORE_SRC.contains("fn save_artifact("));
    assert!(CHECKPOINT_STORE_SRC.contains("fn export_noncanonical_final_bundle("));
    assert!(CHECKPOINT_STORE_SRC.contains("self.check_link_evidence(&link)?;"));
    assert!(CHECKPOINT_STORE_SRC.contains("self.persist_artifact_bin(&artifact)?;"));
    assert!(STAGE12_FINALIZE_SRC.contains("build_attest_proof"));
    assert!(STAGE12_FINALIZE_SRC.contains(".seal_artifact("));
}

#[test]
fn test_stage4_raw_lane_local() {
    assert!(STAGE4_STORAGE_VIEW_SRC.contains("stay noncanonical"));
    assert!(STAGE4_STORAGE_VIEW_SRC.contains("export_noncanonical_final_bundle"));
    assert!(STAGE4_STORAGE_VIEW_SRC.contains("\"final_lane\": \"noncanonical_export\""));
    assert!(!STAGE4_STORAGE_VIEW_SRC.contains(".save_artifact(artifact)"));
}

#[test]
fn test_noncanonical_export_stays_explicit() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    let artifact = draft.finalize(proof).expect("artifact");
    let checkpoint_id =
        z00z_storage::checkpoint::derive_checkpoint_id(&artifact).expect("derive checkpoint id");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        snap_id,
        exec_id,
    )
    .expect("link");
    let audit = checkpoint_fixtures::audit(checkpoint_id);

    let exported_id = store
        .export_noncanonical_final_bundle(&artifact, &link, &audit)
        .expect("export noncanonical bundle");

    assert_eq!(exported_id, checkpoint_id);
    assert_eq!(
        store
            .load_noncanonical_artifact(&checkpoint_id)
            .expect("load exported artifact"),
        artifact
    );
    assert_eq!(
        store
            .load_noncanonical_link(&checkpoint_id)
            .expect("load exported link"),
        link
    );
    assert_eq!(
        store
            .load_noncanonical_audit(&checkpoint_id)
            .expect("load exported audit"),
        audit
    );
}

#[test]
fn test_canonical_load_rejects_noncanonical_export_lane() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    let artifact = draft.finalize(proof).expect("artifact");
    let checkpoint_id =
        z00z_storage::checkpoint::derive_checkpoint_id(&artifact).expect("derive checkpoint id");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        snap_id,
        exec_id,
    )
    .expect("link");
    let audit = checkpoint_fixtures::audit(checkpoint_id);
    store
        .export_noncanonical_final_bundle(&artifact, &link, &audit)
        .expect("export noncanonical bundle");

    let art_err = store
        .load_artifact(&checkpoint_id)
        .expect_err("canonical artifact load must reject noncanonical export");
    let link_err = store
        .load_link(&checkpoint_id)
        .expect_err("canonical link load must reject noncanonical export");
    let audit_err = store
        .load_audit(&checkpoint_id)
        .expect_err("canonical audit load must reject noncanonical export");

    assert!(matches!(art_err, CheckpointError::ArtifactCompatMix));
    assert!(matches!(link_err, CheckpointError::ArtifactCompatMix));
    assert!(matches!(audit_err, CheckpointError::ArtifactCompatMix));
}

#[test]
fn test_link_key_mismatch_rejects() {
    let dir = temp_dir();
    let store = CheckpointFsStore::new(dir.path());
    let checkpoint_id = CheckpointId::new([9u8; 32]);
    let wrong_id = CheckpointId::new([8u8; 32]);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        PrepSnapshotId::new([7u8; 32]),
        CheckpointExecInputId::new([6u8; 32]),
    )
    .expect("link");
    let bytes = BincodeCodec.serialize(&link).expect("encode link");
    let path = store
        .link_dir()
        .join(format!("{}.bin", hex_id(wrong_id.as_bytes())));
    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("mkdir");
    }
    write_file(&path, &bytes).expect("write link");

    let err = store
        .load_link(&wrong_id)
        .expect_err("wrong key must reject");

    assert!(matches!(err, CheckpointError::KeyMix));
}

#[test]
fn test_wrapper_shaped_payload_rejects() {
    let audit = checkpoint_fixtures::audit(CheckpointId::new([1u8; 32]));
    let bytes = BincodeCodec.serialize(&audit).expect("encode audit");
    let err = load_artifact(&bytes).expect_err("audit bytes must reject");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_incomplete_link_payload_rejects() {
    let err = decode_link_bin(&[1u8, 2, 3]).expect_err("bad link bytes");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_link_without_artifact_rejects() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let bad = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([1u8; 32]),
        PrepSnapshotId::new([7u8; 32]),
        CheckpointExecInputId::new([6u8; 32]),
    )
    .expect("link");

    let err = store
        .save_link(&bad)
        .expect_err("missing artifact must reject");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_artifact_keeps_codec_error() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let checkpoint_id = CheckpointId::new([4u8; 32]);
    let art_path = store
        .artifact_dir()
        .join(format!("{}.bin", hex_id(checkpoint_id.as_bytes())));
    if let Some(parent) = art_path.parent() {
        create_dir_all(parent).expect("mkdir");
    }
    write_file(&art_path, &[1u8, 2, 3]).expect("write bad artifact");
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        PrepSnapshotId::new([7u8; 32]),
        CheckpointExecInputId::new([6u8; 32]),
    )
    .expect("link");

    let err = store
        .save_link(&link)
        .expect_err("broken artifact must keep codec error");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_save_link_requires_exec_row_at_write_time() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes"));
    let proof = attest_proof(&draft, snap_id, exec_id);
    let artifact = draft.finalize(proof).expect("artifact");
    let checkpoint_id = save_artifact_file(dir.path(), &artifact);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        snap_id,
        exec_id,
    )
    .expect("link");

    let err = store
        .save_link(&link)
        .expect_err("missing exec row must reject link write");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_save_link_requires_snapshot_row_at_write_time() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = PrepSnapshotId::new([7u8; 32]);
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    let artifact = draft.finalize(proof).expect("artifact");
    let checkpoint_id = save_artifact_file(dir.path(), &artifact);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        snap_id,
        exec_id,
    )
    .expect("link");

    let err = store
        .save_link(&link)
        .expect_err("missing snapshot row must reject link write");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_save_link_rejects_root_drift_at_write_time() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, CheckRoot::new([0xA5; 32]));
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    let artifact = draft.finalize(proof).expect("artifact");
    let checkpoint_id = save_artifact_file(dir.path(), &artifact);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        checkpoint_id,
        snap_id,
        exec_id,
    )
    .expect("link");

    let err = store
        .save_link(&link)
        .expect_err("root drift must reject link write");

    assert!(matches!(err, CheckpointError::RootMix));
}

#[test]
fn test_seal_rejects_exec_row() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = derive_exec_id(&encode_exec_bin(&exec).expect("exec bytes"));
    let proof = attest_proof(&draft, snap_id, exec_id);
    let err = store
        .seal_artifact(&draft, proof, snap_id, exec_id)
        .expect_err("missing exec row must reject");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_seal_rejects_snapshot_row() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = PrepSnapshotId::new([7u8; 32]);
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);

    let err = store
        .seal_artifact(&draft, proof, snap_id, exec_id)
        .expect_err("missing snapshot row must reject");

    assert!(matches!(err, CheckpointError::LinkMix));
}

#[test]
fn test_load_rejects_exec_row() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    stage_contract(&mut store, &draft, &exec, exec_id);
    let link = store
        .seal_artifact(&draft, proof, snap_id, exec_id)
        .expect("seal artifact");

    let exec_path = store
        .exec_dir()
        .join(format!("{}.bin", hex_id(exec_id.as_bytes())));
    std::fs::remove_file(exec_path).expect("remove exec row");

    let err = store
        .load_link(&link.checkpoint_id())
        .expect_err("missing exec row must reject link reload");

    assert!(matches!(err, CheckpointError::ReplayMix));
}

#[test]
fn test_load_rejects_snapshot_row() {
    let dir = temp_dir();
    let mut store = CheckpointFsStore::new(dir.path());
    let draft = checkpoint_fixtures::draft();
    let snap_id = save_snapshot(dir.path(), draft.prev_root());
    let exec = exec(snap_id, draft.prev_root());
    let exec_id = store.save_exec_input(&exec).expect("save exec");
    let proof = attest_proof(&draft, snap_id, exec_id);
    stage_contract(&mut store, &draft, &exec, exec_id);
    let link = store
        .seal_artifact(&draft, proof, snap_id, exec_id)
        .expect("seal artifact");

    let snap_path = PrepFsStore::new(dir.path())
        .snapshot_dir()
        .join(format!("{}.bin", hex_id(snap_id.as_bytes())));
    std::fs::remove_file(snap_path).expect("remove snapshot row");

    let err = store
        .load_link(&link.checkpoint_id())
        .expect_err("missing snapshot row must reject link reload");

    assert!(matches!(err, CheckpointError::LinkMix));
}
