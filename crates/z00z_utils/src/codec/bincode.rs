//! Bincode codec implementation using bincode v2.0 API
//!
//! This implementation provides compact binary serialization using bincode v2.0.

use super::traits::{Codec, CodecError};
use serde::{de::DeserializeOwned, de::DeserializeSeed, Serialize};

const LIMIT_1MB_BYTES: usize = 1024 * 1024;
const LIMIT_10MB_BYTES: usize = 10 * 1024 * 1024;
const LIMIT_24MB_BYTES: usize = 24 * 1024 * 1024;
const LIMIT_48MB_BYTES: usize = 48 * 1024 * 1024;
const LIMIT_100MB_BYTES: usize = 100 * 1024 * 1024;

const DEFAULT_MAX_DESERIALIZE_SIZE: u64 = LIMIT_10MB_BYTES as u64;

/// Bincode codec for compact binary serialization
///
/// Uses bincode v2.0 with standard configuration for efficient binary encoding.
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::codec::{Codec, BincodeCodec};
///
/// let codec = BincodeCodec;
/// let data = vec![1, 2, 3];
/// let bytes = codec.serialize(&data)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BincodeCodec;

impl BincodeCodec {
    /// Encode one value with the frozen legacy bincode wire used by imported
    /// cryptographic artifacts.
    pub fn serialize_legacy<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, CodecError> {
        bincode::serde::encode_to_vec(value, bincode::config::legacy())
            .map_err(|error| CodecError::Bincode(error.to_string()))
    }

    /// Decode one exact legacy-wire value under a compile-time ceiling.
    ///
    /// New project-owned objects use the standard configuration. This method
    /// exists only for frozen dependency artifacts whose canonical wire was
    /// already selected as legacy bincode.
    pub fn deserialize_legacy_bounded<T: DeserializeOwned, const LIMIT: usize>(
        &self,
        bytes: &[u8],
    ) -> Result<T, CodecError> {
        if bytes.len() > LIMIT {
            return Err(CodecError::DeserializeSizeLimitExceeded {
                size: bytes.len(),
                limit: LIMIT as u64,
            });
        }
        let (value, consumed): (T, usize) = bincode::serde::decode_from_slice(
            bytes,
            bincode::config::legacy().with_limit::<LIMIT>(),
        )
        .map_err(|error| CodecError::Bincode(error.to_string()))?;
        if consumed != bytes.len() {
            return Err(CodecError::TrailingBytes {
                consumed,
                total: bytes.len(),
            });
        }
        Ok(value)
    }

    /// Decode one exact bincode value under a compile-time allocation ceiling.
    ///
    /// The const ceiling is part of the call site instead of an attacker-controlled
    /// runtime value. This is the preferred entry point for new persisted objects.
    pub fn deserialize_bounded_const<T: DeserializeOwned, const LIMIT: usize>(
        &self,
        bytes: &[u8],
    ) -> Result<T, CodecError> {
        if bytes.len() > LIMIT {
            return Err(CodecError::DeserializeSizeLimitExceeded {
                size: bytes.len(),
                limit: LIMIT as u64,
            });
        }
        let (value, consumed): (T, usize) = bincode::serde::decode_from_slice(
            bytes,
            bincode::config::standard().with_limit::<LIMIT>(),
        )
        .map_err(|error| CodecError::Bincode(error.to_string()))?;
        if consumed != bytes.len() {
            return Err(CodecError::TrailingBytes {
                consumed,
                total: bytes.len(),
            });
        }
        Ok(value)
    }

    /// Decode one exact value through a caller-owned seed under a compile-time
    /// ceiling. The seed is the only supported path for recursive evidence that
    /// needs construction-time bounds stronger than a derived `Deserialize` impl.
    pub fn deserialize_seeded_bounded<'de, S, const LIMIT: usize>(
        &self,
        bytes: &'de [u8],
        seed: S,
    ) -> Result<S::Value, CodecError>
    where
        S: DeserializeSeed<'de>,
    {
        if bytes.len() > LIMIT {
            return Err(CodecError::DeserializeSizeLimitExceeded {
                size: bytes.len(),
                limit: LIMIT as u64,
            });
        }
        let (value, consumed) = bincode::serde::seed_decode_from_slice(
            seed,
            bytes,
            bincode::config::standard().with_limit::<LIMIT>(),
        )
        .map_err(|error| CodecError::Bincode(error.to_string()))?;
        if consumed != bytes.len() {
            return Err(CodecError::TrailingBytes {
                consumed,
                total: bytes.len(),
            });
        }
        Ok(value)
    }

    /// Deserialize bytes into a value, enforcing a maximum input size.
    ///
    /// # Security
    ///
    /// Enforces an upper bound on the input size to reduce the risk of resource exhaustion
    /// attacks from untrusted payloads.
    pub fn deserialize_bounded<T: DeserializeOwned>(
        &self,
        bytes: &[u8],
        max_bytes: u64,
    ) -> Result<T, CodecError> {
        let limit = usize::try_from(max_bytes)
            .map_err(|_| CodecError::Bincode("invalid max_bytes for this target".to_string()))?;

        let size = bytes.len();
        if size > limit {
            return Err(CodecError::DeserializeSizeLimitExceeded {
                size,
                limit: max_bytes,
            });
        }

        match limit {
            LIMIT_1MB_BYTES => self.deserialize_bounded_const::<T, LIMIT_1MB_BYTES>(bytes),
            LIMIT_10MB_BYTES => self.deserialize_bounded_const::<T, LIMIT_10MB_BYTES>(bytes),
            LIMIT_24MB_BYTES => self.deserialize_bounded_const::<T, LIMIT_24MB_BYTES>(bytes),
            LIMIT_48MB_BYTES => self.deserialize_bounded_const::<T, LIMIT_48MB_BYTES>(bytes),
            LIMIT_100MB_BYTES => self.deserialize_bounded_const::<T, LIMIT_100MB_BYTES>(bytes),
            _ => Err(CodecError::Bincode(
                "unsupported max_bytes for bincode; use 1MB, 10MB, 24MB, 48MB, or 100MB"
                    .to_string(),
            )),
        }
    }
}

impl Codec for BincodeCodec {
    type Error = CodecError;

    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
        bincode::serde::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| CodecError::Bincode(e.to_string()))
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, Self::Error> {
        self.deserialize_bounded(bytes, DEFAULT_MAX_DESERIALIZE_SIZE)
    }

    fn name(&self) -> &'static str {
        "bincode"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

    struct U64Seed;

    impl<'de> DeserializeSeed<'de> for U64Seed {
        type Value = u64;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            u64::deserialize(deserializer)
        }
    }

    struct RejectSeed;

    impl<'de> DeserializeSeed<'de> for RejectSeed {
        type Value = u64;

        fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Err(D::Error::custom("seed rejected"))
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
        active: bool,
    }

    #[test]
    fn test_bincode_codec_serialize() {
        let codec = BincodeCodec;
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            active: true,
        };

        let bytes = codec.serialize(&data).unwrap();
        assert!(!bytes.is_empty());
        // Bincode should be compact
        assert!(bytes.len() < 100);
    }

    #[test]
    fn test_bincode_codec_deserialize() {
        let codec = BincodeCodec;
        let data = TestStruct {
            name: "hello".to_string(),
            value: 99,
            active: false,
        };

        let bytes = codec.serialize(&data).unwrap();
        let result: TestStruct = codec.deserialize(&bytes).unwrap();

        assert_eq!(result.name, "hello");
        assert_eq!(result.value, 99);
        assert!(!result.active);
    }

    #[test]
    fn test_bincode_codec_round_trip() {
        let codec = BincodeCodec;
        let original = TestStruct {
            name: "round-trip".to_string(),
            value: 123,
            active: true,
        };

        let bytes = codec.serialize(&original).unwrap();
        let deserialized: TestStruct = codec.deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_bincode_codec_name() {
        let codec = BincodeCodec;
        assert_eq!(codec.name(), "bincode");
    }

    #[test]
    fn test_bincode_codec_compact() {
        let codec = BincodeCodec;
        let data = vec![1u32; 1000];

        let bytes = codec.serialize(&data).unwrap();
        // Should serialize successfully
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_legacy_round_trip() {
        let value = TestStruct {
            name: "legacy".to_owned(),
            value: 7,
            active: true,
        };
        let bytes = BincodeCodec.serialize_legacy(&value).unwrap();
        let decoded = BincodeCodec
            .deserialize_legacy_bounded::<TestStruct, 1024>(&bytes)
            .unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_legacy_trailing_rejected() {
        let mut bytes = BincodeCodec.serialize_legacy(&7_u64).unwrap();
        bytes.push(0);
        assert!(BincodeCodec
            .deserialize_legacy_bounded::<u64, 1024>(&bytes)
            .is_err());
    }

    #[test]
    fn test_bincode_codec_types() {
        let codec = BincodeCodec;

        // Tuple
        let tuple = (42i32, "hello".to_string(), true);
        let bytes = codec.serialize(&tuple).unwrap();
        let decoded: (i32, String, bool) = codec.deserialize(&bytes).unwrap();
        assert_eq!(tuple, decoded);

        // Option
        let opt: Option<i32> = Some(42);
        let bytes = codec.serialize(&opt).unwrap();
        let decoded: Option<i32> = codec.deserialize(&bytes).unwrap();
        assert_eq!(opt, decoded);

        // Result
        let result: Result<i32, String> = Ok(42);
        let bytes = codec.serialize(&result).unwrap();
        let decoded: Result<i32, String> = codec.deserialize(&bytes).unwrap();
        assert_eq!(result, decoded);
    }

    #[test]
    fn test_bincode_codec_error_handling() {
        let codec = BincodeCodec;
        let invalid_data = b"not valid bincode";

        let result: Result<TestStruct, _> = codec.deserialize(invalid_data);
        assert!(result.is_err());
        match result.unwrap_err() {
            CodecError::Bincode(_) => {} // Expected
            _ => panic!("Expected Bincode error"),
        }
    }

    #[test]
    fn test_bincode_malicious_vec_rejected() {
        // Craft a payload that looks like a Vec<u8> with an absurdly large length prefix.
        // This should be rejected without attempting a huge allocation.
        let len_prefix = bincode::serde::encode_to_vec(u64::MAX, bincode::config::standard())
            .expect("encode len prefix");

        let result = std::panic::catch_unwind(|| {
            BincodeCodec.deserialize_bounded::<Vec<u8>>(&len_prefix, 1024)
        });

        assert!(result.is_ok(), "must not panic on malicious input");
        assert!(result.unwrap().is_err(), "malicious vec must be rejected");
    }

    #[test]
    fn test_bincode_rejects_oversize_vec() {
        let len_prefix = bincode::serde::encode_to_vec(
            u64::try_from(LIMIT_24MB_BYTES + 1).expect("fixed limit fits u64"),
            bincode::config::standard(),
        )
        .expect("encode len prefix");
        let result = std::panic::catch_unwind(|| {
            BincodeCodec.deserialize_bounded::<Vec<u8>>(&len_prefix, LIMIT_24MB_BYTES as u64)
        });
        assert!(result.is_ok(), "must not panic on declared oversized vec");
        assert!(
            result.unwrap().is_err(),
            "declared oversized vec must reject"
        );
    }

    #[test]
    fn test_bincode_trailing_bytes_rejected() {
        let codec = BincodeCodec;
        let value = 42u64;

        let mut bytes = codec.serialize(&value).unwrap();
        bytes.extend_from_slice(&[0xAA, 0xBB]);

        let result: Result<u64, _> = codec.deserialize(&bytes);
        assert!(matches!(result, Err(CodecError::TrailingBytes { .. })));
    }

    #[test]
    fn test_seeded_bound_propagates_errors() {
        let codec = BincodeCodec;
        let bytes = codec.serialize(&42_u64).unwrap();
        assert_eq!(
            codec
                .deserialize_seeded_bounded::<U64Seed, 1024>(&bytes, U64Seed)
                .unwrap(),
            42
        );

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(matches!(
            codec.deserialize_seeded_bounded::<U64Seed, 1024>(&trailing, U64Seed),
            Err(CodecError::TrailingBytes { .. })
        ));
        assert!(codec
            .deserialize_seeded_bounded::<RejectSeed, 1024>(&bytes, RejectSeed)
            .is_err());
        assert!(codec
            .deserialize_seeded_bounded::<U64Seed, 1>(&[0; 2], U64Seed)
            .is_err());
    }

    #[test]
    fn test_legacy_vec_length_rejects() {
        let hostile = BincodeCodec
            .serialize_legacy(&u64::MAX)
            .expect("legacy length prefix");
        let result = std::panic::catch_unwind(|| {
            BincodeCodec.deserialize_legacy_bounded::<Vec<u8>, 131_072>(&hostile)
        });
        assert!(result.is_ok(), "hostile legacy length must not panic");
        assert!(
            result.unwrap().is_err(),
            "hostile legacy length must reject"
        );
    }
}
