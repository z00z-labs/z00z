use z00z_storage::{
    checkpoint::{
        decode_pq_anchor_bin, decode_pq_anchor_json, encode_pq_anchor_bin, encode_pq_anchor_json,
        repo_default_path, CheckpointContractConfigV3, PostQuantumCheckpointAnchorModeV1,
        PostQuantumCheckpointAnchorV1, PostQuantumCheckpointAnchorVersion,
        PostQuantumCheckpointEnforcementStageV1, StateSnapshotV1, StateSnapshotVersion,
        POST_QUANTUM_ENFORCEMENT_STAGE, POST_QUANTUM_REQUIRED_ARTIFACTS,
        VERIFIED_BACKEND_CANDIDATE_STAGE,
    },
    CheckpointError,
};

fn cfg() -> CheckpointContractConfigV3 {
    CheckpointContractConfigV3::load(repo_default_path()).expect("repo checkpoint contract")
}

fn root(byte: u8) -> [u8; 32] {
    [byte; 32]
}

#[test]
fn test_pq_cadence_distinguishes_height_999_and_1000() {
    let cfg = cfg();

    assert!(!cfg.has_pq_checkpoint(999));
    assert!(cfg.has_pq_checkpoint(1000));
    assert!(cfg.has_pq_checkpoint(2000));
}

#[test]
fn test_pq_required_artifacts_match_contract() {
    let cfg = cfg();

    assert_eq!(
        cfg.post_quantum.required_artifacts,
        POST_QUANTUM_REQUIRED_ARTIFACTS
            .iter()
            .map(|field| (*field).to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_state_snapshot_requires_nonzero_pq_anchor_root() {
    let err = StateSnapshotV1::new(
        StateSnapshotVersion::CURRENT,
        10_000,
        10,
        10_000,
        root(1),
        root(2),
        root(3),
        root(4),
        root(5),
        root(6),
        [0u8; 32],
        root(8),
    )
    .expect_err("zero pq anchor root must reject");

    assert!(matches!(err, CheckpointError::SnapshotMix));
}

#[test]
fn test_pq_anchor_writer_stays_non_authoritative_without_a_bound_writer() {
    let cfg = cfg();

    assert!(cfg
        .build_pq_anchor(
            1000,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
        )
        .expect("non-authoritative pq anchor build")
        .is_none());

    cfg.validate_pq_anchor(1000, root(1), root(2), root(3), root(4), None)
        .expect("non-authoritative cadence remains declared-only");
}

#[test]
fn test_live_pq_stage_rejects_without_a_bound_writer() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages =
        vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];
    cfg.post_quantum.is_live_cadence_enforced = true;

    let missing = cfg
        .validate_pq_anchor(1000, root(1), root(2), root(3), root(4), None)
        .expect_err("unbound PQ writer stage must reject");

    assert!(matches!(missing, CheckpointError::ContractConfig(_)));

    let build = cfg
        .build_pq_anchor(
            1000,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
        )
        .expect_err("an unbound backend must not build a live PQ anchor");
    assert!(matches!(build, CheckpointError::ContractConfig(_)));

    let anchor = retained_phase068_anchor();
    let validate = cfg
        .validate_pq_anchor(1000, root(1), root(2), root(3), root(4), Some(&anchor))
        .expect_err("an unbound backend must not validate a live PQ anchor");
    assert!(matches!(validate, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_live_pq_entry_points_reject_disabled_live_cadence() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages =
        vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];

    let build_err = cfg
        .build_pq_anchor(
            1000,
            root(1),
            root(2),
            root(3),
            root(4),
            root(5),
            root(6),
            root(7),
            root(8),
            root(9),
        )
        .expect_err("live PQ entry points must reject disabled cadence");
    assert!(matches!(build_err, CheckpointError::ContractConfig(_)));

    let validate_err = cfg
        .validate_pq_anchor(1000, root(1), root(2), root(3), root(4), None)
        .expect_err("live PQ validation must reject disabled cadence");
    assert!(matches!(validate_err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_pq_anchor_roundtrip_and_unknown_field_reject() {
    let anchor = retained_phase068_anchor();

    assert_eq!(
        decode_pq_anchor_bin(&encode_pq_anchor_bin(&anchor).expect("pq anchor bin"))
            .expect("pq anchor"),
        anchor
    );
    assert_eq!(
        decode_pq_anchor_json(&encode_pq_anchor_json(&anchor).expect("pq anchor json"))
            .expect("pq anchor"),
        anchor
    );

    let mut value = serde_json::to_value(&anchor).expect("pq anchor json");
    value["unexpected_field"] = serde_json::json!("drift");
    let bytes = serde_json::to_vec(&value).expect("pq anchor bytes");

    let err = decode_pq_anchor_json(&bytes).expect_err("unknown pq field must reject");

    assert!(matches!(err, CheckpointError::Codec(_)));
}

#[test]
fn test_pq_anchor_live_validation_rejects_without_a_bound_backend() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages =
        vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];
    cfg.post_quantum.is_live_cadence_enforced = true;
    let anchor = retained_phase068_anchor();

    let err = cfg
        .validate_pq_anchor(1000, root(1), root(2), root(3), root(99), Some(&anchor))
        .expect_err("live PQ anchor validation requires a bound backend");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_candidate_stage_rejects_without_promotion_evidence() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = VERIFIED_BACKEND_CANDIDATE_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages = vec!["verified_backend_enabled".to_string()];
    cfg.post_quantum.is_live_cadence_enforced = false;

    let err = cfg
        .validate()
        .expect_err("an unbound verified-backend stage must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    cfg.post_quantum.is_live_cadence_enforced = true;
    let err = cfg
        .validate()
        .expect_err("an unbound verified-backend stage must reject with cadence enabled");
    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

fn retained_phase068_anchor() -> PostQuantumCheckpointAnchorV1 {
    PostQuantumCheckpointAnchorV1::new(
        PostQuantumCheckpointAnchorVersion::CURRENT,
        1000,
        1000,
        root(1),
        root(2),
        root(3),
        root(4),
        root(5),
        root(6),
        root(7),
        root(8),
        root(9),
        PostQuantumCheckpointAnchorModeV1::Plonky3EpochProof,
        PostQuantumCheckpointEnforcementStageV1::PqAnchorWriter,
    )
    .expect("Phase 068 retained anchor fixture")
}
