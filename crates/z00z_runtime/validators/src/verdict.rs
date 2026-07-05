#![forbid(unsafe_code)]

use z00z_aggregators::{
    BatchId, CommitSubject, OrderedBatch, PublicationBinding, PublishedBatch,
    RuntimeObjectPackageV1, ShardExecTicket, ShardPlacementView, ShardQuorumCertificate,
};
use z00z_crypto::{expert::traits::DomainSeparation, DomainHasher256};
use z00z_storage::settlement::{ClaimNullifier, ObjectRejectCode, ObjectValidatorVerdict};
use z00z_storage::{
    checkpoint::{
        derive_checkpoint_id, derive_exec_id, encode_exec_bin, encode_link_bin, CheckpointArtifact,
        CheckpointExecInput, CheckpointExecOut, CheckpointExecTx, CheckpointId, CheckpointInRef,
        CheckpointLink, CheckpointStatement,
    },
    settlement::{CheckRoot, DefinitionId, SerialId},
};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::tx::{
    asset_wire_to_leaf, build_tx_package_digest, verify_package_public_spend_contract,
    TxOutputWire, TxPackage, TxVerifier, TxVerifierImpl,
};

const SETTLEMENT_THEOREM_DIGEST_TAG: &[u8] = b"z00z.settlement_theorem.bundle";

struct SettlementTheoremDigestDomain;

impl DomainSeparation for SettlementTheoremDigestDomain {
    fn version() -> u8 {
        1
    }

    fn domain() -> &'static str {
        "z00z.settlement_theorem.digest"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettlementTheoremBundle {
    tx_package: TxPackage,
    artifact: CheckpointArtifact,
    exec_input: CheckpointExecInput,
    link: CheckpointLink,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettlementError {
    #[error("transaction package theorem check failed: {0}")]
    TxTheorem(String),
    #[error("checkpoint artifact lacks current checkpoint statement")]
    CheckpointStatement,
    #[error("checkpoint proof payload mismatch")]
    CheckpointProof,
    #[error("checkpoint link mismatch")]
    CheckpointLink,
    #[error("checkpoint execution replay mismatch")]
    CheckpointReplay,
    #[error("checkpoint root mismatch")]
    CheckpointRoot,
    #[error("transaction package is not included in checkpoint execution")]
    TxMissing,
    #[error("settlement encoding failed: {0}")]
    Codec(String),
}

pub struct SettlementTheorem<'a> {
    pub tx_package: &'a TxPackage,
    pub artifact: &'a CheckpointArtifact,
    pub exec_input: &'a CheckpointExecInput,
    pub link: &'a CheckpointLink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedBatch {
    pub published: PublishedBatch,
    pub ordered: OrderedBatch,
    pub theorem: SettlementTheoremBundle,
    pub subject: Option<CommitSubject>,
    pub certificate: Option<ShardQuorumCertificate>,
    pub nullifiers: Vec<ClaimNullifier>,
    pub placement: Option<ShardPlacementView>,
    pub exec_ticket: Option<ShardExecTicket>,
}

fn runtime_placement_parts<'a>(
    placement: Option<&'a ShardPlacementView>,
    exec_ticket: Option<&'a ShardExecTicket>,
) -> Option<&'a ShardPlacementView> {
    exec_ticket
        .as_ref()
        .map(|ticket| &ticket.placement)
        .or(placement)
}

fn runtime_exec_parts(exec_ticket: Option<&ShardExecTicket>) -> Option<&ShardExecTicket> {
    exec_ticket
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
#[doc(hidden)]
pub fn kani_runtime_placement_parts<'a>(
    placement: Option<&'a ShardPlacementView>,
    exec_ticket: Option<&'a ShardExecTicket>,
) -> Option<&'a ShardPlacementView> {
    runtime_placement_parts(placement, exec_ticket)
}

#[allow(unexpected_cfgs)]
#[cfg(kani)]
#[doc(hidden)]
pub fn kani_runtime_exec_parts(exec_ticket: Option<&ShardExecTicket>) -> Option<&ShardExecTicket> {
    runtime_exec_parts(exec_ticket)
}

impl SettlementTheoremBundle {
    pub fn new(
        tx_package: TxPackage,
        artifact: CheckpointArtifact,
        exec_input: CheckpointExecInput,
        link: CheckpointLink,
    ) -> Result<Self, SettlementError> {
        let bundle = Self {
            tx_package,
            artifact,
            exec_input,
            link,
        };
        verify_settlement_theorem(&bundle.theorem())?;
        Ok(bundle)
    }

    #[must_use]
    pub fn theorem(&self) -> SettlementTheorem<'_> {
        SettlementTheorem {
            tx_package: &self.tx_package,
            artifact: &self.artifact,
            exec_input: &self.exec_input,
            link: &self.link,
        }
    }

    #[must_use]
    pub fn tx_package(&self) -> &TxPackage {
        &self.tx_package
    }

    #[must_use]
    pub fn artifact(&self) -> &CheckpointArtifact {
        &self.artifact
    }

    #[must_use]
    pub fn exec_input(&self) -> &CheckpointExecInput {
        &self.exec_input
    }

    #[must_use]
    pub fn link(&self) -> &CheckpointLink {
        &self.link
    }

    #[must_use]
    pub fn theorem_digest(&self) -> [u8; 32] {
        digest_settlement_theorem(&self.theorem())
    }
}

impl ResolvedBatch {
    #[must_use]
    pub fn new(
        published: PublishedBatch,
        ordered: OrderedBatch,
        theorem: SettlementTheoremBundle,
        subject: Option<CommitSubject>,
        certificate: Option<ShardQuorumCertificate>,
        nullifiers: Vec<ClaimNullifier>,
        placement: Option<ShardPlacementView>,
        exec_ticket: Option<ShardExecTicket>,
    ) -> Self {
        Self {
            published,
            ordered,
            theorem,
            subject,
            certificate,
            nullifiers,
            placement,
            exec_ticket,
        }
    }

    #[must_use]
    pub fn runtime_placement(&self) -> Option<&ShardPlacementView> {
        runtime_placement_parts(self.placement.as_ref(), self.exec_ticket.as_ref())
    }

    #[must_use]
    pub fn runtime_exec(&self) -> Option<&ShardExecTicket> {
        runtime_exec_parts(self.exec_ticket.as_ref())
    }

    #[must_use]
    pub fn tx_package(&self) -> &TxPackage {
        self.theorem.tx_package()
    }

    #[must_use]
    pub fn artifact(&self) -> &CheckpointArtifact {
        self.theorem.artifact()
    }

    #[must_use]
    pub fn exec_input(&self) -> &CheckpointExecInput {
        self.theorem.exec_input()
    }

    #[must_use]
    pub fn link(&self) -> &CheckpointLink {
        self.theorem.link()
    }

    pub fn object_packages(&self) -> impl Iterator<Item = &RuntimeObjectPackageV1> {
        self.ordered.object_packages()
    }

    #[must_use]
    pub fn theorem_digest(&self) -> [u8; 32] {
        self.theorem.theorem_digest()
    }

    #[must_use]
    pub fn quorum_binding_enabled(&self) -> bool {
        self.published.quorum_binding_enabled()
            || self.subject.is_some()
            || self.certificate.is_some()
    }
}

pub fn verify_settlement_theorem(theorem: &SettlementTheorem<'_>) -> Result<(), SettlementError> {
    verify_tx_package(theorem.tx_package)?;
    let stmt = match theorem.artifact.statement() {
        CheckpointStatement::CURRENT(stmt) => stmt,
        CheckpointStatement::Detached => return Err(SettlementError::CheckpointStatement),
    };

    if theorem.artifact.cp_proof() != stmt.backend_payload().as_slice() {
        return Err(SettlementError::CheckpointProof);
    }

    let exec_bytes = encode_exec_bin(theorem.exec_input)
        .map_err(|err| SettlementError::Codec(err.to_string()))?;
    let exec_id = derive_exec_id(&exec_bytes);
    if stmt.exec_input_id() != exec_id || theorem.link.exec_input_id() != exec_id {
        return Err(SettlementError::CheckpointReplay);
    }
    if stmt.prep_snapshot_id() != theorem.exec_input.prep_snapshot_id()
        || theorem.link.prep_snapshot_id() != stmt.prep_snapshot_id()
    {
        return Err(SettlementError::CheckpointLink);
    }
    if stmt.prev_root() != theorem.exec_input.prev_root()
        || theorem.artifact.prev_root() != theorem.exec_input.prev_root()
    {
        return Err(SettlementError::CheckpointRoot);
    }
    if tx_prev_root(theorem.tx_package)? != theorem.exec_input.prev_root() {
        return Err(SettlementError::CheckpointRoot);
    }

    let checkpoint_id = derive_checkpoint_id(theorem.artifact)
        .map_err(|err| SettlementError::Codec(err.to_string()))?;
    if theorem.link.checkpoint_id() != checkpoint_id {
        return Err(SettlementError::CheckpointLink);
    }
    encode_link_bin(theorem.link).map_err(|_| SettlementError::CheckpointLink)?;
    verify_tx_inclusion(theorem.tx_package, theorem.exec_input)
}

fn verify_tx_package(tx_package: &TxPackage) -> Result<(), SettlementError> {
    let bytes = JsonCodec
        .serialize(tx_package)
        .map_err(|err| SettlementError::Codec(err.to_string()))?;
    TxVerifierImpl::new()
        .verify_structure(&bytes)
        .map_err(|err| SettlementError::TxTheorem(err.to_string()))?;
    let expected_digest = build_tx_package_digest(
        &tx_package.kind,
        &tx_package.package_type,
        tx_package.version,
        tx_package.chain_id,
        &tx_package.chain_type,
        &tx_package.chain_name,
        &tx_package.tx,
    )
    .map_err(|err| SettlementError::TxTheorem(err.to_string()))?;
    if tx_package.tx_digest_hex != expected_digest {
        return Err(SettlementError::TxTheorem(
            "tx_digest_hex does not match payload".to_string(),
        ));
    }
    verify_package_public_spend_contract(tx_package)
        .map_err(|err| SettlementError::TxTheorem(err.to_string()))
}

fn digest_settlement_theorem(theorem: &SettlementTheorem<'_>) -> [u8; 32] {
    let tx_digest = hex::decode(&theorem.tx_package.tx_digest_hex)
        .expect("validated theorem bundle tx digest must decode");
    let tx_digest: [u8; 32] = tx_digest
        .try_into()
        .expect("validated theorem bundle tx digest must stay 32 bytes");
    let checkpoint_id = derive_checkpoint_id(theorem.artifact)
        .expect("validated theorem bundle checkpoint id must derive");
    let exec_bytes = encode_exec_bin(theorem.exec_input)
        .expect("validated theorem bundle exec input must encode");
    let exec_id = derive_exec_id(&exec_bytes);
    let link_bytes =
        encode_link_bin(theorem.link).expect("validated theorem bundle link must encode");

    let mut bytes = Vec::with_capacity(192 + link_bytes.len());
    bytes.extend_from_slice(SETTLEMENT_THEOREM_DIGEST_TAG);
    bytes.push(1);
    push_len_prefixed(&mut bytes, &tx_digest);
    bytes.extend_from_slice(checkpoint_id.as_bytes());
    bytes.extend_from_slice(exec_id.as_bytes());
    push_len_prefixed(&mut bytes, &link_bytes);

    let digest = DomainHasher256::<SettlementTheoremDigestDomain>::new_with_label("digest")
        .chain(bytes)
        .finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_ref());
    out
}

fn push_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn tx_prev_root(tx_package: &TxPackage) -> Result<CheckRoot, SettlementError> {
    let spend = tx_package.tx.proof.spend.as_ref().ok_or_else(|| {
        SettlementError::TxTheorem("transaction package is missing spend proof".to_string())
    })?;
    decode_check_root(&spend.prev_root_hex)
}

fn verify_tx_inclusion(
    tx_package: &TxPackage,
    exec_input: &CheckpointExecInput,
) -> Result<(), SettlementError> {
    let proof_bytes = JsonCodec
        .serialize(&tx_package.tx.proof)
        .map_err(|err| SettlementError::Codec(err.to_string()))?;
    for tx_row in exec_input.txs() {
        if tx_row_matches(tx_package, tx_row, &proof_bytes)? {
            return Ok(());
        }
    }

    Err(SettlementError::TxMissing)
}

fn tx_row_matches(
    tx_package: &TxPackage,
    tx_row: &CheckpointExecTx,
    proof_bytes: &[u8],
) -> Result<bool, SettlementError> {
    Ok(tx_row.tx_proof() == proof_bytes
        && input_refs_match(&tx_package.tx.inputs, tx_row.input_refs())?
        && outputs_match(&tx_package.tx.outputs, tx_row.outputs())?)
}

fn input_refs_match(
    tx_inputs: &[z00z_wallets::tx::TxInputWire],
    exec_inputs: &[CheckpointInRef],
) -> Result<bool, SettlementError> {
    if tx_inputs.len() != exec_inputs.len() {
        return Ok(false);
    }

    tx_inputs
        .iter()
        .zip(exec_inputs)
        .try_fold(true, |is_match, (tx_input, exec_input)| {
            let asset_id = decode_asset_id(&tx_input.asset_id_hex)?;
            Ok(is_match
                && exec_input.terminal_id().into_bytes() == asset_id
                && exec_input.serial_id() == SerialId::new(tx_input.serial_id))
        })
}

fn outputs_match(
    tx_outputs: &[TxOutputWire],
    exec_outputs: &[CheckpointExecOut],
) -> Result<bool, SettlementError> {
    if tx_outputs.len() != exec_outputs.len() {
        return Ok(false);
    }

    tx_outputs
        .iter()
        .zip(exec_outputs)
        .try_fold(true, |is_match, (tx_output, exec_output)| {
            let wire = tx_output
                .asset_wire
                .clone()
                .to_wire()
                .map_err(|err| SettlementError::Codec(err.to_string()))?;
            let leaf = asset_wire_to_leaf(&wire).map_err(SettlementError::Codec)?;
            Ok(is_match
                && exec_output.definition_id() == DefinitionId::new(wire.definition.id)
                && exec_output.leaf() == &leaf)
        })
}

fn decode_asset_id(value: &str) -> Result<[u8; 32], SettlementError> {
    let bytes = hex::decode(value).map_err(|err| SettlementError::Codec(err.to_string()))?;
    let asset_id: [u8; 32] = bytes
        .try_into()
        .map_err(|_| SettlementError::Codec("asset id must be 32 bytes".to_string()))?;
    if hex::encode(asset_id) != value {
        return Err(SettlementError::Codec(
            "asset id must be canonical lowercase hex".to_string(),
        ));
    }
    Ok(asset_id)
}

fn decode_check_root(value: &str) -> Result<CheckRoot, SettlementError> {
    let bytes = hex::decode(value).map_err(|err| SettlementError::Codec(err.to_string()))?;
    let root: [u8; 32] = bytes
        .try_into()
        .map_err(|_| SettlementError::Codec("checkpoint root must be 32 bytes".to_string()))?;
    if hex::encode(root) != value {
        return Err(SettlementError::Codec(
            "checkpoint root must be canonical lowercase hex".to_string(),
        ));
    }
    Ok(CheckRoot::new(root))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub batch_id: BatchId,
    pub checkpoint_id: Option<CheckpointId>,
    pub publication: Option<PublicationBinding>,
    pub kind: VerdictKind,
    pub reject: Option<RejectClass>,
    pub object_verdicts: Vec<ObjectValidatorVerdict>,
}

impl Verdict {
    #[must_use]
    pub fn object_reject_codes(&self) -> Vec<ObjectRejectCode> {
        self.object_verdicts
            .iter()
            .filter_map(|item| item.reject)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerdictKind {
    Accepted,
    Rejected,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectClass {
    ArtifactMissing,
    ArtifactVersion,
    ShapeInvalid,
    PolicyUnknown,
    AuthInvalid,
    ProofInvalid,
    ReplayConflict,
    ReconcileInvalid,
    StateRootMismatch,
    ProviderInvalid,
}

#[must_use]
pub const fn reject_class(code: ObjectRejectCode) -> RejectClass {
    match code {
        ObjectRejectCode::UnknownPolicy => RejectClass::PolicyUnknown,
        ObjectRejectCode::Replay => RejectClass::ReplayConflict,
        ObjectRejectCode::StaleRoot => RejectClass::StateRootMismatch,
        ObjectRejectCode::MissingRight
        | ObjectRejectCode::RightOutOfScope
        | ObjectRejectCode::RightExpired
        | ObjectRejectCode::RightRevoked
        | ObjectRejectCode::RightConsumed
        | ObjectRejectCode::MissingSignature
        | ObjectRejectCode::MissingAttestation
        | ObjectRejectCode::ForcedAcceptance => RejectClass::AuthInvalid,
        ObjectRejectCode::UnknownAction
        | ObjectRejectCode::InvalidBacking
        | ObjectRejectCode::WrongFamilyProof
        | ObjectRejectCode::VoucherUsedAsCash
        | ObjectRejectCode::RightUsedAsValue
        | ObjectRejectCode::DoubleRedeem
        | ObjectRejectCode::ResidualMismatch
        | ObjectRejectCode::FeeBoundary
        | ObjectRejectCode::ExpiredVoucherUse => RejectClass::ProofInvalid,
    }
}
