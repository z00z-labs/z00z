use super::*;

fn create_test_definition() -> AssetDefinition {
    AssetDefinition::new(
        [42u8; 32],
        AssetClass::Coin,
        "Test Coin".to_string(),
        "TST".to_string(),
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        BURNABLE, // burnable flag
        None,
    )
    .expect("valid test definition")
}

#[test]
fn test_asset_definition_creation() {
    let def = create_test_definition();
    assert_eq!(def.name, "Test Coin");
    assert_eq!(def.symbol, "TST");
    assert_eq!(def.decimals, 8);
    assert_eq!(def.serials, 1000);
    assert_eq!(def.nominal, 100_000_000);
}

#[test]
fn test_total_supply_calculation() {
    let def = create_test_definition();
    assert_eq!(def.total_supply().unwrap(), 100_000_000_000); // 1000 × 100M
}

#[test]
fn test_total_supply_overflow() {
    // Create definition with values that would overflow
    let def = AssetDefinition {
        id: [0u8; 32],
        class: AssetClass::Coin,
        name: "Overflow Test".to_string(),
        symbol: "OVF".to_string(),
        decimals: 8,
        serials: u32::MAX, // 4,294,967,295
        nominal: u64::MAX, // Will overflow when multiplied
        domain_name: "test.io".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: 0,
        metadata: None,
    };

    let result = def.total_supply();
    assert!(result.is_err());
    match result {
        Err(AssetError::ArithmeticOverflow(msg)) => {
            assert!(msg.contains("total_supply overflow"));
        }
        _ => panic!("Expected ArithmeticOverflow error"),
    }
}

#[test]
fn test_serial_supply() {
    let def = create_test_definition();
    assert_eq!(def.serial_supply(), 100_000_000);
}

#[test]
fn test_is_burnable() {
    let def = create_test_definition();
    assert!(def.is_burnable());
}

#[test]
fn test_validation_zero_serials() {
    let result = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Test".to_string(),
        "TST".to_string(),
        8,
        0, // Invalid: serials = 0
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_validation_zero_nominal() {
    let result = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Test".to_string(),
        "TST".to_string(),
        8,
        1000,
        0, // Invalid: nominal = 0
        "test.io".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_validation_nft_decimals() {
    let result = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Nft,
        "NFT".to_string(),
        "NFT".to_string(),
        8, // Invalid: NFT must have 0 decimals
        1,
        1,
        "nft.io".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_validation_name_too_long() {
    let long_name = "A".repeat(65); // 65 > 64
    let result = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        long_name,
        "TST".to_string(),
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_validation_symbol_too_long() {
    let long_symbol = "A".repeat(17); // 17 > 16
    let result = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Test".to_string(),
        long_symbol,
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_domain_requires_dot_format() {
    let result = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Test".to_string(),
        "TST".to_string(),
        8,
        1000,
        100_000_000,
        "test_coin".to_string(),
        1,
        1,
        0,
        None,
    );
    assert!(matches!(result, Err(AssetError::InvalidAsset(_))));
}

#[test]
fn test_flags_mintable() {
    let def = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Token,
        "Test".to_string(),
        "TST".to_string(),
        8,
        1,
        1_000_000,
        "test.io".to_string(),
        1,
        1,
        MINTABLE, // Mintable flag (bit 2)
        None,
    )
    .unwrap();
    assert!(def.is_mintable());
}

#[test]
fn test_yaml_compatible_aliases() {
    // Test that YAML-compatible method names work correctly
    let def = AssetDefinition::new(
        [0u8; 32],
        AssetClass::Coin,
        "Test".to_string(),
        "TST".to_string(),
        8,
        1000,
        1_000_000,
        "test.io".to_string(),
        1,
        1,
        GAS | FUNGIBLE | MINTABLE | BURNABLE, // All flags
        None,
    )
    .unwrap();

    // Test all YAML-compatible aliases
    assert!(def.gas());
    assert!(def.fungible());
    assert!(def.mintable());
    assert!(def.burnable());

    // Verify they match original methods
    assert_eq!(def.gas(), def.is_gas());
    assert_eq!(def.fungible(), def.is_fungible());
    assert_eq!(def.mintable(), def.is_mintable());
    assert_eq!(def.burnable(), def.is_burnable());
}

#[test]
fn test_id_stable() {
    let first = create_test_definition();
    let second = create_test_definition();

    assert_eq!(first.id, second.id);
}

#[test]
fn test_id_changes() {
    let base = create_test_definition();
    let changed = AssetDefinition::new(
        [42u8; 32],
        AssetClass::Coin,
        "Test Coin".to_string(),
        "TS2".to_string(),
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        BURNABLE,
        None,
    )
    .expect("valid changed definition");

    assert_ne!(base.id, changed.id);
}

#[test]
fn test_meta_changes() {
    let mut left_meta = BTreeMap::new();
    left_meta.insert("ticker".to_string(), "TST".to_string());

    let mut right_meta = BTreeMap::new();
    right_meta.insert("ticker".to_string(), "ALT".to_string());

    let left = AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "Test Coin".to_string(),
        "TST".to_string(),
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        BURNABLE,
        Some(left_meta),
    )
    .expect("valid left definition");

    let right = AssetDefinition::new(
        [1u8; 32],
        AssetClass::Coin,
        "Test Coin".to_string(),
        "TST".to_string(),
        8,
        1000,
        100_000_000,
        "test.io".to_string(),
        1,
        1,
        BURNABLE,
        Some(right_meta),
    )
    .expect("valid right definition");

    assert_ne!(left.id, right.id);
}

#[test]
fn test_mismatch_rejected() {
    let def = AssetDefinition {
        id: [7u8; 32],
        class: AssetClass::Coin,
        name: "Test Coin".to_string(),
        symbol: "TST".to_string(),
        decimals: 8,
        serials: 1000,
        nominal: 100_000_000,
        domain_name: "test.io".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: BURNABLE,
        metadata: None,
    };

    assert!(matches!(def.validate_id(), Err(AssetError::Integrity(_))));
}

#[test]
fn test_reserved_policy_bits_rejected() {
    let def = AssetDefinition {
        id: [7u8; 32],
        class: AssetClass::Coin,
        name: "Test Coin".to_string(),
        symbol: "TST".to_string(),
        decimals: 8,
        serials: 1000,
        nominal: 100_000_000,
        domain_name: "test.io".to_string(),
        version: 1,
        crypto_version: 1,
        policy_flags: 0b1000_0000,
        metadata: None,
    };

    assert!(matches!(def.validate(), Err(AssetError::InvalidAsset(_))));
}
