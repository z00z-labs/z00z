//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0xdc, 0x64, 0x8d, 0xda, 0xd1, 0x25, 0x6f, 0x01, 0x66, 0x13, 0xd2, 0xb1, 0xd1, 0x14, 0x50, 0xca,
    0x45, 0xf3, 0x2f, 0x63, 0xec, 0x3a, 0x06, 0xa3, 0x3b, 0x22, 0xa9, 0xe7, 0x44, 0xa9, 0x1f, 0x9e,
];
