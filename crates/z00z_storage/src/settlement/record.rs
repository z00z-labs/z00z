use thiserror::Error;
use z00z_core::vouchers::{VoucherLifecycleV1, VoucherValidityWindowV1};
use z00z_utils::codec::{BincodeCodec, Codec, CodecError};

use crate::settlement::TerminalLeaf;

use super::{
    leaf::{
        encode_right_leaf, encode_terminal_leaf, encode_voucher_leaf, RIGHT_LEAF_TAG,
        TERMINAL_LEAF_TAG, VOUCHER_LEAF_TAG,
    },
    BucketId, DefinitionId, SerialId, SettlementPath, SettlementPathErr, SettlementStateRoot,
    TerminalId,
};

/// Versioned terminal settlement leaf family for the generalized storage generation.
///
/// Shared cross-object family vocabulary remains a root-owned facade in
/// `z00z_core::{ObjectFamily, ObjectRoleV1}`; storage persists the concrete
/// owner-family leaves directly and does not introduce a second asset-owned
/// family authority path.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettlementLeaf {
    Terminal(TerminalLeaf),
    Right(RightLeaf),
    Voucher(VoucherLeaf),
}

impl serde::Serialize for SettlementLeaf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            #[derive(serde::Serialize)]
            #[serde(tag = "family", content = "leaf")]
            enum ReadableSettlementLeaf<'a> {
                Terminal(&'a TerminalLeaf),
                Right(&'a RightLeaf),
                Voucher(&'a VoucherLeaf),
            }

            match self {
                Self::Terminal(leaf) => {
                    ReadableSettlementLeaf::Terminal(leaf).serialize(serializer)
                }
                Self::Right(leaf) => ReadableSettlementLeaf::Right(leaf).serialize(serializer),
                Self::Voucher(leaf) => ReadableSettlementLeaf::Voucher(leaf).serialize(serializer),
            }
        } else {
            serializer.serialize_bytes(&self.encode().map_err(serde::ser::Error::custom)?)
        }
    }
}

impl<'de> serde::Deserialize<'de> for SettlementLeaf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            #[derive(serde::Deserialize)]
            #[serde(tag = "family", content = "leaf")]
            #[allow(clippy::large_enum_variant)]
            enum ReadableSettlementLeaf {
                Terminal(TerminalLeaf),
                Right(RightLeaf),
                Voucher(VoucherLeaf),
            }

            return Ok(match ReadableSettlementLeaf::deserialize(deserializer)? {
                ReadableSettlementLeaf::Terminal(leaf) => Self::Terminal(leaf),
                ReadableSettlementLeaf::Right(leaf) => Self::Right(leaf),
                ReadableSettlementLeaf::Voucher(leaf) => Self::Voucher(leaf),
            });
        }

        struct SettlementLeafBytesVisitor;

        impl<'de> serde::de::Visitor<'de> for SettlementLeafBytesVisitor {
            type Value = SettlementLeaf;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("encoded settlement leaf bytes")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                SettlementLeaf::decode(bytes).map_err(E::custom)
            }

            fn visit_byte_buf<E>(self, bytes: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                SettlementLeaf::decode(&bytes).map_err(E::custom)
            }
        }

        deserializer.deserialize_byte_buf(SettlementLeafBytesVisitor)
    }
}

impl From<TerminalLeaf> for SettlementLeaf {
    fn from(leaf: TerminalLeaf) -> Self {
        Self::Terminal(leaf)
    }
}

impl From<z00z_core::assets::AssetLeaf> for SettlementLeaf {
    fn from(leaf: z00z_core::assets::AssetLeaf) -> Self {
        Self::Terminal(TerminalLeaf::from(leaf))
    }
}

impl From<&TerminalLeaf> for SettlementLeaf {
    fn from(leaf: &TerminalLeaf) -> Self {
        Self::Terminal(leaf.clone())
    }
}

impl From<&z00z_core::assets::AssetLeaf> for SettlementLeaf {
    fn from(leaf: &z00z_core::assets::AssetLeaf) -> Self {
        Self::Terminal(TerminalLeaf::from(leaf.clone()))
    }
}

impl From<RightLeaf> for SettlementLeaf {
    fn from(leaf: RightLeaf) -> Self {
        Self::Right(leaf)
    }
}

impl From<&RightLeaf> for SettlementLeaf {
    fn from(leaf: &RightLeaf) -> Self {
        Self::Right(*leaf)
    }
}

impl From<VoucherLeaf> for SettlementLeaf {
    fn from(leaf: VoucherLeaf) -> Self {
        Self::Voucher(leaf)
    }
}

impl From<&VoucherLeaf> for SettlementLeaf {
    fn from(leaf: &VoucherLeaf) -> Self {
        Self::Voucher(leaf.clone())
    }
}

impl From<&SettlementLeaf> for SettlementLeaf {
    fn from(leaf: &SettlementLeaf) -> Self {
        leaf.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightClass {
    MachineCapability,
    DataAccess,
    ServiceEntitlement,
    ValidatorMandate,
    OneTimeUse,
}

impl SettlementLeaf {
    #[must_use]
    pub const fn family_tag(&self) -> u8 {
        match self {
            Self::Terminal(_) => TERMINAL_LEAF_TAG,
            Self::Right(_) => RIGHT_LEAF_TAG,
            Self::Voucher(_) => VOUCHER_LEAF_TAG,
        }
    }

    #[must_use]
    pub fn terminal_id(&self) -> TerminalId {
        match self {
            Self::Terminal(leaf) => leaf.terminal_id(),
            Self::Right(leaf) => leaf.terminal_id,
            Self::Voucher(leaf) => leaf.terminal_id,
        }
    }

    #[must_use]
    pub fn serial_id(&self) -> Option<SerialId> {
        match self {
            Self::Terminal(leaf) => Some(SerialId::new(leaf.serial_id)),
            Self::Right(_) => None,
            Self::Voucher(_) => None,
        }
    }

    #[must_use]
    pub fn as_terminal(&self) -> Option<&TerminalLeaf> {
        match self {
            Self::Terminal(leaf) => Some(leaf),
            Self::Right(_) | Self::Voucher(_) => None,
        }
    }

    #[must_use]
    pub fn as_right(&self) -> Option<&RightLeaf> {
        match self {
            Self::Terminal(_) | Self::Voucher(_) => None,
            Self::Right(leaf) => Some(leaf),
        }
    }

    #[must_use]
    pub fn as_voucher(&self) -> Option<&VoucherLeaf> {
        match self {
            Self::Terminal(_) | Self::Right(_) => None,
            Self::Voucher(leaf) => Some(leaf),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        match self {
            Self::Terminal(leaf) => encode_terminal_leaf(leaf),
            Self::Right(leaf) => encode_right_leaf(leaf),
            Self::Voucher(leaf) => encode_voucher_leaf(leaf),
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let Some((tag, payload)) = bytes.split_first() else {
            return Err(CodecError::Bincode(
                "empty settlement leaf payload".to_string(),
            ));
        };
        let codec = BincodeCodec;
        match *tag {
            TERMINAL_LEAF_TAG => Ok(Self::Terminal(codec.deserialize(payload)?)),
            RIGHT_LEAF_TAG => Ok(Self::Right(codec.deserialize(payload)?)),
            VOUCHER_LEAF_TAG => Ok(Self::Voucher(codec.deserialize(payload)?)),
            other => Err(CodecError::Bincode(format!(
                "unknown settlement leaf family tag: {other}"
            ))),
        }
    }

    pub fn check_path(&self, path: SettlementPath) -> Result<(), RightErr> {
        path.check()?;
        match self {
            Self::Terminal(leaf) => {
                if path.terminal_id != leaf.terminal_id() {
                    return Err(RightErr::PathTerminalMix);
                }
            }
            Self::Right(leaf) => leaf.check_path(path)?,
            Self::Voucher(leaf) => leaf.check_path(path)?,
        }
        Ok(())
    }
}

/// Narrow terminal object for one bounded non-coin settlement right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RightLeaf {
    pub version: u8,
    pub terminal_id: TerminalId,
    pub right_class: RightClass,
    pub issuer_scope: [u8; 32],
    pub provider_scope: [u8; 32],
    pub holder_commitment: [u8; 32],
    pub control_commitment: [u8; 32],
    pub beneficiary_commitment: [u8; 32],
    pub payload_commitment: [u8; 32],
    pub valid_from: u64,
    pub valid_until: u64,
    pub challenge_from: u64,
    pub challenge_until: u64,
    pub use_nonce: [u8; 32],
    pub revocation_policy_id: [u8; 32],
    pub transition_policy_id: [u8; 32],
    pub challenge_policy_id: [u8; 32],
    pub disclosure_policy_id: [u8; 32],
    pub retention_policy_id: [u8; 32],
}

/// Storage-owned committed backing reference for one voucher leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherBackingRef {
    ReserveCommitment([u8; 32]),
    ConsumedAsset {
        definition_id: [u8; 32],
        serial_id: u32,
    },
    GenesisReserve([u8; 32]),
}

/// Narrow terminal object for one conditional value claim.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VoucherLeaf {
    pub version: u8,
    pub terminal_id: TerminalId,
    pub issuer_commitment: [u8; 32],
    pub holder_commitment: [u8; 32],
    pub beneficiary_commitment: [u8; 32],
    pub refund_target_commitment: [u8; 32],
    pub backing: VoucherBackingRef,
    pub face_value: u64,
    pub remaining_value: u64,
    pub policy_id: [u8; 32],
    pub action_pool_id: [u8; 32],
    pub lifecycle: VoucherLifecycleV1,
    pub validity: VoucherValidityWindowV1,
    pub receiver_must_accept: bool,
    pub allow_reject: bool,
    pub replay_nonce: [u8; 32],
    pub disclosure_commitment: Option<[u8; 32]>,
    pub audit_commitment: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightAction {
    Create,
    Transfer,
    Consume,
    Expire,
    Revoke,
    Challenge,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RightActionCtx {
    pub now: u64,
    pub expected_holder: Option<[u8; 32]>,
    pub expected_control: Option<[u8; 32]>,
    pub revoked: bool,
    pub seen_use_nonce: Option<[u8; 32]>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RightErr {
    #[error(transparent)]
    Path(#[from] SettlementPathErr),
    #[error("settlement leaf family does not match path authority")]
    PathFamilyMix,
    #[error("settlement path terminal id does not match leaf terminal id")]
    PathTerminalMix,
    #[error("right validity window is malformed")]
    ValidityWindow,
    #[error("right is expired for requested action")]
    Expired,
    #[error("right is already revoked for requested action")]
    Revoked,
    #[error("right holder/control binding mismatch")]
    HolderControlMix,
    #[error("right one-time-use marker was already consumed")]
    OneTimeReplay,
    #[error("right transition invariants drifted")]
    TransitionMix,
    #[error("right transition policy is missing")]
    TransitionPolicyMix,
    #[error("right challenge policy is missing")]
    ChallengePolicyMix,
    #[error("right challenge window is malformed or inactive")]
    ChallengeWindow,
    #[error("right revocation policy is missing")]
    RevocationPolicyMix,
}

impl RightLeaf {
    fn check_transfer_monotonic(&self, prior: &Self) -> Result<(), RightErr> {
        if self.valid_from < prior.valid_from
            || self.valid_until > prior.valid_until
            || self.challenge_from < prior.challenge_from
            || self.challenge_until > prior.challenge_until
        {
            return Err(RightErr::TransitionMix);
        }

        if self.revocation_policy_id != prior.revocation_policy_id
            || self.transition_policy_id != prior.transition_policy_id
            || self.challenge_policy_id != prior.challenge_policy_id
        {
            return Err(RightErr::TransitionMix);
        }

        Ok(())
    }

    pub fn check(&self) -> Result<(), RightErr> {
        if self.terminal_id.is_zero() {
            return Err(RightErr::Path(SettlementPathErr::ZeroTerminalId));
        }
        if self.valid_until < self.valid_from {
            return Err(RightErr::ValidityWindow);
        }
        if (self.challenge_from != 0 || self.challenge_until != 0)
            && self.challenge_until < self.challenge_from
        {
            return Err(RightErr::ChallengeWindow);
        }
        Ok(())
    }

    pub fn check_path(&self, path: SettlementPath) -> Result<(), RightErr> {
        path.check()?;
        if path.terminal_id != self.terminal_id {
            return Err(RightErr::PathTerminalMix);
        }
        Ok(())
    }

    pub fn validate_action(
        &self,
        action: RightAction,
        ctx: RightActionCtx,
        prior: Option<&Self>,
    ) -> Result<(), RightErr> {
        self.check()?;

        if let Some(holder) = ctx.expected_holder {
            if self.holder_commitment != holder {
                return Err(RightErr::HolderControlMix);
            }
        }
        if let Some(control) = ctx.expected_control {
            if self.control_commitment != control {
                return Err(RightErr::HolderControlMix);
            }
        }
        if ctx.revoked {
            return Err(RightErr::Revoked);
        }
        if action != RightAction::Create && ctx.now < self.valid_from {
            return Err(RightErr::ValidityWindow);
        }
        if ctx.now > self.valid_until && action != RightAction::Expire {
            return Err(RightErr::Expired);
        }
        if let Some(use_nonce) = ctx.seen_use_nonce {
            if self.use_nonce != [0u8; 32] && self.use_nonce == use_nonce {
                return Err(RightErr::OneTimeReplay);
            }
        }

        match action {
            RightAction::Create => Ok(()),
            RightAction::Transfer | RightAction::Consume | RightAction::Challenge => {
                let Some(prior) = prior else {
                    return Err(RightErr::TransitionMix);
                };
                if self.transition_policy_id == [0u8; 32] {
                    return Err(RightErr::TransitionPolicyMix);
                }
                if prior.terminal_id != self.terminal_id
                    || prior.right_class != self.right_class
                    || prior.issuer_scope != self.issuer_scope
                    || prior.provider_scope != self.provider_scope
                    || prior.payload_commitment != self.payload_commitment
                    || prior.disclosure_policy_id != self.disclosure_policy_id
                    || prior.retention_policy_id != self.retention_policy_id
                {
                    return Err(RightErr::TransitionMix);
                }
                if action == RightAction::Transfer {
                    self.check_transfer_monotonic(prior)?;
                }
                if action == RightAction::Challenge {
                    if self.challenge_policy_id == [0u8; 32] {
                        return Err(RightErr::ChallengePolicyMix);
                    }
                    if self.challenge_from == 0 && self.challenge_until == 0
                        || ctx.now < self.challenge_from
                        || ctx.now > self.challenge_until
                    {
                        return Err(RightErr::ChallengeWindow);
                    }
                }
                Ok(())
            }
            RightAction::Expire => {
                if ctx.now <= self.valid_until {
                    return Err(RightErr::ValidityWindow);
                }
                Ok(())
            }
            RightAction::Revoke => {
                if self.revocation_policy_id == [0u8; 32] {
                    return Err(RightErr::RevocationPolicyMix);
                }
                Ok(())
            }
        }
    }
}

impl VoucherLeaf {
    #[must_use]
    pub fn marker(path: SettlementPath) -> Self {
        Self {
            version: 1,
            terminal_id: path.terminal_id,
            issuer_commitment: [0u8; 32],
            holder_commitment: [0u8; 32],
            beneficiary_commitment: [0u8; 32],
            refund_target_commitment: [0u8; 32],
            backing: VoucherBackingRef::ReserveCommitment([0u8; 32]),
            face_value: 0,
            remaining_value: 0,
            policy_id: [0u8; 32],
            action_pool_id: [0u8; 32],
            lifecycle: VoucherLifecycleV1::PendingAcceptance,
            validity: VoucherValidityWindowV1 {
                valid_from: 0,
                valid_until: 0,
            },
            receiver_must_accept: true,
            allow_reject: false,
            replay_nonce: [0u8; 32],
            disclosure_commitment: None,
            audit_commitment: None,
        }
    }

    pub fn check_path(&self, path: SettlementPath) -> Result<(), RightErr> {
        path.check()?;
        if path.terminal_id != self.terminal_id {
            return Err(RightErr::PathTerminalMix);
        }
        Ok(())
    }
}

/// Separate processing-support envelope for one settlement transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeeEnvelope {
    pub version: u8,
    pub payer_commitment: [u8; 32],
    pub sponsor_commitment: [u8; 32],
    pub budget_units: u64,
    pub budget_commitment: [u8; 32],
    pub domain_id: [u8; 32],
    pub expires_at: u64,
    pub nonce: [u8; 32],
    pub transition_id: [u8; 32],
    pub replay_key: [u8; 32],
    pub support_ref: Option<[u8; 32]>,
    pub failure_policy_id: [u8; 32],
}

/// Monotonic bucket-policy epoch bound into adaptive bucket proofs.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct BucketEpoch(u64);

impl BucketEpoch {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for BucketEpoch {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

/// Privacy-reviewed occupancy scope carried by adaptive policy evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyScope {
    Bucket,
    Pair,
    Set,
}

impl OccupancyScope {
    #[must_use]
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::Bucket => 1,
            Self::Pair => 2,
            Self::Set => 3,
        }
    }
}

/// Privacy-reviewed occupancy class carried by adaptive policy evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccupancyClass {
    Empty,
    MergeLow,
    Steady,
    SplitReady,
    SetCommit,
}

impl OccupancyClass {
    #[must_use]
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::MergeLow => 1,
            Self::Steady => 2,
            Self::SplitReady => 3,
            Self::SetCommit => 4,
        }
    }
}

/// Versioned privacy-bounded occupancy evidence for adaptive proofs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BucketOccupancyEvidence {
    pub version: u8,
    pub scope: OccupancyScope,
    pub class: OccupancyClass,
    pub bind: [u8; 32],
}

impl BucketOccupancyEvidence {
    #[must_use]
    pub const fn new(scope: OccupancyScope, class: OccupancyClass, bind: [u8; 32]) -> Self {
        Self {
            version: 1,
            scope,
            class,
            bind,
        }
    }
}

/// Local-only exact occupancy diagnostics for scheduling and operator metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BucketOccupancyMetric {
    pub definition_id: DefinitionId,
    pub serial_id: SerialId,
    pub bucket_id: BucketId,
    pub epoch: BucketEpoch,
    pub bucket_root: [u8; 32],
    pub class: OccupancyClass,
    pub exact_count: u64,
}

/// Adaptive bucket metadata committed by the generalized HJMT policy layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdaptiveBucket {
    pub definition_id: DefinitionId,
    pub serial_id: SerialId,
    pub bucket_id: BucketId,
    pub epoch: BucketEpoch,
    pub bucket_policy_id: [u8; 32],
    pub bucket_root: [u8; 32],
    pub occupancy_evidence: BucketOccupancyEvidence,
}

/// Proof that one bucket split under a committed policy transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SplitProof {
    pub prior_root: SettlementStateRoot,
    pub next_root: SettlementStateRoot,
    pub prior_epoch: BucketEpoch,
    pub next_epoch: BucketEpoch,
    pub prior_bucket_root: [u8; 32],
    pub left_bucket_root: [u8; 32],
    pub right_bucket_root: [u8; 32],
    pub bucket_policy_id: [u8; 32],
    pub occupancy_evidence: BucketOccupancyEvidence,
    pub key_range_commitment: [u8; 32],
    pub journal_digest: [u8; 32],
}

/// Proof that compatible buckets merged under a committed policy transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeProof {
    pub prior_root: SettlementStateRoot,
    pub next_root: SettlementStateRoot,
    pub prior_epoch: BucketEpoch,
    pub next_epoch: BucketEpoch,
    pub left_bucket_root: [u8; 32],
    pub right_bucket_root: [u8; 32],
    pub merged_bucket_root: [u8; 32],
    pub bucket_policy_id: [u8; 32],
    pub left_evidence: BucketOccupancyEvidence,
    pub right_evidence: BucketOccupancyEvidence,
    pub pair_evidence: BucketOccupancyEvidence,
    pub key_range_commitment: [u8; 32],
    pub journal_digest: [u8; 32],
}

/// Proof that terminal leaves moved across one HJMT policy epoch change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyTransitionProof {
    pub prior_root: SettlementStateRoot,
    pub next_root: SettlementStateRoot,
    pub prior_epoch: BucketEpoch,
    pub next_epoch: BucketEpoch,
    pub prior_policy_id: [u8; 32],
    pub next_policy_id: [u8; 32],
    pub terminal_set_commitment: [u8; 32],
    pub occupancy_evidence: BucketOccupancyEvidence,
    pub replay_digest: [u8; 32],
}

/// Global-tree committed leaf that binds one definition namespace to its child root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DefinitionRootLeaf {
    pub definition_id: DefinitionId,
    pub definition_root: [u8; 32],
}

impl DefinitionRootLeaf {
    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(64);
        out.extend_from_slice(self.definition_id.as_bytes());
        out.extend_from_slice(&self.definition_root);
        out
    }
}

/// Definition-tree committed leaf that binds one serial bucket to its child root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerialRootLeaf {
    pub definition_id: DefinitionId,
    pub serial_id: SerialId,
    pub serial_root: [u8; 32],
}

impl SerialRootLeaf {
    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(68);
        out.extend_from_slice(self.definition_id.as_bytes());
        out.extend_from_slice(&self.serial_id.get().to_le_bytes());
        out.extend_from_slice(&self.serial_root);
        out
    }
}

/// Serial-tree committed leaf that binds one fixed bucket to its child terminal root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BucketRootLeaf {
    pub definition_id: DefinitionId,
    pub serial_id: SerialId,
    pub bucket_id: BucketId,
    pub terminal_jmt_root: [u8; 32],
    pub bucket_policy_id: [u8; 32],
}

impl BucketRootLeaf {
    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(132);
        out.extend_from_slice(self.definition_id.as_bytes());
        out.extend_from_slice(&self.serial_id.get().to_le_bytes());
        out.extend_from_slice(self.bucket_id.as_bytes());
        out.extend_from_slice(&self.terminal_jmt_root);
        out.extend_from_slice(&self.bucket_policy_id);
        out
    }
}

/// Storage-facing record that keeps the full canonical path and terminal payload together.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoreItem {
    path: SettlementPath,
    leaf: SettlementLeaf,
}

impl StoreItem {
    pub fn check_path(&self) -> Result<(), ModelErr> {
        check_path(self.path, &self.leaf)
    }

    pub fn new(
        path: impl Into<SettlementPath>,
        leaf: impl Into<SettlementLeaf>,
    ) -> Result<Self, ModelErr> {
        let item = Self {
            path: path.into(),
            leaf: leaf.into(),
        };
        item.check_path()?;
        Ok(item)
    }

    #[must_use]
    pub const fn path(&self) -> SettlementPath {
        self.path
    }

    #[must_use]
    pub fn leaf(&self) -> &SettlementLeaf {
        &self.leaf
    }

    pub fn terminal_leaf(&self) -> Result<&TerminalLeaf, ModelErr> {
        self.leaf.as_terminal().ok_or(ModelErr::WrongLeafFamily)
    }

    pub fn right_leaf(&self) -> Result<&RightLeaf, ModelErr> {
        self.leaf.as_right().ok_or(ModelErr::WrongLeafFamily)
    }

    pub fn voucher_leaf(&self) -> Result<&VoucherLeaf, ModelErr> {
        self.leaf.as_voucher().ok_or(ModelErr::WrongLeafFamily)
    }

    pub(crate) fn into_parts(self) -> (SettlementPath, SettlementLeaf) {
        (self.path, self.leaf)
    }
}

/// Snapshot-facing record that keeps the full path together with the terminal payload and witness bytes.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SnapItem {
    path: SettlementPath,
    leaf: SettlementLeaf,
    wit: Vec<u8>,
}

impl SnapItem {
    pub fn check_path(&self) -> Result<(), ModelErr> {
        check_path(self.path, &self.leaf)
    }

    pub fn new(
        path: impl Into<SettlementPath>,
        leaf: impl Into<SettlementLeaf>,
        wit: Vec<u8>,
    ) -> Result<Self, ModelErr> {
        let item = Self {
            path: path.into(),
            leaf: leaf.into(),
            wit,
        };
        item.check_path()?;
        Ok(item)
    }

    #[must_use]
    pub const fn path(&self) -> SettlementPath {
        self.path
    }

    #[must_use]
    pub fn leaf(&self) -> &SettlementLeaf {
        &self.leaf
    }

    pub fn terminal_leaf(&self) -> Result<&TerminalLeaf, ModelErr> {
        self.leaf.as_terminal().ok_or(ModelErr::WrongLeafFamily)
    }

    pub fn right_leaf(&self) -> Result<&RightLeaf, ModelErr> {
        self.leaf.as_right().ok_or(ModelErr::WrongLeafFamily)
    }

    pub fn voucher_leaf(&self) -> Result<&VoucherLeaf, ModelErr> {
        self.leaf.as_voucher().ok_or(ModelErr::WrongLeafFamily)
    }

    #[must_use]
    pub fn wit(&self) -> &[u8] {
        &self.wit
    }
}

/// Proof-facing record that binds the global root, exact path, and committed child-root leaves.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProofItem {
    settlement_state_root: SettlementStateRoot,
    path: SettlementPath,
    definition_root_leaf: DefinitionRootLeaf,
    serial_root_leaf: SerialRootLeaf,
    leaf: SettlementLeaf,
}

impl ProofItem {
    pub fn check_path(&self) -> Result<(), ModelErr> {
        check_proof_path(
            self.path,
            self.definition_root_leaf,
            self.serial_root_leaf,
            &self.leaf,
        )
    }

    pub fn new_settlement(
        settlement_state_root: SettlementStateRoot,
        path: impl Into<SettlementPath>,
        definition_root_leaf: DefinitionRootLeaf,
        serial_root_leaf: SerialRootLeaf,
        leaf: impl Into<SettlementLeaf>,
    ) -> Result<Self, ModelErr> {
        let item = Self {
            settlement_state_root,
            path: path.into(),
            definition_root_leaf,
            serial_root_leaf,
            leaf: leaf.into(),
        };
        item.check_path()?;
        Ok(item)
    }

    #[must_use]
    pub const fn root(&self) -> SettlementStateRoot {
        self.settlement_state_root
    }

    #[must_use]
    pub const fn settlement_root(&self) -> SettlementStateRoot {
        self.settlement_state_root
    }

    #[must_use]
    pub const fn path(&self) -> SettlementPath {
        self.path
    }

    #[must_use]
    pub const fn def_leaf(&self) -> DefinitionRootLeaf {
        self.definition_root_leaf
    }

    #[must_use]
    pub const fn ser_leaf(&self) -> SerialRootLeaf {
        self.serial_root_leaf
    }

    #[must_use]
    pub fn leaf(&self) -> &SettlementLeaf {
        &self.leaf
    }

    pub fn terminal_leaf(&self) -> Result<&TerminalLeaf, ModelErr> {
        self.leaf.as_terminal().ok_or(ModelErr::WrongLeafFamily)
    }

    pub fn right_leaf(&self) -> Result<&RightLeaf, ModelErr> {
        self.leaf.as_right().ok_or(ModelErr::WrongLeafFamily)
    }

    pub fn voucher_leaf(&self) -> Result<&VoucherLeaf, ModelErr> {
        self.leaf.as_voucher().ok_or(ModelErr::WrongLeafFamily)
    }
}

/// Errors for the reference hierarchy model.
#[derive(Debug, Error)]
pub enum ModelErr {
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),
    #[error(transparent)]
    Right(#[from] RightErr),
    #[error("path definition id does not match proof leaf definition id")]
    PathDefMix,
    #[error("path terminal id does not match leaf terminal id")]
    PathLeafMix,
    #[error("path serial leaf does not match proof path")]
    PathSerMix,
    #[error("path serial id does not match leaf serial id")]
    PathSerialMix,
    #[error("definition path is missing")]
    NoDef,
    #[error("serial path is missing")]
    NoSerial,
    #[error("settlement path is missing")]
    NoTerminal,
    #[error("settlement leaf family does not match the requested terminal-only lane")]
    WrongLeafFamily,
}

/// Errors for root-semantics separation.
#[derive(Debug, Error)]
pub enum RootErr {
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("root record terminal id does not match leaf terminal id")]
    RecTerminalMix,
    #[error("root record serial id does not match leaf serial id")]
    RecSerialMix,
    #[error("duplicate root record path")]
    RecDup,
    #[error("tx digest cannot bind checkpoint root")]
    TxRootMix,
    #[error("settlement root generation does not match the V2 derivation contract")]
    GenerationMix,
}

fn check_path(path: SettlementPath, leaf: &SettlementLeaf) -> Result<(), ModelErr> {
    leaf.check_path(path)?;

    if let SettlementLeaf::Terminal(terminal_leaf) = leaf {
        if terminal_leaf.serial_id != path.serial_id.get() {
            return Err(ModelErr::PathSerialMix);
        }
    }

    Ok(())
}

fn check_proof_path(
    path: SettlementPath,
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
    leaf: &SettlementLeaf,
) -> Result<(), ModelErr> {
    if def_leaf.definition_id != path.definition_id {
        return Err(ModelErr::PathDefMix);
    }
    if ser_leaf.definition_id != path.definition_id || ser_leaf.serial_id != path.serial_id {
        return Err(ModelErr::PathSerMix);
    }

    check_path(path, leaf)
}
