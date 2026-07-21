//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0xd3, 0x63, 0xc0, 0x32, 0x78, 0x4d, 0xd8, 0xaf, 0xa2, 0x23, 0xd4, 0xef, 0x1f, 0x66,
    0x4e, 0x6d, 0x10, 0xbb, 0xcd, 0x63, 0x7b, 0x97, 0x3a, 0xda, 0x25, 0xa0, 0xf1, 0xad,
    0x18, 0x73, 0xbd, 0xd9,
];
