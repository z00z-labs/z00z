//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0x66, 0x8a, 0xb9, 0x02, 0x02, 0xf4, 0x15, 0xc6, 0xd4, 0x8a, 0x1c, 0xc6, 0xad, 0x0d, 0xe4, 0x41,
    0x66, 0xdf, 0x95, 0xfe, 0xbb, 0xd1, 0xba, 0xd0, 0x51, 0x24, 0xfa, 0xff, 0x4e, 0x88, 0x15, 0x53,
];
