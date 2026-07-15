use std::path::PathBuf;

use tempfile::NamedTempFile;
use z00z_storage::{
    checkpoint::{
        repo_default_path, CheckpointContractConfigV1, POST_QUANTUM_ENFORCEMENT_STAGE,
        VERIFIED_BACKEND_ENABLED_STAGE, VERIFIED_BACKEND_PROOF_OBJECT,
        VERIFIED_BACKEND_REQUIRED_BENCHMARKS, VERIFIED_BACKEND_REQUIRED_NEGATIVE_TESTS,
        VERIFIED_BACKEND_STATEMENT_STABILITY,
    },
    CheckpointError,
};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{read_to_string, write_file},
};

const REQUIRED_FIELDS: &[&str] = &[
    "height",
    "prev_root",
    "new_root",
    "prev_settlement_root",
    "new_settlement_root",
    "checkpoint_exec_input_id",
    "prep_snapshot_id",
    "tx_data_root",
    "delta_root",
    "witness_root",
    "journal_digest",
    "da_ref",
];

const OPTIONAL_FIELDS: &[&str] = &[
    "claim_root",
    "prior_recursive_output_root",
    "pq_anchor_root",
];

fn cfg() -> CheckpointContractConfigV1 {
    CheckpointContractConfigV1::load(repo_default_path()).expect("repo checkpoint contract")
}

fn repo_config_yaml() -> String {
    read_to_string(repo_default_path()).expect("repo checkpoint contract yaml")
}

fn load_temp_contract(yaml: &str) -> Result<CheckpointContractConfigV1, CheckpointError> {
    let file = NamedTempFile::new().expect("temp checkpoint contract yaml");
    write_file(file.path(), yaml.as_bytes()).expect("write temp checkpoint contract yaml");
    CheckpointContractConfigV1::load(file.path())
}

#[test]
fn test_statement_field_order_matches_phase068_contract() {
    let cfg = cfg();

    assert_eq!(
        cfg.statement.required_fields,
        REQUIRED_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        cfg.statement.optional_fields,
        OPTIONAL_FIELDS
            .iter()
            .map(|field| (*field).to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_streaming_branch_is_live_and_non_authoritative() {
    let cfg = cfg();

    assert!(cfg.branches.recursive.is_enabled);
    assert!(!cfg.branches.recursive.is_authoritative);
    assert_eq!(cfg.branches.recursive.mode, "streaming_transition_v2");
    assert_eq!(
        cfg.branches.recursive.proof_system,
        "nova_streaming_compressed_v2"
    );
    assert!(cfg.branches.recursive.has_prior_output_binding);
    assert_eq!(cfg.branches.recursive.min_chain_steps, 3);
    assert_eq!(cfg.branches.recursive.target_chain_steps, 5);
}

#[test]
fn test_path_strings_match_contract() {
    let cfg = cfg();

    assert_eq!(
        cfg.paths.checkpoint_artifacts,
        PathBuf::from("artifacts/checkpoints/final")
    );
    assert_eq!(
        cfg.paths.checkpoint_links,
        PathBuf::from("artifacts/checkpoints/links")
    );
    assert_eq!(
        cfg.paths.exec_inputs,
        PathBuf::from("artifacts/checkpoints/exec_input")
    );
    assert_eq!(
        cfg.paths.prep_snapshots,
        PathBuf::from("artifacts/checkpoints/prep_snapshot")
    );
    assert_eq!(
        cfg.paths.delta_journals,
        PathBuf::from("artifacts/checkpoints/delta_journal")
    );
    assert_eq!(
        cfg.paths.witness_archives,
        PathBuf::from("artifacts/checkpoints/witness_archive")
    );
    assert_eq!(
        cfg.paths.nova_block_proofs,
        PathBuf::from("artifacts/checkpoints/nova_block")
    );
    assert_eq!(
        cfg.paths.pq_checkpoints,
        PathBuf::from("artifacts/checkpoints/pq_anchor")
    );
    assert_eq!(
        cfg.paths.plonky3_epoch_proofs,
        PathBuf::from("artifacts/checkpoints/plonky3_epoch")
    );
    assert_eq!(
        cfg.paths.epoch_manifests,
        PathBuf::from("artifacts/checkpoints/epoch_manifest")
    );
    assert_eq!(
        cfg.paths.archive_manifests,
        PathBuf::from("artifacts/checkpoints/archive_manifest")
    );
    assert_eq!(
        cfg.paths.da_references,
        PathBuf::from("artifacts/checkpoints/da_reference")
    );
    assert_eq!(
        cfg.paths.publication_evidence,
        PathBuf::from("artifacts/checkpoints/publication_evidence")
    );
    assert_eq!(
        cfg.paths.checkpoint_lifecycles,
        PathBuf::from("artifacts/checkpoints/lifecycle")
    );
    assert_eq!(
        cfg.paths.state_snapshots,
        PathBuf::from("artifacts/checkpoints/state_snapshot")
    );
    assert_eq!(
        cfg.paths.retrieval_audits,
        PathBuf::from("artifacts/checkpoints/retrieval_audit")
    );
    assert_eq!(
        cfg.paths.archive_receipts,
        PathBuf::from("artifacts/checkpoints/archive_receipt")
    );
    assert_eq!(
        cfg.paths.da_exports,
        PathBuf::from("artifacts/da/checkpoints")
    );
    assert_eq!(
        cfg.paths.documentation_packets,
        PathBuf::from("artifacts/checkpoints/recursive_docs")
    );
}

#[test]
fn test_resolve_paths_keeps_contract() {
    let cfg = cfg();
    let root = PathBuf::from("/tmp/checkpoint-root");
    let resolved = cfg.resolve_paths(&root);

    for (actual, rel) in [
        (
            &resolved.checkpoint_artifacts,
            &cfg.paths.checkpoint_artifacts,
        ),
        (&resolved.checkpoint_links, &cfg.paths.checkpoint_links),
        (&resolved.exec_inputs, &cfg.paths.exec_inputs),
        (&resolved.prep_snapshots, &cfg.paths.prep_snapshots),
        (&resolved.delta_journals, &cfg.paths.delta_journals),
        (&resolved.witness_archives, &cfg.paths.witness_archives),
        (&resolved.nova_block_proofs, &cfg.paths.nova_block_proofs),
        (&resolved.pq_checkpoints, &cfg.paths.pq_checkpoints),
        (
            &resolved.plonky3_epoch_proofs,
            &cfg.paths.plonky3_epoch_proofs,
        ),
        (&resolved.epoch_manifests, &cfg.paths.epoch_manifests),
        (&resolved.archive_manifests, &cfg.paths.archive_manifests),
        (&resolved.da_references, &cfg.paths.da_references),
        (
            &resolved.publication_evidence,
            &cfg.paths.publication_evidence,
        ),
        (
            &resolved.checkpoint_lifecycles,
            &cfg.paths.checkpoint_lifecycles,
        ),
        (&resolved.state_snapshots, &cfg.paths.state_snapshots),
        (&resolved.retrieval_audits, &cfg.paths.retrieval_audits),
        (&resolved.archive_receipts, &cfg.paths.archive_receipts),
        (&resolved.da_exports, &cfg.paths.da_exports),
        (
            &resolved.documentation_packets,
            &cfg.paths.documentation_packets,
        ),
    ] {
        assert_eq!(actual, &root.join(rel));
    }

    assert_ne!(resolved.checkpoint_artifacts, resolved.prep_snapshots);
    assert_ne!(resolved.checkpoint_artifacts, resolved.pq_checkpoints);
    assert_ne!(resolved.checkpoint_artifacts, resolved.da_exports);
}

#[test]
fn test_verified_backend_surface_matches_phase068_contract() {
    let cfg = cfg();

    assert_eq!(
        cfg.verified_backend.proof_object,
        VERIFIED_BACKEND_PROOF_OBJECT
    );
    assert_eq!(
        cfg.verified_backend.statement_stability,
        VERIFIED_BACKEND_STATEMENT_STABILITY
    );
    assert_eq!(
        cfg.verified_backend
            .negative_tests
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        VERIFIED_BACKEND_REQUIRED_NEGATIVE_TESTS.to_vec()
    );
    assert_eq!(
        cfg.verified_backend
            .benchmarks
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        VERIFIED_BACKEND_REQUIRED_BENCHMARKS.to_vec()
    );
}

#[test]
fn test_unknown_yaml_key_rejects() {
    let mut yaml = repo_config_yaml();
    yaml.push_str("\nunknown_key: true\n");

    let err = load_temp_contract(&yaml).expect_err("unknown YAML key must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_missing_nova_profile_field_rejects() {
    let yaml = repo_config_yaml().replace("    proof_system: nova_streaming_compressed_v2\n", "");

    let err = load_temp_contract(&yaml).expect_err("missing nova profile field must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_missing_plonky3_profile_field_rejects() {
    let yaml = repo_config_yaml().replace("    field: koala_bear\n", "");

    let err = load_temp_contract(&yaml).expect_err("missing plonky3 profile field must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_nova_branch_is_live_and_disabled_state_rejects() {
    let mut cfg = cfg();
    cfg.validate()
        .expect("live nova branch configuration must validate");

    cfg.branches.nova.is_enabled = false;

    let err = cfg
        .validate()
        .expect_err("disabled live nova branch must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_plonky3_branch_enable_rejects_without_a_bound_backend() {
    let mut cfg = cfg();
    cfg.branches.plonky3_epoch.is_enabled = true;

    let err = cfg
        .validate()
        .expect_err("unimplemented plonky3 branch must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_parent_dir_path_rejects() {
    let mut cfg = cfg();
    cfg.paths.da_exports = PathBuf::from("artifacts/../escape");

    let err = cfg
        .validate()
        .expect_err("normalized relative path gate must reject parent dir");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_pq_anchor_limit_cannot_drop_below_plonky3_limit() {
    let mut cfg = cfg();
    cfg.limits.max_pq_anchor_bytes = cfg.limits.max_plonky3_epoch_proof_bytes - 1;

    let err = cfg
        .validate()
        .expect_err("pq anchor limit must dominate plonky3 proof limit");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_spec_stage_rejects_live_pq_cadence() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = "spec_only".to_string();
    cfg.authority_promotion.allowed_next_stages = vec![POST_QUANTUM_ENFORCEMENT_STAGE.to_string()];
    cfg.post_quantum.enforce_live_cadence = true;

    let err = cfg
        .validate()
        .expect_err("pre-pq stage must not claim live pq cadence");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_stage_skip_rejects() {
    let mut cfg = cfg();
    cfg.authority_promotion.allowed_next_stages = vec![POST_QUANTUM_ENFORCEMENT_STAGE.to_string()];

    let err = cfg.validate().expect_err("stage skip must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_verified_backend_stage_remains_unavailable_without_promotion_evidence() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = VERIFIED_BACKEND_ENABLED_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages.clear();
    cfg.authority_promotion.recursive_authority_allowed = true;
    cfg.authority_promotion.verified_backend_allowed = true;
    cfg.post_quantum.enforce_live_cadence = true;

    let err = cfg
        .validate()
        .expect_err("verified backend enablement must require approved review");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    cfg.branches.nova.is_available = true;
    cfg.branches.plonky3_epoch.is_available = true;
    let err = cfg
        .validate()
        .expect_err("verified backend stays unavailable without promotion evidence");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_pq_writer_stage_rejects_without_a_bound_writer() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = POST_QUANTUM_ENFORCEMENT_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages.clear();
    cfg.post_quantum.enforce_live_cadence = false;

    let err = cfg
        .validate()
        .expect_err("unbound PQ writer stage must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    cfg.post_quantum.enforce_live_cadence = true;
    let err = cfg
        .validate()
        .expect_err("unbound PQ writer stage must reject even with cadence enabled");
    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_v2_config_encoder_never_emits_legacy_pq_fields() {
    let cfg = cfg();
    let yaml = String::from_utf8(YamlCodec.serialize(&cfg).expect("encode V2 config"))
        .expect("V2 YAML utf8");

    assert!(yaml.contains("epoch_evidence_commitment: non_authenticating_digest_v2"));
    assert!(!yaml.contains("pq_signature_or_commitment"));
    assert!(!yaml.contains("is_pq_authoritative"));
    assert!(!yaml.contains("cadence_blocks: 0"));
    load_temp_contract(&yaml).expect("reencoded V2 config remains valid");
}

#[test]
fn test_v2_config_rejects_overflowed_object_caps_and_unavailable_promotion() {
    let mut overflow_cfg = cfg();
    overflow_cfg.limits.max_nova_block_proof_bytes = usize::MAX;
    assert!(matches!(
        overflow_cfg.validate(),
        Err(CheckpointError::ContractConfig(_))
    ));

    let mut cfg = cfg();
    cfg.authority_promotion.stage = VERIFIED_BACKEND_ENABLED_STAGE.to_string();
    cfg.authority_promotion.allowed_next_stages.clear();
    cfg.authority_promotion.recursive_authority_allowed = true;
    cfg.authority_promotion.verified_backend_allowed = true;
    cfg.post_quantum.enforce_live_cadence = true;
    assert!(matches!(
        cfg.validate(),
        Err(CheckpointError::ContractConfig(_))
    ));
}
