use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

use jmt::{proof::UpdateMerkleProof, KeyHash, RootHash, SimpleHasher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use z00z_crypto::{
    expert::hash_domain, frame_bytes, hash_zk::hash_zk, CheckpointSha256V2, CheckpointShaRole,
};
use z00z_utils::codec::{BincodeCodec, Codec};

use super::proof::{
    chk_blob_settlement, hjmt_default_child_commitment, hjmt_default_value_commitment,
    HjmtProofFamily, ProofBlob, ProofChkErr, HJMT_DEFAULT_COMMITMENT_VERSION,
};
use super::{
    keys::{definition_key, serial_key},
    BucketPolicy, DefinitionId, RootGeneration, SerialId, SettlementLeaf, SettlementLeafFamily,
    SettlementPath, SettlementStateRoot,
};

hash_domain!(StorBatchProofDom, "z00z.storage.batch.proof", 1);
hash_domain!(StorProofBindDom, "z00z.storage.proof.bind", 1);
hash_domain!(StorPolicySetDom, "z00z.hjmt.policy-set.v1", 1);
hash_domain!(StorShardRootLeafDom, "z00z.hjmt.shard-root-leaf.v1", 1);
hash_domain!(
    StorCheckpointPublicationDom,
    "z00z.hjmt.checkpoint-publication.v1",
    1
);

pub const BATCH_PROOF_ENCODING_VERSION: u8 = 1;
const OPENING_VERSION_V1: u8 = 1;
const PRIOR_CTX_VERSION_V1: u8 = 1;
const DELETION_FACT_VERSION_V1: u8 = 1;
const ROOT_BIND_VER: u8 = 1;
const WITNESS_CHUNK_LABEL: &str = "checkpoint_witness_chunk_v1";
const WITNESS_PAYLOAD_LABEL: &str = "checkpoint_witness_payload_v1";
const WITNESS_ROOT_LABEL: &str = "checkpoint_witness_root_v1";
const WITNESS_CHUNK_VER: u8 = 1;
const WITNESS_CHUNK_BATCH: u8 = 1;
pub const JMT_UPDATE_TRACE_VERSION_V2: u8 = 3;
#[cfg(test)]
pub(crate) const JMT_UPDATE_TRACE_MAX_BYTES_V2: usize = 67_108_864;
pub(crate) const JMT_UPDATE_TRACE_MAX_OPS_V2: usize = 1_000;
const JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2: usize = 24 * 1024 * 1024;
const JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2: usize = 64 * 1024;
const JMT_UPDATE_TRACE_MAX_VALUES_BYTES_V2: usize = 24 * 1024 * 1024;
#[cfg(test)]
pub(crate) const JMT_UPDATE_TRACE_ENVELOPE_MAX_BYTES_V2: usize = 48 * 1024 * 1024;
pub(crate) const JMT_SPARSE_PLACEHOLDER_HASH_V2: [u8; 32] = *b"SPARSE_MERKLE_PLACEHOLDER_HASH__";
const JMT_LEAF_DOMAIN_V2: &[u8] = b"JMT::LeafNode";
const JMT_INTERNAL_DOMAIN_V2: &[u8] = b"JMT::IntrnalNode";
pub(crate) const JMT_UPDATE_TRACE_KIND_MUTATING_V2: u8 = 1;
pub(crate) const JMT_UPDATE_TRACE_KIND_NOOP_V2: u8 = 2;
const JMT_UPDATE_TRACE_NOOP_LABEL_V2: &str = "settlement_update_trace_noop_v2";
pub(crate) const JMT_CIRCUIT_MICRO_OP_VERSION_V2: u8 = 3;
pub(crate) const JMT_CIRCUIT_UPDATE_BEGIN_V2: u8 = 1;
pub(crate) const JMT_CIRCUIT_OPERATION_BEGIN_V2: u8 = 2;
pub(crate) const JMT_CIRCUIT_OPERATION_VALUE_V2: u8 = 3;
pub(crate) const JMT_CIRCUIT_OPERATION_PROOF_V2: u8 = 4;
pub(crate) const JMT_CIRCUIT_OPERATION_END_V2: u8 = 5;
pub(crate) const JMT_CIRCUIT_UPDATE_END_V2: u8 = 6;
pub(crate) const JMT_CIRCUIT_OPERATION_SIBLING_V2: u8 = 7;
pub(crate) const JMT_CIRCUIT_OPERATION_PROOF_END_V2: u8 = 8;
pub(crate) const JMT_CIRCUIT_OPERATION_SPLIT_SIBLING_V2: u8 = 9;
const JMT_CIRCUIT_RAW_BLOCK_BYTES_V2: usize = 64;
// Sibling records carry two raw SHA blocks for the sibling node and two for
// the derived old-parent node: 19 framing bytes plus four 64-byte blocks.
const JMT_CIRCUIT_MAX_RECORD_BYTES_V2: usize = 403;
pub(crate) const JMT_CIRCUIT_HEADER_BYTES_V2: usize = 1 + 1 + 1 + 32 + 4;
static BATCH_PROOF_TRANSCRIPT_DOMAIN: OnceLock<[u8; 32]> = OnceLock::new();

type TerminalRootKeyV2 = ([u8; 32], u32, [u8; 32]);
type BucketParentValueV2 = ([u8; 32], u32, [u8; 32], [u8; 32], [u8; 32]);

/// Serializable JMT hasher marker for the pinned JMT raw SHA-256 primitive.
///
/// JMT encodes the hasher type in serde's generic bounds. The pinned
/// `sha2::Sha256` marker is not serializable, while this project-owned marker
/// preserves byte-identical node hashes. Its state is deliberately excluded
/// from proof transport: JMT serializes only proof data, and each verification
/// creates a fresh raw-SHA state for the pinned JMT node function.
#[derive(Serialize, Deserialize)]
pub(crate) struct JmtSha256V2 {
    #[serde(skip, default)]
    state: Sha256,
}

impl Default for JmtSha256V2 {
    fn default() -> Self {
        Self {
            state: <Sha256 as Digest>::new(),
        }
    }
}

impl SimpleHasher for JmtSha256V2 {
    fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, bytes: &[u8]) {
        <Sha256 as Digest>::update(&mut self.state, bytes);
    }

    fn finalize(self) -> [u8; 32] {
        <Sha256 as Digest>::finalize(self.state).into()
    }
}

/// Storage-owned role of one pinned JMT tree update.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) enum JmtTreeRoleV2 {
    Definition,
    Serial([u8; 32]),
    Bucket([u8; 32], u32),
    Terminal([u8; 32], u32, [u8; 32]),
    PathIndex,
}

impl From<super::tree_id::HjmtTreeId> for JmtTreeRoleV2 {
    fn from(tree: super::tree_id::HjmtTreeId) -> Self {
        match tree {
            super::tree_id::HjmtTreeId::Definition => Self::Definition,
            super::tree_id::HjmtTreeId::Serial(definition_id) => {
                Self::Serial(definition_id.into_bytes())
            }
            super::tree_id::HjmtTreeId::Bucket(definition_id, serial_id) => {
                Self::Bucket(definition_id.into_bytes(), serial_id.get())
            }
            super::tree_id::HjmtTreeId::BucketTerminal(definition_id, serial_id, bucket_id) => {
                Self::Terminal(
                    definition_id.into_bytes(),
                    serial_id.get(),
                    bucket_id.into_bytes(),
                )
            }
            super::tree_id::HjmtTreeId::PathIndex => Self::PathIndex,
        }
    }
}

impl JmtTreeRoleV2 {
    #[cfg(test)]
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        match self {
            Self::Definition => out.push(1),
            Self::Serial(definition_id) => {
                out.push(2);
                out.extend_from_slice(definition_id);
            }
            Self::Bucket(definition_id, serial_id) => {
                out.push(3);
                out.extend_from_slice(definition_id);
                out.extend_from_slice(&serial_id.to_le_bytes());
            }
            Self::Terminal(definition_id, serial_id, terminal_id) => {
                out.push(4);
                out.extend_from_slice(definition_id);
                out.extend_from_slice(&serial_id.to_le_bytes());
                out.extend_from_slice(terminal_id);
            }
            Self::PathIndex => out.push(5),
        }
    }

    #[cfg(test)]
    fn decode_canonical(reader: &mut CanonicalReader<'_>) -> Result<Self, ProofChkErr> {
        match reader.take_u8()? {
            1 => Ok(Self::Definition),
            2 => Ok(Self::Serial(reader.take_array()?)),
            3 => Ok(Self::Bucket(reader.take_array()?, reader.take_u32()?)),
            4 => Ok(Self::Terminal(
                reader.take_array()?,
                reader.take_u32()?,
                reader.take_array()?,
            )),
            5 => Ok(Self::PathIndex),
            _ => Err(ProofChkErr::JmtUpdateTraceCanonical),
        }
    }

    /// Fixed-width circuit encoding. Every unused coordinate is zero so a
    /// role cannot borrow bytes from a wider hierarchy variant.
    fn encode_circuit_canonical(&self, out: &mut Vec<u8>) {
        let (tag, definition_id, serial_id, terminal_id) = match self {
            Self::Definition => (1, [0; 32], 0, [0; 32]),
            Self::Serial(definition_id) => (2, *definition_id, 0, [0; 32]),
            Self::Bucket(definition_id, serial_id) => (3, *definition_id, *serial_id, [0; 32]),
            Self::Terminal(definition_id, serial_id, terminal_id) => {
                (4, *definition_id, *serial_id, *terminal_id)
            }
            Self::PathIndex => (5, [0; 32], 0, [0; 32]),
        };
        out.push(tag);
        out.extend_from_slice(&definition_id);
        out.extend_from_slice(&serial_id.to_le_bytes());
        out.extend_from_slice(&terminal_id);
    }

    fn decode_circuit_canonical(reader: &mut CanonicalReader<'_>) -> Result<Self, ProofChkErr> {
        let tag = reader.take_u8()?;
        let definition_id = reader.take_array()?;
        let serial_id = reader.take_u32()?;
        let terminal_id = reader.take_array()?;
        match tag {
            1 if definition_id == [0; 32] && serial_id == 0 && terminal_id == [0; 32] => {
                Ok(Self::Definition)
            }
            2 if serial_id == 0 && terminal_id == [0; 32] => Ok(Self::Serial(definition_id)),
            3 if terminal_id == [0; 32] => Ok(Self::Bucket(definition_id, serial_id)),
            4 => Ok(Self::Terminal(definition_id, serial_id, terminal_id)),
            5 if definition_id == [0; 32] && serial_id == 0 && terminal_id == [0; 32] => {
                Ok(Self::PathIndex)
            }
            _ => Err(ProofChkErr::JmtUpdateTraceCanonical),
        }
    }
}

/// One ordered key/value operation bound into an update trace.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JmtUpdateOpV2 {
    key: [u8; 32],
    prior_value: Option<Vec<u8>>,
    value: Option<Vec<u8>>,
}

/// Project-owned classification of one upstream update-proof transition.
///
/// It is decoded independently from the opaque pinned-JMT wire and never
/// crosses the storage facade.  Keeping the classification here lets the V2
/// predicate reject a proof whose case algebra does not agree with its typed
/// key/value operation before it relies on the upstream root verifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JmtMutationCaseV2 {
    EmptyInsert,
    ExistingUpdate,
    SplitInsert { common_prefix_bits: u16 },
    DeleteToEmpty,
    DeletePreserveInternal,
    DeleteCoalesceLeaf,
}

/// Exact serde mirror of the pinned `jmt 0.12` update-proof wire.
///
/// The upstream fields are intentionally private, so this mirror is the sole
/// project-owned read-only decoder for its already version-pinned bincode
/// witness.  It is checked by canonical re-encoding and then paired with the
/// unchanged upstream `verify_update`; it is not a second update executor.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct JmtUpdateProofWireV2(Vec<JmtSparseProofWireV2>);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct JmtSparseProofWireV2 {
    leaf: Option<JmtLeafWireV2>,
    siblings: Vec<JmtSiblingWireV2>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct JmtLeafWireV2 {
    key_hash: [u8; 32],
    value_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct JmtInternalWireV2 {
    left_child: [u8; 32],
    right_child: [u8; 32],
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum JmtSiblingWireV2 {
    Null,
    Internal(JmtInternalWireV2),
    Leaf(JmtLeafWireV2),
}

impl JmtUpdateOpV2 {
    fn from_live(
        (key, prior_value, value): (KeyHash, Option<Vec<u8>>, Option<Vec<u8>>),
    ) -> Result<Self, ProofChkErr> {
        if [prior_value.as_ref(), value.as_ref()]
            .into_iter()
            .flatten()
            .any(|value| value.len() > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2)
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        Ok(Self {
            key: key.0,
            prior_value,
            value,
        })
    }

    fn into_live(self) -> (KeyHash, Option<Vec<u8>>) {
        (KeyHash(self.key), self.value)
    }

    #[cfg(test)]
    fn encode_canonical(&self, out: &mut Vec<u8>) -> Result<(), ProofChkErr> {
        out.extend_from_slice(&self.key);
        match &self.prior_value {
            None => out.push(0),
            Some(value) => {
                if value.len() > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2 {
                    return Err(ProofChkErr::JmtUpdateTraceLimit);
                }
                out.push(1);
                append_len_prefixed(out, value)?;
            }
        }
        match &self.value {
            None => out.push(0),
            Some(value) => {
                if value.len() > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2 {
                    return Err(ProofChkErr::JmtUpdateTraceLimit);
                }
                out.push(1);
                append_len_prefixed(out, value)?;
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn decode_canonical(reader: &mut CanonicalReader<'_>) -> Result<Self, ProofChkErr> {
        let key = reader.take_array()?;
        let prior_value = match reader.take_u8()? {
            0 => None,
            1 => Some(reader.take_vec(JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2)?),
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let value = match reader.take_u8()? {
            0 => None,
            1 => Some(reader.take_vec(JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2)?),
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        Ok(Self {
            key,
            prior_value,
            value,
        })
    }

    /// Test-only observability for the internal JMT canonicalization tests.
    ///
    /// Production code intentionally cannot recover raw replay keys from the
    /// opaque V2 witness boundary.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn key(&self) -> [u8; 32] {
        self.key
    }
}

/// Strict transport wire for one update proof emitted by the pinned JMT owner.
///
/// The upstream proof remains an opaque dependency payload at this facade;
/// operations, roots, versions, and tree role are project-owned typed fields.
/// The circuit-facing trace decodes the same wire later and independently
/// constrains its cases instead of trusting this native verification result.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JmtUpdateTraceV2 {
    version: u8,
    tree: JmtTreeRoleV2,
    old_version: u64,
    new_version: u64,
    old_root: [u8; 32],
    new_root: [u8; 32],
    operations: Vec<JmtUpdateOpV2>,
    proof_wire: Vec<u8>,
}

impl JmtUpdateTraceV2 {
    fn terminal_operation_count(&self) -> Result<u64, ProofChkErr> {
        if !matches!(self.tree, JmtTreeRoleV2::Terminal(_, _, _)) {
            return Ok(0);
        }
        u64::try_from(self.operations.len()).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)
    }

    pub(crate) fn from_update(
        tree: JmtTreeRoleV2,
        old_version: u64,
        new_version: u64,
        old_root: RootHash,
        new_root: RootHash,
        operations: Vec<(KeyHash, Option<Vec<u8>>)>,
        prior_values: Vec<Option<Vec<u8>>>,
        proof: UpdateMerkleProof<JmtSha256V2>,
    ) -> Result<Self, ProofChkErr> {
        validate_live_jmt_operations_v2(&operations)?;
        if !jmt_version_pair_is_canonical(old_version, new_version) {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let proof_wire = BincodeCodec.serialize(&proof)?;
        if proof_wire.len() > JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }

        if prior_values.len() != operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let out = Self {
            version: JMT_UPDATE_TRACE_VERSION_V2,
            tree,
            old_version,
            new_version,
            old_root: old_root.0,
            new_root: new_root.0,
            operations: operations
                .into_iter()
                .zip(prior_values)
                .map(|((key, value), prior_value)| (key, prior_value, value))
                .map(JmtUpdateOpV2::from_live)
                .collect::<Result<Vec<_>, _>>()?,
            proof_wire,
        };
        out.verify_semantics()?;
        out.verify_native()?;
        Ok(out)
    }

    #[cfg(test)]
    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        if self.version != JMT_UPDATE_TRACE_VERSION_V2
            || !jmt_version_pair_is_canonical(self.old_version, self.new_version)
            || self.operations.len() > JMT_UPDATE_TRACE_MAX_OPS_V2
            || self.proof_wire.len() > JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        check_jmt_operations(&self.operations)?;
        let capacity = 1_usize
            .checked_add(1 + 32 + 4 + 32)
            .and_then(|value| value.checked_add(8 * 2 + 32 * 2 + 4))
            .and_then(|value| value.checked_add(self.proof_wire.len() + 4))
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        bytes.push(self.version);
        self.tree.encode_canonical(&mut bytes);
        bytes.extend_from_slice(&self.old_version.to_le_bytes());
        bytes.extend_from_slice(&self.new_version.to_le_bytes());
        bytes.extend_from_slice(&self.old_root);
        bytes.extend_from_slice(&self.new_root);
        let operations =
            u32::try_from(self.operations.len()).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        bytes.extend_from_slice(&operations.to_le_bytes());
        for operation in &self.operations {
            operation.encode_canonical(&mut bytes)?;
        }
        append_len_prefixed(&mut bytes, &self.proof_wire)?;
        if bytes.len() > JMT_UPDATE_TRACE_MAX_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        Ok(bytes)
    }

    #[cfg(test)]
    pub(crate) fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        if bytes.len() > JMT_UPDATE_TRACE_MAX_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let mut reader = CanonicalReader::new(bytes);
        let version = reader.take_u8()?;
        let tree = JmtTreeRoleV2::decode_canonical(&mut reader)?;
        let old_version = reader.take_u64()?;
        let new_version = reader.take_u64()?;
        let old_root = reader.take_array()?;
        let new_root = reader.take_array()?;
        let count =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        if count == 0 || count > JMT_UPDATE_TRACE_MAX_OPS_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let mut operations = Vec::new();
        operations
            .try_reserve_exact(count)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        for _ in 0..count {
            operations.push(JmtUpdateOpV2::decode_canonical(&mut reader)?);
        }
        let proof_wire = reader.take_vec(JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2)?;
        reader.finish()?;
        let out = Self {
            version,
            tree,
            old_version,
            new_version,
            old_root,
            new_root,
            operations,
            proof_wire,
        };
        if out.version != JMT_UPDATE_TRACE_VERSION_V2 {
            return Err(ProofChkErr::UnsupportedJmtUpdateVersion);
        }
        if out.operations.len() > JMT_UPDATE_TRACE_MAX_OPS_V2
            || out.proof_wire.len() > JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2
            || out.canonical_bytes()? != bytes
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        out.verify_semantics()?;
        out.verify_native()?;
        Ok(out)
    }

    pub(crate) fn verify_native(&self) -> Result<(), ProofChkErr> {
        if self.version != JMT_UPDATE_TRACE_VERSION_V2
            || !jmt_version_pair_is_canonical(self.old_version, self.new_version)
            || self.operations.len() > JMT_UPDATE_TRACE_MAX_OPS_V2
            || self.proof_wire.len() > JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        check_jmt_operations(&self.operations)?;
        let proof: UpdateMerkleProof<JmtSha256V2> = BincodeCodec
            .deserialize_bounded(&self.proof_wire, JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2 as u64)?;
        let operations = self
            .operations
            .clone()
            .into_iter()
            .map(JmtUpdateOpV2::into_live)
            .collect::<Vec<_>>();
        proof
            .verify_update(RootHash(self.old_root), RootHash(self.new_root), operations)
            .map_err(|_| ProofChkErr::JmtUpdateProofMix)
    }

    /// Independently execute each pinned-JMT proof against typed V2 data.
    ///
    /// The project-owned mirror recomputes every old root, update/split path,
    /// and delete-coalescing result with the pinned raw-SHA node algebra. The
    /// upstream verifier remains a second, corroborating check and is never
    /// the only semantic executor for this witness.
    fn verify_semantics(&self) -> Result<(), ProofChkErr> {
        let (_cases, computed_root) = self.semantic_cases_and_root()?;
        if computed_root != self.new_root {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        Ok(())
    }

    fn semantic_cases_and_root(&self) -> Result<(Vec<JmtMutationCaseV2>, [u8; 32]), ProofChkErr> {
        let proof: JmtUpdateProofWireV2 = BincodeCodec
            .deserialize_bounded(&self.proof_wire, JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2 as u64)?;
        if BincodeCodec.serialize(&proof)? != self.proof_wire
            || proof.0.len() != self.operations.len()
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut cases = Vec::new();
        cases
            .try_reserve_exact(proof.0.len())
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        let mut current_root = self.old_root;
        for (proof, operation) in proof.0.iter().zip(&self.operations) {
            let (case, next_root) =
                verify_jmt_transition_semantics(proof, operation, current_root)?;
            cases.push(case);
            current_root = next_root;
        }
        Ok((cases, current_root))
    }

    /// Test-only observability for native JMT trace invariants.  These
    /// accessors are deliberately absent from production builds, where the
    /// trace must remain an opaque circuit witness.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn old_root(&self) -> [u8; 32] {
        self.old_root
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn new_root(&self) -> [u8; 32] {
        self.new_root
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn operations(&self) -> &[JmtUpdateOpV2] {
        &self.operations
    }

    #[cfg(test)]
    pub(crate) fn semantic_cases_for_test(&self) -> Result<Vec<JmtMutationCaseV2>, ProofChkErr> {
        self.semantic_cases_and_root().map(|(cases, _)| cases)
    }
}

fn verify_jmt_transition_semantics(
    proof: &JmtSparseProofWireV2,
    operation: &JmtUpdateOpV2,
    expected_old_root: [u8; 32],
) -> Result<(JmtMutationCaseV2, [u8; 32]), ProofChkErr> {
    if proof.siblings.len() > 256 {
        return Err(ProofChkErr::JmtUpdateTraceLimit);
    }

    match (operation.prior_value.as_deref(), proof.leaf) {
        (Some(prior), Some(leaf))
            if leaf.key_hash == operation.key && leaf.value_hash == jmt_value_hash(prior) => {}
        (None, None) => {}
        (None, Some(leaf)) if leaf.key_hash != operation.key => {}
        _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
    }

    let old_path_key = proof.leaf.map_or(operation.key, |leaf| leaf.key_hash);
    let old_leaf_hash = proof
        .leaf
        .map_or(JMT_SPARSE_PLACEHOLDER_HASH_V2, jmt_leaf_hash);
    if jmt_root_from_path(old_leaf_hash, &old_path_key, &proof.siblings)? != expected_old_root {
        return Err(ProofChkErr::JmtUpdateProofMix);
    }

    match (&operation.value, proof.leaf) {
        (Some(value), None) => Ok((
            JmtMutationCaseV2::EmptyInsert,
            jmt_root_from_path(
                jmt_leaf_hash(JmtLeafWireV2 {
                    key_hash: operation.key,
                    value_hash: jmt_value_hash(value),
                }),
                &operation.key,
                &proof.siblings,
            )?,
        )),
        (Some(value), Some(leaf)) if leaf.key_hash == operation.key => Ok((
            JmtMutationCaseV2::ExistingUpdate,
            jmt_root_from_path(
                jmt_leaf_hash(JmtLeafWireV2 {
                    key_hash: operation.key,
                    value_hash: jmt_value_hash(value),
                }),
                &operation.key,
                &proof.siblings,
            )?,
        )),
        (Some(value), Some(leaf)) => {
            let common_prefix_bits = common_prefix_bits(&leaf.key_hash, &operation.key);
            if common_prefix_bits < proof.siblings.len() {
                return Err(ProofChkErr::JmtUpdateTraceCanonical);
            }
            let split_siblings = jmt_split_siblings(proof, common_prefix_bits)?;
            Ok((
                JmtMutationCaseV2::SplitInsert {
                    common_prefix_bits: u16::try_from(common_prefix_bits)
                        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?,
                },
                jmt_root_from_path(
                    jmt_leaf_hash(JmtLeafWireV2 {
                        key_hash: operation.key,
                        value_hash: jmt_value_hash(value),
                    }),
                    &operation.key,
                    &split_siblings,
                )?,
            ))
        }
        (None, Some(leaf)) if leaf.key_hash == operation.key => {
            let first_non_default = proof
                .siblings
                .iter()
                .position(|sibling| !matches!(sibling, JmtSiblingWireV2::Null));
            match first_non_default {
                None => Ok((
                    JmtMutationCaseV2::DeleteToEmpty,
                    JMT_SPARSE_PLACEHOLDER_HASH_V2,
                )),
                Some(index) if matches!(proof.siblings[index], JmtSiblingWireV2::Internal(_)) => {
                    Ok((
                        JmtMutationCaseV2::DeletePreserveInternal,
                        jmt_root_from_path(
                            JMT_SPARSE_PLACEHOLDER_HASH_V2,
                            &operation.key,
                            &proof.siblings[index..],
                        )?,
                    ))
                }
                Some(index) if matches!(proof.siblings[index], JmtSiblingWireV2::Leaf(_)) => {
                    let leaf_hash = jmt_sibling_hash(&proof.siblings[index]);
                    let mut tail = index
                        .checked_add(1)
                        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
                    while matches!(proof.siblings.get(tail), Some(JmtSiblingWireV2::Null)) {
                        tail = tail
                            .checked_add(1)
                            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
                    }
                    Ok((
                        JmtMutationCaseV2::DeleteCoalesceLeaf,
                        jmt_root_from_path(leaf_hash, &operation.key, &proof.siblings[tail..])?,
                    ))
                }
                Some(_) => Err(ProofChkErr::JmtUpdateTraceCanonical),
            }
        }
        (None, None) | (None, Some(_)) => Err(ProofChkErr::JmtUpdateTraceCanonical),
    }
}

fn common_prefix_bits(left: &[u8; 32], right: &[u8; 32]) -> usize {
    let mut prefix = 0_usize;
    for (left, right) in left.iter().zip(right) {
        let diff = left ^ right;
        if diff == 0 {
            prefix = prefix.checked_add(8).expect("32-byte prefix is bounded");
        } else {
            return prefix + usize::try_from(diff.leading_zeros()).expect("u32 fits usize");
        }
    }
    prefix
}

fn jmt_split_siblings(
    proof: &JmtSparseProofWireV2,
    common_prefix_bits: usize,
) -> Result<Vec<JmtSiblingWireV2>, ProofChkErr> {
    let former_leaf = proof.leaf.ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
    let sibling_count = proof.siblings.len();
    let common_prefix_nibbles = common_prefix_bits / 4;
    let next_nibble_bits = common_prefix_bits % 4;
    let prefix_span = common_prefix_nibbles
        .checked_add(1)
        .and_then(|value| value.checked_mul(4))
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
    let default_leaves_to_add = prefix_span
        .checked_sub(sibling_count)
        .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?
        / 4
        * 4;
    let default_siblings_prev_root = (4 - sibling_count % 4) % 4;
    let default_siblings = default_siblings_prev_root
        .checked_add(default_leaves_to_add)
        .and_then(|value| value.checked_add(next_nibble_bits))
        .and_then(|value| value.checked_sub(4))
        .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
    let capacity = 1_usize
        .checked_add(default_siblings)
        .and_then(|value| value.checked_add(sibling_count))
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
    let mut siblings = Vec::new();
    siblings
        .try_reserve_exact(capacity)
        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
    siblings.push(JmtSiblingWireV2::Leaf(former_leaf));
    siblings.resize(1 + default_siblings, JmtSiblingWireV2::Null);
    siblings.extend_from_slice(&proof.siblings);
    Ok(siblings)
}

fn jmt_root_from_path(
    mut current_hash: [u8; 32],
    key: &[u8; 32],
    siblings: &[JmtSiblingWireV2],
) -> Result<[u8; 32], ProofChkErr> {
    if siblings.len() > 256 {
        return Err(ProofChkErr::JmtUpdateTraceLimit);
    }
    let sibling_count = siblings.len();
    for (index, sibling) in siblings.iter().enumerate() {
        let bit_index = sibling_count
            .checked_sub(index + 1)
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let bit = (key[bit_index / 8] >> (7 - bit_index % 8)) & 1 != 0;
        let sibling_hash = jmt_sibling_hash(sibling);
        current_hash = if bit {
            jmt_internal_hash(sibling_hash, current_hash)
        } else {
            jmt_internal_hash(current_hash, sibling_hash)
        };
    }
    Ok(current_hash)
}

fn jmt_sibling_hash(sibling: &JmtSiblingWireV2) -> [u8; 32] {
    match sibling {
        JmtSiblingWireV2::Null => JMT_SPARSE_PLACEHOLDER_HASH_V2,
        JmtSiblingWireV2::Internal(node) => jmt_internal_hash(node.left_child, node.right_child),
        JmtSiblingWireV2::Leaf(node) => jmt_leaf_hash(*node),
    }
}

fn jmt_leaf_hash(leaf: JmtLeafWireV2) -> [u8; 32] {
    jmt_hash(&[JMT_LEAF_DOMAIN_V2, &leaf.key_hash, &leaf.value_hash])
}

fn jmt_value_hash(value: &[u8]) -> [u8; 32] {
    jmt_hash(&[value])
}

fn jmt_internal_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    jmt_hash(&[JMT_INTERNAL_DOMAIN_V2, &left, &right])
}

fn jmt_hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = JmtSha256V2::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize()
}

fn jmt_raw_sha_block_count(message_bytes: usize) -> Result<usize, ProofChkErr> {
    message_bytes
        .checked_add(9)
        .map(|bytes| bytes.div_ceil(64))
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)
}

fn jmt_raw_sha_block(
    message: &[u8],
    block_index: usize,
    block_count: usize,
) -> Result<[u8; 64], ProofChkErr> {
    if block_count != jmt_raw_sha_block_count(message.len())? || block_index >= block_count {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    let bit_length = u64::try_from(message.len())
        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
        .checked_mul(8)
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
    let start = block_index
        .checked_mul(64)
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
    let mut block = [0_u8; 64];
    for (offset, byte) in block.iter_mut().enumerate() {
        let absolute = start
            .checked_add(offset)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        *byte = if let Some(byte) = message.get(absolute) {
            *byte
        } else if absolute == message.len() {
            0x80
        } else if block_index + 1 == block_count && offset >= 56 {
            bit_length.to_be_bytes()[offset - 56]
        } else {
            0
        };
    }
    Ok(block)
}

fn validate_jmt_raw_sha_value_block(
    block: &[u8; 64],
    message_bytes: usize,
    block_index: usize,
    block_count: usize,
) -> Result<(), ProofChkErr> {
    if block_count != jmt_raw_sha_block_count(message_bytes)? || block_index >= block_count {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    let bit_length = u64::try_from(message_bytes)
        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
        .checked_mul(8)
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
    let start = block_index
        .checked_mul(64)
        .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
    for (offset, byte) in block.iter().enumerate() {
        let absolute = start
            .checked_add(offset)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        if absolute < message_bytes {
            continue;
        }
        let expected = if absolute == message_bytes {
            0x80
        } else if block_index + 1 == block_count && offset >= 56 {
            bit_length.to_be_bytes()[offset - 56]
        } else {
            0
        };
        if *byte != expected {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
    }
    Ok(())
}

fn jmt_leaf_raw_blocks(leaf: JmtLeafWireV2) -> Result<[[u8; 64]; 2], ProofChkErr> {
    let mut message = [0_u8; 77];
    message[..JMT_LEAF_DOMAIN_V2.len()].copy_from_slice(JMT_LEAF_DOMAIN_V2);
    message[13..45].copy_from_slice(&leaf.key_hash);
    message[45..77].copy_from_slice(&leaf.value_hash);
    Ok([
        jmt_raw_sha_block(&message, 0, 2)?,
        jmt_raw_sha_block(&message, 1, 2)?,
    ])
}

fn jmt_internal_raw_blocks(node: JmtInternalWireV2) -> Result<[[u8; 64]; 2], ProofChkErr> {
    let mut message = [0_u8; 80];
    message[..JMT_INTERNAL_DOMAIN_V2.len()].copy_from_slice(JMT_INTERNAL_DOMAIN_V2);
    message[16..48].copy_from_slice(&node.left_child);
    message[48..80].copy_from_slice(&node.right_child);
    Ok([
        jmt_raw_sha_block(&message, 0, 2)?,
        jmt_raw_sha_block(&message, 1, 2)?,
    ])
}

fn decode_jmt_leaf_raw_blocks(blocks: [[u8; 64]; 2]) -> Result<JmtLeafWireV2, ProofChkErr> {
    let mut key_hash = [0_u8; 32];
    key_hash.copy_from_slice(&blocks[0][13..45]);
    let mut value_hash = [0_u8; 32];
    value_hash[..19].copy_from_slice(&blocks[0][45..64]);
    value_hash[19..].copy_from_slice(&blocks[1][..13]);
    let leaf = JmtLeafWireV2 {
        key_hash,
        value_hash,
    };
    if jmt_leaf_raw_blocks(leaf)? != blocks {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    Ok(leaf)
}

fn decode_jmt_internal_raw_blocks(blocks: [[u8; 64]; 2]) -> Result<JmtInternalWireV2, ProofChkErr> {
    let mut left_child = [0_u8; 32];
    left_child.copy_from_slice(&blocks[0][16..48]);
    let mut right_child = [0_u8; 32];
    right_child[..16].copy_from_slice(&blocks[0][48..64]);
    right_child[16..].copy_from_slice(&blocks[1][..16]);
    let node = JmtInternalWireV2 {
        left_child,
        right_child,
    };
    if jmt_internal_raw_blocks(node)? != blocks {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    Ok(node)
}

fn jmt_version_pair_is_canonical(old_version: u64, new_version: u64) -> bool {
    (old_version == 0 && new_version == 0)
        || old_version
            .checked_add(1)
            .is_some_and(|expected| expected == new_version)
}

/// Reject a non-canonical or oversized JMT mutation before it reaches JMT.
///
/// `HjmtStore` sorts its caller supplied operations and calls this function
/// immediately before `put_value_set_with_proof`; `JmtUpdateTraceV2` repeats
/// the check at its transport boundary.  Keeping the predicate here makes the
/// pre-mutation and proof-transport limits one canonical policy.
pub(crate) fn validate_live_jmt_operations_v2(
    operations: &[(KeyHash, Option<Vec<u8>>)],
) -> Result<(), ProofChkErr> {
    if operations.is_empty()
        || operations.len() > JMT_UPDATE_TRACE_MAX_OPS_V2
        || operations
            .windows(2)
            .any(|pair| pair[0].0 .0 >= pair[1].0 .0)
    {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }

    operations.iter().try_fold(0_usize, |total, (_, value)| {
        let next = total
            .checked_add(value.as_ref().map_or(0, Vec::len))
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        if value
            .as_ref()
            .is_some_and(|value| value.len() > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2)
            || next > JMT_UPDATE_TRACE_MAX_VALUES_BYTES_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        Ok(next)
    })?;
    Ok(())
}

fn check_jmt_operations(operations: &[JmtUpdateOpV2]) -> Result<(), ProofChkErr> {
    if operations.is_empty() || operations.windows(2).any(|pair| pair[0].key >= pair[1].key) {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    operations.iter().try_fold(0_usize, |total, operation| {
        let next = total
            .checked_add(operation.value.as_ref().map_or(0, Vec::len))
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        if operation
            .value
            .as_ref()
            .is_some_and(|value| value.len() > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2)
            || next > JMT_UPDATE_TRACE_MAX_VALUES_BYTES_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        Ok(next)
    })?;
    Ok(())
}

/// One frozen storage envelope for all traced JMT updates of one V2 transition.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SettlementUpdateTraceEnvelopeV2 {
    version: u8,
    root_generation: u8,
    kind: u8,
    trace_digest: [u8; 32],
    updates: Vec<JmtUpdateTraceV2>,
}

impl SettlementUpdateTraceEnvelopeV2 {
    pub(crate) fn new(
        root_generation: RootGeneration,
        updates: Vec<JmtUpdateTraceV2>,
    ) -> Result<Self, ProofChkErr> {
        if root_generation != RootGeneration::SettlementV2 || updates.is_empty() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        for update in &updates {
            update.verify_native()?;
        }
        let mut envelope = Self {
            version: JMT_UPDATE_TRACE_VERSION_V2,
            root_generation: root_generation.version(),
            kind: JMT_UPDATE_TRACE_KIND_MUTATING_V2,
            trace_digest: [0_u8; 32],
            updates,
        };
        envelope.trace_digest = envelope.circuit_trace_digest()?;
        Ok(envelope)
    }

    /// Build the explicit zero-update envelope used only by the
    /// authority-defined recursive V2 no-op transition.
    pub(crate) fn new_noop(root_generation: RootGeneration) -> Result<Self, ProofChkErr> {
        if root_generation != RootGeneration::SettlementV2 {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let envelope = Self {
            version: JMT_UPDATE_TRACE_VERSION_V2,
            root_generation: root_generation.version(),
            kind: JMT_UPDATE_TRACE_KIND_NOOP_V2,
            trace_digest: noop_update_trace_digest(),
            updates: Vec::new(),
        };
        Ok(envelope)
    }

    #[cfg(test)]
    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        if self.version != JMT_UPDATE_TRACE_VERSION_V2
            || self.root_generation != RootGeneration::SettlementV2.version()
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        match self.kind {
            JMT_UPDATE_TRACE_KIND_MUTATING_V2
                if !self.updates.is_empty()
                    && self.trace_digest == self.circuit_trace_digest()? => {}
            JMT_UPDATE_TRACE_KIND_NOOP_V2
                if self.updates.is_empty() && self.trace_digest == noop_update_trace_digest() => {}
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        }
        let mut total = 1_usize
            .checked_add(1 + 1 + 32 + 4)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        let mut encoded_updates = Vec::new();
        encoded_updates
            .try_reserve_exact(self.updates.len())
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        for update in &self.updates {
            let encoded = update.canonical_bytes()?;
            total = total
                .checked_add(4)
                .and_then(|value| value.checked_add(encoded.len()))
                .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
            if total > JMT_UPDATE_TRACE_ENVELOPE_MAX_BYTES_V2 {
                return Err(ProofChkErr::JmtUpdateTraceLimit);
            }
            encoded_updates.push(encoded);
        }
        let count =
            u32::try_from(encoded_updates.len()).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(total)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        bytes.push(self.version);
        bytes.push(self.root_generation);
        bytes.push(self.kind);
        bytes.extend_from_slice(&self.trace_digest);
        bytes.extend_from_slice(&count.to_le_bytes());
        for update in encoded_updates {
            append_len_prefixed(&mut bytes, &update)?;
        }
        if bytes.len() > JMT_UPDATE_TRACE_ENVELOPE_MAX_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        Ok(bytes)
    }

    #[cfg(test)]
    pub(crate) fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        if bytes.len() > JMT_UPDATE_TRACE_ENVELOPE_MAX_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let mut reader = CanonicalReader::new(bytes);
        let version = reader.take_u8()?;
        let root_generation = reader.take_u8()?;
        let kind = reader.take_u8()?;
        let trace_digest = reader.take_array()?;
        let count =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        if count > JMT_UPDATE_TRACE_MAX_OPS_V2
            || (kind == JMT_UPDATE_TRACE_KIND_MUTATING_V2 && count == 0)
            || (kind == JMT_UPDATE_TRACE_KIND_NOOP_V2 && count != 0)
            || !matches!(
                kind,
                JMT_UPDATE_TRACE_KIND_MUTATING_V2 | JMT_UPDATE_TRACE_KIND_NOOP_V2
            )
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let mut updates = Vec::new();
        updates
            .try_reserve_exact(count)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        for _ in 0..count {
            let encoded = reader.take_borrowed(JMT_UPDATE_TRACE_MAX_BYTES_V2)?;
            updates.push(JmtUpdateTraceV2::from_canon(encoded)?);
        }
        reader.finish()?;
        let envelope = Self {
            version,
            root_generation,
            kind,
            trace_digest,
            updates,
        };
        if envelope.canonical_bytes()? != bytes {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        for update in &envelope.updates {
            update.verify_native()?;
        }
        Ok(envelope)
    }

    #[must_use]
    pub(crate) const fn trace_digest(&self) -> [u8; 32] {
        self.trace_digest
    }

    #[must_use]
    pub(crate) fn updates(&self) -> &[JmtUpdateTraceV2] {
        &self.updates
    }

    pub(crate) fn terminal_operation_count(&self) -> Result<u64, ProofChkErr> {
        self.updates.iter().try_fold(0_u64, |count, update| {
            count
                .checked_add(update.terminal_operation_count()?)
                .ok_or(ProofChkErr::JmtUpdateTraceLimit)
        })
    }

    #[must_use]
    pub(crate) const fn is_noop(&self) -> bool {
        self.kind == JMT_UPDATE_TRACE_KIND_NOOP_V2
    }

    /// Fixed canonical envelope header used by the recursive source. The
    /// update body has exactly one live representation: circuit micro-ops.
    pub(crate) fn circuit_header_bytes(
        &self,
    ) -> Result<[u8; JMT_CIRCUIT_HEADER_BYTES_V2], ProofChkErr> {
        match self.kind {
            JMT_UPDATE_TRACE_KIND_MUTATING_V2
                if !self.updates.is_empty()
                    && self.trace_digest == self.circuit_trace_digest()? => {}
            JMT_UPDATE_TRACE_KIND_NOOP_V2
                if self.updates.is_empty() && self.trace_digest == noop_update_trace_digest() => {}
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        }
        if self.version != JMT_UPDATE_TRACE_VERSION_V2
            || self.root_generation != RootGeneration::SettlementV2.version()
            || self.updates.len() > JMT_UPDATE_TRACE_MAX_OPS_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut header = [0_u8; JMT_CIRCUIT_HEADER_BYTES_V2];
        header[0] = self.version;
        header[1] = self.root_generation;
        header[2] = self.kind;
        header[3..35].copy_from_slice(&self.trace_digest);
        header[35..39].copy_from_slice(
            &u32::try_from(self.updates.len())
                .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                .to_le_bytes(),
        );
        Ok(header)
    }

    fn circuit_trace_digest(&self) -> Result<[u8; 32], ProofChkErr> {
        let mut digest = CheckpointSha256V2::new(CheckpointShaRole::Trace);
        self.visit_circuit_micro_operations(|record| {
            digest
                .update_part(record)
                .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)
        })?;
        Ok(digest.finalize())
    }

    /// Visit the canonical circuit-consumable JMT micro-operation transcript.
    ///
    /// The opaque pinned-JMT bincode proof is decoded once into the exact
    /// project-owned mirror.  Each callback receives one bounded record and
    /// the record is dropped before the next one is built; callers never need
    /// an update-sized event tape.  These bytes are the sole preimage of the
    /// mutating envelope's `trace_digest` and expose every operation, old leaf,
    /// and sibling needed by the subsequent in-circuit raw-SHA path machine.
    pub(crate) fn visit_circuit_micro_operations(
        &self,
        mut visit: impl FnMut(&[u8]) -> Result<(), ProofChkErr>,
    ) -> Result<(), ProofChkErr> {
        if self.is_noop() {
            return if self.updates.is_empty() {
                Ok(())
            } else {
                Err(ProofChkErr::JmtUpdateTraceCanonical)
            };
        }
        if self.kind != JMT_UPDATE_TRACE_KIND_MUTATING_V2 || self.updates.is_empty() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }

        for (update_index, update) in self.updates.iter().enumerate() {
            let update_index =
                u32::try_from(update_index).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
            let proof: JmtUpdateProofWireV2 = BincodeCodec.deserialize_bounded(
                &update.proof_wire,
                JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2 as u64,
            )?;
            if BincodeCodec.serialize(&proof)? != update.proof_wire
                || proof.0.len() != update.operations.len()
            {
                return Err(ProofChkErr::JmtUpdateTraceCanonical);
            }

            let mut record = Vec::with_capacity(1 + 1 + 4 + 69 + 8 * 2 + 32 * 2 + 4);
            record.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
            record.push(JMT_CIRCUIT_UPDATE_BEGIN_V2);
            record.extend_from_slice(&update_index.to_le_bytes());
            update.tree.encode_circuit_canonical(&mut record);
            record.extend_from_slice(&update.old_version.to_le_bytes());
            record.extend_from_slice(&update.new_version.to_le_bytes());
            record.extend_from_slice(&update.old_root);
            record.extend_from_slice(&update.new_root);
            record.extend_from_slice(
                &u32::try_from(update.operations.len())
                    .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                    .to_le_bytes(),
            );
            visit(&record)?;

            let mut update_current_root = update.old_root;
            for (operation_index, (operation, sparse_proof)) in
                update.operations.iter().zip(&proof.0).enumerate()
            {
                let (mutation_case, expected_next_root) =
                    verify_jmt_transition_semantics(sparse_proof, operation, update_current_root)?;
                let operation_index =
                    u32::try_from(operation_index).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
                let mut begin = Vec::with_capacity(1 + 1 + 4 + 4 + 32 + 1 + 4 + 1 + 4);
                begin.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                begin.push(JMT_CIRCUIT_OPERATION_BEGIN_V2);
                begin.extend_from_slice(&update_index.to_le_bytes());
                begin.extend_from_slice(&operation_index.to_le_bytes());
                begin.extend_from_slice(&operation.key);
                begin.push(u8::from(operation.value.is_some()));
                begin.extend_from_slice(
                    &u32::try_from(operation.value.as_ref().map_or(0, Vec::len))
                        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                        .to_le_bytes(),
                );
                begin.push(u8::from(operation.prior_value.is_some()));
                begin.extend_from_slice(
                    &u32::try_from(operation.prior_value.as_ref().map_or(0, Vec::len))
                        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                        .to_le_bytes(),
                );
                visit(&begin)?;

                // Prior and replacement values share one record grammar. The
                // kind byte is canonical: prior blocks (1) precede new-value
                // blocks (0), so the streaming decoder needs no second tape.
                for (value_kind, candidate) in [
                    (1_u8, operation.prior_value.as_ref()),
                    (0_u8, operation.value.as_ref()),
                ] {
                    let Some(value) = candidate else {
                        continue;
                    };
                    let block_count = jmt_raw_sha_block_count(value.len())?;
                    for block_index in 0..block_count {
                        let mut value_record = Vec::with_capacity(19 + 64);
                        value_record.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                        value_record.push(JMT_CIRCUIT_OPERATION_VALUE_V2);
                        value_record.extend_from_slice(&update_index.to_le_bytes());
                        value_record.extend_from_slice(&operation_index.to_le_bytes());
                        value_record.extend_from_slice(
                            &u32::try_from(block_index)
                                .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                                .to_le_bytes(),
                        );
                        value_record.extend_from_slice(
                            &u32::try_from(block_count)
                                .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                                .to_le_bytes(),
                        );
                        value_record.push(value_kind);
                        value_record.extend_from_slice(&jmt_raw_sha_block(
                            value,
                            block_index,
                            block_count,
                        )?);
                        visit(&value_record)?;
                    }
                }

                // The proof is one fixed-width header followed by exactly one
                // fixed-width record per sibling.  This is the sole proof
                // representation in the recursive source and lets the R1CS
                // path machine consume O(1) state without an opaque arena.
                let mut proof_record = Vec::with_capacity(19 + 128);
                proof_record.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                proof_record.push(JMT_CIRCUIT_OPERATION_PROOF_V2);
                proof_record.extend_from_slice(&update_index.to_le_bytes());
                proof_record.extend_from_slice(&operation_index.to_le_bytes());
                proof_record.push(u8::from(sparse_proof.leaf.is_some()));
                proof_record.extend_from_slice(
                    &u16::try_from(sparse_proof.siblings.len())
                        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                        .to_le_bytes(),
                );
                let (case_tag, common_prefix_bits) = match mutation_case {
                    JmtMutationCaseV2::EmptyInsert => (1_u8, 0_u16),
                    JmtMutationCaseV2::ExistingUpdate => (2, 0),
                    JmtMutationCaseV2::SplitInsert { common_prefix_bits } => {
                        (3, common_prefix_bits)
                    }
                    JmtMutationCaseV2::DeleteToEmpty => (4, 0),
                    JmtMutationCaseV2::DeletePreserveInternal => (5, 0),
                    JmtMutationCaseV2::DeleteCoalesceLeaf => (6, 0),
                };
                let split_siblings =
                    if matches!(mutation_case, JmtMutationCaseV2::SplitInsert { .. }) {
                        jmt_split_siblings(sparse_proof, usize::from(common_prefix_bits))?
                    } else {
                        sparse_proof.siblings.clone()
                    };
                let split_prelude_count = split_siblings
                    .len()
                    .checked_sub(sparse_proof.siblings.len())
                    .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
                proof_record.push(case_tag);
                proof_record.extend_from_slice(
                    &u16::try_from(split_prelude_count)
                        .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                        .to_le_bytes(),
                );
                proof_record.extend_from_slice(&[0; 3]);
                match sparse_proof.leaf {
                    None => proof_record.extend_from_slice(&[0; 128]),
                    Some(leaf) => {
                        for block in jmt_leaf_raw_blocks(leaf)? {
                            proof_record.extend_from_slice(&block);
                        }
                    }
                }
                let mut new_current = if let Some(value) = operation.value.as_ref() {
                    let new_leaf = JmtLeafWireV2 {
                        key_hash: operation.key,
                        value_hash: jmt_value_hash(value),
                    };
                    for block in jmt_leaf_raw_blocks(new_leaf)? {
                        proof_record.extend_from_slice(&block);
                    }
                    jmt_leaf_hash(new_leaf)
                } else {
                    proof_record.extend_from_slice(&[0; 128]);
                    JMT_SPARSE_PLACEHOLDER_HASH_V2
                };
                visit(&proof_record)?;

                for (split_index, sibling) in
                    split_siblings.iter().take(split_prelude_count).enumerate()
                {
                    let bit_index = split_siblings
                        .len()
                        .checked_sub(split_index + 1)
                        .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
                    let direction = (operation.key[bit_index / 8] >> (7 - bit_index % 8)) & 1;
                    let sibling_hash = jmt_sibling_hash(sibling);
                    let parent = if direction == 0 {
                        JmtInternalWireV2 {
                            left_child: new_current,
                            right_child: sibling_hash,
                        }
                    } else {
                        JmtInternalWireV2 {
                            left_child: sibling_hash,
                            right_child: new_current,
                        }
                    };
                    let mut split_record = Vec::with_capacity(19 + 256);
                    split_record.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                    split_record.push(JMT_CIRCUIT_OPERATION_SPLIT_SIBLING_V2);
                    split_record.extend_from_slice(&update_index.to_le_bytes());
                    split_record.extend_from_slice(&operation_index.to_le_bytes());
                    split_record.extend_from_slice(
                        &u16::try_from(split_index)
                            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                            .to_le_bytes(),
                    );
                    let (sibling_type, sibling_blocks) = match sibling {
                        JmtSiblingWireV2::Null => (0_u8, [[0; 64]; 2]),
                        JmtSiblingWireV2::Leaf(node) => (2, jmt_leaf_raw_blocks(*node)?),
                        JmtSiblingWireV2::Internal(_) => {
                            return Err(ProofChkErr::JmtUpdateTraceCanonical)
                        }
                    };
                    split_record.push(sibling_type);
                    split_record.push(direction);
                    split_record.extend_from_slice(&[0; 5]);
                    for block in sibling_blocks {
                        split_record.extend_from_slice(&block);
                    }
                    for block in jmt_internal_raw_blocks(parent)? {
                        split_record.extend_from_slice(&block);
                    }
                    visit(&split_record)?;
                    new_current = jmt_internal_hash(parent.left_child, parent.right_child);
                }

                let mut old_current = sparse_proof
                    .leaf
                    .map_or(JMT_SPARSE_PLACEHOLDER_HASH_V2, jmt_leaf_hash);
                for (sibling_index, sibling) in sparse_proof.siblings.iter().enumerate() {
                    let mut sibling_record = Vec::with_capacity(19 + 256);
                    sibling_record.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                    sibling_record.push(JMT_CIRCUIT_OPERATION_SIBLING_V2);
                    sibling_record.extend_from_slice(&update_index.to_le_bytes());
                    sibling_record.extend_from_slice(&operation_index.to_le_bytes());
                    sibling_record.extend_from_slice(
                        &u16::try_from(sibling_index)
                            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?
                            .to_le_bytes(),
                    );
                    let (sibling_type, sibling_blocks) = match sibling {
                        JmtSiblingWireV2::Null => (0, [[0; 64]; 2]),
                        JmtSiblingWireV2::Internal(node) => (1, jmt_internal_raw_blocks(*node)?),
                        JmtSiblingWireV2::Leaf(node) => (2, jmt_leaf_raw_blocks(*node)?),
                    };
                    let bit_index = sparse_proof
                        .siblings
                        .len()
                        .checked_sub(sibling_index + 1)
                        .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
                    let old_path_key = sparse_proof
                        .leaf
                        .map_or(operation.key, |leaf| leaf.key_hash);
                    let direction = (old_path_key[bit_index / 8] >> (7 - bit_index % 8)) & 1;
                    let sibling_hash = jmt_sibling_hash(sibling);
                    let parent = if direction == 0 {
                        JmtInternalWireV2 {
                            left_child: old_current,
                            right_child: sibling_hash,
                        }
                    } else {
                        JmtInternalWireV2 {
                            left_child: sibling_hash,
                            right_child: old_current,
                        }
                    };
                    sibling_record.push(sibling_type);
                    let first_non_default = sparse_proof
                        .siblings
                        .iter()
                        .position(|candidate| !matches!(candidate, JmtSiblingWireV2::Null));
                    let coalesce_tail = first_non_default.and_then(|first| {
                        if !matches!(sparse_proof.siblings[first], JmtSiblingWireV2::Leaf(_)) {
                            return None;
                        }
                        let mut tail = first + 1;
                        while matches!(
                            sparse_proof.siblings.get(tail),
                            Some(JmtSiblingWireV2::Null)
                        ) {
                            tail += 1;
                        }
                        Some(tail)
                    });
                    let new_parent_active = match mutation_case {
                        JmtMutationCaseV2::EmptyInsert | JmtMutationCaseV2::ExistingUpdate => true,
                        JmtMutationCaseV2::SplitInsert { .. } => true,
                        JmtMutationCaseV2::DeleteToEmpty => false,
                        JmtMutationCaseV2::DeletePreserveInternal => {
                            first_non_default.is_some_and(|first| sibling_index >= first)
                        }
                        JmtMutationCaseV2::DeleteCoalesceLeaf => {
                            coalesce_tail.is_some_and(|tail| sibling_index >= tail)
                        }
                    };
                    if matches!(mutation_case, JmtMutationCaseV2::DeleteCoalesceLeaf)
                        && first_non_default == Some(sibling_index)
                    {
                        new_current = sibling_hash;
                    }
                    let new_parent = if new_parent_active {
                        Some(if direction == 0 {
                            JmtInternalWireV2 {
                                left_child: new_current,
                                right_child: sibling_hash,
                            }
                        } else {
                            JmtInternalWireV2 {
                                left_child: sibling_hash,
                                right_child: new_current,
                            }
                        })
                    } else {
                        None
                    };
                    sibling_record.push(direction);
                    sibling_record.push(u8::from(new_parent_active));
                    sibling_record.extend_from_slice(&[0; 4]);
                    for block in sibling_blocks {
                        sibling_record.extend_from_slice(&block);
                    }
                    for block in jmt_internal_raw_blocks(parent)? {
                        sibling_record.extend_from_slice(&block);
                    }
                    match new_parent {
                        Some(parent) => {
                            for block in jmt_internal_raw_blocks(parent)? {
                                sibling_record.extend_from_slice(&block);
                            }
                            new_current = jmt_internal_hash(parent.left_child, parent.right_child);
                        }
                        None => sibling_record.extend_from_slice(&[0; 128]),
                    }
                    old_current = jmt_internal_hash(parent.left_child, parent.right_child);
                    visit(&sibling_record)?;
                }

                let mut proof_end = Vec::with_capacity(1 + 1 + 4 + 4);
                proof_end.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                proof_end.push(JMT_CIRCUIT_OPERATION_PROOF_END_V2);
                proof_end.extend_from_slice(&update_index.to_le_bytes());
                proof_end.extend_from_slice(&operation_index.to_le_bytes());
                visit(&proof_end)?;

                if new_current != expected_next_root {
                    return Err(ProofChkErr::JmtUpdateTraceCanonical);
                }
                update_current_root = expected_next_root;

                let mut end = Vec::with_capacity(1 + 1 + 4 + 4);
                end.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
                end.push(JMT_CIRCUIT_OPERATION_END_V2);
                end.extend_from_slice(&update_index.to_le_bytes());
                end.extend_from_slice(&operation_index.to_le_bytes());
                visit(&end)?;
            }

            let mut end = Vec::with_capacity(1 + 1 + 4);
            end.push(JMT_CIRCUIT_MICRO_OP_VERSION_V2);
            end.push(JMT_CIRCUIT_UPDATE_END_V2);
            end.extend_from_slice(&update_index.to_le_bytes());
            visit(&end)?;
        }
        Ok(())
    }

    #[must_use]
    pub(crate) fn is_noop_digest(trace_digest: [u8; 32]) -> bool {
        trace_digest == noop_update_trace_digest()
    }

    /// Independently execute the typed HJMT parent/child promotion relation.
    ///
    /// JMT proof verification establishes each tree transition. This second
    /// machine proves that every changed parent leaf names the exact committed
    /// child-tree root, in the frozen terminal → bucket → serial → definition
    /// order, and that the final definition update is the storage-owned root
    /// exposed to the recursive relation. It never calls the HJMT executor.
    pub(crate) fn verify_hierarchy_semantics(
        &self,
        expected_definition_root: [u8; 32],
    ) -> Result<([u8; 32], [u8; 32]), ProofChkErr> {
        if self.is_noop() {
            return if self.updates.is_empty() && self.trace_digest == noop_update_trace_digest() {
                Ok((expected_definition_root, expected_definition_root))
            } else {
                Err(ProofChkErr::JmtUpdateTraceCanonical)
            };
        }
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        enum Stage {
            Terminal,
            Bucket,
            Serial,
            Definition,
            PathIndex,
        }

        let mut stage = Stage::Terminal;
        let mut terminal_roots = BTreeMap::<TerminalRootKeyV2, RootTransitionV2>::new();
        let mut bucket_roots = BTreeMap::<([u8; 32], u32), RootTransitionV2>::new();
        let mut serial_roots = BTreeMap::<[u8; 32], RootTransitionV2>::new();
        let mut used_terminal_roots = BTreeSet::new();
        let mut used_bucket_roots = BTreeSet::new();
        let mut used_serial_roots = BTreeSet::new();
        let mut definition_transition = None;

        for update in &self.updates {
            match &update.tree {
                JmtTreeRoleV2::Terminal(definition_id, serial_id, bucket_id) => {
                    if stage != Stage::Terminal
                        || update.old_root == update.new_root
                        || terminal_roots
                            .insert(
                                (*definition_id, *serial_id, *bucket_id),
                                RootTransitionV2::new(update.old_root, update.new_root),
                            )
                            .is_some()
                    {
                        return Err(ProofChkErr::JmtUpdateTraceCanonical);
                    }
                }
                JmtTreeRoleV2::Bucket(definition_id, serial_id) => {
                    if stage > Stage::Bucket
                        || update.old_root == update.new_root
                        || bucket_roots
                            .insert(
                                (*definition_id, *serial_id),
                                RootTransitionV2::new(update.old_root, update.new_root),
                            )
                            .is_some()
                    {
                        return Err(ProofChkErr::JmtUpdateTraceCanonical);
                    }
                    stage = Stage::Bucket;
                    for operation in &update.operations {
                        let terminal_key = verify_bucket_promotion(
                            *definition_id,
                            *serial_id,
                            operation,
                            &terminal_roots,
                        )?;
                        if !used_terminal_roots.insert(terminal_key) {
                            return Err(ProofChkErr::JmtUpdateTraceCanonical);
                        }
                    }
                }
                JmtTreeRoleV2::Serial(definition_id) => {
                    if stage > Stage::Serial
                        || update.old_root == update.new_root
                        || serial_roots
                            .insert(
                                *definition_id,
                                RootTransitionV2::new(update.old_root, update.new_root),
                            )
                            .is_some()
                    {
                        return Err(ProofChkErr::JmtUpdateTraceCanonical);
                    }
                    stage = Stage::Serial;
                    for operation in &update.operations {
                        let bucket_key =
                            verify_serial_promotion(*definition_id, operation, &bucket_roots)?;
                        if !used_bucket_roots.insert(bucket_key) {
                            return Err(ProofChkErr::JmtUpdateTraceCanonical);
                        }
                    }
                }
                JmtTreeRoleV2::Definition => {
                    if stage > Stage::Definition
                        || definition_transition.is_some()
                        || update.old_root == update.new_root
                    {
                        return Err(ProofChkErr::JmtUpdateTraceCanonical);
                    }
                    stage = Stage::Definition;
                    definition_transition = Some((update.old_root, update.new_root));
                    for operation in &update.operations {
                        let definition_id = verify_definition_promotion(operation, &serial_roots)?;
                        if !used_serial_roots.insert(definition_id) {
                            return Err(ProofChkErr::JmtUpdateTraceCanonical);
                        }
                    }
                    if update.new_root != expected_definition_root {
                        return Err(ProofChkErr::JmtUpdateTraceCanonical);
                    }
                }
                JmtTreeRoleV2::PathIndex => {
                    if stage < Stage::Definition {
                        return Err(ProofChkErr::JmtUpdateTraceCanonical);
                    }
                    stage = Stage::PathIndex;
                }
            }
        }

        let Some(definition_transition) = definition_transition else {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        };
        if stage < Stage::Definition
            || terminal_roots.len() != used_terminal_roots.len()
            || bucket_roots.len() != used_bucket_roots.len()
            || serial_roots.len() != used_serial_roots.len()
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        Ok(definition_transition)
    }
}

/// Streaming inverse of [`SettlementUpdateTraceEnvelopeV2::visit_circuit_micro_operations`].
///
/// The recursive trace carries one fixed header plus this transcript; the
/// former opaque envelope body is never copied into the source spool. The
/// decoder retains only the typed update currently being assembled and the
/// already completed typed updates needed by hierarchy verification.
pub(crate) struct SettlementUpdateTraceCircuitDecoderV2 {
    kind: u8,
    expected_digest: [u8; 32],
    expected_updates: usize,
    updates: Vec<JmtUpdateTraceV2>,
    current_update: Option<CircuitUpdateBuilderV2>,
}

struct CircuitUpdateBuilderV2 {
    tree: JmtTreeRoleV2,
    old_version: u64,
    new_version: u64,
    old_root: [u8; 32],
    new_root: [u8; 32],
    current_root: [u8; 32],
    expected_operations: usize,
    operations: Vec<JmtUpdateOpV2>,
    proofs: Vec<JmtSparseProofWireV2>,
    current_operation: Option<CircuitOperationBuilderV2>,
}

struct CircuitOperationBuilderV2 {
    key: [u8; 32],
    prior_value_present: bool,
    expected_prior_value_bytes: usize,
    prior_value: Vec<u8>,
    next_prior_value_chunk: usize,
    value_present: bool,
    expected_value_bytes: usize,
    value: Vec<u8>,
    next_value_chunk: usize,
    proof_leaf: Option<Option<JmtLeafWireV2>>,
    expected_siblings: Option<usize>,
    siblings: Vec<JmtSiblingWireV2>,
    proof_closed: bool,
    old_current_hash: Option<[u8; 32]>,
    mutation_case: Option<u8>,
    expected_split_siblings: usize,
    split_siblings: Vec<JmtSiblingWireV2>,
    new_current_hash: Option<[u8; 32]>,
    new_parent_started: bool,
    coalesced_leaf_seen: bool,
}

impl SettlementUpdateTraceCircuitDecoderV2 {
    pub(crate) fn new(header: &[u8]) -> Result<Self, ProofChkErr> {
        if header.len() != JMT_CIRCUIT_HEADER_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut reader = CanonicalReader::new(header);
        let version = reader.take_u8()?;
        let root_generation = reader.take_u8()?;
        let kind = reader.take_u8()?;
        let expected_digest = reader.take_array()?;
        let expected_updates =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        reader.finish()?;
        if version != JMT_UPDATE_TRACE_VERSION_V2
            || root_generation != RootGeneration::SettlementV2.version()
            || expected_updates > JMT_UPDATE_TRACE_MAX_OPS_V2
            || !matches!(
                kind,
                JMT_UPDATE_TRACE_KIND_MUTATING_V2 | JMT_UPDATE_TRACE_KIND_NOOP_V2
            )
            || (kind == JMT_UPDATE_TRACE_KIND_MUTATING_V2 && expected_updates == 0)
            || (kind == JMT_UPDATE_TRACE_KIND_NOOP_V2
                && (expected_updates != 0 || expected_digest != noop_update_trace_digest()))
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut updates = Vec::new();
        updates
            .try_reserve_exact(expected_updates)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        Ok(Self {
            kind,
            expected_digest,
            expected_updates,
            updates,
            current_update: None,
        })
    }

    pub(crate) fn accept(&mut self, record: &[u8]) -> Result<(), ProofChkErr> {
        if self.kind != JMT_UPDATE_TRACE_KIND_MUTATING_V2
            || record.len() > JMT_CIRCUIT_MAX_RECORD_BYTES_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut reader = CanonicalReader::new(record);
        if reader.take_u8()? != JMT_CIRCUIT_MICRO_OP_VERSION_V2 {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        match reader.take_u8()? {
            JMT_CIRCUIT_UPDATE_BEGIN_V2 => self.accept_update_begin(&mut reader),
            JMT_CIRCUIT_OPERATION_BEGIN_V2 => self.accept_operation_begin(&mut reader),
            JMT_CIRCUIT_OPERATION_VALUE_V2 => self.accept_operation_value(&mut reader),
            JMT_CIRCUIT_OPERATION_PROOF_V2 => self.accept_operation_proof(&mut reader),
            JMT_CIRCUIT_OPERATION_END_V2 => self.accept_operation_end(&mut reader),
            JMT_CIRCUIT_UPDATE_END_V2 => self.accept_update_end(&mut reader),
            JMT_CIRCUIT_OPERATION_SIBLING_V2 => self.accept_operation_sibling(&mut reader),
            JMT_CIRCUIT_OPERATION_PROOF_END_V2 => self.accept_operation_proof_end(&mut reader),
            JMT_CIRCUIT_OPERATION_SPLIT_SIBLING_V2 => {
                self.accept_operation_split_sibling(&mut reader)
            }
            _ => Err(ProofChkErr::JmtUpdateTraceCanonical),
        }
    }

    fn accept_update_begin(&mut self, reader: &mut CanonicalReader<'_>) -> Result<(), ProofChkErr> {
        let update_index =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        if self.current_update.is_some()
            || update_index != self.updates.len()
            || update_index >= self.expected_updates
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let tree = JmtTreeRoleV2::decode_circuit_canonical(reader)?;
        let old_version = reader.take_u64()?;
        let new_version = reader.take_u64()?;
        let old_root = reader.take_array()?;
        let new_root = reader.take_array()?;
        let expected_operations =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        reader.finish()?;
        if !jmt_version_pair_is_canonical(old_version, new_version)
            || expected_operations == 0
            || expected_operations > JMT_UPDATE_TRACE_MAX_OPS_V2
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut operations = Vec::new();
        let mut proofs = Vec::new();
        operations
            .try_reserve_exact(expected_operations)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        proofs
            .try_reserve_exact(expected_operations)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        self.current_update = Some(CircuitUpdateBuilderV2 {
            tree,
            old_version,
            new_version,
            old_root,
            new_root,
            current_root: old_root,
            expected_operations,
            operations,
            proofs,
            current_operation: None,
        });
        Ok(())
    }

    fn accept_operation_begin(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len()
            || operation_index != update.operations.len()
            || operation_index >= update.expected_operations
            || update.current_operation.is_some()
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let key = reader.take_array()?;
        let value_present = match reader.take_u8()? {
            0 => false,
            1 => true,
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let expected_value_bytes =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        let prior_value_present = match reader.take_u8()? {
            0 => false,
            1 => true,
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let expected_prior_value_bytes =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        reader.finish()?;
        if expected_value_bytes > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2
            || (!value_present && expected_value_bytes != 0)
            || expected_prior_value_bytes > JMT_UPDATE_TRACE_MAX_VALUE_BYTES_V2
            || (!prior_value_present && expected_prior_value_bytes != 0)
        {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let mut prior_value = Vec::new();
        prior_value
            .try_reserve_exact(expected_prior_value_bytes)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        let mut value = Vec::new();
        value
            .try_reserve_exact(expected_value_bytes)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        update.current_operation = Some(CircuitOperationBuilderV2 {
            key,
            prior_value_present,
            expected_prior_value_bytes,
            prior_value,
            next_prior_value_chunk: 0,
            value_present,
            expected_value_bytes,
            value,
            next_value_chunk: 0,
            proof_leaf: None,
            expected_siblings: None,
            siblings: Vec::new(),
            proof_closed: false,
            old_current_hash: None,
            mutation_case: None,
            expected_split_siblings: 0,
            split_siblings: Vec::new(),
            new_current_hash: None,
            new_parent_started: false,
            coalesced_leaf_seen: false,
        });
        Ok(())
    }

    fn accept_operation_value(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len() || operation_index != update.operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let operation = update
            .current_operation
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let block_index =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        let block_count =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        let value_kind = reader.take_u8()?;
        let block: [u8; 64] = reader
            .take_exact(64)?
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?;
        let (present, expected_bytes, next_chunk, target) = match value_kind {
            1 if operation.value.is_empty() && operation.next_value_chunk == 0 => (
                operation.prior_value_present,
                operation.expected_prior_value_bytes,
                &mut operation.next_prior_value_chunk,
                &mut operation.prior_value,
            ),
            0 if operation.prior_value.len() == operation.expected_prior_value_bytes
                && operation.next_prior_value_chunk
                    == if operation.prior_value_present {
                        jmt_raw_sha_block_count(operation.expected_prior_value_bytes)?
                    } else {
                        0
                    } =>
            {
                (
                    operation.value_present,
                    operation.expected_value_bytes,
                    &mut operation.next_value_chunk,
                    &mut operation.value,
                )
            }
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let expected_block_count = jmt_raw_sha_block_count(expected_bytes)?;
        let offset = block_index
            .checked_mul(JMT_CIRCUIT_RAW_BLOCK_BYTES_V2)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        let meaningful = expected_bytes
            .saturating_sub(offset)
            .min(JMT_CIRCUIT_RAW_BLOCK_BYTES_V2);
        if !present
            || operation.proof_leaf.is_some()
            || block_index != *next_chunk
            || block_count != expected_block_count
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        validate_jmt_raw_sha_value_block(&block, expected_bytes, block_index, block_count)?;
        reader.finish()?;
        target.extend_from_slice(&block[..meaningful]);
        *next_chunk = next_chunk
            .checked_add(1)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        Ok(())
    }

    fn accept_operation_proof(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len() || operation_index != update.operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let operation = update
            .current_operation
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if operation.proof_leaf.is_some()
            || operation.proof_closed
            || operation.prior_value.len() != operation.expected_prior_value_bytes
            || operation.next_prior_value_chunk
                != if operation.prior_value_present {
                    jmt_raw_sha_block_count(operation.expected_prior_value_bytes)?
                } else {
                    0
                }
            || operation.value.len() != operation.expected_value_bytes
            || operation.next_value_chunk
                != if operation.value_present {
                    jmt_raw_sha_block_count(operation.expected_value_bytes)?
                } else {
                    0
                }
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let leaf_present = reader.take_u8()?;
        let sibling_count = usize::from(reader.take_u16()?);
        if sibling_count > 256 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let mutation_case = reader.take_u8()?;
        let split_sibling_count = usize::from(reader.take_u16()?);
        if reader.take_exact(3)? != [0; 3] || !(1..=6).contains(&mutation_case) {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let blocks: [[u8; 64]; 4] = [
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ];
        let leaf = match leaf_present {
            0 if blocks[..2] == [[0; 64]; 2] => None,
            1 => Some(decode_jmt_leaf_raw_blocks([blocks[0], blocks[1]])?),
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let new_current_hash = if operation.value_present {
            let new_leaf = decode_jmt_leaf_raw_blocks([blocks[2], blocks[3]])?;
            if new_leaf.key_hash != operation.key
                || new_leaf.value_hash != jmt_value_hash(&operation.value)
            {
                return Err(ProofChkErr::JmtUpdateTraceCanonical);
            }
            jmt_leaf_hash(new_leaf)
        } else if blocks[2..] == [[0; 64]; 2] {
            JMT_SPARSE_PLACEHOLDER_HASH_V2
        } else {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        };
        let case_shape = match mutation_case {
            1 => operation.value_present && leaf.is_none() && split_sibling_count == 0,
            2 => {
                operation.value_present
                    && leaf.is_some_and(|old| old.key_hash == operation.key)
                    && split_sibling_count == 0
            }
            3 => {
                operation.value_present
                    && leaf.is_some_and(|old| old.key_hash != operation.key)
                    && split_sibling_count > 0
            }
            4..=6 => {
                !operation.value_present
                    && leaf.is_some_and(|old| old.key_hash == operation.key)
                    && split_sibling_count == 0
            }
            _ => false,
        };
        if !case_shape {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let mut siblings = Vec::new();
        siblings
            .try_reserve_exact(sibling_count)
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        reader.finish()?;
        operation.proof_leaf = Some(leaf);
        operation.expected_siblings = Some(sibling_count);
        operation.siblings = siblings;
        operation.old_current_hash =
            Some(leaf.map_or(JMT_SPARSE_PLACEHOLDER_HASH_V2, jmt_leaf_hash));
        operation.mutation_case = Some(mutation_case);
        operation.expected_split_siblings = split_sibling_count;
        operation.new_current_hash = Some(new_current_hash);
        operation.new_parent_started = matches!(mutation_case, 1..=3);
        Ok(())
    }

    fn accept_operation_sibling(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len() || operation_index != update.operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let operation = update
            .current_operation
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let sibling_index = usize::from(reader.take_u16()?);
        let expected_siblings = operation
            .expected_siblings
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if operation.proof_closed
            || sibling_index != operation.siblings.len()
            || sibling_index >= expected_siblings
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let sibling_type = reader.take_u8()?;
        let direction = reader.take_u8()?;
        let new_parent_active = reader.take_u8()?;
        if direction > 1 || new_parent_active > 1 || reader.take_exact(4)? != [0; 4] {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let blocks: [[u8; 64]; 6] = [
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ];
        let sibling = match sibling_type {
            0 if blocks[..2] == [[0; 64]; 2] => JmtSiblingWireV2::Null,
            1 => {
                JmtSiblingWireV2::Internal(decode_jmt_internal_raw_blocks([blocks[0], blocks[1]])?)
            }
            2 => JmtSiblingWireV2::Leaf(decode_jmt_leaf_raw_blocks([blocks[0], blocks[1]])?),
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let sibling_count = operation
            .expected_siblings
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let bit_index = sibling_count
            .checked_sub(sibling_index + 1)
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let old_path_key = operation
            .proof_leaf
            .flatten()
            .map_or(operation.key, |leaf| leaf.key_hash);
        let expected_direction = (old_path_key[bit_index / 8] >> (7 - bit_index % 8)) & 1;
        if direction != expected_direction {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let old_current = operation
            .old_current_hash
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let sibling_hash = jmt_sibling_hash(&sibling);
        let parent = decode_jmt_internal_raw_blocks([blocks[2], blocks[3]])?;
        let expected_parent = if direction == 0 {
            JmtInternalWireV2 {
                left_child: old_current,
                right_child: sibling_hash,
            }
        } else {
            JmtInternalWireV2 {
                left_child: sibling_hash,
                right_child: old_current,
            }
        };
        if parent != expected_parent {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        operation.old_current_hash = Some(jmt_internal_hash(parent.left_child, parent.right_child));
        let case = operation
            .mutation_case
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let expected_new_active = match case {
            1..=3 => true,
            4 => false,
            5 => {
                if !operation.new_parent_started {
                    match &sibling {
                        JmtSiblingWireV2::Null => false,
                        JmtSiblingWireV2::Internal(_) => {
                            operation.new_parent_started = true;
                            true
                        }
                        JmtSiblingWireV2::Leaf(_) => {
                            return Err(ProofChkErr::JmtUpdateTraceCanonical)
                        }
                    }
                } else {
                    true
                }
            }
            6 => {
                if operation.new_parent_started {
                    true
                } else if !operation.coalesced_leaf_seen {
                    match &sibling {
                        JmtSiblingWireV2::Null => false,
                        JmtSiblingWireV2::Leaf(_) => {
                            operation.coalesced_leaf_seen = true;
                            operation.new_current_hash = Some(sibling_hash);
                            false
                        }
                        JmtSiblingWireV2::Internal(_) => {
                            return Err(ProofChkErr::JmtUpdateTraceCanonical)
                        }
                    }
                } else if matches!(sibling, JmtSiblingWireV2::Null) {
                    false
                } else {
                    operation.new_parent_started = true;
                    true
                }
            }
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        if new_parent_active != u8::from(expected_new_active) {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        if expected_new_active {
            let current = operation
                .new_current_hash
                .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
            let expected_new_parent = if direction == 0 {
                JmtInternalWireV2 {
                    left_child: current,
                    right_child: sibling_hash,
                }
            } else {
                JmtInternalWireV2 {
                    left_child: sibling_hash,
                    right_child: current,
                }
            };
            if decode_jmt_internal_raw_blocks([blocks[4], blocks[5]])? != expected_new_parent {
                return Err(ProofChkErr::JmtUpdateTraceCanonical);
            }
            operation.new_current_hash = Some(jmt_internal_hash(
                expected_new_parent.left_child,
                expected_new_parent.right_child,
            ));
        } else if blocks[4..] != [[0; 64]; 2] {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        reader.finish()?;
        operation.siblings.push(sibling);
        Ok(())
    }

    fn accept_operation_split_sibling(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len() || operation_index != update.operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let operation = update
            .current_operation
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let split_index = usize::from(reader.take_u16()?);
        let sibling_type = reader.take_u8()?;
        let direction = reader.take_u8()?;
        if operation.mutation_case != Some(3)
            || split_index != operation.split_siblings.len()
            || split_index >= operation.expected_split_siblings
            || direction > 1
            || reader.take_exact(5)? != [0; 5]
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let blocks: [[u8; 64]; 4] = [
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
            reader
                .take_exact(64)?
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ];
        let sibling = match sibling_type {
            0 if blocks[..2] == [[0; 64]; 2] => JmtSiblingWireV2::Null,
            2 => JmtSiblingWireV2::Leaf(decode_jmt_leaf_raw_blocks([blocks[0], blocks[1]])?),
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        };
        let former_leaf = operation
            .proof_leaf
            .flatten()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if (split_index == 0 && sibling != JmtSiblingWireV2::Leaf(former_leaf))
            || (split_index > 0 && sibling != JmtSiblingWireV2::Null)
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let total_siblings = operation
            .expected_split_siblings
            .checked_add(
                operation
                    .expected_siblings
                    .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?,
            )
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        let bit_index = total_siblings
            .checked_sub(split_index + 1)
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let expected_direction = (operation.key[bit_index / 8] >> (7 - bit_index % 8)) & 1;
        let current = operation
            .new_current_hash
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        let sibling_hash = jmt_sibling_hash(&sibling);
        let expected_parent = if expected_direction == 0 {
            JmtInternalWireV2 {
                left_child: current,
                right_child: sibling_hash,
            }
        } else {
            JmtInternalWireV2 {
                left_child: sibling_hash,
                right_child: current,
            }
        };
        if direction != expected_direction
            || decode_jmt_internal_raw_blocks([blocks[2], blocks[3]])? != expected_parent
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        operation.new_current_hash = Some(jmt_internal_hash(
            expected_parent.left_child,
            expected_parent.right_child,
        ));
        operation.split_siblings.push(sibling);
        reader.finish()
    }

    fn accept_operation_proof_end(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        reader.finish()?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len() || operation_index != update.operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let operation = update
            .current_operation
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if operation.proof_closed
            || operation.proof_leaf.is_none()
            || operation.expected_siblings != Some(operation.siblings.len())
            || operation.expected_split_siblings != operation.split_siblings.len()
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        if operation.old_current_hash != Some(update.current_root) {
            return Err(ProofChkErr::JmtUpdateProofMix);
        }
        let first_non_default = operation
            .siblings
            .iter()
            .position(|sibling| !matches!(sibling, JmtSiblingWireV2::Null));
        match operation.mutation_case {
            Some(3) => {
                let proof = JmtSparseProofWireV2 {
                    leaf: operation.proof_leaf.flatten(),
                    siblings: operation.siblings.clone(),
                };
                let old_leaf = proof.leaf.ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
                let common = common_prefix_bits(&old_leaf.key_hash, &operation.key);
                let expected = jmt_split_siblings(&proof, common)?;
                if expected[..operation.expected_split_siblings] != operation.split_siblings {
                    return Err(ProofChkErr::JmtUpdateTraceCanonical);
                }
            }
            Some(4) if first_non_default.is_some() => {
                return Err(ProofChkErr::JmtUpdateTraceCanonical)
            }
            Some(5)
                if !first_non_default.is_some_and(|index| {
                    matches!(operation.siblings[index], JmtSiblingWireV2::Internal(_))
                }) =>
            {
                return Err(ProofChkErr::JmtUpdateTraceCanonical)
            }
            Some(6)
                if !first_non_default.is_some_and(|index| {
                    matches!(operation.siblings[index], JmtSiblingWireV2::Leaf(_))
                }) =>
            {
                return Err(ProofChkErr::JmtUpdateTraceCanonical)
            }
            Some(1..=6) => {}
            _ => return Err(ProofChkErr::JmtUpdateTraceCanonical),
        }
        operation.proof_closed = true;
        Ok(())
    }

    fn accept_operation_end(
        &mut self,
        reader: &mut CanonicalReader<'_>,
    ) -> Result<(), ProofChkErr> {
        let (update_index, operation_index) = take_circuit_operation_indices(reader)?;
        reader.finish()?;
        let update = self
            .current_update
            .as_mut()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len() || operation_index != update.operations.len() {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let operation = update
            .current_operation
            .take()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if operation.value.len() != operation.expected_value_bytes {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        if operation.prior_value.len() != operation.expected_prior_value_bytes {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        if !operation.proof_closed {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let computed_new_root = operation.new_current_hash;
        let proof = JmtSparseProofWireV2 {
            leaf: operation
                .proof_leaf
                .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?,
            siblings: operation.siblings,
        };
        let operation = JmtUpdateOpV2 {
            key: operation.key,
            prior_value: operation
                .prior_value_present
                .then_some(operation.prior_value),
            value: operation.value_present.then_some(operation.value),
        };
        let (_, next_root) =
            verify_jmt_transition_semantics(&proof, &operation, update.current_root)?;
        if computed_new_root != Some(next_root) {
            return Err(ProofChkErr::JmtUpdateProofMix);
        }
        update.current_root = next_root;
        update.operations.push(operation);
        update.proofs.push(proof);
        Ok(())
    }

    fn accept_update_end(&mut self, reader: &mut CanonicalReader<'_>) -> Result<(), ProofChkErr> {
        let update_index =
            usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        reader.finish()?;
        let update = self
            .current_update
            .take()
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        if update_index != self.updates.len()
            || update.current_operation.is_some()
            || update.operations.len() != update.expected_operations
            || update.proofs.len() != update.expected_operations
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        let proof_wire = BincodeCodec.serialize(&JmtUpdateProofWireV2(update.proofs))?;
        if proof_wire.len() > JMT_UPDATE_TRACE_MAX_PROOF_BYTES_V2 {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        let trace = JmtUpdateTraceV2 {
            version: JMT_UPDATE_TRACE_VERSION_V2,
            tree: update.tree,
            old_version: update.old_version,
            new_version: update.new_version,
            old_root: update.old_root,
            new_root: update.new_root,
            operations: update.operations,
            proof_wire,
        };
        trace.verify_semantics()?;
        trace.verify_native()?;
        self.updates.push(trace);
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<SettlementUpdateTraceEnvelopeV2, ProofChkErr> {
        if self.current_update.is_some() || self.updates.len() != self.expected_updates {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        if self.kind == JMT_UPDATE_TRACE_KIND_NOOP_V2 {
            return SettlementUpdateTraceEnvelopeV2::new_noop(RootGeneration::SettlementV2);
        }
        let envelope = SettlementUpdateTraceEnvelopeV2 {
            version: JMT_UPDATE_TRACE_VERSION_V2,
            root_generation: RootGeneration::SettlementV2.version(),
            kind: self.kind,
            trace_digest: self.expected_digest,
            updates: self.updates,
        };
        if envelope.circuit_trace_digest()? != envelope.trace_digest {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
        Ok(envelope)
    }
}

fn take_circuit_operation_indices(
    reader: &mut CanonicalReader<'_>,
) -> Result<(usize, usize), ProofChkErr> {
    Ok((
        usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?,
        usize::try_from(reader.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?,
    ))
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RootTransitionV2 {
    old: [u8; 32],
    new: [u8; 32],
}

impl RootTransitionV2 {
    const fn new(old: [u8; 32], new: [u8; 32]) -> Self {
        Self { old, new }
    }
}

fn verify_parent_transition<T>(
    operation: &JmtUpdateOpV2,
    child: RootTransitionV2,
    decode: impl Fn(&[u8]) -> Result<(T, [u8; 32]), ProofChkErr>,
) -> Result<(Option<T>, Option<T>), ProofChkErr> {
    let prior = match operation.prior_value.as_deref() {
        Some(value) => {
            let (coordinates, child_root) = decode(value)?;
            if child_root != child.old {
                return Err(ProofChkErr::JmtUpdateTraceCanonical);
            }
            Some(coordinates)
        }
        None if child.old == JMT_SPARSE_PLACEHOLDER_HASH_V2 => None,
        None => return Err(ProofChkErr::JmtUpdateTraceCanonical),
    };
    let next = match operation.value.as_deref() {
        Some(value) => {
            let (coordinates, child_root) = decode(value)?;
            if child_root != child.new {
                return Err(ProofChkErr::JmtUpdateTraceCanonical);
            }
            Some(coordinates)
        }
        None if child.new == JMT_SPARSE_PLACEHOLDER_HASH_V2 => None,
        None => return Err(ProofChkErr::JmtUpdateTraceCanonical),
    };
    Ok((prior, next))
}

fn verify_bucket_promotion(
    definition_id: [u8; 32],
    serial_id: u32,
    operation: &JmtUpdateOpV2,
    terminal_roots: &BTreeMap<TerminalRootKeyV2, RootTransitionV2>,
) -> Result<TerminalRootKeyV2, ProofChkErr> {
    let key = (definition_id, serial_id, operation.key);
    let child = *terminal_roots
        .get(&key)
        .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
    let (prior, next) = verify_parent_transition(operation, child, |value| {
        let (value_definition, value_serial, bucket_id, child_root, policy) =
            decode_bucket_parent_value(value)?;
        Ok((
            (value_definition, value_serial, bucket_id, policy),
            child_root,
        ))
    })?;
    for coordinates in [prior, next].into_iter().flatten() {
        if coordinates.0 != definition_id
            || coordinates.1 != serial_id
            || coordinates.2 != operation.key
        {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
    }
    Ok(key)
}

fn verify_serial_promotion(
    definition_id: [u8; 32],
    operation: &JmtUpdateOpV2,
    bucket_roots: &BTreeMap<([u8; 32], u32), RootTransitionV2>,
) -> Result<([u8; 32], u32), ProofChkErr> {
    let mut matches = bucket_roots.iter().filter_map(|(key, child)| {
        (key.0 == definition_id
            && operation.key == serial_key(DefinitionId::new(key.0), SerialId::new(key.1)).0)
            .then_some((*key, *child))
    });
    let (key, child) = matches.next().ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
    if matches.next().is_some() {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    let (prior, next) = verify_parent_transition(operation, child, |value| {
        let (value_definition, serial_id, child_root) = decode_serial_parent_value(value)?;
        Ok(((value_definition, serial_id), child_root))
    })?;
    for coordinates in [prior, next].into_iter().flatten() {
        if coordinates != key {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
    }
    Ok(key)
}

fn verify_definition_promotion(
    operation: &JmtUpdateOpV2,
    serial_roots: &BTreeMap<[u8; 32], RootTransitionV2>,
) -> Result<[u8; 32], ProofChkErr> {
    let mut matches = serial_roots.iter().filter_map(|(definition_id, child)| {
        (operation.key == definition_key(DefinitionId::new(*definition_id)).0)
            .then_some((*definition_id, *child))
    });
    let (definition_id, child) = matches.next().ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
    if matches.next().is_some() {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    let (prior, next) = verify_parent_transition(operation, child, |value| {
        decode_definition_parent_value(value)
    })?;
    for value_definition in [prior, next].into_iter().flatten() {
        if value_definition != definition_id {
            return Err(ProofChkErr::JmtUpdateTraceCanonical);
        }
    }
    Ok(definition_id)
}

fn decode_definition_parent_value(value: &[u8]) -> Result<([u8; 32], [u8; 32]), ProofChkErr> {
    if value.len() != 64 {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    Ok((
        value[..32]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        value[32..]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
    ))
}

fn decode_serial_parent_value(value: &[u8]) -> Result<([u8; 32], u32, [u8; 32]), ProofChkErr> {
    if value.len() != 68 {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    Ok((
        value[..32]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        u32::from_le_bytes(
            value[32..36]
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ),
        value[36..]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
    ))
}

fn decode_bucket_parent_value(value: &[u8]) -> Result<BucketParentValueV2, ProofChkErr> {
    if value.len() != 132 {
        return Err(ProofChkErr::JmtUpdateTraceCanonical);
    }
    Ok((
        value[..32]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        u32::from_le_bytes(
            value[32..36]
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ),
        value[36..68]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        value[68..100]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        value[100..]
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
    ))
}

/// Cursor for the project-owned V2 trace wire.
///
/// Every variable-length field is bounded and proven resident in the input
/// before a `Vec` is reserved or copied.  This is intentionally separate from
/// the upstream opaque proof payload, whose bytes are admitted only after this
/// outer grammar has enforced the exact 24 MiB cap.
struct CanonicalReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> CanonicalReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take_u8(&mut self) -> Result<u8, ProofChkErr> {
        let value = *self
            .bytes
            .get(self.offset)
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        self.offset = self
            .offset
            .checked_add(1)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        Ok(value)
    }

    fn take_u32(&mut self) -> Result<u32, ProofChkErr> {
        let bytes = self.take_exact(4)?;
        Ok(u32::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ))
    }

    fn take_u16(&mut self) -> Result<u16, ProofChkErr> {
        let bytes = self.take_exact(2)?;
        Ok(u16::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ))
    }

    fn take_u64(&mut self) -> Result<u64, ProofChkErr> {
        let bytes = self.take_exact(8)?;
        Ok(u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)?,
        ))
    }

    fn take_array(&mut self) -> Result<[u8; 32], ProofChkErr> {
        self.take_exact(32)?
            .try_into()
            .map_err(|_| ProofChkErr::JmtUpdateTraceCanonical)
    }

    #[cfg(test)]
    fn take_vec(&mut self, max_len: usize) -> Result<Vec<u8>, ProofChkErr> {
        let bytes = self.take_borrowed(max_len)?;
        let mut value = Vec::new();
        value
            .try_reserve_exact(bytes.len())
            .map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        value.extend_from_slice(bytes);
        Ok(value)
    }

    #[cfg(test)]
    fn take_borrowed(&mut self, max_len: usize) -> Result<&'a [u8], ProofChkErr> {
        let len =
            usize::try_from(self.take_u32()?).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
        if len > max_len {
            return Err(ProofChkErr::JmtUpdateTraceLimit);
        }
        self.take_exact(len)
    }

    fn take_exact(&mut self, len: usize) -> Result<&'a [u8], ProofChkErr> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(ProofChkErr::JmtUpdateTraceLimit)?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(ProofChkErr::JmtUpdateTraceCanonical)?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(&self) -> Result<(), ProofChkErr> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(ProofChkErr::JmtUpdateTraceCanonical)
        }
    }
}

#[cfg(test)]
fn append_len_prefixed(out: &mut Vec<u8>, value: &[u8]) -> Result<(), ProofChkErr> {
    let len = u32::try_from(value.len()).map_err(|_| ProofChkErr::JmtUpdateTraceLimit)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    Ok(())
}

pub(crate) fn noop_update_trace_digest() -> [u8; 32] {
    hash_zk::<StorBatchProofDom>(
        JMT_UPDATE_TRACE_NOOP_LABEL_V2,
        &[&[JMT_UPDATE_TRACE_VERSION_V2, JMT_UPDATE_TRACE_KIND_NOOP_V2]],
    )
}

#[must_use]
pub fn batch_proof_transcript_domain_v1() -> [u8; 32] {
    *BATCH_PROOF_TRANSCRIPT_DOMAIN.get_or_init(|| {
        hash_zk::<StorBatchProofDom>(
            "proof_batch_transcript_domain_v1",
            &[&[BATCH_PROOF_ENCODING_VERSION]],
        )
    })
}

#[must_use]
pub(crate) fn batch_proof_accept_transcript_v1(bytes: &[u8]) -> [u8; 32] {
    hash_zk::<StorBatchProofDom>("proof_batch_accept_transcript_v1", &[bytes])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum BatchProofFamilyTagV1 {
    Inclusion = 0x01,
    Deletion = 0x02,
    NonExistence = 0x03,
}

impl BatchProofFamilyTagV1 {
    #[must_use]
    pub const fn from_live(family: HjmtProofFamily) -> Self {
        match family {
            HjmtProofFamily::Inclusion => Self::Inclusion,
            HjmtProofFamily::Deletion => Self::Deletion,
            HjmtProofFamily::NonExistence => Self::NonExistence,
        }
    }

    #[must_use]
    pub const fn into_live(self) -> HjmtProofFamily {
        match self {
            Self::Inclusion => HjmtProofFamily::Inclusion,
            Self::Deletion => HjmtProofFamily::Deletion,
            Self::NonExistence => HjmtProofFamily::NonExistence,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Inclusion),
            0x02 => Ok(Self::Deletion),
            0x03 => Ok(Self::NonExistence),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum TerminalFamilyTagV1 {
    Asset = 0x01,
    Right = 0x02,
    Voucher = 0x03,
}

impl TerminalFamilyTagV1 {
    #[must_use]
    pub const fn from_live(family: SettlementLeafFamily) -> Self {
        match family {
            SettlementLeafFamily::Terminal => Self::Asset,
            SettlementLeafFamily::Right => Self::Right,
            SettlementLeafFamily::Voucher => Self::Voucher,
        }
    }

    #[must_use]
    pub const fn into_live(self) -> SettlementLeafFamily {
        match self {
            Self::Asset => SettlementLeafFamily::Terminal,
            Self::Right => SettlementLeafFamily::Right,
            Self::Voucher => SettlementLeafFamily::Voucher,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Asset),
            0x02 => Ok(Self::Right),
            0x03 => Ok(Self::Voucher),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum LeafFamilyTagV1 {
    Asset = 0x01,
    Right = 0x02,
    Voucher = 0x03,
}

impl LeafFamilyTagV1 {
    #[must_use]
    pub const fn from_live(family: SettlementLeafFamily) -> Self {
        match family {
            SettlementLeafFamily::Terminal => Self::Asset,
            SettlementLeafFamily::Right => Self::Right,
            SettlementLeafFamily::Voucher => Self::Voucher,
        }
    }

    #[must_use]
    pub const fn into_live(self) -> SettlementLeafFamily {
        match self {
            Self::Asset => SettlementLeafFamily::Terminal,
            Self::Right => SettlementLeafFamily::Right,
            Self::Voucher => SettlementLeafFamily::Voucher,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Asset),
            0x02 => Ok(Self::Right),
            0x03 => Ok(Self::Voucher),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum RootGenerationTagV1 {
    RootGeneration0 = 0x00,
    RootGeneration1 = 0x01,
    RootGeneration2 = 0x02,
}

impl RootGenerationTagV1 {
    #[must_use]
    pub const fn from_live(root: SettlementStateRoot) -> Self {
        match root.generation() {
            RootGeneration::SettlementV1 => Self::RootGeneration1,
            RootGeneration::SettlementV2 => Self::RootGeneration2,
        }
    }

    #[must_use]
    pub const fn into_version(self) -> u8 {
        match self {
            Self::RootGeneration0 => 0,
            Self::RootGeneration1 => 1,
            Self::RootGeneration2 => 2,
        }
    }

    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x00 => Ok(Self::RootGeneration0),
            0x01 => Ok(Self::RootGeneration1),
            0x02 => Ok(Self::RootGeneration2),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeDomainTagV1 {
    Terminal = 0x01,
    Bucket = 0x02,
    Serial = 0x03,
    Definition = 0x04,
    Shard = 0x05,
    Global = 0x06,
}

impl NodeDomainTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::Terminal),
            0x02 => Ok(Self::Bucket),
            0x03 => Ok(Self::Serial),
            0x04 => Ok(Self::Definition),
            0x05 => Ok(Self::Shard),
            0x06 => Ok(Self::Global),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum SiblingSideTagV1 {
    LeftSibling = 0x00,
    RightSibling = 0x01,
}

impl SiblingSideTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x00 => Ok(Self::LeftSibling),
            0x01 => Ok(Self::RightSibling),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpeningKindTagV1 {
    InclusionLeaf = 0x01,
    DeletionFact = 0x02,
    AbsenceOpening = 0x03,
}

impl OpeningKindTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x01 => Ok(Self::InclusionLeaf),
            0x02 => Ok(Self::DeletionFact),
            0x03 => Ok(Self::AbsenceOpening),
            _ => Err(ProofChkErr::BatchTagMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofLimits {
    pub max_path_count: u32,
    pub max_witness_node_count: u32,
    pub max_opening_count: u32,
    pub max_reference_count: u32,
    pub max_total_bytes: u32,
}

impl BatchProofLimits {
    #[must_use]
    pub const fn v1() -> Self {
        Self {
            max_path_count: 1_024,
            max_witness_node_count: 16_384,
            max_opening_count: 1_024,
            max_reference_count: 1_024,
            max_total_bytes: 8 * 1024 * 1024,
        }
    }

    #[must_use]
    pub const fn from_policy(policy: BucketPolicy) -> Self {
        let base = Self::v1();
        let _ = policy;
        Self {
            max_path_count: base.max_path_count,
            ..base
        }
    }

    #[must_use]
    pub(crate) const fn contains(self, other: Self) -> bool {
        other.max_path_count <= self.max_path_count
            && other.max_witness_node_count <= self.max_witness_node_count
            && other.max_opening_count <= self.max_opening_count
            && other.max_reference_count <= self.max_reference_count
            && other.max_total_bytes <= self.max_total_bytes
    }

    pub(crate) fn encode(self, out: &mut Vec<u8>) {
        put_u32(out, self.max_path_count);
        put_u32(out, self.max_witness_node_count);
        put_u32(out, self.max_opening_count);
        put_u32(out, self.max_reference_count);
        put_u32(out, self.max_total_bytes);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        Ok(Self {
            max_path_count: take_u32(bytes, pos)?,
            max_witness_node_count: take_u32(bytes, pos)?,
            max_opening_count: take_u32(bytes, pos)?,
            max_reference_count: take_u32(bytes, pos)?,
            max_total_bytes: take_u32(bytes, pos)?,
        })
    }
}

impl Default for BatchProofLimits {
    fn default() -> Self {
        Self::v1()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum PublicationModeTagV1 {
    Synchronous = 0x00,
    CheckpointWindow = 0x01,
    EmergencyFreeze = 0x02,
}

impl PublicationModeTagV1 {
    pub(crate) fn decode(tag: u8) -> Result<Self, ProofChkErr> {
        match tag {
            0x00 => Ok(Self::Synchronous),
            0x01 => Ok(Self::CheckpointWindow),
            0x02 => Ok(Self::EmergencyFreeze),
            _ => Err(ProofChkErr::PublicationModeMix),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySetMemberV1 {
    pub policy_generation: u64,
    pub bucket_policy_digest: [u8; 32],
    pub activation_checkpoint: u64,
    pub retirement_checkpoint: Option<u64>,
}

impl PolicySetMemberV1 {
    #[must_use]
    pub const fn new(
        policy_generation: u64,
        bucket_policy_digest: [u8; 32],
        activation_checkpoint: u64,
        retirement_checkpoint: Option<u64>,
    ) -> Self {
        Self {
            policy_generation,
            bucket_policy_digest,
            activation_checkpoint,
            retirement_checkpoint,
        }
    }

    #[must_use]
    pub const fn policy_digest(self) -> [u8; 32] {
        self.bucket_policy_digest
    }

    #[must_use]
    pub const fn is_active_at(self, checkpoint: u64) -> bool {
        self.activation_checkpoint <= checkpoint
            && match self.retirement_checkpoint {
                Some(retire_at) => checkpoint < retire_at,
                None => true,
            }
    }

    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_u64(out, self.policy_generation);
        out.extend_from_slice(&self.bucket_policy_digest);
        put_u64(out, self.activation_checkpoint);
        put_opt_u64(out, self.retirement_checkpoint);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        Ok(Self {
            policy_generation: take_pub_u64(bytes, pos)?,
            bucket_policy_digest: take_pub_32(bytes, pos)?,
            activation_checkpoint: take_pub_u64(bytes, pos)?,
            retirement_checkpoint: take_pub_opt_u64(bytes, pos)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySetCommitmentV1 {
    pub members: Vec<PolicySetMemberV1>,
}

impl PolicySetCommitmentV1 {
    #[must_use]
    pub fn new(members: Vec<PolicySetMemberV1>) -> Self {
        Self { members }
    }

    #[must_use]
    pub fn singleton_live(
        policy_generation: u64,
        bucket_policy_digest: [u8; 32],
        activation_checkpoint: u64,
    ) -> Self {
        Self::new(vec![PolicySetMemberV1::new(
            policy_generation,
            bucket_policy_digest,
            activation_checkpoint,
            None,
        )])
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.members.is_empty() {
            return Err(ProofChkErr::PublicationPolicyMix);
        }
        let mut prev: Option<PolicySetMemberV1> = None;
        for member in &self.members {
            if member
                .retirement_checkpoint
                .is_some_and(|retire_at| retire_at <= member.activation_checkpoint)
            {
                return Err(ProofChkErr::PublicationPolicyMix);
            }
            if let Some(last) = prev {
                let last_key = (
                    last.policy_generation,
                    last.bucket_policy_digest,
                    last.activation_checkpoint,
                );
                let next_key = (
                    member.policy_generation,
                    member.bucket_policy_digest,
                    member.activation_checkpoint,
                );
                if next_key <= last_key {
                    return Err(ProofChkErr::PublicationPolicyMix);
                }
                if member.policy_generation == last.policy_generation
                    && member.bucket_policy_digest == last.bucket_policy_digest
                    && member.activation_checkpoint < last.retirement_checkpoint.unwrap_or(u64::MAX)
                {
                    return Err(ProofChkErr::PublicationPolicyMix);
                }
            }
            prev = Some(*member);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        self.check_contract_v1()?;
        let mut out = Vec::new();
        for member in &self.members {
            member.encode(&mut out);
        }
        Ok(out)
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0usize;
        let mut members = Vec::new();
        while pos < bytes.len() {
            members.push(PolicySetMemberV1::decode(bytes, &mut pos)?);
        }
        let out = Self { members };
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn digest(&self) -> Result<[u8; 32], ProofChkErr> {
        Ok(hash_zk::<StorPolicySetDom>("", &[&self.canonical_bytes()?]))
    }

    pub fn verify_member(
        &self,
        policy_generation: u64,
        policy_digest: [u8; 32],
        proof_checkpoint: u64,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        let found = self.members.iter().any(|member| {
            member.policy_generation == policy_generation
                && member.policy_digest() == policy_digest
                && member.is_active_at(proof_checkpoint)
        });
        if !found {
            return Err(ProofChkErr::PublicationPolicyMix);
        }
        Ok(())
    }
}

// Storage owns the committed publication route snapshot. Wallet-facing code
// verifies this public proof surface and must not read raw backend or journal
// internals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationRouteSnapshotV1 {
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub activation_checkpoint: u64,
    pub shard_ids: Vec<u32>,
}

impl PublicationRouteSnapshotV1 {
    #[must_use]
    pub fn new(
        routing_generation: u64,
        route_table_digest: [u8; 32],
        activation_checkpoint: u64,
        shard_ids: Vec<u32>,
    ) -> Self {
        Self {
            routing_generation,
            route_table_digest,
            activation_checkpoint,
            shard_ids,
        }
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.shard_ids.is_empty() {
            return Err(ProofChkErr::PublicationCountMix);
        }
        if self.shard_ids.len() > u32::MAX as usize {
            return Err(ProofChkErr::PublicationCountMix);
        }
        let mut prev = None;
        for shard_id in &self.shard_ids {
            if let Some(last) = prev {
                if *shard_id <= last {
                    return if *shard_id == last {
                        Err(ProofChkErr::PublicationDupShard)
                    } else {
                        Err(ProofChkErr::PublicationOrderMix)
                    };
                }
            }
            prev = Some(*shard_id);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationHandoffRowV1 {
    pub shard_id: u32,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub checkpoint_id: [u8; 32],
}

impl PublicationHandoffRowV1 {
    #[must_use]
    pub const fn new(
        shard_id: u32,
        routing_generation: u64,
        route_table_digest: [u8; 32],
        checkpoint_id: [u8; 32],
    ) -> Self {
        Self {
            shard_id,
            routing_generation,
            route_table_digest,
            checkpoint_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardRootLeafV1 {
    pub shard_id: u32,
    pub shard_root: [u8; 32],
    pub shard_epoch: u64,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub policy_set_digest: [u8; 32],
    pub journal_checkpoint: u64,
    pub local_sequence: u64,
    pub transition_flags: u32,
}

impl ShardRootLeafV1 {
    #[must_use]
    pub const fn new(
        shard_id: u32,
        shard_root: [u8; 32],
        shard_epoch: u64,
        routing_generation: u64,
        route_table_digest: [u8; 32],
        policy_set_digest: [u8; 32],
        journal_checkpoint: u64,
        local_sequence: u64,
        transition_flags: u32,
    ) -> Self {
        Self {
            shard_id,
            shard_root,
            shard_epoch,
            routing_generation,
            route_table_digest,
            policy_set_digest,
            journal_checkpoint,
            local_sequence,
            transition_flags,
        }
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.transition_flags & !0b111 != 0 {
            return Err(ProofChkErr::PublicationFlagMix);
        }
        Ok(())
    }

    pub fn check_route_binding_v1(
        &self,
        expected_route_table_digest: [u8; 32],
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.route_table_digest != expected_route_table_digest {
            return Err(ProofChkErr::PublicationRouteMix);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        self.check_contract_v1()?;
        let mut out = Vec::with_capacity(136);
        put_u32(&mut out, self.shard_id);
        out.extend_from_slice(&self.shard_root);
        put_u64(&mut out, self.shard_epoch);
        put_u64(&mut out, self.routing_generation);
        out.extend_from_slice(&self.route_table_digest);
        out.extend_from_slice(&self.policy_set_digest);
        put_u64(&mut out, self.journal_checkpoint);
        put_u64(&mut out, self.local_sequence);
        put_u32(&mut out, self.transition_flags);
        Ok(out)
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0usize;
        let out = Self {
            shard_id: take_pub_u32(bytes, &mut pos)?,
            shard_root: take_pub_32(bytes, &mut pos)?,
            shard_epoch: take_pub_u64(bytes, &mut pos)?,
            routing_generation: take_pub_u64(bytes, &mut pos)?,
            route_table_digest: take_pub_32(bytes, &mut pos)?,
            policy_set_digest: take_pub_32(bytes, &mut pos)?,
            journal_checkpoint: take_pub_u64(bytes, &mut pos)?,
            local_sequence: take_pub_u64(bytes, &mut pos)?,
            transition_flags: take_pub_u32(bytes, &mut pos)?,
        };
        if pos != bytes.len() {
            return Err(ProofChkErr::PublicationTrailingBytes);
        }
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn digest(&self) -> Result<[u8; 32], ProofChkErr> {
        Ok(hash_zk::<StorShardRootLeafDom>(
            "",
            &[&self.canonical_bytes()?],
        ))
    }

    pub fn verify_policy_member(
        &self,
        policy_set: &PolicySetCommitmentV1,
        policy_generation: u64,
        policy_digest: [u8; 32],
        proof_checkpoint: u64,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.policy_set_digest != policy_set.digest()? {
            return Err(ProofChkErr::PublicationPolicyMix);
        }
        policy_set.verify_member(policy_generation, policy_digest, proof_checkpoint)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointPublicationV1 {
    pub root_generation: RootGenerationTagV1,
    pub publication_mode: PublicationModeTagV1,
    pub publication_checkpoint: u64,
    pub route_table_digest: [u8; 32],
    pub prior_public_root: SettlementStateRoot,
    pub shard_leaves: Vec<ShardRootLeafV1>,
}

impl CheckpointPublicationV1 {
    #[must_use]
    pub fn new(
        root_generation: RootGenerationTagV1,
        publication_mode: PublicationModeTagV1,
        publication_checkpoint: u64,
        route_table_digest: [u8; 32],
        prior_public_root: SettlementStateRoot,
        shard_leaves: Vec<ShardRootLeafV1>,
    ) -> Self {
        Self {
            root_generation,
            publication_mode,
            publication_checkpoint,
            route_table_digest,
            prior_public_root,
            shard_leaves,
        }
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        if self.root_generation != RootGenerationTagV1::RootGeneration1 {
            return Err(ProofChkErr::PublicationRootGenerationMix);
        }
        if self.shard_leaves.is_empty() {
            return Err(ProofChkErr::PublicationCountMix);
        }
        if self.shard_leaves.len() > u32::MAX as usize {
            return Err(ProofChkErr::PublicationCountMix);
        }
        let mut prev = None;
        for leaf in &self.shard_leaves {
            leaf.check_contract_v1()?;
            if leaf.route_table_digest != self.route_table_digest {
                return Err(ProofChkErr::PublicationRouteMix);
            }
            if let Some(last) = prev {
                if leaf.shard_id <= last {
                    return if leaf.shard_id == last {
                        Err(ProofChkErr::PublicationDupShard)
                    } else {
                        Err(ProofChkErr::PublicationOrderMix)
                    };
                }
            }
            prev = Some(leaf.shard_id);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        self.check_contract_v1()?;
        let mut out = Vec::new();
        out.push(self.root_generation as u8);
        out.push(self.publication_mode as u8);
        put_u64(&mut out, self.publication_checkpoint);
        out.extend_from_slice(&self.route_table_digest);
        put_state_root(&mut out, self.prior_public_root);
        put_u32(&mut out, self.shard_leaves.len() as u32);
        for leaf in &self.shard_leaves {
            out.extend_from_slice(&leaf.canonical_bytes()?);
        }
        Ok(out)
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0usize;
        let root_generation = RootGenerationTagV1::decode(take_pub_u8(bytes, &mut pos)?)?;
        let publication_mode = PublicationModeTagV1::decode(take_pub_u8(bytes, &mut pos)?)?;
        let publication_checkpoint = take_pub_u64(bytes, &mut pos)?;
        let route_table_digest = take_pub_32(bytes, &mut pos)?;
        let prior_public_root = take_pub_state_root(bytes, &mut pos)?;
        let leaf_count = take_pub_u32(bytes, &mut pos)? as usize;
        let mut shard_leaves = Vec::with_capacity(leaf_count);
        for _ in 0..leaf_count {
            let start = pos;
            pos = pos.saturating_add(136);
            if pos > bytes.len() {
                return Err(ProofChkErr::PublicationTrunc);
            }
            shard_leaves.push(ShardRootLeafV1::from_canon(&bytes[start..pos])?);
        }
        if pos != bytes.len() {
            return Err(ProofChkErr::PublicationTrailingBytes);
        }
        let out = Self {
            root_generation,
            publication_mode,
            publication_checkpoint,
            route_table_digest,
            prior_public_root,
            shard_leaves,
        };
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn digest(&self) -> Result<[u8; 32], ProofChkErr> {
        Ok(hash_zk::<StorCheckpointPublicationDom>(
            "",
            &[&self.canonical_bytes()?],
        ))
    }

    pub fn public_root_v1(&self) -> Result<SettlementStateRoot, ProofChkErr> {
        Ok(SettlementStateRoot::settlement_v1(self.digest()?))
    }

    pub fn check_prior_root_v1(
        &self,
        expected_prior_root: SettlementStateRoot,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.prior_public_root != expected_prior_root {
            return Err(ProofChkErr::PublicationPriorRootMix);
        }
        Ok(())
    }

    pub fn check_monotonic_successor_v1(
        &self,
        prev: &CheckpointPublicationV1,
    ) -> Result<(), ProofChkErr> {
        prev.check_contract_v1()?;
        self.check_contract_v1()?;
        if self.publication_checkpoint <= prev.publication_checkpoint {
            return Err(ProofChkErr::PublicationCheckpointMix);
        }
        let same_route = self.route_table_digest == prev.route_table_digest;
        if same_route && self.shard_leaves.len() != prev.shard_leaves.len() {
            return Err(ProofChkErr::PublicationCountMix);
        }

        let mut last_by_shard = std::collections::BTreeMap::new();
        for leaf in &prev.shard_leaves {
            last_by_shard.insert(leaf.shard_id, *leaf);
        }
        for leaf in &self.shard_leaves {
            let Some(prev_leaf) = last_by_shard.get(&leaf.shard_id).copied() else {
                if same_route {
                    return Err(ProofChkErr::PublicationCountMix);
                }
                continue;
            };
            if leaf.routing_generation < prev_leaf.routing_generation {
                return Err(ProofChkErr::PublicationMonotonicityMix);
            }
            if leaf.journal_checkpoint < prev_leaf.journal_checkpoint {
                return Err(ProofChkErr::PublicationMonotonicityMix);
            }
            if leaf.routing_generation == prev_leaf.routing_generation {
                let prev_key = (
                    prev_leaf.shard_epoch,
                    prev_leaf.local_sequence,
                    prev_leaf.journal_checkpoint,
                );
                let next_key = (
                    leaf.shard_epoch,
                    leaf.local_sequence,
                    leaf.journal_checkpoint,
                );
                if next_key < prev_key {
                    return Err(ProofChkErr::PublicationMonotonicityMix);
                }
                if leaf != &prev_leaf && next_key <= prev_key {
                    return Err(ProofChkErr::PublicationMonotonicityMix);
                }
            }
        }
        if same_route {
            for prev_leaf in &prev.shard_leaves {
                if !self
                    .shard_leaves
                    .iter()
                    .any(|leaf| leaf.shard_id == prev_leaf.shard_id)
                {
                    return Err(ProofChkErr::PublicationCountMix);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShardProofContextV1 {
    pub shard_id: u32,
    pub routing_generation: u64,
    pub route_table_digest: [u8; 32],
    pub policy_generation: u64,
    pub bucket_policy_digest: [u8; 32],
    pub proof_family: HjmtProofFamily,
    pub leaf_family: SettlementLeafFamily,
}

impl ShardProofContextV1 {
    #[must_use]
    pub const fn new(
        shard_id: u32,
        routing_generation: u64,
        route_table_digest: [u8; 32],
        policy_generation: u64,
        bucket_policy_digest: [u8; 32],
        proof_family: HjmtProofFamily,
        leaf_family: SettlementLeafFamily,
    ) -> Self {
        Self {
            shard_id,
            routing_generation,
            route_table_digest,
            policy_generation,
            bucket_policy_digest,
            proof_family,
            leaf_family,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointPublicationProofV1 {
    pub root_generation: RootGenerationTagV1,
    pub public_root: SettlementStateRoot,
    pub publication: CheckpointPublicationV1,
    pub shard_leaf_index: u32,
    pub shard_context: ShardProofContextV1,
    pub policy_set: PolicySetCommitmentV1,
    pub shard_proof: ProofBlob,
}

impl CheckpointPublicationProofV1 {
    #[must_use]
    pub fn new(
        root_generation: RootGenerationTagV1,
        public_root: SettlementStateRoot,
        publication: CheckpointPublicationV1,
        shard_leaf_index: u32,
        shard_context: ShardProofContextV1,
        policy_set: PolicySetCommitmentV1,
        shard_proof: ProofBlob,
    ) -> Self {
        Self {
            root_generation,
            public_root,
            publication,
            shard_leaf_index,
            shard_context,
            policy_set,
            shard_proof,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProofChkErr> {
        Ok(BincodeCodec.serialize(self)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let out: Self = BincodeCodec.deserialize(bytes)?;
        out.check_contract_v1()?;
        Ok(out)
    }

    pub fn shard_leaf_v1(&self) -> Result<ShardRootLeafV1, ProofChkErr> {
        self.publication
            .shard_leaves
            .get(self.shard_leaf_index as usize)
            .copied()
            .ok_or(ProofChkErr::PublicationProofIndexMix)
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        self.publication.check_contract_v1()?;
        self.policy_set.check_contract_v1()?;
        if self.root_generation != RootGenerationTagV1::RootGeneration1
            || self.root_generation != self.publication.root_generation
            || self.public_root.generation_version() != self.root_generation.into_version()
        {
            return Err(ProofChkErr::PublicationProofGenerationMix);
        }
        if self.publication.public_root_v1()? != self.public_root {
            return Err(ProofChkErr::RootMix);
        }

        let shard_leaf = self.shard_leaf_v1()?;
        if shard_leaf.shard_id != self.shard_context.shard_id {
            return Err(ProofChkErr::PublicationProofShardMix);
        }
        if shard_leaf.routing_generation != self.shard_context.routing_generation
            || shard_leaf.route_table_digest != self.shard_context.route_table_digest
            || self.publication.route_table_digest != self.shard_context.route_table_digest
        {
            return Err(ProofChkErr::PublicationProofRouteMix);
        }
        if shard_leaf.policy_set_digest != self.policy_set.digest()? {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        }

        let proof_item = self.shard_proof.item();
        if proof_item.settlement_root().into_bytes() != shard_leaf.shard_root {
            return Err(ProofChkErr::RootMix);
        }
        if self.shard_proof.hjmt_proof_family() != Some(self.shard_context.proof_family) {
            return Err(ProofChkErr::ProofFamilyMix);
        }
        if self.shard_proof.hjmt_leaf_family() != Some(self.shard_context.leaf_family) {
            return Err(ProofChkErr::LeafMix);
        }

        let Some(journal_checkpoint) = self.shard_proof.hjmt_journal_checkpoint() else {
            return Err(ProofChkErr::PublicationProofCheckpointMix);
        };
        if journal_checkpoint != shard_leaf.journal_checkpoint {
            return Err(ProofChkErr::PublicationProofCheckpointMix);
        }

        let Some(bucket_policy) = self.shard_proof.hjmt_bucket_policy() else {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        };
        if u64::from(bucket_policy.compatibility_generation())
            != self.shard_context.policy_generation
            || bucket_policy.bucket_policy_id() != self.shard_context.bucket_policy_digest
        {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        }

        shard_leaf.verify_policy_member(
            &self.policy_set,
            self.shard_context.policy_generation,
            self.shard_context.bucket_policy_digest,
            journal_checkpoint,
        )?;
        Ok(())
    }

    pub fn check_public_root_v1(
        &self,
        expected_public_root: SettlementStateRoot,
    ) -> Result<(), ProofChkErr> {
        self.check_contract_v1()?;
        if self.public_root != expected_public_root {
            return Err(ProofChkErr::RootMix);
        }
        Ok(())
    }

    pub fn verify_v1(&self) -> Result<ProofBlob, ProofChkErr> {
        self.verify_against_public_root_v1(self.public_root)
    }

    pub fn verify_against_public_root_v1(
        &self,
        expected_public_root: SettlementStateRoot,
    ) -> Result<ProofBlob, ProofChkErr> {
        self.check_public_root_v1(expected_public_root)?;
        let shard_leaf = self.shard_leaf_v1()?;
        let proof_item = self.shard_proof.item().clone();
        let checked = chk_blob_settlement(
            &self.shard_proof.encode()?,
            SettlementStateRoot::settlement_v1(shard_leaf.shard_root),
            &proof_item.path(),
            proof_item.def_leaf(),
            proof_item.ser_leaf(),
            proof_item.leaf().clone(),
        )?;

        if checked.hjmt_journal_checkpoint() != Some(shard_leaf.journal_checkpoint) {
            return Err(ProofChkErr::PublicationProofCheckpointMix);
        }
        let Some(bucket_policy) = checked.hjmt_bucket_policy() else {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        };
        if u64::from(bucket_policy.compatibility_generation())
            != self.shard_context.policy_generation
            || bucket_policy.bucket_policy_id() != self.shard_context.bucket_policy_digest
        {
            return Err(ProofChkErr::PublicationProofPolicyMix);
        }
        Ok(checked)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofHeaderV1 {
    pub encoding_version: u8,
    pub transcript_domain: [u8; 32],
    pub proof_family: BatchProofFamilyTagV1,
    pub root_generation: RootGenerationTagV1,
    pub settlement_root: SettlementStateRoot,
    pub backend_root: [u8; 32],
    pub root_bind_version: u8,
    pub root_bind: [u8; 32],
    pub leaf_family_set: Vec<LeafFamilyTagV1>,
    pub bucket_bits: u8,
    pub bucket_id_width: u8,
    pub min_bucket_count: u32,
    pub max_target_leaf_count: u32,
    pub policy_generation: u64,
    pub bucket_policy_digest: [u8; 32],
    pub journal_checkpoint: Option<u64>,
    pub journal_digest: [u8; 32],
    pub checkpoint_bind: [u8; 32],
    pub batch_limits: BatchProofLimits,
}

impl BatchProofHeaderV1 {
    #[must_use]
    pub fn new(
        proof_family: BatchProofFamilyTagV1,
        settlement_root: SettlementStateRoot,
        backend_root: [u8; 32],
        leaf_family_set: Vec<LeafFamilyTagV1>,
        policy: BucketPolicy,
        journal_checkpoint: Option<u64>,
        journal_digest: [u8; 32],
        batch_limits: BatchProofLimits,
    ) -> Self {
        let checkpoint_bind = journal_checkpoint
            .map(|checkpoint| {
                checkpoint_bind_v1(settlement_root, backend_root, checkpoint, journal_digest)
            })
            .unwrap_or([0u8; 32]);
        Self {
            encoding_version: BATCH_PROOF_ENCODING_VERSION,
            transcript_domain: batch_proof_transcript_domain_v1(),
            proof_family,
            root_generation: RootGenerationTagV1::from_live(settlement_root),
            settlement_root,
            backend_root,
            root_bind_version: ROOT_BIND_VER,
            root_bind: root_bind_v1(settlement_root, backend_root),
            leaf_family_set,
            bucket_bits: policy.bucket_bits(),
            bucket_id_width: policy.bucket_id_width(),
            min_bucket_count: policy.min_bucket_count(),
            max_target_leaf_count: policy.max_target_leaf_count(),
            policy_generation: u64::from(policy.compatibility_generation()),
            bucket_policy_digest: policy.bucket_policy_id(),
            journal_checkpoint,
            journal_digest,
            checkpoint_bind,
            batch_limits,
        }
    }

    #[must_use]
    pub fn from_policy(
        proof_family: BatchProofFamilyTagV1,
        settlement_root: SettlementStateRoot,
        backend_root: [u8; 32],
        leaf_family_set: Vec<LeafFamilyTagV1>,
        policy: BucketPolicy,
        journal_checkpoint: Option<u64>,
        journal_digest: [u8; 32],
    ) -> Self {
        Self::new(
            proof_family,
            settlement_root,
            backend_root,
            leaf_family_set,
            policy,
            journal_checkpoint,
            journal_digest,
            BatchProofLimits::from_policy(policy),
        )
    }

    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.encoding_version);
        out.extend_from_slice(&self.transcript_domain);
        out.push(self.proof_family as u8);
        out.push(self.root_generation as u8);
        put_state_root(out, self.settlement_root);
        out.extend_from_slice(&self.backend_root);
        out.push(self.root_bind_version);
        out.extend_from_slice(&self.root_bind);
        put_u32(out, self.leaf_family_set.len() as u32);
        for family in &self.leaf_family_set {
            out.push(*family as u8);
        }
        out.push(self.bucket_bits);
        out.push(self.bucket_id_width);
        put_u32(out, self.min_bucket_count);
        put_u32(out, self.max_target_leaf_count);
        put_u64(out, self.policy_generation);
        out.extend_from_slice(&self.bucket_policy_digest);
        put_opt_u64(out, self.journal_checkpoint);
        out.extend_from_slice(&self.journal_digest);
        out.extend_from_slice(&self.checkpoint_bind);
        self.batch_limits.encode(out);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        let encoding_version = take_u8(bytes, pos)?;
        let transcript_domain = take_32(bytes, pos)?;
        let proof_family = BatchProofFamilyTagV1::decode(take_u8(bytes, pos)?)?;
        let root_generation = RootGenerationTagV1::decode(take_u8(bytes, pos)?)?;
        let settlement_root = take_state_root(bytes, pos)?;
        let backend_root = take_32(bytes, pos)?;
        let root_bind_version = take_u8(bytes, pos)?;
        let root_bind = take_32(bytes, pos)?;
        let leaf_count = take_count(bytes, pos, 2)?;
        let mut leaf_family_set = Vec::with_capacity(leaf_count);
        for _ in 0..leaf_count {
            leaf_family_set.push(LeafFamilyTagV1::decode(take_u8(bytes, pos)?)?);
        }
        let bucket_bits = take_u8(bytes, pos)?;
        let bucket_id_width = take_u8(bytes, pos)?;
        let min_bucket_count = take_u32(bytes, pos)?;
        let max_target_leaf_count = take_u32(bytes, pos)?;
        let policy_generation = take_u64(bytes, pos)?;
        let bucket_policy_digest = take_32(bytes, pos)?;
        let journal_checkpoint = take_opt_u64(bytes, pos)?;
        let journal_digest = take_32(bytes, pos)?;
        let checkpoint_bind = take_32(bytes, pos)?;
        let batch_limits = BatchProofLimits::decode(bytes, pos)?;
        Ok(Self {
            encoding_version,
            transcript_domain,
            proof_family,
            root_generation,
            settlement_root,
            backend_root,
            root_bind_version,
            root_bind,
            leaf_family_set,
            bucket_bits,
            bucket_id_width,
            min_bucket_count,
            max_target_leaf_count,
            policy_generation,
            bucket_policy_digest,
            journal_checkpoint,
            journal_digest,
            checkpoint_bind,
            batch_limits,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPathEntryV1 {
    pub path: SettlementPath,
    pub terminal_family: TerminalFamilyTagV1,
    pub leaf_family: LeafFamilyTagV1,
    pub shard_id: Option<u32>,
    pub routing_generation: Option<u64>,
    pub opening_index: u32,
    pub reference_index: u32,
}

impl BatchPathEntryV1 {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_path(out, self.path);
        out.push(self.terminal_family as u8);
        out.push(self.leaf_family as u8);
        put_opt_u32(out, self.shard_id);
        put_opt_u64(out, self.routing_generation);
        put_u32(out, self.opening_index);
        put_u32(out, self.reference_index);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        Ok(Self {
            path: take_path(bytes, pos)?,
            terminal_family: TerminalFamilyTagV1::decode(take_u8(bytes, pos)?)?,
            leaf_family: LeafFamilyTagV1::decode(take_u8(bytes, pos)?)?,
            shard_id: take_opt_u32(bytes, pos)?,
            routing_generation: take_opt_u64(bytes, pos)?,
            opening_index: take_u32(bytes, pos)?,
            reference_index: take_u32(bytes, pos)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessNodeV1 {
    pub tree_level: u16,
    pub node_domain: NodeDomainTagV1,
    pub child_index: u8,
    pub sibling_side: SiblingSideTagV1,
    pub subtree_marker: bool,
    pub hash_material: Vec<[u8; 32]>,
}

impl WitnessNodeV1 {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_u16(out, self.tree_level);
        out.push(self.node_domain as u8);
        out.push(self.child_index);
        out.push(self.sibling_side as u8);
        put_bool(out, self.subtree_marker);
        put_u32(out, self.hash_material.len() as u32);
        for hash in &self.hash_material {
            out.extend_from_slice(hash);
        }
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        let tree_level = take_u16(bytes, pos)?;
        let node_domain = NodeDomainTagV1::decode(take_u8(bytes, pos)?)?;
        let child_index = take_u8(bytes, pos)?;
        let sibling_side = SiblingSideTagV1::decode(take_u8(bytes, pos)?)?;
        let subtree_marker = take_bool(bytes, pos)?;
        let hash_material_count = take_u32(bytes, pos)?;
        if hash_material_count != 1 {
            return Err(ProofChkErr::BatchHashCountMix);
        }
        let mut hash_material = Vec::with_capacity(hash_material_count as usize);
        for _ in 0..hash_material_count {
            hash_material.push(take_32(bytes, pos)?);
        }
        Ok(Self {
            tree_level,
            node_domain,
            child_index,
            sibling_side,
            subtree_marker,
            hash_material,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpeningEntryV1 {
    pub opening_kind: OpeningKindTagV1,
    pub payload: Vec<u8>,
}

impl OpeningEntryV1 {
    #[must_use]
    pub fn from_inclusion(opening: InclusionOpeningV1) -> Self {
        Self {
            opening_kind: OpeningKindTagV1::InclusionLeaf,
            payload: opening.encode(),
        }
    }

    #[must_use]
    pub fn from_nonexistence(opening: NonExistenceOpeningV1) -> Self {
        Self {
            opening_kind: OpeningKindTagV1::AbsenceOpening,
            payload: opening.encode(),
        }
    }

    #[must_use]
    pub fn from_deletion(opening: DeletionFactV1) -> Self {
        Self {
            opening_kind: OpeningKindTagV1::DeletionFact,
            payload: opening.encode(),
        }
    }

    pub fn decode_inclusion(&self) -> Result<InclusionOpeningV1, ProofChkErr> {
        if self.opening_kind != OpeningKindTagV1::InclusionLeaf {
            return Err(ProofChkErr::BatchOpeningKindMix);
        }
        InclusionOpeningV1::decode(&self.payload)
    }

    pub fn decode_nonexistence(&self) -> Result<NonExistenceOpeningV1, ProofChkErr> {
        if self.opening_kind != OpeningKindTagV1::AbsenceOpening {
            return Err(ProofChkErr::BatchOpeningKindMix);
        }
        NonExistenceOpeningV1::decode(&self.payload)
    }

    pub fn decode_deletion(&self) -> Result<DeletionFactV1, ProofChkErr> {
        if self.opening_kind != OpeningKindTagV1::DeletionFact {
            return Err(ProofChkErr::BatchOpeningKindMix);
        }
        DeletionFactV1::decode(&self.payload)
    }

    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.opening_kind as u8);
        put_bytes(out, &self.payload);
    }

    pub(crate) fn decode(bytes: &[u8], pos: &mut usize) -> Result<Self, ProofChkErr> {
        let opening_kind = OpeningKindTagV1::decode(take_u8(bytes, pos)?)?;
        let payload = take_bytes(bytes, pos)?;
        Ok(Self {
            opening_kind,
            payload,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InclusionOpeningV1 {
    pub version: u8,
    pub leaf_bytes: Vec<u8>,
}

impl InclusionOpeningV1 {
    pub fn new(leaf: &SettlementLeaf) -> Result<Self, ProofChkErr> {
        Ok(Self {
            version: OPENING_VERSION_V1,
            leaf_bytes: leaf.encode()?,
        })
    }

    pub fn decode_leaf(&self) -> Result<SettlementLeaf, ProofChkErr> {
        Ok(SettlementLeaf::decode(&self.leaf_bytes)?)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_bytes(&mut out, &self.leaf_bytes);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let leaf_bytes = take_bytes(bytes, &mut pos)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != OPENING_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            leaf_bytes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonExistenceOpeningV1 {
    pub version: u8,
    pub marker_leaf_bytes: Vec<u8>,
    pub default_commitment_version: u8,
    pub default_commitment: [u8; 32],
    pub default_child_commitment: [u8; 32],
}

impl NonExistenceOpeningV1 {
    pub fn new(marker_leaf: &SettlementLeaf) -> Result<Self, ProofChkErr> {
        Ok(Self {
            version: OPENING_VERSION_V1,
            marker_leaf_bytes: marker_leaf.encode()?,
            default_commitment_version: HJMT_DEFAULT_COMMITMENT_VERSION,
            default_commitment: hjmt_default_value_commitment(),
            default_child_commitment: hjmt_default_child_commitment(),
        })
    }

    pub fn decode_marker_leaf(&self) -> Result<SettlementLeaf, ProofChkErr> {
        Ok(SettlementLeaf::decode(&self.marker_leaf_bytes)?)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_bytes(&mut out, &self.marker_leaf_bytes);
        out.push(self.default_commitment_version);
        out.extend_from_slice(&self.default_commitment);
        out.extend_from_slice(&self.default_child_commitment);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let marker_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let default_commitment_version = take_u8(bytes, &mut pos)?;
        let default_commitment = take_32(bytes, &mut pos)?;
        let default_child_commitment = take_32(bytes, &mut pos)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != OPENING_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            marker_leaf_bytes,
            default_commitment_version,
            default_commitment,
            default_child_commitment,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorProofContextV1 {
    pub version: u8,
    pub prior_hjmt_version: u64,
    pub prior_settlement_root: SettlementStateRoot,
    pub prior_backend_root: [u8; 32],
    pub prior_root_bind_version: u8,
    pub prior_root_bind: [u8; 32],
    pub prior_journal_digest: [u8; 32],
    pub prior_checkpoint_bind: [u8; 32],
    pub definition_root_leaf_bytes: Vec<u8>,
    pub serial_root_leaf_bytes: Vec<u8>,
    pub bucket_root_leaf_bytes: Vec<u8>,
    pub definition_proof_bytes: Vec<u8>,
    pub serial_proof_bytes: Vec<u8>,
    pub bucket_proof_bytes: Vec<u8>,
    pub prior_terminal_proof_bytes: Vec<u8>,
}

impl PriorProofContextV1 {
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_u64(&mut out, self.prior_hjmt_version);
        put_state_root(&mut out, self.prior_settlement_root);
        out.extend_from_slice(&self.prior_backend_root);
        out.push(self.prior_root_bind_version);
        out.extend_from_slice(&self.prior_root_bind);
        out.extend_from_slice(&self.prior_journal_digest);
        out.extend_from_slice(&self.prior_checkpoint_bind);
        put_bytes(&mut out, &self.definition_root_leaf_bytes);
        put_bytes(&mut out, &self.serial_root_leaf_bytes);
        put_bytes(&mut out, &self.bucket_root_leaf_bytes);
        put_bytes(&mut out, &self.definition_proof_bytes);
        put_bytes(&mut out, &self.serial_proof_bytes);
        put_bytes(&mut out, &self.bucket_proof_bytes);
        put_bytes(&mut out, &self.prior_terminal_proof_bytes);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let prior_hjmt_version = take_u64(bytes, &mut pos)?;
        let prior_settlement_root = take_state_root(bytes, &mut pos)?;
        let prior_backend_root = take_32(bytes, &mut pos)?;
        let prior_root_bind_version = take_u8(bytes, &mut pos)?;
        let prior_root_bind = take_32(bytes, &mut pos)?;
        let prior_journal_digest = take_32(bytes, &mut pos)?;
        let prior_checkpoint_bind = take_32(bytes, &mut pos)?;
        let definition_root_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let serial_root_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let bucket_root_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let definition_proof_bytes = take_bytes(bytes, &mut pos)?;
        let serial_proof_bytes = take_bytes(bytes, &mut pos)?;
        let bucket_proof_bytes = take_bytes(bytes, &mut pos)?;
        let prior_terminal_proof_bytes = take_bytes(bytes, &mut pos)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != PRIOR_CTX_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            prior_hjmt_version,
            prior_settlement_root,
            prior_backend_root,
            prior_root_bind_version,
            prior_root_bind,
            prior_journal_digest,
            prior_checkpoint_bind,
            definition_root_leaf_bytes,
            serial_root_leaf_bytes,
            bucket_root_leaf_bytes,
            definition_proof_bytes,
            serial_proof_bytes,
            bucket_proof_bytes,
            prior_terminal_proof_bytes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionFactV1 {
    pub version: u8,
    pub deleted_leaf_bytes: Vec<u8>,
    pub default_commitment_version: u8,
    pub default_child_commitment: [u8; 32],
    pub prior_context: PriorProofContextV1,
}

impl DeletionFactV1 {
    pub fn new(
        deleted_leaf: &SettlementLeaf,
        prior_context: PriorProofContextV1,
    ) -> Result<Self, ProofChkErr> {
        Ok(Self {
            version: DELETION_FACT_VERSION_V1,
            deleted_leaf_bytes: deleted_leaf.encode()?,
            default_commitment_version: HJMT_DEFAULT_COMMITMENT_VERSION,
            default_child_commitment: hjmt_default_child_commitment(),
            prior_context,
        })
    }

    pub fn decode_deleted_leaf(&self) -> Result<SettlementLeaf, ProofChkErr> {
        Ok(SettlementLeaf::decode(&self.deleted_leaf_bytes)?)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.version);
        put_bytes(&mut out, &self.deleted_leaf_bytes);
        out.push(self.default_commitment_version);
        out.extend_from_slice(&self.default_child_commitment);
        let prior = self.prior_context.encode();
        put_bytes(&mut out, &prior);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        let mut pos = 0;
        let version = take_u8(bytes, &mut pos)?;
        let deleted_leaf_bytes = take_bytes(bytes, &mut pos)?;
        let default_commitment_version = take_u8(bytes, &mut pos)?;
        let default_child_commitment = take_32(bytes, &mut pos)?;
        let prior_bytes = take_bytes(bytes, &mut pos)?;
        let prior_context = PriorProofContextV1::decode(&prior_bytes)?;
        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }
        if version != DELETION_FACT_VERSION_V1 {
            return Err(ProofChkErr::UnsupportedBatchProofVersion);
        }
        Ok(Self {
            version,
            deleted_leaf_bytes,
            default_commitment_version,
            default_child_commitment,
            prior_context,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathWitnessRefV1 {
    pub witness_indexes: Vec<u32>,
}

impl PathWitnessRefV1 {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        put_u32(out, self.witness_indexes.len() as u32);
        for index in &self.witness_indexes {
            put_u32(out, *index);
        }
    }

    pub(crate) fn decode(
        bytes: &[u8],
        pos: &mut usize,
        max_count: u32,
    ) -> Result<Self, ProofChkErr> {
        let count = take_count(bytes, pos, max_count)?;
        let mut witness_indexes = Vec::with_capacity(count);
        for _ in 0..count {
            witness_indexes.push(take_u32(bytes, pos)?);
        }
        Ok(Self { witness_indexes })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchProofBlobV1 {
    pub header: BatchProofHeaderV1,
    pub path_table: Vec<BatchPathEntryV1>,
    pub witness_dag: Vec<WitnessNodeV1>,
    pub opening_table: Vec<OpeningEntryV1>,
    pub reference_table: Vec<PathWitnessRefV1>,
}

impl BatchProofBlobV1 {
    #[must_use]
    pub fn new(
        header: BatchProofHeaderV1,
        path_table: Vec<BatchPathEntryV1>,
        witness_dag: Vec<WitnessNodeV1>,
        opening_table: Vec<OpeningEntryV1>,
        reference_table: Vec<PathWitnessRefV1>,
    ) -> Self {
        Self {
            header,
            path_table,
            witness_dag,
            opening_table,
            reference_table,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProofChkErr> {
        let mut out = Vec::new();
        self.header.encode(&mut out);
        put_u32(&mut out, self.path_table.len() as u32);
        for entry in &self.path_table {
            entry.encode(&mut out);
        }
        put_u32(&mut out, self.witness_dag.len() as u32);
        for node in &self.witness_dag {
            node.encode(&mut out);
        }
        put_u32(&mut out, self.opening_table.len() as u32);
        for entry in &self.opening_table {
            entry.encode(&mut out);
        }
        put_u32(&mut out, self.reference_table.len() as u32);
        for entry in &self.reference_table {
            entry.encode(&mut out);
        }
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProofChkErr> {
        Self::decode_with_limits(bytes, BatchProofLimits::v1())
    }

    pub fn decode_with_limits(bytes: &[u8], ceil: BatchProofLimits) -> Result<Self, ProofChkErr> {
        if bytes.len() > ceil.max_total_bytes as usize {
            return Err(ProofChkErr::BatchLimitMix);
        }
        let mut pos = 0;
        let header = BatchProofHeaderV1::decode(bytes, &mut pos)?;
        if !ceil.contains(header.batch_limits) {
            return Err(ProofChkErr::BatchLimitMix);
        }
        if bytes.len() > header.batch_limits.max_total_bytes as usize {
            return Err(ProofChkErr::BatchLimitMix);
        }
        let path_cap = header.batch_limits.max_path_count.min(ceil.max_path_count);
        let witness_cap = header
            .batch_limits
            .max_witness_node_count
            .min(ceil.max_witness_node_count);
        let opening_cap = header
            .batch_limits
            .max_opening_count
            .min(ceil.max_opening_count);
        let ref_cap = header
            .batch_limits
            .max_reference_count
            .min(ceil.max_reference_count);

        let path_count = take_count(bytes, &mut pos, path_cap)?;
        let mut path_table = Vec::with_capacity(path_count);
        for _ in 0..path_count {
            path_table.push(BatchPathEntryV1::decode(bytes, &mut pos)?);
        }

        let witness_count = take_count(bytes, &mut pos, witness_cap)?;
        let mut witness_dag = Vec::with_capacity(witness_count);
        for _ in 0..witness_count {
            witness_dag.push(WitnessNodeV1::decode(bytes, &mut pos)?);
        }

        let opening_count = take_count(bytes, &mut pos, opening_cap)?;
        let mut opening_table = Vec::with_capacity(opening_count);
        for _ in 0..opening_count {
            opening_table.push(OpeningEntryV1::decode(bytes, &mut pos)?);
        }

        let ref_count = take_count(bytes, &mut pos, ref_cap)?;
        let mut reference_table = Vec::with_capacity(ref_count);
        for _ in 0..ref_count {
            reference_table.push(PathWitnessRefV1::decode(bytes, &mut pos, witness_cap)?);
        }

        if pos != bytes.len() {
            return Err(ProofChkErr::BatchTrailingBytes);
        }

        let batch = Self {
            header,
            path_table,
            witness_dag,
            opening_table,
            reference_table,
        };
        batch.check_contract_v1()?;
        Ok(batch)
    }

    pub fn check_contract_v1(&self) -> Result<(), ProofChkErr> {
        super::proof_batch_verify::check_batch_contract_v1(self)
    }
}

#[derive(Serialize)]
struct CheckpointWitnessChunkV1 {
    version: u8,
    ordinal: u32,
    chunk_kind: u8,
    encoding_version: u8,
    byte_length: u32,
    content_digest: [u8; 32],
    linked_terminal_ids: Vec<super::TerminalId>,
}

impl CheckpointWitnessChunkV1 {
    fn canonical_bytes(&self) -> Result<Vec<u8>, ProofChkErr> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&frame_bytes(&[self.version]));
        bytes.extend_from_slice(&frame_bytes(&self.ordinal.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&[self.chunk_kind]));
        bytes.extend_from_slice(&frame_bytes(&[self.encoding_version]));
        bytes.extend_from_slice(&frame_bytes(&self.byte_length.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&self.content_digest));
        let linked_count = u32::try_from(self.linked_terminal_ids.len())
            .map_err(|_| ProofChkErr::BatchLimitMix)?;
        bytes.extend_from_slice(&frame_bytes(&linked_count.to_le_bytes()));
        for terminal_id in &self.linked_terminal_ids {
            bytes.extend_from_slice(&frame_bytes(terminal_id.as_bytes()));
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy)]
struct BatchCtxV1 {
    settlement_root: SettlementStateRoot,
    backend_root: [u8; 32],
    journal_checkpoint: u64,
    journal_digest: [u8; 32],
}

fn batch_ctx_v1(batches: &[BatchProofBlobV1]) -> Result<BatchCtxV1, ProofChkErr> {
    let mut ctx: Option<BatchCtxV1> = None;
    for batch in batches {
        batch.check_contract_v1()?;
        let checkpoint = batch
            .header
            .journal_checkpoint
            .ok_or(ProofChkErr::BatchCheckpointMix)?;
        let candidate = BatchCtxV1 {
            settlement_root: batch.header.settlement_root,
            backend_root: batch.header.backend_root,
            journal_checkpoint: checkpoint,
            journal_digest: batch.header.journal_digest,
        };
        if let Some(expect) = ctx {
            if candidate.settlement_root != expect.settlement_root
                || candidate.backend_root != expect.backend_root
            {
                return Err(ProofChkErr::BatchRootMix);
            }
            if candidate.journal_checkpoint != expect.journal_checkpoint
                || candidate.journal_digest != expect.journal_digest
            {
                return Err(ProofChkErr::BatchCheckpointMix);
            }
        } else {
            ctx = Some(candidate);
        }
    }
    ctx.ok_or(ProofChkErr::BatchLimitMix)
}

pub fn derive_journal_digest_v1(batches: &[BatchProofBlobV1]) -> Result<[u8; 32], ProofChkErr> {
    Ok(batch_ctx_v1(batches)?.journal_digest)
}

pub fn derive_witness_root_v1(batches: &[BatchProofBlobV1]) -> Result<[u8; 32], ProofChkErr> {
    batch_ctx_v1(batches)?;
    let batch_count = u32::try_from(batches.len()).map_err(|_| ProofChkErr::BatchLimitMix)?;
    let mut root_bytes = frame_bytes(&batch_count.to_le_bytes());
    for (ordinal, batch) in batches.iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| ProofChkErr::BatchLimitMix)?;
        let batch_bytes = batch.encode()?;
        let byte_length =
            u32::try_from(batch_bytes.len()).map_err(|_| ProofChkErr::BatchLimitMix)?;
        let content_digest =
            hash_zk::<StorBatchProofDom>(WITNESS_PAYLOAD_LABEL, &[batch_bytes.as_slice()]);
        let chunk = CheckpointWitnessChunkV1 {
            version: WITNESS_CHUNK_VER,
            ordinal,
            chunk_kind: WITNESS_CHUNK_BATCH,
            encoding_version: batch.header.encoding_version,
            byte_length,
            content_digest,
            linked_terminal_ids: batch
                .path_table
                .iter()
                .map(|entry| entry.path.terminal_id())
                .collect(),
        };
        let chunk_bytes = chunk.canonical_bytes()?;
        let chunk_hash =
            hash_zk::<StorBatchProofDom>(WITNESS_CHUNK_LABEL, &[chunk_bytes.as_slice()]);
        root_bytes.extend_from_slice(&frame_bytes(&chunk_hash));
    }
    Ok(hash_zk::<StorBatchProofDom>(
        WITNESS_ROOT_LABEL,
        &[root_bytes.as_slice()],
    ))
}

fn root_bind_v1(root: SettlementStateRoot, backend_root: [u8; 32]) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_root_bind_v1",
        &[&generation, &root_bytes, &backend_root],
    )
}

fn checkpoint_bind_v1(
    root: SettlementStateRoot,
    backend_root: [u8; 32],
    checkpoint: u64,
    journal_digest: [u8; 32],
) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    let checkpoint_bytes = checkpoint.to_be_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_hjmt_checkpoint_bind_v1",
        &[
            &generation,
            &root_bytes,
            &backend_root,
            &checkpoint_bytes,
            &journal_digest,
        ],
    )
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn put_bool(out: &mut Vec<u8>, value: bool) {
    out.push(u8::from(value));
}

fn put_opt_u32(out: &mut Vec<u8>, value: Option<u32>) {
    match value {
        Some(value) => {
            out.push(0x01);
            put_u32(out, value);
        }
        None => out.push(0x00),
    }
}

fn put_opt_u64(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            out.push(0x01);
            put_u64(out, value);
        }
        None => out.push(0x00),
    }
}

fn put_bytes(out: &mut Vec<u8>, value: &[u8]) {
    put_u32(out, value.len() as u32);
    out.extend_from_slice(value);
}

fn put_state_root(out: &mut Vec<u8>, root: SettlementStateRoot) {
    out.push(root.generation_version());
    out.extend_from_slice(root.as_bytes());
}

fn put_path(out: &mut Vec<u8>, path: SettlementPath) {
    out.extend_from_slice(path.definition_id.as_bytes());
    put_u32(out, path.serial_id.get());
    out.extend_from_slice(path.terminal_id.as_bytes());
}

fn take_u8(bytes: &[u8], pos: &mut usize) -> Result<u8, ProofChkErr> {
    let Some(value) = bytes.get(*pos).copied() else {
        return Err(ProofChkErr::BatchTrunc);
    };
    *pos += 1;
    Ok(value)
}

fn take_bool(bytes: &[u8], pos: &mut usize) -> Result<bool, ProofChkErr> {
    match take_u8(bytes, pos)? {
        0x00 => Ok(false),
        0x01 => Ok(true),
        _ => Err(ProofChkErr::BatchBoolMix),
    }
}

fn take_u16(bytes: &[u8], pos: &mut usize) -> Result<u16, ProofChkErr> {
    let end = pos.saturating_add(2);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 2];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u16::from_be_bytes(raw))
}

fn take_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, ProofChkErr> {
    let end = pos.saturating_add(4);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u32::from_be_bytes(raw))
}

fn take_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, ProofChkErr> {
    let end = pos.saturating_add(8);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u64::from_be_bytes(raw))
}

fn take_opt_u32(bytes: &[u8], pos: &mut usize) -> Result<Option<u32>, ProofChkErr> {
    match take_u8(bytes, pos)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(take_u32(bytes, pos)?)),
        _ => Err(ProofChkErr::BatchBoolMix),
    }
}

fn take_opt_u64(bytes: &[u8], pos: &mut usize) -> Result<Option<u64>, ProofChkErr> {
    match take_u8(bytes, pos)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(take_u64(bytes, pos)?)),
        _ => Err(ProofChkErr::BatchBoolMix),
    }
}

fn take_32(bytes: &[u8], pos: &mut usize) -> Result<[u8; 32], ProofChkErr> {
    let end = pos.saturating_add(32);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(raw)
}

fn take_count(bytes: &[u8], pos: &mut usize, max: u32) -> Result<usize, ProofChkErr> {
    let count = take_u32(bytes, pos)?;
    if count > max {
        return Err(ProofChkErr::BatchLimitMix);
    }
    Ok(count as usize)
}

fn take_bytes(bytes: &[u8], pos: &mut usize) -> Result<Vec<u8>, ProofChkErr> {
    let len = take_u32(bytes, pos)? as usize;
    let end = pos.saturating_add(len);
    if end > bytes.len() {
        return Err(ProofChkErr::BatchTrunc);
    }
    let out = bytes[*pos..end].to_vec();
    *pos = end;
    Ok(out)
}

fn take_state_root(bytes: &[u8], pos: &mut usize) -> Result<SettlementStateRoot, ProofChkErr> {
    let version = take_u8(bytes, pos)?;
    let root = take_32(bytes, pos)?;
    SettlementStateRoot::from_version(version, root).ok_or(ProofChkErr::BatchRootGenerationMix)
}

fn take_path(bytes: &[u8], pos: &mut usize) -> Result<SettlementPath, ProofChkErr> {
    let definition_id = super::DefinitionId::new(take_32(bytes, pos)?);
    let serial_id = super::SerialId::new(take_u32(bytes, pos)?);
    let terminal_id = super::TerminalId::new(take_32(bytes, pos)?);
    Ok(SettlementPath::new(definition_id, serial_id, terminal_id))
}

fn take_pub_u8(bytes: &[u8], pos: &mut usize) -> Result<u8, ProofChkErr> {
    let Some(value) = bytes.get(*pos).copied() else {
        return Err(ProofChkErr::PublicationTrunc);
    };
    *pos += 1;
    Ok(value)
}

fn take_pub_u32(bytes: &[u8], pos: &mut usize) -> Result<u32, ProofChkErr> {
    let end = pos.saturating_add(4);
    if end > bytes.len() {
        return Err(ProofChkErr::PublicationTrunc);
    }
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u32::from_be_bytes(raw))
}

fn take_pub_u64(bytes: &[u8], pos: &mut usize) -> Result<u64, ProofChkErr> {
    let end = pos.saturating_add(8);
    if end > bytes.len() {
        return Err(ProofChkErr::PublicationTrunc);
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(u64::from_be_bytes(raw))
}

fn take_pub_opt_u64(bytes: &[u8], pos: &mut usize) -> Result<Option<u64>, ProofChkErr> {
    match take_pub_u8(bytes, pos)? {
        0x00 => Ok(None),
        0x01 => Ok(Some(take_pub_u64(bytes, pos)?)),
        _ => Err(ProofChkErr::PublicationPolicyMix),
    }
}

fn take_pub_32(bytes: &[u8], pos: &mut usize) -> Result<[u8; 32], ProofChkErr> {
    let end = pos.saturating_add(32);
    if end > bytes.len() {
        return Err(ProofChkErr::PublicationTrunc);
    }
    let mut raw = [0u8; 32];
    raw.copy_from_slice(&bytes[*pos..end]);
    *pos = end;
    Ok(raw)
}

fn take_pub_state_root(bytes: &[u8], pos: &mut usize) -> Result<SettlementStateRoot, ProofChkErr> {
    let version = take_pub_u8(bytes, pos)?;
    let root = take_pub_32(bytes, pos)?;
    SettlementStateRoot::from_version(version, root)
        .ok_or(ProofChkErr::PublicationRootGenerationMix)
}
