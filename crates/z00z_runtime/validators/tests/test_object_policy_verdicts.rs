#[path = "test_theorem_support.rs"]
mod theorem_support;

use std::collections::BTreeSet;

use z00z_aggregators::{
    BatchId, BatchPlanned, BatchRoute, IngressBoundary, ObjectWitnessBundleV1, OrderedBatch,
    PlanDigest, RightWitnessRefV1, RightWitnessStateV1, RuntimeObjectPackageV1, ShardId,
    WorkPayload,
};
use z00z_core::{
    actions::{
        ActionDescriptorV1, ActionPoolDescriptorV1, LifecycleEffectV1, RequiredSignatureV1,
        WitnessRequirementV1,
    },
    policies::{
        ConservationContributionV1, ExpiryRuleV1, PolicyDescriptorV1, ReplayProtectionV1,
        UnknownPolicyHandlingV1,
    },
    rights::{RightActionV1, RightRequirementV1, RightScopeV1},
    vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1},
    ObjectFamily,
};
use z00z_storage::{
    checkpoint::derive_checkpoint_id,
    settlement::{
        DefinitionId, FeeEnvelope, ObjectDeltaSetV1, PublicationRouteSnapshotV1, RightAction,
        RightActionCtx, RightClass, RightLeaf, SerialId, SettlementActionV1, SettlementLeaf,
        SettlementObjectDeltaV1, SettlementPath, TerminalId, TerminalLeaf, VoucherAction,
        VoucherActionCtx, VoucherBackingRef, VoucherLeaf,
    },
};
use z00z_validators::{
    ObjectPolicyRegistryV1, ObjectRejectCode, RejectClass, ResolvedBatch, SettlementTheoremBundle,
    ValidatorBoundary, VerdictKind,
};
use z00z_wallets::tx::{
    validator_mandate_lock_payload_commitment, validator_mandate_lock_unlock_ready,
};

#[test]
fn validator_accepts_known_object_package() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = voucher_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let package = package_template(
        &published.pub_in,
        policy.policy_id().expect("policy id").bytes(),
        action_pool.action_pool_id().expect("pool id").bytes(),
        action_id,
    );
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Accepted);
    assert!(verdict.reject.is_none());
    assert_eq!(verdict.object_verdicts.len(), 1);
    assert_eq!(verdict.object_verdicts[0].reject, None);
    assert!(verdict.publication.is_some());
}

#[test]
fn test_rejects_unknown_policy() {
    let boundary = ValidatorBoundary;
    let (policy, action_pool, action_id) = voucher_policy_contract();
    let (published, theorem) = published_artifact();
    let package = package_template(
        &published.pub_in,
        policy.policy_id().expect("policy id").bytes(),
        action_pool.action_pool_id().expect("pool id").bytes(),
        action_id,
    );
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &ObjectPolicyRegistryV1::default());

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::PolicyUnknown));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::UnknownPolicy)
    );
}

#[test]
fn test_rejects_missing_right() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = voucher_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let package = package_template(
        &published.pub_in,
        policy.policy_id().expect("policy id").bytes(),
        action_pool.action_pool_id().expect("pool id").bytes(),
        action_id,
    );
    let resolved = resolved_batch(
        published,
        theorem,
        RuntimeObjectPackageV1 {
            required_rights: vec![RightWitnessRefV1 {
                right_policy: "kyc_v1".to_string(),
                witness_state: RightWitnessStateV1::Missing,
            }],
            ..package
        },
        None,
    );

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::AuthInvalid));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::MissingRight)
    );
}

#[test]
fn validator_rejects_fee_boundary_violation() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = voucher_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let mut package = package_template(
        &published.pub_in,
        policy.policy_id().expect("policy id").bytes(),
        action_pool.action_pool_id().expect("pool id").bytes(),
        action_id,
    );
    package.fee_support_ref = Some([0xEE; 32]);
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::ProofInvalid));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::FeeBoundary)
    );
}

#[test]
fn test_rejects_bad_fee() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = voucher_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let mut package = package_template(
        &published.pub_in,
        policy.policy_id().expect("policy id").bytes(),
        action_pool.action_pool_id().expect("pool id").bytes(),
        action_id,
    );
    let support_ref = Some([0xEF; 32]);
    package.delta_set.fee_envelope = Some(FeeEnvelope {
        version: 1,
        payer_commitment: [0xA1; 32],
        sponsor_commitment: [0xA2; 32],
        budget_units: 5,
        budget_commitment: FeeEnvelope::budget_bind(6, support_ref),
        domain_id: [0xA3; 32],
        expires_at: 100,
        nonce: [0xA4; 32],
        transition_id: [0xA5; 32],
        replay_key: [0xA6; 32],
        support_ref,
        failure_policy_id: [0xA7; 32],
    });
    package.fee_support_ref = support_ref;
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::ProofInvalid));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::FeeBoundary)
    );
}

#[test]
fn test_rejects_locked_spend() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = validator_locked_asset_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let package = validator_locked_asset_package(
        &published.pub_in,
        &policy,
        &action_pool,
        action_id,
        0x51,
        RightWitnessStateV1::Missing,
        false,
        true,
    );
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::AuthInvalid));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::MissingRight)
    );
}

#[test]
fn test_accepts_unlock_after_expiry() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = validator_locked_asset_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let package = validator_locked_asset_package(
        &published.pub_in,
        &policy,
        &action_pool,
        action_id,
        0x52,
        RightWitnessStateV1::Present,
        true,
        true,
    );
    let resolved = resolved_batch(published, theorem, package.clone(), None);

    let lock_leaf = package
        .delta_set
        .deleted_objects
        .iter()
        .find_map(|delta| match delta.prior_leaf.as_ref() {
            Some(SettlementLeaf::Right(right)) => Some(*right),
            _ => None,
        })
        .expect("lock right leaf");
    assert!(validator_mandate_lock_unlock_ready(&lock_leaf, 101));
    lock_leaf
        .validate_action(
            RightAction::Expire,
            RightActionCtx {
                now: 101,
                ..RightActionCtx::default()
            },
            None,
        )
        .expect("unlock becomes eligible only after expiry");

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Accepted, "{verdict:?}");
    assert_eq!(verdict.object_verdicts[0].reject, None, "{verdict:?}");
}

#[test]
fn test_rejects_unlock_without_right() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = validator_locked_asset_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let package = validator_locked_asset_package(
        &published.pub_in,
        &policy,
        &action_pool,
        action_id,
        0x53,
        RightWitnessStateV1::Present,
        false,
        true,
    );
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::AuthInvalid));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::MissingRight)
    );
}

#[test]
fn test_rejects_unlock_replay() {
    let boundary = ValidatorBoundary;
    let mut registry = ObjectPolicyRegistryV1::default();
    let (policy, action_pool, action_id) = validator_locked_asset_policy_contract();
    let (published, theorem) = published_artifact();
    registry
        .register(policy.clone(), action_pool.clone())
        .expect("registry");
    let package = validator_locked_asset_package(
        &published.pub_in,
        &policy,
        &action_pool,
        action_id,
        0x54,
        RightWitnessStateV1::Present,
        true,
        false,
    );
    let resolved = resolved_batch(published, theorem, package, None);

    let verdict = boundary.verdict_for_batch(&resolved, &registry);

    assert_eq!(verdict.kind, VerdictKind::Rejected);
    assert_eq!(verdict.reject, Some(RejectClass::ReplayConflict));
    assert_eq!(
        verdict.object_verdicts[0].reject,
        Some(ObjectRejectCode::Replay)
    );
}

fn resolved_batch(
    published: z00z_aggregators::PublishedBatch,
    theorem: SettlementTheoremBundle,
    package: RuntimeObjectPackageV1,
    placement: Option<z00z_aggregators::ShardPlacementView>,
) -> ResolvedBatch {
    let item = IngressBoundary
        .normalize(WorkPayload::Tx(Box::new(theorem.tx_package().clone())))
        .expect("ingress normalize")
        .with_object_package(package);
    let batch_id = published.batch_id;
    ResolvedBatch::new(
        published,
        OrderedBatch {
            batch_id,
            items: vec![item],
            created_leaves: Vec::new(),
            planned: BatchPlanned {
                batch_id,
                route: BatchRoute {
                    shard_id: ShardId::new(1),
                    routing_generation: 7,
                },
                route_table_digest: PlanDigest::new([0x41; 32]),
                intake_ids: Vec::new(),
                op_count: 1,
                plan_digest: PlanDigest::new([0x51; 32]),
            },
        },
        theorem,
        Vec::new(),
        placement,
        None,
    )
}

fn published_artifact() -> (z00z_aggregators::PublishedBatch, SettlementTheoremBundle) {
    let theorem = theorem_support::theorem_bundle();
    let artifact = theorem.artifact();
    let checkpoint_id = derive_checkpoint_id(artifact).expect("checkpoint id");
    let batch_id = BatchId::from_bytes([0x29; 32]);
    (
        z00z_aggregators::PublishedBatch {
            batch_id,
            checkpoint_id,
            publication_checkpoint: 11,
            publication_route: PublicationRouteSnapshotV1::new(7, [0x41; 32], 10, vec![1]),
            pub_in: artifact.pub_in(),
            da_provider: "local-da".to_string(),
            blob_ref: "blob://typed-object".to_string(),
        },
        theorem,
    )
}

fn package_template(
    pub_in: &z00z_storage::checkpoint::CheckpointPubIn,
    policy_hash: [u8; 32],
    action_pool_id: [u8; 32],
    action_id: [u8; 32],
) -> RuntimeObjectPackageV1 {
    let voucher_path = SettlementPath::new(
        DefinitionId::new([0x11; 32]),
        SerialId::new(7),
        TerminalId::new([0x21; 32]),
    );
    let asset_path = SettlementPath::new(
        DefinitionId::new([0x11; 32]),
        SerialId::new(8),
        TerminalId::new([0x22; 32]),
    );
    let voucher = VoucherLeaf {
        version: 1,
        terminal_id: voucher_path.terminal_id,
        issuer_commitment: [0x01; 32],
        holder_commitment: [0x02; 32],
        beneficiary_commitment: [0x03; 32],
        refund_target_commitment: [0x04; 32],
        backing: VoucherBackingRef::ReserveCommitment([0x05; 32]),
        face_value: 25,
        remaining_value: 25,
        policy_id: policy_hash,
        action_pool_id,
        lifecycle: VoucherLifecycleV1::Active,
        validity: VoucherValidityWindowV1 {
            valid_from: 1,
            valid_until: 100,
        },
        receiver_must_accept: true,
        allow_reject: true,
        replay_nonce: [0x06; 32],
        disclosure_commitment: None,
        audit_commitment: None,
    };
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::Voucher(VoucherAction::RedeemFull),
        policy_hash,
        Some(VoucherActionCtx {
            now: 10,
            expected_holder: Some(voucher.holder_commitment),
            expected_beneficiary: Some(voucher.beneficiary_commitment),
            expected_refund_target: Some(voucher.refund_target_commitment),
            acceptance_confirmed: true,
            ..VoucherActionCtx::default()
        }),
        vec![SettlementObjectDeltaV1::deleted(
            voucher_path,
            SettlementLeaf::Voucher(voucher.clone()),
            Some(voucher.remaining_value),
        )],
        vec![SettlementObjectDeltaV1::created(
            asset_path,
            SettlementLeaf::Terminal(asset_leaf(asset_path)),
            Some(voucher.remaining_value),
        )],
        Vec::new(),
        None,
        pub_in.prev_settlement_root(),
        pub_in.new_settlement_root(),
    );

    RuntimeObjectPackageV1 {
        primary_family: ObjectFamily::Voucher,
        selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
        selected_action_id: action_id,
        policy_descriptor_hash: policy_hash,
        action_pool_id,
        required_rights: vec![RightWitnessRefV1 {
            right_policy: "kyc_v1".to_string(),
            witness_state: RightWitnessStateV1::Present,
        }],
        object_witnesses: ObjectWitnessBundleV1 {
            signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
            attestation_labels: BTreeSet::new(),
            has_acceptance_proof: true,
            has_replay_nonce: true,
            has_prior_root_binding: true,
            has_disclosure_commitment: false,
        },
        delta_set,
        fee_support_ref: None,
        prior_root: pub_in.prev_settlement_root(),
        expected_new_root: pub_in.new_settlement_root(),
    }
}

fn voucher_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32]) {
    let action = ActionDescriptorV1 {
        label: "voucher_redeem_full".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        lifecycle_effect: LifecycleEffectV1::Redeem,
        witness_requirements: BTreeSet::from([
            WitnessRequirementV1::Signature(RequiredSignatureV1::Holder),
            WitnessRequirementV1::AcceptanceProof,
            WitnessRequirementV1::ReplayNonce,
            WitnessRequirementV1::PriorStateRoot,
            WitnessRequirementV1::RightReference("kyc_v1".to_string()),
        ]),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "voucher_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "voucher_policy_v1".to_string(),
        domain_name: "z00z.runtime.validators.voucher_policy.v1".to_string(),
        primary_family: ObjectFamily::Voucher,
        allowed_input_families: BTreeSet::from([ObjectFamily::Voucher]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::from([RightRequirementV1 {
            right_policy: "kyc_v1".to_string(),
            allowed_actions: BTreeSet::from([RightActionV1::Use]),
            scope: RightScopeV1::ObjectFamily(ObjectFamily::Voucher),
            max_uses: Some(1),
            delegation_allowed: false,
            attenuation_only: true,
        }]),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Holder]),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::NonceAndRoot,
        conservation: ConservationContributionV1::ConditionalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn validator_locked_asset_policy_contract() -> (PolicyDescriptorV1, ActionPoolDescriptorV1, [u8; 32])
{
    let action = ActionDescriptorV1 {
        label: "validator_unlock_after_expiry".to_string(),
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        lifecycle_effect: LifecycleEffectV1::Expire,
        witness_requirements: BTreeSet::from([
            WitnessRequirementV1::Signature(RequiredSignatureV1::Controller),
            WitnessRequirementV1::RightReference("validator_mandate_lock_v1".to_string()),
            WitnessRequirementV1::ReplayNonce,
            WitnessRequirementV1::PriorStateRoot,
        ]),
        receiver_must_accept: false,
        preserves_beneficiary: true,
        preserves_refund_authority: true,
    };
    let action_id = action.action_id().expect("action id").bytes();
    let action_pool = ActionPoolDescriptorV1 {
        label: "validator_lock_pool_v1".to_string(),
        actions: BTreeSet::from([action]),
    };
    let policy = PolicyDescriptorV1 {
        label: "validator_lock_policy_v1".to_string(),
        domain_name: "z00z.runtime.validators.validator_lock_policy.v1".to_string(),
        primary_family: ObjectFamily::Asset,
        allowed_input_families: BTreeSet::from([ObjectFamily::Asset, ObjectFamily::Right]),
        allowed_output_families: BTreeSet::from([ObjectFamily::Asset]),
        action_pool_id: action_pool.action_pool_id().expect("pool id"),
        action_ids: action_pool.action_ids().expect("action ids"),
        conditions: BTreeSet::new(),
        required_rights: BTreeSet::from([RightRequirementV1 {
            right_policy: "validator_mandate_lock_v1".to_string(),
            allowed_actions: BTreeSet::from([RightActionV1::Use]),
            scope: RightScopeV1::ObjectFamily(ObjectFamily::Asset),
            max_uses: Some(1),
            delegation_allowed: false,
            attenuation_only: true,
        }]),
        required_signatures: BTreeSet::from([RequiredSignatureV1::Controller]),
        required_attestations: BTreeSet::new(),
        expiry_rule: ExpiryRuleV1::ValidUntil,
        replay_protection: ReplayProtectionV1::NonceAndRoot,
        conservation: ConservationContributionV1::FinalValue,
        unknown_policy_handling: UnknownPolicyHandlingV1::default(),
    };
    (policy, action_pool, action_id)
}

fn validator_lock_leaf(mark: u8, locked_asset_id: [u8; 32], locked_amount: u64) -> RightLeaf {
    let terminal_id = TerminalId::new([mark.wrapping_add(0x20); 32]);
    let mut leaf = RightLeaf {
        version: 1,
        terminal_id,
        right_class: RightClass::ValidatorMandate,
        issuer_scope: [mark; 32],
        provider_scope: [mark.wrapping_add(1); 32],
        holder_commitment: [mark.wrapping_add(2); 32],
        control_commitment: [mark.wrapping_add(3); 32],
        beneficiary_commitment: [mark.wrapping_add(4); 32],
        payload_commitment: [0u8; 32],
        valid_from: 1,
        valid_until: 100,
        challenge_from: 101,
        challenge_until: 140,
        use_nonce: [mark.wrapping_add(5); 32],
        revocation_policy_id: [mark.wrapping_add(6); 32],
        transition_policy_id: [mark.wrapping_add(7); 32],
        challenge_policy_id: [mark.wrapping_add(8); 32],
        disclosure_policy_id: [mark.wrapping_add(9); 32],
        retention_policy_id: [mark.wrapping_add(10); 32],
    };
    leaf.payload_commitment =
        validator_mandate_lock_payload_commitment(&locked_asset_id, locked_amount, &leaf);
    leaf
}

fn validator_locked_asset_package(
    pub_in: &z00z_storage::checkpoint::CheckpointPubIn,
    policy: &PolicyDescriptorV1,
    action_pool: &ActionPoolDescriptorV1,
    action_id: [u8; 32],
    mark: u8,
    witness_state: RightWitnessStateV1,
    include_right_delta: bool,
    has_replay_nonce: bool,
) -> RuntimeObjectPackageV1 {
    let locked_asset_path = asset_path(mark);
    let unlocked_asset_path = asset_path(mark.wrapping_add(1));
    let locked_amount = 25_u64;
    let policy_hash = policy.policy_id().expect("policy id").bytes();
    let action_pool_id = action_pool.action_pool_id().expect("pool id").bytes();
    let mut deleted_objects = vec![SettlementObjectDeltaV1::deleted(
        locked_asset_path,
        SettlementLeaf::Terminal(asset_leaf(locked_asset_path)),
        Some(locked_amount),
    )];
    if include_right_delta {
        let right_path = SettlementPath::new(
            DefinitionId::new([mark.wrapping_add(2); 32]),
            SerialId::new(u32::from(mark) + 100),
            TerminalId::new([mark.wrapping_add(3); 32]),
        );
        let mut right = validator_lock_leaf(
            mark,
            locked_asset_path.terminal_id.into_bytes(),
            locked_amount,
        );
        right.terminal_id = right_path.terminal_id;
        deleted_objects.push(SettlementObjectDeltaV1::deleted(
            right_path,
            SettlementLeaf::Right(right),
            None,
        ));
    }
    let delta_set = ObjectDeltaSetV1::new(
        SettlementActionV1::AssetMutation,
        policy_hash,
        None,
        deleted_objects,
        vec![SettlementObjectDeltaV1::created(
            unlocked_asset_path,
            SettlementLeaf::Terminal(asset_leaf(unlocked_asset_path)),
            Some(locked_amount),
        )],
        Vec::new(),
        None,
        pub_in.prev_settlement_root(),
        pub_in.new_settlement_root(),
    );

    RuntimeObjectPackageV1 {
        primary_family: ObjectFamily::Asset,
        selected_action: SettlementActionV1::AssetMutation,
        selected_action_id: action_id,
        policy_descriptor_hash: policy_hash,
        action_pool_id,
        required_rights: vec![RightWitnessRefV1 {
            right_policy: "validator_mandate_lock_v1".to_string(),
            witness_state,
        }],
        object_witnesses: ObjectWitnessBundleV1 {
            signatures: BTreeSet::from([RequiredSignatureV1::Controller]),
            attestation_labels: BTreeSet::new(),
            has_acceptance_proof: false,
            has_replay_nonce,
            has_prior_root_binding: true,
            has_disclosure_commitment: false,
        },
        delta_set,
        fee_support_ref: None,
        prior_root: pub_in.prev_settlement_root(),
        expected_new_root: pub_in.new_settlement_root(),
    }
}

fn asset_leaf(path: SettlementPath) -> TerminalLeaf {
    let mut leaf = TerminalLeaf::dummy_for_scan(path.serial_id.get());
    leaf.asset_id = path.terminal_id.into_bytes();
    leaf.serial_id = path.serial_id.get();
    leaf
}

fn asset_path(mark: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([mark.wrapping_add(0x10); 32]),
        SerialId::new(u32::from(mark) + 10),
        TerminalId::new([mark.wrapping_add(0x11); 32]),
    )
}
