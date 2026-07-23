#[test]
fn test_receipt_requires_postwrite_check() {
    let receipt = include_str!("../src/checkpoint/receipt.rs");
    let adapter = include_str!("../src/checkpoint/adapter.rs");
    let facade = include_str!("../src/checkpoint/recursive_v2.rs");
    let sidecar = include_str!("../src/checkpoint/sidecar.rs");
    let production = &adapter[..adapter
        .find("\n#[cfg(test)]\nmod tests {")
        .expect("adapter tests")];

    assert!(receipt.contains("pub struct CryptographicVerificationReceiptV2"));
    assert!(!receipt.contains("Deserialize"));
    assert!(!receipt.contains("accepted:"));
    assert!(!receipt.contains("verdict:"));
    assert_eq!(
        production.matches("postwrite.prepare_receipt()?").count(),
        1
    );
    let prepare = production
        .find("postwrite.prepare_receipt()?")
        .expect("pre-gate receipt validation");
    let issue = production[prepare..]
        .find("let receipt = ready.issue()?")
        .map(|offset| offset + prepare)
        .expect("final receipt gate");
    assert!(prepare < issue);
    assert!(!production.contains("self.receipts"));
    assert!(!production.contains("ensure_dir(\"receipts\")"));
    let return_tail = &production[issue..];
    let issuance_tail = &return_tail[..return_tail
        .find("Ok((reloaded_sidecar, sidecar_digest, receipt, receipt_digest))")
        .expect("infallible receipt issuance return")];
    assert!(!issuance_tail["let receipt = ready.issue()?".len()..].contains('?'));
    assert!(receipt.contains("issued: ReceiptIssuedPartsV2"));
    assert!(receipt.contains("pub(super) struct PreparedReceiptV2"));
    assert!(!receipt.contains("EncodedReceiptV2"));
    assert!(receipt.contains("_reloaded: ReloadedEvidenceV2"));
    assert!(adapter.contains("pub(super) struct ReloadedEvidenceV2"));
    assert!(adapter.contains("struct LiveGateStageV2<S>"));
    assert!(adapter.contains("pub(super) struct PostwriteVerifiedV2"));
    assert!(!adapter.contains("success_digest"));
    assert!(!adapter.contains("pub fn advance"));
    assert!(!adapter.contains("impl Clone for LiveGateStageV2"));
    assert!(!adapter.contains("impl Copy for LiveGateStageV2"));
    assert!(!adapter.contains("impl Default for LiveGateStageV2"));
    assert!(!receipt.contains("verified_reload"));
    let sidecar_schema = &sidecar[..sidecar
        .find("impl RecursiveCheckpointSidecarV2")
        .expect("sidecar implementation")];
    assert!(!sidecar_schema.contains("gate_trace_digest:"));
    assert!(!sidecar_schema.contains("proof_bytes:"));
    assert!(!sidecar_schema.contains("verifier_verdict:"));
    assert!(!sidecar_schema.contains("receipt:"));
    for exact_reference_field in [
        "version: u16",
        "mode: [u8; 27]",
        "backend_label: [u8; 28]",
        "statement_digest: [u8; 32]",
        "public_input_digest: [u8; 32]",
        "prior_output_root: [u8; 32]",
        "output_root: [u8; 32]",
        "verifier_bundle_digest: [u8; 32]",
        "envelope_digest: [u8; 32]",
        "envelope_byte_length: u64",
        "nova_retention_state_digest: [u8; 32]",
    ] {
        assert!(sidecar_schema.contains(exact_reference_field));
    }

    let edges = [
        "authority_resolved",
        "family_selected",
        "outer_bounded",
        "inner_bounded",
        "curve_valid",
        "bundle_matched",
        "backend_verified",
        "limbs_matched",
        "bindings_matched",
        "endpoint_reloaded",
        "prewrite_complete",
        "atomic_write",
        "bytes_reloaded",
        "post_backend_verified",
        "post_endpoint_matched",
    ];
    let mut tail = production;
    for edge in edges {
        let offset = tail
            .find(edge)
            .unwrap_or_else(|| panic!("missing consuming gate edge: {edge}"));
        tail = &tail[offset + edge.len()..];
    }
    assert!(adapter.contains("fn $method<T>(\n                self,"));
    assert!(adapter.contains(
        "fn issue_receipt(mut self) -> Result<LiveGateStageV2<ReceiptIssuedV2>, CheckpointError>"
    ));
    assert!(adapter.contains("fn new(\n        stage: LiveGateStageV2<PostEndpointReadyV2>"));
    assert!(!adapter.contains("pub(super) fn new(\n        stage: LiveGateStageV2"));
    assert!(facade.contains("CryptographicVerificationReceiptV2"));
}
