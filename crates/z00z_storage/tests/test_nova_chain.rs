use z00z_storage::checkpoint::recursive_v2::{
    CheckpointVersionRegistryV2, NovaCadenceActionV2, NovaCadenceManifestV2, NovaCadenceRequestV2,
    NovaCompressionAuthorityV2, NovaCompressionPolicyV2, NovaEvidenceRoleV2, NovaRoleDeliveryV2,
    RecursiveBoundedObjectV2, RegistryFramingV2, RegistryLifecycleV2,
};

#[test]
fn test_cadence_authority() {
    let manifest = NovaCadenceManifestV2::authority_pinned();
    manifest.validate().expect("active cadence manifest");
    assert_eq!(manifest.fold_cadence_blocks(), 1);
    assert_eq!(manifest.recovery_snapshot_cadence_blocks(), 100);
    assert_eq!(manifest.compression_cadence_blocks(), 1_000);
    assert_eq!(manifest.publication_cadence_blocks(), 1_000);
    assert!(manifest.max_hot_recovery_bytes() > 0);

    let policy = NovaCompressionPolicyV2::authority_pinned().expect("active policy");
    assert_eq!(
        policy
            .action(
                1,
                NovaCompressionAuthorityV2::Scheduled,
                NovaCadenceRequestV2::Scheduled,
            )
            .expect("per-block fold"),
        NovaCadenceActionV2 {
            is_fold_required: true,
            is_recovery_snapshot_required: false,
            is_compression_required: false,
            is_publication_required: false,
        }
    );
    assert_eq!(
        policy
            .action(
                100,
                NovaCompressionAuthorityV2::Scheduled,
                NovaCadenceRequestV2::Scheduled,
            )
            .expect("recovery boundary"),
        NovaCadenceActionV2 {
            is_fold_required: true,
            is_recovery_snapshot_required: true,
            is_compression_required: false,
            is_publication_required: false,
        }
    );
    assert_eq!(
        policy
            .action(
                1_000,
                NovaCompressionAuthorityV2::Scheduled,
                NovaCadenceRequestV2::Scheduled,
            )
            .expect("publication boundary"),
        NovaCadenceActionV2 {
            is_fold_required: true,
            is_recovery_snapshot_required: true,
            is_compression_required: true,
            is_publication_required: true,
        }
    );
    let compress = policy
        .action(
            17,
            NovaCompressionAuthorityV2::LocalOperator,
            NovaCadenceRequestV2::Compress,
        )
        .expect("authorized compression request");
    assert!(
        compress.is_fold_required
            && compress.is_compression_required
            && !compress.is_publication_required
    );
    assert!(!compress.is_recovery_snapshot_required);
    let publish = policy
        .action(
            17,
            NovaCompressionAuthorityV2::RecoveryWorkflow,
            NovaCadenceRequestV2::Publish,
        )
        .expect("authorized publication request");
    assert!(
        publish.is_fold_required
            && !publish.is_compression_required
            && publish.is_publication_required
    );
    assert!(policy
        .action(
            17,
            NovaCompressionAuthorityV2::Peer,
            NovaCadenceRequestV2::Publish,
        )
        .is_err());
    assert!(policy
        .action(
            17,
            NovaCompressionAuthorityV2::Wallet,
            NovaCadenceRequestV2::Compress,
        )
        .is_err());
}

#[test]
fn test_cadence_registry_rows() {
    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("active registry");
    let cadence = registry
        .row(RecursiveBoundedObjectV2::NovaCadenceManifest)
        .expect("cadence row");
    let snapshot = registry
        .row(RecursiveBoundedObjectV2::NovaAccumulatorSnapshot)
        .expect("snapshot row");
    assert_eq!(cadence.framing, RegistryFramingV2::EmbeddedPreheader);
    assert_eq!(cadence.lifecycle, RegistryLifecycleV2::LiveReadWrite);
    assert!(cadence.reader_reachable && cadence.writer_reachable);
    assert_eq!(snapshot.framing, RegistryFramingV2::LocalTyped);
    assert_eq!(snapshot.lifecycle, RegistryLifecycleV2::LocalOnly);
    assert!(snapshot.reader_reachable && snapshot.writer_reachable);
    assert_ne!(cadence.object, snapshot.object);
    assert_ne!(cadence.cryptographic_domain, snapshot.cryptographic_domain);

    let bytes = NovaCadenceManifestV2::authority_pinned()
        .encode()
        .expect("registry-framed cadence manifest");
    assert!(registry
        .validate_preheader(&bytes, RecursiveBoundedObjectV2::NovaAccumulatorSnapshot)
        .is_err());
}

#[test]
fn test_ordinary_roles_empty() {
    let action = NovaCadenceActionV2 {
        is_fold_required: true,
        is_recovery_snapshot_required: true,
        is_compression_required: true,
        is_publication_required: true,
    };
    for role in [
        NovaEvidenceRoleV2::CanonicalValidator,
        NovaEvidenceRoleV2::Watcher,
        NovaEvidenceRoleV2::Wallet,
    ] {
        assert_eq!(
            NovaRoleDeliveryV2::for_action(role, action, false),
            NovaRoleDeliveryV2 {
                public_parameters_bytes: 0,
                prover_key_bytes: 0,
                verifier_key_fetches: 0,
                proof_envelope_fetches: 0,
            }
        );
    }

    assert_eq!(
        NovaRoleDeliveryV2::for_action(NovaEvidenceRoleV2::RecursiveVerifier, action, false),
        NovaRoleDeliveryV2 {
            public_parameters_bytes: 0,
            prover_key_bytes: 0,
            verifier_key_fetches: 1,
            proof_envelope_fetches: 1,
        }
    );
    assert_eq!(
        NovaRoleDeliveryV2::for_action(NovaEvidenceRoleV2::RecursiveVerifier, action, true),
        NovaRoleDeliveryV2 {
            public_parameters_bytes: 0,
            prover_key_bytes: 0,
            verifier_key_fetches: 0,
            proof_envelope_fetches: 1,
        }
    );
}
