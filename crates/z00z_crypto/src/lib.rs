#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
//! Z00Z Cryptography Library - Universal Backend Abstraction
//!
//! This crate provides a unified cryptographic API for the Z00Z protocol,
//! abstracting the underlying backend implementation (currently Tari, but extensible
//! to post-quantum alternatives).
//!
//! # Public API
//!
//! External modules should use only the public functions defined in this crate.
//! Backend choice is hidden inside `z00z_crypto`.
//!
//! ## Hidden Type (Security Best Practice)
//!
//! Wrap sensitive data to prevent accidental logging.
//!
//! ## Hex Encoding (Developer Tools)
//!
//! For debugging, CLI tools, and API responses:
//!
//! ```ignore
//! use z00z_crypto::expert::encoding::{to_hex, from_hex, Hex};
//!
//! let hex_str = to_hex(commitment.as_bytes());
//! let bytes = from_hex(&hex_str)?;
//! ```
//!
//! **Note:** For production use ByteArray binary encoding, not hex.

pub mod aead;
pub mod claim;
pub mod domains;
pub mod error;
pub mod expert;
pub mod hash;
pub mod kdf;
pub mod protocol;
pub mod secret;
pub mod validation;
pub mod vendor;

pub mod types;

mod backend;
mod lib_api;

// ============================================================================
// Public types
// ============================================================================
// All protocol-level types are defined in `types.rs` and re-exported here.

// Re-export public error type
pub use claim::{
    claim_stmt_hash, ClaimAuthoritySig, ClaimError, ClaimProofVer, ClaimSourceProof, ClaimStmt,
    CLAIM_ROOT_VERSION,
};
pub use error::CryptoError;

pub use hash::{
    blake2b_hash, compute_consensus_hash, poseidon2_framed_words_v1,
    poseidon2_goldilocks_params_v1, poseidon2_hash, ConsensusHash, Poseidon2GoldilocksParamsV1,
    WalletHash, POSEIDON2_GOLDILOCKS_COUNT_BYTES_V1, POSEIDON2_GOLDILOCKS_DELIMITER_V1,
    POSEIDON2_GOLDILOCKS_FRAME_BYTES_V1, POSEIDON2_GOLDILOCKS_MODULUS_V1,
    POSEIDON2_GOLDILOCKS_OUTPUT_WORDS_V1, POSEIDON2_GOLDILOCKS_RATE_V1,
    POSEIDON2_GOLDILOCKS_WIDTH_V1, POSEIDON2_GOLDILOCKS_WORD_BYTES_V1,
};
pub use hash::{
    sha256_256, sha256_256_role, CheckpointSha256BlockStreamV2, CheckpointSha256BlockV2,
    CheckpointSha256BlockVisitError, CheckpointSha256Error, CheckpointSha256V2, CheckpointShaRole,
    SHA256_BLOCK_BYTES_V2, SHA256_DIGEST_BYTES_V2, SHA256_IV_V2, SHA256_MAX_BYTES_V2,
};
pub use hash::{Blake2bHasher, Blake2bHasher256, Blake2bVarHasher, DomainHasher, DomainHasher256};
pub use hash::{ConsensusHash32, WalletHash32};
pub use types::{
    validate_amount, validate_amount_relaxed, validate_asset_amount, validate_commitment_non_zero,
    validate_proof_size, validate_transfer_amount, Z00ZBasepointTable, Z00ZCommitment,
    Z00ZCompressedRistretto, Z00ZRistrettoPoint, Z00ZScalar, AGGREGATION_FACTOR, CHECKSUM_BYTES,
    LENGTH_BYTES, MAX_BATCH_MEMORY, MAX_BATCH_PROOF_COUNT, MAX_PROOF_SIZE, MAX_PROOF_SIZE_EXTENDED,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS, RANGE_PROOF_BITS_EXTENDED, VERSION, VERSION_BYTES,
};

pub use vendor::tari::{
    AggregatedPublicStatement, BulletproofsPlusService, CommitmentAndPublicKeySignature,
    CompressedRistrettoComAndPubSig, ExtendedRangeProofService, HomomorphicCommitmentFactory,
    RangeProofService, RistrettoAggregatedPublicStatement, RistrettoComAndPubSig, RistrettoComSig,
    RistrettoSchnorr, RistrettoStatement, SchnorrSignature, Statement,
};

pub type Z00ZRangeProof = Vec<u8>;
pub type Z00ZSchnorrSignature = RistrettoSchnorr;

pub type RangeProof = Z00ZRangeProof;
pub type KernelSignature = Z00ZSchnorrSignature;

// Re-export framing primitives for canonical encoding
pub use hash::{frame_bytes, frame_str, frame_u32_le, frame_u64_le};
pub use protocol::zkpack::ZkPackEncrypted;

// Re-export HMAC primitives (RFC 2104 compliant)
pub use hash::{hmac_sha256, hmac_sha256_raw, try_hmac_sha256, try_hmac_sha256_raw, verify_hmac};

// Re-export collision-resistant derivation functions
pub use hash::{derive_domain_hash, hash_asset_id};

// Re-export KDF primitives
pub use kdf::{
    compute_owner_tag, derive_argon2id32_key, derive_asset_id, derive_db_encryption_key,
    derive_encrypt_and_mac_keys, derive_leaf_ad, derive_owner_handle, derive_pack_key,
    derive_pack_nonce, derive_symmetric_key_from_ecdh, derive_view_pk, derive_view_sk,
    generate_hedged_r, hash_to_scalar_domain, hkdf_expand_32, kdf_consensus, kdf_from_dh,
    kdf_wallet, kdf_wallet_variable, try_hash_to_scalar_domain, Argon2Params, KdfError,
    SecretBytes32,
};
pub use protocol::commitments::{
    generate_blinding_factor, verify_opening, BlindingFactorGenerator, Commitment, CommitmentErr,
    CommitmentOpening,
};
pub use protocol::stealth_bind::{
    compute_leaf_ad, compute_tag16, encode_leaf_preimage, range_ctx_hash, LEAF_PREIMAGE_SIZE,
};

pub mod kdf_consensus {
    pub use crate::kdf::{
        compute_owner_tag, derive_asset_id, derive_leaf_ad, derive_owner_handle, derive_view_sk,
    };
}

pub mod kdf_extended {
    pub use crate::kdf::{
        derive_db_encryption_key, derive_encrypt_and_mac_keys, derive_pack_key, derive_pack_nonce,
        derive_symmetric_key_from_ecdh, derive_view_pk, generate_hedged_r, hash_to_scalar_domain,
        kdf_consensus, kdf_from_dh, kdf_wallet, kdf_wallet_variable, try_hash_to_scalar_domain,
    };
}

pub mod commitment {
    use crate::{create_commitment, Z00ZCommitment, Z00ZScalar};

    pub fn commit_value(value: u64, blinding: &Z00ZScalar) -> Z00ZCommitment {
        create_commitment(value, blinding).expect("invalid blinding")
    }

    pub fn verify_opening(c: &Z00ZCommitment, value: u64, blinding: &Z00ZScalar) -> bool {
        create_commitment(value, blinding)
            .map(|expected| expected.as_bytes() == c.as_bytes())
            .unwrap_or(false)
    }

    pub fn h_base() -> Z00ZCommitment {
        commit_value(0, &Z00ZScalar::one())
    }
}
pub use protocol::range_proofs::{
    verify_asset_output_proofs_batch, AssetOutputProof, AssetRangeProof, RangeProofErr,
};

// Re-export secret bytes wrapper
pub use secret::SecretBytes;

// Re-export AEAD primitives (only high-level, DoS-protected API)
pub use aead::{
    build_aad, build_aad_multipart, build_aad_multipart_extended, open, open_extended_aad,
    random_nonce, seal, seal_extended_aad, AeadError, MAX_AAD_SIZE, MAX_AAD_SIZE_EXTENDED,
    MAX_AEAD_ENVELOPE_SIZE, MAX_AEAD_PLAINTEXT_SIZE, MIN_ENVELOPE_SIZE, POLY1305_TAG_SIZE,
    XCHACHA20_POLY1305_ID, XCHACHA_KEY_SIZE, XCHACHA_NONCE_SIZE,
};
pub use aead_transport::{
    decrypt_asset_pack, decrypt_asset_package_transport, encrypt_asset_pack,
    encrypt_asset_package_transport, AssetPackCt,
};

pub mod aead_transport {
    pub use crate::aead::transport::{
        decrypt_asset_pack, decrypt_asset_package_transport, encrypt_asset_pack,
        encrypt_asset_package_transport, AssetPackCt,
    };
}

#[cfg(feature = "experimental-zkpack")]
pub mod experimental {
    pub mod zkpack {
        pub use crate::aead::zkpack::{open_zkpack, seal_zkpack, Pack};
    }
}

pub mod hash_policy {
    pub use crate::hash::policy::{
        blake2b_hash, compute_consensus_hash, compute_wallet_seed_hash, hash_cache_key,
        hash_db_record_id, hash_fn_for_domain, poseidon2_hash, ConsensusHash, HashFunction,
        WalletHash,
    };
}

pub mod hash_types {
    pub use crate::hash::typed::{ConsensusHash32, WalletHash32};
}

pub mod hash_zk {
    pub use crate::hash::zk::{hash_consensus, hash_to_scalar_zk, hash_zk};
}

pub mod hashing {
    pub use crate::hash::convenience::{
        safe_take_32, take_32, try_take_32, try_take_n, AssetIdHasher, ChecksumHasher,
        TestNonceHasher,
    };
}

// ============================================================================
// RE-EXPORTS - Minimal Surface Area
// ============================================================================
// Only re-export what's absolutely needed. Keep Tari dependency private.
// ============================================================================

// Enhanced Hidden type with safer API
pub mod hidden;
pub use hidden::Hidden;

// Wallet key derivation helpers live in `z00z_wallets::key`.

// ============================================================================
// BACKEND IMPLEMENTATION (PRIVATE)
// ============================================================================
// This is the ONLY place where the concrete crypto backend is selected directly.
// Higher-level operations delegate through Z00Z wrapper types or public API
// functions, while a small compatibility surface still re-exports selected Tari types.
// ============================================================================

use backend::CryptoBackend;
use backend::TariCryptoBackend;

static TARI_BACKEND: TariCryptoBackend = TariCryptoBackend;

/// Get default cryptographic backend (currently Tari).
///
/// This function is PRIVATE because backend choice is an implementation detail.
/// External code uses public functions that automatically delegate to this backend.
///
/// # Design Rationale
///
/// - **Private:** Backend is an implementation detail
/// - **Static:** Zero runtime cost, compile-time known
/// - **Inline:** Compiler can optimize away the call entirely
#[inline]
pub(crate) fn default_backend() -> &'static impl CryptoBackend {
    &TARI_BACKEND
}

#[cfg(not(target_arch = "wasm32"))]
pub use lib_api::sign_kernel_signature;
pub use lib_api::verify_kernel_signature;
pub use lib_api::{
    batch_verify_range_proofs, batch_verify_range_proofs_with, create_commitment,
    create_range_proof, create_range_proof_rng, derive_hash, initialize, verify_range_proof,
    VerifyCommitmentInput,
};
