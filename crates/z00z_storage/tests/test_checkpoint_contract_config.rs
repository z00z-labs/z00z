use std::path::PathBuf;

use tempfile::NamedTempFile;
use z00z_storage::{
    checkpoint::{repo_default_path, CheckpointContractConfigV3},
    CheckpointError,
};
use z00z_utils::io::{read_to_string, write_file};

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

fn cfg() -> CheckpointContractConfigV3 {
    CheckpointContractConfigV3::load(repo_default_path()).expect("repo checkpoint contract")
}

fn repo_config_yaml() -> String {
    read_to_string(repo_default_path()).expect("repo checkpoint contract yaml")
}

fn load_temp_contract(yaml: &str) -> Result<CheckpointContractConfigV3, CheckpointError> {
    let file = NamedTempFile::new().expect("temp checkpoint contract yaml");
    write_file(file.path(), yaml.as_bytes()).expect("write temp checkpoint contract yaml");
    CheckpointContractConfigV3::load(file.path())
}

#[test]
fn test_statement_schema_and_forbidden_admission_are_exact() {
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
    assert_eq!(
        cfg.statement.canonical_admission_forbidden_when_present,
        ["pq_anchor_root".to_string()]
    );
}

#[test]
fn test_recursive_authority_profile_is_the_live_schema3_target() {
    let cfg = cfg();

    assert!(cfg.branches.recursive.is_enabled);
    assert!(!cfg.branches.recursive.is_authoritative);
    assert_eq!(cfg.branches.recursive.mode, "hybrid_nova_plonky3");
    assert_eq!(cfg.branches.recursive.proof_system, "recursive_hybrid_v2");
    assert!(cfg.branches.recursive.has_prior_output_binding);
    assert_eq!(cfg.branches.recursive.min_chain_steps, 3);
    assert_eq!(cfg.branches.recursive.target_chain_steps, 5);

    assert!(cfg.branches.nova.is_enabled);
    assert_eq!(cfg.branches.nova.hot_recovery_snapshot_count, 2);
    assert_eq!(cfg.branches.nova.max_retained_bodies_per_epoch, 2);
    assert_eq!(cfg.branches.nova.max_pending_pq_epochs, 8);
    assert_eq!(cfg.branches.nova.post_pq_grace_certified_epochs, 2);
    assert_eq!(
        cfg.branches.nova.pending_pq_cap_action,
        "stop_compression_publication_keep_fold_finality"
    );
    assert_eq!(
        cfg.branches.nova.proof_body_retention,
        "nova_retention_state_v2"
    );

    assert!(cfg.branches.plonky3_epoch.is_enabled);
    assert!(cfg.branches.plonky3_epoch.has_security_budget_manifest);
    assert_eq!(
        cfg.branches.plonky3_epoch.soundness_composition,
        "conservative_sum"
    );
    assert_eq!(
        cfg.branches.plonky3_epoch.minimum_composed_security_bits,
        100
    );
    assert!(cfg.branches.plonky3_epoch.has_history_successor);
    assert!(cfg.branches.plonky3_epoch.has_rotation_bridge);
}

#[test]
fn test_archive_retention_and_caps_match_the_authority() {
    let cfg = cfg();

    assert_eq!(cfg.archive_retention.reconstruction_threshold, 10);
    assert_eq!(cfg.archive_retention.total_shards, 16);
    assert_eq!(cfg.archive_retention.max_shards_per_failure_domain, 2);
    assert_eq!(cfg.archive_retention.erasure_coding_profile, "rs_10_16_v1");
    assert!(!cfg.archive_retention.is_full_replica_fallback_allowed);

    assert_eq!(cfg.limits.max_recursive_proof_envelope_bytes, 17_825_792);
    assert_eq!(cfg.limits.max_recursive_sidecar_bytes, 25_165_824);
    assert_eq!(cfg.limits.max_nova_block_proof_bytes, 131_072);
    assert_eq!(cfg.limits.max_nova_retained_proof_bodies, 16);
    assert_eq!(cfg.limits.max_nova_retained_body_bytes, 2_097_152);
    assert_eq!(cfg.limits.max_nova_hot_recovery_bytes, 0);
    assert_eq!(cfg.limits.max_pq_anchor_bytes, 4_096);
    assert_eq!(cfg.limits.max_epoch_anchor_bytes, 4_096);
    assert_eq!(cfg.limits.max_epoch_close_certificate_bytes, 4_096);
}

#[test]
fn test_retention_and_pruning_authority_is_explicit() {
    let cfg = cfg();

    assert_eq!(cfg.pruning.stage, "declared_only");
    assert_eq!(cfg.pruning.activation_scope, "local_test_only");
    assert_eq!(cfg.pruning.watcher_mode, "window_archive_watcher_v2");
    assert_eq!(cfg.pruning.prune_scope, "expired_challenge_objects");
    assert!(cfg.pruning.has_retention_ledger_cas);
    assert_eq!(cfg.pruning.must_keep_latest_snapshot_generations, 3);

    assert_eq!(
        cfg.retention.finalized_checkpoint_record_bodies,
        "challenge_window_plus_current_head"
    );
    assert_eq!(
        cfg.retention.nova_block_proofs,
        "bounded_nova_retention_state_v2"
    );
    assert_eq!(
        cfg.retention.compact_epoch_history_rotation_anchors,
        "permanent"
    );
    assert_eq!(cfg.retention.epoch_close_finality_certificates, "permanent");
}

#[test]
fn test_path_strings_match_contract() {
    let cfg = cfg();

    for (actual, expected) in [
        (
            &cfg.paths.checkpoint_artifacts,
            "artifacts/checkpoints/final",
        ),
        (
            &cfg.paths.recursive_sidecars,
            "artifacts/checkpoints/recursive_shadow",
        ),
        (
            &cfg.paths.nova_block_proofs,
            "artifacts/checkpoints/nova_block",
        ),
        (
            &cfg.paths.epoch_close_anchors,
            "artifacts/checkpoints/epoch_close_anchor",
        ),
        (
            &cfg.paths.epoch_evidence_anchors,
            "artifacts/checkpoints/epoch_evidence_anchor",
        ),
        (
            &cfg.paths.retention_tickets,
            "artifacts/checkpoints/retention_ticket",
        ),
        (
            &cfg.paths.retention_ledger,
            "artifacts/checkpoints/retention_ledger",
        ),
        (
            &cfg.paths.history_proofs,
            "artifacts/checkpoints/plonky3_history",
        ),
        (
            &cfg.paths.history_rotation_bridges,
            "artifacts/checkpoints/history_rotation_bridge",
        ),
    ] {
        assert_eq!(actual, &PathBuf::from(expected));
    }
}

#[test]
fn test_resolve_paths_keeps_every_live_target_beneath_the_root() {
    let cfg = cfg();
    let root = PathBuf::from("/tmp/checkpoint-root");
    let resolved = cfg.resolve_paths(&root);

    for actual in [
        &resolved.checkpoint_artifacts,
        &resolved.recursive_sidecars,
        &resolved.nova_block_proofs,
        &resolved.epoch_close_anchors,
        &resolved.epoch_evidence_anchors,
        &resolved.retention_tickets,
        &resolved.retention_ledger,
        &resolved.history_proofs,
        &resolved.history_rotation_bridges,
        &resolved.documentation_packets,
    ] {
        assert!(actual.starts_with(&root));
    }
}

#[test]
fn test_schema3_yaml_has_one_canonical_vocabulary() {
    let yaml = repo_config_yaml();

    for required in [
        "canonical_admission_forbidden_when_present:",
        "is_recursive_authority_allowed:",
        "is_verified_backend_allowed:",
        "is_live_cadence_enforced:",
        "has_statement_digest_bind:",
        "has_checkpoint_link_bind:",
        "has_transition_range_proof:",
        "has_independent_transition_proof:",
        "has_retention_ledger_cas:",
    ] {
        assert!(
            yaml.contains(required),
            "missing canonical field {required}"
        );
    }
    for forbidden in [
        "recursive_authority_allowed:",
        "verified_backend_allowed:",
        "enforce_live_cadence:",
        "must_bind_statement_digest:",
        "must_bind_checkpoint_link:",
        "must_prove_canonical_transition_range:",
        "must_not_depend_only_on_nova:",
        "\nverified_backend:",
        "\n    no_op:",
    ] {
        assert!(
            !yaml
                .lines()
                .any(|line| line.trim_start().starts_with(forbidden.trim_start())),
            "legacy vocabulary is forbidden: {forbidden}"
        );
    }
}

#[test]
fn test_unknown_and_missing_yaml_fields_reject() {
    let yaml = repo_config_yaml();
    let unknown = format!("{yaml}unknown_key: true\n");
    assert!(matches!(
        load_temp_contract(&unknown),
        Err(CheckpointError::ContractConfig(_))
    ));

    for field in [
        "    hot_recovery_snapshot_count: 2\n",
        "    has_security_budget_manifest: true\n",
        "  canonical_admission_forbidden_when_present:\n",
        "  reconstruction_threshold: 10\n",
        "  max_recursive_proof_envelope_bytes: 17825792\n",
        "  has_retention_ledger_cas: true\n",
    ] {
        let changed = yaml.replacen(field, "", 1);
        assert_ne!(changed, yaml, "fixture must contain {field:?}");
        assert!(
            load_temp_contract(&changed).is_err(),
            "missing mandatory field must reject: {field:?}"
        );
    }
}

#[test]
fn test_every_pinned_identity_axis_rejects_drift() {
    let yaml = repo_config_yaml();

    for (from, to) in [
        (
            "  identifier: checkpoint-contract-client-notary-v2\n",
            "  identifier: alternate\n",
        ),
        ("  generation: 2\n", "  generation: 3\n"),
        (
            "  manifest_digest: c58e3b8341626573f956b1a9db13b30bc3b3ef33f71bff63ff1e080e9d78e71b\n",
            "  manifest_digest: a58e3b8341626573f956b1a9db13b30bc3b3ef33f71bff63ff1e080e9d78e71b\n",
        ),
        (
            "  registry_digest: 4007f54c2b3d714aba7e5d8a86dbedf84e5946c9f1c314d1fb11c831af5000fb\n",
            "  registry_digest: a007f54c2b3d714aba7e5d8a86dbedf84e5946c9f1c314d1fb11c831af5000fb\n",
        ),
        ("  parameter_generation: 2\n", "  parameter_generation: 3\n"),
    ] {
        let changed = yaml.replacen(from, to, 1);
        assert_ne!(changed, yaml, "fixture must contain {from:?}");
        assert!(
            load_temp_contract(&changed).is_err(),
            "identity drift must reject: {from:?}"
        );
    }
}

#[test]
fn test_target_authority_leaf_drift_rejects() {
    let yaml = repo_config_yaml();

    for (from, to) in [
        (
            "    mode: hybrid_nova_plonky3\n",
            "    mode: streaming_transition_v2\n",
        ),
        (
            "    proof_system: recursive_hybrid_v2\n",
            "    proof_system: nova_streaming_compressed_v2\n",
        ),
        (
            "  reconstruction_threshold: 10\n",
            "  reconstruction_threshold: 9\n",
        ),
        ("  total_shards: 16\n", "  total_shards: 15\n"),
        (
            "  erasure_coding_profile: rs_10_16_v1\n",
            "  erasure_coding_profile: replica_v1\n",
        ),
        (
            "  max_pq_anchor_bytes: 4096\n",
            "  max_pq_anchor_bytes: 4097\n",
        ),
        ("  stage: declared_only\n", "  stage: live\n"),
    ] {
        let changed = yaml.replacen(from, to, 1);
        assert_ne!(changed, yaml, "fixture must contain {from:?}");
        assert!(
            load_temp_contract(&changed).is_err(),
            "authority drift must reject: {from:?}"
        );
    }
}

#[test]
fn test_normalized_sharding_and_mailbox_authority_are_live_schema() {
    let cfg = cfg();
    let sharding = &cfg.current_state_sharding;
    assert!(sharding.is_required);
    assert_eq!(sharding.shard_count, 16);
    assert_eq!(sharding.replication_factor, 3);
    assert_eq!(sharding.write_quorum, 2);
    assert_eq!(sharding.read_quorum, 1);
    assert_eq!(sharding.min_failure_domains, 4);
    assert!(!sharding.is_full_state_replica_allowed);
    assert_eq!(sharding.route_key_profile, "hjmt_terminal_hash_range_v1");
    assert_eq!(sharding.rollout_profile, "cow_copy_delta_catchup_cas_v1");
    assert!(sharding.has_seed_recovery);

    let mailbox = &cfg.offline_receipt_mailbox;
    assert!(mailbox.is_required);
    assert!(!mailbox.is_runtime_enabled);
    assert_eq!(mailbox.semantic_owner_phase, 71);
    assert_eq!(mailbox.phase_069_role, "reserved_unreachable_handoff");
    assert_eq!(mailbox.admission_stage, "declared_only");
    assert_eq!(mailbox.max_admission_bytes_per_block, 0);
    assert_eq!(mailbox.max_partition_block_admission_bytes, 0);
    assert_eq!(mailbox.max_entry_bytes, 8_192);
    assert_eq!(mailbox.logical_partition_count, 16);
    assert!(!mailbox.is_cross_partition_fanout_allowed);
    assert!(!mailbox.has_adversarial_uniformity_claim);
    assert!(!mailbox.is_sender_ack_retention_required);
}

#[test]
fn test_canonical_digest_is_nonzero_and_stable() {
    let cfg = cfg();
    let first = cfg.canonical_digest().expect("canonical ConfigV3 digest");
    let second = cfg.canonical_digest().expect("canonical ConfigV3 digest");

    assert_ne!(first, [0; 32]);
    assert_eq!(first, second);
}
