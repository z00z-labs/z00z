use z00z_storage::{
    checkpoint::recursive_v2::{
        CheckpointVersionRegistryV2, Plonky3EpochProofV2, RecursiveBoundedObjectV2,
        RecursiveCheckpointRejectReasonV2,
    },
    CheckpointError,
};

#[test]
fn test_nova_only_marker() {
    let mut payload = Vec::from(*b"Z00ZPEP2");
    payload.extend_from_slice(&2_u16.to_le_bytes());
    payload.push(1);

    let registry = CheckpointVersionRegistryV2::authority_pinned().expect("registry");
    let preheader = registry
        .encode_preheader(RecursiveBoundedObjectV2::Plonky3EpochProof, payload.len())
        .expect("epoch preheader");
    let mut envelope = Vec::from(preheader);
    envelope.extend_from_slice(&payload);

    assert!(matches!(
        Plonky3EpochProofV2::decode_local(&envelope),
        Err(CheckpointError::RecursiveRejected(
            RecursiveCheckpointRejectReasonV2::Plonky3DependsOnlyOnNova
        ))
    ));
}
