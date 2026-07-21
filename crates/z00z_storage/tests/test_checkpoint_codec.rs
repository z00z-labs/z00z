use z00z_storage::fixture_support::checkpoint_fixtures;

use z00z_storage::{
    checkpoint::audit::{decode_audit_bin, decode_audit_json, encode_audit_bin, encode_audit_json},
    checkpoint::{
        decode_art_bin, decode_art_json, decode_da_reference_bin, decode_da_reference_json,
        decode_draft_bin, decode_draft_json, decode_exec_bin, decode_exec_json, decode_link_bin,
        decode_link_json, decode_publication_evidence_bin, decode_publication_evidence_json,
        encode_art_bin, encode_art_json, encode_da_reference_bin, encode_da_reference_json,
        encode_draft_bin, encode_draft_json, encode_exec_bin, encode_exec_json, encode_link_bin,
        encode_link_json, encode_publication_evidence_bin, encode_publication_evidence_json,
        guard_verified_backend_codec_support, repo_default_path, CheckpointContractConfigV3,
        CheckpointDaLocatorKind, CheckpointDaProviderFamily, CheckpointDaReferenceV1,
        CheckpointDaReferenceVersion, CheckpointExecInputId, CheckpointId, CheckpointProofSystem,
        CheckpointPublicationEvidenceV1, CheckpointPublicationEvidenceVersion,
        CheckpointPublicationState, VERIFIED_BACKEND_CANDIDATE_STAGE,
        VERIFIED_BACKEND_ENABLED_STAGE,
    },
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
fn test_verified_backend_codec_guard_rejects_before_enabled_stage() {
    let cfg = CheckpointContractConfigV3::load(repo_default_path()).expect("repo config");

    let err = guard_verified_backend_codec_support(&cfg, CheckpointProofSystem::VERIFIED)
        .expect_err("default config must reject verified backend codec claims");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    let mut cfg = cfg;
    cfg.authority_promotion.stage = VERIFIED_BACKEND_CANDIDATE_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages = vec![VERIFIED_BACKEND_ENABLED_STAGE.to_string()];
    cfg.post_quantum.is_live_cadence_enforced = true;

    let err = guard_verified_backend_codec_support(&cfg, CheckpointProofSystem::VERIFIED)
        .expect_err("candidate stage must reject verified backend codec claims");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
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
