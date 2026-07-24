//! Authority-pinned version axes and bounded dispatch for checkpoint objects.
//!
//! This module is deliberately data-first: API suffixes never select a decoder.

use std::collections::BTreeSet;

use z00z_crypto::sha256_256;

use crate::CheckpointError;

use super::{
    recursive_circuit::{
        RecursiveCircuitProfileV2, RECURSIVE_CIRCUIT_PROFILE_VERSION_V2,
        RECURSIVE_CIRCUIT_SPEC_VERSION_V2,
    },
    recursive_trace::RecursiveTraceOpcodeV2,
};

pub const CHECKPOINT_VERSION_REGISTRY_API_V2: u16 = 2;
pub const CHECKPOINT_VERSION_REGISTRY_GENERATION_V2: u32 = 7;
pub const RECURSIVE_OBJECT_PREHEADER_BYTES_V2: usize = 48;
pub const RECURSIVE_OBJECT_MAGIC_V2: [u8; 4] = *b"ZCP2";
pub const RECURSIVE_RUNTIME_PROFILE_V2: &str = "checkpoint-contract-client-notary-v2";
pub const RECURSIVE_RUNTIME_PROFILE_GENERATION_V2: u16 = 2;
pub const RECURSIVE_PARAMETER_GENERATION_V2: u32 = 2;
pub const RUNTIME_MANIFEST_MAX_BYTES_V2: usize = 4 * 1024;
pub(crate) const NOVA_PROOF_ENVELOPE_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.nova-proof-envelope.v2";
pub(crate) const NOVA_ENVELOPE_DIGEST_LABEL_V2: &str = "component_digest";
pub(crate) const NOVA_CADENCE_DOMAIN_V2: &str = "z00z.storage.checkpoint.nova-cadence-manifest.v2";
pub(crate) const NOVA_SNAPSHOT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.nova-accumulator-snapshot.v2";
pub(crate) const RECURSIVE_CHECKPOINT_SIDECAR_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.recursive-sidecar.v2";
pub(crate) const RECURSIVE_SIDECAR_DIGEST_LABEL_V2: &str = "canonical_sidecar";
pub(crate) const CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2: &str =
    "z00z.storage.checkpoint.crypto-verification-receipt.v2";
pub(crate) const RECEIPT_DIGEST_LABEL_V2: &str = "canonical_receipt";
const RUNTIME_PROFILE_MANIFEST_MAGIC_V2: [u8; 4] = *b"ZRPM";
const RUNTIME_PROFILE_MANIFEST_CODEC_V2: u16 = 1;
const RUNTIME_PROFILE_BACKEND_V2: &str = "nova-snark";
const RUNTIME_PROFILE_SUITE_V2: &str = "nova-0.73.0-pallas-vesta-spartan-ipa";
const RUNTIME_PROFILE_FIELD_V2: &str = "pallas-vesta";
const RUNTIME_PROFILE_HASH_V2: &str = "sha256";
const RUNTIME_PROFILE_RECURSION_V2: &str = "nova-ivc-spartan-ipa";
const RUNTIME_PROFILE_SPEC_DOMAIN_V2: &str = "z00z.checkpoint.runtime-profile-spec.v2";
const RUNTIME_PROFILE_PARAMETER_DOMAIN_V2: &str = "z00z.checkpoint.runtime-parameter-family.v2";
const RUNTIME_PROFILE_MANIFEST_DOMAIN_V2: &str = "z00z.checkpoint.runtime-profile-manifest.v2";
const VERSION_REGISTRY_DOMAIN_V2: &str = "z00z.checkpoint.version-registry.v2";
const VERSION_REGISTRY_DIGEST_LABEL_V2: &str = "registry_digest";
const RUNTIME_PROFILE_DIGEST_LABEL_V2: &str = "manifest_digest";
const RUNTIME_PROFILE_COMPONENT_LABEL_V2: &str = "component_digest";
pub const RECURSIVE_PROFILE_MANIFEST_DIGEST_V2: [u8; 32] = [
    0xc5, 0x8e, 0x3b, 0x83, 0x41, 0x62, 0x65, 0x73, 0xf9, 0x56, 0xb1, 0xa9, 0xdb, 0x13, 0xb3, 0x0b,
    0xc3, 0xb3, 0xef, 0x33, 0xf7, 0x1b, 0xff, 0x63, 0xff, 0x1e, 0x08, 0x0e, 0x9d, 0x78, 0xe7, 0x1b,
];

/// Literal production pin for the canonical generation-7 registry bytes.
///
/// `authority_pinned` recomputes and compares this value before exposing any
/// row, so changing a row without an explicit generation/pin rotation fails
/// closed in production rather than only in a unit assertion.
pub const CHECKPOINT_VERSION_REGISTRY_DIGEST_V2: [u8; 32] = [
    0x7e, 0x95, 0x08, 0x73, 0x88, 0x15, 0xc6, 0x70, 0x95, 0x57, 0x24, 0x14, 0x4f, 0x72, 0xd2, 0xac,
    0xee, 0x60, 0x66, 0xf2, 0xb8, 0x8a, 0x2f, 0x2f, 0xb1, 0xde, 0x55, 0xf2, 0xcf, 0x72, 0xfb, 0x9b,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RegistryLifecycleV2 {
    LiveReadWrite = 1,
    OfflineReadOnly = 2,
    LocalOnly = 3,
    ReservedUnreachable = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RegistryFramingV2 {
    EmbeddedPreheader = 1,
    TypedLegacyAdapter = 2,
    TypedConfigSchema = 3,
    LocalTyped = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryOperationV2 {
    Read,
    Write,
}

/// The registered checkpoint family selected by its declared framing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RecursiveBoundedObjectV2 {
    CheckpointTransitionStatement = 0x0680_0101,
    CheckpointDaReference = 0x0680_0102,
    CheckpointPublicationEvidence = 0x0680_0103,
    CheckpointLifecycle = 0x0680_0104,
    ArchiveProviderReceipt = 0x0680_0105,
    RetrievalAudit = 0x0680_0106,
    StateSnapshot = 0x0680_0107,
    CheckpointArchiveManifest = 0x0680_0108,
    PruningDecision = 0x0680_0109,
    PostQuantumCheckpointAnchor = 0x0680_010a,
    NovaBlockProof = 0x0690_0101,
    RecursiveCheckpointSidecar = 0x0690_0102,
    CryptographicVerificationReceipt = 0x0690_0103,
    CheckpointConfigHead = 0x0690_0104,
    ConfigMigrationRecord = 0x0690_0105,
    NovaCadenceManifest = 0x0690_0106,
    NovaAccumulatorSnapshot = 0x0690_0107,
    Plonky3BaseProof = 0x0690_0108,
    Plonky3BaseVerificationReceipt = 0x0690_0109,
    RecursiveSecurityBudgetManifest = 0x0690_010a,
    CheckpointContractConfigV2 = 0x0690_0201,
    CheckpointContractConfigV3 = 0x0690_0202,
    WalletBackup = 0x0690_0301,
    WalletBackupHead = 0x0690_0302,
    WalletBackupShardManifest = 0x0690_0303,
    EncryptedReceiptMailboxEntry = 0x0710_0101,
    ReceiptMailboxAdmission = 0x0710_0102,
    ReceiptMailboxActivation = 0x0710_0103,
    ReceiptMailboxReplicaReceipt = 0x0710_0104,
    ReceiptMailboxAck = 0x0710_0105,
    ReceiptMailboxGcTicket = 0x0710_0106,
    ReceiptMailboxRejectReason = 0x0710_0107,
}

impl RecursiveBoundedObjectV2 {
    fn from_type_id(type_id: u32) -> Option<Self> {
        Some(match type_id {
            0x0680_0101 => Self::CheckpointTransitionStatement,
            0x0680_0102 => Self::CheckpointDaReference,
            0x0680_0103 => Self::CheckpointPublicationEvidence,
            0x0680_0104 => Self::CheckpointLifecycle,
            0x0680_0105 => Self::ArchiveProviderReceipt,
            0x0680_0106 => Self::RetrievalAudit,
            0x0680_0107 => Self::StateSnapshot,
            0x0680_0108 => Self::CheckpointArchiveManifest,
            0x0680_0109 => Self::PruningDecision,
            0x0680_010a => Self::PostQuantumCheckpointAnchor,
            0x0690_0101 => Self::NovaBlockProof,
            0x0690_0102 => Self::RecursiveCheckpointSidecar,
            0x0690_0103 => Self::CryptographicVerificationReceipt,
            0x0690_0104 => Self::CheckpointConfigHead,
            0x0690_0105 => Self::ConfigMigrationRecord,
            0x0690_0106 => Self::NovaCadenceManifest,
            0x0690_0107 => Self::NovaAccumulatorSnapshot,
            0x0690_0108 => Self::Plonky3BaseProof,
            0x0690_0109 => Self::Plonky3BaseVerificationReceipt,
            0x0690_010a => Self::RecursiveSecurityBudgetManifest,
            0x0690_0201 => Self::CheckpointContractConfigV2,
            0x0690_0202 => Self::CheckpointContractConfigV3,
            0x0690_0301 => Self::WalletBackup,
            0x0690_0302 => Self::WalletBackupHead,
            0x0690_0303 => Self::WalletBackupShardManifest,
            0x0710_0101 => Self::EncryptedReceiptMailboxEntry,
            0x0710_0102 => Self::ReceiptMailboxAdmission,
            0x0710_0103 => Self::ReceiptMailboxActivation,
            0x0710_0104 => Self::ReceiptMailboxReplicaReceipt,
            0x0710_0105 => Self::ReceiptMailboxAck,
            0x0710_0106 => Self::ReceiptMailboxGcTicket,
            0x0710_0107 => Self::ReceiptMailboxRejectReason,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointVersionRowV2 {
    pub object: RecursiveBoundedObjectV2,
    pub api_owner: &'static str,
    pub framing: RegistryFramingV2,
    pub write_wire_version: Option<u16>,
    pub read_wire_versions: &'static [u16],
    /// Unique cryptographic-domain identity selected by the typed row.
    pub cryptographic_domain: &'static str,
    /// Cryptographic-domain generation, independent of transcript grammar.
    pub cryptographic_domain_generation: u16,
    /// Cryptographic transcript generation, independent of domain and roots.
    pub transcript_generation: Option<u16>,
    /// Settlement/root construction generation, when the object commits one.
    pub root_generation: Option<u16>,
    /// Public-input encoding generation, when the object exposes one.
    pub public_input_encoding_generation: Option<u16>,
    pub max_encoded_len: u64,
    pub config_schema_generation: Option<u16>,
    pub runtime_profile: Option<&'static str>,
    pub runtime_profile_generation: Option<u16>,
    pub runtime_profile_manifest_digest: Option<[u8; 32]>,
    pub authority_generation: u32,
    pub parameter_generation: Option<u32>,
    /// Phase that owns the meaning of this row, independent of its Rust API.
    pub semantic_owner_phase: u16,
    /// Explicit dispatch reachability. These flags are authority metadata and
    /// must agree with lifecycle; callers never infer future activation.
    pub reader_reachable: bool,
    pub writer_reachable: bool,
    pub lifecycle: RegistryLifecycleV2,
    pub activation_boundary: u64,
    pub migration_owner: Option<&'static str>,
    pub reject_mapping: &'static str,
}

/// Canonical, non-dispatchable authority descriptor for the active runtime
/// profile. It is loaded under its own cap before any registry row is trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProfileManifestV2 {
    identifier: String,
    generation: u16,
    config_schemas: Vec<u16>,
    backend: String,
    suite: String,
    field: String,
    hash: String,
    recursion: String,
    circuit_digest: [u8; 32],
    spec_digest: [u8; 32],
    grammar_digest: [u8; 32],
    parameter_digest: [u8; 32],
}

impl RuntimeProfileManifestV2 {
    fn authority_pinned() -> Self {
        let circuit_digest = RecursiveCircuitProfileV2::authority_pinned().digest();
        let grammar_digest = RecursiveTraceOpcodeV2::grammar_digest();
        let spec_digest = digest_parts(
            RUNTIME_PROFILE_SPEC_DOMAIN_V2,
            &[
                &[RECURSIVE_CIRCUIT_PROFILE_VERSION_V2],
                &[RECURSIVE_CIRCUIT_SPEC_VERSION_V2],
                &circuit_digest,
                &grammar_digest,
            ],
        );
        let parameter_digest = digest_parts(
            RUNTIME_PROFILE_PARAMETER_DOMAIN_V2,
            &[
                RUNTIME_PROFILE_BACKEND_V2.as_bytes(),
                RUNTIME_PROFILE_SUITE_V2.as_bytes(),
                RUNTIME_PROFILE_FIELD_V2.as_bytes(),
                RUNTIME_PROFILE_HASH_V2.as_bytes(),
                RUNTIME_PROFILE_RECURSION_V2.as_bytes(),
            ],
        );
        Self {
            identifier: RECURSIVE_RUNTIME_PROFILE_V2.to_string(),
            generation: RECURSIVE_RUNTIME_PROFILE_GENERATION_V2,
            config_schemas: vec![2, 3],
            backend: RUNTIME_PROFILE_BACKEND_V2.to_string(),
            suite: RUNTIME_PROFILE_SUITE_V2.to_string(),
            field: RUNTIME_PROFILE_FIELD_V2.to_string(),
            hash: RUNTIME_PROFILE_HASH_V2.to_string(),
            recursion: RUNTIME_PROFILE_RECURSION_V2.to_string(),
            circuit_digest,
            spec_digest,
            grammar_digest,
            parameter_digest,
        }
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CheckpointError> {
        let mut bytes = Vec::with_capacity(512);
        bytes.extend_from_slice(&RUNTIME_PROFILE_MANIFEST_MAGIC_V2);
        bytes.extend_from_slice(&RUNTIME_PROFILE_MANIFEST_CODEC_V2.to_le_bytes());
        put_manifest_str(&mut bytes, &self.identifier)?;
        bytes.extend_from_slice(&self.generation.to_le_bytes());
        let schema_count = u16::try_from(self.config_schemas.len())
            .map_err(|_| registry_error("too many compatible config schemas"))?;
        bytes.extend_from_slice(&schema_count.to_le_bytes());
        for schema in &self.config_schemas {
            bytes.extend_from_slice(&schema.to_le_bytes());
        }
        for value in [
            self.backend.as_str(),
            self.suite.as_str(),
            self.field.as_str(),
            self.hash.as_str(),
            self.recursion.as_str(),
        ] {
            put_manifest_str(&mut bytes, value)?;
        }
        for digest in [
            self.circuit_digest,
            self.spec_digest,
            self.grammar_digest,
            self.parameter_digest,
        ] {
            bytes.extend_from_slice(&digest);
        }
        if bytes.len() > RUNTIME_MANIFEST_MAX_BYTES_V2 {
            return Err(registry_error("runtime profile manifest exceeds cap"));
        }
        Ok(bytes)
    }

    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() > RUNTIME_MANIFEST_MAX_BYTES_V2 {
            return Err(registry_error("runtime profile manifest exceeds cap"));
        }
        let mut reader = ManifestReader::new(bytes);
        if reader.take_array::<4>()? != RUNTIME_PROFILE_MANIFEST_MAGIC_V2
            || reader.take_u16()? != RUNTIME_PROFILE_MANIFEST_CODEC_V2
        {
            return Err(registry_error("runtime profile manifest header mismatch"));
        }
        let identifier = reader.take_string()?;
        let generation = reader.take_u16()?;
        let schema_count = usize::from(reader.take_u16()?);
        if schema_count == 0 || schema_count > 16 {
            return Err(registry_error("runtime profile schema count is invalid"));
        }
        let mut config_schemas = Vec::with_capacity(schema_count);
        for _ in 0..schema_count {
            config_schemas.push(reader.take_u16()?);
        }
        let manifest = Self {
            identifier,
            generation,
            config_schemas,
            backend: reader.take_string()?,
            suite: reader.take_string()?,
            field: reader.take_string()?,
            hash: reader.take_string()?,
            recursion: reader.take_string()?,
            circuit_digest: reader.take_array()?,
            spec_digest: reader.take_array()?,
            grammar_digest: reader.take_array()?,
            parameter_digest: reader.take_array()?,
        };
        if !reader.is_done() {
            return Err(registry_error(
                "runtime profile manifest has trailing bytes",
            ));
        }
        manifest.validate_authority()?;
        if manifest.canonical_bytes()?.as_slice() != bytes {
            return Err(registry_error("runtime profile manifest is not canonical"));
        }
        Ok(manifest)
    }

    pub fn digest(&self) -> Result<[u8; 32], CheckpointError> {
        let bytes = self.canonical_bytes()?;
        Ok(digest_parts(RUNTIME_PROFILE_MANIFEST_DOMAIN_V2, &[&bytes]))
    }

    fn validate_authority(&self) -> Result<(), CheckpointError> {
        if self != &Self::authority_pinned() {
            return Err(registry_error(
                "runtime profile manifest authority mismatch",
            ));
        }
        Ok(())
    }
}

struct ManifestReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ManifestReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], CheckpointError> {
        let end = self
            .pos
            .checked_add(N)
            .ok_or_else(|| registry_error("runtime profile manifest length overflow"))?;
        let value = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| registry_error("runtime profile manifest is truncated"))?
            .try_into()
            .expect("slice length is fixed");
        self.pos = end;
        Ok(value)
    }

    fn take_u16(&mut self) -> Result<u16, CheckpointError> {
        Ok(u16::from_le_bytes(self.take_array()?))
    }

    fn take_string(&mut self) -> Result<String, CheckpointError> {
        let len = usize::from(self.take_u16()?);
        if len == 0 || len > 256 {
            return Err(registry_error("runtime profile string length is invalid"));
        }
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| registry_error("runtime profile manifest length overflow"))?;
        let bytes = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| registry_error("runtime profile manifest is truncated"))?;
        let value = std::str::from_utf8(bytes)
            .map_err(|_| registry_error("runtime profile string is not UTF-8"))?
            .to_string();
        self.pos = end;
        Ok(value)
    }

    fn is_done(&self) -> bool {
        self.pos == self.bytes.len()
    }
}

fn put_manifest_str(bytes: &mut Vec<u8>, value: &str) -> Result<(), CheckpointError> {
    if value.is_empty() || value.len() > 256 {
        return Err(registry_error("runtime profile string length is invalid"));
    }
    let len = u16::try_from(value.len())
        .map_err(|_| registry_error("runtime profile string length overflow"))?;
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn digest_parts(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    let label = if domain == RUNTIME_PROFILE_MANIFEST_DOMAIN_V2 {
        RUNTIME_PROFILE_DIGEST_LABEL_V2
    } else {
        RUNTIME_PROFILE_COMPONENT_LABEL_V2
    };
    sha256_256(domain, label, parts)
}

const fn cryptographic_domain(object: RecursiveBoundedObjectV2) -> &'static str {
    match object {
        RecursiveBoundedObjectV2::CheckpointTransitionStatement => "z00z.checkpoint.transition.v1",
        RecursiveBoundedObjectV2::CheckpointDaReference => {
            "z00z.storage.checkpoint.da-reference.v1"
        }
        RecursiveBoundedObjectV2::CheckpointPublicationEvidence => {
            "z00z.storage.checkpoint.publication-evidence.v1"
        }
        RecursiveBoundedObjectV2::CheckpointLifecycle => "z00z.storage.checkpoint.lifecycle.v1",
        RecursiveBoundedObjectV2::ArchiveProviderReceipt => {
            "z00z.storage.checkpoint.archive-provider-receipt.v1"
        }
        RecursiveBoundedObjectV2::RetrievalAudit => "z00z.storage.checkpoint.retrieval-audit.v1",
        RecursiveBoundedObjectV2::StateSnapshot => "z00z.storage.checkpoint.state-snapshot.v1",
        RecursiveBoundedObjectV2::CheckpointArchiveManifest => {
            "z00z.storage.checkpoint.archive-manifest.v1"
        }
        RecursiveBoundedObjectV2::PruningDecision => "z00z.storage.checkpoint.pruning-decision.v1",
        RecursiveBoundedObjectV2::PostQuantumCheckpointAnchor => {
            "z00z.storage.checkpoint.pq-anchor.v1"
        }
        RecursiveBoundedObjectV2::NovaBlockProof => NOVA_PROOF_ENVELOPE_DOMAIN_V2,
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar => {
            RECURSIVE_CHECKPOINT_SIDECAR_DOMAIN_V2
        }
        RecursiveBoundedObjectV2::CryptographicVerificationReceipt => {
            CRYPTOGRAPHIC_VERIFICATION_RECEIPT_DOMAIN_V2
        }
        RecursiveBoundedObjectV2::CheckpointConfigHead => "z00z.storage.checkpoint.config-head.v3",
        RecursiveBoundedObjectV2::ConfigMigrationRecord => {
            "z00z.storage.checkpoint.config-migration-record.v3"
        }
        RecursiveBoundedObjectV2::NovaCadenceManifest => NOVA_CADENCE_DOMAIN_V2,
        RecursiveBoundedObjectV2::NovaAccumulatorSnapshot => NOVA_SNAPSHOT_DOMAIN_V2,
        RecursiveBoundedObjectV2::Plonky3BaseProof => {
            "z00z.storage.checkpoint.plonky3.base-proof.v2"
        }
        RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt => {
            "z00z.storage.checkpoint.plonky3.base-verification-receipt.v2"
        }
        RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest => {
            "z00z.storage.checkpoint.plonky3.security-budget.v2"
        }
        RecursiveBoundedObjectV2::CheckpointContractConfigV2 => {
            "z00z.storage.checkpoint.contract-config.v2"
        }
        RecursiveBoundedObjectV2::CheckpointContractConfigV3 => {
            "z00z.storage.checkpoint.contract-config.v3"
        }
        RecursiveBoundedObjectV2::WalletBackup => "z00z.wallet.backup.v5",
        RecursiveBoundedObjectV2::WalletBackupHead => "z00z.wallet.backup-head.v5",
        RecursiveBoundedObjectV2::WalletBackupShardManifest => {
            "z00z.wallet.backup-shard-manifest.v5"
        }
        RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry => {
            "z00z.mailbox.encrypted-receipt-entry.v1"
        }
        RecursiveBoundedObjectV2::ReceiptMailboxAdmission => "z00z.mailbox.admission.v1",
        RecursiveBoundedObjectV2::ReceiptMailboxActivation => "z00z.mailbox.activation.v1",
        RecursiveBoundedObjectV2::ReceiptMailboxReplicaReceipt => "z00z.mailbox.replica-receipt.v1",
        RecursiveBoundedObjectV2::ReceiptMailboxAck => "z00z.mailbox.ack.v1",
        RecursiveBoundedObjectV2::ReceiptMailboxGcTicket => "z00z.mailbox.gc-ticket.v1",
        RecursiveBoundedObjectV2::ReceiptMailboxRejectReason => "z00z.mailbox.reject-reason.v1",
    }
}

const WIRE_V1: &[u16] = &[1];
const WIRE_V2: &[u16] = &[2];
const WIRE_V5: &[u16] = &[5];
const NO_WIRE: &[u16] = &[];

const fn live_v2(
    object: RecursiveBoundedObjectV2,
    owner: &'static str,
    cap: u64,
) -> CheckpointVersionRowV2 {
    CheckpointVersionRowV2 {
        object,
        api_owner: owner,
        framing: RegistryFramingV2::EmbeddedPreheader,
        write_wire_version: Some(2),
        read_wire_versions: WIRE_V2,
        cryptographic_domain: cryptographic_domain(object),
        cryptographic_domain_generation: 2,
        transcript_generation: Some(2),
        root_generation: Some(2),
        public_input_encoding_generation: Some(1),
        max_encoded_len: cap,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: true,
        lifecycle: RegistryLifecycleV2::LiveReadWrite,
        activation_boundary: 0,
        migration_owner: None,
        reject_mapping: "recursive_v2_registry_reject",
    }
}

const fn local_plonky3_v2(
    object: RecursiveBoundedObjectV2,
    owner: &'static str,
    cap: u64,
) -> CheckpointVersionRowV2 {
    CheckpointVersionRowV2 {
        object,
        api_owner: owner,
        framing: RegistryFramingV2::LocalTyped,
        write_wire_version: Some(2),
        read_wire_versions: WIRE_V2,
        cryptographic_domain: cryptographic_domain(object),
        cryptographic_domain_generation: 2,
        transcript_generation: Some(2),
        root_generation: Some(2),
        public_input_encoding_generation: Some(2),
        max_encoded_len: cap,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: true,
        lifecycle: RegistryLifecycleV2::LocalOnly,
        activation_boundary: 1,
        migration_owner: None,
        reject_mapping: "plonky3_local_v2_reject",
    }
}

const fn reserved_mailbox(
    object: RecursiveBoundedObjectV2,
    owner: &'static str,
    cap: u64,
) -> CheckpointVersionRowV2 {
    CheckpointVersionRowV2 {
        object,
        api_owner: owner,
        framing: RegistryFramingV2::EmbeddedPreheader,
        write_wire_version: None,
        read_wire_versions: WIRE_V1,
        cryptographic_domain: cryptographic_domain(object),
        cryptographic_domain_generation: 1,
        transcript_generation: Some(1),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: cap,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: None,
        semantic_owner_phase: 71,
        reader_reachable: false,
        writer_reachable: false,
        lifecycle: RegistryLifecycleV2::ReservedUnreachable,
        activation_boundary: u64::MAX,
        migration_owner: Some("Phase071MailboxActivation"),
        reject_mapping: "reserved_unreachable",
    }
}

const fn reserved_wallet_backup(
    object: RecursiveBoundedObjectV2,
    owner: &'static str,
) -> CheckpointVersionRowV2 {
    CheckpointVersionRowV2 {
        object,
        api_owner: owner,
        framing: RegistryFramingV2::EmbeddedPreheader,
        write_wire_version: None,
        read_wire_versions: WIRE_V5,
        cryptographic_domain: cryptographic_domain(object),
        cryptographic_domain_generation: 5,
        transcript_generation: Some(5),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 0,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: false,
        writer_reachable: false,
        lifecycle: RegistryLifecycleV2::ReservedUnreachable,
        activation_boundary: u64::MAX,
        migration_owner: Some("WalletBackupV5Activation"),
        reject_mapping: "reserved_wallet_backup",
    }
}

const fn inherited_v1(
    object: RecursiveBoundedObjectV2,
    owner: &'static str,
    cap: u64,
    lifecycle: RegistryLifecycleV2,
    migration_owner: Option<&'static str>,
) -> CheckpointVersionRowV2 {
    CheckpointVersionRowV2 {
        object,
        api_owner: owner,
        framing: RegistryFramingV2::TypedLegacyAdapter,
        write_wire_version: if matches!(lifecycle, RegistryLifecycleV2::LiveReadWrite) {
            Some(1)
        } else {
            None
        },
        read_wire_versions: WIRE_V1,
        cryptographic_domain: cryptographic_domain(object),
        cryptographic_domain_generation: 1,
        transcript_generation: Some(1),
        root_generation: if matches!(
            object,
            RecursiveBoundedObjectV2::CheckpointTransitionStatement
        ) {
            Some(1)
        } else {
            None
        },
        public_input_encoding_generation: None,
        max_encoded_len: cap,
        config_schema_generation: None,
        runtime_profile: None,
        runtime_profile_generation: None,
        runtime_profile_manifest_digest: None,
        authority_generation: 1,
        parameter_generation: None,
        semantic_owner_phase: 68,
        reader_reachable: true,
        writer_reachable: matches!(lifecycle, RegistryLifecycleV2::LiveReadWrite),
        lifecycle,
        activation_boundary: 0,
        migration_owner,
        reject_mapping: "typed_legacy_registry_reject",
    }
}

const fn config_schema(
    object: RecursiveBoundedObjectV2,
    owner: &'static str,
    schema: u16,
    lifecycle: RegistryLifecycleV2,
) -> CheckpointVersionRowV2 {
    let is_v3 = matches!(object, RecursiveBoundedObjectV2::CheckpointContractConfigV3);
    CheckpointVersionRowV2 {
        object,
        api_owner: owner,
        framing: RegistryFramingV2::TypedConfigSchema,
        write_wire_version: None,
        read_wire_versions: NO_WIRE,
        cryptographic_domain: cryptographic_domain(object),
        cryptographic_domain_generation: schema,
        transcript_generation: None,
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 256 * 1024,
        config_schema_generation: Some(schema),
        runtime_profile: if is_v3 {
            Some(RECURSIVE_RUNTIME_PROFILE_V2)
        } else {
            None
        },
        runtime_profile_generation: if is_v3 {
            Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2)
        } else {
            None
        },
        runtime_profile_manifest_digest: if is_v3 {
            Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2)
        } else {
            None
        },
        authority_generation: if is_v3 { 2 } else { 1 },
        parameter_generation: if is_v3 {
            Some(RECURSIVE_PARAMETER_GENERATION_V2)
        } else {
            None
        },
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: is_v3,
        lifecycle,
        activation_boundary: 0,
        migration_owner: if is_v3 {
            None
        } else {
            Some("ConfigV3RenameLedger")
        },
        reject_mapping: "typed_config_registry_reject",
    }
}

const REGISTRY_ROWS_V2: &[CheckpointVersionRowV2] = &[
    inherited_v1(
        RecursiveBoundedObjectV2::CheckpointTransitionStatement,
        "CheckpointTransitionStatementV1",
        64 * 1024 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::CheckpointDaReference,
        "CheckpointDaReferenceV1",
        64 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::CheckpointPublicationEvidence,
        "CheckpointPublicationEvidenceV1",
        256 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::CheckpointLifecycle,
        "CheckpointLifecycleV1",
        64 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::ArchiveProviderReceipt,
        "ArchiveProviderReceiptV1",
        256 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::RetrievalAudit,
        "RetrievalAuditV1",
        4 * 1024 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::StateSnapshot,
        "StateSnapshotV1",
        16 * 1024 * 1024,
        RegistryLifecycleV2::LiveReadWrite,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::CheckpointArchiveManifest,
        "CheckpointArchiveManifestV1",
        8 * 1024 * 1024,
        RegistryLifecycleV2::OfflineReadOnly,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::PruningDecision,
        "PruningDecisionV1",
        256 * 1024,
        RegistryLifecycleV2::OfflineReadOnly,
        None,
    ),
    inherited_v1(
        RecursiveBoundedObjectV2::PostQuantumCheckpointAnchor,
        "PostQuantumCheckpointAnchorV1",
        4 * 1024,
        RegistryLifecycleV2::OfflineReadOnly,
        None,
    ),
    live_v2(
        RecursiveBoundedObjectV2::NovaBlockProof,
        "NovaProofEnvelopeV2",
        17 * 1024 * 1024,
    ),
    live_v2(
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
        "RecursiveCheckpointSidecarV2",
        24 * 1024 * 1024,
    ),
    live_v2(
        RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
        "CryptographicVerificationReceiptV2",
        16 * 1024,
    ),
    CheckpointVersionRowV2 {
        object: RecursiveBoundedObjectV2::CheckpointConfigHead,
        api_owner: "CheckpointConfigHeadV3",
        framing: RegistryFramingV2::LocalTyped,
        write_wire_version: Some(1),
        read_wire_versions: WIRE_V1,
        cryptographic_domain: cryptographic_domain(RecursiveBoundedObjectV2::CheckpointConfigHead),
        cryptographic_domain_generation: 3,
        transcript_generation: Some(1),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 8 * 1024,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: true,
        lifecycle: RegistryLifecycleV2::LocalOnly,
        activation_boundary: 0,
        migration_owner: None,
        reject_mapping: "config_head_reject",
    },
    CheckpointVersionRowV2 {
        object: RecursiveBoundedObjectV2::ConfigMigrationRecord,
        api_owner: "ConfigMigrationRecordV3",
        framing: RegistryFramingV2::LocalTyped,
        write_wire_version: Some(1),
        read_wire_versions: WIRE_V1,
        cryptographic_domain: cryptographic_domain(RecursiveBoundedObjectV2::ConfigMigrationRecord),
        cryptographic_domain_generation: 3,
        transcript_generation: Some(1),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 128 * 1024,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: true,
        lifecycle: RegistryLifecycleV2::LocalOnly,
        activation_boundary: 0,
        migration_owner: None,
        reject_mapping: "config_migration_reject",
    },
    CheckpointVersionRowV2 {
        object: RecursiveBoundedObjectV2::NovaCadenceManifest,
        api_owner: "NovaCadenceManifestV2",
        framing: RegistryFramingV2::EmbeddedPreheader,
        write_wire_version: Some(2),
        read_wire_versions: WIRE_V2,
        cryptographic_domain: cryptographic_domain(RecursiveBoundedObjectV2::NovaCadenceManifest),
        cryptographic_domain_generation: 2,
        transcript_generation: Some(2),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 4 * 1024,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: true,
        lifecycle: RegistryLifecycleV2::LiveReadWrite,
        activation_boundary: 1,
        migration_owner: None,
        reject_mapping: "nova_cadence_manifest_reject",
    },
    CheckpointVersionRowV2 {
        object: RecursiveBoundedObjectV2::NovaAccumulatorSnapshot,
        api_owner: "NovaAccumulatorSnapshotV2",
        framing: RegistryFramingV2::LocalTyped,
        write_wire_version: Some(2),
        read_wire_versions: WIRE_V2,
        cryptographic_domain: cryptographic_domain(
            RecursiveBoundedObjectV2::NovaAccumulatorSnapshot,
        ),
        cryptographic_domain_generation: 2,
        transcript_generation: Some(2),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 540 * 1024 * 1024,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: Some(RECURSIVE_PARAMETER_GENERATION_V2),
        semantic_owner_phase: 69,
        reader_reachable: true,
        writer_reachable: true,
        lifecycle: RegistryLifecycleV2::LocalOnly,
        activation_boundary: 1,
        migration_owner: None,
        reject_mapping: "nova_accumulator_snapshot_reject",
    },
    local_plonky3_v2(
        RecursiveBoundedObjectV2::Plonky3BaseProof,
        "Plonky3BaseProofV2",
        32 * 1024 * 1024,
    ),
    local_plonky3_v2(
        RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt,
        "Plonky3BaseVerificationReceiptV2",
        16 * 1024,
    ),
    local_plonky3_v2(
        RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest,
        "RecursiveSecurityBudgetManifestV2",
        16 * 1024,
    ),
    config_schema(
        RecursiveBoundedObjectV2::CheckpointContractConfigV2,
        "CheckpointContractConfigV2",
        2,
        RegistryLifecycleV2::OfflineReadOnly,
    ),
    config_schema(
        RecursiveBoundedObjectV2::CheckpointContractConfigV3,
        "CheckpointContractConfigV3",
        3,
        RegistryLifecycleV2::LiveReadWrite,
    ),
    reserved_wallet_backup(RecursiveBoundedObjectV2::WalletBackup, "WalletBackupV5"),
    reserved_wallet_backup(
        RecursiveBoundedObjectV2::WalletBackupHead,
        "WalletBackupHeadV5",
    ),
    reserved_wallet_backup(
        RecursiveBoundedObjectV2::WalletBackupShardManifest,
        "WalletBackupShardManifestV5",
    ),
    reserved_mailbox(
        RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry,
        "EncryptedReceiptMailboxEntryV1",
        8 * 1024,
    ),
    reserved_mailbox(
        RecursiveBoundedObjectV2::ReceiptMailboxAdmission,
        "ReceiptMailboxAdmissionV1",
        16 * 1024,
    ),
    reserved_mailbox(
        RecursiveBoundedObjectV2::ReceiptMailboxActivation,
        "ReceiptMailboxActivationV1",
        16 * 1024,
    ),
    reserved_mailbox(
        RecursiveBoundedObjectV2::ReceiptMailboxReplicaReceipt,
        "ReceiptMailboxReplicaReceiptV1",
        16 * 1024,
    ),
    reserved_mailbox(
        RecursiveBoundedObjectV2::ReceiptMailboxAck,
        "ReceiptMailboxAckV1",
        8 * 1024,
    ),
    reserved_mailbox(
        RecursiveBoundedObjectV2::ReceiptMailboxGcTicket,
        "ReceiptMailboxGcTicketV1",
        8 * 1024,
    ),
    CheckpointVersionRowV2 {
        object: RecursiveBoundedObjectV2::ReceiptMailboxRejectReason,
        api_owner: "ReceiptMailboxRejectReasonV1",
        framing: RegistryFramingV2::LocalTyped,
        write_wire_version: None,
        read_wire_versions: WIRE_V1,
        cryptographic_domain: cryptographic_domain(
            RecursiveBoundedObjectV2::ReceiptMailboxRejectReason,
        ),
        cryptographic_domain_generation: 1,
        transcript_generation: Some(1),
        root_generation: None,
        public_input_encoding_generation: None,
        max_encoded_len: 256,
        config_schema_generation: Some(3),
        runtime_profile: Some(RECURSIVE_RUNTIME_PROFILE_V2),
        runtime_profile_generation: Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2),
        runtime_profile_manifest_digest: Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2),
        authority_generation: 2,
        parameter_generation: None,
        semantic_owner_phase: 71,
        reader_reachable: false,
        writer_reachable: false,
        lifecycle: RegistryLifecycleV2::ReservedUnreachable,
        activation_boundary: u64::MAX,
        migration_owner: Some("Phase071MailboxActivation"),
        reject_mapping: "reserved_local_outcome_codec_v1",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedRecursivePreheaderV2 {
    pub object: RecursiveBoundedObjectV2,
    pub declared_len: u64,
    pub header_len: usize,
}

#[derive(Debug, Clone)]
pub struct CheckpointVersionRegistryV2 {
    rows: &'static [CheckpointVersionRowV2],
    profile_manifest: RuntimeProfileManifestV2,
    profile_manifest_bytes: Vec<u8>,
    canonical_bytes: Vec<u8>,
    digest: [u8; 32],
}

impl CheckpointVersionRegistryV2 {
    pub fn authority_pinned() -> Result<Self, CheckpointError> {
        let profile_manifest = RuntimeProfileManifestV2::authority_pinned();
        let profile_manifest_bytes = profile_manifest.canonical_bytes()?;
        let loaded = RuntimeProfileManifestV2::decode_canonical(&profile_manifest_bytes)?;
        if loaded != profile_manifest {
            return Err(registry_error("runtime profile manifest load mismatch"));
        }
        let profile_manifest_digest = loaded.digest()?;
        if profile_manifest_digest != RECURSIVE_PROFILE_MANIFEST_DIGEST_V2 {
            return Err(registry_error("runtime profile manifest digest mismatch"));
        }
        validate_rows(REGISTRY_ROWS_V2, profile_manifest_digest)?;
        let canonical_bytes = registry_canonical_bytes(
            REGISTRY_ROWS_V2,
            &profile_manifest_bytes,
            profile_manifest_digest,
        )?;
        let digest = registry_digest(&canonical_bytes);
        if digest != CHECKPOINT_VERSION_REGISTRY_DIGEST_V2 {
            return Err(registry_error("version registry digest pin mismatch"));
        }
        Ok(Self {
            rows: REGISTRY_ROWS_V2,
            profile_manifest,
            profile_manifest_bytes,
            canonical_bytes,
            digest,
        })
    }

    #[must_use]
    pub fn rows(&self) -> &'static [CheckpointVersionRowV2] {
        self.rows
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[must_use]
    pub fn profile_manifest(&self) -> &RuntimeProfileManifestV2 {
        &self.profile_manifest
    }

    #[must_use]
    pub fn profile_manifest_bytes(&self) -> &[u8] {
        &self.profile_manifest_bytes
    }

    /// Return the one canonical authority byte encoding committed by `digest`.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn row(
        &self,
        object: RecursiveBoundedObjectV2,
    ) -> Result<&'static CheckpointVersionRowV2, CheckpointError> {
        self.rows
            .iter()
            .find(|row| row.object == object)
            .ok_or_else(|| registry_error("object is absent from registry"))
    }

    /// Validate the semantic version tuple at the typed owner's boundary.
    /// These axes never select a decoder and remain independent from API and
    /// wire suffixes.
    pub fn validate_semantic_axes(
        &self,
        expected: RecursiveBoundedObjectV2,
        cryptographic_domain_generation: u16,
        transcript_generation: Option<u16>,
        root_generation: Option<u16>,
        public_input_encoding_generation: Option<u16>,
        operation: RegistryOperationV2,
    ) -> Result<&'static CheckpointVersionRowV2, CheckpointError> {
        let row = self.row(expected)?;
        let reachable = match operation {
            RegistryOperationV2::Read => row.reader_reachable,
            RegistryOperationV2::Write => row.writer_reachable,
        };
        if !reachable {
            return Err(registry_error(
                "registry semantic-axis boundary is unreachable",
            ));
        }
        if row.cryptographic_domain_generation != cryptographic_domain_generation
            || row.transcript_generation != transcript_generation
            || row.root_generation != root_generation
            || row.public_input_encoding_generation != public_input_encoding_generation
        {
            return Err(registry_error("registry semantic version axes mismatch"));
        }
        Ok(row)
    }

    /// Validate the trusted tuple supplied by one typed legacy endpoint before
    /// that endpoint allocates or invokes its one exact legacy decoder.
    pub fn validate_typed_legacy(
        &self,
        expected: RecursiveBoundedObjectV2,
        selected_type_id: u32,
        wire_version: u16,
        encoded_len: u64,
        operation: RegistryOperationV2,
    ) -> Result<&'static CheckpointVersionRowV2, CheckpointError> {
        let selected = RecursiveBoundedObjectV2::from_type_id(selected_type_id)
            .ok_or_else(|| registry_error("unknown typed legacy object"))?;
        if selected != expected {
            return Err(registry_error("cross-type typed legacy object"));
        }
        let row = self.row(expected)?;
        if row.framing != RegistryFramingV2::TypedLegacyAdapter {
            return Err(registry_error("object has no typed legacy adapter"));
        }
        if encoded_len > row.max_encoded_len {
            return Err(registry_error("typed legacy object exceeds cap"));
        }
        validate_operation(row, Some(wire_version), operation)?;
        Ok(row)
    }

    /// Validate a typed legacy tuple and cap, then invoke its one owner decoder
    /// exactly once. The registry never probes or retries another codec.
    pub fn decode_typed_legacy<T, F>(
        &self,
        expected: RecursiveBoundedObjectV2,
        selected_type_id: u32,
        wire_version: u16,
        bytes: &[u8],
        operation: RegistryOperationV2,
        decoder: F,
    ) -> Result<T, CheckpointError>
    where
        F: FnOnce(&[u8]) -> Result<T, CheckpointError>,
    {
        let encoded_len = u64::try_from(bytes.len())
            .map_err(|_| registry_error("typed legacy object length overflow"))?;
        self.validate_typed_legacy(
            expected,
            selected_type_id,
            wire_version,
            encoded_len,
            operation,
        )?;
        decoder(bytes)
    }

    /// Validate one authority-selected config schema without probing another
    /// schema or treating the schema number as a portable wire version.
    pub fn validate_config_schema(
        &self,
        expected: RecursiveBoundedObjectV2,
        schema_generation: u16,
        encoded_len: u64,
        operation: RegistryOperationV2,
    ) -> Result<&'static CheckpointVersionRowV2, CheckpointError> {
        let row = self.row(expected)?;
        if row.framing != RegistryFramingV2::TypedConfigSchema {
            return Err(registry_error("object has no typed config schema"));
        }
        if row.config_schema_generation != Some(schema_generation) {
            return Err(registry_error("config schema generation mismatch"));
        }
        if encoded_len > row.max_encoded_len {
            return Err(registry_error("typed config object exceeds cap"));
        }
        validate_operation(row, None, operation)?;
        Ok(row)
    }

    /// Select the exact config owner from its canonical top-level version and
    /// invoke one bounded decoder exactly once, with no schema fallback.
    pub fn decode_config_schema<T, F>(
        &self,
        expected: RecursiveBoundedObjectV2,
        bytes: &[u8],
        operation: RegistryOperationV2,
        decoder: F,
    ) -> Result<T, CheckpointError>
    where
        F: FnOnce(&[u8]) -> Result<T, CheckpointError>,
    {
        let encoded_len = u64::try_from(bytes.len())
            .map_err(|_| registry_error("typed config object length overflow"))?;
        let row = self.row(expected)?;
        if row.framing != RegistryFramingV2::TypedConfigSchema {
            return Err(registry_error("object has no typed config schema"));
        }
        if encoded_len > row.max_encoded_len {
            return Err(registry_error("typed config object exceeds cap"));
        }
        let schema_generation = config_schema_version(bytes)?;
        self.validate_config_schema(expected, schema_generation, encoded_len, operation)?;
        decoder(bytes)
    }

    /// Validate the complete fixed-size header and cap before a payload decoder
    /// may allocate. Reserved rows reject even when all bytes otherwise match.
    pub fn validate_preheader(
        &self,
        bytes: &[u8],
        expected: RecursiveBoundedObjectV2,
    ) -> Result<ValidatedRecursivePreheaderV2, CheckpointError> {
        if bytes.len() < RECURSIVE_OBJECT_PREHEADER_BYTES_V2 {
            return Err(registry_error("truncated recursive object preheader"));
        }
        if bytes[..4] != RECURSIVE_OBJECT_MAGIC_V2 {
            return Err(registry_error("wrong recursive object magic"));
        }
        let type_id = u32::from_le_bytes(bytes[4..8].try_into().expect("fixed header"));
        let object = RecursiveBoundedObjectV2::from_type_id(type_id)
            .ok_or_else(|| registry_error("unknown recursive object type"))?;
        if object != expected {
            return Err(registry_error("cross-type recursive object"));
        }
        let row = self.row(object)?;
        if !matches!(
            row.framing,
            RegistryFramingV2::EmbeddedPreheader | RegistryFramingV2::LocalTyped
        ) {
            return Err(registry_error("object has no embedded preheader"));
        }
        if !row.reader_reachable {
            return Err(registry_error("reserved recursive object is unreachable"));
        }
        let wire = u16::from_le_bytes(bytes[8..10].try_into().expect("fixed header"));
        let cryptographic_domain_generation =
            u16::from_le_bytes(bytes[10..12].try_into().expect("fixed header"));
        let transcript = u16::from_le_bytes(bytes[12..14].try_into().expect("fixed header"));
        let root = u16::from_le_bytes(bytes[14..16].try_into().expect("fixed header"));
        let public_input = u16::from_le_bytes(bytes[16..18].try_into().expect("fixed header"));
        let authority = u32::from_le_bytes(bytes[18..22].try_into().expect("fixed header"));
        let parameter = u32::from_le_bytes(bytes[22..26].try_into().expect("fixed header"));
        let config = u16::from_le_bytes(bytes[26..28].try_into().expect("fixed header"));
        let profile = u16::from_le_bytes(bytes[28..30].try_into().expect("fixed header"));
        let declared_len = u64::from_le_bytes(bytes[30..38].try_into().expect("fixed header"));
        let registry_api = u16::from_le_bytes(bytes[38..40].try_into().expect("fixed header"));
        let registry_generation =
            u32::from_le_bytes(bytes[40..44].try_into().expect("fixed header"));
        let reserved = u32::from_le_bytes(bytes[44..48].try_into().expect("fixed header"));
        if !row.read_wire_versions.contains(&wire)
            || cryptographic_domain_generation != row.cryptographic_domain_generation
            || Some(transcript) != row.transcript_generation
            || root != row.root_generation.unwrap_or_default()
            || public_input != row.public_input_encoding_generation.unwrap_or_default()
            || authority != row.authority_generation
            || parameter != row.parameter_generation.unwrap_or_default()
            || config != row.config_schema_generation.unwrap_or_default()
            || profile != row.runtime_profile_generation.unwrap_or_default()
            || registry_api != CHECKPOINT_VERSION_REGISTRY_API_V2
            || registry_generation != CHECKPOINT_VERSION_REGISTRY_GENERATION_V2
            || reserved != 0
        {
            return Err(registry_error("recursive object version axes mismatch"));
        }
        self.validate_semantic_axes(
            expected,
            cryptographic_domain_generation,
            Some(transcript),
            Some(root).filter(|value| *value != 0),
            Some(public_input).filter(|value| *value != 0),
            RegistryOperationV2::Read,
        )?;
        if declared_len > row.max_encoded_len {
            return Err(registry_error(
                "recursive object declared length exceeds cap",
            ));
        }
        let total = RECURSIVE_OBJECT_PREHEADER_BYTES_V2
            .checked_add(usize::try_from(declared_len).map_err(|_| registry_error("length"))?)
            .ok_or_else(|| registry_error("recursive object length overflow"))?;
        if bytes.len() != total {
            return Err(registry_error("recursive object length is not exact"));
        }
        Ok(ValidatedRecursivePreheaderV2 {
            object,
            declared_len,
            header_len: RECURSIVE_OBJECT_PREHEADER_BYTES_V2,
        })
    }

    pub fn encode_preheader(
        &self,
        object: RecursiveBoundedObjectV2,
        payload_len: usize,
    ) -> Result<[u8; RECURSIVE_OBJECT_PREHEADER_BYTES_V2], CheckpointError> {
        let row = self.row(object)?;
        if !matches!(
            row.framing,
            RegistryFramingV2::EmbeddedPreheader | RegistryFramingV2::LocalTyped
        ) {
            return Err(registry_error("object has no embedded preheader"));
        }
        if !row.writer_reachable || row.write_wire_version.is_none() {
            return Err(registry_error("registry row has no active writer"));
        }
        let payload_len = u64::try_from(payload_len).map_err(|_| registry_error("length"))?;
        if payload_len > row.max_encoded_len {
            return Err(registry_error("recursive object payload exceeds cap"));
        }
        let mut header = [0u8; RECURSIVE_OBJECT_PREHEADER_BYTES_V2];
        header[..4].copy_from_slice(&RECURSIVE_OBJECT_MAGIC_V2);
        header[4..8].copy_from_slice(&(object as u32).to_le_bytes());
        header[8..10].copy_from_slice(&row.write_wire_version.unwrap().to_le_bytes());
        header[10..12].copy_from_slice(&row.cryptographic_domain_generation.to_le_bytes());
        let transcript = row
            .transcript_generation
            .ok_or_else(|| registry_error("preheader row has no transcript generation"))?;
        header[12..14].copy_from_slice(&transcript.to_le_bytes());
        header[14..16].copy_from_slice(&row.root_generation.unwrap_or_default().to_le_bytes());
        header[16..18].copy_from_slice(
            &row.public_input_encoding_generation
                .unwrap_or_default()
                .to_le_bytes(),
        );
        header[18..22].copy_from_slice(&row.authority_generation.to_le_bytes());
        header[22..26].copy_from_slice(&row.parameter_generation.unwrap_or_default().to_le_bytes());
        header[26..28].copy_from_slice(
            &row.config_schema_generation
                .unwrap_or_default()
                .to_le_bytes(),
        );
        header[28..30].copy_from_slice(
            &row.runtime_profile_generation
                .unwrap_or_default()
                .to_le_bytes(),
        );
        header[30..38].copy_from_slice(&payload_len.to_le_bytes());
        header[38..40].copy_from_slice(&CHECKPOINT_VERSION_REGISTRY_API_V2.to_le_bytes());
        header[40..44].copy_from_slice(&CHECKPOINT_VERSION_REGISTRY_GENERATION_V2.to_le_bytes());
        Ok(header)
    }
}

fn config_schema_version(bytes: &[u8]) -> Result<u16, CheckpointError> {
    const PREFIX: &[u8] = b"version: ";
    let line_end = bytes
        .iter()
        .position(|byte| *byte == b'\n')
        .ok_or_else(|| registry_error("config version line is missing"))?;
    let line = &bytes[..line_end];
    let digits = line
        .strip_prefix(PREFIX)
        .ok_or_else(|| registry_error("config version line is not canonical"))?;
    if digits.is_empty()
        || !digits.iter().all(u8::is_ascii_digit)
        || (digits.len() > 1 && digits[0] == b'0')
    {
        return Err(registry_error("config version scalar is not canonical"));
    }
    let text = std::str::from_utf8(digits)
        .map_err(|_| registry_error("config version scalar is not UTF-8"))?;
    text.parse::<u16>()
        .map_err(|_| registry_error("config version scalar is out of range"))
}

fn validate_operation(
    row: &CheckpointVersionRowV2,
    wire_version: Option<u16>,
    operation: RegistryOperationV2,
) -> Result<(), CheckpointError> {
    match operation {
        RegistryOperationV2::Read => {
            if !row.reader_reachable {
                return Err(registry_error("registry row is unreachable"));
            }
            if let Some(wire) = wire_version {
                if !row.read_wire_versions.contains(&wire) {
                    return Err(registry_error("registry read version mismatch"));
                }
            }
        }
        RegistryOperationV2::Write => {
            if !row.writer_reachable {
                return Err(registry_error("registry row has no active writer"));
            }
            if let Some(wire) = wire_version {
                if row.write_wire_version != Some(wire) {
                    return Err(registry_error("registry write version mismatch"));
                }
            } else if row.framing != RegistryFramingV2::TypedConfigSchema {
                return Err(registry_error("wire version is required for writer"));
            }
        }
    }
    Ok(())
}

fn validate_rows(
    rows: &[CheckpointVersionRowV2],
    profile_manifest_digest: [u8; 32],
) -> Result<(), CheckpointError> {
    let mut type_ids = BTreeSet::new();
    let mut api_owners = BTreeSet::new();
    let mut cryptographic_domains = BTreeSet::new();
    let mut prior_type_id = None;
    for row in rows {
        let type_id = row.object as u32;
        if !type_ids.insert(type_id)
            || !api_owners.insert(row.api_owner)
            || !cryptographic_domains.insert(row.cryptographic_domain)
            || row.api_owner.is_empty()
            || row.cryptographic_domain.is_empty()
            || row.reject_mapping.is_empty()
            || (row.max_encoded_len == 0
                && row.lifecycle != RegistryLifecycleV2::ReservedUnreachable)
        {
            return Err(registry_error(
                "duplicate type id/domain identity or invalid cap",
            ));
        }
        if prior_type_id.is_some_and(|prior| prior >= type_id) {
            return Err(registry_error("registry rows are not canonical"));
        }
        prior_type_id = Some(type_id);
        if !row.writer_reachable && row.write_wire_version.is_some() {
            return Err(registry_error("reserved row exposes a writer"));
        }
        if row.lifecycle == RegistryLifecycleV2::ReservedUnreachable
            && row.activation_boundary != u64::MAX
        {
            return Err(registry_error("reserved row activation is reachable"));
        }
        if row.lifecycle != RegistryLifecycleV2::ReservedUnreachable
            && row.activation_boundary == u64::MAX
        {
            return Err(registry_error("live row activation is unreachable"));
        }
        if row.cryptographic_domain_generation == 0
            || row.authority_generation == 0
            || row.parameter_generation == Some(0)
            || row.config_schema_generation == Some(0)
            || row.runtime_profile_generation == Some(0)
            || row.transcript_generation == Some(0)
            || row.root_generation == Some(0)
            || row.public_input_encoding_generation == Some(0)
            || row.semantic_owner_phase == 0
        {
            return Err(registry_error("registry generation is zero"));
        }
        let expected_reachability = match row.lifecycle {
            RegistryLifecycleV2::LiveReadWrite | RegistryLifecycleV2::LocalOnly => (true, true),
            RegistryLifecycleV2::OfflineReadOnly => (true, false),
            RegistryLifecycleV2::ReservedUnreachable => (false, false),
        };
        if (row.reader_reachable, row.writer_reachable) != expected_reachability {
            return Err(registry_error(
                "registry reachability disagrees with lifecycle",
            ));
        }
        if row
            .read_wire_versions
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
            || row.read_wire_versions.contains(&0)
        {
            return Err(registry_error("read wire versions are not canonical"));
        }
        if row
            .write_wire_version
            .is_some_and(|wire| !row.read_wire_versions.contains(&wire))
        {
            return Err(registry_error("writer is absent from read versions"));
        }
        let has_profile = row.runtime_profile.is_some();
        if has_profile != row.runtime_profile_generation.is_some()
            || has_profile != row.runtime_profile_manifest_digest.is_some()
            || row.runtime_profile.is_some_and(str::is_empty)
        {
            return Err(registry_error("runtime profile tuple is incomplete"));
        }
        if row
            .runtime_profile_manifest_digest
            .is_some_and(|digest| digest != profile_manifest_digest)
        {
            return Err(registry_error("runtime profile manifest digest drift"));
        }
        if row.migration_owner.is_some_and(str::is_empty) {
            return Err(registry_error("migration owner is empty"));
        }
        match row.framing {
            RegistryFramingV2::TypedConfigSchema => {
                if !row.read_wire_versions.is_empty()
                    || row.write_wire_version.is_some()
                    || row.config_schema_generation.is_none()
                    || row.transcript_generation.is_some()
                {
                    return Err(registry_error("typed config axes are invalid"));
                }
            }
            RegistryFramingV2::TypedLegacyAdapter => {
                if row.read_wire_versions.is_empty()
                    || row.config_schema_generation.is_some()
                    || row.transcript_generation.is_none()
                {
                    return Err(registry_error("typed legacy axes are invalid"));
                }
            }
            RegistryFramingV2::EmbeddedPreheader | RegistryFramingV2::LocalTyped => {
                if row.read_wire_versions.is_empty() || row.transcript_generation.is_none() {
                    return Err(registry_error("preheader row has no read version"));
                }
            }
        }
    }
    Ok(())
}

fn registry_canonical_bytes(
    rows: &[CheckpointVersionRowV2],
    profile_manifest_bytes: &[u8],
    profile_manifest_digest: [u8; 32],
) -> Result<Vec<u8>, CheckpointError> {
    let mut bytes = Vec::with_capacity(16 * 1024);
    bytes.extend_from_slice(b"ZREG");
    bytes.extend_from_slice(&CHECKPOINT_VERSION_REGISTRY_API_V2.to_le_bytes());
    bytes.extend_from_slice(&CHECKPOINT_VERSION_REGISTRY_GENERATION_V2.to_le_bytes());
    put_registry_bytes(&mut bytes, profile_manifest_bytes)?;
    bytes.extend_from_slice(&profile_manifest_digest);
    let row_count = u32::try_from(rows.len()).map_err(|_| registry_error("too many rows"))?;
    bytes.extend_from_slice(&row_count.to_le_bytes());
    for row in rows {
        bytes.extend_from_slice(&(row.object as u32).to_le_bytes());
        put_registry_bytes(&mut bytes, row.api_owner.as_bytes())?;
        bytes.extend_from_slice(&[row.framing as u8, row.lifecycle as u8]);
        put_registry_u16_option(&mut bytes, row.write_wire_version);
        let read_count = u32::try_from(row.read_wire_versions.len())
            .map_err(|_| registry_error("too many read versions"))?;
        bytes.extend_from_slice(&read_count.to_le_bytes());
        for wire in row.read_wire_versions {
            bytes.extend_from_slice(&wire.to_le_bytes());
        }
        put_registry_bytes(&mut bytes, row.cryptographic_domain.as_bytes())?;
        bytes.extend_from_slice(&row.cryptographic_domain_generation.to_le_bytes());
        put_registry_u16_option(&mut bytes, row.transcript_generation);
        put_registry_u16_option(&mut bytes, row.root_generation);
        put_registry_u16_option(&mut bytes, row.public_input_encoding_generation);
        bytes.extend_from_slice(&row.max_encoded_len.to_le_bytes());
        put_registry_u16_option(&mut bytes, row.config_schema_generation);
        put_registry_str_option(&mut bytes, row.runtime_profile)?;
        put_registry_u16_option(&mut bytes, row.runtime_profile_generation);
        put_registry_digest_option(&mut bytes, row.runtime_profile_manifest_digest);
        bytes.extend_from_slice(&row.authority_generation.to_le_bytes());
        put_registry_u32_option(&mut bytes, row.parameter_generation);
        bytes.extend_from_slice(&row.semantic_owner_phase.to_le_bytes());
        bytes.extend_from_slice(&[
            u8::from(row.reader_reachable),
            u8::from(row.writer_reachable),
        ]);
        bytes.extend_from_slice(&row.activation_boundary.to_le_bytes());
        put_registry_str_option(&mut bytes, row.migration_owner)?;
        put_registry_bytes(&mut bytes, row.reject_mapping.as_bytes())?;
    }
    Ok(bytes)
}

fn registry_digest(canonical_bytes: &[u8]) -> [u8; 32] {
    sha256_256(
        VERSION_REGISTRY_DOMAIN_V2,
        VERSION_REGISTRY_DIGEST_LABEL_V2,
        &[canonical_bytes],
    )
}

fn put_registry_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<(), CheckpointError> {
    let len = u64::try_from(value.len()).map_err(|_| registry_error("registry length overflow"))?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value);
    Ok(())
}

fn put_registry_u16_option(out: &mut Vec<u8>, value: Option<u16>) {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn put_registry_u32_option(out: &mut Vec<u8>, value: Option<u32>) {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn put_registry_str_option(out: &mut Vec<u8>, value: Option<&str>) -> Result<(), CheckpointError> {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        put_registry_bytes(out, value.as_bytes())?;
    }
    Ok(())
}

fn put_registry_digest_option(out: &mut Vec<u8>, value: Option<[u8; 32]>) {
    out.push(u8::from(value.is_some()));
    if let Some(value) = value {
        out.extend_from_slice(&value);
    }
}

fn registry_error(message: impl Into<String>) -> CheckpointError {
    CheckpointError::ContractConfig(format!("version registry reject: {}", message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_OBJECTS: &[RecursiveBoundedObjectV2] = &[
        RecursiveBoundedObjectV2::CheckpointTransitionStatement,
        RecursiveBoundedObjectV2::CheckpointDaReference,
        RecursiveBoundedObjectV2::CheckpointPublicationEvidence,
        RecursiveBoundedObjectV2::CheckpointLifecycle,
        RecursiveBoundedObjectV2::ArchiveProviderReceipt,
        RecursiveBoundedObjectV2::RetrievalAudit,
        RecursiveBoundedObjectV2::StateSnapshot,
        RecursiveBoundedObjectV2::CheckpointArchiveManifest,
        RecursiveBoundedObjectV2::PruningDecision,
        RecursiveBoundedObjectV2::PostQuantumCheckpointAnchor,
        RecursiveBoundedObjectV2::NovaBlockProof,
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
        RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
        RecursiveBoundedObjectV2::CheckpointConfigHead,
        RecursiveBoundedObjectV2::ConfigMigrationRecord,
        RecursiveBoundedObjectV2::NovaCadenceManifest,
        RecursiveBoundedObjectV2::NovaAccumulatorSnapshot,
        RecursiveBoundedObjectV2::Plonky3BaseProof,
        RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt,
        RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest,
        RecursiveBoundedObjectV2::CheckpointContractConfigV2,
        RecursiveBoundedObjectV2::CheckpointContractConfigV3,
        RecursiveBoundedObjectV2::WalletBackup,
        RecursiveBoundedObjectV2::WalletBackupHead,
        RecursiveBoundedObjectV2::WalletBackupShardManifest,
        RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry,
        RecursiveBoundedObjectV2::ReceiptMailboxAdmission,
        RecursiveBoundedObjectV2::ReceiptMailboxActivation,
        RecursiveBoundedObjectV2::ReceiptMailboxReplicaReceipt,
        RecursiveBoundedObjectV2::ReceiptMailboxAck,
        RecursiveBoundedObjectV2::ReceiptMailboxGcTicket,
        RecursiveBoundedObjectV2::ReceiptMailboxRejectReason,
    ];

    const LEGACY_OBJECTS: &[RecursiveBoundedObjectV2] = &[
        RecursiveBoundedObjectV2::CheckpointTransitionStatement,
        RecursiveBoundedObjectV2::CheckpointDaReference,
        RecursiveBoundedObjectV2::CheckpointPublicationEvidence,
        RecursiveBoundedObjectV2::CheckpointLifecycle,
        RecursiveBoundedObjectV2::ArchiveProviderReceipt,
        RecursiveBoundedObjectV2::RetrievalAudit,
        RecursiveBoundedObjectV2::StateSnapshot,
        RecursiveBoundedObjectV2::CheckpointArchiveManifest,
        RecursiveBoundedObjectV2::PruningDecision,
        RecursiveBoundedObjectV2::PostQuantumCheckpointAnchor,
    ];

    const PREHEADER_OBJECTS: &[RecursiveBoundedObjectV2] = &[
        RecursiveBoundedObjectV2::NovaBlockProof,
        RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
        RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
        RecursiveBoundedObjectV2::CheckpointConfigHead,
        RecursiveBoundedObjectV2::ConfigMigrationRecord,
        RecursiveBoundedObjectV2::NovaCadenceManifest,
        RecursiveBoundedObjectV2::NovaAccumulatorSnapshot,
        RecursiveBoundedObjectV2::Plonky3BaseProof,
        RecursiveBoundedObjectV2::Plonky3BaseVerificationReceipt,
        RecursiveBoundedObjectV2::RecursiveSecurityBudgetManifest,
    ];

    const BACKUP_OBJECTS: &[RecursiveBoundedObjectV2] = &[
        RecursiveBoundedObjectV2::WalletBackup,
        RecursiveBoundedObjectV2::WalletBackupHead,
        RecursiveBoundedObjectV2::WalletBackupShardManifest,
    ];

    fn manifest_fixture() -> (Vec<u8>, [u8; 32]) {
        let manifest = RuntimeProfileManifestV2::authority_pinned();
        let bytes = manifest.canonical_bytes().unwrap();
        let digest = manifest.digest().unwrap();
        (bytes, digest)
    }

    fn assert_manifest_change(mut change: impl FnMut(&mut RuntimeProfileManifestV2)) {
        let baseline = RuntimeProfileManifestV2::authority_pinned();
        let baseline_digest = baseline.digest().unwrap();
        let mut changed = baseline;
        change(&mut changed);
        assert_ne!(changed.digest().unwrap(), baseline_digest);
        assert!(
            RuntimeProfileManifestV2::decode_canonical(&changed.canonical_bytes().unwrap())
                .is_err()
        );
    }

    fn assert_digest_change(mut change: impl FnMut(&mut CheckpointVersionRowV2)) {
        let (manifest, manifest_digest) = manifest_fixture();
        let baseline_bytes =
            registry_canonical_bytes(REGISTRY_ROWS_V2, &manifest, manifest_digest).unwrap();
        let baseline = registry_digest(&baseline_bytes);
        let mut rows = REGISTRY_ROWS_V2.to_vec();
        change(&mut rows[0]);
        let changed_bytes = registry_canonical_bytes(&rows, &manifest, manifest_digest).unwrap();
        assert_ne!(registry_digest(&changed_bytes), baseline);
    }

    fn assert_rows_reject(mut change: impl FnMut(&mut CheckpointVersionRowV2)) {
        let (_, manifest_digest) = manifest_fixture();
        let mut rows = REGISTRY_ROWS_V2.to_vec();
        change(&mut rows[0]);
        assert!(validate_rows(&rows, manifest_digest).is_err());
    }

    #[test]
    fn test_profile_manifest_canonical() {
        let manifest = RuntimeProfileManifestV2::authority_pinned();
        let bytes = manifest.canonical_bytes().unwrap();
        assert!(bytes.len() <= RUNTIME_MANIFEST_MAX_BYTES_V2);
        assert_eq!(
            RuntimeProfileManifestV2::decode_canonical(&bytes).unwrap(),
            manifest
        );
        assert_eq!(
            manifest.digest().unwrap(),
            RECURSIVE_PROFILE_MANIFEST_DIGEST_V2
        );

        let mut changed = bytes.clone();
        changed[0] ^= 1;
        assert!(RuntimeProfileManifestV2::decode_canonical(&changed).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(RuntimeProfileManifestV2::decode_canonical(&trailing).is_err());
        assert!(RuntimeProfileManifestV2::decode_canonical(&vec![
            0;
            RUNTIME_MANIFEST_MAX_BYTES_V2 + 1
        ])
        .is_err());

        assert_manifest_change(|value| value.identifier.push('x'));
        assert_manifest_change(|value| value.generation += 1);
        assert_manifest_change(|value| value.config_schemas.push(4));
        assert_manifest_change(|value| value.backend.push('x'));
        assert_manifest_change(|value| value.suite.push('x'));
        assert_manifest_change(|value| value.field.push('x'));
        assert_manifest_change(|value| value.hash.push('x'));
        assert_manifest_change(|value| value.recursion.push('x'));
        assert_manifest_change(|value| value.circuit_digest[0] ^= 1);
        assert_manifest_change(|value| value.spec_digest[0] ^= 1);
        assert_manifest_change(|value| value.grammar_digest[0] ^= 1);
        assert_manifest_change(|value| value.parameter_digest[0] ^= 1);
    }

    #[test]
    fn test_registry_rejects_cross_type() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        assert!(registry
            .encode_preheader(RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry, 0)
            .is_err());

        let header = registry
            .encode_preheader(RecursiveBoundedObjectV2::NovaBlockProof, 0)
            .unwrap();
        assert!(registry
            .validate_preheader(
                &header,
                RecursiveBoundedObjectV2::RecursiveCheckpointSidecar
            )
            .is_err());
    }

    #[test]
    fn test_registry_preheader_caps_decode() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        assert_eq!(
            super::super::contract_config_v3::hex_digest(registry.digest()),
            "7e9508738815c670955724144f72d2acee6066f2b88a2f2fb1de55f2cf72fb9b"
        );
        let mut bytes = registry
            .encode_preheader(
                RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
                3,
            )
            .unwrap()
            .to_vec();
        bytes.extend_from_slice(&[1, 2, 3]);
        let validated = registry
            .validate_preheader(
                &bytes,
                RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
            )
            .unwrap();
        assert_eq!(validated.declared_len, 3);
        bytes.push(4);
        assert!(registry
            .validate_preheader(
                &bytes,
                RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
            )
            .is_err());

        let mut wrong_version = registry
            .encode_preheader(RecursiveBoundedObjectV2::NovaBlockProof, 0)
            .unwrap();
        wrong_version[8..10].copy_from_slice(&1_u16.to_le_bytes());
        assert!(registry
            .validate_preheader(&wrong_version, RecursiveBoundedObjectV2::NovaBlockProof)
            .is_err());

        for range in [
            10..12,
            12..14,
            14..16,
            16..18,
            18..22,
            22..26,
            26..28,
            28..30,
            30..38,
            38..40,
            40..44,
            44..48,
        ] {
            let mut mutated = registry
                .encode_preheader(RecursiveBoundedObjectV2::NovaBlockProof, 0)
                .unwrap();
            mutated[range.start] ^= 1;
            assert!(
                registry
                    .validate_preheader(&mutated, RecursiveBoundedObjectV2::NovaBlockProof)
                    .is_err(),
                "preheader boundary mutation at {:?} must reject",
                range
            );
        }

        let over_cap_payload = 17 * 1024 * 1024 + 1;
        let mut over_cap = registry
            .encode_preheader(RecursiveBoundedObjectV2::NovaBlockProof, 0)
            .unwrap()
            .to_vec();
        over_cap[30..38].copy_from_slice(&(over_cap_payload as u64).to_le_bytes());
        over_cap.resize(RECURSIVE_OBJECT_PREHEADER_BYTES_V2 + over_cap_payload, 0);
        assert!(registry
            .validate_preheader(&over_cap, RecursiveBoundedObjectV2::NovaBlockProof)
            .is_err());
    }

    #[test]
    fn test_registry_rows_complete() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        assert_eq!(registry.rows().len(), 32);
        assert_eq!(
            registry
                .rows()
                .iter()
                .map(|row| row.object)
                .collect::<Vec<_>>(),
            ALL_OBJECTS
        );
        assert!(registry.rows().iter().all(|row| {
            !row.read_wire_versions.is_empty()
                || row.framing == RegistryFramingV2::TypedConfigSchema
        }));
        assert_eq!(
            registry
                .rows()
                .iter()
                .map(|row| row.object as u32)
                .collect::<BTreeSet<_>>()
                .len(),
            registry.rows().len()
        );

        for object in LEGACY_OBJECTS {
            let row = registry.row(*object).unwrap();
            assert_eq!(row.framing, RegistryFramingV2::TypedLegacyAdapter);
            assert_eq!(row.read_wire_versions, WIRE_V1);
            assert_eq!(row.authority_generation, 1);
            assert_eq!(row.runtime_profile, None);
            assert_eq!(row.runtime_profile_manifest_digest, None);
        }

        for object in PREHEADER_OBJECTS {
            let row = registry.row(*object).unwrap();
            assert!(matches!(
                row.framing,
                RegistryFramingV2::EmbeddedPreheader | RegistryFramingV2::LocalTyped
            ));
            assert_eq!(row.runtime_profile, Some(RECURSIVE_RUNTIME_PROFILE_V2));
            assert_eq!(
                row.runtime_profile_manifest_digest,
                Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2)
            );
            assert_eq!(
                row.parameter_generation,
                Some(RECURSIVE_PARAMETER_GENERATION_V2)
            );
        }

        let config_v2 = registry
            .row(RecursiveBoundedObjectV2::CheckpointContractConfigV2)
            .unwrap();
        assert_eq!(config_v2.lifecycle, RegistryLifecycleV2::OfflineReadOnly);
        assert_eq!(config_v2.migration_owner, Some("ConfigV3RenameLedger"));
        let config_v3 = registry
            .row(RecursiveBoundedObjectV2::CheckpointContractConfigV3)
            .unwrap();
        assert_eq!(config_v3.lifecycle, RegistryLifecycleV2::LiveReadWrite);
        assert_eq!(
            config_v3.runtime_profile_manifest_digest,
            Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2)
        );

        for object in BACKUP_OBJECTS {
            let row = registry.row(*object).unwrap();
            assert_eq!(row.read_wire_versions, WIRE_V5);
            assert_eq!(row.transcript_generation, Some(5));
            assert_eq!(row.max_encoded_len, 0);
            assert_eq!(row.write_wire_version, None);
            assert_eq!(row.lifecycle, RegistryLifecycleV2::ReservedUnreachable);
            assert_eq!(row.activation_boundary, u64::MAX);
            assert_eq!(row.config_schema_generation, Some(3));
            assert_eq!(row.runtime_profile, Some(RECURSIVE_RUNTIME_PROFILE_V2));
            assert_eq!(
                row.runtime_profile_manifest_digest,
                Some(RECURSIVE_PROFILE_MANIFEST_DIGEST_V2)
            );
        }

        let reject = RecursiveBoundedObjectV2::ReceiptMailboxRejectReason;
        let reject_row = registry.row(reject).unwrap();
        assert_eq!(reject_row.read_wire_versions, WIRE_V1);
        assert_eq!(
            reject_row.lifecycle,
            RegistryLifecycleV2::ReservedUnreachable
        );
        assert_eq!(reject_row.semantic_owner_phase, 71);
        assert!(!reject_row.reader_reachable);
        assert!(!reject_row.writer_reachable);
        assert_eq!(reject_row.config_schema_generation, Some(3));
        assert_eq!(
            reject_row.runtime_profile_generation,
            Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2)
        );
        assert_eq!(reject_row.authority_generation, 2);
        assert_eq!(reject_row.reject_mapping, "reserved_local_outcome_codec_v1");

        for object in [
            RecursiveBoundedObjectV2::NovaBlockProof,
            RecursiveBoundedObjectV2::RecursiveCheckpointSidecar,
            RecursiveBoundedObjectV2::CryptographicVerificationReceipt,
        ] {
            let row = registry.row(object).unwrap();
            assert_eq!(row.transcript_generation, Some(2));
            assert_eq!(row.root_generation, Some(2));
            assert_eq!(row.public_input_encoding_generation, Some(1));
            assert_eq!(row.semantic_owner_phase, 69);
        }

        for object in [
            RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry,
            RecursiveBoundedObjectV2::ReceiptMailboxAdmission,
            RecursiveBoundedObjectV2::ReceiptMailboxActivation,
            RecursiveBoundedObjectV2::ReceiptMailboxReplicaReceipt,
            RecursiveBoundedObjectV2::ReceiptMailboxAck,
            RecursiveBoundedObjectV2::ReceiptMailboxGcTicket,
            RecursiveBoundedObjectV2::ReceiptMailboxRejectReason,
        ] {
            let row = registry.row(object).unwrap();
            assert_eq!(row.semantic_owner_phase, 71);
            assert_eq!(row.config_schema_generation, Some(3));
            assert_eq!(
                row.runtime_profile_generation,
                Some(RECURSIVE_RUNTIME_PROFILE_GENERATION_V2)
            );
            assert_eq!(row.authority_generation, 2);
            assert!(!row.reader_reachable);
            assert!(!row.writer_reachable);
            assert_eq!(row.migration_owner, Some("Phase071MailboxActivation"));
        }
        assert_eq!(
            registry
                .row(RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry)
                .unwrap()
                .max_encoded_len,
            8_192
        );

        for row in registry
            .rows()
            .iter()
            .filter(|row| row.lifecycle == RegistryLifecycleV2::ReservedUnreachable)
        {
            assert!(registry.encode_preheader(row.object, 0).is_err());
            let mut header = [0_u8; RECURSIVE_OBJECT_PREHEADER_BYTES_V2];
            header[..4].copy_from_slice(&RECURSIVE_OBJECT_MAGIC_V2);
            header[4..8].copy_from_slice(&(row.object as u32).to_le_bytes());
            assert!(registry.validate_preheader(&header, row.object).is_err());
        }
    }

    #[test]
    fn test_legacy_dispatch_cross_product() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        for expected in LEGACY_OBJECTS {
            let row = registry.row(*expected).unwrap();
            assert!(registry
                .validate_typed_legacy(
                    *expected,
                    *expected as u32,
                    1,
                    row.max_encoded_len,
                    RegistryOperationV2::Read,
                )
                .is_ok());
            assert_eq!(
                registry
                    .validate_typed_legacy(
                        *expected,
                        *expected as u32,
                        1,
                        0,
                        RegistryOperationV2::Write,
                    )
                    .is_ok(),
                row.lifecycle == RegistryLifecycleV2::LiveReadWrite
            );
            assert!(
                registry
                    .validate_typed_legacy(
                        *expected,
                        *expected as u32,
                        2,
                        0,
                        RegistryOperationV2::Read,
                    )
                    .is_err()
            );
            assert!(registry
                .validate_typed_legacy(
                    *expected,
                    *expected as u32,
                    1,
                    row.max_encoded_len + 1,
                    RegistryOperationV2::Read,
                )
                .is_err());
            assert!(registry
                .validate_typed_legacy(*expected, u32::MAX, 1, 0, RegistryOperationV2::Read,)
                .is_err());
            assert!(registry.encode_preheader(*expected, 0).is_err());
            for selected in ALL_OBJECTS {
                if selected != expected {
                    assert!(registry
                        .validate_typed_legacy(
                            *expected,
                            *selected as u32,
                            1,
                            0,
                            RegistryOperationV2::Read,
                        )
                        .is_err());
                }
            }
        }
    }

    #[test]
    fn test_single_legacy_decoder() {
        use std::cell::Cell;

        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let calls = Cell::new(0_u8);
        let object = RecursiveBoundedObjectV2::CheckpointDaReference;
        let decoded = registry
            .decode_typed_legacy(
                object,
                object as u32,
                1,
                &[7],
                RegistryOperationV2::Read,
                |bytes| {
                    calls.set(calls.get() + 1);
                    Ok(bytes[0])
                },
            )
            .unwrap();
        assert_eq!(decoded, 7);
        assert_eq!(calls.get(), 1);

        let result = registry.decode_typed_legacy(
            object,
            RecursiveBoundedObjectV2::RetrievalAudit as u32,
            1,
            &[7],
            RegistryOperationV2::Read,
            |_| {
                calls.set(calls.get() + 1);
                Ok(0)
            },
        );
        assert!(result.is_err());
        assert_eq!(calls.get(), 1);

        let row = registry.row(object).unwrap();
        let oversized = vec![0; usize::try_from(row.max_encoded_len).unwrap() + 1];
        let result = registry.decode_typed_legacy(
            object,
            object as u32,
            1,
            &oversized,
            RegistryOperationV2::Read,
            |_| {
                calls.set(calls.get() + 1);
                Ok(0)
            },
        );
        assert!(result.is_err());
        assert_eq!(calls.get(), 1);

        let result: Result<u8, CheckpointError> = registry.decode_typed_legacy(
            object,
            object as u32,
            1,
            &[7],
            RegistryOperationV2::Read,
            |_| {
                calls.set(calls.get() + 1);
                Err(registry_error("owner decoder reject"))
            },
        );
        assert!(result.is_err());
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn test_config_dispatch_cross_product() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let v2 = RecursiveBoundedObjectV2::CheckpointContractConfigV2;
        let v3 = RecursiveBoundedObjectV2::CheckpointContractConfigV3;
        assert!(registry
            .validate_config_schema(v2, 2, 256 * 1024, RegistryOperationV2::Read)
            .is_ok());
        assert!(registry
            .validate_config_schema(v2, 2, 0, RegistryOperationV2::Write)
            .is_err());
        assert!(registry
            .validate_config_schema(v3, 3, 256 * 1024, RegistryOperationV2::Write)
            .is_ok());
        assert!(registry
            .validate_config_schema(v3, 2, 0, RegistryOperationV2::Read)
            .is_err());
        assert!(registry
            .validate_config_schema(v3, 3, 256 * 1024 + 1, RegistryOperationV2::Read)
            .is_err());
        assert!(registry
            .validate_config_schema(
                RecursiveBoundedObjectV2::NovaBlockProof,
                3,
                0,
                RegistryOperationV2::Read,
            )
            .is_err());
        assert!(registry.encode_preheader(v2, 0).is_err());
        assert!(registry.encode_preheader(v3, 0).is_err());
    }

    #[test]
    fn test_single_config_decoder() {
        use std::cell::Cell;

        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let calls = Cell::new(0_u8);
        let v2 = RecursiveBoundedObjectV2::CheckpointContractConfigV2;
        let v3 = RecursiveBoundedObjectV2::CheckpointContractConfigV3;
        let v2_bytes = b"version: 2\nprofile: legacy\n";
        let bytes = b"version: 3\nprofile: pinned\n";
        let decoded = registry
            .decode_config_schema(v2, v2_bytes, RegistryOperationV2::Read, |input| {
                calls.set(calls.get() + 1);
                Ok(input.len())
            })
            .unwrap();
        assert_eq!(decoded, v2_bytes.len());
        assert_eq!(calls.get(), 1);

        let wrong_type =
            registry.decode_config_schema(v3, v2_bytes, RegistryOperationV2::Read, |_| {
                calls.set(calls.get() + 1);
                Ok(0)
            });
        assert!(wrong_type.is_err());
        assert_eq!(calls.get(), 1);

        let decoded = registry
            .decode_config_schema(v3, bytes, RegistryOperationV2::Read, |input| {
                calls.set(calls.get() + 1);
                Ok(input.len())
            })
            .unwrap();
        assert_eq!(decoded, bytes.len());
        assert_eq!(calls.get(), 2);

        for invalid in [
            b"version: 2\n".as_slice(),
            b"version: 03\n".as_slice(),
            b"---\nversion: 3\n".as_slice(),
            b"version: 3 # alias\n".as_slice(),
            b"version: 3".as_slice(),
        ] {
            let result =
                registry.decode_config_schema(v3, invalid, RegistryOperationV2::Read, |_| {
                    calls.set(calls.get() + 1);
                    Ok(0)
                });
            assert!(result.is_err());
        }
        assert_eq!(calls.get(), 2);

        let mut oversized = b"version: 2\n".to_vec();
        oversized.resize(256 * 1024 + 1, b'x');
        let result =
            registry.decode_config_schema(v2, &oversized, RegistryOperationV2::Read, |_| {
                calls.set(calls.get() + 1);
                Ok(0)
            });
        assert!(result.is_err());
        assert_eq!(calls.get(), 2);

        let result: Result<u8, CheckpointError> =
            registry.decode_config_schema(v3, bytes, RegistryOperationV2::Read, |_| {
                calls.set(calls.get() + 1);
                Err(registry_error("owner decoder reject"))
            });
        assert!(result.is_err());
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn test_preheader_cross_product() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        for actual in PREHEADER_OBJECTS {
            let header = registry.encode_preheader(*actual, 0).unwrap();
            for expected in ALL_OBJECTS {
                assert_eq!(
                    registry.validate_preheader(&header, *expected).is_ok(),
                    actual == expected
                );
            }

            let mut unknown = header;
            unknown[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(registry.validate_preheader(&unknown, *actual).is_err());

            let mut wrong_wire = header;
            wrong_wire[8..10].copy_from_slice(&u16::MAX.to_le_bytes());
            assert!(registry.validate_preheader(&wrong_wire, *actual).is_err());
        }
    }

    #[test]
    fn test_registry_digest_complete() {
        assert_digest_change(|row| row.object = RecursiveBoundedObjectV2::CheckpointDaReference);
        assert_digest_change(|row| row.api_owner = "ChangedOwner");
        assert_digest_change(|row| row.framing = RegistryFramingV2::LocalTyped);
        assert_digest_change(|row| row.write_wire_version = None);
        assert_digest_change(|row| row.read_wire_versions = &[1, 2]);
        assert_digest_change(|row| row.cryptographic_domain = "changed");
        assert_digest_change(|row| row.cryptographic_domain_generation += 1);
        assert_digest_change(|row| row.transcript_generation = Some(9));
        assert_digest_change(|row| row.root_generation = None);
        assert_digest_change(|row| row.public_input_encoding_generation = Some(9));
        assert_digest_change(|row| row.max_encoded_len += 1);
        assert_digest_change(|row| row.config_schema_generation = Some(3));
        assert_digest_change(|row| row.runtime_profile = Some("changed"));
        assert_digest_change(|row| row.runtime_profile_generation = Some(3));
        assert_digest_change(|row| row.runtime_profile_manifest_digest = Some([9; 32]));
        assert_digest_change(|row| row.authority_generation += 1);
        assert_digest_change(|row| row.parameter_generation = Some(3));
        assert_digest_change(|row| row.semantic_owner_phase += 1);
        assert_digest_change(|row| row.reader_reachable = false);
        assert_digest_change(|row| row.writer_reachable = false);
        assert_digest_change(|row| row.lifecycle = RegistryLifecycleV2::OfflineReadOnly);
        assert_digest_change(|row| row.activation_boundary += 1);
        assert_digest_change(|row| row.migration_owner = Some("changed"));
        assert_digest_change(|row| row.reject_mapping = "changed");

        let mut none_rows = REGISTRY_ROWS_V2.to_vec();
        none_rows[0].config_schema_generation = None;
        let mut zero_rows = none_rows.clone();
        zero_rows[0].config_schema_generation = Some(0);
        let (manifest, manifest_digest) = manifest_fixture();
        let none_bytes = registry_canonical_bytes(&none_rows, &manifest, manifest_digest).unwrap();
        let zero_bytes = registry_canonical_bytes(&zero_rows, &manifest, manifest_digest).unwrap();
        assert_ne!(registry_digest(&none_bytes), registry_digest(&zero_bytes));

        let mut changed_manifest = manifest.clone();
        changed_manifest[0] ^= 1;
        let baseline_bytes =
            registry_canonical_bytes(REGISTRY_ROWS_V2, &manifest, manifest_digest).unwrap();
        let changed_manifest_bytes =
            registry_canonical_bytes(REGISTRY_ROWS_V2, &changed_manifest, manifest_digest).unwrap();
        assert_ne!(
            registry_digest(&baseline_bytes),
            registry_digest(&changed_manifest_bytes)
        );
        let mut changed_digest = manifest_digest;
        changed_digest[0] ^= 1;
        let changed_digest_bytes =
            registry_canonical_bytes(REGISTRY_ROWS_V2, &manifest, changed_digest).unwrap();
        assert_ne!(
            registry_digest(&baseline_bytes),
            registry_digest(&changed_digest_bytes)
        );
    }

    #[test]
    fn test_registry_axes_fail_closed() {
        assert_rows_reject(|row| {
            row.cryptographic_domain = "z00z.storage.checkpoint.da-reference.v1"
        });
        assert_rows_reject(|row| row.cryptographic_domain_generation = 0);
        assert_rows_reject(|row| row.transcript_generation = Some(0));
        assert_rows_reject(|row| row.root_generation = Some(0));
        assert_rows_reject(|row| row.public_input_encoding_generation = Some(0));
        assert_rows_reject(|row| row.semantic_owner_phase = 0);
        assert_rows_reject(|row| row.reader_reachable = false);
        assert_rows_reject(|row| row.writer_reachable = false);
    }

    #[test]
    fn test_semantic_axes_reject() {
        let registry = CheckpointVersionRegistryV2::authority_pinned().unwrap();
        let object = RecursiveBoundedObjectV2::NovaBlockProof;
        assert!(registry
            .validate_semantic_axes(
                object,
                2,
                Some(2),
                Some(2),
                Some(1),
                RegistryOperationV2::Read,
            )
            .is_ok());
        for tuple in [
            (1, Some(2), Some(2), Some(1)),
            (2, Some(1), Some(2), Some(1)),
            (2, Some(2), Some(1), Some(1)),
            (2, Some(2), Some(2), Some(2)),
            (2, None, Some(2), Some(1)),
        ] {
            assert!(registry
                .validate_semantic_axes(
                    object,
                    tuple.0,
                    tuple.1,
                    tuple.2,
                    tuple.3,
                    RegistryOperationV2::Read,
                )
                .is_err());
        }

        assert!(registry
            .validate_semantic_axes(
                RecursiveBoundedObjectV2::CheckpointContractConfigV3,
                3,
                None,
                None,
                None,
                RegistryOperationV2::Write,
            )
            .is_ok());
        assert!(registry
            .validate_semantic_axes(
                RecursiveBoundedObjectV2::EncryptedReceiptMailboxEntry,
                1,
                Some(1),
                None,
                None,
                RegistryOperationV2::Read,
            )
            .is_err());
    }
}
