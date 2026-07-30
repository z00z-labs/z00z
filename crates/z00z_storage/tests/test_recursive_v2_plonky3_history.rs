use z00z_storage::{
    checkpoint::{
        recursive_v2::{
            composed_history_error_exponent_v2, CheckpointVersionRegistryV2,
            HistoryAccumulatorInputsV2, HistoryAccumulatorStatementV2, HistoryBranchV2,
            Plonky3HistoryAuthorityResolverV2, Plonky3HistoryProofV2, RecursiveBoundedObjectV2,
            RecursiveCheckpointRejectReasonV2, RecursiveSecurityBudgetManifestV2,
        },
        CheckpointConfigResolverV3,
    },
    CheckpointError,
};

fn digest(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn base_statement() -> HistoryAccumulatorStatementV2 {
    let active = CheckpointConfigResolverV3::resolve_active().expect("active");
    let config_identity = active.identity();
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active().expect("authority");
    let identity = authority.identity();
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned().expect("security");
    let cadence = active.config().branches.plonky3_epoch.cadence_blocks;
    let inherited = security
        .inherited_error_exponent()
        .expect("inherited bound");
    HistoryAccumulatorStatementV2::new(HistoryAccumulatorInputsV2 {
        branch: HistoryBranchV2::Base,
        first_epoch: 0,
        last_epoch: 0,
        first_height: 1,
        last_height: cadence,
        cadence_blocks: cadence,
        history_length: 1,
        accepted_epoch_count: 1,
        config_generation: config_identity.config_generation,
        authority_generation: config_identity.authority_generation,
        activation_height: config_identity.activation_height,
        rollback_floor: config_identity.rollback_floor,
        parameter_generation: identity.parameter_generation,
        runtime_profile_generation: config_identity.runtime_profile_generation,
        composition_rule_generation: security.composition_rule_generation(),
        per_proof_error_exponent: security.per_proof_error_exponent(),
        inherited_error_exponent: inherited,
        cumulative_error_exponent: composed_history_error_exponent_v2(
            security.per_proof_error_exponent(),
            1,
            inherited,
        )
        .expect("composition"),
        minimum_residual_bits: security.minimum_residual_bits(),
        chain_context_digest: digest(1),
        genesis_trust_anchor_digest: digest(2),
        genesis_state_root: digest(3),
        previous_terminal_state_root: digest(3),
        current_terminal_state_root: digest(4),
        previous_epoch_anchor_root: digest(5),
        current_epoch_anchor_root: digest(6),
        exact_epoch_statement_digest: digest(7),
        predicate_digest: digest(8),
        verifier_parameter_digest: identity.verifier_parameter_digest,
        security_budget_digest: identity.security_budget_digest,
        config_digest: config_identity.config_digest,
        registry_digest: config_identity.registry_digest,
        runtime_profile_manifest_digest: config_identity.runtime_profile_manifest_digest,
        authority_bundle_digest: config_identity.history_authority_bundle_digest,
        verifier_bundle_digest: identity.verifier_bundle_digest,
        epoch_anchor_mmr_root: digest(9),
        predecessor_statement_digest: None,
    })
    .expect("statement")
}

#[test]
fn test_history_authority_reload() {
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active().expect("authority");
    let statement = base_statement();
    assert_eq!(statement.authority_identity(), authority.identity());
    assert_eq!(
        HistoryAccumulatorStatementV2::decode_canonical(statement.canonical_bytes())
            .expect("decode"),
        statement
    );
}

#[test]
fn test_mixed_bundle_preallocation() {
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active().expect("authority");
    let statement = base_statement();
    let inputs = statement.inputs();
    let mut payload = Vec::new();
    payload.extend_from_slice(b"Z00ZHPR2");
    payload.extend_from_slice(&2u16.to_le_bytes());
    payload.push(1);
    payload.extend_from_slice(
        &u32::try_from(statement.canonical_bytes().len())
            .expect("statement length")
            .to_le_bytes(),
    );
    payload.extend_from_slice(statement.canonical_bytes());
    payload.push(0);
    payload.extend_from_slice(&0u32.to_le_bytes());
    for value in [
        statement.digest(),
        [0; 32],
        inputs.verifier_parameter_digest,
        inputs.security_budget_digest,
        digest(0xa1),
        digest(0xa2),
        digest(0xa3),
    ] {
        payload.extend_from_slice(&value);
    }
    payload.extend_from_slice(&1u32.to_le_bytes());
    payload.push(0xff);
    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("registry");
    let preheader = registry
        .encode_preheader(RecursiveBoundedObjectV2::Plonky3HistoryProof, payload.len())
        .expect("preheader");
    let mut envelope = preheader.to_vec();
    envelope.extend_from_slice(&payload);

    assert!(matches!(
        Plonky3HistoryProofV2::decode_local_with_authority(&envelope, &authority),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3TranscriptMismatch
        ))
    ));
}
