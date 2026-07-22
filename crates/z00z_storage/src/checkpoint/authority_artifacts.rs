//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0xd7, 0x6c, 0x54, 0x5b, 0x57, 0xd2, 0xcf, 0xfe, 0xc8, 0xe2, 0xe2, 0x56, 0x42, 0x41, 0x85, 0x8d,
    0x5a, 0xb6, 0x6d, 0x91, 0xea, 0x53, 0x7e, 0xd1, 0x83, 0x06, 0x46, 0x1e, 0xaa, 0xb0, 0xe1, 0xff,
];
