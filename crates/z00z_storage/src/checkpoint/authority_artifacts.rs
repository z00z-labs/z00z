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
