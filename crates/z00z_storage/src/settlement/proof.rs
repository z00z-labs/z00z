use jmt::{proof::SparseMerkleProof, KeyHash, RootHash};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::OnceLock;
use thiserror::Error;
use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_utils::codec::{BincodeCodec, Codec, CodecError};

use crate::backend::codec::ns_key;

use super::keys::{definition_key, serial_key, terminal_key};
use super::leaf::TerminalLeaf;
use super::tree_id::TreeId;
use super::{
    BucketId, BucketPolicy, BucketRootLeaf, DefinitionRootLeaf, PriorProofContextV1, ProofItem,
    RightClass, RightLeaf, SerialRootLeaf, SettlementLeaf, SettlementPath, SettlementStateRoot,
    VoucherLeaf,
};

hash_domain!(StorProofBindDom, "z00z.storage.proof.bind", 1);

pub const HJMT_PROOF_ENVELOPE_VERSION: u8 = 2;
pub const HJMT_DEFAULT_COMMITMENT_VERSION: u8 = 1;
const ROOT_BIND_VER: u8 = 1;
static HJMT_DEFAULT_VALUE_COMMITMENT: OnceLock<[u8; 32]> = OnceLock::new();
static HJMT_DEFAULT_CHILD_COMMITMENT: OnceLock<[u8; 32]> = OnceLock::new();

#[must_use]
pub fn hjmt_default_value_commitment() -> [u8; 32] {
    *HJMT_DEFAULT_VALUE_COMMITMENT.get_or_init(|| {
        hash_zk::<StorProofBindDom>(
            "proof_hjmt_default_value_commitment_v1",
            &[&[HJMT_DEFAULT_COMMITMENT_VERSION]],
        )
    })
}

#[must_use]
pub fn hjmt_default_child_commitment() -> [u8; 32] {
    *HJMT_DEFAULT_CHILD_COMMITMENT.get_or_init(|| {
        hash_zk::<StorProofBindDom>(
            "proof_hjmt_default_child_commitment_v1",
            &[&[HJMT_DEFAULT_COMMITMENT_VERSION]],
        )
    })
}

/// Proof families understood by the current hjmt proof envelope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HjmtProofFamily {
    #[default]
    Inclusion,
    Deletion,
    NonExistence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettlementLeafFamily {
    #[default]
    Terminal,
    Right,
    Voucher,
}

impl SettlementLeafFamily {
    #[must_use]
    pub fn from_leaf(leaf: &SettlementLeaf) -> Self {
        match leaf {
            SettlementLeaf::Terminal(_) => Self::Terminal,
            SettlementLeaf::Right(_) => Self::Right,
            SettlementLeaf::Voucher(_) => Self::Voucher,
        }
    }

    #[must_use]
    pub fn marker_leaf(self, path: SettlementPath) -> SettlementLeaf {
        match self {
            Self::Terminal => SettlementLeaf::Terminal(TerminalLeaf {
                asset_id: path.terminal_id().into_bytes(),
                serial_id: path.serial_id.get(),
                r_pub: [0u8; 32],
                owner_tag: [0u8; 32],
                c_amount: [0u8; 32],
                enc_pack: z00z_crypto::ZkPackEncrypted {
                    version: 1,
                    ciphertext: Vec::new(),
                    tag: [0u8; 16],
                },
                range_proof: Vec::new(),
                tag16: 0,
            }),
            Self::Right => SettlementLeaf::Right(RightLeaf {
                version: 1,
                terminal_id: path.terminal_id,
                right_class: RightClass::MachineCapability,
                issuer_scope: [0u8; 32],
                provider_scope: [0u8; 32],
                holder_commitment: [0u8; 32],
                control_commitment: [0u8; 32],
                beneficiary_commitment: [0u8; 32],
                payload_commitment: [0u8; 32],
                valid_from: 0,
                valid_until: 0,
                challenge_from: 0,
                challenge_until: 0,
                use_nonce: [0u8; 32],
                revocation_policy_id: [0u8; 32],
                transition_policy_id: [0u8; 32],
                challenge_policy_id: [0u8; 32],
                disclosure_policy_id: [0u8; 32],
                retention_policy_id: [0u8; 32],
            }),
            Self::Voucher => SettlementLeaf::Voucher(VoucherLeaf::marker(path)),
        }
    }
}

/// Hjmt mode supports explicit inclusion, deletion, and non-existence families.
pub fn check_hjmt_proof_family(family: HjmtProofFamily) -> Result<(), ProofChkErr> {
    match family {
        HjmtProofFamily::Inclusion | HjmtProofFamily::Deletion | HjmtProofFamily::NonExistence => {
            Ok(())
        }
    }
}

/// Canonical proof-codec validation failures for storage-owned witness blobs.
#[derive(Debug, Error)]
pub enum ProofChkErr {
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("settlement-state root mismatch")]
    RootMix,
    #[error("settlement root generation mismatch")]
    RootGenerationMix,
    #[error("unsupported proof-root binding version")]
    BindVerMix,
    #[error("proof root binding mismatch")]
    RootBindMix,
    #[error("settlement path mismatch")]
    PathMix,
    #[error("definition root leaf mismatch")]
    DefMix,
    #[error("serial root leaf mismatch")]
    SerMix,
    #[error("terminal leaf mismatch")]
    LeafMix,
    #[error("terminal leaf hash mismatch")]
    LeafHashMix,
    #[error("definition proof mismatch")]
    DefProofMix,
    #[error("serial proof mismatch")]
    SerProofMix,
    #[error("terminal proof mismatch")]
    TerminalProofMix,
    #[error("unsupported hjmt proof version")]
    UnsupportedHjmtProofVersion,
    #[error("hjmt proof family mismatch")]
    ProofFamilyMix,
    #[error("hjmt default commitment mismatch")]
    DefaultCommitmentMix,
    #[error("hjmt journal checkpoint mismatch")]
    JournalCheckpointMix,
    #[error("bucket policy mismatch")]
    BucketPolicyMix,
    #[error("bucket root leaf mismatch")]
    BucketMix,
    #[error("bucket proof mismatch")]
    BucketProofMix,
    #[error("prior settlement root mismatch")]
    PriorRootMix,
    #[error("prior definition root leaf mismatch")]
    PriorDefMix,
    #[error("prior serial root leaf mismatch")]
    PriorSerMix,
    #[error("prior bucket root leaf mismatch")]
    PriorBucketMix,
    #[error("prior definition proof mismatch")]
    PriorDefProofMix,
    #[error("prior serial proof mismatch")]
    PriorSerProofMix,
    #[error("prior bucket proof mismatch")]
    PriorBucketProofMix,
    #[error("prior terminal proof mismatch")]
    PriorTerminalProofMix,
    #[error("unsupported batch proof version")]
    UnsupportedBatchProofVersion,
    #[error("unsupported JMT update-trace version")]
    UnsupportedJmtUpdateVersion,
    #[error("JMT update trace exceeded the configured bound")]
    JmtUpdateTraceLimit,
    #[error("JMT update trace is noncanonical")]
    JmtUpdateTraceCanonical,
    #[error("JMT update proof verification failed")]
    JmtUpdateProofMix,
    #[error("batch proof truncated")]
    BatchTrunc,
    #[error("batch proof trailing bytes")]
    BatchTrailingBytes,
    #[error("batch proof limit mismatch")]
    BatchLimitMix,
    #[error("batch proof tag mismatch")]
    BatchTagMix,
    #[error("batch proof boolean mismatch")]
    BatchBoolMix,
    #[error("batch proof transcript mismatch")]
    BatchTranscriptMix,
    #[error("batch proof root generation mismatch")]
    BatchRootGenerationMix,
    #[error("publication contract truncated")]
    PublicationTrunc,
    #[error("publication contract trailing bytes")]
    PublicationTrailingBytes,
    #[error("publication mode mismatch")]
    PublicationModeMix,
    #[error("publication root generation mismatch")]
    PublicationRootGenerationMix,
    #[error("publication ordering mismatch")]
    PublicationOrderMix,
    #[error("publication duplicate shard")]
    PublicationDupShard,
    #[error("publication transition flags mismatch")]
    PublicationFlagMix,
    #[error("publication leaf count mismatch")]
    PublicationCountMix,
    #[error("publication route binding mismatch")]
    PublicationRouteMix,
    #[error("publication prior root mismatch")]
    PublicationPriorRootMix,
    #[error("publication checkpoint mismatch")]
    PublicationCheckpointMix,
    #[error("publication monotonicity mismatch")]
    PublicationMonotonicityMix,
    #[error("publication policy-set mismatch")]
    PublicationPolicyMix,
    #[error("publication proof root generation mismatch")]
    PublicationProofGenerationMix,
    #[error("publication proof shard binding mismatch")]
    PublicationProofShardMix,
    #[error("publication proof leaf index mismatch")]
    PublicationProofIndexMix,
    #[error("publication proof route binding mismatch")]
    PublicationProofRouteMix,
    #[error("publication proof checkpoint binding mismatch")]
    PublicationProofCheckpointMix,
    #[error("publication proof policy binding mismatch")]
    PublicationProofPolicyMix,
    #[error("batch proof path mismatch")]
    BatchPathMix,
    #[error("batch proof leaf family mismatch")]
    BatchLeafFamilyMix,
    #[error("batch proof default commitment mismatch")]
    BatchDefaultCommitmentMix,
    #[error("batch proof shard context mismatch")]
    BatchShardCtxMix,
    #[error("batch proof opening kind mismatch")]
    BatchOpeningKindMix,
    #[error("batch proof index mismatch")]
    BatchIndexMix,
    #[error("batch proof ordering mismatch")]
    BatchOrderMix,
    #[error("batch proof duplicate path")]
    BatchDupPath,
    #[error("batch proof witness domain mismatch")]
    BatchWitnessDomainMix,
    #[error("batch proof hash material count mismatch")]
    BatchHashCountMix,
    #[error("batch proof policy mismatch")]
    BatchPolicyMix,
    #[error("batch proof checkpoint mismatch")]
    BatchCheckpointMix,
    #[error("batch proof root binding version mismatch")]
    BatchBindVerMix,
    #[error("batch proof root binding mismatch")]
    BatchRootBindMix,
    #[error("batch proof witness step mismatch")]
    BatchWitnessStepMix,
    #[error("batch proof subtree marker mismatch")]
    BatchSubtreeMix,
    #[error("batch proof root mismatch")]
    BatchRootMix,
}

impl PartialEq for ProofChkErr {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for ProofChkErr {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HjmtPriorProofEnvelope {
    version: u64,
    settlement_root: SettlementStateRoot,
    backend_root: [u8; 32],
    root_bind_ver: u8,
    root_bind: [u8; 32],
    #[serde(default)]
    journal_digest: Option<[u8; 32]>,
    #[serde(default)]
    checkpoint_bind: Option<[u8; 32]>,
    definition_root_leaf: DefinitionRootLeaf,
    serial_root_leaf: SerialRootLeaf,
    bucket_root_leaf: BucketRootLeaf,
    definition_proof: Vec<u8>,
    serial_proof: Vec<u8>,
    bucket_proof: Vec<u8>,
    terminal_proof: Vec<u8>,
}

impl HjmtPriorProofEnvelope {
    #[must_use]
    pub(crate) fn new(
        version: u64,
        settlement_root: SettlementStateRoot,
        backend_root: [u8; 32],
        journal_digest: [u8; 32],
        definition_root_leaf: DefinitionRootLeaf,
        serial_root_leaf: SerialRootLeaf,
        bucket_root_leaf: BucketRootLeaf,
        definition_proof: Vec<u8>,
        serial_proof: Vec<u8>,
        bucket_proof: Vec<u8>,
        terminal_proof: Vec<u8>,
    ) -> Self {
        Self {
            version,
            settlement_root,
            backend_root,
            root_bind_ver: ROOT_BIND_VER,
            root_bind: root_bind(settlement_root, backend_root),
            journal_digest: Some(journal_digest),
            checkpoint_bind: Some(hjmt_checkpoint_bind(
                settlement_root,
                backend_root,
                version,
                journal_digest,
            )),
            definition_root_leaf,
            serial_root_leaf,
            bucket_root_leaf,
            definition_proof,
            serial_proof,
            bucket_proof,
            terminal_proof,
        }
    }

    #[must_use]
    pub(crate) const fn version(&self) -> u64 {
        self.version
    }

    #[must_use]
    pub(crate) const fn settlement_root(&self) -> SettlementStateRoot {
        self.settlement_root
    }

    #[must_use]
    pub(crate) const fn backend_root(&self) -> [u8; 32] {
        self.backend_root
    }

    #[must_use]
    pub(crate) const fn journal_digest(&self) -> Option<[u8; 32]> {
        self.journal_digest
    }

    pub(crate) fn to_prior_context_v1(&self) -> PriorProofContextV1 {
        let journal_digest = self.journal_digest.unwrap_or_else(|| {
            hjmt_checkpoint_digest(self.settlement_root, self.backend_root, self.version)
        });
        PriorProofContextV1 {
            version: 1,
            prior_hjmt_version: self.version,
            prior_settlement_root: self.settlement_root,
            prior_backend_root: self.backend_root,
            prior_root_bind_version: self.root_bind_ver,
            prior_root_bind: self.root_bind,
            prior_journal_digest: journal_digest,
            prior_checkpoint_bind: self.checkpoint_bind.unwrap_or_else(|| {
                hjmt_checkpoint_bind(
                    self.settlement_root,
                    self.backend_root,
                    self.version,
                    journal_digest,
                )
            }),
            definition_root_leaf_bytes: self.definition_root_leaf.encode(),
            serial_root_leaf_bytes: self.serial_root_leaf.encode(),
            bucket_root_leaf_bytes: self.bucket_root_leaf.encode(),
            definition_proof_bytes: self.definition_proof.clone(),
            serial_proof_bytes: self.serial_proof.clone(),
            bucket_proof_bytes: self.bucket_proof.clone(),
            prior_terminal_proof_bytes: self.terminal_proof.clone(),
        }
    }

    fn check_root_bind(&self) -> Result<(), ProofChkErr> {
        if self.root_bind_ver != ROOT_BIND_VER {
            return Err(ProofChkErr::BindVerMix);
        }
        if self.root_bind != root_bind(self.settlement_root, self.backend_root) {
            return Err(ProofChkErr::PriorRootMix);
        }
        let journal_digest = self
            .journal_digest
            .ok_or(ProofChkErr::JournalCheckpointMix)?;
        if self.checkpoint_bind
            != Some(hjmt_checkpoint_bind(
                self.settlement_root,
                self.backend_root,
                self.version,
                journal_digest,
            ))
        {
            return Err(ProofChkErr::JournalCheckpointMix);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HjmtProofEnvelope {
    version: u8,
    #[serde(default)]
    family: HjmtProofFamily,
    #[serde(default)]
    leaf_family: SettlementLeafFamily,
    #[serde(default)]
    journal_checkpoint: Option<u64>,
    #[serde(default)]
    journal_digest: Option<[u8; 32]>,
    #[serde(default)]
    checkpoint_bind: Option<[u8; 32]>,
    #[serde(default)]
    default_commitment_version: Option<u8>,
    #[serde(default)]
    default_commitment: Option<[u8; 32]>,
    #[serde(default)]
    default_child_commitment: Option<[u8; 32]>,
    bucket_policy: BucketPolicy,
    bucket_root_leaf: BucketRootLeaf,
    bucket_proof: Vec<u8>,
    #[serde(default)]
    prior: Option<HjmtPriorProofEnvelope>,
}

/// Storage-owned witness payload that binds typed settlement proof semantics to
/// backend proof bytes.
///
/// Version-1 verifiers accept only the inclusion family. Live hjmt proofs use
/// the same blob type with an optional hjmt envelope that carries bucket-
/// policy and bucket-proof context.
///
/// Version 1 is inclusion-only. Deletion, non-existence, bucket-policy, and
/// bucket-proof semantics belong to the live hjmt proof envelope and must be
/// rejected by version-1 verifiers instead of being inferred here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofBlob {
    item: ProofItem,
    terminal_leaf_hash: [u8; 32],
    #[serde(default)]
    backend_root: [u8; 32],
    #[serde(default)]
    root_bind_ver: u8,
    #[serde(default)]
    root_bind: [u8; 32],
    definition_proof: Vec<u8>,
    serial_proof: Vec<u8>,
    terminal_proof: Vec<u8>,
    #[serde(default)]
    hjmt: Option<HjmtProofEnvelope>,
}

impl ProofBlob {
    /// Build one canonical storage witness blob.
    ///
    /// The `backend_root` input is proof-local physical context used to verify
    /// branch proofs. It is bound to the semantic `SettlementStateRoot` through
    /// `root_bind` and must not be treated as checkpoint or state authority.
    #[must_use]
    pub fn new(
        item: ProofItem,
        terminal_leaf_hash: [u8; 32],
        backend_root: [u8; 32],
        definition_proof: Vec<u8>,
        serial_proof: Vec<u8>,
        terminal_proof: Vec<u8>,
    ) -> Self {
        let root_bind = root_bind(item.settlement_root(), backend_root);
        Self {
            item,
            terminal_leaf_hash,
            backend_root,
            root_bind_ver: ROOT_BIND_VER,
            root_bind,
            definition_proof,
            serial_proof,
            terminal_proof,
            hjmt: None,
        }
    }

    /// Build one canonical hjmt inclusion proof blob.
    ///
    /// The `backend_root` is the hjmt definition-tree root used by storage
    /// proof verification. It remains diagnostic/proof-local and is bound to
    /// the semantic `SettlementStateRoot` through `root_bind`.
    #[must_use]
    pub fn new_forest(
        item: ProofItem,
        terminal_leaf_hash: [u8; 32],
        backend_root: [u8; 32],
        bucket_policy: BucketPolicy,
        bucket_root_leaf: BucketRootLeaf,
        definition_proof: Vec<u8>,
        serial_proof: Vec<u8>,
        bucket_proof: Vec<u8>,
        terminal_proof: Vec<u8>,
        family: HjmtProofFamily,
        journal_checkpoint: Option<u64>,
        journal_digest: Option<[u8; 32]>,
    ) -> Self {
        let leaf_family = SettlementLeafFamily::from_leaf(item.leaf());
        let root_bind = root_bind(item.settlement_root(), backend_root);
        let checkpoint_bind = journal_checkpoint
            .zip(journal_digest)
            .map(|(checkpoint, digest)| {
                hjmt_checkpoint_bind(item.settlement_root(), backend_root, checkpoint, digest)
            });
        Self {
            item,
            terminal_leaf_hash,
            backend_root,
            root_bind_ver: ROOT_BIND_VER,
            root_bind,
            definition_proof,
            serial_proof,
            terminal_proof,
            hjmt: Some(HjmtProofEnvelope {
                version: HJMT_PROOF_ENVELOPE_VERSION,
                family,
                leaf_family,
                journal_checkpoint,
                journal_digest,
                checkpoint_bind,
                default_commitment_version: Some(HJMT_DEFAULT_COMMITMENT_VERSION),
                default_commitment: None,
                default_child_commitment: Some(hjmt_default_child_commitment()),
                bucket_policy,
                bucket_root_leaf,
                bucket_proof,
                prior: None,
            }),
        }
    }

    /// Encode one witness blob into the storage-owned byte format.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        let codec = BincodeCodec;
        codec.serialize(self)
    }

    /// Decode one witness blob from the storage-owned byte format.
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        let codec = BincodeCodec;
        codec.deserialize(bytes)
    }

    /// Return the typed semantic proof carried by this witness blob.
    #[must_use]
    pub fn item(&self) -> &ProofItem {
        &self.item
    }

    /// Return the canonical JMT value hash of the terminal committed settlement leaf.
    #[must_use]
    pub const fn terminal_leaf_hash(&self) -> [u8; 32] {
        self.terminal_leaf_hash
    }

    /// Return diagnostic/proof-local physical shared-root bytes.
    ///
    /// These bytes are used only by storage-owned backend proof verification.
    /// They are not a `SettlementStateRoot` or `CheckRoot` and must not be treated
    /// as semantic authority by callers.
    #[must_use]
    pub const fn backend_root(&self) -> [u8; 32] {
        self.backend_root
    }

    /// Return the semantic/backend-root binding version carried by this blob.
    #[must_use]
    pub const fn root_bind_ver(&self) -> u8 {
        self.root_bind_ver
    }

    /// Return the semantic/backend-root commitment carried by this blob.
    #[must_use]
    pub const fn root_bind(&self) -> [u8; 32] {
        self.root_bind
    }

    /// Return the opaque backend proof bytes for the definition leaf.
    #[must_use]
    pub fn definition_proof(&self) -> &[u8] {
        &self.definition_proof
    }

    /// Return the opaque backend proof bytes for the serial leaf.
    #[must_use]
    pub fn serial_proof(&self) -> &[u8] {
        &self.serial_proof
    }

    /// Return the opaque backend proof bytes carried by this witness blob.
    #[must_use]
    pub fn terminal_proof(&self) -> &[u8] {
        &self.terminal_proof
    }

    /// Rebind the typed semantic item while preserving the aggregate branch-proof payloads.
    #[must_use]
    pub fn rebind(&self, item: ProofItem) -> Self {
        let root_bind = root_bind(item.settlement_root(), self.backend_root);
        let mut hjmt = self.hjmt.clone();
        if let Some(hjmt) = &mut hjmt {
            hjmt.checkpoint_bind =
                hjmt.journal_checkpoint
                    .zip(hjmt.journal_digest)
                    .map(|(checkpoint, digest)| {
                        hjmt_checkpoint_bind(
                            item.settlement_root(),
                            self.backend_root,
                            checkpoint,
                            digest,
                        )
                    });
        }
        Self {
            item,
            terminal_leaf_hash: self.terminal_leaf_hash,
            backend_root: self.backend_root,
            root_bind_ver: ROOT_BIND_VER,
            root_bind,
            definition_proof: self.definition_proof.clone(),
            serial_proof: self.serial_proof.clone(),
            terminal_proof: self.terminal_proof.clone(),
            hjmt,
        }
    }

    /// Return a copy with a substituted semantic/backend root binding.
    #[must_use]
    pub fn with_root_bind(mut self, root_bind_ver: u8, root_bind: [u8; 32]) -> Self {
        self.root_bind_ver = root_bind_ver;
        self.root_bind = root_bind;
        self
    }

    /// Return a copy with a substituted terminal leaf hash.
    #[must_use]
    pub fn with_terminal_leaf_hash(mut self, terminal_leaf_hash: [u8; 32]) -> Self {
        self.terminal_leaf_hash = terminal_leaf_hash;
        self
    }

    /// Return a copy with substituted definition proof bytes.
    #[must_use]
    pub fn with_definition_proof(mut self, proof: Vec<u8>) -> Self {
        self.definition_proof = proof;
        self
    }

    /// Return a copy with substituted serial proof bytes.
    #[must_use]
    pub fn with_serial_proof(mut self, proof: Vec<u8>) -> Self {
        self.serial_proof = proof;
        self
    }

    /// Return a copy with substituted terminal proof bytes.
    #[must_use]
    pub fn with_terminal_proof(mut self, proof: Vec<u8>) -> Self {
        self.terminal_proof = proof;
        self
    }

    /// Return the hjmt proof-envelope version when this blob is hjmt-backed.
    #[must_use]
    pub fn hjmt_envelope_version(&self) -> Option<u8> {
        self.hjmt.as_ref().map(|hjmt| hjmt.version)
    }

    /// Return the committed hjmt bucket policy when this blob is hjmt-backed.
    #[must_use]
    pub fn hjmt_bucket_policy(&self) -> Option<BucketPolicy> {
        self.hjmt.as_ref().map(|hjmt| hjmt.bucket_policy)
    }

    /// Return the committed hjmt bucket root leaf when this blob is hjmt-backed.
    #[must_use]
    pub fn hjmt_bucket_root_leaf(&self) -> Option<BucketRootLeaf> {
        self.hjmt.as_ref().map(|hjmt| hjmt.bucket_root_leaf)
    }

    /// Return the hjmt bucket proof bytes when this blob is hjmt-backed.
    #[must_use]
    pub fn hjmt_bucket_proof(&self) -> Option<&[u8]> {
        self.hjmt.as_ref().map(|hjmt| hjmt.bucket_proof.as_slice())
    }

    #[must_use]
    pub fn hjmt_proof_family(&self) -> Option<HjmtProofFamily> {
        self.hjmt.as_ref().map(|hjmt| hjmt.family)
    }

    #[must_use]
    pub fn hjmt_leaf_family(&self) -> Option<SettlementLeafFamily> {
        self.hjmt.as_ref().map(|hjmt| hjmt.leaf_family)
    }

    #[must_use]
    pub fn hjmt_journal_checkpoint(&self) -> Option<u64> {
        self.hjmt.as_ref().and_then(|hjmt| hjmt.journal_checkpoint)
    }

    #[must_use]
    pub fn hjmt_journal_digest(&self) -> Option<[u8; 32]> {
        self.hjmt.as_ref().and_then(|hjmt| hjmt.journal_digest)
    }

    #[must_use]
    pub fn hjmt_default_commitment_version(&self) -> Option<u8> {
        self.hjmt
            .as_ref()
            .and_then(|hjmt| hjmt.default_commitment_version)
    }

    #[must_use]
    pub fn hjmt_default_commitment(&self) -> Option<[u8; 32]> {
        self.hjmt.as_ref().and_then(|hjmt| hjmt.default_commitment)
    }

    #[must_use]
    pub fn hjmt_default_child_commitment(&self) -> Option<[u8; 32]> {
        self.hjmt
            .as_ref()
            .and_then(|hjmt| hjmt.default_child_commitment)
    }

    #[must_use]
    pub(crate) fn hjmt_prior(&self) -> Option<&HjmtPriorProofEnvelope> {
        self.hjmt.as_ref().and_then(|hjmt| hjmt.prior.as_ref())
    }

    #[cfg(feature = "test-params-fast")]
    #[must_use]
    pub fn hjmt_prior_backend_root(&self) -> Option<[u8; 32]> {
        self.hjmt_prior().map(HjmtPriorProofEnvelope::backend_root)
    }

    /// Return a copy with a substituted hjmt envelope version.
    #[must_use]
    pub fn with_hjmt_envelope_version(mut self, version: u8) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.version = version;
        }
        self
    }

    /// Return a copy with a substituted hjmt bucket policy.
    #[must_use]
    pub fn with_hjmt_bucket_policy(mut self, bucket_policy: BucketPolicy) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.bucket_policy = bucket_policy;
        }
        self
    }

    /// Return a copy with a substituted hjmt bucket root leaf.
    #[must_use]
    pub fn with_hjmt_bucket_root_leaf(mut self, bucket_root_leaf: BucketRootLeaf) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.bucket_root_leaf = bucket_root_leaf;
        }
        self
    }

    /// Return a copy with substituted hjmt bucket proof bytes.
    #[must_use]
    pub fn with_hjmt_bucket_proof(mut self, proof: Vec<u8>) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.bucket_proof = proof;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_proof_family(mut self, family: HjmtProofFamily) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.family = family;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_leaf_family(mut self, leaf_family: SettlementLeafFamily) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.leaf_family = leaf_family;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_journal_checkpoint(mut self, journal_checkpoint: Option<u64>) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.journal_checkpoint = journal_checkpoint;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_journal_digest(mut self, journal_digest: Option<[u8; 32]>) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.journal_digest = journal_digest;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_default_commitment_version(mut self, version: Option<u8>) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.default_commitment_version = version;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_default_commitment(mut self, default_commitment: Option<[u8; 32]>) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.default_commitment = default_commitment;
        }
        self
    }

    #[must_use]
    pub fn with_hjmt_default_child_commitment(
        mut self,
        default_child_commitment: Option<[u8; 32]>,
    ) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.default_child_commitment = default_child_commitment;
        }
        self
    }

    #[must_use]
    pub(crate) fn with_hjmt_prior(mut self, prior: HjmtPriorProofEnvelope) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            hjmt.prior = Some(prior);
        }
        self
    }

    #[cfg(feature = "test-params-fast")]
    #[must_use]
    pub fn with_hjmt_prior_blob(mut self, prior_blob: &ProofBlob) -> Self {
        if let Some(hjmt) = &mut self.hjmt {
            if let Some(prior) = hjmt.prior.as_mut() {
                prior.backend_root = prior_blob.backend_root();
                prior.definition_root_leaf = prior_blob.item().def_leaf();
                prior.serial_root_leaf = prior_blob.item().ser_leaf();
                if let Some(bucket_leaf) = prior_blob.hjmt_bucket_root_leaf() {
                    prior.bucket_root_leaf = bucket_leaf;
                }
                prior.definition_proof = prior_blob.definition_proof().to_vec();
                prior.serial_proof = prior_blob.serial_proof().to_vec();
                if let Some(bucket_proof) = prior_blob.hjmt_bucket_proof() {
                    prior.bucket_proof = bucket_proof.to_vec();
                } else {
                    prior.bucket_proof.clear();
                }
                prior.terminal_proof = prior_blob.terminal_proof().to_vec();
                prior.journal_digest = prior_blob.hjmt_journal_digest();
                prior.root_bind_ver = ROOT_BIND_VER;
                prior.root_bind = root_bind(prior.settlement_root, prior.backend_root);
                prior.checkpoint_bind = prior.journal_digest.map(|journal_digest| {
                    hjmt_checkpoint_bind(
                        prior.settlement_root,
                        prior.backend_root,
                        prior.version,
                        journal_digest,
                    )
                });
            }
        }
        self
    }

    fn check_root_bind(&self) -> Result<(), ProofChkErr> {
        if self.root_bind_ver != ROOT_BIND_VER {
            return Err(ProofChkErr::BindVerMix);
        }
        if self.item.settlement_root() != self.item.root() {
            return Err(ProofChkErr::RootGenerationMix);
        }
        if self.root_bind != root_bind(self.item.settlement_root(), self.backend_root) {
            return Err(ProofChkErr::RootBindMix);
        }
        Ok(())
    }
}

/// Decode only the typed proof item through the storage-owned proof codec.
pub fn proof_blob_item(bytes: &[u8]) -> Result<ProofItem, ProofChkErr> {
    Ok(ProofBlob::decode(bytes)?.item().clone())
}

/// Decode and rebind one proof blob root through the storage-owned proof codec.
pub fn proof_blob_rebind_root(
    bytes: &[u8],
    root: SettlementStateRoot,
) -> Result<Vec<u8>, ProofChkErr> {
    let blob = ProofBlob::decode(bytes)?;
    let item = blob.item();
    let next = ProofItem::new_settlement(
        root,
        item.path(),
        item.def_leaf(),
        item.ser_leaf(),
        item.leaf().clone(),
    )
    .map_err(|_| ProofChkErr::PathMix)?;
    Ok(blob.rebind(next).encode()?)
}

/// Sanitized typed summary of one verified storage witness.
///
/// This view keeps the semantic proof item, leaf hash, and semantic/backend
/// root binding, but intentionally strips the raw branch proofs carried by
/// `ProofBlob` so callers do not mistake storage membership evidence for
/// Pedersen conservation evidence. Any backend-root bytes exposed here are
/// diagnostic/proof-local and must remain paired with `root_bind` plus
/// `SettlementStateRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofScanOut {
    item: ProofItem,
    terminal_leaf_hash: [u8; 32],
    backend_root: [u8; 32],
    root_bind_ver: u8,
    root_bind: [u8; 32],
}

impl ProofScanOut {
    #[must_use]
    pub(crate) fn from_blob(blob: ProofBlob) -> Self {
        Self {
            item: blob.item,
            terminal_leaf_hash: blob.terminal_leaf_hash,
            backend_root: blob.backend_root,
            root_bind_ver: blob.root_bind_ver,
            root_bind: blob.root_bind,
        }
    }

    #[must_use]
    pub fn item(&self) -> &ProofItem {
        &self.item
    }

    #[must_use]
    pub const fn root(&self) -> SettlementStateRoot {
        self.item.root()
    }

    #[must_use]
    pub const fn settlement_root(&self) -> SettlementStateRoot {
        self.item.settlement_root()
    }

    #[must_use]
    pub const fn path(&self) -> SettlementPath {
        self.item.path()
    }

    #[must_use]
    pub const fn def_leaf(&self) -> DefinitionRootLeaf {
        self.item.def_leaf()
    }

    #[must_use]
    pub const fn ser_leaf(&self) -> SerialRootLeaf {
        self.item.ser_leaf()
    }

    #[must_use]
    pub fn leaf(&self) -> &SettlementLeaf {
        self.item.leaf()
    }

    pub fn terminal_leaf(&self) -> Result<&TerminalLeaf, crate::settlement::ModelErr> {
        self.item.terminal_leaf()
    }

    pub fn voucher_leaf(&self) -> Result<&VoucherLeaf, crate::settlement::ModelErr> {
        self.item.voucher_leaf()
    }

    #[must_use]
    pub const fn terminal_leaf_hash(&self) -> [u8; 32] {
        self.terminal_leaf_hash
    }

    /// Return diagnostic/proof-local physical shared-root bytes.
    ///
    /// Callers may log or compare these bytes only together with the semantic
    /// root and `root_bind`; they are not checkpoint or state authority.
    #[must_use]
    pub const fn backend_root(&self) -> [u8; 32] {
        self.backend_root
    }

    #[must_use]
    pub const fn root_bind_ver(&self) -> u8 {
        self.root_bind_ver
    }

    #[must_use]
    pub const fn root_bind(&self) -> [u8; 32] {
        self.root_bind
    }

    /// Return a copy with a substituted semantic/backend root binding.
    #[must_use]
    pub fn with_root_bind(mut self, root_bind_ver: u8, root_bind: [u8; 32]) -> Self {
        self.root_bind_ver = root_bind_ver;
        self.root_bind = root_bind;
        self
    }

    /// Return a copy with a substituted terminal leaf hash.
    #[must_use]
    pub fn with_terminal_leaf_hash(mut self, terminal_leaf_hash: [u8; 32]) -> Self {
        self.terminal_leaf_hash = terminal_leaf_hash;
        self
    }

    pub fn check_leaf_hash(&self) -> Result<(), ProofChkErr> {
        if self.terminal_leaf_hash != leaf_hash(self.leaf())? {
            return Err(ProofChkErr::LeafHashMix);
        }
        Ok(())
    }

    pub fn check_root_bind(&self) -> Result<(), ProofChkErr> {
        if self.root_bind_ver != ROOT_BIND_VER {
            return Err(ProofChkErr::BindVerMix);
        }
        if self.settlement_root().into_bytes() != self.root().into_bytes() {
            return Err(ProofChkErr::RootGenerationMix);
        }
        if self.root_bind != root_bind(self.settlement_root(), self.backend_root) {
            return Err(ProofChkErr::RootBindMix);
        }
        Ok(())
    }
}

fn root_bind(root: SettlementStateRoot, backend_root: [u8; 32]) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_root_bind_v1",
        &[&generation, &root_bytes, &backend_root],
    )
}

fn hjmt_checkpoint_bind(
    root: SettlementStateRoot,
    backend_root: [u8; 32],
    checkpoint: u64,
    journal_digest: [u8; 32],
) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    let checkpoint = checkpoint.to_be_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_hjmt_checkpoint_bind_v1",
        &[
            &generation,
            &root_bytes,
            &backend_root,
            &checkpoint,
            &journal_digest,
        ],
    )
}

#[must_use]
pub(crate) fn hjmt_checkpoint_digest(
    root: SettlementStateRoot,
    backend_root: [u8; 32],
    checkpoint: u64,
) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    let checkpoint = checkpoint.to_be_bytes();
    hash_zk::<StorProofBindDom>(
        "proof_hjmt_checkpoint_digest_v1",
        &[&generation, &root_bytes, &backend_root, &checkpoint],
    )
}

fn leaf_hash(leaf: impl Into<SettlementLeaf>) -> Result<[u8; 32], CodecError> {
    let payload = leaf.into().encode()?;
    Ok(jmt::ValueHash::with::<Sha256>(&payload).0)
}

fn bucket_key(bucket_id: BucketId) -> KeyHash {
    KeyHash(bucket_id.into_bytes())
}

fn dec_branch(bytes: &[u8]) -> Result<SparseMerkleProof<Sha256>, CodecError> {
    let codec = BincodeCodec;
    codec.deserialize(bytes)
}

fn leaf_payload(leaf: impl Into<SettlementLeaf>) -> Result<Vec<u8>, CodecError> {
    leaf.into().encode()
}

/// Check that one typed proof record matches the expected settlement root, path, and leaves.
pub fn chk_item_settlement(
    item: &ProofItem,
    root: SettlementStateRoot,
    path: &SettlementPath,
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
    leaf: impl Into<SettlementLeaf>,
) -> Result<(), ProofChkErr> {
    let leaf = leaf.into();
    if item.settlement_root() != root {
        return Err(ProofChkErr::RootGenerationMix);
    }
    if item.path() != *path {
        return Err(ProofChkErr::PathMix);
    }
    if item.def_leaf() != def_leaf {
        return Err(ProofChkErr::DefMix);
    }
    if item.ser_leaf() != ser_leaf {
        return Err(ProofChkErr::SerMix);
    }
    if item.leaf() != &leaf {
        return Err(ProofChkErr::LeafMix);
    }

    Ok(())
}

/// Decode and validate one storage witness blob against the expected settlement snapshot input.
pub fn chk_blob_settlement(
    bytes: &[u8],
    root: SettlementStateRoot,
    path: &SettlementPath,
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
    leaf: impl Into<SettlementLeaf>,
) -> Result<ProofBlob, ProofChkErr> {
    let leaf = leaf.into();
    let blob = ProofBlob::decode(bytes)?;
    chk_item_settlement(blob.item(), root, path, def_leaf, ser_leaf, &leaf)?;
    let expected_leaf_hash = leaf_hash(&leaf)?;
    if blob.terminal_leaf_hash() != expected_leaf_hash {
        return Err(ProofChkErr::LeafHashMix);
    }
    blob.check_root_bind()?;

    if let Some(hjmt) = blob.hjmt.as_ref() {
        return chk_hjmt_blob(&blob, hjmt, path, def_leaf, ser_leaf, &leaf);
    }

    let def_proof = dec_branch(blob.definition_proof())?;
    def_proof
        .verify_existence(
            RootHash::from(blob.backend_root()),
            ns_key(TreeId::Definition, definition_key(path.definition_id)),
            def_leaf.encode(),
        )
        .map_err(|_| ProofChkErr::DefProofMix)?;

    let ser_proof = dec_branch(blob.serial_proof())?;
    ser_proof
        .verify_existence(
            RootHash::from(blob.backend_root()),
            ns_key(
                TreeId::Serial(path.definition_id),
                serial_key(path.definition_id, path.serial_id),
            ),
            ser_leaf.encode(),
        )
        .map_err(|_| ProofChkErr::SerProofMix)?;

    let terminal_proof = dec_branch(blob.terminal_proof())?;
    let payload = leaf_payload(&leaf)?;
    terminal_proof
        .verify_existence(
            RootHash::from(blob.backend_root()),
            ns_key(
                TreeId::Terminal(path.definition_id, path.serial_id),
                terminal_key(path.terminal_id()),
            ),
            payload,
        )
        .map_err(|_| ProofChkErr::TerminalProofMix)?;

    Ok(blob)
}

/// Decode and validate one inclusion-only storage witness blob against the expected settlement snapshot input.
pub fn chk_blob_settlement_inclusion(
    bytes: &[u8],
    root: SettlementStateRoot,
    path: &SettlementPath,
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
    leaf: impl Into<SettlementLeaf>,
) -> Result<ProofBlob, ProofChkErr> {
    if ProofBlob::decode(bytes)?.hjmt_proof_family() != Some(HjmtProofFamily::Inclusion) {
        return Err(ProofChkErr::ProofFamilyMix);
    }
    chk_blob_settlement(bytes, root, path, def_leaf, ser_leaf, leaf)
}

/// Decode and validate one inclusion proof using the carried child-root leaves.
pub fn chk_blob_settlement_inclusion_bound(
    bytes: &[u8],
    root: SettlementStateRoot,
    path: &SettlementPath,
    leaf: impl Into<SettlementLeaf>,
) -> Result<ProofBlob, ProofChkErr> {
    let blob = ProofBlob::decode(bytes)?;
    chk_blob_settlement_inclusion(
        bytes,
        root,
        path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        leaf,
    )
}

/// Decode and validate one inclusion proof using its carried settlement path.
pub fn chk_blob_settlement_inclusion_carried(
    bytes: &[u8],
    root: SettlementStateRoot,
    leaf: impl Into<SettlementLeaf>,
) -> Result<ProofBlob, ProofChkErr> {
    let blob = ProofBlob::decode(bytes)?;
    let path = blob.item().path();
    chk_blob_settlement_inclusion(
        bytes,
        root,
        &path,
        blob.item().def_leaf(),
        blob.item().ser_leaf(),
        leaf,
    )
}

fn chk_hjmt_blob(
    blob: &ProofBlob,
    hjmt: &HjmtProofEnvelope,
    path: &SettlementPath,
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
    leaf: impl Into<SettlementLeaf>,
) -> Result<ProofBlob, ProofChkErr> {
    let leaf = leaf.into();
    if hjmt.version != HJMT_PROOF_ENVELOPE_VERSION {
        return Err(ProofChkErr::UnsupportedHjmtProofVersion);
    }
    let checkpoint = hjmt
        .journal_checkpoint
        .ok_or(ProofChkErr::JournalCheckpointMix)?;
    let journal_digest = hjmt
        .journal_digest
        .ok_or(ProofChkErr::JournalCheckpointMix)?;
    if hjmt.checkpoint_bind
        != Some(hjmt_checkpoint_bind(
            blob.item().settlement_root(),
            blob.backend_root(),
            checkpoint,
            journal_digest,
        ))
    {
        return Err(ProofChkErr::JournalCheckpointMix);
    }
    if hjmt.default_commitment_version != Some(HJMT_DEFAULT_COMMITMENT_VERSION)
        || hjmt.default_child_commitment != Some(hjmt_default_child_commitment())
    {
        return Err(ProofChkErr::DefaultCommitmentMix);
    }
    if SettlementLeafFamily::from_leaf(&leaf) != hjmt.leaf_family {
        return Err(ProofChkErr::LeafMix);
    }
    check_hjmt_proof_family(hjmt.family)?;

    let expected_bucket = hjmt.bucket_policy.derive_bucket_id(*path);
    let bucket_leaf = hjmt.bucket_root_leaf;
    if bucket_leaf.definition_id != path.definition_id
        || bucket_leaf.serial_id != path.serial_id
        || bucket_leaf.bucket_id != expected_bucket
    {
        return Err(ProofChkErr::BucketMix);
    }
    if bucket_leaf.bucket_policy_id != hjmt.bucket_policy.bucket_policy_id() {
        return Err(ProofChkErr::BucketPolicyMix);
    }

    let bucket_leaf = chk_hjmt_current_bucket_leaf(blob, hjmt, path, def_leaf, ser_leaf)?;

    match hjmt.family {
        HjmtProofFamily::Inclusion => {
            let bucket_leaf = bucket_leaf.ok_or(ProofChkErr::BucketProofMix)?;
            let terminal_proof = dec_branch(blob.terminal_proof())?;
            let payload = leaf_payload(&leaf)?;
            terminal_proof
                .verify_existence(
                    RootHash::from(bucket_leaf.terminal_jmt_root),
                    terminal_key(path.terminal_id()),
                    payload,
                )
                .map_err(|_| ProofChkErr::TerminalProofMix)?;
        }
        HjmtProofFamily::NonExistence => {
            let default_commitment = hjmt.default_commitment.unwrap_or_default();
            if default_commitment != hjmt_default_value_commitment() {
                return Err(ProofChkErr::DefaultCommitmentMix);
            }
            if leaf != hjmt.leaf_family.marker_leaf(*path) {
                return Err(ProofChkErr::LeafMix);
            }
            if let Some(bucket_leaf) = bucket_leaf {
                let terminal_proof = dec_branch(blob.terminal_proof())?;
                terminal_proof
                    .verify_nonexistence(
                        RootHash::from(bucket_leaf.terminal_jmt_root),
                        terminal_key(path.terminal_id()),
                    )
                    .map_err(|_| ProofChkErr::TerminalProofMix)?;
            }
        }
        HjmtProofFamily::Deletion => {
            if let Some(bucket_leaf) = bucket_leaf {
                let terminal_proof = dec_branch(blob.terminal_proof())?;
                terminal_proof
                    .verify_nonexistence(
                        RootHash::from(bucket_leaf.terminal_jmt_root),
                        terminal_key(path.terminal_id()),
                    )
                    .map_err(|_| ProofChkErr::TerminalProofMix)?;
            }

            let prior = hjmt.prior.as_ref().ok_or(ProofChkErr::PriorRootMix)?;
            prior.check_root_bind()?;
            if prior.settlement_root == blob.item().settlement_root() {
                return Err(ProofChkErr::PriorRootMix);
            }
            if prior.definition_root_leaf.definition_id != path.definition_id {
                return Err(ProofChkErr::PriorDefMix);
            }
            if prior.serial_root_leaf.definition_id != path.definition_id
                || prior.serial_root_leaf.serial_id != path.serial_id
            {
                return Err(ProofChkErr::PriorSerMix);
            }
            if prior.bucket_root_leaf.definition_id != path.definition_id
                || prior.bucket_root_leaf.serial_id != path.serial_id
                || prior.bucket_root_leaf.bucket_id != expected_bucket
            {
                return Err(ProofChkErr::PriorBucketMix);
            }
            if prior.bucket_root_leaf.bucket_policy_id != hjmt.bucket_policy.bucket_policy_id() {
                return Err(ProofChkErr::PriorBucketMix);
            }

            let prior_def_proof = dec_branch(&prior.definition_proof)?;
            prior_def_proof
                .verify_existence(
                    RootHash::from(prior.backend_root),
                    definition_key(path.definition_id),
                    prior.definition_root_leaf.encode(),
                )
                .map_err(|_| ProofChkErr::PriorDefProofMix)?;

            let prior_ser_proof = dec_branch(&prior.serial_proof)?;
            prior_ser_proof
                .verify_existence(
                    RootHash::from(prior.definition_root_leaf.definition_root),
                    serial_key(path.definition_id, path.serial_id),
                    prior.serial_root_leaf.encode(),
                )
                .map_err(|_| ProofChkErr::PriorSerProofMix)?;

            let prior_bucket_proof = dec_branch(&prior.bucket_proof)?;
            prior_bucket_proof
                .verify_existence(
                    RootHash::from(prior.serial_root_leaf.serial_root),
                    bucket_key(prior.bucket_root_leaf.bucket_id),
                    prior.bucket_root_leaf.encode(),
                )
                .map_err(|_| ProofChkErr::PriorBucketProofMix)?;

            let prior_terminal_proof = dec_branch(&prior.terminal_proof)?;
            let payload = leaf_payload(&leaf)?;
            prior_terminal_proof
                .verify_existence(
                    RootHash::from(prior.bucket_root_leaf.terminal_jmt_root),
                    terminal_key(path.terminal_id()),
                    payload,
                )
                .map_err(|_| ProofChkErr::PriorTerminalProofMix)?;
        }
    }

    Ok(blob.clone())
}

fn chk_hjmt_current_bucket_leaf(
    blob: &ProofBlob,
    hjmt: &HjmtProofEnvelope,
    path: &SettlementPath,
    def_leaf: DefinitionRootLeaf,
    ser_leaf: SerialRootLeaf,
) -> Result<Option<BucketRootLeaf>, ProofChkErr> {
    let def_proof = dec_branch(blob.definition_proof())?;
    if def_proof
        .verify_existence(
            RootHash::from(blob.backend_root()),
            definition_key(path.definition_id),
            def_leaf.encode(),
        )
        .is_err()
    {
        if hjmt.family == HjmtProofFamily::Inclusion {
            return Err(ProofChkErr::DefProofMix);
        }
        def_proof
            .verify_nonexistence(
                RootHash::from(blob.backend_root()),
                definition_key(path.definition_id),
            )
            .map_err(|_| ProofChkErr::DefProofMix)?;
        return Ok(None);
    }

    let ser_proof = dec_branch(blob.serial_proof())?;
    if ser_proof
        .verify_existence(
            RootHash::from(def_leaf.definition_root),
            serial_key(path.definition_id, path.serial_id),
            ser_leaf.encode(),
        )
        .is_err()
    {
        if hjmt.family == HjmtProofFamily::Inclusion {
            return Err(ProofChkErr::SerProofMix);
        }
        ser_proof
            .verify_nonexistence(
                RootHash::from(def_leaf.definition_root),
                serial_key(path.definition_id, path.serial_id),
            )
            .map_err(|_| ProofChkErr::SerProofMix)?;
        return Ok(None);
    }

    let bucket_leaf = hjmt.bucket_root_leaf;
    let bucket_proof = dec_branch(&hjmt.bucket_proof)?;
    if bucket_proof
        .verify_existence(
            RootHash::from(ser_leaf.serial_root),
            bucket_key(bucket_leaf.bucket_id),
            bucket_leaf.encode(),
        )
        .is_err()
    {
        if hjmt.family == HjmtProofFamily::Inclusion {
            return Err(ProofChkErr::BucketProofMix);
        }
        bucket_proof
            .verify_nonexistence(
                RootHash::from(ser_leaf.serial_root),
                bucket_key(bucket_leaf.bucket_id),
            )
            .map_err(|_| ProofChkErr::BucketProofMix)?;
        return Ok(None);
    }

    Ok(Some(bucket_leaf))
}
