use z00z_storage::{
    checkpoint::{
        recursive_v2::{
            composed_history_error_exponent_v2, CheckpointVersionRegistryV2,
            HistoryAccumulatorInputsV2, HistoryAccumulatorStatementV2, HistoryBranchV2,
            HistoryRotationBridgeV2, HistoryRotationInputsV2, Plonky3HistoryAuthorityResolverV2,
            RecursiveCheckpointRejectReasonV2, RecursiveSecurityBudgetManifestV2,
        },
        CheckpointConfigResolverV3,
    },
    CheckpointError,
};

fn digest(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn history_inputs(branch: HistoryBranchV2) -> HistoryAccumulatorInputsV2 {
    let active = CheckpointConfigResolverV3::resolve_active().expect("active");
    let identity = active.identity();
    let cadence = active.config().branches.plonky3_epoch.cadence_blocks;
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned().expect("security");
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active().expect("authority");
    let authority_identity = authority.identity();
    let (last_epoch, history_length, predecessor) = match branch {
        HistoryBranchV2::Base => (0, 1, None),
        HistoryBranchV2::Successor => (1, 2, Some((digest(21), digest(22)))),
    };
    let cumulative_error_exponent = composed_history_error_exponent_v2(
        security.per_proof_error_exponent(),
        history_length,
        security
            .inherited_error_exponent()
            .expect("inherited bound"),
    )
    .expect("composition");
    HistoryAccumulatorInputsV2 {
        branch,
        first_epoch: 0,
        last_epoch,
        first_height: 1,
        last_height: (last_epoch + 1) * cadence,
        cadence_blocks: cadence,
        history_length,
        accepted_epoch_count: history_length,
        config_generation: identity.config_generation,
        authority_generation: identity.authority_generation,
        activation_height: identity.activation_height,
        rollback_floor: identity.rollback_floor,
        parameter_generation: security.parameter_generation(),
        runtime_profile_generation: identity.runtime_profile_generation,
        composition_rule_generation: security.composition_rule_generation(),
        per_proof_error_exponent: security.per_proof_error_exponent(),
        inherited_error_exponent: security
            .inherited_error_exponent()
            .expect("inherited bound"),
        cumulative_error_exponent,
        minimum_residual_bits: security.minimum_residual_bits(),
        chain_context_digest: digest(1),
        genesis_trust_anchor_digest: digest(2),
        genesis_state_root: digest(3),
        previous_terminal_state_root: if branch == HistoryBranchV2::Base {
            digest(3)
        } else {
            digest(4)
        },
        current_terminal_state_root: digest(5),
        previous_epoch_anchor_root: digest(6),
        current_epoch_anchor_root: digest(7),
        exact_epoch_statement_digest: digest(8),
        predicate_digest: digest(10),
        verifier_parameter_digest: authority_identity.verifier_parameter_digest,
        security_budget_digest: security.digest(),
        config_digest: identity.config_digest,
        registry_digest: CheckpointVersionRegistryV2::authority_pinned()
            .expect("registry")
            .digest(),
        runtime_profile_manifest_digest: identity.runtime_profile_manifest_digest,
        authority_bundle_digest: identity.history_authority_bundle_digest,
        verifier_bundle_digest: authority_identity.verifier_bundle_digest,
        epoch_anchor_mmr_root: digest(13),
        predecessor_statement_digest: predecessor.map(|pair| pair.0),
    }
}

#[test]
fn test_history_branches() {
    for branch in [HistoryBranchV2::Base, HistoryBranchV2::Successor] {
        let statement =
            HistoryAccumulatorStatementV2::new(history_inputs(branch)).expect("history");
        assert_eq!(statement.branch(), branch);
        assert_eq!(
            HistoryAccumulatorStatementV2::decode_canonical(statement.canonical_bytes())
                .expect("decode"),
            statement
        );
    }
}

#[test]
fn test_history_link_rollback() {
    let mut inputs = history_inputs(HistoryBranchV2::Successor);
    inputs.predecessor_statement_digest = None;
    assert!(matches!(
        HistoryAccumulatorStatementV2::new(inputs),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid
        ))
    ));

    let mut inputs = history_inputs(HistoryBranchV2::Successor);
    inputs.history_length = 1;
    assert!(matches!(
        HistoryAccumulatorStatementV2::new(inputs),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid
        ))
    ));
}

#[test]
fn test_security_reset_mutations() {
    for mutate in [
        |value: &mut HistoryAccumulatorInputsV2| value.accepted_epoch_count = 0,
        |value: &mut HistoryAccumulatorInputsV2| value.cumulative_error_exponent += 1,
        |value: &mut HistoryAccumulatorInputsV2| value.inherited_error_exponent += 1,
        |value: &mut HistoryAccumulatorInputsV2| value.minimum_residual_bits = 99,
    ] {
        let mut inputs = history_inputs(HistoryBranchV2::Successor);
        mutate(&mut inputs);
        assert!(matches!(
            HistoryAccumulatorStatementV2::new(inputs),
            Err(CheckpointError::RecursiveRejected(
                RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid
            ))
        ));
    }
}

fn rotation_inputs() -> HistoryRotationInputsV2 {
    let active = CheckpointConfigResolverV3::resolve_active().expect("active");
    let identity = active.identity();
    let cadence = active.config().branches.plonky3_epoch.cadence_blocks;
    let security = RecursiveSecurityBudgetManifestV2::authority_pinned().expect("security");
    let authority = Plonky3HistoryAuthorityResolverV2::resolve_active().expect("authority");
    let authority_identity = authority.identity();
    HistoryRotationInputsV2 {
        inherited_accepted_epoch_count: 1,
        activation_height: cadence + 1,
        first_new_epoch: 1,
        old_parameter_generation: security.parameter_generation() - 1,
        new_parameter_generation: security.parameter_generation(),
        inherited_error_exponent: security.lifetime_residual_bits(),
        new_per_proof_error_exponent: security.per_proof_error_exponent(),
        minimum_residual_bits: security.minimum_residual_bits(),
        chain_context_digest: digest(1),
        predicate_digest: digest(2),
        old_registry_digest: digest(3),
        new_registry_digest: identity.registry_digest,
        old_runtime_profile_manifest_digest: digest(4),
        new_runtime_profile_manifest_digest: identity.runtime_profile_manifest_digest,
        old_verifier_manifest_digest: digest(5),
        new_verifier_manifest_digest: authority_identity.verifier_parameter_digest,
        old_security_budget_digest: digest(7),
        new_security_budget_digest: security.digest(),
        old_history_statement_digest: digest(8),
        first_new_epoch_statement_digest: digest(10),
        old_terminal_state_root: digest(12),
        first_new_epoch_start_root: digest(12),
        previous_epoch_anchor_root: digest(13),
        new_epoch_anchor_root: digest(14),
        authority_rotation_commitment: authority.rotation_commitment(),
        new_config_digest: identity.config_digest,
        output_history_statement_digest: digest(16),
        old_authority_identity_digest: digest(17),
        new_authority_identity_digest: authority_identity.digest(),
    }
}

#[test]
fn test_rotation_authorities() {
    let bridge = HistoryRotationBridgeV2::new(rotation_inputs()).expect("bridge");
    assert_eq!(
        HistoryRotationBridgeV2::decode_canonical(bridge.canonical_bytes()).expect("decode"),
        bridge
    );

    let mut reset = rotation_inputs();
    reset.inherited_error_exponent = 0;
    assert!(matches!(
        HistoryRotationBridgeV2::new(reset),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid
        ))
    ));

    let mut unauthorized = rotation_inputs();
    unauthorized.authority_rotation_commitment[0] ^= 1;
    assert!(matches!(
        HistoryRotationBridgeV2::new(unauthorized),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3SecurityBudgetInvalid
        ))
    ));
}
