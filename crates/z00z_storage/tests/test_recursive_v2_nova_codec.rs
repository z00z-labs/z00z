use z00z_storage::checkpoint::recursive_v2::{
    CheckpointVersionRegistryV2, RecursiveBoundedObjectV2, RecursiveCheckpointSidecarCodecV2,
};

#[test]
fn test_sidecar_codec_rejects_malformed() {
    let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();

    let envelope_header = registry
        .encode_preheader(RecursiveBoundedObjectV2::NovaBlockProof, 0)
        .unwrap();
    assert!(RecursiveCheckpointSidecarCodecV2::decode_bin(&envelope_header).is_err());

    let mut old_version = registry
        .encode_preheader(RecursiveBoundedObjectV2::RecursiveCheckpointSidecar, 0)
        .unwrap();
    old_version[8..10].copy_from_slice(&1_u16.to_le_bytes());
    assert!(RecursiveCheckpointSidecarCodecV2::decode_bin(&old_version).is_err());

    let mut trailing = registry
        .encode_preheader(RecursiveBoundedObjectV2::RecursiveCheckpointSidecar, 0)
        .unwrap()
        .to_vec();
    trailing.push(0);
    assert!(RecursiveCheckpointSidecarCodecV2::decode_bin(&trailing).is_err());

    let mut oversized = registry
        .encode_preheader(RecursiveBoundedObjectV2::RecursiveCheckpointSidecar, 0)
        .unwrap();
    oversized[20..28].copy_from_slice(&(64_u64 * 1024 + 1).to_le_bytes());
    assert!(RecursiveCheckpointSidecarCodecV2::decode_bin(&oversized).is_err());
}
