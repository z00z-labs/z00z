use serde::ser::SerializeStruct;
use thiserror::Error;
use z00z_crypto::{
    expert::hash_domain,
    hash::{sha256_256_role, CheckpointShaRole},
    hash_zk::hash_zk,
};

use super::record::RootErr;

hash_domain!(StorBucketPolicyDom, "z00z.storage.settlement.bucket.v1", 1);

pub const BUCKET_POLICY_VERSION: u8 = 1;
pub const BUCKET_HASH_DOMAIN: &str = "z00z.storage.settlement.bucket.v1";
pub const BUCKET_CANONICAL_ENCODING: &str = "z00z.storage.settlement.bucket.canonical-le-v1";
pub const BUCKET_ID_WIDTH: u8 = 32;
pub const MAX_BUCKET_BITS: u8 = 32;
pub const DEFAULT_BUCKET_BITS: u8 = 8;
pub const DEFAULT_MIN_BUCKET_COUNT: u32 = 2;
pub const DEFAULT_MAX_TARGET_LEAF_COUNT: u32 = 4096;
pub const DEFAULT_COMPATIBILITY_GENERATION: u32 = 1;

/// Strongly typed definition namespace key.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct DefinitionId([u8; 32]);

impl DefinitionId {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for DefinitionId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Storage-owned fixed bucket identity derived from `SettlementPath` and policy.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct BucketId([u8; 32]);

impl BucketId {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub(crate) fn from_hash_prefix(mut bytes: [u8; 32], bucket_bits: u8) -> Self {
        let full_bytes = usize::from(bucket_bits / 8);
        let rem_bits = bucket_bits % 8;

        if rem_bits == 0 {
            for byte in bytes.iter_mut().skip(full_bytes) {
                *byte = 0;
            }
            return Self(bytes);
        }

        let keep_mask = 0xffu8 << (8 - rem_bits);
        if let Some(byte) = bytes.get_mut(full_bytes) {
            *byte &= keep_mask;
        }
        for byte in bytes.iter_mut().skip(full_bytes + 1) {
            *byte = 0;
        }
        Self(bytes)
    }
}

impl From<[u8; 32]> for BucketId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BucketPolicyError {
    #[error("bucket_bits must be in 1..=32")]
    BucketBits,
    #[error("bucket_id_width must be 32")]
    BucketWidth,
    #[error("min_bucket_count must be non-zero")]
    MinBucketCount,
    #[error("bucket policy would allow fewer buckets than min_bucket_count")]
    BucketCount,
}

/// Versioned fixed-bucket policy committed into verifier-visible metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BucketPolicy {
    bucket_policy_id: [u8; 32],
    bucket_bits: u8,
    bucket_id_width: u8,
    min_bucket_count: u32,
    max_target_leaf_count: u32,
    compatibility_generation: u32,
}

impl BucketPolicy {
    pub const DEFAULT_BUCKET_BITS: u8 = DEFAULT_BUCKET_BITS;
    pub const DEFAULT_MIN_BUCKET_COUNT: u32 = DEFAULT_MIN_BUCKET_COUNT;
    pub const DEFAULT_MAX_TARGET_LEAF_COUNT: u32 = DEFAULT_MAX_TARGET_LEAF_COUNT;
    pub const DEFAULT_COMPATIBILITY_GENERATION: u32 = DEFAULT_COMPATIBILITY_GENERATION;

    pub fn new(
        bucket_bits: u8,
        min_bucket_count: u32,
        max_target_leaf_count: u32,
        compatibility_generation: u32,
    ) -> Result<Self, BucketPolicyError> {
        Self::with_width(
            bucket_bits,
            BUCKET_ID_WIDTH,
            min_bucket_count,
            max_target_leaf_count,
            compatibility_generation,
        )
    }

    pub fn with_width(
        bucket_bits: u8,
        bucket_id_width: u8,
        min_bucket_count: u32,
        max_target_leaf_count: u32,
        compatibility_generation: u32,
    ) -> Result<Self, BucketPolicyError> {
        validate_bucket_policy(bucket_bits, bucket_id_width, min_bucket_count)?;
        let bucket_policy_id = policy_id(
            bucket_bits,
            bucket_id_width,
            min_bucket_count,
            max_target_leaf_count,
            compatibility_generation,
        );
        Ok(Self {
            bucket_policy_id,
            bucket_bits,
            bucket_id_width,
            min_bucket_count,
            max_target_leaf_count,
            compatibility_generation,
        })
    }

    #[must_use]
    pub fn default_fixed() -> Self {
        Self {
            bucket_policy_id: policy_id(
                DEFAULT_BUCKET_BITS,
                BUCKET_ID_WIDTH,
                DEFAULT_MIN_BUCKET_COUNT,
                DEFAULT_MAX_TARGET_LEAF_COUNT,
                DEFAULT_COMPATIBILITY_GENERATION,
            ),
            bucket_bits: DEFAULT_BUCKET_BITS,
            bucket_id_width: BUCKET_ID_WIDTH,
            min_bucket_count: DEFAULT_MIN_BUCKET_COUNT,
            max_target_leaf_count: DEFAULT_MAX_TARGET_LEAF_COUNT,
            compatibility_generation: DEFAULT_COMPATIBILITY_GENERATION,
        }
    }

    #[must_use]
    pub const fn bucket_policy_id(self) -> [u8; 32] {
        self.bucket_policy_id
    }

    #[must_use]
    pub const fn bucket_bits(self) -> u8 {
        self.bucket_bits
    }

    #[must_use]
    pub const fn bucket_id_width(self) -> u8 {
        self.bucket_id_width
    }

    #[must_use]
    pub const fn min_bucket_count(self) -> u32 {
        self.min_bucket_count
    }

    #[must_use]
    pub const fn max_target_leaf_count(self) -> u32 {
        self.max_target_leaf_count
    }

    #[must_use]
    pub const fn compatibility_generation(self) -> u32 {
        self.compatibility_generation
    }

    #[must_use]
    pub fn bucket_count(self) -> u64 {
        1u64 << self.bucket_bits
    }

    #[must_use]
    pub fn derive_bucket_id(self, path: impl Into<SettlementPath>) -> BucketId {
        let path: SettlementPath = path.into();
        let serial = path.serial_id.get().to_le_bytes();
        let policy_id = self.bucket_policy_id;
        let terminal_id = path.terminal_id();
        let hash = hash_zk::<StorBucketPolicyDom>(
            "derive_bucket_id_v1",
            &[
                path.definition_id.as_bytes(),
                &serial,
                terminal_id.as_bytes(),
                &policy_id,
            ],
        );
        BucketId::from_hash_prefix(hash, self.bucket_bits)
    }

    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + 1 + 1 + 4 + 4 + 4);
        out.extend_from_slice(&self.bucket_policy_id);
        out.push(self.bucket_bits);
        out.push(self.bucket_id_width);
        out.extend_from_slice(&self.min_bucket_count.to_le_bytes());
        out.extend_from_slice(&self.max_target_leaf_count.to_le_bytes());
        out.extend_from_slice(&self.compatibility_generation.to_le_bytes());
        out
    }
}

impl Default for BucketPolicy {
    fn default() -> Self {
        Self::default_fixed()
    }
}

impl serde::Serialize for BucketPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            let mut state = serializer.serialize_struct("BucketPolicy", 6)?;
            state.serialize_field("bucket_policy_id", &self.bucket_policy_id)?;
            state.serialize_field("bucket_bits", &self.bucket_bits)?;
            state.serialize_field("bucket_id_width", &self.bucket_id_width)?;
            state.serialize_field("min_bucket_count", &self.min_bucket_count)?;
            state.serialize_field("max_target_leaf_count", &self.max_target_leaf_count)?;
            state.serialize_field("compatibility_generation", &self.compatibility_generation)?;
            state.end()
        } else {
            serializer.serialize_bytes(&self.encode())
        }
    }
}

impl<'de> serde::Deserialize<'de> for BucketPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !deserializer.is_human_readable() {
            struct BucketPolicyBytesVisitor;

            impl<'de> serde::de::Visitor<'de> for BucketPolicyBytesVisitor {
                type Value = BucketPolicy;

                fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.write_str("46-byte encoded bucket policy")
                }

                fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    decode_bucket_policy(bytes)
                }

                fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    decode_bucket_policy(&bytes)
                }
            }

            return deserializer.deserialize_byte_buf(BucketPolicyBytesVisitor);
        }

        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            bucket_policy_id: [u8; 32],
            bucket_bits: u8,
            bucket_id_width: u8,
            min_bucket_count: u32,
            max_target_leaf_count: u32,
            compatibility_generation: u32,
        }

        let wire = Wire::deserialize(deserializer)?;
        let policy = BucketPolicy::with_width(
            wire.bucket_bits,
            wire.bucket_id_width,
            wire.min_bucket_count,
            wire.max_target_leaf_count,
            wire.compatibility_generation,
        )
        .map_err(serde::de::Error::custom)?;
        if policy.bucket_policy_id != wire.bucket_policy_id {
            return Err(serde::de::Error::custom("bucket_policy_id mismatch"));
        }
        Ok(policy)
    }
}

fn decode_bucket_policy<E>(bytes: &[u8]) -> Result<BucketPolicy, E>
where
    E: serde::de::Error,
{
    const BUCKET_POLICY_LEN: usize = 32 + 1 + 1 + 4 + 4 + 4;

    if bytes.len() != BUCKET_POLICY_LEN {
        return Err(E::custom("invalid bucket policy length"));
    }

    let mut bucket_policy_id = [0u8; 32];
    bucket_policy_id.copy_from_slice(&bytes[..32]);
    let bucket_bits = bytes[32];
    let bucket_id_width = bytes[33];
    let min_bucket_count = u32::from_le_bytes(
        bytes[34..38]
            .try_into()
            .map_err(|_| E::custom("invalid bucket policy min_bucket_count bytes"))?,
    );
    let max_target_leaf_count = u32::from_le_bytes(
        bytes[38..42]
            .try_into()
            .map_err(|_| E::custom("invalid bucket policy max_target_leaf_count bytes"))?,
    );
    let compatibility_generation = u32::from_le_bytes(
        bytes[42..46]
            .try_into()
            .map_err(|_| E::custom("invalid bucket policy compatibility_generation bytes"))?,
    );

    let policy = BucketPolicy::with_width(
        bucket_bits,
        bucket_id_width,
        min_bucket_count,
        max_target_leaf_count,
        compatibility_generation,
    )
    .map_err(E::custom)?;
    if policy.bucket_policy_id != bucket_policy_id {
        return Err(E::custom("bucket_policy_id mismatch"));
    }
    Ok(policy)
}

fn validate_bucket_policy(
    bucket_bits: u8,
    bucket_id_width: u8,
    min_bucket_count: u32,
) -> Result<(), BucketPolicyError> {
    if !(1..=MAX_BUCKET_BITS).contains(&bucket_bits) {
        return Err(BucketPolicyError::BucketBits);
    }
    if bucket_id_width != BUCKET_ID_WIDTH {
        return Err(BucketPolicyError::BucketWidth);
    }
    if min_bucket_count == 0 {
        return Err(BucketPolicyError::MinBucketCount);
    }
    let bucket_count = 1u64 << bucket_bits;
    if bucket_count < u64::from(min_bucket_count) {
        return Err(BucketPolicyError::BucketCount);
    }
    Ok(())
}

fn policy_id(
    bucket_bits: u8,
    bucket_id_width: u8,
    min_bucket_count: u32,
    max_target_leaf_count: u32,
    compatibility_generation: u32,
) -> [u8; 32] {
    let version = [BUCKET_POLICY_VERSION];
    let bits = [bucket_bits];
    let width = [bucket_id_width];
    let min = min_bucket_count.to_le_bytes();
    let max = max_target_leaf_count.to_le_bytes();
    let generation = compatibility_generation.to_le_bytes();
    hash_zk::<StorBucketPolicyDom>(
        "bucket_policy_id_v1",
        &[
            &version,
            BUCKET_HASH_DOMAIN.as_bytes(),
            BUCKET_CANONICAL_ENCODING.as_bytes(),
            &bits,
            &width,
            &min,
            &max,
            &generation,
        ],
    )
}

/// Strongly typed serial bucket key.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct SerialId(u32);

impl SerialId {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for SerialId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

/// Storage-family root generation accepted by the generalized settlement surface.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(u8)]
pub enum RootGeneration {
    SettlementV1 = 1,
    SettlementV2 = 2,
}

impl RootGeneration {
    #[must_use]
    pub const fn version(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn from_version(version: u8) -> Option<Self> {
        match version {
            1 => Some(Self::SettlementV1),
            2 => Some(Self::SettlementV2),
            _ => None,
        }
    }
}

/// Generalized settlement-state root for the live storage generation.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SettlementStateRoot {
    generation: RootGeneration,
    root: [u8; 32],
}

impl SettlementStateRoot {
    #[must_use]
    pub const fn new(generation: RootGeneration, root: [u8; 32]) -> Self {
        Self { generation, root }
    }

    #[must_use]
    pub const fn settlement_v1(root: [u8; 32]) -> Self {
        Self::new(RootGeneration::SettlementV1, root)
    }

    /// Creates the sole post-cutover settlement-root generation.
    #[must_use]
    pub const fn settlement_v2(root: [u8; 32]) -> Self {
        Self::new(RootGeneration::SettlementV2, root)
    }

    #[must_use]
    pub const fn generation(self) -> RootGeneration {
        self.generation
    }

    #[must_use]
    pub const fn generation_version(self) -> u8 {
        self.generation.version()
    }

    #[must_use]
    pub const fn from_version(version: u8, root: [u8; 32]) -> Option<Self> {
        match RootGeneration::from_version(version) {
            Some(generation) => Some(Self::new(generation, root)),
            None => None,
        }
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.root
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.root
    }
}

/// Derive the sole V2 settlement root from the canonical HJMT definition root.
///
/// The layout value is an authority-pinned storage layout version, while the
/// policy digest is the canonical bucket-policy identifier.  Both are bound so
/// a raw HJMT root can never masquerade as a settlement root.
pub fn derive_settlement_root_v2(
    generation: RootGeneration,
    layout: u32,
    policy_digest: [u8; 32],
    definition_root: [u8; 32],
) -> Result<SettlementStateRoot, RootErr> {
    if generation != RootGeneration::SettlementV2 {
        return Err(RootErr::GenerationMix);
    }
    let generation = [generation.version()];
    let layout = layout.to_le_bytes();
    let digest = sha256_256_role(
        CheckpointShaRole::SettlementRoot,
        &[&generation, &layout, &policy_digest, &definition_root],
    );
    Ok(SettlementStateRoot::settlement_v2(digest))
}

/// Authoritative claim-source root exported by storage-owned proof APIs.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ClaimSourceRoot {
    root_version: u8,
    root: SettlementStateRoot,
}

impl ClaimSourceRoot {
    #[must_use]
    pub const fn new(root_version: u8, root: SettlementStateRoot) -> Self {
        Self { root_version, root }
    }

    #[must_use]
    pub const fn new_settlement(root_version: u8, root: SettlementStateRoot) -> Self {
        Self::new(root_version, root)
    }

    #[must_use]
    pub const fn root_version(self) -> u8 {
        self.root_version
    }

    #[must_use]
    pub const fn root(self) -> SettlementStateRoot {
        self.root
    }

    #[must_use]
    pub const fn settlement_root(self) -> SettlementStateRoot {
        self.root
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.root.into_bytes()
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        self.root.as_bytes()
    }
}

/// Checkpoint root bound to state-transition APIs.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CheckRoot([u8; 32]);

impl CheckRoot {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for CheckRoot {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

impl From<SettlementStateRoot> for CheckRoot {
    fn from(root: SettlementStateRoot) -> Self {
        Self::new(root.into_bytes())
    }
}

/// Transaction-envelope digest.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TxDigest([u8; 32]);

impl TxDigest {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_check(self) -> Result<CheckRoot, RootErr> {
        let _ = self;
        Err(RootErr::TxRootMix)
    }
}

impl From<[u8; 32]> for TxDigest {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Generalized terminal identity for live settlement leaves.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TerminalId([u8; 32]);

impl TerminalId {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn into_bytes(self) -> [u8; 32] {
        self.0
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl From<[u8; 32]> for TerminalId {
    fn from(bytes: [u8; 32]) -> Self {
        Self::new(bytes)
    }
}

/// Canonical generalized settlement path.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct SettlementPath {
    pub definition_id: DefinitionId,
    pub serial_id: SerialId,
    pub terminal_id: TerminalId,
}

impl SettlementPath {
    #[must_use]
    pub const fn new(
        definition_id: DefinitionId,
        serial_id: SerialId,
        terminal_id: TerminalId,
    ) -> Self {
        Self {
            definition_id,
            serial_id,
            terminal_id,
        }
    }

    pub fn check(self) -> Result<Self, SettlementPathErr> {
        if self.terminal_id.is_zero() {
            return Err(SettlementPathErr::ZeroTerminalId);
        }
        Ok(self)
    }

    pub fn check_unique(paths: &[Self]) -> Result<(), SettlementPathErr> {
        let mut seen = std::collections::BTreeSet::new();
        for path in paths {
            path.check()?;
            if !seen.insert(path.terminal_id) {
                return Err(SettlementPathErr::DupTerminalId);
            }
        }
        Ok(())
    }

    #[must_use]
    pub const fn terminal_id(self) -> TerminalId {
        self.terminal_id
    }

    #[must_use]
    pub fn bucket_id(self, policy: BucketPolicy) -> BucketId {
        policy.derive_bucket_id(self)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettlementPathErr {
    #[error("settlement terminal id must be non-zero")]
    ZeroTerminalId,
    #[error("duplicate settlement terminal id")]
    DupTerminalId,
}

#[cfg(test)]
mod tests {
    use super::{derive_settlement_root_v2, RootGeneration};

    #[test]
    fn test_settlement_root_vector() {
        let root = derive_settlement_root_v2(
            RootGeneration::SettlementV2,
            0x0102_0304,
            [0x11; 32],
            [0x22; 32],
        )
        .expect("V2 generation");
        assert_eq!(root.generation(), RootGeneration::SettlementV2);
        assert_eq!(
            root.into_bytes(),
            [
                0xe2, 0x67, 0x5c, 0xcb, 0x9e, 0x97, 0x97, 0x5b, 0x79, 0x84, 0x1d, 0xec, 0x9f, 0xfb,
                0x9f, 0xee, 0xd9, 0x9e, 0xab, 0xb3, 0xd6, 0xb7, 0x6b, 0x9e, 0x19, 0x2c, 0x6d, 0x55,
                0xd9, 0xed, 0x44, 0xa2,
            ]
        );
    }

    #[test]
    fn test_settlement_root_inputs() {
        let baseline =
            derive_settlement_root_v2(RootGeneration::SettlementV2, 1, [0x11; 32], [0x22; 32])
                .expect("V2 generation");
        let changed_layout =
            derive_settlement_root_v2(RootGeneration::SettlementV2, 2, [0x11; 32], [0x22; 32])
                .expect("V2 generation");
        let changed_policy =
            derive_settlement_root_v2(RootGeneration::SettlementV2, 1, [0x12; 32], [0x22; 32])
                .expect("V2 generation");
        let changed_root =
            derive_settlement_root_v2(RootGeneration::SettlementV2, 1, [0x11; 32], [0x23; 32])
                .expect("V2 generation");
        assert_ne!(baseline, changed_layout);
        assert_ne!(baseline, changed_policy);
        assert_ne!(baseline, changed_root);
    }

    #[test]
    fn test_settlement_root_generation() {
        assert!(
            derive_settlement_root_v2(RootGeneration::SettlementV1, 1, [0x11; 32], [0x22; 32])
                .is_err()
        );
    }
}
