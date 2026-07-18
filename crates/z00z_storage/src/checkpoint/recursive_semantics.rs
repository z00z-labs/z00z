//! Frozen payload codecs and structural identifiers for recursive checkpoint V2.
//!
//! The trace producer and the independent evaluator share these byte codecs,
//! not a semantic result or an update executor. Keeping the codec here makes
//! the trace grammar have one canonical payload path.

use z00z_crypto::{sha256_256_role, CheckpointSha256V2, CheckpointShaRole};

use crate::settlement::{
    DefinitionId, ScopeFlow, ScopeFlowItem, ScopeLeafKind, ScopeOpKind, SerialId, SettlementPath,
    SettlementStateRoot, TerminalId,
};

use super::recursive_reject::RecursiveV2Error;

const CANONICAL_HEX32_BYTES: usize = 64;
const MAX_TX_ID_BYTES: usize = 256;
/// Largest canonical replay-item payload accepted by the sole V2 codec.
///
/// Profile construction uses this exact upper bound so a profile cannot accept
/// a maximum replay row that its source owner is unable to encode.
pub(crate) const RECURSIVE_FLOW_PAYLOAD_MAX_BYTES_V2: u32 = 1
    + 2
    + MAX_TX_ID_BYTES as u32
    + 2
    + CANONICAL_HEX32_BYTES as u32
    + 4
    + 2
    + CANONICAL_HEX32_BYTES as u32
    + 4;
/// Frozen wire version of the one canonical uniqueness-precommit payload.
///
/// The streaming Nova relation imports this exact codec constant; it does not
/// maintain an independent payload grammar or length source.
pub(crate) const UNIQUENESS_PRECOMMIT_VERSION_V2: u8 = 1;
/// Exact byte width of the one canonical uniqueness-precommit payload.
pub(crate) const UNIQUENESS_PRECOMMIT_BYTES_V2: usize = 1 + 4 + 4 + 32 * 5;
/// Number of domain-separated challenge digests carried for each ID set.
///
/// The two `(alpha, beta)` pairs require four independent SHA-256 outputs per
/// set. Spent and output sets therefore carry eight challenge digests in the
/// only canonical transcript.
pub(crate) const UNIQUENESS_CHALLENGES_PER_SET_V2: usize = 4;
/// Exact byte width of the one canonical uniqueness-challenge payload.
///
/// Layout: version, the committed list precommit, `P`, both set-specific `U`
/// values, then four challenge digests for spent followed by four for output.
/// The streaming Nova relation imports this exact codec constant; challenge
/// bytes do not have a second circuit-local grammar or length source.
pub(crate) const UNIQUENESS_CHALLENGE_BYTES_V2: usize =
    1 + 32 + 32 + 32 * 2 + 32 * UNIQUENESS_CHALLENGES_PER_SET_V2 * 2;
/// Exact canonical `NetMerge` payload width.
///
/// The streaming Nova relation imports this exact codec width while decoding
/// the sole canonical source record.  It does not define another grammar or
/// accept a length supplied by the witness.
pub(crate) const NET_MERGE_BYTES_V2: usize = 1 + 1 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2 + 32;
/// Exact semantic row width committed by both Original and Sorted streams.
///
/// Layout: definition ID, serial ID, terminal ID, and the exact old/new JMT
/// value hash. The set is bound by the row tag and by its domain-separated
/// list/challenge transcript.
pub(crate) const UNIQUENESS_SEMANTIC_ROW_BYTES_V2: usize = 32 + 4 + 32 + 32;
/// Number of little-endian `u16` limbs in one complete semantic row.
pub(crate) const UNIQUENESS_SEMANTIC_ROW_LIMBS_V2: usize = UNIQUENESS_SEMANTIC_ROW_BYTES_V2 / 2;
/// Maximum total degree contributed by one semantic-row factor.
pub(crate) const UNIQUENESS_ROW_FACTOR_DEGREE_V2: usize = UNIQUENESS_SEMANTIC_ROW_LIMBS_V2 - 1;
/// Bit width of every mapped local permutation challenge.
pub(crate) const UNIQUENESS_CHALLENGE_BITS_V2: u32 = 248;
/// Conservative A-16 random-oracle precommit-query assumption.
///
/// This is deliberately a logarithm. `2^128` is not an operational request
/// limit and cannot be represented as a finite live verifier counter.
pub(crate) const UNIQUENESS_RO_QUERY_LOG2_V2: u32 = 128;
/// Exact canonical `UniquenessSorted` payload width.
pub(crate) const UNIQUENESS_SORTED_ROW_BYTES_V2: usize =
    1 + 1 + 1 + 1 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2;
const HIERARCHY_PROMOTION_BYTES_V2: usize = 1 + 32 + 32;

/// The two disjoint identifier families admitted by the V2 uniqueness
/// transcript.  Numeric values are frozen source bytes, never host labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum UniquenessSetKindV2 {
    Spent = 0,
    Output = 1,
}

/// Which precommitted list one uniqueness product row belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum UniquenessListKindV2 {
    Original = 0,
    Sorted = 1,
}

/// Which side of the non-adaptive two-pass uniqueness transcript owns a row.
///
/// `Commit` rows are consumed before any challenge row and reconstruct the
/// four ordered list commitments. `Product` rows are consumed only after the
/// SHA-derived challenges and evaluate the two independent product pairs.
/// The discriminator is part of the sole canonical source codec, so the same
/// row cannot be reinterpreted between passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum UniquenessPassV2 {
    Commit = 0,
    Product = 1,
}

impl UniquenessPassV2 {
    fn decode(value: u8) -> Result<Self, RecursiveV2Error> {
        match value {
            0 => Ok(Self::Commit),
            1 => Ok(Self::Product),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }
}

impl UniquenessListKindV2 {
    fn decode(value: u8) -> Result<Self, RecursiveV2Error> {
        match value {
            0 => Ok(Self::Original),
            1 => Ok(Self::Sorted),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }
}

impl UniquenessSetKindV2 {
    const fn tag(self) -> u8 {
        self as u8
    }
}

impl UniquenessSetKindV2 {
    fn decode(value: u8) -> Result<Self, RecursiveV2Error> {
        match value {
            0 => Ok(Self::Spent),
            1 => Ok(Self::Output),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalFlowHeaderV2 {
    pub(crate) batch_id: [u8; 32],
    pub(crate) shard_id: u32,
    pub(crate) routing_generation: u64,
    pub(crate) route_table_digest: [u8; 32],
    pub(crate) prev_root: [u8; 32],
    pub(crate) post_root: [u8; 32],
    pub(crate) spent_count: u32,
    pub(crate) output_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalFlowItemV2 {
    pub(crate) tx_id: Vec<u8>,
    pub(crate) op_kind: ScopeOpKind,
    pub(crate) definition_id: [u8; 32],
    pub(crate) serial_id: u32,
    pub(crate) terminal_id: [u8; 32],
    pub(crate) leaf_value_hash: [u8; 32],
    pub(crate) leaf_kind: ScopeLeafKind,
    pub(crate) first_definition: bool,
    pub(crate) first_serial: bool,
    pub(crate) first_object: bool,
}

/// One fixed-width semantic replay row shared by uniqueness and Net.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UniquenessSemanticRowV2 {
    pub(crate) definition_id: [u8; 32],
    pub(crate) serial_id: u32,
    pub(crate) terminal_id: [u8; 32],
    pub(crate) leaf_value_hash: [u8; 32],
}

impl UniquenessSemanticRowV2 {
    pub(crate) fn from_flow_item(item: &ScopeFlowItem) -> Result<Self, RecursiveV2Error> {
        Ok(Self {
            definition_id: decode_canonical_hex32(&item.definition_id)?,
            serial_id: item.serial_id,
            terminal_id: decode_canonical_hex32(&item.terminal_id)?,
            leaf_value_hash: item.leaf_value_hash,
        })
    }

    #[must_use]
    pub(crate) fn from_canonical_flow_item(item: &CanonicalFlowItemV2) -> Self {
        Self {
            definition_id: item.definition_id,
            serial_id: item.serial_id,
            terminal_id: item.terminal_id,
            leaf_value_hash: item.leaf_value_hash,
        }
    }

    #[must_use]
    pub(crate) fn canonical_bytes(self) -> [u8; UNIQUENESS_SEMANTIC_ROW_BYTES_V2] {
        let mut bytes = [0_u8; UNIQUENESS_SEMANTIC_ROW_BYTES_V2];
        bytes[..32].copy_from_slice(&self.definition_id);
        bytes[32..36].copy_from_slice(&self.serial_id.to_le_bytes());
        bytes[36..68].copy_from_slice(&self.terminal_id);
        bytes[68..].copy_from_slice(&self.leaf_value_hash);
        bytes
    }

    pub(crate) fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, RecursiveV2Error> {
        if bytes.len() != UNIQUENESS_SEMANTIC_ROW_BYTES_V2 {
            return Err(RecursiveV2Error::Canonical);
        }
        Ok(Self {
            definition_id: bytes[..32]
                .try_into()
                .map_err(|_| RecursiveV2Error::Canonical)?,
            serial_id: u32::from_le_bytes(
                bytes[32..36]
                    .try_into()
                    .map_err(|_| RecursiveV2Error::Canonical)?,
            ),
            terminal_id: bytes[36..68]
                .try_into()
                .map_err(|_| RecursiveV2Error::Canonical)?,
            leaf_value_hash: bytes[68..]
                .try_into()
                .map_err(|_| RecursiveV2Error::Canonical)?,
        })
    }

    #[must_use]
    pub(crate) fn same_storage_path(self, other: Self) -> bool {
        self.definition_id == other.definition_id
            && self.serial_id == other.serial_id
            && self.terminal_id == other.terminal_id
    }
}

/// Canonical two-pointer Net result for one terminal-ID union row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetEffectKindV2 {
    Close = 0,
    Delete = 1,
    Insert = 2,
    Replace = 3,
    Unchanged = 4,
}

impl NetEffectKindV2 {
    fn decode(byte: u8) -> Result<Self, RecursiveV2Error> {
        match byte {
            0 => Ok(Self::Close),
            1 => Ok(Self::Delete),
            2 => Ok(Self::Insert),
            3 => Ok(Self::Replace),
            4 => Ok(Self::Unchanged),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }
}

/// Fixed-width semantic Net row. `path_and_old.leaf_value_hash` is the old
/// leaf hash and `new_leaf_value_hash` is the new hash. Insert/delete encode
/// the absent side as zero; Close stores only the transcript binding in the
/// first 32-byte field and zeroes every other field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NetEffectV2 {
    pub(crate) kind: NetEffectKindV2,
    pub(crate) path_and_old: UniquenessSemanticRowV2,
    pub(crate) new_leaf_value_hash: [u8; 32],
}

impl NetEffectV2 {
    pub(crate) fn from_rows(
        spent: Option<UniquenessSemanticRowV2>,
        output: Option<UniquenessSemanticRowV2>,
    ) -> Result<Self, RecursiveV2Error> {
        match (spent, output) {
            (Some(old), Some(new)) => {
                if !old.same_storage_path(new) {
                    return Err(RecursiveV2Error::Invariant);
                }
                Ok(Self {
                    kind: if old.leaf_value_hash == new.leaf_value_hash {
                        NetEffectKindV2::Unchanged
                    } else {
                        NetEffectKindV2::Replace
                    },
                    path_and_old: old,
                    new_leaf_value_hash: new.leaf_value_hash,
                })
            }
            (Some(old), None) => Ok(Self {
                kind: NetEffectKindV2::Delete,
                path_and_old: old,
                new_leaf_value_hash: [0_u8; 32],
            }),
            (None, Some(new)) => {
                let mut path_and_old = new;
                path_and_old.leaf_value_hash = [0_u8; 32];
                Ok(Self {
                    kind: NetEffectKindV2::Insert,
                    path_and_old,
                    new_leaf_value_hash: new.leaf_value_hash,
                })
            }
            (None, None) => Err(RecursiveV2Error::Invariant),
        }
    }

    fn close(
        precommit_digest: [u8; 32],
        context: [u8; 32],
        spent_precommit: [u8; 32],
        output_precommit: [u8; 32],
    ) -> Self {
        Self {
            kind: NetEffectKindV2::Close,
            path_and_old: UniquenessSemanticRowV2 {
                definition_id: precommit_digest,
                serial_id: 0,
                terminal_id: context,
                leaf_value_hash: spent_precommit,
            },
            new_leaf_value_hash: output_precommit,
        }
    }
}

impl CanonicalFlowItemV2 {
    pub(crate) fn path(&self) -> SettlementPath {
        SettlementPath::new(
            DefinitionId::new(self.definition_id),
            SerialId::new(self.serial_id),
            TerminalId::new(self.terminal_id),
        )
    }
}

/// Complete pre-challenge uniqueness commitment for the two disjoint replay
/// sets.  This is a typed source record, not a caller-provided flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UniquenessPrecommitV2 {
    pub(crate) spent_count: u32,
    pub(crate) output_count: u32,
    pub(crate) spent_original_digest: [u8; 32],
    pub(crate) spent_sorted_digest: [u8; 32],
    pub(crate) output_original_digest: [u8; 32],
    pub(crate) output_sorted_digest: [u8; 32],
    pub(crate) precommit_digest: [u8; 32],
}

/// The acyclic uniqueness transcript materialized before any grand product.
///
/// This is the sole typed representation of `P`, both set-specific `U`
/// values, and the eight SHA-256 outputs. Challenge-to-field mapping and
/// product accumulation are circuit relations; this type carries canonical
/// bytes only and exposes no native validity verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UniquenessChallengesV2 {
    pub(crate) context: [u8; 32],
    pub(crate) spent_precommit: [u8; 32],
    pub(crate) output_precommit: [u8; 32],
    pub(crate) spent: [[u8; 32]; UNIQUENESS_CHALLENGES_PER_SET_V2],
    pub(crate) output: [[u8; 32]; UNIQUENESS_CHALLENGES_PER_SET_V2],
}

pub(crate) fn encode_uniqueness_precommit(flow: &ScopeFlow) -> Result<Vec<u8>, RecursiveV2Error> {
    let mut spent = Vec::new();
    let mut output = Vec::new();
    for kind in [ScopeOpKind::Delete, ScopeOpKind::Put] {
        for item in flow.items.iter().filter(|item| item.op_kind == kind) {
            let row = UniquenessSemanticRowV2::from_flow_item(item)?;
            match item.op_kind {
                ScopeOpKind::Delete => spent.push(row),
                ScopeOpKind::Put => output.push(row),
            }
        }
    }
    uniqueness_precommit_from_rows(&spent, &output).and_then(encode_uniqueness_precommit_value)
}

pub(crate) fn decode_uniqueness_precommit(
    bytes: &[u8],
) -> Result<UniquenessPrecommitV2, RecursiveV2Error> {
    if bytes.len() != UNIQUENESS_PRECOMMIT_BYTES_V2 {
        return Err(RecursiveV2Error::Canonical);
    }
    let mut cursor = 0;
    if take_u8(bytes, &mut cursor)? != UNIQUENESS_PRECOMMIT_VERSION_V2 {
        return Err(RecursiveV2Error::Version);
    }
    let value = UniquenessPrecommitV2 {
        spent_count: take_u32(bytes, &mut cursor)?,
        output_count: take_u32(bytes, &mut cursor)?,
        spent_original_digest: take_array32(bytes, &mut cursor)?,
        spent_sorted_digest: take_array32(bytes, &mut cursor)?,
        output_original_digest: take_array32(bytes, &mut cursor)?,
        output_sorted_digest: take_array32(bytes, &mut cursor)?,
        precommit_digest: take_array32(bytes, &mut cursor)?,
    };
    if cursor != bytes.len()
        || value.precommit_digest
            != derive_precommit_digest(
                value.spent_count,
                value.output_count,
                value.spent_original_digest,
                value.spent_sorted_digest,
                value.output_original_digest,
                value.output_sorted_digest,
            )
    {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(value)
}

pub(crate) fn uniqueness_precommit_from_rows(
    spent: &[UniquenessSemanticRowV2],
    output: &[UniquenessSemanticRowV2],
) -> Result<UniquenessPrecommitV2, RecursiveV2Error> {
    let spent_count = u32::try_from(spent.len()).map_err(|_| RecursiveV2Error::Limit)?;
    let output_count = u32::try_from(output.len()).map_err(|_| RecursiveV2Error::Limit)?;
    let spent_original_digest = digest_semantic_rows(CheckpointShaRole::SpentOriginalIds, spent)?;
    let output_original_digest =
        digest_semantic_rows(CheckpointShaRole::OutputOriginalIds, output)?;
    let mut spent_sorted = spent.to_vec();
    let mut output_sorted = output.to_vec();
    spent_sorted.sort_unstable_by_key(|row| row.terminal_id);
    output_sorted.sort_unstable_by_key(|row| row.terminal_id);
    if spent_sorted
        .windows(2)
        .any(|pair| pair[0].terminal_id == pair[1].terminal_id)
        || output_sorted
            .windows(2)
            .any(|pair| pair[0].terminal_id == pair[1].terminal_id)
        || !cross_set_replacements_are_same_path(&spent_sorted, &output_sorted)
    {
        return Err(RecursiveV2Error::DuplicateIdentifier);
    }
    let spent_sorted_digest =
        digest_semantic_rows(CheckpointShaRole::SpentSortedIds, &spent_sorted)?;
    let output_sorted_digest =
        digest_semantic_rows(CheckpointShaRole::OutputSortedIds, &output_sorted)?;
    let precommit_digest = derive_precommit_digest(
        spent_count,
        output_count,
        spent_original_digest,
        spent_sorted_digest,
        output_original_digest,
        output_sorted_digest,
    );
    Ok(UniquenessPrecommitV2 {
        spent_count,
        output_count,
        spent_original_digest,
        spent_sorted_digest,
        output_original_digest,
        output_sorted_digest,
        precommit_digest,
    })
}

fn encode_uniqueness_precommit_value(
    value: UniquenessPrecommitV2,
) -> Result<Vec<u8>, RecursiveV2Error> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(UNIQUENESS_PRECOMMIT_BYTES_V2)
        .map_err(|_| RecursiveV2Error::Limit)?;
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.extend_from_slice(&value.spent_count.to_le_bytes());
    bytes.extend_from_slice(&value.output_count.to_le_bytes());
    bytes.extend_from_slice(&value.spent_original_digest);
    bytes.extend_from_slice(&value.spent_sorted_digest);
    bytes.extend_from_slice(&value.output_original_digest);
    bytes.extend_from_slice(&value.output_sorted_digest);
    bytes.extend_from_slice(&value.precommit_digest);
    Ok(bytes)
}

pub(crate) fn encode_uniqueness_challenge(
    pre_context: [u8; 32],
    grammar_digest: [u8; 32],
    precommit: UniquenessPrecommitV2,
) -> Vec<u8> {
    let challenges = derive_uniqueness_challenges(pre_context, grammar_digest, precommit);
    let mut bytes = Vec::with_capacity(UNIQUENESS_CHALLENGE_BYTES_V2);
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.extend_from_slice(&precommit.precommit_digest);
    bytes.extend_from_slice(&challenges.context);
    bytes.extend_from_slice(&challenges.spent_precommit);
    bytes.extend_from_slice(&challenges.output_precommit);
    for digest in challenges.spent.iter().chain(&challenges.output) {
        bytes.extend_from_slice(digest);
    }
    bytes
}

pub(crate) fn decode_uniqueness_challenge(
    bytes: &[u8],
    pre_context: [u8; 32],
    grammar_digest: [u8; 32],
    precommit: UniquenessPrecommitV2,
) -> Result<UniquenessChallengesV2, RecursiveV2Error> {
    if bytes.len() != UNIQUENESS_CHALLENGE_BYTES_V2 || bytes[0] != UNIQUENESS_PRECOMMIT_VERSION_V2 {
        return Err(RecursiveV2Error::Canonical);
    }
    let committed_precommit: [u8; 32] = bytes[1..33]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let expected = encode_uniqueness_challenge(pre_context, grammar_digest, precommit);
    if committed_precommit != precommit.precommit_digest || expected.as_slice() != bytes {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(derive_uniqueness_challenges(
        pre_context,
        grammar_digest,
        precommit,
    ))
}

fn derive_uniqueness_challenges(
    pre_context: [u8; 32],
    grammar_digest: [u8; 32],
    precommit: UniquenessPrecommitV2,
) -> UniquenessChallengesV2 {
    let spent_precommit = derive_set_precommit(
        pre_context,
        UniquenessSetKindV2::Spent,
        precommit.spent_count,
        precommit.spent_original_digest,
        precommit.spent_sorted_digest,
    );
    let output_precommit = derive_set_precommit(
        pre_context,
        UniquenessSetKindV2::Output,
        precommit.output_count,
        precommit.output_original_digest,
        precommit.output_sorted_digest,
    );
    UniquenessChallengesV2 {
        context: pre_context,
        spent_precommit,
        output_precommit,
        spent: derive_set_challenges(spent_precommit, grammar_digest, UniquenessSetKindV2::Spent),
        output: derive_set_challenges(
            output_precommit,
            grammar_digest,
            UniquenessSetKindV2::Output,
        ),
    }
}

fn derive_set_precommit(
    pre_context: [u8; 32],
    set: UniquenessSetKindV2,
    count: u32,
    original: [u8; 32],
    sorted: [u8; 32],
) -> [u8; 32] {
    sha256_256_role(
        CheckpointShaRole::IdPrecommit,
        &[
            &pre_context,
            &[set.tag()],
            &count.to_le_bytes(),
            &original,
            &sorted,
        ],
    )
}

/// Derive the same pair from the immutable pass-one trace precommit fields.
/// This is the bridge used by the acyclic statement builder after replay has
/// independently established the exact counts and all four list digests.
#[allow(clippy::too_many_arguments)]
pub(crate) fn uniqueness_set_precommits_from_parts(
    pre_context: [u8; 32],
    spent_count: u32,
    output_count: u32,
    spent_original: [u8; 32],
    spent_sorted: [u8; 32],
    output_original: [u8; 32],
    output_sorted: [u8; 32],
) -> ([u8; 32], [u8; 32]) {
    (
        derive_set_precommit(
            pre_context,
            UniquenessSetKindV2::Spent,
            spent_count,
            spent_original,
            spent_sorted,
        ),
        derive_set_precommit(
            pre_context,
            UniquenessSetKindV2::Output,
            output_count,
            output_original,
            output_sorted,
        ),
    )
}

fn derive_set_challenges(
    set_precommit: [u8; 32],
    grammar_digest: [u8; 32],
    set: UniquenessSetKindV2,
) -> [[u8; 32]; UNIQUENESS_CHALLENGES_PER_SET_V2] {
    std::array::from_fn(|index| {
        let pair = u8::try_from(index / 2).expect("two challenge pairs fit u8");
        let coordinate = u8::try_from(index % 2).expect("challenge coordinate fits u8");
        sha256_256_role(
            CheckpointShaRole::IdChallenge,
            &[
                &set_precommit,
                &grammar_digest,
                &[set.tag()],
                &[pair],
                &[coordinate],
            ],
        )
    })
}

/// Encode one sorted identifier row for the canonical uniqueness transcript.
///
/// Original rows are the terminal identifiers decoded from `ReplayInput` and
/// `ReplayOutput`; sorted rows must be separately source-authenticated so the
/// circuit can later prove their exact order and permutation relation instead
/// of accepting a native sorter verdict.
pub(crate) fn encode_uniqueness_sorted_row(
    pass: UniquenessPassV2,
    set: UniquenessSetKindV2,
    list: UniquenessListKindV2,
    row: UniquenessSemanticRowV2,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(UNIQUENESS_SORTED_ROW_BYTES_V2);
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.push(pass as u8);
    bytes.push(set as u8);
    bytes.push(list as u8);
    bytes.extend_from_slice(&row.canonical_bytes());
    bytes
}

/// Decode the one fixed-width sorted identifier source record.
pub(crate) fn decode_uniqueness_sorted_row(
    bytes: &[u8],
) -> Result<
    (
        UniquenessPassV2,
        UniquenessSetKindV2,
        UniquenessListKindV2,
        UniquenessSemanticRowV2,
    ),
    RecursiveV2Error,
> {
    if bytes.len() != UNIQUENESS_SORTED_ROW_BYTES_V2
        || bytes.first().copied() != Some(UNIQUENESS_PRECOMMIT_VERSION_V2)
    {
        return Err(RecursiveV2Error::Canonical);
    }
    let pass = UniquenessPassV2::decode(bytes[1])?;
    let set = UniquenessSetKindV2::decode(bytes[2])?;
    let list = UniquenessListKindV2::decode(bytes[3])?;
    Ok((
        pass,
        set,
        list,
        UniquenessSemanticRowV2::from_canonical_bytes(&bytes[4..])?,
    ))
}

pub(crate) fn encode_net_merge(
    precommit: UniquenessPrecommitV2,
    challenge: UniquenessChallengesV2,
) -> Vec<u8> {
    encode_net_effect(NetEffectV2::close(
        precommit.precommit_digest,
        challenge.context,
        challenge.spent_precommit,
        challenge.output_precommit,
    ))
}

#[must_use]
pub(crate) fn encode_net_effect(effect: NetEffectV2) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(NET_MERGE_BYTES_V2);
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.push(effect.kind as u8);
    bytes.extend_from_slice(&effect.path_and_old.canonical_bytes());
    bytes.extend_from_slice(&effect.new_leaf_value_hash);
    bytes
}

pub(crate) fn decode_net_effect(bytes: &[u8]) -> Result<NetEffectV2, RecursiveV2Error> {
    if bytes.len() != NET_MERGE_BYTES_V2
        || bytes.first().copied() != Some(UNIQUENESS_PRECOMMIT_VERSION_V2)
    {
        return Err(RecursiveV2Error::Canonical);
    }
    let effect = NetEffectV2 {
        kind: NetEffectKindV2::decode(bytes[1])?,
        path_and_old: UniquenessSemanticRowV2::from_canonical_bytes(
            &bytes[2..2 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2],
        )?,
        new_leaf_value_hash: bytes[2 + UNIQUENESS_SEMANTIC_ROW_BYTES_V2..]
            .try_into()
            .map_err(|_| RecursiveV2Error::Canonical)?,
    };
    match effect.kind {
        NetEffectKindV2::Close => {
            if effect.path_and_old.serial_id != 0 {
                return Err(RecursiveV2Error::Canonical);
            }
        }
        NetEffectKindV2::Delete => {
            if effect.new_leaf_value_hash != [0_u8; 32] {
                return Err(RecursiveV2Error::Canonical);
            }
        }
        NetEffectKindV2::Insert => {
            if effect.path_and_old.leaf_value_hash != [0_u8; 32] {
                return Err(RecursiveV2Error::Canonical);
            }
        }
        NetEffectKindV2::Replace => {
            if effect.path_and_old.leaf_value_hash == effect.new_leaf_value_hash {
                return Err(RecursiveV2Error::Canonical);
            }
        }
        NetEffectKindV2::Unchanged => {
            if effect.path_and_old.leaf_value_hash != effect.new_leaf_value_hash {
                return Err(RecursiveV2Error::Canonical);
            }
        }
    }
    Ok(effect)
}

pub(crate) fn decode_net_merge(
    bytes: &[u8],
    precommit: UniquenessPrecommitV2,
    challenge: UniquenessChallengesV2,
) -> Result<(), RecursiveV2Error> {
    let effect = decode_net_effect(bytes)?;
    if effect.kind != NetEffectKindV2::Close || bytes != encode_net_merge(precommit, challenge) {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(())
}

pub(crate) fn encode_hierarchy_promotion(
    definition_root: [u8; 32],
    update_trace_digest: [u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HIERARCHY_PROMOTION_BYTES_V2);
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.extend_from_slice(&definition_root);
    bytes.extend_from_slice(&update_trace_digest);
    bytes
}

pub(crate) fn decode_hierarchy_promotion(
    bytes: &[u8],
    expected_definition_root: [u8; 32],
    expected_update_trace_digest: [u8; 32],
) -> Result<(), RecursiveV2Error> {
    if decode_hierarchy_promotion_fields(bytes)?
        != (expected_definition_root, expected_update_trace_digest)
    {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(())
}

pub(crate) fn decode_hierarchy_promotion_fields(
    bytes: &[u8],
) -> Result<([u8; 32], [u8; 32]), RecursiveV2Error> {
    if bytes.len() != HIERARCHY_PROMOTION_BYTES_V2
        || bytes.first().copied() != Some(UNIQUENESS_PRECOMMIT_VERSION_V2)
    {
        return Err(RecursiveV2Error::Canonical);
    }
    let definition_root = bytes[1..33]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let update_trace_digest = bytes[33..65]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    Ok((definition_root, update_trace_digest))
}

fn digest_semantic_rows(
    role: CheckpointShaRole,
    rows: &[UniquenessSemanticRowV2],
) -> Result<[u8; 32], RecursiveV2Error> {
    let mut digest = CheckpointSha256V2::new(role);
    let count = u32::try_from(rows.len()).map_err(|_| RecursiveV2Error::Limit)?;
    digest.update_part(&count.to_le_bytes())?;
    for row in rows {
        digest.update_part(&row.canonical_bytes())?;
    }
    Ok(digest.finalize())
}

fn derive_precommit_digest(
    spent_count: u32,
    output_count: u32,
    spent_original_digest: [u8; 32],
    spent_sorted_digest: [u8; 32],
    output_original_digest: [u8; 32],
    output_sorted_digest: [u8; 32],
) -> [u8; 32] {
    sha256_256_role(
        CheckpointShaRole::IdPrecommit,
        &[
            b"z00z.recursive.v2.uniqueness-precommit",
            &spent_count.to_le_bytes(),
            &output_count.to_le_bytes(),
            &spent_original_digest,
            &spent_sorted_digest,
            &output_original_digest,
            &output_sorted_digest,
        ],
    )
}

fn cross_set_replacements_are_same_path(
    left: &[UniquenessSemanticRowV2],
    right: &[UniquenessSemanticRowV2],
) -> bool {
    let (mut l, mut r) = (0_usize, 0_usize);
    while l < left.len() && r < right.len() {
        match left[l].terminal_id.cmp(&right[r].terminal_id) {
            std::cmp::Ordering::Less => l += 1,
            std::cmp::Ordering::Greater => r += 1,
            std::cmp::Ordering::Equal => {
                if !left[l].same_storage_path(right[r]) {
                    return false;
                }
                l += 1;
                r += 1;
            }
        }
    }
    true
}

#[cfg(test)]
pub(crate) fn encode_flow_header(flow: &ScopeFlow) -> Result<Vec<u8>, RecursiveV2Error> {
    let prev_root = decode_canonical_hex32(&flow.root_flow.prev_root)?;
    let post_root = decode_canonical_hex32(&flow.root_flow.post_root)?;
    encode_flow_header_fields(flow, prev_root, post_root)
}

/// Encode a transition header with roots derived by the sole V2 root owner.
///
/// `ScopeFlow` is also used by the non-recursive settlement lane and therefore
/// retains its native HJMT roots.  Recursive V2 never reuses those fields as a
/// root assertion: it substitutes the independently derived V2 pre/post roots
/// before the record is committed to the proving trace.
pub(crate) fn encode_flow_header_with_v2_roots(
    flow: &ScopeFlow,
    prev_root: SettlementStateRoot,
    post_root: SettlementStateRoot,
) -> Result<Vec<u8>, RecursiveV2Error> {
    encode_flow_header_fields(flow, *prev_root.as_bytes(), *post_root.as_bytes())
}

fn encode_flow_header_fields(
    flow: &ScopeFlow,
    prev_root: [u8; 32],
    post_root: [u8; 32],
) -> Result<Vec<u8>, RecursiveV2Error> {
    let spent_count = u32::try_from(
        flow.items
            .iter()
            .filter(|item| item.op_kind == ScopeOpKind::Delete)
            .count(),
    )
    .map_err(|_| RecursiveV2Error::Limit)?;
    let output_count = u32::try_from(
        flow.items
            .iter()
            .filter(|item| item.op_kind == ScopeOpKind::Put)
            .count(),
    )
    .map_err(|_| RecursiveV2Error::Limit)?;
    let mut bytes = Vec::with_capacity(6 * (2 + CANONICAL_HEX32_BYTES) + 12);
    append_string(&mut bytes, &flow.batch_id)?;
    bytes.extend_from_slice(&flow.shard_id.to_le_bytes());
    bytes.extend_from_slice(&flow.routing_generation.to_le_bytes());
    append_string(&mut bytes, &flow.route_table_digest)?;
    append_hex32(&mut bytes, prev_root)?;
    append_hex32(&mut bytes, post_root)?;
    bytes.extend_from_slice(&spent_count.to_le_bytes());
    bytes.extend_from_slice(&output_count.to_le_bytes());
    let _ = decode_flow_header(&bytes)?;
    Ok(bytes)
}

pub(crate) fn decode_flow_header(bytes: &[u8]) -> Result<CanonicalFlowHeaderV2, RecursiveV2Error> {
    let mut cursor = 0;
    let batch_id = take_hex32(bytes, &mut cursor)?;
    let shard_id = take_u32(bytes, &mut cursor)?;
    let routing_generation = take_u64(bytes, &mut cursor)?;
    let route_table_digest = take_hex32(bytes, &mut cursor)?;
    let prev_root = take_hex32(bytes, &mut cursor)?;
    let post_root = take_hex32(bytes, &mut cursor)?;
    let spent_count = take_u32(bytes, &mut cursor)?;
    let output_count = take_u32(bytes, &mut cursor)?;
    if cursor != bytes.len() {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(CanonicalFlowHeaderV2 {
        batch_id,
        shard_id,
        routing_generation,
        route_table_digest,
        prev_root,
        post_root,
        spent_count,
        output_count,
    })
}

pub(crate) fn encode_flow_item(item: &ScopeFlowItem) -> Result<Vec<u8>, RecursiveV2Error> {
    let mut bytes = Vec::with_capacity(2 + item.tx_id.len() + 2 * (2 + CANONICAL_HEX32_BYTES) + 10);
    bytes.push(match item.op_kind {
        ScopeOpKind::Put => 1,
        ScopeOpKind::Delete => 2,
    });
    append_string(&mut bytes, &item.tx_id)?;
    append_string(&mut bytes, &item.definition_id)?;
    bytes.extend_from_slice(&item.serial_id.to_le_bytes());
    append_string(&mut bytes, &item.terminal_id)?;
    bytes.extend_from_slice(&item.leaf_value_hash);
    bytes.push(match item.leaf_family {
        ScopeLeafKind::Terminal => 1,
        ScopeLeafKind::Right => 2,
    });
    bytes.extend_from_slice(&[
        u8::from(item.first_seen.definition),
        u8::from(item.first_seen.serial),
        u8::from(item.first_seen.object),
    ]);
    let _ = decode_flow_item(&bytes)?;
    Ok(bytes)
}

/// Frozen discriminator for the four checkpoint-core `CommitTypedEvent` rows.
/// Flow items have exactly one representation, `ReplayInput`/`ReplayOutput`,
/// and therefore never enter this codec. It is deliberately outside the flow
/// codec's `{Put=1, Delete=2}` first-byte alphabet.
pub(crate) const TYPED_CHECKPOINT_COMMITMENT_VERSION_V2: u8 = 0xc2;
pub(crate) const TYPED_CHECKPOINT_COMMITMENT_BYTES_V2: usize = 1 + 1 + 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TypedCheckpointCommitmentKindV2 {
    DeltaRoot = 1,
    WitnessRoot = 2,
    JournalDigest = 3,
    CheckpointLinkDigest = 4,
}

impl TypedCheckpointCommitmentKindV2 {
    pub(crate) const ALL: [Self; 4] = [
        Self::DeltaRoot,
        Self::WitnessRoot,
        Self::JournalDigest,
        Self::CheckpointLinkDigest,
    ];

    fn decode(value: u8) -> Result<Self, RecursiveV2Error> {
        match value {
            1 => Ok(Self::DeltaRoot),
            2 => Ok(Self::WitnessRoot),
            3 => Ok(Self::JournalDigest),
            4 => Ok(Self::CheckpointLinkDigest),
            _ => Err(RecursiveV2Error::Canonical),
        }
    }
}

pub(crate) fn encode_typed_checkpoint_commitment(
    kind: TypedCheckpointCommitmentKindV2,
    digest: [u8; 32],
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(TYPED_CHECKPOINT_COMMITMENT_BYTES_V2);
    bytes.push(TYPED_CHECKPOINT_COMMITMENT_VERSION_V2);
    bytes.push(kind as u8);
    bytes.extend_from_slice(&digest);
    bytes
}

pub(crate) fn decode_typed_checkpoint_commitment(
    bytes: &[u8],
) -> Result<(TypedCheckpointCommitmentKindV2, [u8; 32]), RecursiveV2Error> {
    if bytes.len() != TYPED_CHECKPOINT_COMMITMENT_BYTES_V2
        || bytes[0] != TYPED_CHECKPOINT_COMMITMENT_VERSION_V2
    {
        return Err(RecursiveV2Error::Canonical);
    }
    let kind = TypedCheckpointCommitmentKindV2::decode(bytes[1])?;
    let digest = bytes[2..]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    Ok((kind, digest))
}

pub(crate) fn decode_flow_item(bytes: &[u8]) -> Result<CanonicalFlowItemV2, RecursiveV2Error> {
    let mut cursor = 0;
    let op_kind = match take_u8(bytes, &mut cursor)? {
        1 => ScopeOpKind::Put,
        2 => ScopeOpKind::Delete,
        _ => return Err(RecursiveV2Error::Canonical),
    };
    let tx_id = take_string(bytes, &mut cursor, MAX_TX_ID_BYTES)?;
    if tx_id.is_empty() || !tx_id.iter().all(u8::is_ascii_graphic) {
        return Err(RecursiveV2Error::Canonical);
    }
    let definition_id = take_hex32(bytes, &mut cursor)?;
    let serial_id = take_u32(bytes, &mut cursor)?;
    let terminal_id = take_hex32(bytes, &mut cursor)?;
    let leaf_value_hash = take_array32(bytes, &mut cursor)?;
    let leaf_kind = match take_u8(bytes, &mut cursor)? {
        1 => ScopeLeafKind::Terminal,
        2 => ScopeLeafKind::Right,
        _ => return Err(RecursiveV2Error::Canonical),
    };
    let first_definition = take_bool(bytes, &mut cursor)?;
    let first_serial = take_bool(bytes, &mut cursor)?;
    let first_object = take_bool(bytes, &mut cursor)?;
    if cursor != bytes.len() {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(CanonicalFlowItemV2 {
        tx_id,
        op_kind,
        definition_id,
        serial_id,
        terminal_id,
        leaf_value_hash,
        leaf_kind,
        first_definition,
        first_serial,
        first_object,
    })
}

fn append_string(out: &mut Vec<u8>, value: &str) -> Result<(), RecursiveV2Error> {
    let len = u16::try_from(value.len()).map_err(|_| RecursiveV2Error::Limit)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn append_hex32(out: &mut Vec<u8>, value: [u8; 32]) -> Result<(), RecursiveV2Error> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = [0_u8; CANONICAL_HEX32_BYTES];
    for (index, byte) in value.into_iter().enumerate() {
        let offset = index.checked_mul(2).ok_or(RecursiveV2Error::Overflow)?;
        encoded[offset] = HEX[usize::from(byte >> 4)];
        encoded[offset + 1] = HEX[usize::from(byte & 0x0f)];
    }
    let value = std::str::from_utf8(&encoded).map_err(|_| RecursiveV2Error::Canonical)?;
    append_string(out, value)
}

fn take_string(
    bytes: &[u8],
    cursor: &mut usize,
    max_len: usize,
) -> Result<Vec<u8>, RecursiveV2Error> {
    let len = usize::from(take_u16(bytes, cursor)?);
    if len > max_len {
        return Err(RecursiveV2Error::Limit);
    }
    let end = cursor.checked_add(len).ok_or(RecursiveV2Error::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(RecursiveV2Error::Canonical)?
        .to_vec();
    *cursor = end;
    Ok(value)
}

fn take_hex32(bytes: &[u8], cursor: &mut usize) -> Result<[u8; 32], RecursiveV2Error> {
    let value = take_string(bytes, cursor, CANONICAL_HEX32_BYTES)?;
    let value = std::str::from_utf8(&value).map_err(|_| RecursiveV2Error::Canonical)?;
    decode_canonical_hex32(value)
}

pub(crate) fn decode_canonical_hex32(value: &str) -> Result<[u8; 32], RecursiveV2Error> {
    let bytes = value.as_bytes();
    if bytes.len() != CANONICAL_HEX32_BYTES
        || !bytes
            .iter()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(RecursiveV2Error::Canonical);
    }
    let mut output = [0_u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        let offset = index.checked_mul(2).ok_or(RecursiveV2Error::Overflow)?;
        *byte = (hex_nibble(bytes[offset])? << 4) | hex_nibble(bytes[offset + 1])?;
    }
    Ok(output)
}

fn hex_nibble(value: u8) -> Result<u8, RecursiveV2Error> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(RecursiveV2Error::Canonical),
    }
}

fn take_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8, RecursiveV2Error> {
    let value = *bytes.get(*cursor).ok_or(RecursiveV2Error::Canonical)?;
    *cursor = cursor.checked_add(1).ok_or(RecursiveV2Error::Overflow)?;
    Ok(value)
}

fn take_u16(bytes: &[u8], cursor: &mut usize) -> Result<u16, RecursiveV2Error> {
    let end = cursor.checked_add(2).ok_or(RecursiveV2Error::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(RecursiveV2Error::Canonical)?
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    *cursor = end;
    Ok(u16::from_le_bytes(value))
}

fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, RecursiveV2Error> {
    let end = cursor.checked_add(4).ok_or(RecursiveV2Error::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(RecursiveV2Error::Canonical)?
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    *cursor = end;
    Ok(u32::from_le_bytes(value))
}

fn take_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, RecursiveV2Error> {
    let end = cursor.checked_add(8).ok_or(RecursiveV2Error::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(RecursiveV2Error::Canonical)?
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    *cursor = end;
    Ok(u64::from_le_bytes(value))
}

fn take_array32(bytes: &[u8], cursor: &mut usize) -> Result<[u8; 32], RecursiveV2Error> {
    let end = cursor.checked_add(32).ok_or(RecursiveV2Error::Overflow)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(RecursiveV2Error::Canonical)?
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    *cursor = end;
    Ok(value)
}

fn take_bool(bytes: &[u8], cursor: &mut usize) -> Result<bool, RecursiveV2Error> {
    match take_u8(bytes, cursor)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(RecursiveV2Error::Canonical),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        decode_canonical_hex32, decode_flow_header, decode_flow_item, decode_net_effect,
        decode_uniqueness_challenge, decode_uniqueness_precommit, decode_uniqueness_sorted_row,
        encode_flow_header, encode_flow_item, encode_net_effect, encode_uniqueness_challenge,
        encode_uniqueness_precommit, encode_uniqueness_sorted_row, NetEffectKindV2, NetEffectV2,
        UniquenessListKindV2, UniquenessPassV2, UniquenessSemanticRowV2, UniquenessSetKindV2,
        UNIQUENESS_CHALLENGE_BITS_V2, UNIQUENESS_ROW_FACTOR_DEGREE_V2, UNIQUENESS_RO_QUERY_LOG2_V2,
        UNIQUENESS_SEMANTIC_ROW_LIMBS_V2,
    };
    use crate::checkpoint::recursive_trace::RecursiveTraceOpcodeV2;
    use crate::settlement::{
        ScopeFlow, ScopeFlowItem, ScopeLeafKind, ScopeOpKind, ScopeRootFlow, ScopeSeen,
    };

    type FormalPolynomial = BTreeMap<(usize, usize), i128>;

    fn multiply_polynomials(left: &FormalPolynomial, right: &FormalPolynomial) -> FormalPolynomial {
        let mut product = FormalPolynomial::new();
        for ((left_alpha, left_beta), left_coefficient) in left {
            for ((right_alpha, right_beta), right_coefficient) in right {
                let exponent = (left_alpha + right_alpha, left_beta + right_beta);
                *product.entry(exponent).or_default() += left_coefficient * right_coefficient;
            }
        }
        product.retain(|_, coefficient| *coefficient != 0);
        product
    }

    fn multiset_polynomial(rows: &[[i16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2]]) -> FormalPolynomial {
        let mut product = FormalPolynomial::from([((0, 0), 1)]);
        for row in rows {
            let mut factor = FormalPolynomial::from([((1, 0), 1)]);
            for (beta_degree, limb) in row.iter().copied().enumerate() {
                if limb != 0 {
                    factor.insert((0, beta_degree), -i128::from(limb));
                }
            }
            product = multiply_polynomials(&product, &factor);
        }
        product
    }

    fn subtract_polynomials(left: &FormalPolynomial, right: &FormalPolynomial) -> FormalPolynomial {
        let mut difference = left.clone();
        for (exponent, coefficient) in right {
            *difference.entry(*exponent).or_default() -= coefficient;
        }
        difference.retain(|_, coefficient| *coefficient != 0);
        difference
    }

    fn polynomial_degree(polynomial: &FormalPolynomial) -> usize {
        polynomial
            .keys()
            .map(|(alpha, beta)| alpha + beta)
            .max()
            .unwrap_or(0)
    }

    fn evaluate_row(row: &[u16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2], beta: u64, modulus: u64) -> u64 {
        row.iter().rev().fold(0_u64, |value, limb| {
            (value * beta + u64::from(*limb)) % modulus
        })
    }

    fn evaluate_product(
        rows: &[[u16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2]],
        alpha: u64,
        beta: u64,
        modulus: u64,
    ) -> u64 {
        rows.iter().fold(1_u64, |product, row| {
            let encoded = evaluate_row(row, beta, modulus);
            product * ((alpha + modulus - encoded) % modulus) % modulus
        })
    }

    #[test]
    fn full_row_polynomial_bound() {
        assert_eq!(UNIQUENESS_SEMANTIC_ROW_LIMBS_V2, 50);
        assert_eq!(UNIQUENESS_ROW_FACTOR_DEGREE_V2, 49);
        assert_eq!(UNIQUENESS_CHALLENGE_BITS_V2, 248);
        assert_eq!(UNIQUENESS_RO_QUERY_LOG2_V2, 128);

        let mut first = [0_i16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2];
        first[0] = 1;
        first[UNIQUENESS_ROW_FACTOR_DEGREE_V2] = 2;
        let mut second = [0_i16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2];
        second[1] = 3;
        second[UNIQUENESS_ROW_FACTOR_DEGREE_V2] = 1;
        let mut changed = second;
        changed[17] = 4;

        let original = multiset_polynomial(&[first, second]);
        let reordered = multiset_polynomial(&[second, first]);
        assert_eq!(
            original, reordered,
            "multiset order must not change the product"
        );

        let different = multiset_polynomial(&[first, changed]);
        let difference = subtract_polynomials(&original, &different);
        assert!(
            !difference.is_empty(),
            "unequal semantic-row multisets need a nonzero polynomial"
        );
        assert!(
            polynomial_degree(&difference) <= UNIQUENESS_ROW_FACTOR_DEGREE_V2 * 2,
            "two full semantic rows must have total degree at most 98"
        );

        let duplicated = multiset_polynomial(&[first, first]);
        assert_ne!(
            original, duplicated,
            "a duplicate cannot be erased from the formal multiset product"
        );
    }

    #[test]
    fn toy_field_pair_independence() {
        const MODULUS: u64 = 257;
        let mut first = [0_u16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2];
        first[0] = 1;
        first[2] = 2;
        let mut second = [0_u16; UNIQUENESS_SEMANTIC_ROW_LIMBS_V2];
        second[0] = 4;
        second[1] = 1;
        let mut changed = second;
        changed[2] = 3;
        let original = [first, second];
        let reordered = [second, first];
        let different = [first, changed];

        let mut collision_count = 0_u64;
        let mut zero_factor_seen = false;
        for alpha in 0..MODULUS {
            for beta in 0..MODULUS {
                let expected = evaluate_product(&original, alpha, beta, MODULUS);
                assert_eq!(
                    expected,
                    evaluate_product(&reordered, alpha, beta, MODULUS),
                    "equal multisets must agree at every toy-field point"
                );
                if expected == evaluate_product(&different, alpha, beta, MODULUS) {
                    collision_count += 1;
                }
                zero_factor_seen |= alpha == evaluate_row(&first, beta, MODULUS);
            }
        }

        assert!(
            zero_factor_seen,
            "the corpus must cover a zero grand-product factor"
        );
        assert!(
            collision_count > 0,
            "the toy corpus must expose finite-field collisions"
        );
        let single_pair_space = MODULUS * MODULUS;
        assert!(
            collision_count <= 4 * MODULUS,
            "the exhaustive zero count must respect the concrete degree-four bound"
        );
        let two_pair_collisions = u128::from(collision_count).pow(2);
        let two_pair_space = u128::from(single_pair_space).pow(2);
        assert!(
            two_pair_collisions < two_pair_space,
            "two independently sampled pairs must square a nontrivial one-pair collision rate"
        );
    }

    #[test]
    fn flow_payloads_are_canonical_and_round_trip_through_one_codec() {
        let hex = "11".repeat(32);
        let header = ScopeFlow {
            batch_id: hex.clone(),
            shard_id: 7,
            routing_generation: 9,
            route_table_digest: "22".repeat(32),
            items: Vec::new(),
            root_flow: ScopeRootFlow {
                prev_root: "33".repeat(32),
                post_root: "44".repeat(32),
            },
        };
        let item = ScopeFlowItem {
            tx_id: "semantic-0000".to_string(),
            op_kind: ScopeOpKind::Put,
            definition_id: hex,
            serial_id: 12,
            terminal_id: "55".repeat(32),
            leaf_value_hash: [0x56; 32],
            leaf_family: ScopeLeafKind::Terminal,
            first_seen: ScopeSeen {
                definition: true,
                serial: true,
                object: true,
            },
        };

        let decoded_header = decode_flow_header(&encode_flow_header(&header).expect("header"))
            .expect("canonical header");
        let decoded_item =
            decode_flow_item(&encode_flow_item(&item).expect("item")).expect("canonical item");
        assert_eq!(decoded_header.batch_id, [0x11; 32]);
        assert_eq!(decoded_header.post_root, [0x44; 32]);
        assert_eq!(decoded_item.terminal_id, [0x55; 32]);
        assert_eq!(decoded_item.leaf_value_hash, [0x56; 32]);
        assert_eq!(decoded_item.op_kind, ScopeOpKind::Put);
        assert!(decoded_item.first_object);
        assert!(decode_canonical_hex32(&"AA".repeat(32)).is_err());
    }

    #[test]
    fn uniqueness_payloads_bind_both_replay_sets_before_challenge() {
        let flow = ScopeFlow {
            batch_id: "11".repeat(32),
            shard_id: 7,
            routing_generation: 9,
            route_table_digest: "22".repeat(32),
            items: vec![
                ScopeFlowItem {
                    tx_id: "delete-0000".to_string(),
                    op_kind: ScopeOpKind::Delete,
                    definition_id: "33".repeat(32),
                    serial_id: 1,
                    terminal_id: "44".repeat(32),
                    leaf_value_hash: [0x45; 32],
                    leaf_family: ScopeLeafKind::Terminal,
                    first_seen: ScopeSeen {
                        definition: false,
                        serial: false,
                        object: false,
                    },
                },
                ScopeFlowItem {
                    tx_id: "put-0000".to_string(),
                    op_kind: ScopeOpKind::Put,
                    definition_id: "55".repeat(32),
                    serial_id: 2,
                    terminal_id: "66".repeat(32),
                    leaf_value_hash: [0x67; 32],
                    leaf_family: ScopeLeafKind::Terminal,
                    first_seen: ScopeSeen {
                        definition: true,
                        serial: true,
                        object: true,
                    },
                },
            ],
            root_flow: ScopeRootFlow {
                prev_root: "77".repeat(32),
                post_root: "88".repeat(32),
            },
        };
        let encoded = encode_uniqueness_precommit(&flow).expect("precommit");
        let precommit = decode_uniqueness_precommit(&encoded).expect("strict precommit");
        let grammar = RecursiveTraceOpcodeV2::grammar_digest();
        let challenge = encode_uniqueness_challenge([9; 32], grammar, precommit);
        let decoded = decode_uniqueness_challenge(&challenge, [9; 32], grammar, precommit)
            .expect("challenge");
        assert_eq!(decoded.context, [9; 32]);
        let mut outputs = decoded
            .spent
            .into_iter()
            .chain(decoded.output)
            .collect::<Vec<_>>();
        outputs.sort_unstable();
        outputs.dedup();
        assert_eq!(
            outputs.len(),
            8,
            "every set/pair/coordinate must be distinct"
        );

        let mut wrong_grammar = grammar;
        wrong_grammar[0] ^= 1;
        assert!(
            decode_uniqueness_challenge(&challenge, [9; 32], wrong_grammar, precommit).is_err()
        );

        let mut substituted = encoded;
        substituted[9] ^= 1;
        assert!(decode_uniqueness_precommit(&substituted).is_err());
    }

    #[test]
    fn sorted_identifier_rows_have_one_strict_canonical_codec() {
        let row = super::UniquenessSemanticRowV2 {
            definition_id: [0xA4; 32],
            serial_id: 7,
            terminal_id: [0xA5; 32],
            leaf_value_hash: [0xA6; 32],
        };
        let encoded = encode_uniqueness_sorted_row(
            UniquenessPassV2::Commit,
            UniquenessSetKindV2::Spent,
            UniquenessListKindV2::Sorted,
            row,
        );
        assert_eq!(
            decode_uniqueness_sorted_row(&encoded).expect("canonical sorted row"),
            (
                UniquenessPassV2::Commit,
                UniquenessSetKindV2::Spent,
                UniquenessListKindV2::Sorted,
                row,
            )
        );

        let mut unknown_pass = encoded.clone();
        unknown_pass[1] = 2;
        assert!(decode_uniqueness_sorted_row(&unknown_pass).is_err());
        let mut unknown_set = encoded.clone();
        unknown_set[2] = 2;
        assert!(decode_uniqueness_sorted_row(&unknown_set).is_err());
        let mut unknown_list = encoded.clone();
        unknown_list[3] = 2;
        assert!(decode_uniqueness_sorted_row(&unknown_list).is_err());
        assert!(decode_uniqueness_sorted_row(&encoded[..encoded.len() - 1]).is_err());
    }

    #[test]
    fn net_effect_codec_covers_delete_insert_replace_and_unchanged() {
        let old = UniquenessSemanticRowV2 {
            definition_id: [0x11; 32],
            serial_id: 7,
            terminal_id: [0x22; 32],
            leaf_value_hash: [0x33; 32],
        };
        let mut new = old;
        new.leaf_value_hash = [0x44; 32];
        for (spent, output, expected) in [
            (Some(old), None, NetEffectKindV2::Delete),
            (None, Some(new), NetEffectKindV2::Insert),
            (Some(old), Some(new), NetEffectKindV2::Replace),
            (Some(old), Some(old), NetEffectKindV2::Unchanged),
        ] {
            let effect = NetEffectV2::from_rows(spent, output).expect("canonical Net effect");
            assert_eq!(effect.kind, expected);
            assert_eq!(
                decode_net_effect(&encode_net_effect(effect)).expect("round-trip Net effect"),
                effect,
            );
        }

        let mut changed_path = new;
        changed_path.definition_id[0] ^= 1;
        assert!(NetEffectV2::from_rows(Some(old), Some(changed_path)).is_err());
        let mut false_unchanged =
            encode_net_effect(NetEffectV2::from_rows(Some(old), Some(old)).expect("unchanged"));
        false_unchanged[2 + super::UNIQUENESS_SEMANTIC_ROW_BYTES_V2] ^= 1;
        assert!(decode_net_effect(&false_unchanged).is_err());
    }
}
