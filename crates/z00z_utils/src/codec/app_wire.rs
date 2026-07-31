//! Canonical bounded wire envelope for public Z00Z application contracts.

use thiserror::Error;

/// Four-byte frame discriminator.
pub const APP_WIRE_MAGIC: [u8; 4] = *b"ZAWF";
/// The only accepted Phase 001 wire version.
pub const APP_WIRE_VERSION: u16 = 1;
/// Maximum encoded frame size.
pub const MAX_APP_WIRE_FRAME_BYTES: usize = 64 * 1024;
/// Maximum number of fields in one envelope.
pub const MAX_APP_WIRE_FIELDS: usize = 64;
/// Maximum value size for one field.
pub const MAX_APP_WIRE_FIELD_BYTES: usize = 16 * 1024;

const HEADER_BYTES: usize = 8;
const FIELD_HEADER_BYTES: usize = 6;

/// One canonically ordered application-wire field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppWireField {
    /// Stable non-zero field identifier.
    pub id: u16,
    /// Opaque value bytes interpreted by the owning public contract.
    pub value: Vec<u8>,
}

impl AppWireField {
    /// Creates a field without hiding allocation or cloning.
    #[must_use]
    pub fn new(id: u16, value: Vec<u8>) -> Self {
        Self { id, value }
    }
}

/// Versioned application-wire envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppWireEnvelope {
    /// Wire version. Encoding and decoding accept only [`APP_WIRE_VERSION`].
    pub version: u16,
    /// Strictly increasing fields with no duplicate identifiers.
    pub fields: Vec<AppWireField>,
}

impl AppWireEnvelope {
    /// Creates a Phase 001 envelope.
    #[must_use]
    pub fn v1(fields: Vec<AppWireField>) -> Self {
        Self {
            version: APP_WIRE_VERSION,
            fields,
        }
    }
}

/// Stable bounded-wire failures.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AppWireError {
    /// The frame exceeds the allocation bound.
    #[error("app wire frame too large: {actual} > {maximum}")]
    FrameTooLarge {
        /// Observed frame size.
        actual: usize,
        /// Maximum accepted frame size.
        maximum: usize,
    },
    /// The frame magic is not the canonical discriminator.
    #[error("invalid app wire magic")]
    InvalidMagic,
    /// The frame ended before a declared value was complete.
    #[error("truncated app wire frame at byte {offset}")]
    Truncated {
        /// First unavailable byte.
        offset: usize,
    },
    /// The version is not supported.
    #[error("unknown app wire version: {0}")]
    UnknownVersion(u16),
    /// The field count exceeds the fixed bound.
    #[error("too many app wire fields: {actual} > {maximum}")]
    TooManyFields {
        /// Declared field count.
        actual: usize,
        /// Maximum accepted field count.
        maximum: usize,
    },
    /// A zero identifier is not canonical.
    #[error("app wire field identifier must be non-zero")]
    ZeroFieldId,
    /// A field identifier occurs more than once.
    #[error("duplicate app wire field: {0}")]
    DuplicateField(u16),
    /// Fields are not in strictly increasing canonical order.
    #[error("non-canonical app wire field order: {previous} then {current}")]
    NonCanonicalFieldOrder {
        /// Previous identifier.
        previous: u16,
        /// Current identifier.
        current: u16,
    },
    /// A declared field exceeds its allocation bound.
    #[error("app wire field {id} too large: {actual} > {maximum}")]
    FieldTooLarge {
        /// Field identifier.
        id: u16,
        /// Declared or provided value size.
        actual: usize,
        /// Maximum accepted field size.
        maximum: usize,
    },
    /// Bytes remain after the declared field set.
    #[error("trailing app wire bytes: consumed {consumed} of {total}")]
    TrailingBytes {
        /// Canonical frame length.
        consumed: usize,
        /// Supplied input length.
        total: usize,
    },
    /// The supplied bytes have an alternative, non-canonical representation.
    #[error("non-canonical app wire encoding")]
    NonCanonicalEncoding,
}

/// Stateless encoder and decoder for [`AppWireEnvelope`].
#[derive(Clone, Copy, Debug, Default)]
pub struct AppWireCodec;

impl AppWireCodec {
    /// Encodes one canonical bounded frame.
    pub fn encode(&self, envelope: &AppWireEnvelope) -> Result<Vec<u8>, AppWireError> {
        validate_version(envelope.version)?;
        validate_fields(&envelope.fields)?;
        let capacity = encoded_len(&envelope.fields)?;
        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(&APP_WIRE_MAGIC);
        bytes.extend_from_slice(&envelope.version.to_be_bytes());
        bytes.extend_from_slice(&(envelope.fields.len() as u16).to_be_bytes());
        for field in &envelope.fields {
            bytes.extend_from_slice(&field.id.to_be_bytes());
            bytes.extend_from_slice(&(field.value.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&field.value);
        }
        debug_assert_eq!(bytes.len(), capacity);
        Ok(bytes)
    }

    /// Decodes and canonicalizes one bounded frame before returning data.
    pub fn decode(&self, bytes: &[u8]) -> Result<AppWireEnvelope, AppWireError> {
        if bytes.len() > MAX_APP_WIRE_FRAME_BYTES {
            return Err(AppWireError::FrameTooLarge {
                actual: bytes.len(),
                maximum: MAX_APP_WIRE_FRAME_BYTES,
            });
        }
        if bytes.len() < HEADER_BYTES {
            return Err(AppWireError::Truncated {
                offset: bytes.len(),
            });
        }
        if bytes[..4] != APP_WIRE_MAGIC {
            return Err(AppWireError::InvalidMagic);
        }
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        validate_version(version)?;
        let field_count = u16::from_be_bytes([bytes[6], bytes[7]]) as usize;
        if field_count > MAX_APP_WIRE_FIELDS {
            return Err(AppWireError::TooManyFields {
                actual: field_count,
                maximum: MAX_APP_WIRE_FIELDS,
            });
        }

        let mut offset = HEADER_BYTES;
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let header_end = offset
                .checked_add(FIELD_HEADER_BYTES)
                .ok_or(AppWireError::Truncated { offset })?;
            let header = bytes
                .get(offset..header_end)
                .ok_or(AppWireError::Truncated { offset })?;
            let id = u16::from_be_bytes([header[0], header[1]]);
            let length = u32::from_be_bytes([header[2], header[3], header[4], header[5]]) as usize;
            validate_next_id(fields.last().map(|field: &AppWireField| field.id), id)?;
            if length > MAX_APP_WIRE_FIELD_BYTES {
                return Err(AppWireError::FieldTooLarge {
                    id,
                    actual: length,
                    maximum: MAX_APP_WIRE_FIELD_BYTES,
                });
            }
            offset = header_end;
            let value_end = offset
                .checked_add(length)
                .ok_or(AppWireError::FieldTooLarge {
                    id,
                    actual: usize::MAX,
                    maximum: MAX_APP_WIRE_FIELD_BYTES,
                })?;
            let value = bytes
                .get(offset..value_end)
                .ok_or(AppWireError::Truncated { offset })?;
            fields.push(AppWireField::new(id, value.to_vec()));
            offset = value_end;
        }
        if offset != bytes.len() {
            return Err(AppWireError::TrailingBytes {
                consumed: offset,
                total: bytes.len(),
            });
        }
        let envelope = AppWireEnvelope { version, fields };
        if self.encode(&envelope)?.as_slice() != bytes {
            return Err(AppWireError::NonCanonicalEncoding);
        }
        Ok(envelope)
    }
}

fn validate_version(version: u16) -> Result<(), AppWireError> {
    if version != APP_WIRE_VERSION {
        return Err(AppWireError::UnknownVersion(version));
    }
    Ok(())
}

fn validate_fields(fields: &[AppWireField]) -> Result<(), AppWireError> {
    if fields.len() > MAX_APP_WIRE_FIELDS {
        return Err(AppWireError::TooManyFields {
            actual: fields.len(),
            maximum: MAX_APP_WIRE_FIELDS,
        });
    }
    let mut previous = None;
    for field in fields {
        validate_next_id(previous, field.id)?;
        if field.value.len() > MAX_APP_WIRE_FIELD_BYTES {
            return Err(AppWireError::FieldTooLarge {
                id: field.id,
                actual: field.value.len(),
                maximum: MAX_APP_WIRE_FIELD_BYTES,
            });
        }
        previous = Some(field.id);
    }
    Ok(())
}

fn validate_next_id(previous: Option<u16>, current: u16) -> Result<(), AppWireError> {
    if current == 0 {
        return Err(AppWireError::ZeroFieldId);
    }
    let Some(previous) = previous else {
        return Ok(());
    };
    if current == previous {
        return Err(AppWireError::DuplicateField(current));
    }
    if current < previous {
        return Err(AppWireError::NonCanonicalFieldOrder { previous, current });
    }
    Ok(())
}

fn encoded_len(fields: &[AppWireField]) -> Result<usize, AppWireError> {
    let mut total = HEADER_BYTES;
    for field in fields {
        total = total
            .checked_add(FIELD_HEADER_BYTES)
            .and_then(|value| value.checked_add(field.value.len()))
            .ok_or(AppWireError::FrameTooLarge {
                actual: usize::MAX,
                maximum: MAX_APP_WIRE_FRAME_BYTES,
            })?;
    }
    if total > MAX_APP_WIRE_FRAME_BYTES {
        return Err(AppWireError::FrameTooLarge {
            actual: total,
            maximum: MAX_APP_WIRE_FRAME_BYTES,
        });
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical() -> AppWireEnvelope {
        AppWireEnvelope::v1(vec![
            AppWireField::new(1, b"status".to_vec()),
            AppWireField::new(7, vec![0, 1, 2, 255]),
        ])
    }

    #[test]
    fn canonical_roundtrip_and_bytes_are_stable() {
        let codec = AppWireCodec;
        let bytes = codec.encode(&canonical()).expect("canonical encode");
        assert_eq!(
            bytes,
            vec![
                b'Z', b'A', b'W', b'F', 0, 1, 0, 2, 0, 1, 0, 0, 0, 6, b's', b't', b'a', b't', b'u',
                b's', 0, 7, 0, 0, 0, 4, 0, 1, 2, 255,
            ]
        );
        assert_eq!(codec.decode(&bytes), Ok(canonical()));
    }

    #[test]
    fn unknown_version_is_rejected() {
        let mut bytes = AppWireCodec.encode(&canonical()).expect("encode");
        bytes[5] = 2;
        assert_eq!(
            AppWireCodec.decode(&bytes),
            Err(AppWireError::UnknownVersion(2))
        );
    }

    #[test]
    fn duplicate_and_noncanonical_fields_are_rejected() {
        let duplicate = AppWireEnvelope::v1(vec![
            AppWireField::new(1, vec![]),
            AppWireField::new(1, vec![]),
        ]);
        assert_eq!(
            AppWireCodec.encode(&duplicate),
            Err(AppWireError::DuplicateField(1))
        );
        let unordered = AppWireEnvelope::v1(vec![
            AppWireField::new(2, vec![]),
            AppWireField::new(1, vec![]),
        ]);
        assert_eq!(
            AppWireCodec.encode(&unordered),
            Err(AppWireError::NonCanonicalFieldOrder {
                previous: 2,
                current: 1,
            })
        );
    }

    #[test]
    fn bounds_are_checked_before_value_allocation() {
        let oversized_frame = vec![0; MAX_APP_WIRE_FRAME_BYTES + 1];
        assert_eq!(
            AppWireCodec.decode(&oversized_frame),
            Err(AppWireError::FrameTooLarge {
                actual: MAX_APP_WIRE_FRAME_BYTES + 1,
                maximum: MAX_APP_WIRE_FRAME_BYTES,
            })
        );
        let oversized_field = AppWireEnvelope::v1(vec![AppWireField::new(
            1,
            vec![0; MAX_APP_WIRE_FIELD_BYTES + 1],
        )]);
        assert_eq!(
            AppWireCodec.encode(&oversized_field),
            Err(AppWireError::FieldTooLarge {
                id: 1,
                actual: MAX_APP_WIRE_FIELD_BYTES + 1,
                maximum: MAX_APP_WIRE_FIELD_BYTES,
            })
        );
    }

    #[test]
    fn truncated_and_trailing_frames_are_rejected() {
        let mut truncated = AppWireCodec.encode(&canonical()).expect("encode");
        truncated.pop();
        assert!(matches!(
            AppWireCodec.decode(&truncated),
            Err(AppWireError::Truncated { .. })
        ));

        let mut trailing = AppWireCodec.encode(&canonical()).expect("encode");
        let consumed = trailing.len();
        trailing.push(0);
        assert_eq!(
            AppWireCodec.decode(&trailing),
            Err(AppWireError::TrailingBytes {
                consumed,
                total: consumed + 1,
            })
        );
    }

    #[test]
    fn zero_identifier_is_noncanonical() {
        let zero = AppWireEnvelope::v1(vec![AppWireField::new(0, vec![])]);
        assert_eq!(AppWireCodec.encode(&zero), Err(AppWireError::ZeroFieldId));
    }
}
