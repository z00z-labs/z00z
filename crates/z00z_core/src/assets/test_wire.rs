use super::*;
use crate::assets::leaf::AssetPackPlain;
use crate::assets::serial_id::{deserialize_serial_id, serialize_serial_id, SerialIdError};
use crate::assets::AssetClass;
use std::sync::Arc;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::expert::keys::RistrettoSecretKey;
use z00z_crypto::expert::traits::SecretKeyTrait;
use z00z_crypto::vendor::tari::PedersenCommitmentFactory;
use z00z_crypto::HomomorphicCommitmentFactory;
use z00z_crypto::Z00ZScalar;
use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_utils::rng::SystemRngProvider;
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

fn test_time() -> SystemTimeProvider {
    SystemTimeProvider
}

fn derive_test_nonce(
    rng: &mut (impl rand::RngCore + rand::CryptoRng),
    time_provider: &dyn TimeProvider,
) -> [u8; 32] {
    crate::assets::nonce::derive_nonce_minimal(rng, time_provider).expect("test nonce")
}

fn create_test_definition(id: u8) -> AssetDefinition {
    AssetDefinition::new(
        [id; 32],
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
        None,
    )
    .expect("valid test definition")
}

fn create_test_asset(id: u8) -> Asset {
    let def = create_test_definition(id);
    let arc_def = GLOBAL_ASSET_REGISTRY.insert(def).unwrap();

    let blinding = Z00ZScalar::random(&mut SystemRngProvider.rng()).unwrap();

    Asset::new(
        arc_def,
        1,
        1000,
        &blinding,
        derive_test_nonce(&mut SystemRngProvider.rng(), &test_time()),
        &mut SystemRngProvider.rng(),
    )
    .expect("asset creation should succeed")
}

#[test]
fn test_from_asset() {
    let asset = create_test_asset(1);
    let wire = AssetWire::from_asset(&asset);

    assert_eq!(wire.definition.id, asset.definition.id);
    assert_eq!(wire.serial_id, asset.serial_id);
    assert_eq!(wire.amount, asset.amount);
    assert_eq!(wire.nonce, asset.nonce);
}

#[test]
fn test_from_asset_scrubs_secret() {
    let mut asset = create_test_asset(17);
    asset.secret = Some([9u8; 32]);

    let wire = AssetWire::from_asset(&asset);

    assert!(
        wire.secret.is_none(),
        "AssetWire must not export runtime secret material"
    );
}

#[test]
fn test_to_asset() {
    let asset = create_test_asset(2);
    let wire = AssetWire::from_asset(&asset);

    let asset2 = wire.to_asset().unwrap();

    assert_eq!(asset2.definition.id, asset.definition.id);
    assert_eq!(asset2.serial_id, asset.serial_id);
    assert_eq!(asset2.amount, asset.amount);
}

#[test]
fn test_round_trip_bincode() {
    let asset = create_test_asset(3);
    let wire = AssetWire::from_asset(&asset);

    let codec = BincodeCodec;
    let bytes = codec.serialize(&wire).unwrap();
    let decoded: AssetWire = codec.deserialize(&bytes).unwrap();

    assert_eq!(decoded.definition.id, wire.definition.id);
    assert_eq!(decoded.serial_id, wire.serial_id);
    assert_eq!(decoded.amount, wire.amount);
}

#[test]
fn test_validate() {
    let asset = create_test_asset(4);
    let wire = AssetWire::from_asset(&asset);

    assert!(wire.validate().is_ok());
}

#[test]
fn test_validate_invalid_serial_id() {
    let asset = create_test_asset(5);
    let mut wire = AssetWire::from_asset(&asset);
    wire.serial_id = 1001;

    match wire.validate() {
        Err(AssetError::InvalidSerialId {
            serial_id,
            max_allowed,
        }) => {
            assert_eq!(serial_id, 1001);
            assert_eq!(max_allowed, 1000);
        }
        _ => panic!("Expected InvalidSerialId"),
    }
}

#[test]
fn test_check_serial_error_mapping() {
    let err = check_serial(5, 3).unwrap_err();
    match err {
        AssetError::InvalidSerialId {
            serial_id,
            max_allowed,
        } => {
            assert_eq!(serial_id, 5);
            assert_eq!(max_allowed, 3);
        }
        _ => panic!("Expected InvalidSerialId"),
    }
}

#[test]
fn test_serial_id_compat_roundtrip() {
    let serial_id = 42u32;
    let bytes = serialize_serial_id(serial_id);
    let decoded = deserialize_serial_id(&bytes).expect("deserialize serial_id");
    assert_eq!(decoded, serial_id);
}

#[test]
fn test_serial_id_invalid_length() {
    let err = deserialize_serial_id(&[1, 2, 3]).unwrap_err();
    assert!(matches!(
        err,
        SerialIdError::InvalidLength {
            expected: 4,
            got: 3
        }
    ));
}

#[test]
fn test_verify_asset_pack_encoding() {
    let value = 12345u64;
    let blinding = [7u8; 32];
    let s_out = [9u8; 32];
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&value.to_le_bytes());
    bytes.extend_from_slice(&blinding);
    bytes.extend_from_slice(&s_out);

    let decoded = verify_asset_pack_encoding(&bytes).expect("asset pack decode");
    assert_eq!(decoded.value, value);
    assert_eq!(decoded.blinding, blinding);
    assert_eq!(decoded.s_out, s_out);
}

#[test]
fn test_verify_asset_short_bytes() {
    let bytes = vec![0u8; AssetPackPlain::SIZE - 1];
    let err = verify_asset_pack_encoding(&bytes).unwrap_err();
    assert_eq!(err, AssetPackError::TruncatedAssetPack);
}

#[test]
fn test_asset_pkg_wire_roundtrip() {
    let asset = create_test_asset(6);
    let pkg = AssetPkgWire::from_asset(&asset);
    let codec = BincodeCodec;
    let bytes = codec.serialize(&pkg).expect("serialize pkg");
    let decoded: AssetPkgWire = codec.deserialize(&bytes).expect("deserialize pkg");

    assert_eq!(decoded.serial_id, pkg.serial_id);
    assert_eq!(decoded.amount, pkg.amount);
    assert_eq!(decoded.definition.id, pkg.definition.id);
}

#[test]
fn test_from_asset_not_arc() {
    let asset = create_test_asset(10);
    let wire = AssetWire::from_asset(&asset);

    assert_eq!(wire.definition.id, asset.definition.id);
    assert_eq!(wire.definition.name, asset.definition.name);

    let mut wire2 = wire.clone();
    wire2.definition.name = "Modified".to_string();
    assert_ne!(wire2.definition.name, asset.definition.name);
    assert_eq!(asset.definition.name, "Test Coin");
}

#[test]
fn test_to_asset_registry_arc() {
    let asset = create_test_asset(11);
    let wire = AssetWire::from_asset(&asset);

    let asset2 = wire.to_asset().unwrap();

    assert_eq!(asset2.definition.id, asset.definition.id);
    assert!(Arc::ptr_eq(&asset2.definition, &asset.definition));
}

#[test]
fn test_to_asset_serial_id() {
    let mut wire = AssetWire::from_asset(&create_test_asset(12));
    wire.serial_id = wire.definition.serials;

    let result = wire.to_asset();
    assert!(result.is_err());
}

#[test]
fn test_wire_secret_gate() {
    let mut wire = AssetWire::from_asset(&create_test_asset(15));
    wire.secret = Some([9u8; 32]);

    let result = wire.to_asset();
    assert!(matches!(result, Err(AssetError::InvalidAsset(_))));
}

#[test]
fn test_wire_proof_gate() {
    let mut wire = AssetWire::from_asset(&create_test_asset(16));
    wire.owner_pub = None;
    wire.owner_signature = None;
    wire.range_proof = None;

    let result = wire.to_asset();
    assert!(matches!(result, Err(AssetError::MissingRangeProof)));
}

#[test]
fn test_asset_wire_proof_roundtrip() {
    let rng = &mut SystemRngProvider.rng();
    let def = create_test_definition(13);
    let arc_def = GLOBAL_ASSET_REGISTRY.insert(def).unwrap();

    let blinding = Z00ZScalar::random(rng).unwrap();
    let asset = Asset::new(
        arc_def,
        1,
        1000,
        &blinding,
        derive_test_nonce(rng, &test_time()),
        rng,
    )
    .unwrap();

    let wire = AssetWire::from_asset(&asset);
    assert!(wire.range_proof.is_some());

    let codec = BincodeCodec;
    let bytes = codec.serialize(&wire).unwrap();
    let decoded: AssetWire = codec.deserialize(&bytes).unwrap();

    assert!(decoded.range_proof.is_some());
    assert_eq!(decoded.amount, 1000);
}

#[test]
fn test_verify_complete_roundtrip_asset() {
    let asset = create_test_asset(14);
    let wire = AssetWire::from_asset(&asset);
    let asset2 = wire.to_asset().unwrap();

    asset2.verify_complete().unwrap();
}

#[test]
fn test_asset_pack_matches_helper() {
    let value = 77u64;
    let blinding = [3u8; 32];
    let s_out = [4u8; 32];

    let plain = AssetPackPlain {
        value,
        blinding,
        s_out,
    };

    let plain_bytes = plain.to_bytes();
    let decoded = verify_asset_pack_encoding(&plain_bytes).expect("decode AssetPackPlain bytes");
    assert_eq!(decoded.value, value);
    assert_eq!(decoded.blinding, blinding);
    assert_eq!(decoded.s_out, s_out);
}

#[test]
fn test_asset_pack_commitment_inputs() {
    let rng = &mut SystemRngProvider.rng();
    let blinding = RistrettoSecretKey::random(rng);
    let value = 4242u64;

    let mut blinding_bytes = [0u8; 32];
    blinding_bytes.copy_from_slice(blinding.as_bytes());

    let s_out = [11u8; 32];
    let plain = AssetPackPlain {
        value,
        blinding: blinding_bytes,
        s_out,
    };

    let plain_bytes = plain.to_bytes();
    let decoded = verify_asset_pack_encoding(&plain_bytes).expect("decode AssetPackPlain bytes");

    let commitment_direct = Commitment::from_commitment(
        PedersenCommitmentFactory::default().commit_value(&blinding, value),
    );

    assert_eq!(decoded.blinding, blinding_bytes);
    assert_eq!(decoded.value, value);
    assert_eq!(decoded.s_out, s_out);
    assert_eq!(
        commitment_direct,
        Commitment::from_commitment(
            PedersenCommitmentFactory::default().commit_value(&blinding, decoded.value),
        )
    );
}

#[test]
fn test_asset_pack_original_fields() {
    let value = 1_234_567u64;
    let blinding = [0xAB; 32];
    let s_out = [0xCD; 32];

    let plain = AssetPackPlain {
        value,
        blinding,
        s_out,
    };

    let plain_bytes = plain.to_bytes();
    let decoded = verify_asset_pack_encoding(&plain_bytes).expect("decode AssetPackPlain bytes");

    assert_eq!(decoded.value, value);
    assert_eq!(decoded.blinding, blinding);
    assert_eq!(decoded.s_out, s_out);
}

#[test]
fn test_asset_pack_extra_bytes() {
    let mut bytes = vec![0u8; AssetPackPlain::SIZE + 1];
    bytes[0] = 1;

    let err = verify_asset_pack_encoding(&bytes).unwrap_err();
    assert_eq!(err, AssetPackError::TruncatedAssetPack);
}

#[test]
fn test_asset_pack_is_allowed() {
    let plain = AssetPackPlain {
        value: 0,
        blinding: [0u8; 32],
        s_out: [0u8; 32],
    };

    let plain_bytes = plain.to_bytes();
    let decoded =
        verify_asset_pack_encoding(&plain_bytes).expect("decode zero-value AssetPackPlain bytes");
    assert_eq!(decoded.value, 0);
}

#[test]
fn test_asset_pack_u64_value() {
    let plain = AssetPackPlain {
        value: u64::MAX,
        blinding: [0x11; 32],
        s_out: [0x22; 32],
    };

    let plain_bytes = plain.to_bytes();
    let decoded =
        verify_asset_pack_encoding(&plain_bytes).expect("decode max-value AssetPackPlain bytes");
    assert_eq!(decoded.value, u64::MAX);
    assert_eq!(decoded.blinding, [0x11; 32]);
    assert_eq!(decoded.s_out, [0x22; 32]);
}
