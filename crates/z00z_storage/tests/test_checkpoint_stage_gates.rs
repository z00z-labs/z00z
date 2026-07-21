use z00z_storage::{
    checkpoint::{
        repo_default_path, CheckpointContractConfigV3, AUTHORITY_PROMOTION_STAGE_CONFIG_GATE,
        AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT, AUTHORITY_PROMOTION_STAGE_SPEC_ONLY,
        POST_QUANTUM_ENFORCEMENT_STAGE, VERIFIED_BACKEND_CANDIDATE_STAGE,
        VERIFIED_BACKEND_ENABLED_STAGE,
    },
    CheckpointError,
};

fn cfg() -> CheckpointContractConfigV3 {
    CheckpointContractConfigV3::load(repo_default_path()).expect("repo checkpoint contract")
}

fn stage_cfg(stage: &str, next: &[&str]) -> CheckpointContractConfigV3 {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = stage.to_string();
    cfg.authority_promotion.allowed_next_stages = next.iter().map(|s| (*s).to_string()).collect();
    cfg
}

#[test]
fn test_only_pre_activation_stages_validate_at_t0() {
    let cases: [(&str, &[&str]); 3] = [
        (
            AUTHORITY_PROMOTION_STAGE_SPEC_ONLY,
            &[AUTHORITY_PROMOTION_STAGE_CONFIG_GATE],
        ),
        (
            AUTHORITY_PROMOTION_STAGE_CONFIG_GATE,
            &[AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT],
        ),
        (AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT, &[]),
    ];

    for (stage, next) in cases {
        stage_cfg(stage, next)
            .validate()
            .unwrap_or_else(|err| panic!("stage {stage} -> {:?} must validate: {err}", next));
    }
}

#[test]
fn test_stage_skip_rejects() {
    let err = stage_cfg(
        AUTHORITY_PROMOTION_STAGE_SPEC_ONLY,
        &[AUTHORITY_PROMOTION_STAGE_EXTENDED_STATEMENT],
    )
    .validate()
    .expect_err("stage skip must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_proof_system_stages_reject_without_a_verified_recursive_backend() {
    for stage in [
        POST_QUANTUM_ENFORCEMENT_STAGE,
        VERIFIED_BACKEND_CANDIDATE_STAGE,
        VERIFIED_BACKEND_ENABLED_STAGE,
    ] {
        let err = stage_cfg(stage, &[])
            .validate()
            .expect_err("unimplemented proof-system stage must reject");
        assert!(matches!(err, CheckpointError::ContractConfig(_)));
    }
}

#[test]
fn test_authority_flags_reject_without_completed_promotion_evidence() {
    let mut recursive_cfg = cfg();
    recursive_cfg
        .authority_promotion
        .is_recursive_authority_allowed = true;

    let err = recursive_cfg
        .validate()
        .expect_err("recursive authority requires completed promotion evidence");
    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    let mut cfg = cfg();
    cfg.authority_promotion.is_verified_backend_allowed = true;

    let err = cfg
        .validate()
        .expect_err("verified backend requires completed promotion evidence");
    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}
