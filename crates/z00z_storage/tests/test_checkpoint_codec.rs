use z00z_storage::fixture_support::checkpoint_fixtures;

use z00z_storage::{
    checkpoint::audit::{decode_audit_bin, decode_audit_json, encode_audit_bin, encode_audit_json},
    checkpoint::{
        decode_art_bin, decode_art_json, decode_da_reference_bin, decode_da_reference_json,
        decode_draft_bin, decode_draft_json, decode_exec_bin, decode_exec_json, decode_link_bin,
        decode_link_json, decode_publication_evidence_bin, decode_publication_evidence_json,
        decode_recursive_sidecar_bin, decode_recursive_sidecar_json, encode_art_bin,
        encode_art_json, encode_da_reference_bin, encode_da_reference_json, encode_draft_bin,
        encode_draft_json, encode_exec_bin, encode_exec_json, encode_link_bin, encode_link_json,
        encode_publication_evidence_bin, encode_publication_evidence_json,
        encode_recursive_sidecar_bin, encode_recursive_sidecar_json,
        guard_verified_backend_codec_support, repo_default_path, CheckpointContractConfigV1,
        CheckpointDaLocatorKind, CheckpointDaProviderFamily, CheckpointDaReferenceV1,
        CheckpointDaReferenceVersion, CheckpointExecInputId, CheckpointId, CheckpointLink,
        CheckpointLinkVersion, CheckpointProofSystem, CheckpointPubIn,
        CheckpointPublicationEvidenceV1, CheckpointPublicationEvidenceVersion,
        CheckpointPublicationState, CheckpointTransitionStatementCoreV1,
        CheckpointTransitionStatementFinalV1, CheckpointTransitionStatementV1, CheckpointVersion,
        CreatedEnt, RecursiveCheckpointMeasurementV1, RecursiveCheckpointModeV1,
        RecursiveCheckpointProofFamilyV1, RecursiveCheckpointProofV1, RecursiveCheckpointSidecarV1,
        RecursiveCheckpointVerifierV1, SpentEnt, VERIFIED_BACKEND_CANDIDATE_STAGE,
        VERIFIED_BACKEND_ENABLED_STAGE, VERIFIED_BACKEND_REVIEW_APPROVED,
    },
    settlement::SettlementStateRoot,
    snapshot::PrepSnapshotId,
    CheckpointError,
};

#[test]
fn test_json_roundtrip_keeps_types() {
    assert_eq!(
        decode_draft_json(&encode_draft_json(&checkpoint_fixtures::draft()).expect("draft json"))
            .expect("draft"),
        checkpoint_fixtures::draft()
    );
    assert_eq!(
        decode_art_json(&encode_art_json(&checkpoint_fixtures::artifact()).expect("art json"))
            .expect("artifact"),
        checkpoint_fixtures::artifact()
    );
    assert_eq!(
        decode_link_json(
            &encode_link_json(&checkpoint_fixtures::link(
                CheckpointId::new([6u8; 32]),
                CheckpointExecInputId::new([8u8; 32]),
            ))
            .expect("link json"),
        )
        .expect("link"),
        checkpoint_fixtures::link(
            CheckpointId::new([6u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
    );
    assert_eq!(
        decode_exec_json(&encode_exec_json(&checkpoint_fixtures::exec()).expect("exec json"))
            .expect("exec"),
        checkpoint_fixtures::exec()
    );
    assert_eq!(
        decode_audit_json(
            &encode_audit_json(&checkpoint_fixtures::audit(CheckpointId::new([1u8; 32])))
                .expect("audit json"),
        )
        .expect("audit"),
        checkpoint_fixtures::audit(CheckpointId::new([1u8; 32]))
    );
    assert_eq!(
        decode_da_reference_json(&encode_da_reference_json(&da_reference()).expect("da ref json"))
            .expect("da reference"),
        da_reference()
    );
    assert_eq!(
        decode_publication_evidence_json(
            &encode_publication_evidence_json(&publication_evidence())
                .expect("publication evidence json")
        )
        .expect("publication evidence"),
        publication_evidence()
    );
    assert_eq!(
        decode_recursive_sidecar_json(
            &encode_recursive_sidecar_json(&recursive_sidecar()).expect("recursive sidecar json")
        )
        .expect("recursive sidecar"),
        recursive_sidecar()
    );
}

#[test]
fn test_bin_roundtrip_keeps_types() {
    assert_eq!(
        decode_draft_bin(&encode_draft_bin(&checkpoint_fixtures::draft()).expect("draft bin"))
            .expect("draft"),
        checkpoint_fixtures::draft()
    );
    assert_eq!(
        decode_art_bin(&encode_art_bin(&checkpoint_fixtures::artifact()).expect("art bin"))
            .expect("artifact"),
        checkpoint_fixtures::artifact()
    );
    assert_eq!(
        decode_link_bin(
            &encode_link_bin(&checkpoint_fixtures::link(
                CheckpointId::new([6u8; 32]),
                CheckpointExecInputId::new([8u8; 32]),
            ))
            .expect("link bin"),
        )
        .expect("link"),
        checkpoint_fixtures::link(
            CheckpointId::new([6u8; 32]),
            CheckpointExecInputId::new([8u8; 32]),
        )
    );
    assert_eq!(
        decode_exec_bin(&encode_exec_bin(&checkpoint_fixtures::exec()).expect("exec bin"))
            .expect("exec"),
        checkpoint_fixtures::exec()
    );
    assert_eq!(
        decode_audit_bin(
            &encode_audit_bin(&checkpoint_fixtures::audit(CheckpointId::new([1u8; 32])))
                .expect("audit bin"),
        )
        .expect("audit"),
        checkpoint_fixtures::audit(CheckpointId::new([1u8; 32]))
    );
    assert_eq!(
        decode_da_reference_bin(&encode_da_reference_bin(&da_reference()).expect("da ref bin"))
            .expect("da reference"),
        da_reference()
    );
    assert_eq!(
        decode_publication_evidence_bin(
            &encode_publication_evidence_bin(&publication_evidence())
                .expect("publication evidence bin")
        )
        .expect("publication evidence"),
        publication_evidence()
    );
    assert_eq!(
        decode_recursive_sidecar_bin(
            &encode_recursive_sidecar_bin(&recursive_sidecar()).expect("recursive sidecar bin")
        )
        .expect("recursive sidecar"),
        recursive_sidecar()
    );
}

#[test]
fn test_wrong_class_payload_rejects() {
    let bytes = encode_audit_json(&checkpoint_fixtures::audit(CheckpointId::new([1u8; 32])))
        .expect("audit json");
    let err = decode_art_json(&bytes).expect_err("audit must not decode as artifact");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_prior_stage6_wrapper_rejects() {
    let bytes = checkpoint_fixtures::prior_stage6_json();

    let draft_err = decode_draft_json(&bytes).expect_err("prior wrapper must not load as draft");
    let art_err = decode_art_json(&bytes).expect_err("prior wrapper must not load as artifact");

    assert!(matches!(draft_err, CheckpointError::Codec(_)));
    assert!(matches!(art_err, CheckpointError::Codec(_)));
}

#[test]
fn test_malformed_transport_rejects() {
    let err = decode_exec_json(br#"{"version":1,"prev_root":"bad"}"#).expect_err("bad transport");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_sidecar_unknown_rejects() {
    let mut value = serde_json::to_value(recursive_sidecar()).expect("recursive sidecar json");
    value["unexpected_field"] = serde_json::json!("drift");
    let bytes = serde_json::to_vec(&value).expect("recursive sidecar bytes");

    let err =
        decode_recursive_sidecar_json(&bytes).expect_err("unknown recursive field must reject");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_verified_backend_codec_guard_rejects_before_enabled_stage() {
    let cfg = CheckpointContractConfigV1::load(repo_default_path()).expect("repo config");

    let err = guard_verified_backend_codec_support(&cfg, CheckpointProofSystem::VERIFIED)
        .expect_err("default config must reject verified backend codec claims");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    let mut cfg = cfg;
    cfg.authority_promotion.stage = VERIFIED_BACKEND_CANDIDATE_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages = vec![VERIFIED_BACKEND_ENABLED_STAGE.to_string()];
    cfg.post_quantum.enforce_live_cadence = true;

    let err = guard_verified_backend_codec_support(&cfg, CheckpointProofSystem::VERIFIED)
        .expect_err("candidate stage must reject verified backend codec claims");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_codec_guard_accepts_complete_enabled_contract() {
    let mut cfg = CheckpointContractConfigV1::load(repo_default_path()).expect("repo config");
    cfg.authority_promotion.stage = VERIFIED_BACKEND_ENABLED_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages.clear();
    cfg.authority_promotion.recursive_authority_allowed = true;
    cfg.authority_promotion.verified_backend_allowed = true;
    cfg.post_quantum.enforce_live_cadence = true;
    cfg.verified_backend.security_review.status = VERIFIED_BACKEND_REVIEW_APPROVED.to_string();

    guard_verified_backend_codec_support(&cfg, CheckpointProofSystem::VERIFIED)
        .expect("complete enabled contract must declare codec readiness");
    guard_verified_backend_codec_support(&cfg, CheckpointProofSystem::OPAQUE_ATTEST)
        .expect("opaque attest must remain allowed");
}

fn da_reference() -> CheckpointDaReferenceV1 {
    CheckpointDaReferenceV1::new(
        CheckpointDaReferenceVersion::CURRENT,
        CheckpointDaProviderFamily::LocalArchive,
        CheckpointDaLocatorKind::OpaqueProviderRef,
        "local-da://codec/001",
        [21u8; 32],
        [22u8; 32],
        [23u8; 32],
        1000,
    )
    .expect("da reference")
}

fn publication_evidence() -> CheckpointPublicationEvidenceV1 {
    let da_reference = da_reference();
    CheckpointPublicationEvidenceV1::new(
        CheckpointPublicationEvidenceVersion::CURRENT,
        [22u8; 32],
        da_reference.da_ref(),
        [23u8; 32],
        [21u8; 32],
        CheckpointPublicationState::DaPublicationReady,
        CheckpointDaProviderFamily::LocalArchive,
        1000,
        1000,
        [24u8; 32],
    )
    .expect("publication evidence")
}

fn recursive_sidecar() -> RecursiveCheckpointSidecarV1 {
    let cfg = CheckpointContractConfigV1::load(repo_default_path()).expect("repo config");
    let verifier = RecursiveCheckpointVerifierV1::new(&cfg).expect("recursive verifier");
    let pub_in = CheckpointPubIn::new_settlement(
        SettlementStateRoot::settlement_v1([41u8; 32]),
        SettlementStateRoot::settlement_v1([42u8; 32]),
        vec![SpentEnt::new([43u8; 32])],
        vec![CreatedEnt::new([44u8; 32], [45u8; 32])],
    );
    let statement = CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        77,
        pub_in,
        PrepSnapshotId::new([46u8; 32]),
        CheckpointExecInputId::new([47u8; 32]),
    );
    let core =
        CheckpointTransitionStatementCoreV1::new([48u8; 32], [49u8; 32], [50u8; 32], [51u8; 32])
            .with_prior_recursive_output_root([41u8; 32]);
    let final_bind = CheckpointTransitionStatementFinalV1::new([52u8; 32]);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([53u8; 32]),
        statement.prep_snapshot_id(),
        statement.exec_input_id(),
    )
    .expect("checkpoint link");
    let public_input = verifier
        .build_public_input(
            &statement,
            &core,
            &final_bind,
            &link,
            RecursiveCheckpointModeV1::FastClassicalCompressed,
            "nova_compressed_v1",
            0,
            3,
            [54u8; 32],
        )
        .expect("public input");
    let proof = RecursiveCheckpointProofV1::new(
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        "nova_compressed_v1",
        &public_input,
        vec![55u8; 96],
    )
    .expect("recursive proof");
    let measurement = RecursiveCheckpointMeasurementV1::new(
        "nova_compressed_v1",
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
        cfg.post_quantum.cadence_blocks,
        2,
        RecursiveCheckpointProofFamilyV1::Nova,
        0,
        96,
        4096,
        13,
        7,
        8192,
        statement.canonical_bytes_v1().len() as u64,
        public_input.canonical_bytes().len() as u64,
    )
    .expect("measurement");
    RecursiveCheckpointSidecarV1::accepted(
        public_input,
        Some(link.checkpoint_id()),
        proof,
        measurement,
    )
    .expect("recursive sidecar")
}
