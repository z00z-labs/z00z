use z00z_storage::{
    checkpoint::{
        repo_default_path, CheckpointExecInputId, CheckpointId, CheckpointLink,
        CheckpointLinkVersion, CheckpointPubIn, CheckpointTransitionStatementCoreV1,
        CheckpointTransitionStatementFinalV1, CheckpointTransitionStatementV1, CheckpointVersion,
        CreatedEnt, SpentEnt,
    },
    settlement::SettlementStateRoot,
    snapshot::PrepSnapshotId,
};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn test_v3_config_keeps_pq_evidence_non_authoritative() {
    let yaml = std::fs::read_to_string(repo_default_path()).expect("contract YAML");
    assert!(yaml.starts_with("version: 3\n"));
    assert!(yaml.contains("epoch_evidence_commitment: non_authenticating_digest_v2"));
    assert!(yaml.contains("- epoch_evidence_commitment"));
    assert!(yaml.contains("security_role: pq_oriented_evidence_only"));
    assert!(!yaml.contains("pq_signature_or_commitment"));
    assert!(!yaml.contains("is_pq_authoritative"));
    assert!(!yaml.contains("pq_epoch_finality"));
}

#[test]
fn test_statement_golden_vector_is_stable() {
    let statement = CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        42,
        CheckpointPubIn::new_settlement(
            SettlementStateRoot::settlement_v1([1u8; 32]),
            SettlementStateRoot::settlement_v1([2u8; 32]),
            vec![SpentEnt::new([3u8; 32])],
            vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        ),
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let repeated_statement = CheckpointTransitionStatementV1::new(
        CheckpointVersion::CURRENT,
        42,
        CheckpointPubIn::new_settlement(
            SettlementStateRoot::settlement_v1([1u8; 32]),
            SettlementStateRoot::settlement_v1([2u8; 32]),
            vec![SpentEnt::new([3u8; 32])],
            vec![CreatedEnt::new([4u8; 32], [5u8; 32])],
        ),
        PrepSnapshotId::new([6u8; 32]),
        CheckpointExecInputId::new([7u8; 32]),
    );
    let core =
        CheckpointTransitionStatementCoreV1::new([8u8; 32], [9u8; 32], [10u8; 32], [11u8; 32]);
    let final_bind = CheckpointTransitionStatementFinalV1::new([12u8; 32]);
    let link = CheckpointLink::new(
        CheckpointLinkVersion::CURRENT,
        CheckpointId::new([13u8; 32]),
        statement.prep_snapshot_id(),
        statement.exec_input_id(),
    )
    .expect("link");
    assert_eq!(statement.canonical_bytes_v1().len(), 696);
    assert_eq!(
        repeated_statement.canonical_bytes_v1(),
        statement.canonical_bytes_v1()
    );
    assert_eq!(
        repeated_statement.final_statement_digest_v1(&core, &final_bind),
        statement.final_statement_digest_v1(&core, &final_bind)
    );
    assert_eq!(
        hex(&statement.final_statement_digest_v1(&core, &final_bind)),
        "4084e2a07020cf743dbe3427f213c38aa0771320f05c9deb60efb1b1f569f290"
    );
    assert_eq!(
        hex(&link.link_bind()),
        "46dbc0ce7ad836f61041c6fd941af042ba796e6eb56e9b4898764d4fe634924e"
    );
}
