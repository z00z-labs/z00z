#[allow(unused_imports)]
pub use crate::domains::{AssetIdHashDomain, ChecksumHashDomain, TestNonceDomain};
use crate::{types::Z00ZScalar, CryptoError};

mod blake2_hash;
pub mod convenience;
pub mod domains;
mod hmac_sha256;
pub mod policy;
mod sha256_hash;
#[cfg(test)]
mod test_hash;
#[cfg(test)]
mod test_hmac_rfc4231;
#[cfg(test)]
mod test_simple_hash;
pub mod typed;
pub mod zk;

fn dst(domain: &str, label: &str) -> Vec<u8> {
    if domain.contains('\0') || label.contains('\0') {
        panic!("BUG: domain/label cannot contain null bytes");
    }

    let mut out = Vec::with_capacity(32 + domain.len() + label.len() + 16);
    out.extend_from_slice(b"z00z.hash.v1\0");

    let domain_len = (domain.len() as u64).to_le_bytes();
    out.extend_from_slice(&domain_len);
    out.extend_from_slice(domain.as_bytes());

    let label_len = (label.len() as u64).to_le_bytes();
    out.extend_from_slice(&label_len);
    out.extend_from_slice(label.as_bytes());

    out
}

fn chain_len_prefixed(hasher: &mut impl blake2::digest::Update, bytes: &[u8]) {
    let len = (bytes.len() as u64).to_le_bytes();
    hasher.update(&len);
    hasher.update(bytes);
}

pub use blake2_hash::{
    blake2b_256, blake2b_256_simple, blake2b_512, blake2b_512_simple, derive_domain_hash,
    derive_key_from_seed, hash_asset_id, Blake2bHasher, Blake2bHasher256, Blake2bVarHasher,
    DomainHasher, DomainHasher256,
};
pub use domains::CheckpointShaRole;
pub use hmac_sha256::{
    hmac_sha256, hmac_sha256_raw, try_hmac_sha256, try_hmac_sha256_raw, verify_hmac,
};
pub use sha256_hash::{
    sha256_256, sha256_256_role, sha256_256_simple, CheckpointSha256BlockStreamV2,
    CheckpointSha256BlockV2, CheckpointSha256BlockVisitError, CheckpointSha256Error,
    CheckpointSha256V2, SHA256_BLOCK_BYTES_V2, SHA256_DIGEST_BYTES_V2, SHA256_IV_V2,
    SHA256_MAX_BYTES_V2,
};

pub use convenience::{
    safe_take_32, take_32, try_take_32, try_take_n, AssetIdHasher, ChecksumHasher, TestNonceHasher,
};
pub use domains::{ALL_DOMAINS, CONS_DOMAINS, WALLET_DOMAINS};
pub use policy::{
    blake2b_hash, compute_consensus_hash, compute_wallet_seed_hash, hash_cache_key,
    hash_db_record_id, hash_fn_for_domain, poseidon2_framed_words_v1,
    poseidon2_goldilocks_params_v1, poseidon2_hash, ConsensusHash, HashFunction,
    Poseidon2GoldilocksParamsV1, WalletHash, POSEIDON2_GOLDILOCKS_COUNT_BYTES_V1,
    POSEIDON2_GOLDILOCKS_DELIMITER_V1, POSEIDON2_GOLDILOCKS_FRAME_BYTES_V1,
    POSEIDON2_GOLDILOCKS_MODULUS_V1, POSEIDON2_GOLDILOCKS_OUTPUT_WORDS_V1,
    POSEIDON2_GOLDILOCKS_RATE_V1, POSEIDON2_GOLDILOCKS_WIDTH_V1,
    POSEIDON2_GOLDILOCKS_WORD_BYTES_V1,
};
pub use typed::{ConsensusHash32, WalletHash32};
pub use zk::{hash_consensus, hash_to_scalar_zk, hash_zk};

fn encode_h2s_input(domain: &[u8], inputs: &[&[u8]]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&frame_bytes(domain));
    for chunk in inputs {
        encoded.extend_from_slice(&frame_bytes(chunk));
    }
    encoded
}

#[deprecated(
    since = "0.0.0",
    note = "Use hash_to_scalar_zk::<D>() with explicit domain type parameter instead."
)]
pub fn hash_to_scalar_domain(domain: &[u8], inputs: &[&[u8]]) -> Z00ZScalar {
    try_hash_to_scalar_domain(domain, inputs)
        .expect("hash_to_scalar_domain fallback is forbidden on the stable surface")
}

pub fn try_hash_to_scalar_domain(
    domain: &[u8],
    inputs: &[&[u8]],
) -> Result<Z00ZScalar, CryptoError> {
    let encoded = encode_h2s_input(domain, inputs);
    let scalar = hash_to_scalar_zk::<crate::domains::HashToScalarDomain>("H2S", &[&encoded])?;
    if scalar.is_zero() {
        return Err(CryptoError::InvalidScalar);
    }
    Ok(scalar)
}

pub fn frame_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len() + 4);
    output.extend_from_slice(&(input.len() as u32).to_le_bytes());
    output.extend_from_slice(input);
    output
}

pub fn frame_str(s: &str) -> Vec<u8> {
    frame_bytes(s.as_bytes())
}

pub fn frame_u64_le(x: u64) -> [u8; 8] {
    x.to_le_bytes()
}

pub fn frame_u32_le(x: u32) -> [u8; 4] {
    x.to_le_bytes()
}
