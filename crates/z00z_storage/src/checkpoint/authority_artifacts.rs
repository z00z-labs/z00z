//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0xd7, 0x43, 0x17, 0xe7, 0x4b, 0x2a, 0x74, 0xe4, 0xf3, 0x91, 0x27, 0x84, 0x58, 0xef, 0x96, 0xca,
    0xc7, 0xac, 0x03, 0x14, 0x16, 0xc7, 0x7d, 0x9b, 0x69, 0x66, 0x66, 0x4a, 0x3c, 0xea, 0x29, 0xeb,
];
