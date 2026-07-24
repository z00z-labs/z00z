//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0x98, 0x4f, 0x6a, 0x28, 0x29, 0x6e, 0x0d, 0x83, 0xbd, 0x2f, 0x38, 0x1e, 0xdd, 0x46, 0xe0, 0xfc,
    0x6a, 0x83, 0xf9, 0xa4, 0x0c, 0x17, 0x4f, 0xef, 0xd8, 0x12, 0xbd, 0x98, 0xe7, 0xe3, 0xa8, 0x19,
];

/// Exact upstream revision selected by the live Plonky3 base-proof authority.
pub(crate) const ACTIVE_PLONKY3_SOURCE_REVISION_V2: &str =
    "b36339709a7a67ee9760fb578b3d4339fd983709";

/// One canonical crates.io family for every direct Plonky3 dependency.
pub(crate) const ACTIVE_PLONKY3_CRATES_IO_VERSION_V2: &str = "0.6.1";

/// Exact circuit/circuit-prover API family at the pinned upstream revision.
pub(crate) const ACTIVE_PLONKY3_CIRCUIT_VERSION_V2: &str = "0.1.0";
