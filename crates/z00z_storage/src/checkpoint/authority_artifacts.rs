//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0x71, 0xa2, 0x4d, 0xfe, 0xce, 0x53, 0x38, 0x67, 0xb3, 0x97, 0xd7, 0x5d, 0xc1, 0x01, 0x73, 0x38,
    0xfa, 0x55, 0xee, 0x98, 0x6b, 0xdd, 0x35, 0x54, 0x1c, 0x45, 0x36, 0x3a, 0x5a, 0x86, 0x80, 0x23,
];
