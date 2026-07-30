use z00z_storage::{
    checkpoint::{
        recursive_v2::{
            CheckpointVersionRegistryV2, EpochCadenceClassV2, EpochManifestInputsV2,
            EpochManifestV2, EpochRangeInputsV2, EpochRangeStatementV2,
            Plonky3HistoryAuthorityResolverV2, RecursiveCheckpointRejectReasonV2,
            RecursiveSecurityBudgetManifestV2,
        },
        CheckpointConfigResolverV3,
    },
    CheckpointError,
};

fn digest(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn range_inputs() -> EpochRangeInputsV2 {
    let active = CheckpointConfigResolverV3::resolve_active().expect("active config");
    let identity = active.identity();
    let cadence = active.config().branches.plonky3_epoch.cadence_blocks;
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned().expect("security");
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active().expect("authority");
    let history_identity = authority.identity();
    EpochRangeInputsV2 {
        cadence_class: EpochCadenceClassV2::Production,
        epoch_index: 0,
        start_height: 1,
        end_height: cadence,
        cadence_blocks: cadence,
        leaf_count: u32::try_from(cadence).expect("cadence fits"),
        parameter_generation: identity.parameter_generation,
        chain_context_digest: digest(1),
        predicate_digest: digest(2),
        parameter_digest: history_identity.verifier_parameter_digest,
        verifier_bundle_digest: history_identity.verifier_bundle_digest,
        security_budget_digest: security.digest(),
        config_digest: identity.config_digest,
        registry_digest: CheckpointVersionRegistryV2::authority_pinned()
            .expect("registry")
            .digest(),
        runtime_profile_manifest_digest: identity.runtime_profile_manifest_digest,
        frontier_authority_digest: digest(16),
        epoch_close_anchor_digest: digest(5),
        start_root: digest(6),
        end_root: digest(7),
        statement_digest_root: digest(8),
        checkpoint_artifact_root: digest(18),
        checkpoint_link_root: digest(9),
        delta_root: digest(10),
        witness_root: digest(11),
        challenge_content_root: digest(12),
        da_payload_commitment: digest(13),
        verified_base_proof_root: digest(14),
        recursive_base_proof_commitment: digest(17),
        nova_chain_root: Some(digest(15)),
    }
}

fn statement() -> EpochRangeStatementV2 {
    EpochRangeStatementV2::new(range_inputs()).expect("statement")
}

#[test]
fn test_epoch_range_roundtrip() {
    let statement = statement();
    assert_eq!(statement.start_height(), 1);
    assert_eq!(statement.end_height(), 2_000);
    assert_eq!(statement.leaf_count(), 2_000);
    assert!(statement.is_production_cadence());
    assert_eq!(
        EpochRangeStatementV2::decode_canonical(statement.canonical_bytes()).expect("decode"),
        statement
    );
}

#[test]
fn test_range_mutations_reject() {
    for mutate in [
        |value: &mut EpochRangeInputsV2| value.end_height -= 1,
        |value: &mut EpochRangeInputsV2| value.leaf_count -= 1,
        |value: &mut EpochRangeInputsV2| value.statement_digest_root = [0; 32],
        |value: &mut EpochRangeInputsV2| value.checkpoint_link_root = [0; 32],
        |value: &mut EpochRangeInputsV2| value.verified_base_proof_root = [0; 32],
        |value: &mut EpochRangeInputsV2| value.frontier_authority_digest = [0; 32],
        |value: &mut EpochRangeInputsV2| value.recursive_base_proof_commitment = [0; 32],
    ] {
        let mut inputs = range_inputs();
        mutate(&mut inputs);
        assert!(matches!(
            EpochRangeStatementV2::new(inputs),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3CanonicalRangeMissing
            ))
        ));
    }
}

#[test]
fn test_manifest_root_separation() {
    let statement = statement();
    let inputs = EpochManifestInputsV2 {
        checkpoint_artifact_root: statement.inputs().checkpoint_artifact_root,
        archive_availability_manifest_root: digest(32),
    };
    EpochManifestV2::preflight_availability(&statement, inputs).expect("manifest preflight");
    assert_ne!(
        inputs.checkpoint_artifact_root,
        inputs.archive_availability_manifest_root
    );
}

#[test]
fn test_manifest_binding_rejects() {
    let statement = statement();
    let base = EpochManifestInputsV2 {
        checkpoint_artifact_root: statement.inputs().checkpoint_artifact_root,
        archive_availability_manifest_root: digest(32),
    };
    for mutate in [
        |value: &mut EpochManifestInputsV2| value.checkpoint_artifact_root = [0; 32],
        |value: &mut EpochManifestInputsV2| value.checkpoint_artifact_root[0] ^= 1,
        |value: &mut EpochManifestInputsV2| value.archive_availability_manifest_root = [0; 32],
    ] {
        let mut inputs = base;
        mutate(&mut inputs);
        assert!(matches!(
            EpochManifestV2::preflight_availability(&statement, inputs),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::EpochManifestIncomplete
            ))
        ));
    }
}

#[test]
fn test_manifest_size_policy() {
    assert_eq!(
        EpochManifestV2::classify_encoded_len(2 * 1024 * 1024)
            .expect("target")
            .name(),
        "within_target"
    );
    assert_eq!(
        EpochManifestV2::classify_encoded_len(2 * 1024 * 1024 + 1)
            .expect("publishable target miss")
            .name(),
        "target_missed"
    );
    assert!(matches!(
        EpochManifestV2::classify_encoded_len(4 * 1024 * 1024 + 1),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::ProofSizeBudgetExceeded
        ))
    ));
}
