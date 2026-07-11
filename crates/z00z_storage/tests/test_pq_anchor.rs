use z00z_storage::{
    checkpoint::{
        decode_pq_anchor_bin, decode_pq_anchor_json, encode_pq_anchor_bin, encode_pq_anchor_json,
        repo_default_path, CheckpointContractConfigV1, PostQuantumCheckpointAnchorV1,
        StateSnapshotV1, StateSnapshotVersion, POST_QUANTUM_ENFORCEMENT_STAGE,
        POST_QUANTUM_REQUIRED_ARTIFACTS, VERIFIED_BACKEND_CANDIDATE_STAGE,
    },
    CheckpointError,
};

fn cfg() -> CheckpointContractConfigV1 {
    CheckpointContractConfigV1::load(repo_default_path()).expect("repo checkpoint contract")
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
fn test_pq_anchor_writer_stays_declared_only_before_live_stage() {
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
        .expect("pre-stage pq anchor build")
        .is_none());

    cfg.validate_pq_anchor(1000, root(1), root(2), root(3), root(4), None)
        .expect("pre-stage cadence remains declared-only");
}

#[test]
fn test_live_stage_requires_complete_pq_anchor_on_cadence() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages =
        vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];
    cfg.post_quantum.enforce_live_cadence = true;

    let missing = cfg
        .validate_pq_anchor(1000, root(1), root(2), root(3), root(4), None)
        .expect_err("live cadence must require pq anchor");

    assert!(matches!(missing, CheckpointError::Backend(_)));

    let anchor = cfg
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
        .expect("live pq anchor build")
        .expect("cadence-height anchor");

    cfg.validate_pq_anchor(1000, root(1), root(2), root(3), root(4), Some(&anchor))
        .expect("live cadence accepts complete anchor");

    StateSnapshotV1::new(
        StateSnapshotVersion::CURRENT,
        10_000,
        10,
        10_000,
        root(11),
        root(12),
        root(13),
        root(14),
        root(15),
        root(16),
        anchor.pq_anchor_root(),
        root(17),
    )
    .expect("snapshot accepts pq anchor root");
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
    let anchor = live_anchor();

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
fn test_pq_anchor_mismatch_rejects() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages =
        vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];
    cfg.post_quantum.enforce_live_cadence = true;
    let anchor = live_anchor();

    let err = cfg
        .validate_pq_anchor(1000, root(1), root(2), root(3), root(99), Some(&anchor))
        .expect_err("archive mismatch must reject");

    assert!(matches!(err, CheckpointError::Backend(_)));
}

#[test]
fn test_verified_backend_candidate_stage_still_requires_live_pq_cadence() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = VERIFIED_BACKEND_CANDIDATE_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages = vec!["verified_backend_enabled".to_string()];
    cfg.post_quantum.enforce_live_cadence = false;

    let err = cfg
        .validate()
        .expect_err("verified backend candidate stage must keep live pq cadence");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    cfg.post_quantum.enforce_live_cadence = true;
    cfg.validate().expect("verified backend candidate contract");
}

fn live_anchor() -> PostQuantumCheckpointAnchorV1 {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages =
        vec![VERIFIED_BACKEND_CANDIDATE_STAGE.to_string()];
    cfg.post_quantum.enforce_live_cadence = true;
    cfg.build_pq_anchor(
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
    .expect("live pq anchor build")
    .expect("cadence-height anchor")
}
