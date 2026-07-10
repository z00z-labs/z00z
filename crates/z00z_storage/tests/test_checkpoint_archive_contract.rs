use serde_json::Value;
use z00z_storage::fixture_support::checkpoint_fixtures;
use z00z_storage::{
    checkpoint::{
        decode_archive_manifest_bin, decode_archive_manifest_json, decode_archive_receipt_bin,
        decode_archive_receipt_json, decode_da_reference_bin, decode_da_reference_json,
        decode_publication_evidence_bin, decode_publication_evidence_json,
        decode_retrieval_audit_bin, decode_retrieval_audit_json, decode_state_snapshot_bin,
        decode_state_snapshot_json, encode_archive_manifest_bin, encode_archive_manifest_json,
        encode_archive_receipt_bin, encode_archive_receipt_json, encode_da_reference_bin,
        encode_da_reference_json, encode_publication_evidence_bin,
        encode_publication_evidence_json, encode_retrieval_audit_bin, encode_retrieval_audit_json,
        encode_state_snapshot_bin, encode_state_snapshot_json, ArchiveBackend,
        ArchiveProviderReceiptV1, ArchiveProviderReceiptVersion, CheckpointArchiveManifestV1,
        CheckpointDaLocatorKind, CheckpointDaProviderFamily, CheckpointDaReferenceV1,
        CheckpointDaReferenceVersion, CheckpointPublicationEvidenceV1,
        CheckpointPublicationEvidenceVersion, CheckpointPublicationState, RetrievalAuditV1,
        RetrievalAuditVersion, StateSnapshotV1, StateSnapshotVersion,
    },
    CheckpointError,
};

fn root(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn json_root(byte: u8) -> Value {
    Value::Array((0..32).map(|_| Value::from(byte)).collect())
}

fn manifest() -> CheckpointArchiveManifestV1 {
    checkpoint_fixtures::archive_manifest(
        &checkpoint_fixtures::draft(),
        &checkpoint_fixtures::exec(),
        z00z_storage::checkpoint::CheckpointExecInputId::new([8u8; 32]),
    )
}

fn da_reference() -> CheckpointDaReferenceV1 {
    let manifest = manifest();
    CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::LocalArchive,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        "local-da://archive/001",
        manifest.da_payload_commitment(),
        manifest.statement_core_digest(),
        manifest.archive_manifest_root(),
        1000,
    )
    .expect("da reference")
}

fn publication_evidence() -> CheckpointPublicationEvidenceV1 {
    let manifest = manifest();
    let da_reference = da_reference();
    CheckpointPublicationEvidenceV1::new(
        CheckpointPublicationEvidenceVersion::CURRENT,
        manifest.statement_core_digest(),
        da_reference.da_ref(),
        manifest.archive_manifest_root(),
        manifest.da_payload_commitment(),
        CheckpointPublicationState::DaPublicationReady,
        CheckpointDaProviderFamily::LocalArchive,
        1000,
        1000,
        root(14),
    )
    .expect("publication evidence")
}

fn receipt() -> ArchiveProviderReceiptV1 {
    ArchiveProviderReceiptV1::new(
        ArchiveProviderReceiptVersion::CURRENT,
        ArchiveBackend::IpfsPinned,
        root(11),
        4096,
        root(12),
        root(13),
        true,
        true,
    )
    .expect("archive receipt")
}

fn audit() -> RetrievalAuditV1 {
    RetrievalAuditV1::new(
        RetrievalAuditVersion::CURRENT,
        1000,
        1000,
        root(21),
        root(22),
        root(23),
        root(24),
        3,
        true,
    )
    .expect("retrieval audit")
}

fn snapshot() -> StateSnapshotV1 {
    StateSnapshotV1::new(
        StateSnapshotVersion::CURRENT,
        10_000,
        10,
        10_000,
        root(31),
        root(32),
        root(33),
        root(34),
        root(35),
        root(36),
        root(37),
        root(38),
    )
    .expect("state snapshot")
}

fn tamper_json(bytes: &[u8], field: &str, value: Value) -> Vec<u8> {
    let mut wire: Value = serde_json::from_slice(bytes).expect("json wire");
    wire[field] = value;
    serde_json::to_vec_pretty(&wire).expect("json bytes")
}

#[test]
fn test_bound_archive_objects_roundtrip_json_and_bin() {
    let manifest = manifest();
    let receipt = receipt();
    let da_reference = da_reference();
    let publication_evidence = publication_evidence();
    let audit = audit();
    let snapshot = snapshot();

    assert_eq!(
        decode_archive_manifest_bin(&encode_archive_manifest_bin(&manifest).expect("manifest bin"))
            .expect("manifest"),
        manifest
    );
    assert_eq!(
        decode_archive_manifest_json(
            &encode_archive_manifest_json(&manifest).expect("manifest json")
        )
        .expect("manifest"),
        manifest
    );
    assert_eq!(
        decode_archive_receipt_bin(&encode_archive_receipt_bin(&receipt).expect("receipt bin"))
            .expect("receipt"),
        receipt
    );
    assert_eq!(
        decode_archive_receipt_json(&encode_archive_receipt_json(&receipt).expect("receipt json"))
            .expect("receipt"),
        receipt
    );
    assert_eq!(
        decode_da_reference_bin(&encode_da_reference_bin(&da_reference).expect("da ref bin"))
            .expect("da reference"),
        da_reference
    );
    assert_eq!(
        decode_da_reference_json(&encode_da_reference_json(&da_reference).expect("da ref json"))
            .expect("da reference"),
        da_reference
    );
    assert_eq!(
        decode_publication_evidence_bin(
            &encode_publication_evidence_bin(&publication_evidence)
                .expect("publication evidence bin")
        )
        .expect("publication evidence"),
        publication_evidence
    );
    assert_eq!(
        decode_publication_evidence_json(
            &encode_publication_evidence_json(&publication_evidence)
                .expect("publication evidence json")
        )
        .expect("publication evidence"),
        publication_evidence
    );
    assert_eq!(
        decode_retrieval_audit_bin(&encode_retrieval_audit_bin(&audit).expect("audit bin"))
            .expect("audit"),
        audit
    );
    assert_eq!(
        decode_retrieval_audit_json(&encode_retrieval_audit_json(&audit).expect("audit json"))
            .expect("audit"),
        audit
    );
    assert_eq!(
        decode_state_snapshot_bin(&encode_state_snapshot_bin(&snapshot).expect("snapshot bin"))
            .expect("snapshot"),
        snapshot
    );
    assert_eq!(
        decode_state_snapshot_json(&encode_state_snapshot_json(&snapshot).expect("snapshot json"))
            .expect("snapshot"),
        snapshot
    );
}

#[test]
fn test_manifest_tamper_rejects_json_decode() {
    let manifest = manifest();
    let bytes = encode_archive_manifest_json(&manifest).expect("manifest json");
    let tampered = tamper_json(&bytes, "statement_core_digest", json_root(77));

    let err = decode_archive_manifest_json(&tampered).expect_err("tampered manifest must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_da_reference_tamper_rejects_json_decode() {
    let da_reference = da_reference();
    let bytes = encode_da_reference_json(&da_reference).expect("da ref json");
    let tampered = tamper_json(&bytes, "archive_manifest_root", json_root(78));

    let err = decode_da_reference_json(&tampered).expect_err("tampered da ref must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_publication_evidence_tamper_rejects_json_decode() {
    let publication_evidence = publication_evidence();
    let bytes =
        encode_publication_evidence_json(&publication_evidence).expect("publication evidence json");
    let tampered = tamper_json(&bytes, "payload_commitment", json_root(79));

    let err = decode_publication_evidence_json(&tampered)
        .expect_err("tampered publication evidence must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_receipt_tamper_rejects_json_decode() {
    let receipt = receipt();
    let bytes = encode_archive_receipt_json(&receipt).expect("receipt json");
    let tampered = tamper_json(&bytes, "receipt_digest", json_root(88));

    let err = decode_archive_receipt_json(&tampered).expect_err("tampered receipt must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_state_snapshot_tamper_rejects_json_decode() {
    let snapshot = snapshot();
    let bytes = encode_state_snapshot_json(&snapshot).expect("snapshot json");
    let tampered = tamper_json(&bytes, "pq_anchor_root", json_root(99));

    let err = decode_state_snapshot_json(&tampered).expect_err("tampered snapshot must reject");

    assert!(matches!(err, CheckpointError::SnapshotMix));
}

#[test]
fn test_retrieval_audit_requires_cadence_multiple() {
    let err = RetrievalAuditV1::new(
        RetrievalAuditVersion::CURRENT,
        1500,
        1000,
        root(21),
        root(22),
        root(23),
        root(24),
        3,
        true,
    )
    .expect_err("height outside cadence multiple must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_archive_manifest_rejects_artifact_id_field() {
    let bytes = encode_archive_manifest_json(&manifest()).expect("manifest json");
    let tampered = tamper_json(&bytes, "checkpoint_id", json_root(80));

    let err = decode_archive_manifest_json(&tampered).expect_err("checkpoint id field must reject");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_da_reference_rejects_provider_sdk_field() {
    let bytes = encode_da_reference_json(&da_reference()).expect("da ref json");
    let tampered = tamper_json(
        &bytes,
        "provider_sdk_receipt",
        serde_json::json!({"namespace":"sdk-only"}),
    );

    let err = decode_da_reference_json(&tampered)
        .expect_err("provider sdk field in canonical bytes must reject");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_publication_evidence_requires_ready_window() {
    let manifest = manifest();
    let da_reference = da_reference();
    let err = CheckpointPublicationEvidenceV1::new(
        CheckpointPublicationEvidenceVersion::CURRENT,
        manifest.statement_core_digest(),
        da_reference.da_ref(),
        manifest.archive_manifest_root(),
        manifest.da_payload_commitment(),
        CheckpointPublicationState::DaPublicationReady,
        CheckpointDaProviderFamily::LocalArchive,
        1000,
        999,
        root(14),
    )
    .expect_err("challenge window before readiness must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}

#[test]
fn test_da_reference_rejects_bare_cid_locator() {
    let manifest = manifest();
    let err = CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::LocalArchive,
        CheckpointDaLocatorKind::ContentCid,
        "bafybarecid",
        manifest.da_payload_commitment(),
        manifest.statement_core_digest(),
        manifest.archive_manifest_root(),
        1000,
    )
    .expect_err("bare cid locator must reject");

    assert!(matches!(err, CheckpointError::ArchiveMix));
}
