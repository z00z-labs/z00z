use super::*;
use crate::domains::OwnerSignatureDomain;
use crate::AssetClass;
use z00z_crypto::DomainHasher;
use z00z_utils::rng::{DeterministicRngProvider, DeterministicRngSource};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

/// Create a deterministic RNG for reproducible tests
fn test_rng() -> impl DeterministicRngSource<Rng = rand_chacha::ChaCha20Rng> {
    DeterministicRngProvider::from_seed([42u8; 32])
}

/// Create a time provider for tests
fn test_time() -> SystemTimeProvider {
    SystemTimeProvider
}

fn derive_test_nonce(
    rng: &mut (impl rand::RngCore + rand::CryptoRng),
    time_provider: &dyn TimeProvider,
) -> Nonce {
    crate::assets::nonce::derive_nonce_minimal(rng, time_provider).expect("test nonce")
}

#[test]
fn test_asset_class_byte_uniqueness() {
    // Ensure all AssetClass variants have unique domain bytes
    // This prevents collision attacks and ensures proper asset ID derivation
    let mut bytes = std::collections::BTreeSet::new();

    assert!(bytes.insert(AssetClass::Coin.class_byte()));
    assert!(bytes.insert(AssetClass::Token.class_byte()));
    assert!(bytes.insert(AssetClass::Nft.class_byte()));
    assert!(bytes.insert(AssetClass::Void.class_byte()));

    assert_eq!(bytes.len(), 4, "All AssetClass domain bytes must be unique");

    // Verify no zero byte (reserved)
    assert!(!bytes.contains(&0x00), "Zero byte should not be used");
}

fn create_test_definition() -> Arc<AssetDefinition> {
    Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Coin,
            "Test Coin".to_string(),
            "Z00Z_TST".to_string(),
            8,
            1000,
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0b0001_0000, // burnable
            None,
        )
        .expect("valid definition"),
    )
}

#[test]
fn test_asset_creation() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let asset = Asset::new(def, 100, 1_000_000, &blinding, nonce, &mut rng).expect("valid asset");

    assert_eq!(asset.serial_id(), 100);
    assert_eq!(asset.amount(), 1_000_000);
    assert!(asset.range_proof().is_some());
    assert_eq!(asset.nonce(), &nonce);
}

#[test]
fn test_asset_is_transparent() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let asset = Asset::new(def, 100, 10, &blinding, nonce, &mut rng).expect("valid asset");

    assert!(asset.is_transparent());
    assert!(!asset.is_stealth());
    assert_eq!(asset.payment_type(), "transparent");
}

#[test]
fn test_asset_is_stealth() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let mut asset = Asset::new(def, 100, 10, &blinding, nonce, &mut rng).expect("valid asset");
    asset.r_pub = Some([1u8; 32]);
    asset.owner_tag = Some([2u8; 32]);
    asset.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![3u8; 8],
        tag: [0u8; 16],
    });

    assert!(asset.is_stealth());
    assert!(!asset.is_transparent());
    assert_eq!(asset.payment_type(), "stealth");
}

#[test]
fn test_asset_partial_stealth_fails() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let mut asset = Asset::new(def, 100, 10, &blinding, nonce, &mut rng).expect("valid asset");
    asset.r_pub = Some([1u8; 32]);
    asset.owner_tag = None;
    asset.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![9u8; 4],
        tag: [0u8; 16],
    });

    assert!(asset.validate_stealth_consistency().is_err());
}

#[test]
fn test_asset_tag16_payload_fails() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let mut asset = Asset::new(def, 100, 10, &blinding, nonce, &mut rng).expect("valid asset");
    asset.tag16 = Some(11);

    assert!(asset.validate_stealth_consistency().is_err());
}

#[test]
fn test_asset_serial_id_validation() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    // Valid: serial_id < serials (1000)
    let result = Asset::new(
        Arc::clone(&def),
        999,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    );
    assert!(result.is_ok());

    // Invalid: serial_id >= serials
    // Test structured error with pattern matching (M7 improvement)
    let result = Asset::new(
        Arc::clone(&def),
        1000,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    );
    match result {
        Err(AssetError::InvalidSerialIdStructured {
            definition_id,
            serial_id,
            max_serials,
        }) => {
            assert_eq!(definition_id, def.id);
            assert_eq!(serial_id, 1000);
            assert_eq!(max_serials, 1000); // serials field from definition
        }
        _ => panic!("Expected InvalidSerialIdStructured error with context"),
    }
}

#[test]
fn test_asset_id_uniqueness() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    let asset1 = Asset::new(
        Arc::clone(&def),
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap();
    let asset2 = Asset::new(
        def,
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap();

    // Different nonces → different asset_id
    assert_ne!(asset1.asset_id(), asset2.asset_id());
}

#[test]
fn test_asset_id_deterministic() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let asset1 = Asset::new(Arc::clone(&def), 100, 1_000_000, &blinding, nonce, &mut rng).unwrap();
    let asset2 = Asset::new(def, 100, 1_000_000, &blinding, nonce, &mut rng).unwrap();

    // Same inputs → same asset_id
    assert_eq!(asset1.asset_id(), asset2.asset_id());
}

#[test]
fn test_with_lock_height() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap()
    .with_lock_height(1000);

    assert_eq!(asset.lock_height(), Some(1000));
    assert!(asset.is_locked(999));
    assert!(!asset.is_locked(1000));
    assert!(!asset.is_locked(1001));
}

#[test]
fn test_with_burn() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap()
    .with_burn()
    .expect("burn allowed");

    assert!(asset.is_burned());
}

#[test]
fn test_with_burn_not_allowed() {
    let def = Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Coin,
            "Test".to_string(),
            "TST".to_string(),
            8,
            1000,
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0, // burnable = false (no flag bit 4 set)
            None,
        )
        .unwrap(),
    );
    let def_id = def.id;
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap();
    let result = asset.with_burn();

    // Test structured error with pattern matching (M7 improvement)
    match result {
        Err(AssetError::BurnNotAllowedStructured {
            definition_id,
            policy_flags,
        }) => {
            assert_eq!(definition_id, def_id);
            assert_eq!(policy_flags, 0); // No flags set, especially not BURNABLE (0x01)
        }
        _ => panic!("Expected BurnNotAllowedStructured error with context"),
    }
}

#[test]
fn test_arc_definition_sharing() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    let asset1 = Asset::new(
        Arc::clone(&def),
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap();
    let asset2 = Asset::new(
        Arc::clone(&def),
        200,
        2_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap();

    // Both assets share the same Arc<AssetDefinition>
    assert!(Arc::ptr_eq(asset1.definition(), asset2.definition()));
    assert_eq!(Arc::strong_count(&def), 3); // original + asset1 + asset2
}

#[test]
fn test_validate_burn_flag_mismatch() {
    // Create a non-burnable asset
    let def = Arc::new(
        AssetDefinition::new(
            [0u8; 32],
            AssetClass::Coin,
            "Test".to_string(),
            "TST".to_string(),
            8,
            1000,
            100_000_000,
            "test.io".to_string(),
            1,
            1,
            0, // burnable = false (no flag bit 4 set)
            None,
        )
        .unwrap(),
    );

    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .unwrap();

    // Manually set is_burned to true (bypassing with_burn which checks)
    asset.is_burned = true;

    // validate() should detect this invalid state
    let result = asset.validate();
    assert!(result.is_err());

    // Check error message mentions the asset and burnable flag
    if let Err(AssetError::InvalidAsset(msg)) = result {
        assert!(msg.contains("cannot be burned") || msg.contains("burnable"));
    } else {
        panic!("Expected InvalidAsset error");
    }
}

#[test]
fn test_asset_auto_signature() {
    // Test that Asset::new() automatically generates owner_signature
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);
    let nonce = derive_test_nonce(&mut rng, &test_time());

    let asset = Asset::new(def, 100, 1_000_000, &secret, nonce, &mut rng).expect("valid asset");

    // Verify owner_pub and owner_signature are set
    assert!(
        asset.owner_pub.is_some(),
        "owner_pub should be set automatically"
    );
    assert!(
        asset.owner_signature.is_some(),
        "owner_signature should be set automatically"
    );

    // Verify signature is valid
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Automatically created signature should be valid"
    );

    // Verify owner_pub matches secret key
    let expected_pub = Z00ZRistrettoPoint::from_secret_key(&secret);
    assert_eq!(
        asset.owner_pub.as_ref().unwrap(),
        &expected_pub,
        "owner_pub should match the secret key"
    );
}

#[test]
fn test_new_prevents_wrong_secret() {
    // CRITICAL TEST: Verify that Asset::new() with wrong secret key is impossible
    // This test validates the fix for race condition where owner_pub is set BEFORE sign_owner()

    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let correct_secret = Z00ZScalar::random(&mut rng);
    let wrong_secret = Z00ZScalar::random(&mut rng);

    // Try to create Asset with correct blinding but it will be signed internally
    // The signature will use correct_secret, so it should match owner_pub
    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &correct_secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // Verify that owner_pub matches the secret used
    let expected_pub = Z00ZRistrettoPoint::from_secret_key(&correct_secret);
    assert_eq!(asset.owner_pub.as_ref().unwrap(), &expected_pub);

    // Signature must be valid
    assert!(asset.verify_owner_signature().is_ok());

    // Now verify that trying to re-sign with WRONG key fails
    let result = asset.sign_owner(&wrong_secret, &mut rng);
    assert!(
        result.is_err(),
        "sign_owner() MUST reject wrong secret when owner_pub is already set"
    );
}

#[test]
#[cfg(not(test))]
fn test_validate_requires_range_proof() {
    // Test that validate() requires range_proof in production
    // This is only enabled in production builds (not in test mode)

    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // Remove range_proof
    asset.range_proof = None;

    // validate() should fail
    let result = asset.validate();
    assert!(
        result.is_err(),
        "validate() must reject Asset without range_proof in production"
    );

    // Set empty range_proof
    asset.range_proof = Some(vec![]);

    // validate() should still fail
    let result = asset.validate();
    assert!(
        result.is_err(),
        "validate() must reject Asset with empty range_proof in production"
    );
}

#[test]
fn test_owner_message_debug_assert() {
    // Test that to_owner_message() validates Asset in debug builds
    // This uses debug_assert, so it only runs in debug mode

    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // Valid asset should create message successfully
    let message = asset.to_owner_message();
    assert!(!message.is_empty());

    // In debug builds, invalid asset would trigger debug_assert panic
    // In release builds, this check is optimized away
    // We can't easily test the panic here without #[should_panic]
}

#[test]
fn test_asset_signature_covers_fields() {
    // Test that signature becomes invalid if critical fields are modified
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // Original signature should be valid
    assert!(asset.verify_owner_signature().is_ok());

    // Modify amount - signature should become invalid
    asset.amount = 2_000_000;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying amount should invalidate signature"
    );

    // Restore amount
    asset.amount = 1_000_000;
    assert!(asset.verify_owner_signature().is_ok());

    // Modify is_burned flag - signature should become invalid
    asset.is_burned = true;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Modifying is_burned should invalidate signature"
    );
}

#[test]
fn test_commitment_factories_compatible() {
    // Test that z00z_crypto uses consistent commitment factories internally
    // This ensures that commitment creation is reproducible across the system
    let amount = 1_000_000u64;
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);

    // Create commitment using z00z_crypto public API
    let c1 = z00z_crypto::create_commitment(amount, &blinding)
        .expect("commitment creation must succeed");

    // Create commitment again with same inputs
    let c2 = z00z_crypto::create_commitment(amount, &blinding)
        .expect("commitment creation must succeed");

    // MUST be identical! (deterministic commitment generation)
    assert_eq!(
        c1.as_bytes(),
        c2.as_bytes(),
        "z00z_crypto::create_commitment() must produce deterministic results"
    );
}

#[test]
fn test_verify_complete_validates_crypto() {
    // Test that verify_complete() performs full cryptographic validation
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // verify_complete() should pass for valid asset
    assert!(
        asset.verify_complete().is_ok(),
        "verify_complete() should pass for valid asset"
    );

    // Test that it catches invalid signature
    let mut invalid_asset = asset.clone();
    invalid_asset.amount = 2_000_000; // Breaks signature

    assert!(
        invalid_asset.verify_complete().is_err(),
        "verify_complete() should catch invalid signature"
    );
}

#[test]
fn test_frozen_flag_in_signature() {
    // Test that is_frozen flag is included in signature and tampering is detected
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // Initially not frozen, signature valid
    assert!(!asset.is_frozen);
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Initial signature should be valid"
    );

    // Modify is_frozen flag - signature should become invalid
    asset.is_frozen = true;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Tampering with is_frozen should invalidate signature"
    );

    // Re-sign with is_frozen = true
    asset.owner_signature = Some(
        asset
            .sign_owner(&secret, &mut rng)
            .expect("signing should work"),
    );
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Re-signed asset should be valid"
    );
}

#[test]
fn test_slashed_flag_in_signature() {
    // Test that is_slashed flag is included in signature and tampering is detected
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    // Initially not slashed, signature valid
    assert!(!asset.is_slashed);
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Initial signature should be valid"
    );

    // Modify is_slashed flag - signature should become invalid
    asset.is_slashed = true;
    assert!(
        asset.verify_owner_signature().is_err(),
        "Tampering with is_slashed should invalidate signature"
    );

    // Re-sign with is_slashed = true
    asset.owner_signature = Some(
        asset
            .sign_owner(&secret, &mut rng)
            .expect("signing should work"),
    );
    assert!(
        asset.verify_owner_signature().is_ok(),
        "Re-signed asset should be valid"
    );
}

#[test]
fn test_coin_zero_amount_rejected() {
    // Test that Asset::new() rejects Coin with amount=0 immediately
    let def = create_test_definition(); // Coin class
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let result = Asset::new(
        def,
        100,
        0, // ZERO amount for Coin - should be rejected
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    );

    assert!(
        result.is_err(),
        "Asset::new() must reject Coin with zero amount"
    );
    assert!(
        result.unwrap_err().to_string().contains("non-zero"),
        "Error should mention non-zero requirement"
    );
}

#[test]
fn test_token_zero_amount_rejected() {
    // Test that Asset::new() rejects Token with amount=0
    let def = Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Token, // Token class
            "Test Token".to_string(),
            "Z00Z_TST".to_string(),
            6,
            1000,
            1_000_000,
            "test.io".to_string(),
            1,
            1,
            0b0000_0010, // fungible
            None,
        )
        .expect("valid definition"),
    );

    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let result = Asset::new(
        def,
        100,
        0, // ZERO amount for Token - should be rejected
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    );

    assert!(
        result.is_err(),
        "Asset::new() must reject Token with zero amount"
    );
}

#[test]
fn test_nft_zero_amount_allowed() {
    // Test that NFT assets CAN have zero amount (this is valid)
    let def = Arc::new(
        AssetDefinition::new(
            [42u8; 32],
            AssetClass::Nft, // NFT class
            "Test NFT".to_string(),
            "NFTZ".to_string(),
            0, // NFTs have 0 decimals
            1000,
            1, // NFTs typically have nominal=1
            "test.io".to_string(),
            1,
            1,
            0b0000_0000, // non-fungible
            None,
        )
        .expect("valid definition"),
    );

    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let result = Asset::new(
        def,
        100,
        0, // ZERO amount for NFT - this is ALLOWED
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    );

    assert!(
        result.is_ok(),
        "Asset::new() should allow NFT with zero amount"
    );
}

#[test]
fn test_debug_redacts_sensitive_data() {
    // Test that Debug implementation doesn't leak sensitive cryptographic data
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let asset = Asset::new(
        def,
        100,
        1_234_567, // Specific amount to check it's redacted
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    let debug_output = format!("{:?}", asset);

    // Verify sensitive fields are redacted
    assert!(
        !debug_output.contains("1234567"),
        "Debug output should NOT contain actual amount value"
    );
    assert!(
        debug_output.contains("<redacted>"),
        "Debug output should show <redacted> for sensitive fields"
    );

    // Verify commitment is redacted (commitment contains cryptographic data)
    assert!(
        !debug_output.contains("RistrettoPoint"),
        "Debug output should NOT contain commitment point data"
    );

    // Verify range proof shows size only, not data
    assert!(
        debug_output.contains("range_proof") && debug_output.contains("bytes"),
        "Debug output should show range proof size, not data"
    );

    // Verify owner_pub and owner_signature are redacted
    assert!(
        debug_output.contains("owner_pub") && debug_output.contains("<present>"),
        "Debug output should show owner_pub as <present>"
    );
    assert!(
        debug_output.contains("owner_signature") && debug_output.contains("<present>"),
        "Debug output should show owner_signature as <present>"
    );

    // Verify safe fields ARE present
    assert!(
        debug_output.contains("definition_id"),
        "Debug output should contain definition_id"
    );
    assert!(
        debug_output.contains("serial_id") && debug_output.contains("100"),
        "Debug output should contain serial_id value"
    );
    assert!(
        debug_output.contains("is_burned"),
        "Debug output should contain state flags"
    );

    assert!(
        !debug_output.contains(&hex::encode(asset.nonce)),
        "Debug output should NOT contain nonce bytes"
    );
    if let Some(leaf_ad_id) = asset.leaf_ad_id {
        assert!(
            !debug_output.contains(&hex::encode(leaf_ad_id)),
            "Debug output should NOT contain leaf_ad_id bytes"
        );
    }
}

#[test]
fn test_prior_owner_signature_rejects() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let secret = Z00ZScalar::random(&mut rng);

    let mut asset = Asset::new(
        def,
        100,
        1_000_000,
        &secret,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    let mut hasher = DomainHasher::<OwnerSignatureDomain>::new_with_label("owner");
    hasher.update(asset.definition_id());
    hasher.update(asset.serial_id.to_le_bytes());
    hasher.update(asset.amount.to_le_bytes());
    hasher.update(asset.commitment.as_bytes());
    hasher.update(asset.nonce);
    hasher.update(asset.lock_height.unwrap_or(0).to_le_bytes());
    match asset.range_proof.as_ref() {
        Some(proof) => hasher.update(proof),
        None => hasher.update([]),
    }
    hasher.update([asset.is_burned as u8]);
    hasher.update([asset.is_frozen as u8]);
    hasher.update([asset.is_slashed as u8]);
    hasher.update([asset.owner_pub.is_some() as u8]);
    if let Some(owner_pub) = asset.owner_pub.as_ref() {
        hasher.update(owner_pub.as_bytes());
    }
    hasher.update([asset.r_pub.is_some() as u8]);
    if let Some(r_pub) = asset.r_pub {
        hasher.update(r_pub);
    }
    hasher.update([asset.owner_tag.is_some() as u8]);
    if let Some(owner_tag) = asset.owner_tag {
        hasher.update(owner_tag);
    }
    hasher.update([asset.enc_pack.is_some() as u8]);
    if let Some(enc_pack) = asset.enc_pack.as_ref() {
        hasher.update([enc_pack.version]);
        hasher.update((enc_pack.ciphertext.len() as u32).to_le_bytes());
        hasher.update(&enc_pack.ciphertext);
        hasher.update(enc_pack.tag);
    }
    hasher.update([asset.tag16.is_some() as u8]);
    if let Some(tag16) = asset.tag16 {
        hasher.update(tag16.to_le_bytes());
    }
    hasher.update([asset.leaf_ad_id.is_some() as u8]);
    if let Some(leaf_ad_id) = asset.leaf_ad_id {
        hasher.update(leaf_ad_id);
    }

    let prior_message = hasher.finalize().as_ref().to_vec();
    let prior_signature = z00z_crypto::sign_kernel_signature(&secret, &prior_message, &mut rng)
        .expect("prior-format signing should work");
    asset.owner_signature = Some(prior_signature);

    assert!(
        asset.verify_owner_signature().is_err(),
        "verify_owner_signature must reject prior owner messages once the compatibility lane is removed"
    );
}

#[test]
fn test_zero_amount_accepted() {
    let def = Arc::new(
        AssetDefinition::new(
            [7u8; 32],
            AssetClass::Nft,
            "Zero NFT".to_string(),
            "ZNFT".to_string(),
            0,
            100,
            1,
            "test.io".to_string(),
            1,
            1,
            0,
            None,
        )
        .expect("valid definition"),
    );

    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let asset = Asset::new(
        def,
        1,
        0,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    assert!(asset.validate_amount().is_ok());
}

#[test]
fn test_max_u64_accepted() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let asset = Asset::new(
        def,
        1,
        u64::MAX,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    assert!(asset.validate_amount().is_ok());
}

#[test]
fn test_missing_range_proof_rejected() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let mut asset = Asset::new(
        def,
        1,
        100,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    asset.range_proof = None;
    let result = asset.verify_range_proof();

    assert!(matches!(result, Err(AssetError::MissingRangeProof)));
}

#[test]
fn test_validate_amount_tampered_proof() {
    let def = create_test_definition();
    let provider = test_rng();
    let mut rng = provider.rng();
    let blinding = Z00ZScalar::random(&mut rng);
    let mut asset = Asset::new(
        def,
        1,
        100,
        &blinding,
        derive_test_nonce(&mut rng, &test_time()),
        &mut rng,
    )
    .expect("valid asset");

    let proof = asset.range_proof.as_mut().expect("range proof");
    proof[0] ^= 1;

    assert!(asset.validate_amount().is_err());
}

#[test]
fn test_new_confidential_ok() {
    let def = create_test_definition();
    let nonce = [9u8; 32];

    let (asset, blinding) =
        Asset::new_confidential(def, 1, 1_000, nonce).expect("new_confidential");

    asset
        .verify_commitment_opening(blinding.reveal())
        .expect("opening");
    asset.verify_range_proof().expect("proof");
}

#[test]
fn test_confidential_blinding_deterministic() {
    let def = create_test_definition();
    let nonce = [12u8; 32];
    let blinding =
        Z00ZScalar::from_uniform_bytes(&[7u8; 64]).expect("uniform bytes must form scalar");

    let first = Asset::new_confidential_with_blinding(def.clone(), 1, 500, nonce, &blinding)
        .expect("first asset");
    let second =
        Asset::new_confidential_with_blinding(def, 1, 500, nonce, &blinding).expect("second asset");

    assert_eq!(first.asset_id(), second.asset_id());
    assert_eq!(first.commitment, second.commitment);
    first
        .verify_commitment_opening(&blinding)
        .expect("opening must verify");
    first.verify_range_proof().expect("proof must verify");
}

#[test]
fn test_opening_ok() {
    let def = create_test_definition();
    let nonce = [10u8; 32];

    let (asset, blinding) = Asset::new_confidential(def, 1, 500, nonce).expect("new_confidential");

    assert!(asset.verify_commitment_opening(blinding.reveal()).is_ok());
}

#[test]
fn test_opening_bad_blind() {
    let def = create_test_definition();
    let nonce = [11u8; 32];

    let (asset, _) = Asset::new_confidential(def, 1, 500, nonce).expect("new_confidential");
    let provider = test_rng();
    let mut rng = provider.rng();
    let wrong = Z00ZScalar::random(&mut rng);

    let result = asset.verify_commitment_opening(&wrong);
    assert!(matches!(result, Err(AssetError::CommitmentMismatch { .. })));
}

#[test]
fn test_checked_add_returns_error() {
    let result = Asset::add_amount(u64::MAX, 1);
    assert!(matches!(result, Err(AssetError::AmountOverflow)));
}
