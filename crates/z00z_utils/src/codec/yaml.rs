//! YAML codec implementation

use super::traits::{Codec, CodecError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Mutex;

// `serde_yaml` sits on top of libyaml. In the workspace-wide release test run
// we observed non-deterministic decode drift under parallel YAML-heavy tests,
// so gate codec calls through one process-local lock.
static YAML_CODEC_LOCK: Mutex<()> = Mutex::new(());

/// YAML codec for human-readable configuration files
///
/// Serializes/deserializes data to/from YAML format.
/// Useful for configuration files and documentation.
///
/// # Examples
///
/// ```no_run
/// use z00z_utils::codec::{Codec, YamlCodec};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     server: String,
///     port: u16,
/// }
///
/// let codec = YamlCodec;
/// let config = Config {
///     server: "localhost".into(),
///     port: 8080,
/// };
///
/// let bytes = codec.serialize(&config)?;
/// let yaml_str = String::from_utf8(bytes)?;
/// assert!(yaml_str.contains("server:"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct YamlCodec;

impl Codec for YamlCodec {
    type Error = CodecError;

    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
        let _guard = YAML_CODEC_LOCK
            .lock()
            .expect("yaml codec lock must not be poisoned");
        let yaml_str = serde_yaml::to_string(value).map_err(|e| CodecError::Yaml(e.to_string()))?;
        Ok(yaml_str.into_bytes())
    }

    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, Self::Error> {
        let _guard = YAML_CODEC_LOCK
            .lock()
            .expect("yaml codec lock must not be poisoned");
        let yaml_str = std::str::from_utf8(bytes).map_err(|e| CodecError::Yaml(e.to_string()))?;

        // serde_yaml enforces a single-document input. We still explicitly reject
        // additional documents to surface a consistent "trailing data" error.
        let mut docs = serde_yaml::Deserializer::from_str(yaml_str);
        let first = docs
            .next()
            .ok_or_else(|| CodecError::Yaml("empty YAML input".to_string()))?;

        let value = T::deserialize(first).map_err(|e| CodecError::Yaml(e.to_string()))?;

        if docs.next().is_some() {
            return Err(CodecError::TrailingBytes {
                consumed: 0,
                total: bytes.len(),
            });
        }

        Ok(value)
    }

    fn name(&self) -> &'static str {
        "yaml"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        server: String,
        port: u16,
        debug: bool,
    }

    #[test]
    fn test_yaml_codec_serialize() {
        let codec = YamlCodec;
        let config = Config {
            server: "localhost".to_string(),
            port: 8080,
            debug: true,
        };

        let bytes = codec.serialize(&config).unwrap();
        let yaml_str = String::from_utf8(bytes).unwrap();

        assert!(yaml_str.contains("server:"));
        assert!(yaml_str.contains("localhost"));
        assert!(yaml_str.contains("port:"));
        assert!(yaml_str.contains("8080"));
    }

    #[test]
    fn test_yaml_codec_deserialize() {
        let codec = YamlCodec;
        let yaml = r#"
server: example.com
port: 9000
debug: false
"#;

        let result: Config = codec.deserialize(yaml.as_bytes()).unwrap();
        assert_eq!(result.server, "example.com");
        assert_eq!(result.port, 9000);
        assert!(!result.debug);
    }

    #[test]
    fn test_yaml_multi_doc_rejected() {
        let codec = YamlCodec;
        let yaml = "server: a\nport: 1\ndebug: false\n---\nserver: b\nport: 2\ndebug: true\n";

        let result: Result<Config, _> = codec.deserialize(yaml.as_bytes());
        assert!(matches!(result, Err(CodecError::TrailingBytes { .. })));
    }

    #[test]
    fn test_yaml_trailing_garbage_rejected() {
        let codec = YamlCodec;
        let yaml = "server: a\nport: 1\ndebug: false\n---\nGARBAGE";

        let result: Result<Config, _> = codec.deserialize(yaml.as_bytes());
        assert!(matches!(result, Err(CodecError::TrailingBytes { .. })));
    }

    #[test]
    fn test_yaml_codec_round_trip() {
        let codec = YamlCodec;
        let original = Config {
            server: "production.example.com".to_string(),
            port: 443,
            debug: false,
        };

        let bytes = codec.serialize(&original).unwrap();
        let deserialized: Config = codec.deserialize(&bytes).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_yaml_codec_name() {
        let codec = YamlCodec;
        assert_eq!(codec.name(), "yaml");
    }

    #[test]
    fn test_yaml_codec_nested_structures() {
        let codec = YamlCodec;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct NestedConfig {
            database: DatabaseConfig,
            logging: LoggingConfig,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct DatabaseConfig {
            host: String,
            port: u16,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct LoggingConfig {
            level: String,
            file: String,
        }

        let config = NestedConfig {
            database: DatabaseConfig {
                host: "db.example.com".to_string(),
                port: 5432,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: "/var/log/app.log".to_string(),
            },
        };

        let bytes = codec.serialize(&config).unwrap();
        let decoded: NestedConfig = codec.deserialize(&bytes).unwrap();
        assert_eq!(config, decoded);
    }

    #[test]
    fn test_yaml_codec_with_collections() {
        let codec = YamlCodec;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct DataWithList {
            items: Vec<String>,
            tags: Vec<i32>,
        }

        let data = DataWithList {
            items: vec![
                "first".to_string(),
                "second".to_string(),
                "third".to_string(),
            ],
            tags: vec![1, 2, 3, 4, 5],
        };

        let bytes = codec.serialize(&data).unwrap();
        let decoded: DataWithList = codec.deserialize(&bytes).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_yaml_codec_error_handling() {
        let codec = YamlCodec;
        let invalid_yaml = b"[{invalid: yaml: content:}";

        let result: Result<Config, _> = codec.deserialize(invalid_yaml);
        assert!(result.is_err());
        match result.unwrap_err() {
            CodecError::Yaml(_) => {} // Expected
            _ => panic!("Expected Yaml error"),
        }
    }

    #[test]
    fn test_yaml_codec_pretty() {
        let codec = YamlCodec;
        let config = Config {
            server: "test.com".to_string(),
            port: 443,
            debug: true,
        };

        let bytes = codec.serialize(&config).unwrap();
        let yaml_str = String::from_utf8(bytes).unwrap();
        // YAML is naturally readable
        assert!(yaml_str.contains("server:"));
        assert!(yaml_str.contains("port:"));
    }
}
