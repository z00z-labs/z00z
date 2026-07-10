use serde_json::{json, Value};
use z00z_storage::{
    checkpoint::{
        repo_default_path, CheckpointContractConfigV1, CheckpointExecInputId, CheckpointId,
        CheckpointLink, CheckpointLinkVersion, CheckpointPubIn,
        CheckpointTransitionStatementCoreV1, CheckpointTransitionStatementFinalV1,
        CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt,
        RecursiveCheckpointChainEvidenceV1, RecursiveCheckpointMeasurementV1,
        RecursiveCheckpointModeV1, RecursiveCheckpointProofFamilyV1, RecursiveCheckpointProofV1,
        RecursiveCheckpointRejectReasonV1, RecursiveCheckpointSidecarV1,
        RecursiveCheckpointVerifierV1, SpentEnt,
    },
    settlement::SettlementStateRoot,
    snapshot::PrepSnapshotId,
    CheckpointError,
};

fn cfg() -> CheckpointContractConfigV1 {
    CheckpointContractConfigV1::load(repo_default_path()).expect("repo checkpoint contract")
}

fn verifier() -> RecursiveCheckpointVerifierV1 {
    RecursiveCheckpointVerifierV1::new(&cfg()).expect("recursive verifier")
}

fn backend_label(mode: RecursiveCheckpointModeV1) -> &'static str {
    match mode {
        RecursiveCheckpointModeV1::FastClassicalCompressed => "nova_compressed_v1",
        RecursiveCheckpointModeV1::PqEpochFinality => "plonky3_stark_epoch_v1",
    }
}

fn proof_family(mode: RecursiveCheckpointModeV1) -> RecursiveCheckpointProofFamilyV1 {
    match mode {
        RecursiveCheckpointModeV1::FastClassicalCompressed => {
            RecursiveCheckpointProofFamilyV1::Nova
        }
        RecursiveCheckpointModeV1::PqEpochFinality => RecursiveCheckpointProofFamilyV1::Stark,
    }
}

fn security_bits(mode: RecursiveCheckpointModeV1) -> u16 {
    match mode {
        RecursiveCheckpointModeV1::FastClassicalCompressed => 0,
        RecursiveCheckpointModeV1::PqEpochFinality => 124,
    }
}

fn statement(
    mark: u8,
    height: u64,
    prev_root: [u8; 32],
    new_root: [u8; 32],
) -> CheckpointTransitionStatementV1 {
    let pub_in = CheckpointPubIn::new_settlement(
        SettlementStateRoot::settlement_v1(prev_root),
        SettlementStateRoot::settlement_v1(new_root),
        vec![SpentEnt::new([mark.wrapping_add(1); 32])],
        vec![CreatedEnt::new(
            [mark.wrapping_add(2); 32],
            [mark.wrapping_add(3); 32],
        )],
    );
    CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        height,
        pub_in,
        PrepSnapshotId::new([mark.wrapping_add(4); 32]),
        CheckpointExecInputId::new([mark.wrapping_add(5); 32]),
    )
}

#[derive(Clone)]
struct StepFixture {
    statement: CheckpointTransitionStatementV1,
    core: CheckpointTransitionStatementCoreV1,
    final_bind: CheckpointTransitionStatementFinalV1,
    link: CheckpointLink,
    sidecar: RecursiveCheckpointSidecarV1,
}

fn build_step(
    verifier: &RecursiveCheckpointVerifierV1,
    mode: RecursiveCheckpointModeV1,
    mark: u8,
    height: u64,
    prev_root: [u8; 32],
    new_root: [u8; 32],
    chain_index: u32,
    chain_length: u32,
) -> StepFixture {
    let statement = statement(mark, height, prev_root, new_root);
    let core = CheckpointTransitionStatementCoreV1::new(
        [mark.wrapping_add(6); 32],
        [mark.wrapping_add(7); 32],
        [mark.wrapping_add(8); 32],
        [mark.wrapping_add(9); 32],
    )
    .with_prior_recursive_output_root(prev_root);
    let final_bind = CheckpointTransitionStatementFinalV1::new([mark.wrapping_add(10); 32]);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([mark.wrapping_add(11); 32]),
        statement.prep_snapshot_id(),
        statement.exec_input_id(),
    )
    .expect("checkpoint link");
    let public_input = verifier
        .build_public_input(
            &statement,
            &core,
            &final_bind,
            &link,
            mode,
            backend_label(mode),
            chain_index,
            chain_length,
            [mark.wrapping_add(12); 32],
        )
        .expect("public input");
    let proof_bytes = vec![mark; 96];
    let proof = RecursiveCheckpointProofV1::new(
        mode,
        backend_label(mode),
        &public_input,
        proof_bytes.clone(),
    )
    .expect("recursive proof");
    let measurement = RecursiveCheckpointMeasurementV1::new(
        backend_label(mode),
        mode,
        chain_length,
        cfg().post_quantum.cadence_blocks,
        chain_length.saturating_sub(1).max(1),
        proof_family(mode),
        security_bits(mode),
        proof_bytes.len() as u64,
        4096,
        17,
        9,
        8192,
        statement.canonical_bytes_v1().len() as u64,
        public_input.canonical_bytes().len() as u64,
    )
    .expect("measurement");
    let sidecar = RecursiveCheckpointSidecarV1::accepted(
        public_input,
        Some(link.checkpoint_id()),
        proof,
        measurement,
    )
    .expect("sidecar");
    StepFixture {
        statement,
        core,
        final_bind,
        link,
        sidecar,
    }
}

fn build_chain(
    verifier: &RecursiveCheckpointVerifierV1,
    mode: RecursiveCheckpointModeV1,
    steps: u32,
) -> Vec<StepFixture> {
    let mut out = Vec::new();
    let mut prev_root = [1u8; 32];
    for index in 0..steps {
        let mark = u8::try_from(index + 20).expect("step mark");
        let new_root = [mark; 32];
        out.push(build_step(
            verifier,
            mode,
            mark,
            100 + u64::from(index),
            prev_root,
            new_root,
            index,
            steps,
        ));
        prev_root = new_root;
    }
    out
}

fn json_root(byte: u8) -> Value {
    Value::Array((0..32).map(|_| json!(byte)).collect())
}

#[test]
fn test_shadow_branch_storage_only() {
    let cfg = cfg();

    assert!(cfg.gates.artifacts.has_recursive_sidecar_non_authoritative);
    assert!(cfg.branches.recursive.is_enabled);
    assert!(!cfg.branches.recursive.is_authoritative);
    assert!(cfg.branches.recursive.has_prior_output_binding);
    assert_eq!(cfg.branches.recursive.min_chain_steps, 3);
    assert_eq!(cfg.branches.recursive.target_chain_steps, 5);
}

#[test]
fn test_shadow_stage_pq_next() {
    let mut cfg = cfg();
    cfg.authority_promotion.stage = "recursive_shadow_sidecar".to_string();
    cfg.authority_promotion.allowed_next_stages = vec!["verified_backend_candidate".to_string()];

    let err = cfg
        .validate()
        .expect_err("recursive shadow stage must point only to pq writer");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_shadow_path_isolated() {
    let cfg = cfg();

    assert_ne!(cfg.paths.recursive_sidecars, cfg.paths.checkpoint_artifacts);
    assert_ne!(cfg.paths.recursive_sidecars, cfg.paths.pq_checkpoints);
    assert!(cfg.paths.recursive_sidecars.ends_with("recursive_shadow"));
}

#[test]
fn test_chain_bounds_reject_drift() {
    let mut contract = cfg();
    contract.branches.recursive.min_chain_steps = 2;

    let err = contract
        .validate()
        .expect_err("recursive min chain steps below 3 must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));

    let mut contract = cfg();
    contract.branches.recursive.target_chain_steps = 6;
    let err = contract
        .validate()
        .expect_err("recursive target chain steps above 5 must reject");

    assert!(matches!(err, CheckpointError::ContractConfig(_)));
}

#[test]
fn test_chain_three_step_ok() {
    let verifier = verifier();
    let steps = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    );

    for step in &steps {
        verifier
            .verify_sidecar(
                &step.sidecar,
                &step.statement,
                &step.core,
                &step.final_bind,
                &step.link,
            )
            .expect("sidecar verifies");
    }

    let chain = verifier
        .build_chain(
            &steps
                .iter()
                .map(|step| step.sidecar.clone())
                .collect::<Vec<_>>(),
        )
        .expect("chain evidence");

    assert_eq!(chain.chain_length(), 3);
    verifier.verify_chain(&chain).expect("chain verifies");
}

#[test]
fn test_chain_five_step_ok() {
    let verifier = verifier();
    let steps = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        5,
    );

    let chain = verifier
        .build_chain(
            &steps
                .iter()
                .map(|step| step.sidecar.clone())
                .collect::<Vec<_>>(),
        )
        .expect("five-step chain evidence");

    assert_eq!(chain.chain_length(), 5);
    verifier
        .verify_chain(&chain)
        .expect("five-step chain verifies");
}

#[test]
fn test_statement_digest_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let sidecar = step.sidecar.clone().with_statement_digest([250u8; 32]);

    let err = verifier
        .verify_sidecar(
            &sidecar,
            &step.statement,
            &step.core,
            &step.final_bind,
            &step.link,
        )
        .expect_err("wrong statement digest must reject");

    assert_eq!(
        err,
        RecursiveCheckpointRejectReasonV1::StatementDigestMismatch
    );
}

#[test]
fn test_public_input_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let sidecar = step.sidecar.clone().with_public_input_digest([249u8; 32]);

    let err = verifier
        .verify_sidecar(
            &sidecar,
            &step.statement,
            &step.core,
            &step.final_bind,
            &step.link,
        )
        .expect_err("wrong public input digest must reject");

    assert_eq!(
        err,
        RecursiveCheckpointRejectReasonV1::PublicInputDigestMismatch
    );
}

#[test]
fn test_prior_root_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(1);
    let proof = step
        .sidecar
        .proof()
        .clone()
        .with_prior_output_root([248u8; 32]);
    let sidecar = step.sidecar.clone().with_proof(proof);

    let err = verifier
        .verify_sidecar(
            &sidecar,
            &step.statement,
            &step.core,
            &step.final_bind,
            &step.link,
        )
        .expect_err("wrong prior output root must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::PriorOutputMismatch);
}

#[test]
fn test_backend_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let proof = step
        .sidecar
        .proof()
        .clone()
        .with_backend_label("recursive_mock_v1");
    let sidecar = step.sidecar.clone().with_proof(proof);

    let err = verifier
        .check_sidecar_shape(&sidecar)
        .expect_err("unsupported backend must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::BackendUnsupported);
}

#[test]
fn test_verified_label_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let proof = step
        .sidecar
        .proof()
        .clone()
        .with_backend_label("verified_backend_v1");
    let sidecar = step.sidecar.clone().with_proof(proof);

    let err = verifier
        .check_sidecar_shape(&sidecar)
        .expect_err("verified backend codec must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::VerifiedCodecMissing);
}

#[test]
fn test_empty_proof_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let proof = step.sidecar.proof().clone().with_proof_bytes(Vec::new());
    let sidecar = step.sidecar.clone().with_proof(proof);

    let err = verifier
        .check_sidecar_shape(&sidecar)
        .expect_err("empty proof bytes must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::ProofBytesEmpty);
}

#[test]
fn test_measurement_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let sidecar = step.sidecar.clone().with_measurements(None);

    let err = verifier
        .check_sidecar_shape(&sidecar)
        .expect_err("missing measurement must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::MeasurementMissing);
}

#[test]
fn test_large_proof_rejects() {
    let verifier = verifier();
    let cfg = cfg();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let proof = step.sidecar.proof().clone().with_proof_bytes(vec![
        7u8;
        cfg.limits
            .max_nova_block_proof_bytes
            + 1
    ]);
    let sidecar = step.sidecar.clone().with_proof(proof);

    let err = verifier
        .check_sidecar_shape(&sidecar)
        .expect_err("oversized proof must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::ProofBytesTooLarge);
}

#[test]
fn test_mixed_era_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);
    let proof = step
        .sidecar
        .proof()
        .clone()
        .with_backend_label("plonky3_stark_epoch_v1");
    let sidecar = step.sidecar.clone().with_proof(proof);

    let err = verifier
        .check_sidecar_shape(&sidecar)
        .expect_err("mixed-era proof lane must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::MixedEra);
}

#[test]
fn test_authority_flag_rejects() {
    let mut cfg = cfg();
    cfg.authority_promotion.recursive_authority_allowed = true;

    let err =
        RecursiveCheckpointVerifierV1::new(&cfg).expect_err("recursive authority flag must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::SidecarAuthoritative);
}

#[test]
fn test_canonical_admit_rejects() {
    let verifier = verifier();
    let step = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .remove(0);

    let err = verifier
        .reject_canonical_admission(&step.sidecar)
        .expect_err("sidecar must not become canonical admission");

    assert_eq!(
        err,
        RecursiveCheckpointRejectReasonV1::CanonicalAdmissionAttempt
    );
}

#[test]
fn test_chain_order_rejects() {
    let verifier = verifier();
    let sidecars = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .into_iter()
    .map(|step| step.sidecar)
    .collect::<Vec<_>>();
    let chain =
        RecursiveCheckpointChainEvidenceV1::from_sidecars(&sidecars).expect("chain evidence");
    let mut value = serde_json::to_value(&chain).expect("chain json");
    value["steps"][1]["chain_index"] = json!(0);
    let tampered: RecursiveCheckpointChainEvidenceV1 =
        serde_json::from_value(value).expect("tampered chain");

    let err = verifier
        .verify_chain(&tampered)
        .expect_err("reordered chain must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::StepReordered);
}

#[test]
fn test_chain_tamper_rejects() {
    let verifier = verifier();
    let sidecars = build_chain(
        &verifier,
        RecursiveCheckpointModeV1::FastClassicalCompressed,
        3,
    )
    .into_iter()
    .map(|step| step.sidecar)
    .collect::<Vec<_>>();
    let chain =
        RecursiveCheckpointChainEvidenceV1::from_sidecars(&sidecars).expect("chain evidence");
    let mut value = serde_json::to_value(&chain).expect("chain json");
    value["steps"][1]["prior_output_root"] = json_root(201);
    let tampered: RecursiveCheckpointChainEvidenceV1 =
        serde_json::from_value(value).expect("tampered chain");

    let err = verifier
        .verify_chain(&tampered)
        .expect_err("prior output tamper must reject");

    assert_eq!(err, RecursiveCheckpointRejectReasonV1::PriorOutputMismatch);
}
