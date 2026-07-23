use z00z_storage::checkpoint::{
    recursive_v2::{CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, RegistryLifecycleV2},
    CheckpointContractConfigV3,
};

#[test]
fn test_live_contract_uses_cadences() {
    let config = CheckpointContractConfigV3::load_repo_default().expect("live V3 config");
    let nova = &config.branches.nova;
    assert_eq!(nova.fold_cadence_blocks, 1);
    assert_eq!(nova.recovery_snapshot_cadence_blocks, 100);
    assert_eq!(nova.compression_cadence_blocks, 1_000);
    assert_eq!(nova.publication_cadence_blocks, 1_000);
    assert_eq!(nova.proof_system, "nova_streaming_compressed_v2");
    assert_eq!(nova.mode, "fast_classical_streaming_v2");

    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("pinned registry");
    for object in [
        RecursiveBoundedObjectV2::NovaBlockProof,
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
        RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
    ] {
        let row = registry.row(object).expect("registered live V2 object");
        assert_eq!(row.lifecycle, RegistryLifecycleV2::LiveReadWrite);
        assert_eq!(row.write_wire_version, Some(2));
        assert_eq!(row.read_wire_versions, &[2]);
    }
}

#[test]
fn test_crypto_ingress_v2_only() {
    let adapter = include_str!("../src/checkpoint/adapter.rs");
    let nova = include_str!("../src/checkpoint/nova.rs");
    let transition = include_str!("../src/checkpoint/canonical_transition.rs");
    let facade = include_str!("../src/checkpoint/recursive_v2.rs");
    let checkpoint_root = include_str!("../src/checkpoint/mod.rs");
    let final_artifact = include_str!("../src/checkpoint/artifact_final.rs");

    assert_eq!(
        adapter.matches("pub fn produce(").count(),
        1,
        "checkpoint crypto evidence must have one production ingress"
    );
    assert!(facade.contains("RecursiveCheckpointEvidenceStoreV2"));
    assert!(facade.contains("RecursiveCheckpointContextV2"));
    assert!(facade.contains("RecursiveCheckpointPublicInputV2"));
    assert!(facade.contains("RecursiveCheckpointRejectReasonV2"));
    assert!(facade.contains("NovaProofEnvelopeV2"));
    assert!(!facade.contains("RecursiveNovaStepInputV2"));
    let root_exports = checkpoint_root
        .split_once("pub use self::{")
        .expect("checkpoint root export block")
        .1;
    for recursive_type in [
        "RecursiveCheckpointEvidenceStoreV2",
        "RecursiveCheckpointPublicInputV2",
        "RecursiveCheckpointProofV2",
        "CryptographicVerificationReceiptV2",
        "CheckpointVersionRegistryV2",
    ] {
        assert!(
            !root_exports.contains(recursive_type),
            "recursive V2 type leaked through checkpoint root: {recursive_type}"
        );
    }
    assert!(!adapter.contains("verify_sidecar"));
    assert!(!adapter.contains("verifier_verdict"));
    assert!(!adapter.contains("accepted:"));
    assert!(adapter.contains("if blocks.is_empty()"));
    assert!(!adapter.contains("if blocks.len() < 3"));
    assert!(
        !adapter.contains("if blocks.len() > 5"),
        "the 1/3/5 milestone matrix must not become a production chain-length cap"
    );
    assert!(
        adapter.contains("retained_t3_artifact(\"verifier-bundle.bin\", 64 * 1024 * 1024)"),
        "the T3 milestone ingress must share the format-4 64-MiB bundle ceiling"
    );
    assert!(
        !adapter.contains("384 * 1024 * 1024"),
        "the retired format-3 verifier-bundle ceiling must not return"
    );
    assert!(nova.contains("Self::new_chain_scoped::<RequiredLocalChainEnvelopeV2>"));
    assert!(nova.contains("Self::load_chain_scoped::<RequiredLocalChainEnvelopeV2>"));
    assert!(nova.contains("self.verify_chain_scoped::<RequiredLocalChainEnvelopeV2>"));
    let diagnostic_decl = nova
        .find("struct DiagnosticSingleStepEnvelopeV2")
        .expect("diagnostic-only single-step envelope declaration");
    let diagnostic_cfg = nova[..diagnostic_decl]
        .rfind("#[cfg(test)]")
        .expect("diagnostic envelope is test-only");
    assert!(nova[diagnostic_cfg..diagnostic_decl]
        .lines()
        .all(|line| line.trim().is_empty() || line.trim_start().starts_with("#[")));
    let block_start = adapter
        .find("pub struct RecursiveCheckpointChainBlockV2")
        .expect("raw chain-block request");
    let block_end = adapter[block_start..]
        .find("pub struct RecursiveCheckpointEvidenceStoreV2")
        .map(|offset| block_start + offset)
        .expect("evidence store follows raw block request");
    assert!(
        !adapter[block_start..block_end].contains("CanonicalCheckpointTransitionV2"),
        "the public ingress must construct the bundle-bound transition itself"
    );
    assert!(
        adapter.find("resolve_cached_verifier_v2(").unwrap()
            < adapter
                .find("CanonicalCheckpointTransitionV2::from_exec_with_verifier(")
                .unwrap(),
        "cached strict material/bundle authority resolution must precede trace construction"
    );
    let resolver = nova
        .find("pub(crate) fn resolve_verifier_authority_v2(")
        .expect("one crate-private production authority resolver");
    let resolver_end = nova[resolver..]
        .find("pub(crate) struct NovaRunnerReadyV2")
        .map(|offset| resolver + offset)
        .expect("runner follows resolver");
    let resolver_body = &nova[resolver..resolver_end];
    assert!(resolver_body.contains("ACTIVE_VERIFIER_BUNDLE_DIGEST_V2"));
    assert!(
        resolver_body
            .find("expected_bundle_digest == [0; 32]")
            .unwrap()
            < resolver_body
                .find("ProverMaterialHeaderV2::decode")
                .unwrap(),
        "an unset artifact authority must reject before dependency decode"
    );
    assert!(
        resolver_body.find("prover_material_bytes.len() >").unwrap()
            < resolver_body
                .find("ProverMaterialHeaderV2::decode")
                .unwrap(),
        "the outer material cap must reject before fixed-header decode"
    );
    assert!(
        resolver_body.find("verifier_bundle_bytes.len() >").unwrap()
            < resolver_body
                .find("bundle_payload_digest(b\"verifier-bundle\"")
                .unwrap(),
        "the outer bundle cap must reject before hashing"
    );
    assert!(
        resolver_body
            .find("ProverMaterialHeaderV2::decode")
            .unwrap()
            < resolver_body.find("NovaProverMaterialV2::load").unwrap()
    );
    assert!(
        resolver_body.find("NovaProverMaterialV2::load").unwrap()
            < resolver_body.find("NovaVerifierBundleV2::load").unwrap()
    );
    assert!(
        resolver_body
            .find("bundle_digest != expected_bundle_digest")
            .unwrap()
            < resolver_body.find("NovaVerifierBundleV2::load").unwrap(),
        "the complete authority pin must match before verifier-key decode"
    );
    assert!(!transition.contains("#[cfg(test)]\n    pub(crate) fn from_exec_with_verifier("));
    assert!(final_artifact.contains("proof_sys.claims_verified()"));
    assert!(final_artifact.contains("Err(CheckpointError::ProofSysMix)"));
}
