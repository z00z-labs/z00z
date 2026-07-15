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
const UNIQUENESS_PRECOMMIT_VERSION_V2: u8 = 1;
const UNIQUENESS_PRECOMMIT_BYTES_V2: usize = 1 + 4 + 4 + 32 * 5;
const UNIQUENESS_CHALLENGE_BYTES_V2: usize = 1 + 32 + 32;
const NET_MERGE_BYTES_V2: usize = 1 + 32;
const HIERARCHY_PROMOTION_BYTES_V2: usize = 1 + 32 + 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalFlowHeaderV2 {
    pub(crate) batch_id: [u8; 32],
    pub(crate) shard_id: u32,
    pub(crate) routing_generation: u64,
    pub(crate) route_table_digest: [u8; 32],
    pub(crate) prev_root: [u8; 32],
    pub(crate) post_root: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalFlowItemV2 {
    pub(crate) tx_id: Vec<u8>,
    pub(crate) op_kind: ScopeOpKind,
    pub(crate) definition_id: [u8; 32],
    pub(crate) serial_id: u32,
    pub(crate) terminal_id: [u8; 32],
    pub(crate) leaf_kind: ScopeLeafKind,
    pub(crate) first_definition: bool,
    pub(crate) first_serial: bool,
    pub(crate) first_object: bool,
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

pub(crate) fn encode_uniqueness_precommit(flow: &ScopeFlow) -> Result<Vec<u8>, RecursiveV2Error> {
    let mut spent = Vec::new();
    let mut output = Vec::new();
    for kind in [ScopeOpKind::Delete, ScopeOpKind::Put] {
        for item in flow.items.iter().filter(|item| item.op_kind == kind) {
            let id = decode_canonical_hex32(&item.terminal_id)?;
            match item.op_kind {
                ScopeOpKind::Delete => spent.push(id),
                ScopeOpKind::Put => output.push(id),
            }
        }
    }
    uniqueness_precommit_from_ids(&spent, &output).and_then(encode_uniqueness_precommit_value)
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
        || (value.spent_count == 0) != (value.output_count == 0)
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

pub(crate) fn uniqueness_precommit_from_ids(
    spent: &[[u8; 32]],
    output: &[[u8; 32]],
) -> Result<UniquenessPrecommitV2, RecursiveV2Error> {
    if spent.is_empty() != output.is_empty() {
        return Err(RecursiveV2Error::Invariant);
    }
    let spent_count = u32::try_from(spent.len()).map_err(|_| RecursiveV2Error::Limit)?;
    let output_count = u32::try_from(output.len()).map_err(|_| RecursiveV2Error::Limit)?;
    let spent_original_digest = digest_ids(CheckpointShaRole::SpentOriginalIds, spent)?;
    let output_original_digest = digest_ids(CheckpointShaRole::OutputOriginalIds, output)?;
    let mut spent_sorted = spent.to_vec();
    let mut output_sorted = output.to_vec();
    spent_sorted.sort_unstable();
    output_sorted.sort_unstable();
    if spent_sorted.windows(2).any(|pair| pair[0] == pair[1])
        || output_sorted.windows(2).any(|pair| pair[0] == pair[1])
        || !are_disjoint_sorted(&spent_sorted, &output_sorted)
    {
        return Err(RecursiveV2Error::DuplicateIdentifier);
    }
    let spent_sorted_digest = digest_ids(CheckpointShaRole::SpentSortedIds, &spent_sorted)?;
    let output_sorted_digest = digest_ids(CheckpointShaRole::OutputSortedIds, &output_sorted)?;
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
    authority_digest: [u8; 32],
    precommit: UniquenessPrecommitV2,
) -> Vec<u8> {
    let challenge = sha256_256_role(
        CheckpointShaRole::IdChallenge,
        &[
            b"z00z.recursive.v2.uniqueness-challenge",
            &authority_digest,
            &precommit.precommit_digest,
        ],
    );
    let mut bytes = Vec::with_capacity(UNIQUENESS_CHALLENGE_BYTES_V2);
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.extend_from_slice(&precommit.precommit_digest);
    bytes.extend_from_slice(&challenge);
    bytes
}

pub(crate) fn decode_uniqueness_challenge(
    bytes: &[u8],
    authority_digest: [u8; 32],
    precommit: UniquenessPrecommitV2,
) -> Result<[u8; 32], RecursiveV2Error> {
    if bytes.len() != UNIQUENESS_CHALLENGE_BYTES_V2 || bytes[0] != UNIQUENESS_PRECOMMIT_VERSION_V2 {
        return Err(RecursiveV2Error::Canonical);
    }
    let committed_precommit: [u8; 32] = bytes[1..33]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let challenge: [u8; 32] = bytes[33..65]
        .try_into()
        .map_err(|_| RecursiveV2Error::Canonical)?;
    let expected = encode_uniqueness_challenge(authority_digest, precommit);
    if committed_precommit != precommit.precommit_digest || expected.as_slice() != bytes {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(challenge)
}

pub(crate) fn encode_net_merge(precommit: UniquenessPrecommitV2, challenge: [u8; 32]) -> Vec<u8> {
    let digest = sha256_256_role(
        CheckpointShaRole::Trace,
        &[
            b"z00z.recursive.v2.net-merge",
            &precommit.precommit_digest,
            &challenge,
        ],
    );
    let mut bytes = Vec::with_capacity(NET_MERGE_BYTES_V2);
    bytes.push(UNIQUENESS_PRECOMMIT_VERSION_V2);
    bytes.extend_from_slice(&digest);
    bytes
}

pub(crate) fn decode_net_merge(
    bytes: &[u8],
    precommit: UniquenessPrecommitV2,
    challenge: [u8; 32],
) -> Result<(), RecursiveV2Error> {
    if bytes != encode_net_merge(precommit, challenge) {
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
    if bytes != encode_hierarchy_promotion(expected_definition_root, expected_update_trace_digest) {
        return Err(RecursiveV2Error::Canonical);
    }
    Ok(())
}

fn digest_ids(role: CheckpointShaRole, ids: &[[u8; 32]]) -> Result<[u8; 32], RecursiveV2Error> {
    let mut digest = CheckpointSha256V2::new(role);
    for id in ids {
        digest.update_part(id)?;
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

fn are_disjoint_sorted(left: &[[u8; 32]], right: &[[u8; 32]]) -> bool {
    let (mut l, mut r) = (0_usize, 0_usize);
    while l < left.len() && r < right.len() {
        match left[l].cmp(&right[r]) {
            std::cmp::Ordering::Less => l += 1,
            std::cmp::Ordering::Greater => r += 1,
            std::cmp::Ordering::Equal => return false,
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
    let mut bytes = Vec::with_capacity(6 * (2 + CANONICAL_HEX32_BYTES) + 12);
    append_string(&mut bytes, &flow.batch_id)?;
    bytes.extend_from_slice(&flow.shard_id.to_le_bytes());
    bytes.extend_from_slice(&flow.routing_generation.to_le_bytes());
    append_string(&mut bytes, &flow.route_table_digest)?;
    append_hex32(&mut bytes, prev_root)?;
    append_hex32(&mut bytes, post_root)?;
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
    use super::{
        decode_canonical_hex32, decode_flow_header, decode_flow_item, decode_uniqueness_challenge,
        decode_uniqueness_precommit, encode_flow_header, encode_flow_item,
        encode_uniqueness_challenge, encode_uniqueness_precommit,
    };
    use crate::settlement::{
        ScopeFlow, ScopeFlowItem, ScopeLeafKind, ScopeOpKind, ScopeRootFlow, ScopeSeen,
    };

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
        let challenge = encode_uniqueness_challenge([9; 32], precommit);
        assert_eq!(
            decode_uniqueness_challenge(&challenge, [9; 32], precommit).expect("challenge"),
            challenge[33..65]
        );

        let mut substituted = encoded;
        substituted[9] ^= 1;
        assert!(decode_uniqueness_precommit(&substituted).is_err());
    }
}
