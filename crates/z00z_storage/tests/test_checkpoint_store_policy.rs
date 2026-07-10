use serde_json::Value;
use z00z_storage::{
    checkpoint::{
        decode_pruning_decision_bin, decode_pruning_decision_json, encode_pruning_decision_bin,
        encode_pruning_decision_json, PruningDecisionV1, PruningDecisionVersion, PruningNodeClass,
    },
    CheckpointError,
};

fn decision() -> PruningDecisionV1 {
    PruningDecisionV1::new(
        PruningDecisionVersion::CURRENT,
        PruningNodeClass::FullNode,
        "local_full_node_only",
        42,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .expect("pruning decision")
}

#[test]
fn test_pruning_decision_roundtrip_json_and_bin() {
    let decision = decision();

    assert_eq!(
        decode_pruning_decision_bin(&encode_pruning_decision_bin(&decision).expect("bin"))
            .expect("decision"),
        decision
    );
    assert_eq!(
        decode_pruning_decision_json(&encode_pruning_decision_json(&decision).expect("json"))
            .expect("decision"),
        decision
    );
}

#[test]
fn test_archive_node_pruning_rejects() {
    let err = PruningDecisionV1::new(
        PruningDecisionVersion::CURRENT,
        PruningNodeClass::ArchiveNode,
        "local_full_node_only",
        42,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .expect_err("archive node pruning must reject");

    assert!(matches!(err, CheckpointError::PruningMix));
}

#[test]
fn test_pruning_scope_must_stay_local_full_node_only() {
    let err = PruningDecisionV1::new(
        PruningDecisionVersion::CURRENT,
        PruningNodeClass::FullNode,
        "cluster_global",
        42,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
        true,
    )
    .expect_err("scope drift must reject");

    assert!(matches!(err, CheckpointError::PruningMix));
}

#[test]
fn test_pruning_decision_tamper_rejects_json_decode() {
    let decision = decision();
    let mut wire: Value =
        serde_json::from_slice(&encode_pruning_decision_json(&decision).expect("decision json"))
            .expect("json wire");
    wire["target_epoch"] = Value::from(77u64);
    let tampered = serde_json::to_vec_pretty(&wire).expect("json bytes");

    let err = decode_pruning_decision_json(&tampered).expect_err("tampered decision must reject");

    assert!(matches!(err, CheckpointError::PruningMix));
}
