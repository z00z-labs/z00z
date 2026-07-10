use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use z00z_crypto::{expert::hash_domain, frame_bytes, hash_zk::hash_zk};

use crate::{
    checkpoint::{
        check_exec_root, check_link_ids, CheckpointDraft, CheckpointExecInput, CheckpointLink,
        CheckpointTransitionStatementCoreV1, CheckpointVersion, CreatedEnt, SpentEnt,
    },
    settlement::{
        chk_blob_settlement_inclusion, derive_journal_digest_v1, derive_witness_root_v1,
        BatchProofBlobV1, CheckRoot, ClaimSourceRoot, ProofChkErr, ProofItem, SettlementPath,
        SettlementStateRoot, StoreItem, TerminalId, TerminalLeaf,
    },
    snapshot::{PrepReplayEntry, PrepSnapshot, PrepSnapshotId},
    CheckpointError,
};
use z00z_crypto::CLAIM_ROOT_VERSION;

use super::build_prepare::prepare_tx_sum;
use super::build_state::{proof_root, BuildIdx, BuildState};

hash_domain!(StorCheckpointReplayDom, "z00z.storage.checkpoint.replay", 1);

const DELTA_RECORD_LABEL: &str = "checkpoint_delta_record_v1";
const DELTA_ROOT_LABEL: &str = "checkpoint_delta_root_v1";
const DELTA_SPENT_LABEL: &str = "checkpoint_delta_spent_v1";
const DELTA_CREATED_LABEL: &str = "checkpoint_delta_created_v1";
const DELTA_RECORD_VER: u8 = 1;
const DELTA_KIND_SPENT: u8 = 1;
const DELTA_KIND_CREATED: u8 = 2;

#[derive(Serialize)]
struct CheckpointDeltaRecordV1 {
    version: u8,
    tx_ordinal: u32,
    item_ordinal: u32,
    delta_kind: u8,
    terminal_id: TerminalId,
    payload_digest: [u8; 32],
}

impl CheckpointDeltaRecordV1 {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&frame_bytes(&[self.version]));
        bytes.extend_from_slice(&frame_bytes(&self.tx_ordinal.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&self.item_ordinal.to_le_bytes()));
        bytes.extend_from_slice(&frame_bytes(&[self.delta_kind]));
        bytes.extend_from_slice(&frame_bytes(self.terminal_id.as_bytes()));
        bytes.extend_from_slice(&frame_bytes(&self.payload_digest));
        bytes
    }
}

fn spent_payload_bytes(spent: &SpentEnt) -> Vec<u8> {
    frame_bytes(spent.terminal_id().as_bytes())
}

fn created_payload_bytes(created: &CreatedEnt) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&frame_bytes(created.terminal_id().as_bytes()));
    bytes.extend_from_slice(&frame_bytes(created.leaf_hash()));
    bytes
}

fn map_batch_err(err: ProofChkErr) -> CheckpointError {
    match err {
        ProofChkErr::BatchRootMix
        | ProofChkErr::BatchRootGenerationMix
        | ProofChkErr::RootMix
        | ProofChkErr::RootGenerationMix => CheckpointError::RootMix,
        _ => CheckpointError::ReplayMix,
    }
}

pub fn derive_delta_root_v1(
    exec: &CheckpointExecInput,
    draft: &CheckpointDraft,
) -> Result<[u8; 32], CheckpointError> {
    if exec.txs().is_empty() {
        return Err(CheckpointError::ReplayMix);
    }

    let spent_expected = exec
        .txs()
        .iter()
        .try_fold(0usize, |sum, tx| sum.checked_add(tx.input_refs().len()))
        .ok_or(CheckpointError::ReplayMix)?;
    let created_expected = exec
        .txs()
        .iter()
        .try_fold(0usize, |sum, tx| sum.checked_add(tx.outputs().len()))
        .ok_or(CheckpointError::ReplayMix)?;
    if spent_expected != draft.spent_delta().len()
        || created_expected != draft.created_delta().len()
    {
        return Err(CheckpointError::ReplayMix);
    }

    let record_count = spent_expected
        .checked_add(created_expected)
        .ok_or(CheckpointError::ReplayMix)?;
    let record_count = u32::try_from(record_count).map_err(|_| CheckpointError::ReplayMix)?;
    let mut root_bytes = frame_bytes(&record_count.to_le_bytes());
    let mut spent_idx = 0usize;
    let mut created_idx = 0usize;

    for (tx_ordinal, tx) in exec.txs().iter().enumerate() {
        let tx_ordinal = u32::try_from(tx_ordinal).map_err(|_| CheckpointError::ReplayMix)?;
        for (item_ordinal, input_ref) in tx.input_refs().iter().enumerate() {
            let item_ordinal =
                u32::try_from(item_ordinal).map_err(|_| CheckpointError::ReplayMix)?;
            let spent = draft
                .spent_delta()
                .get(spent_idx)
                .ok_or(CheckpointError::ReplayMix)?;
            if spent.terminal_id() != input_ref.terminal_id() {
                return Err(CheckpointError::ReplayMix);
            }
            let spent_bytes = spent_payload_bytes(spent);
            let payload_digest =
                hash_zk::<StorCheckpointReplayDom>(DELTA_SPENT_LABEL, &[spent_bytes.as_slice()]);
            let record = CheckpointDeltaRecordV1 {
                version: DELTA_RECORD_VER,
                tx_ordinal,
                item_ordinal,
                delta_kind: DELTA_KIND_SPENT,
                terminal_id: spent.terminal_id(),
                payload_digest,
            };
            let record_bytes = record.canonical_bytes();
            let record_hash =
                hash_zk::<StorCheckpointReplayDom>(DELTA_RECORD_LABEL, &[record_bytes.as_slice()]);
            root_bytes.extend_from_slice(&frame_bytes(&record_hash));
            spent_idx += 1;
        }

        for (item_ordinal, output) in tx.outputs().iter().enumerate() {
            let item_ordinal =
                u32::try_from(item_ordinal).map_err(|_| CheckpointError::ReplayMix)?;
            let created = draft
                .created_delta()
                .get(created_idx)
                .ok_or(CheckpointError::ReplayMix)?;
            let terminal_id = output.leaf().terminal_id();
            if created.terminal_id() != terminal_id {
                return Err(CheckpointError::ReplayMix);
            }
            let created_bytes = created_payload_bytes(created);
            let payload_digest = hash_zk::<StorCheckpointReplayDom>(
                DELTA_CREATED_LABEL,
                &[created_bytes.as_slice()],
            );
            let record = CheckpointDeltaRecordV1 {
                version: DELTA_RECORD_VER,
                tx_ordinal,
                item_ordinal,
                delta_kind: DELTA_KIND_CREATED,
                terminal_id,
                payload_digest,
            };
            let record_bytes = record.canonical_bytes();
            let record_hash =
                hash_zk::<StorCheckpointReplayDom>(DELTA_RECORD_LABEL, &[record_bytes.as_slice()]);
            root_bytes.extend_from_slice(&frame_bytes(&record_hash));
            created_idx += 1;
        }
    }

    if spent_idx != draft.spent_delta().len() || created_idx != draft.created_delta().len() {
        return Err(CheckpointError::ReplayMix);
    }

    Ok(hash_zk::<StorCheckpointReplayDom>(
        DELTA_ROOT_LABEL,
        &[root_bytes.as_slice()],
    ))
}

pub fn build_stmt_core_v1(
    draft: &CheckpointDraft,
    exec: &CheckpointExecInput,
    batches: &[BatchProofBlobV1],
) -> Result<CheckpointTransitionStatementCoreV1, CheckpointError> {
    let first_batch = batches.first().ok_or(CheckpointError::ReplayMix)?;
    if first_batch.header.settlement_root != draft.new_settlement_root() {
        return Err(CheckpointError::RootMix);
    }

    let delta_root = derive_delta_root_v1(exec, draft)?;
    let witness_root = derive_witness_root_v1(batches).map_err(map_batch_err)?;
    let journal_digest = derive_journal_digest_v1(batches).map_err(map_batch_err)?;

    Ok(CheckpointTransitionStatementCoreV1::from_exec(
        exec,
        delta_root,
        witness_root,
        journal_digest,
    ))
}

/// Canonical membership witness captured during pre-state resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberWit {
    proof: Vec<u8>,
    proof_item: ProofItem,
}

impl MemberWit {
    /// Build one thin storage witness wrapper over canonical proof bytes.
    pub fn new(proof: Vec<u8>, proof_item: ProofItem) -> Result<Self, StateError> {
        let wit = Self { proof, proof_item };
        let path = wit.proof_item.path();
        let leaf = wit
            .proof_item
            .terminal_leaf()
            .map_err(|_| StateError::BadMember)?
            .clone();
        wit.check(wit.proof_root(), &path, &leaf)?;
        Ok(wit)
    }

    #[must_use]
    pub fn proof(&self) -> &[u8] {
        &self.proof
    }

    #[must_use]
    pub fn proof_item(&self) -> &ProofItem {
        &self.proof_item
    }

    #[must_use]
    pub fn proof_root(&self) -> SettlementStateRoot {
        self.proof_item.root()
    }

    #[must_use]
    pub fn proof_settlement_root(&self) -> SettlementStateRoot {
        self.proof_item.settlement_root()
    }

    pub(super) fn check(
        &self,
        root: SettlementStateRoot,
        path: &SettlementPath,
        leaf: &TerminalLeaf,
    ) -> Result<(), StateError> {
        let item = self.proof_item();
        chk_blob_settlement_inclusion(
            &self.proof,
            root,
            path,
            item.def_leaf(),
            item.ser_leaf(),
            leaf,
        )
        .map(|_| ())
        .map_err(|_| StateError::BadMember)
    }
}

/// Path-bound pre-state input captured before checkpoint apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedInput {
    path: SettlementPath,
    leaf: TerminalLeaf,
    member_wit: MemberWit,
}

impl ResolvedInput {
    /// Build one path-bound resolved input.
    pub fn new(
        path: SettlementPath,
        leaf: TerminalLeaf,
        member_wit: MemberWit,
    ) -> Result<Self, StateError> {
        StoreItem::new(path, leaf.clone()).map_err(|_| StateError::LeafMatch)?;
        member_wit.check(member_wit.proof_root(), &path, &leaf)?;
        Ok(Self {
            path,
            leaf,
            member_wit,
        })
    }

    #[must_use]
    pub const fn path(&self) -> SettlementPath {
        self.path
    }

    #[must_use]
    pub fn leaf(&self) -> &TerminalLeaf {
        &self.leaf
    }

    #[must_use]
    pub fn member_wit(&self) -> &MemberWit {
        &self.member_wit
    }

    #[must_use]
    pub const fn terminal_id(&self) -> TerminalId {
        self.path.terminal_id()
    }

    #[must_use]
    pub const fn serial_id(&self) -> u32 {
        self.path.serial_id.get()
    }
}

/// Minimal tx package summary for deterministic checkpoint apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxPkgSum {
    pub prev_root: CheckRoot,
    pub resolved_inputs: Vec<ResolvedInput>,
    pub outputs: Vec<TerminalLeaf>,
    pub tx_proof: Vec<u8>,
}

impl TxPkgSum {
    #[must_use]
    pub fn input_terminal_ids(&self) -> Vec<TerminalId> {
        self.resolved_inputs
            .iter()
            .map(ResolvedInput::terminal_id)
            .collect()
    }
}

/// Tx proof verifier error contract.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TxProofError {
    #[error("invalid tx proof")]
    Invalid,
    #[error("unsupported tx proof version")]
    Version,
}

/// Spent-index error contract.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SpentIndexError {
    #[error("spent index lookup failed")]
    Lookup,
}

/// Deterministic state-update failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StateError {
    #[error("tx batch must be non-empty")]
    EmptyBatch,
    #[error("tx prev_root mismatch")]
    PrevRoot,
    #[error("tx inputs must be non-empty")]
    EmptyInputs,
    #[error("tx outputs must be non-empty")]
    EmptyOutputs,
    #[error("duplicate input in tx package")]
    DupInput,
    #[error("duplicate output terminal id")]
    DupOut,
    #[error("missing input in state")]
    MissingInput,
    #[error("malformed input reference")]
    BadInputRef,
    #[error("input leaf-match failed")]
    LeafMatch,
    #[error("invalid membership witness")]
    BadMember,
    #[error("invalid resolved input")]
    BadResolve,
    #[error("input spent in interval")]
    SpentAfter,
    #[error("input spent in batch")]
    SpentBatch,
    #[error("state backend failure: {0}")]
    State(String),
    #[error(transparent)]
    TxProof(#[from] TxProofError),
    #[error(transparent)]
    SpentIndex(#[from] SpentIndexError),
}

/// Settlement state hook used by checkpoint apply.
pub trait SettlementState {
    fn root(&self) -> CheckRoot;

    fn get_leaf(&self, id: &TerminalId) -> Result<Option<TerminalLeaf>, StateError>;

    fn del_leaf(&mut self, id: &TerminalId) -> Result<(), StateError>;

    fn put_leaf(&mut self, leaf: TerminalLeaf) -> Result<(), StateError>;

    fn leaf_hash(&self, leaf: &TerminalLeaf) -> Result<[u8; 32], StateError>;
}

/// Tx proof verifier hook.
///
/// This remains an external trust boundary: a checkpoint draft is only as strong
/// as the verifier implementation supplied by the caller.
pub trait TxProofVerifier {
    fn verify_tx(&self, tx: &TxPkgSum) -> Result<(), TxProofError>;
}

/// Spent-delta interval index hook.
///
/// Production callers must supply a spent-index implementation with the same
/// trust level they expect from the finalized checkpoint artifact path.
pub trait SpentIndex {
    fn is_spent(
        &self,
        prev: CheckRoot,
        curr: CheckRoot,
        id: &TerminalId,
    ) -> Result<bool, SpentIndexError>;
}

/// Membership witness source used during checkpoint preparation.
pub trait MemberIndex {
    fn get_wit(
        &self,
        prev_root: CheckRoot,
        id: &TerminalId,
    ) -> Result<Option<MemberWit>, StateError>;
}

/// Dedicated pre-state resolver for one compact tx input.
pub trait InputResolver {
    fn resolve(
        &self,
        prev_root: CheckRoot,
        terminal_id: TerminalId,
        serial_id: u32,
    ) -> Result<ResolvedInput, StateError>;
}

/// Apply tx batch and return one proofless checkpoint draft.
pub fn apply_batch_checkpoint(
    height: u64,
    state: &mut impl SettlementState,
    txs: &[TxPkgSum],
    proof_chk: &impl TxProofVerifier,
    spent_idx: &impl SpentIndex,
) -> Result<CheckpointDraft, StateError> {
    if txs.is_empty() {
        return Err(StateError::EmptyBatch);
    }

    let prev_root = state.root();
    let mut spent_delta = Vec::new();
    let mut created_delta = Vec::new();
    let mut out_seen = BTreeSet::new();
    let mut spent_seen = BTreeSet::new();

    for tx in txs {
        if tx.prev_root != prev_root {
            return Err(StateError::PrevRoot);
        }
        if tx.resolved_inputs.is_empty() {
            return Err(StateError::EmptyInputs);
        }
        if tx.outputs.is_empty() {
            return Err(StateError::EmptyOutputs);
        }

        let mut seen = BTreeSet::new();
        for resolved in &tx.resolved_inputs {
            if !seen.insert(resolved.terminal_id()) {
                return Err(StateError::DupInput);
            }
        }

        for resolved in &tx.resolved_inputs {
            if StoreItem::new(resolved.path(), resolved.leaf.clone()).is_err()
                || resolved.member_wit().proof().is_empty()
                || resolved.member_wit().proof_root() != proof_root(tx.prev_root)
                || resolved.member_wit().proof_settlement_root()
                    != SettlementStateRoot::settlement_v1(tx.prev_root.into_bytes())
                || resolved.member_wit().proof_item().path() != resolved.path()
                || resolved.member_wit().proof_item().leaf()
                    != &crate::settlement::SettlementLeaf::from(resolved.leaf.clone())
            {
                return Err(StateError::BadResolve);
            }
        }

        proof_chk.verify_tx(tx)?;

        for resolved in &tx.resolved_inputs {
            let id = resolved.terminal_id();
            if !spent_seen.insert(id) {
                return Err(StateError::SpentBatch);
            }
            let state_leaf = state.get_leaf(&id)?.ok_or(StateError::MissingInput)?;
            if state_leaf != *resolved.leaf() {
                return Err(StateError::BadResolve);
            }
            if spent_idx.is_spent(tx.prev_root, prev_root, &id)? {
                return Err(StateError::SpentAfter);
            }

            state.del_leaf(&id)?;
            spent_delta.push(SpentEnt::new(id));
        }

        for out in &tx.outputs {
            let terminal_id = out.terminal_id();
            if !out_seen.insert(terminal_id) {
                return Err(StateError::DupOut);
            }
            if state.get_leaf(&terminal_id)?.is_some() {
                return Err(StateError::DupOut);
            }
            let hash = state.leaf_hash(out)?;
            state.put_leaf(out.clone())?;
            created_delta.push(CreatedEnt::new(terminal_id, hash));
        }
    }

    let new_root = state.root();
    Ok(CheckpointDraft::new_settlement(
        CheckpointVersion::CURRENT,
        height,
        SettlementStateRoot::settlement_v1(prev_root.into_bytes()),
        SettlementStateRoot::settlement_v1(new_root.into_bytes()),
        spent_delta,
        created_delta,
    )
    .with_claim_root(ClaimSourceRoot::new_settlement(
        CLAIM_ROOT_VERSION,
        SettlementStateRoot::settlement_v1(new_root.into_bytes()),
    )))
}

/// Build one canonical checkpoint draft from validated snapshot replay and execution input.
///
/// This function validates through external verifier and spent-index hooks, but
/// it does not by itself upgrade the resulting draft into a self-sufficient
/// proof artifact.
pub fn build_cp_draft(
    height: u64,
    snap_id: PrepSnapshotId,
    snapshot: &PrepSnapshot,
    replay: &[PrepReplayEntry],
    link: &CheckpointLink,
    exec: &CheckpointExecInput,
    proof_chk: &impl TxProofVerifier,
    spent_idx: &impl SpentIndex,
) -> Result<CheckpointDraft, StateError> {
    check_exec_root(snapshot, exec).map_err(|err| match err {
        crate::CheckpointError::RootMix => StateError::PrevRoot,
        other => StateError::State(other.to_string()),
    })?;
    check_link_ids(snap_id, link, exec).map_err(|err| StateError::State(err.to_string()))?;

    let mut state = BuildState::new(snapshot, exec)?;
    let resolver = BuildIdx::new(exec.prev_root(), replay)?;
    let mut txs = Vec::with_capacity(exec.txs().len());
    for tx in exec.txs() {
        let outputs = tx
            .outputs()
            .iter()
            .map(|item| item.leaf().clone())
            .collect::<Vec<_>>();
        txs.push(prepare_tx_sum(
            exec.prev_root(),
            &resolver,
            tx.input_refs(),
            &outputs,
            tx.tx_proof(),
        )?);
    }

    apply_batch_checkpoint(height, &mut state, &txs, proof_chk, spent_idx)
}
