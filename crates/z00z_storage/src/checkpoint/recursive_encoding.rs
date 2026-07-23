//! Sole authority-selected recursive object-cap owner.

use super::{
    contract_config_v3::CheckpointConfigResolverV3, version_registry::RecursiveBoundedObjectV2,
};
use crate::CheckpointError;

/// Return the complete encoded-object cap for the authority-selected family.
/// family. The wire never selects a family and unsupported families do not
/// inherit a generic or larger fallback cap.
pub(crate) fn effective_object_cap(
    object: RecursiveBoundedObjectV2,
) -> Result<usize, CheckpointError> {
    let active =
        CheckpointConfigResolverV3::resolve_active().map_err(|_| CheckpointError::Authority)?;
    match object {
        RecursiveBoundedObjectV2::NovaBlockProof => {
            Ok(active.config().limits.max_recursive_proof_envelope_bytes)
        }
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar => {
            Ok(active.config().limits.max_recursive_sidecar_bytes)
        }
        _ => Err(CheckpointError::Version),
    }
}

/// Validate a complete encoded object against its active schema-3 ingress cap.
pub(crate) fn validate_object_ingress(
    object: RecursiveBoundedObjectV2,
    encoded_len: usize,
) -> Result<(), CheckpointError> {
    let active =
        CheckpointConfigResolverV3::resolve_active().map_err(|_| CheckpointError::Authority)?;
    match object {
        RecursiveBoundedObjectV2::NovaBlockProof => active
            .config()
            .validate_recursive_proof_envelope_ingress(encoded_len),
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar => active
            .config()
            .validate_recursive_sidecar_ingress(encoded_len),
        _ => Err(CheckpointError::Version),
    }
}

/// Return the independently configured inner compressed-Nova body cap.
pub(crate) fn effective_nova_proof_body_cap() -> Result<usize, CheckpointError> {
    let active =
        CheckpointConfigResolverV3::resolve_active().map_err(|_| CheckpointError::Authority)?;
    Ok(active.config().limits.max_nova_block_proof_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_caps_match_v3() {
        assert_eq!(
            effective_object_cap(RecursiveBoundedObjectV2::NovaBlockProof).unwrap(),
            17_825_792
        );
        assert_eq!(
            effective_object_cap(RecursiveBoundedObjectV2::RecursiveCheckpointSidecar).unwrap(),
            25_165_824
        );
        assert_eq!(effective_nova_proof_body_cap().unwrap(), 131_072);
        assert!(
            validate_object_ingress(RecursiveBoundedObjectV2::NovaBlockProof, 17_825_792).is_ok()
        );
        assert!(matches!(
            validate_object_ingress(RecursiveBoundedObjectV2::NovaBlockProof, 17_825_793),
            Err(CheckpointError::Limit)
        ));
        assert!(matches!(
            validate_object_ingress(
                RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
                25_165_825
            ),
            Err(CheckpointError::Limit)
        ));
    }
}
