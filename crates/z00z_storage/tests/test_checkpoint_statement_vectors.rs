use z00z_crypto::{expert::traits::DomainSeparation, frame_bytes, frame_str, hash_zk::hash_zk};
use z00z_storage::{
    checkpoint::{
        CheckpointDraft, CheckpointExecInputId, CheckpointStatement,
        CheckpointTransitionStatementCoreV1, CheckpointTransitionStatementFinalV1,
        CheckpointTransitionStatementV1, CheckpointVersion, CreatedEnt, SpentEnt,
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN,
    },
    settlement::{CheckRoot, ClaimSourceRoot, SettlementStateRoot},
    snapshot::PrepSnapshotId,
};

const ARTIFACT_STMT_SRC: &str = include_str!("../src/checkpoint/artifact_stmt.rs");
const CHECKPOINT_MOD_SRC: &str = include_str!("../src/checkpoint/mod.rs");

const CORE_LABEL: &str = "checkpoint_transition_statement_core_v1";
const FINAL_LABEL: &str = "checkpoint_transition_statement_v1";
const PROOF_FAMILY_V1: &str = "checkpoint_transition_shared_v1";

struct TestCheckpointStatementDom;

impl DomainSeparation for TestCheckpointStatementDom {
    fn version() -> u8 {
        1
    }

    fn domain() -> &'static str {
        "z00z.storage.checkpoint.statement"
    }
}

fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        CheckpointVersion::CURRENT,
        88,
        CheckRoot::new([1u8; 32]),
        CheckRoot::new([2u8; 32]),
        vec![SpentEnt::new([3u8; 32])],
        vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
    )
}

fn statement() -> CheckpointTransitionStatementV1 {
    CheckpointTransitionStatementV1::from_draft(
        &draft(),
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    )
}

fn core_inputs() -> CheckpointTransitionStatementCoreV1 {
    CheckpointTransitionStatementCoreV1::new([0x11; 32], [0x22; 32], [0x33; 32], [0x44; 32])
        .with_prior_recursive_output_root([0x55; 32])
}

fn final_inputs() -> CheckpointTransitionStatementFinalV1 {
    CheckpointTransitionStatementFinalV1::new([0x66; 32])
}

fn expected_backend_payload(stmt: &CheckpointTransitionStatementV1) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(164);
    bytes.push(stmt.prev_settlement_root().generation_version());
    bytes.extend_from_slice(stmt.prev_settlement_root().as_bytes());
    bytes.push(stmt.new_settlement_root().generation_version());
    bytes.extend_from_slice(stmt.new_settlement_root().as_bytes());
    match stmt.claim_root() {
        Some(claim_root) => {
            bytes.push(1);
            bytes.push(claim_root.root_version());
            bytes.extend_from_slice(claim_root.as_bytes());
        }
        None => {
            bytes.push(0);
            bytes.push(0);
            bytes.extend_from_slice(&[0u8; 32]);
        }
    }
    bytes.extend_from_slice(stmt.exec_input_id().as_bytes());
    bytes.extend_from_slice(stmt.new_root().as_bytes());
    bytes
}

fn push_field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(&frame_str(name));
    out.extend_from_slice(&frame_bytes(value));
}

fn encode_settlement_root(root: SettlementStateRoot) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(33);
    bytes.push(root.generation_version());
    bytes.extend_from_slice(root.as_bytes());
    bytes
}

fn encode_optional_claim_root(claim_root: Option<ClaimSourceRoot>) -> Vec<u8> {
    match claim_root {
        Some(claim_root) => {
            let mut bytes = Vec::with_capacity(34);
            bytes.push(1);
            bytes.push(claim_root.root_version());
            bytes.extend_from_slice(claim_root.as_bytes());
            bytes
        }
        None => vec![0],
    }
}

fn encode_optional_digest(digest: Option<[u8; 32]>) -> Vec<u8> {
    match digest {
        Some(digest) => {
            let mut bytes = Vec::with_capacity(33);
            bytes.push(1);
            bytes.extend_from_slice(&digest);
            bytes
        }
        None => vec![0],
    }
}

fn encode_spent_delta(spent_delta: &[SpentEnt]) -> Vec<u8> {
    let mut bytes = frame_bytes(&(spent_delta.len() as u32).to_le_bytes());
    for spent in spent_delta {
        bytes.extend_from_slice(&frame_bytes(spent.terminal_id().as_bytes()));
    }
    bytes
}

fn encode_created_delta(created_delta: &[CreatedEnt]) -> Vec<u8> {
    let mut bytes = frame_bytes(&(created_delta.len() as u32).to_le_bytes());
    for created in created_delta {
        bytes.extend_from_slice(&frame_bytes(created.terminal_id().as_bytes()));
        bytes.extend_from_slice(&frame_bytes(created.leaf_hash()));
    }
    bytes
}

fn manual_canonical_bytes(stmt: &CheckpointTransitionStatementV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_field(&mut bytes, "statement_version", &[1]);
    push_field(
        &mut bytes,
        "statement_domain",
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN.as_bytes(),
    );
    push_field(
        &mut bytes,
        "proof_system_family",
        PROOF_FAMILY_V1.as_bytes(),
    );
    push_field(
        &mut bytes,
        "checkpoint_version",
        &[stmt.checkpoint_version().as_u8()],
    );
    push_field(&mut bytes, "height", &stmt.height().to_le_bytes());
    push_field(&mut bytes, "prev_root", stmt.prev_root().as_bytes());
    push_field(&mut bytes, "new_root", stmt.new_root().as_bytes());
    push_field(
        &mut bytes,
        "prev_settlement_root",
        &encode_settlement_root(stmt.prev_settlement_root()),
    );
    push_field(
        &mut bytes,
        "new_settlement_root",
        &encode_settlement_root(stmt.new_settlement_root()),
    );
    push_field(
        &mut bytes,
        "claim_root",
        &encode_optional_claim_root(stmt.claim_root()),
    );
    push_field(
        &mut bytes,
        "spent_delta",
        &encode_spent_delta(stmt.spent_delta()),
    );
    push_field(
        &mut bytes,
        "created_delta",
        &encode_created_delta(stmt.created_delta()),
    );
    push_field(
        &mut bytes,
        "prep_snapshot_id",
        stmt.prep_snapshot_id().as_bytes(),
    );
    push_field(&mut bytes, "exec_input_id", stmt.exec_input_id().as_bytes());
    bytes
}

fn reordered_statement_bytes(stmt: &CheckpointTransitionStatementV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_field(&mut bytes, "statement_version", &[1]);
    push_field(
        &mut bytes,
        "statement_domain",
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN.as_bytes(),
    );
    push_field(
        &mut bytes,
        "proof_system_family",
        PROOF_FAMILY_V1.as_bytes(),
    );
    push_field(
        &mut bytes,
        "checkpoint_version",
        &[stmt.checkpoint_version().as_u8()],
    );
    push_field(&mut bytes, "height", &stmt.height().to_le_bytes());
    push_field(&mut bytes, "new_root", stmt.new_root().as_bytes());
    push_field(&mut bytes, "prev_root", stmt.prev_root().as_bytes());
    push_field(
        &mut bytes,
        "prev_settlement_root",
        &encode_settlement_root(stmt.prev_settlement_root()),
    );
    push_field(
        &mut bytes,
        "new_settlement_root",
        &encode_settlement_root(stmt.new_settlement_root()),
    );
    push_field(
        &mut bytes,
        "claim_root",
        &encode_optional_claim_root(stmt.claim_root()),
    );
    push_field(
        &mut bytes,
        "spent_delta",
        &encode_spent_delta(stmt.spent_delta()),
    );
    push_field(
        &mut bytes,
        "created_delta",
        &encode_created_delta(stmt.created_delta()),
    );
    push_field(
        &mut bytes,
        "prep_snapshot_id",
        stmt.prep_snapshot_id().as_bytes(),
    );
    push_field(&mut bytes, "exec_input_id", stmt.exec_input_id().as_bytes());
    bytes
}

fn manual_core_preimage(
    stmt: &CheckpointTransitionStatementV1,
    core: &CheckpointTransitionStatementCoreV1,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_field(&mut bytes, "statement_version", &[1]);
    push_field(
        &mut bytes,
        "statement_domain",
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN.as_bytes(),
    );
    push_field(
        &mut bytes,
        "proof_system_family",
        PROOF_FAMILY_V1.as_bytes(),
    );
    push_field(&mut bytes, "height", &stmt.height().to_le_bytes());
    push_field(&mut bytes, "prev_root", stmt.prev_root().as_bytes());
    push_field(
        &mut bytes,
        "prev_settlement_root",
        &encode_settlement_root(stmt.prev_settlement_root()),
    );
    push_field(
        &mut bytes,
        "checkpoint_exec_input_id",
        stmt.exec_input_id().as_bytes(),
    );
    push_field(
        &mut bytes,
        "prep_snapshot_id",
        stmt.prep_snapshot_id().as_bytes(),
    );
    push_field(&mut bytes, "tx_data_root", &core.tx_data_root());
    push_field(&mut bytes, "delta_root", &core.delta_root());
    push_field(&mut bytes, "witness_root", &core.witness_root());
    push_field(&mut bytes, "journal_digest", &core.journal_digest());
    push_field(
        &mut bytes,
        "claim_root",
        &encode_optional_claim_root(stmt.claim_root()),
    );
    push_field(
        &mut bytes,
        "prior_recursive_output_root",
        &encode_optional_digest(core.prior_recursive_output_root()),
    );
    push_field(&mut bytes, "new_root", stmt.new_root().as_bytes());
    push_field(
        &mut bytes,
        "new_settlement_root",
        &encode_settlement_root(stmt.new_settlement_root()),
    );
    bytes
}

fn reordered_core_preimage(
    stmt: &CheckpointTransitionStatementV1,
    core: &CheckpointTransitionStatementCoreV1,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_field(&mut bytes, "statement_version", &[1]);
    push_field(
        &mut bytes,
        "statement_domain",
        CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN.as_bytes(),
    );
    push_field(
        &mut bytes,
        "proof_system_family",
        PROOF_FAMILY_V1.as_bytes(),
    );
    push_field(&mut bytes, "height", &stmt.height().to_le_bytes());
    push_field(&mut bytes, "prev_root", stmt.prev_root().as_bytes());
    push_field(
        &mut bytes,
        "prev_settlement_root",
        &encode_settlement_root(stmt.prev_settlement_root()),
    );
    push_field(
        &mut bytes,
        "checkpoint_exec_input_id",
        stmt.exec_input_id().as_bytes(),
    );
    push_field(
        &mut bytes,
        "prep_snapshot_id",
        stmt.prep_snapshot_id().as_bytes(),
    );
    push_field(&mut bytes, "delta_root", &core.delta_root());
    push_field(&mut bytes, "tx_data_root", &core.tx_data_root());
    push_field(&mut bytes, "witness_root", &core.witness_root());
    push_field(&mut bytes, "journal_digest", &core.journal_digest());
    push_field(
        &mut bytes,
        "claim_root",
        &encode_optional_claim_root(stmt.claim_root()),
    );
    push_field(
        &mut bytes,
        "prior_recursive_output_root",
        &encode_optional_digest(core.prior_recursive_output_root()),
    );
    push_field(&mut bytes, "new_root", stmt.new_root().as_bytes());
    push_field(
        &mut bytes,
        "new_settlement_root",
        &encode_settlement_root(stmt.new_settlement_root()),
    );
    bytes
}

fn raw_concat_core_bytes(
    stmt: &CheckpointTransitionStatementV1,
    core: &CheckpointTransitionStatementCoreV1,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(1);
    bytes.extend_from_slice(CHECKPOINT_TRANSITION_STATEMENT_V1_DOMAIN.as_bytes());
    bytes.extend_from_slice(PROOF_FAMILY_V1.as_bytes());
    bytes.extend_from_slice(&stmt.height().to_le_bytes());
    bytes.extend_from_slice(stmt.prev_root().as_bytes());
    bytes.extend_from_slice(stmt.prev_settlement_root().as_bytes());
    bytes.extend_from_slice(stmt.exec_input_id().as_bytes());
    bytes.extend_from_slice(stmt.prep_snapshot_id().as_bytes());
    bytes.extend_from_slice(&core.tx_data_root());
    bytes.extend_from_slice(&core.delta_root());
    bytes.extend_from_slice(&core.witness_root());
    bytes.extend_from_slice(&core.journal_digest());
    if let Some(claim_root) = stmt.claim_root() {
        bytes.push(claim_root.root_version());
        bytes.extend_from_slice(claim_root.as_bytes());
    }
    if let Some(prior_recursive_output_root) = core.prior_recursive_output_root() {
        bytes.extend_from_slice(&prior_recursive_output_root);
    }
    bytes.extend_from_slice(stmt.new_root().as_bytes());
    bytes.extend_from_slice(stmt.new_settlement_root().as_bytes());
    bytes
}

fn manual_final_preimage(
    core_digest: [u8; 32],
    final_bind: &CheckpointTransitionStatementFinalV1,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_field(&mut bytes, "statement_core_digest", &core_digest);
    push_field(&mut bytes, "da_ref", &final_bind.da_ref());
    push_field(
        &mut bytes,
        "pq_anchor_root",
        &encode_optional_digest(final_bind.pq_anchor_root()),
    );
    bytes
}

#[test]
fn test_backend_payload_vector_without_claim_root() {
    let stmt = statement();

    assert_eq!(stmt.backend_payload(), expected_backend_payload(&stmt));
    assert_eq!(stmt.backend_payload().len(), 164);
}

#[test]
fn test_backend_payload_vector_with_claim_root() {
    let claim_root =
        ClaimSourceRoot::new_settlement(1, SettlementStateRoot::settlement_v1([0xA4; 32]));
    let draft = draft().with_claim_root(claim_root);
    let stmt = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );

    assert_eq!(stmt.backend_payload(), expected_backend_payload(&stmt));
    assert_eq!(stmt.backend_payload().len(), 164);
}

#[test]
fn test_exec_input_id_changes_backend_payload() {
    let draft = draft();
    let left = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let right = CheckpointTransitionStatementV1::from_draft(
        &draft,
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([8u8; 32]),
    );

    assert_ne!(left.exec_input_id(), right.exec_input_id());
    assert_ne!(left.backend_payload(), right.backend_payload());
}

#[test]
fn test_artifact_statement_roundtrip_keeps_backend_payload() {
    let draft = draft().with_claim_root(ClaimSourceRoot::new_settlement(
        1,
        SettlementStateRoot::settlement_v1([0xB7; 32]),
    ));
    let proof = draft
        .attest_proof(
            PrepSnapshotId::new([6u8; 32]),
            CheckpointExecInputId::new([7u8; 32]),
        )
        .expect("proof");
    let artifact = draft.finalize(proof).expect("artifact");

    match artifact.statement() {
        CheckpointStatement::V1(stmt) => {
            assert_eq!(stmt.backend_payload(), expected_backend_payload(&stmt));
            assert_eq!(stmt.prep_snapshot_id(), PrepSnapshotId::new([6u8; 32]));
            assert_eq!(stmt.exec_input_id(), CheckpointExecInputId::new([7u8; 32]));
        }
        CheckpointStatement::Detached => panic!("artifact statement must stay attached"),
    }
}

#[test]
fn test_canonical_statement_bytes_v1_are_stable_and_field_named() {
    let stmt = statement();
    let actual = stmt.canonical_bytes_v1();
    let expected = manual_canonical_bytes(&stmt);

    assert_eq!(actual, expected);
    assert_ne!(actual, reordered_statement_bytes(&stmt));
}

#[test]
fn test_statement_core_digest_v1_is_stable_and_mutation_sensitive() {
    let stmt = statement();
    let core = core_inputs();
    let actual = stmt.statement_core_digest_v1(&core);
    let expected_preimage = manual_core_preimage(&stmt, &core);
    let expected =
        hash_zk::<TestCheckpointStatementDom>(CORE_LABEL, &[expected_preimage.as_slice()]);
    let reordered = hash_zk::<TestCheckpointStatementDom>(
        CORE_LABEL,
        &[reordered_core_preimage(&stmt, &core).as_slice()],
    );
    let raw_concat = hash_zk::<TestCheckpointStatementDom>(
        CORE_LABEL,
        &[raw_concat_core_bytes(&stmt, &core).as_slice()],
    );

    struct WrongStatementDom;

    impl DomainSeparation for WrongStatementDom {
        fn version() -> u8 {
            1
        }

        fn domain() -> &'static str {
            "z00z.storage.checkpoint.statement.wrong"
        }
    }

    let wrong_domain = hash_zk::<WrongStatementDom>(CORE_LABEL, &[expected_preimage.as_slice()]);

    assert_eq!(actual, expected);
    assert_ne!(actual, reordered);
    assert_ne!(actual, raw_concat);
    assert_ne!(actual, wrong_domain);
}

#[test]
fn test_final_statement_digest_v1_binds_da_ref_and_explicit_pq_absence() {
    let stmt = statement();
    let core = core_inputs();
    let final_bind = final_inputs();
    let core_digest = stmt.statement_core_digest_v1(&core);
    let actual = stmt.final_statement_digest_v1(&core, &final_bind);
    let expected = hash_zk::<TestCheckpointStatementDom>(
        FINAL_LABEL,
        &[manual_final_preimage(core_digest, &final_bind).as_slice()],
    );
    let with_anchor = final_bind.with_pq_anchor_root([0x77; 32]);
    let anchored = stmt.final_statement_digest_v1(&core, &with_anchor);
    let zero_anchor = stmt.final_statement_digest_v1(
        &core,
        &CheckpointTransitionStatementFinalV1::new([0x66; 32]).with_pq_anchor_root([0u8; 32]),
    );

    assert_eq!(actual, expected);
    assert_ne!(actual, anchored);
    assert_ne!(actual, zero_anchor);
}

#[test]
fn test_v1_statement_surface_has_one_canonical_name_path() {
    assert!(ARTIFACT_STMT_SRC.contains("CheckpointTransitionStatementV1"));
    assert!(CHECKPOINT_MOD_SRC.contains("CheckpointTransitionStatementV1"));
    assert!(!ARTIFACT_STMT_SRC.contains("CheckpointStmt"));
    assert!(!ARTIFACT_STMT_SRC.contains("CheckpointStatement::CURRENT"));
}
