//! Transaction output builder for confidential asset leaves.

#[cfg(test)]
use z00z_storage::settlement::TerminalLeaf;

#[cfg(test)]
use z00z_core::assets::AssetPackPlain;
#[cfg(test)]
use z00z_crypto::{
    compute_leaf_ad, compute_tag16, create_commitment, create_range_proof, domains::AssetIdDomain,
    hash_zk::hash_zk, kdf::compute_owner_tag, Hidden, Z00ZCommitment, Z00ZScalar,
    MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};
#[cfg(test)]
use z00z_utils::rng::SystemRngProvider;

#[cfg(test)]
use crate::stealth::zkpack::ZkPack;

#[cfg(test)]
use crate::WalletError;

#[cfg(test)]
fn commitment_bytes(commitment: &Z00ZCommitment) -> [u8; 32] {
    let bytes = commitment.as_bytes();
    debug_assert_eq!(bytes.len(), 32, "commitment must be exactly 32 bytes");
    bytes
        .try_into()
        .expect("commitment must be exactly 32 bytes")
}

#[cfg(test)]
#[cfg(all(debug_assertions, not(test), not(feature = "test-params-fast")))]
fn ensure_release_for_range_proof() -> Result<(), WalletError> {
    let env = EnvConfig;
    if matches!(env.get("Z00Z_ALLOW_DEBUG_RANGE_PROOF"), Ok(Some(_))) {
        return Ok(());
    }
    Err(WalletError::CryptoError(
        "range proof generation in debug build is disabled for production paths; use --release"
            .to_string(),
    ))
}

#[cfg(test)]
#[cfg(any(not(debug_assertions), test, feature = "test-params-fast"))]
fn ensure_release_for_range_proof() -> Result<(), WalletError> {
    Ok(())
}

#[cfg(test)]
fn build_output_core(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
    blinding: &Z00ZScalar,
    range_proof: Vec<u8>,
) -> Result<TerminalLeaf, WalletError> {
    let commitment = create_commitment(value, blinding)
        .map_err(|err| WalletError::CryptoError(err.to_string()))?;
    let c_amount = commitment_bytes(&commitment);

    let owner_tag = compute_owner_tag(owner_handle, k_dh);
    let asset_id = hash_zk::<AssetIdDomain>("", &[&s_out]);
    let leaf_ad = compute_leaf_ad(&asset_id, serial_id, r_pub, &owner_tag, &c_amount);

    let payload = AssetPackPlain {
        value,
        blinding: blinding.to_bytes(),
        s_out,
    }
    .to_bytes();

    let enc_pack = ZkPack::encrypt(k_dh, &leaf_ad, r_pub, &asset_id, serial_id, &payload);
    let tag16 = compute_tag16(k_dh, &leaf_ad);

    Ok(TerminalLeaf {
        asset_id,
        serial_id,
        r_pub: *r_pub,
        owner_tag,
        c_amount,
        enc_pack,
        range_proof,
        tag16,
    })
}

/// Build one output leaf with caller-provided blinding.
#[cfg(test)]
pub fn build_output_with_blind(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
    blinding: &Hidden<Z00ZScalar>,
) -> Result<TerminalLeaf, WalletError> {
    ensure_release_for_range_proof()?;

    let range_proof = create_range_proof(
        value,
        blinding.reveal(),
        RANGE_PROOF_BITS,
        MIN_VALUE_PROMISE,
    )
    .map_err(|err| WalletError::CryptoError(err.to_string()))?;

    build_output_core(
        k_dh,
        r_pub,
        owner_handle,
        value,
        serial_id,
        s_out,
        blinding.reveal(),
        range_proof,
    )
}

/// Build one output leaf with internally generated blinding.
#[cfg(test)]
pub fn build_output_leaf(
    k_dh: &[u8; 32],
    r_pub: &[u8; 32],
    owner_handle: &[u8; 32],
    value: u64,
    serial_id: u32,
    s_out: [u8; 32],
) -> Result<TerminalLeaf, WalletError> {
    let mut rng = SystemRngProvider.rng();
    let blinding = Hidden::hide(Z00ZScalar::random(&mut rng).unwrap());
    build_output_with_blind(
        k_dh,
        r_pub,
        owner_handle,
        value,
        serial_id,
        s_out,
        &blinding,
    )
}

#[cfg(test)]
mod tests {
    use super::{build_output_leaf, build_output_with_blind};
    use std::time::Instant;
    use z00z_crypto::{
        commitment, create_commitment, create_range_proof, verify_range_proof, Hidden, Z00ZScalar,
        AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
    };

    fn test_scalar(seed: u64) -> Z00ZScalar {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
    }

    #[test]
    fn test_commitment_deterministic() {
        let value = 1000u64;
        let blind = test_scalar(11);
        let first = create_commitment(value, &blind).expect("commitment");
        let second = create_commitment(value, &blind).expect("commitment");
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    #[test]
    fn test_commitment_different_blinding() {
        let value = 1000u64;
        let first_blind = test_scalar(21);
        let second_blind = test_scalar(22);
        let first = create_commitment(value, &first_blind).expect("commitment");
        let second = create_commitment(value, &second_blind).expect("commitment");
        assert_ne!(first.as_bytes(), second.as_bytes());
    }

    #[test]
    fn test_roundtrip_create_open() {
        let value = 999u64;
        let blind = test_scalar(31);
        let commitment_value = create_commitment(value, &blind).expect("commitment");
        assert!(commitment::verify_opening(&commitment_value, value, &blind));
    }

    #[test]
    fn test_full_workflow() {
        let k_dh = [7u8; 32];
        let r_pub = [8u8; 32];
        let owner = [9u8; 32];
        let s_out = [10u8; 32];

        let leaf = build_output_leaf(&k_dh, &r_pub, &owner, 777u64, 42u32, s_out)
            .expect("workflow should succeed");

        assert!(!leaf.range_proof.is_empty());
        assert!(!leaf.enc_pack.ciphertext.is_empty());
        assert_ne!(leaf.c_amount, [0u8; 32]);
    }

    #[test]
    fn test_blinding_different_output() {
        let k_dh = [70u8; 32];
        let r_pub = [80u8; 32];
        let owner = [90u8; 32];
        let s_out = [100u8; 32];
        let first_blind = Hidden::hide(test_scalar(44));
        let second_blind = Hidden::hide(test_scalar(45));

        let first = build_output_with_blind(&k_dh, &r_pub, &owner, 1_000, 1, s_out, &first_blind)
            .expect("first leaf");
        let second = build_output_with_blind(&k_dh, &r_pub, &owner, 1_000, 1, s_out, &second_blind)
            .expect("second leaf");

        assert_ne!(first.c_amount, second.c_amount);
    }

    #[test]
    fn test_range_proof_created() {
        let blind = test_scalar(55);
        let proof =
            create_range_proof(1_000, &blind, RANGE_PROOF_BITS, MIN_VALUE_PROMISE).expect("proof");
        assert!(!proof.is_empty());
    }

    #[test]
    fn test_range_proof_then_verify() {
        let value = 1_000u64;
        let blind = test_scalar(56);
        let commitment = create_commitment(value, &blind).expect("commitment");
        let proof =
            create_range_proof(value, &blind, RANGE_PROOF_BITS, MIN_VALUE_PROMISE).expect("proof");

        verify_range_proof(
            &proof,
            &commitment,
            RANGE_PROOF_BITS,
            AGGREGATION_FACTOR,
            MIN_VALUE_PROMISE,
        )
        .expect("verify");
    }

    #[test]
    fn test_zero_value_proof() {
        let blind = test_scalar(57);
        let proof =
            create_range_proof(0, &blind, RANGE_PROOF_BITS, MIN_VALUE_PROMISE).expect("zero proof");
        assert!(!proof.is_empty());
    }

    #[test]
    fn test_sender_serial_output_bounds() {
        use crate::receiver::ReceiverCard;

        let card = ReceiverCard {
            version: 1,
            owner_handle: [11u8; 32],
            view_pk: [7u8; 32],
            identity_pk: [8u8; 32],
            card_id: None,
            metadata: None,
            signature: [0u8; 64],
        };
        let low = crate::stealth::build_card_stealth_leaf(&card, 1000, 0);
        let high = crate::stealth::build_card_stealth_leaf(&card, 1000, 50_001);
        assert!(low.is_err());
        assert!(high.is_err());
    }

    #[test]
    #[ignore]
    fn test_bench_single_proof() {
        if cfg!(debug_assertions) {
            return;
        }

        let blind = test_scalar(58);
        let start = Instant::now();
        let proof = create_range_proof(1_000, &blind, RANGE_PROOF_BITS, MIN_VALUE_PROMISE)
            .expect("bench proof");
        let elapsed = start.elapsed();

        assert!(!proof.is_empty());
        assert!(elapsed.as_millis() <= 50);
    }
}
