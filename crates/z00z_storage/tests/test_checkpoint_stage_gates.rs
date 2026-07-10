use z00z_storage::{
    checkpoint::{
        repo_default_path, CheckpointContractConfigV1,
        AUTHORITY_PROMOTION_STAGE_CANONICAL_EXTENDED_STATEMENT,
        AUTHORITY_PROMOTION_STAGE_CONFIG_GATE, AUTHORITY_PROMOTION_STAGE_RECURSIVE_SHADOW_SIDECAR,
        AUTHORITY_PROMOTION_STAGE_SPEC_ONLY, POST_QUANTUM_ENFORCEMENT_STAGE,
        VERIFIED_BACKEND_CANDIDATE_STAGE, VERIFIED_BACKEND_ENABLED_STAGE,
        VERIFIED_BACKEND_REVIEW_APPROVED,
    },
    CheckpointError,
};

fn cfg() -> CheckpointContractConfigV1 {
    CheckpointContractConfigV1::load(repo_default_path()).expect("repo checkpoint contract")
}

fn stage_cfg(stage: &str, next: &[&str], live_pq_cadence: bool) -> CheckpointContractConfigV1 {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = stage.to_string();
    cfg.authority_promotion.allowed_next_stages = next.iter().map(|s| (*s).to_string()).collect();
    cfg.post_quantum.enforce_live_cadence = live_pq_cadence;
    cfg
}

fn verified_enabled_cfg() -> CheckpointContractConfigV1 {
    let mut cfg = stage_cfg(VERIFIED_BACKEND_ENABLED_STAGE, &[], true);
    cfg.authority_promotion.recursive_authority_allowed = true;
    cfg.authority_promotion.verified_backend_allowed = true;
    cfg.verified_backend.security_review.status = VERIFIED_BACKEND_REVIEW_APPROVED.to_string();
    cfg
}

#[test]
fn test_all_legal_phase068_stages_validate() {
    let cases: [(&str, &[&str], bool); 6] = [
        (
            AUTHORITY_PROMOTION_STAGE_SPEC_ONLY,
            &[AUTHORITY_PROMOTION_STAGE_CONFIG_GATE],
            false,
        ),
        (
            AUTHORITY_PROMOTION_STAGE_CONFIG_GATE,
            &[AUTHORITY_PROMOTION_STAGE_CANONICAL_EXTENDED_STATEMENT],
            false,
        ),
        (
            AUTHORITY_PROMOTION_STAGE_CANONICAL_EXTENDED_STATEMENT,
            &[AUTHORITY_PROMOTION_STAGE_RECURSIVE_SHADOW_SIDECAR],
            false,
        ),
        (
            AUTHORITY_PROMOTION_STAGE_RECURSIVE_SHADOW_SIDECAR,
            &[POST_QUANTUM_ENFORCEMENT_STAGE],
            false,
        ),
        (
            POST_QUANTUM_ENFORCEMENT_STAGE,
            &[VERIFIED_BACKEND_CANDIDATE_STAGE],
            true,
        ),
        (
            VERIFIED_BACKEND_CANDIDATE_STAGE,
            &[VERIFIED_BACKEND_ENABLED_STAGE],
            true,
        ),
    ];

    for (stage, next, live_pq_cadence) in cases {
        stage_cfg(stage, next, live_pq_cadence)
            .validate()
            .unwrap_or_else(|err| panic!("stage {stage} -> {:?} must validate: {err}", next));
    }

    verified_enabled_cfg()
        .validate()
        .expect("verified backend enabled contract");
}

#[test]
fn test_stage_skip_rejects() {
    let err = stage_cfg(
        AUTHORITY_PROMOTION_STAGE_SPEC_ONLY,
        &[AUTHORITY_PROMOTION_STAGE_RECURSIVE_SHADOW_SIDECAR],
        false,
    )
    .validate()
    .expect_err("stage skip must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_enabled_requires_empty_next_stage_set() {
    let mut cfg = verified_enabled_cfg();
    cfg.authority_promotion.allowed_next_stages = vec![VERIFIED_BACKEND_ENABLED_STAGE.to_string()];

    let err = cfg
        .validate()
        .expect_err("verified backend enabled must be terminal");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_flag_rejects_before_promotion() {
    let mut cfg = stage_cfg(
        POST_QUANTUM_ENFORCEMENT_STAGE,
        &[VERIFIED_BACKEND_CANDIDATE_STAGE],
        true,
    );
    cfg.authority_promotion.verified_backend_allowed = true;

    let err = cfg
        .validate()
        .expect_err("verified backend flag must reject before promotion");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_recursive_authority_flag_rejects_before_promotion() {
    let mut cfg = stage_cfg(
        AUTHORITY_PROMOTION_STAGE_RECURSIVE_SHADOW_SIDECAR,
        &[POST_QUANTUM_ENFORCEMENT_STAGE],
        false,
    );
    cfg.authority_promotion.recursive_authority_allowed = true;

    let err = cfg
        .validate()
        .expect_err("recursive authority flag must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_candidate_requires_benchmark_evidence() {
    let mut cfg = stage_cfg(
        VERIFIED_BACKEND_CANDIDATE_STAGE,
        &[VERIFIED_BACKEND_ENABLED_STAGE],
        true,
    );
    cfg.verified_backend.benchmarks.pop();

    let err = cfg
        .validate()
        .expect_err("missing benchmark evidence must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_candidate_requires_statement_stability() {
    let mut cfg = stage_cfg(
        VERIFIED_BACKEND_CANDIDATE_STAGE,
        &[VERIFIED_BACKEND_ENABLED_STAGE],
        true,
    );
    cfg.verified_backend.statement_stability = "OtherStatementV1".to_string();

    let err = cfg.validate().expect_err("statement drift must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_candidate_requires_chain_evidence_bounds() {
    let mut cfg = stage_cfg(
        VERIFIED_BACKEND_CANDIDATE_STAGE,
        &[VERIFIED_BACKEND_ENABLED_STAGE],
        true,
    );
    cfg.verified_backend.chain_evidence.min_steps = 2;

    let err = cfg
        .validate()
        .expect_err("chain evidence below minimum must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_enabled_requires_review_and_rollback_contract() {
    let mut cfg = verified_enabled_cfg();
    cfg.verified_backend.security_review.status = "pending".to_string();

    let err = cfg
        .validate()
        .expect_err("enabled backend must require approved review");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    let mut cfg = verified_enabled_cfg();
    cfg.verified_backend.rollback.preserves_statement = false;

    let err = cfg
        .validate()
        .expect_err("enabled backend must preserve statement on rollback");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}
