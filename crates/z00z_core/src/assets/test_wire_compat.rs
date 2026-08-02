use super::*;
use crate::assets::AssetClass;
use z00z_crypto::Z00ZScalar;
use z00z_utils::codec::{json, Codec, JsonCodec};
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
fn test_definition_rejects_tampered_id() {
    let definition = create_test_definition(90);
    let mut wire = DefinitionWire::from(&definition);
    wire.id[0] ^= 0xFF;

    let err = AssetDefinition::try_from(wire).expect_err("tampered definition id must fail");

    assert!(matches!(err, AssetError::Integrity(_)));
}

#[test]
fn test_package_wire_preserves_flags() {
    let mut asset = create_test_asset(81);
    asset.is_frozen = true;
    asset.is_slashed = true;

    let pkg = AssetPkgWire::from_asset(&asset);
    let wire = pkg.to_wire().expect("pkg to wire");

    assert!(wire.is_frozen);
    assert!(wire.is_slashed);
    assert!(wire.secret.is_none());
}

#[test]
fn test_package_json_preserves_flags() {
    let mut asset = create_test_asset(82);
    asset.is_frozen = true;
    asset.is_slashed = true;

    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let decoded = decode_asset_pkg_json(&bytes).expect("decode dto json");
    let wire = decoded.to_wire().expect("dto to wire");

    assert!(wire.is_frozen);
    assert!(wire.is_slashed);
}

#[test]
fn test_package_rejects_secret_wire() {
    let mut wire = AssetWire::from_asset(&create_test_asset(83));
    wire.secret = Some([7u8; 32]);

    let err = AssetPkgWire::try_from_wire(&wire).expect_err("secret-bearing wire must fail");

    assert!(matches!(err, AssetError::InvalidAsset(_)));
}

#[test]
fn test_rejects_secret_decode_helper() {
    let pkg = AssetPkgWire::from_asset(&create_test_asset(84));
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;
    let mut value: z00z_utils::codec::Value = codec.deserialize(&bytes).expect("decode dto value");
    let object = value.as_object_mut().expect("dto object");
    object.insert("secret".to_string(), json!(hex::encode([9u8; 32])));
    let bad = codec.serialize(&value).expect("encode bad dto value");

    let err = decode_asset_pkg_json(&bad).expect_err("secret field must fail");

    assert!(err.to_string().contains("forbidden field: secret"));
}

#[test]
fn test_secret_probe_clean() {
    let asset = create_test_asset(72);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");

    let has_secret = payload_has_secret_field(&bytes).expect("probe secret field");

    assert!(!has_secret);
}

#[test]
fn test_secret_probe_hits() {
    let asset = create_test_asset(73);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;
    let mut value: z00z_utils::codec::Value = codec.deserialize(&bytes).expect("decode dto value");
    let root = value.as_object_mut().expect("dto object");
    root.insert("secret".to_string(), json!(hex::encode([9u8; 32])));
    let bad = codec.serialize(&value).expect("encode bad dto value");

    let has_secret = payload_has_secret_field(&bad).expect("probe bad dto json");

    assert!(has_secret);
}

#[test]
fn test_secret_probe_null() {
    let asset = create_test_asset(79);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;
    let mut value: z00z_utils::codec::Value = codec.deserialize(&bytes).expect("decode dto value");
    let root = value.as_object_mut().expect("dto object");
    root.insert("secret".to_string(), z00z_utils::codec::Value::Null);
    let bad = codec.serialize(&value).expect("encode bad dto value");

    let has_secret = payload_has_secret_field(&bad).expect("probe null secret");

    assert!(has_secret);
}

#[test]
fn test_local_flags_when_false() {
    let asset = create_test_asset(74);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let text = String::from_utf8(bytes).expect("dto utf8");

    assert!(!text.contains("\"secret\""));
    assert!(!text.contains("\"is_frozen\""));
    assert!(!text.contains("\"is_slashed\""));
}

#[test]
fn test_dto_asset_roundtrip() {
    let asset = create_test_asset(75);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");

    let decoded = decode_asset_pkg_json(&bytes).expect("decode dto json");
    let roundtrip = decoded.to_asset().expect("dto to asset");

    assert_eq!(roundtrip.asset_id(), asset.asset_id());
    assert_eq!(roundtrip.amount, asset.amount);
}

#[test]
fn test_asset_package_json_roundtrip() {
    let asset = create_test_asset(7);
    let pkg = AssetPkgWire::from_asset(&asset);
    let codec = JsonCodec;

    let text = codec.serialize(&pkg).expect("serialize json");
    let decoded: AssetPkgWire = codec.deserialize(&text).expect("deserialize json");

    assert_eq!(decoded.serial_id, pkg.serial_id);
    assert_eq!(decoded.amount, pkg.amount);
    assert_eq!(decoded.definition.id, pkg.definition.id);
}

#[test]
fn test_pkg_json_roundtrip() {
    let asset = create_test_asset(71);
    let pkg = AssetPkgWire::from_asset(&asset);

    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let decoded = decode_asset_pkg_json(&bytes).expect("decode dto json");

    assert_eq!(decoded.serial_id, pkg.serial_id);
    assert_eq!(decoded.amount, pkg.amount);
    assert_eq!(decoded.definition.id, pkg.definition.id);
}

#[test]
fn test_dto_rejects_unknown() {
    let asset = create_test_asset(76);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;
    let mut value: z00z_utils::codec::Value = codec.deserialize(&bytes).expect("decode dto value");
    let root = value.as_object_mut().expect("dto object");
    root.insert("unexpected_field".to_string(), json!(1));
    let bad = codec.serialize(&value).expect("encode bad dto value");

    let err = decode_asset_pkg_json(&bad).expect_err("unknown field must fail");

    assert!(matches!(err, AssetError::InvalidAsset(_)));
}

#[test]
fn test_dto_rejects_local() {
    let asset = create_test_asset(78);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;

    for value in [
        json!(hex::encode([3u8; 32])),
        z00z_utils::codec::Value::Null,
    ] {
        let mut root: z00z_utils::codec::Value =
            codec.deserialize(&bytes).expect("decode dto value");
        let object = root.as_object_mut().expect("dto object");
        object.insert("secret".to_string(), value);
        let bad = codec.serialize(&root).expect("encode bad dto value");

        assert!(payload_has_secret_field(&bad).expect("probe secret field"));

        let err = decode_asset_pkg_json(&bad).expect_err("local field must fail");
        assert!(matches!(err, AssetError::InvalidAsset(_)));
    }
}

#[test]
fn test_rejects_id_full_stealth() {
    let asset = create_test_asset(80);
    let mut pkg = AssetPkgWire::from_asset(&asset);
    pkg.r_pub = Some([1u8; 32]);
    pkg.owner_tag = Some([2u8; 32]);
    pkg.enc_pack = Some(ZkPackEncrypted {
        version: 1,
        ciphertext: vec![3u8; 72],
        tag: [0u8; 16],
    });
    pkg.tag16 = Some(7);
    pkg.leaf_ad_id = Some([4u8; 32]);

    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;
    let mut value: z00z_utils::codec::Value = codec.deserialize(&bytes).expect("decode dto value");
    let object = value.as_object_mut().expect("dto object");
    object.remove("leaf_ad_id");
    let bad = codec.serialize(&value).expect("encode bad dto value");

    let err = decode_asset_pkg_json(&bad).expect_err("missing leaf_ad_id must fail at dto decode");
    assert!(matches!(err, AssetError::InvalidAsset(_)));
    assert!(err
        .to_string()
        .contains("full stealth fields require leaf_ad_id"));
}

#[test]
fn test_dto_hex_json() {
    let asset = create_test_asset(77);
    let pkg = AssetPkgWire::from_asset(&asset);
    let bytes = encode_asset_pkg_json(&pkg).expect("encode dto json");
    let codec = JsonCodec;
    let value: z00z_utils::codec::Value = codec.deserialize(&bytes).expect("decode dto value");
    let root = value.as_object().expect("dto object");

    assert!(root.get("nonce").and_then(|field| field.as_str()).is_some());
    assert!(root
        .get("commitment")
        .and_then(|field| field.as_str())
        .is_some());
}

#[test]
fn test_pkg_wire_local_clear() {
    let asset = create_test_asset(8);
    let pkg = AssetPkgWire::from_asset(&asset);
    let wire = pkg.to_wire().expect("pkg to wire");

    assert!(!wire.is_frozen);
    assert!(!wire.is_slashed);
    assert!(wire.secret.is_none());
    assert_eq!(wire.serial_id, asset.serial_id);
}

#[test]
fn test_asset_pkg_wire_validate() {
    let asset = create_test_asset(9);
    let pkg = AssetPkgWire::from_asset(&asset);

    assert!(pkg.validate().is_ok());
}

#[test]
fn test_asset_package_rejects_hex() {
    let bad = json!({
        "definition": {
            "id": "zz",
            "class": "Coin",
            "name": "Bad",
            "symbol": "BAD",
            "decimals": 8,
            "serials": 1,
            "nominal": 1,
            "domain_name": "bad.test",
            "version": 1,
            "crypto_version": 1,
            "policy_flags": 0,
            "metadata": null
        },
        "serial_id": 0,
        "amount": 1,
        "commitment": hex::encode([1u8; 32]),
        "range_proof": null,
        "nonce": hex::encode([2u8; 32]),
        "lock_height": null,
        "is_burned": false,
        "owner_pub": null,
        "owner_signature": null,
        "r_pub": null,
        "owner_tag": null,
        "enc_pack": null,
        "tag16": null
    });
    let codec = JsonCodec;
    let bad_bytes = codec.serialize(&bad).expect("serialize bad json");

    let err = codec.deserialize::<AssetPkgWire>(&bad_bytes).unwrap_err();
    assert!(err.to_string().contains("definition.id: invalid hex"));
}
