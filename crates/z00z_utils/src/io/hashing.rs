//! Narrow build-time and integrity hashing helpers.

use sha2::{Digest, Sha256};

/// Compute the standard unframed SHA-256 digest of one byte slice.
#[must_use]
pub fn sha256_256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Encode public bytes as lowercase hexadecimal text.
#[must_use]
pub fn to_lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 15) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{sha256_256, to_lower_hex};

    #[test]
    fn sha256_known_answer_vectors() {
        assert_eq!(
            to_lower_hex(&sha256_256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            to_lower_hex(&sha256_256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
