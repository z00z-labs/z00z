//! Small cryptographic hashing helpers shared across platform boundaries.

use sha2::{Digest, Sha256};

/// Compute the SHA-256 digest of an exact byte slice.
#[must_use]
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Compute the lowercase hexadecimal SHA-256 digest of an exact byte slice.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    sha256(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{sha256, sha256_hex};

    #[test]
    fn matches_empty_sha256_known_answer() {
        assert_eq!(
            sha256_hex(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(sha256(&[]).len(), 32);
    }
}
