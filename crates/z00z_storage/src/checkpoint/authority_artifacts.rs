//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0xb6, 0x91, 0xfa, 0x10, 0x19, 0x4a, 0xf3, 0x6f, 0x11, 0xa0, 0xe3, 0x07, 0xa8, 0xd7, 0xb8, 0xc7,
    0xd4, 0x17, 0x78, 0x8c, 0x92, 0x22, 0x0b, 0x78, 0x18, 0x4e, 0x6b, 0x7b, 0xa5, 0xd4, 0x7d, 0x5f,
];
