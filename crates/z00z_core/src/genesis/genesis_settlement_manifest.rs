use super::*;

use crate::domains::GenesisManifestHashDomain;
use crate::vouchers::VoucherBackingReferenceV1;
use z00z_utils::io::{load_json_bounded, save_json};

pub const GENESIS_POLICIES_REPLAY_DIGEST_LABEL: &str = "genesis_policies_replay_digest";
pub const GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL: &str = "genesis_policies_roundtrip_digest";
pub const GENESIS_RIGHTS_FILE: &str = "genesis_rights.json";
pub const GENESIS_RIGHTS_REPLAY_DIGEST_LABEL: &str = "genesis_rights_replay_digest";
pub const GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL: &str = "genesis_rights_roundtrip_digest";
pub const GENESIS_SETTLEMENT_MANIFEST_FILE: &str = "genesis_settlement_manifest.json";
pub const GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL: &str = "genesis_vouchers_replay_digest";
pub const GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL: &str = "genesis_vouchers_roundtrip_digest";

const MAX_POLICIES_FILE_SIZE: u64 = 16 * 1024 * 1024;
const MAX_RIGHTS_FILE_SIZE: u64 = 16 * 1024 * 1024;
const MAX_VOUCHERS_FILE_SIZE: u64 = 16 * 1024 * 1024;
const MAX_MANIFEST_FILE_SIZE: u64 = 4 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalCollisionReport {
    pub duplicate_right_terminals: usize,
    pub duplicate_voucher_terminals: usize,
    pub asset_right_collisions: usize,
    pub asset_voucher_collisions: usize,
    pub right_voucher_collisions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenesisSettlementManifest {
    pub version: u32,
    pub network: String,
    pub asset_definition_count: usize,
    pub asset_count: usize,
    pub policy_count: usize,
    pub right_template_count: usize,
    pub right_count: usize,
    pub voucher_count: usize,
    pub leaf_count: usize,
    pub root_generation: u64,
    pub generation_seed_hash: String,
    pub corpus_digest: String,
    pub state_hash: String,
    pub policies_replay_digest: String,
    pub policies_output_roundtrip_digest: String,
    pub deterministic_replay_digest: String,
    pub rights_output_roundtrip_digest: String,
    pub vouchers_replay_digest: String,
    pub vouchers_output_roundtrip_digest: String,
    pub terminal_collision_checks: TerminalCollisionReport,
    pub policies_artifact: String,
    pub rights_artifact: String,
    pub vouchers_artifact: String,
    pub manifest_hash: String,
}

pub(crate) fn load_genesis_settlement_manifest(
    path: &Path,
) -> Result<GenesisSettlementManifest, GenesisError> {
    load_json_bounded(path, MAX_MANIFEST_FILE_SIZE).map_err(|error| {
        GenesisError::ConfigLoadFailed(format!(
            "failed to load strict genesis settlement manifest {}: {error}",
            path.display()
        ))
    })
}

fn update_policy_digest(
    hasher: &mut DomainHasher<GenesisManifestHashDomain>,
    record: &GenesisPolicyRecord,
) {
    hasher.update(record.policy_index.to_le_bytes());
    hasher.update(record.policy_id.bytes());
    hasher.update(record.action_pool_id.bytes());
    let action_pool_bytes = record.action_pool.canonical_bytes().unwrap_or_else(|err| {
        unreachable!(
            "validated policy action pool must stay canonical during digest: {}",
            err
        )
    });
    let descriptor_bytes = record.descriptor.canonical_bytes().unwrap_or_else(|err| {
        unreachable!(
            "validated policy descriptor must stay canonical during digest: {}",
            err
        )
    });
    hasher.update(action_pool_bytes);
    hasher.update(descriptor_bytes);
}

fn update_right_digest(
    hasher: &mut DomainHasher<GenesisManifestHashDomain>,
    record: &GenesisRightRecord,
) {
    hasher.update(record.right_id.as_bytes());
    hasher.update(record.right_index.to_le_bytes());
    hasher.update(record.definition_id);
    hasher.update(record.serial_id.to_le_bytes());
    hasher.update(record.domain_name.as_bytes());
    hasher.update(record.metadata_purpose.as_bytes());
    hasher.update([record.leaf.version]);
    hasher.update(record.leaf.terminal_id);
    hasher.update(record.leaf.right_class.as_str().as_bytes());
    hasher.update(record.leaf.issuer_scope);
    hasher.update(record.leaf.provider_scope);
    hasher.update(record.leaf.holder_commitment);
    hasher.update(record.leaf.control_commitment);
    hasher.update(record.leaf.beneficiary_commitment);
    hasher.update(record.leaf.payload_commitment);
    hasher.update(record.leaf.valid_from.to_le_bytes());
    hasher.update(record.leaf.valid_until.to_le_bytes());
    hasher.update(record.leaf.challenge_from.to_le_bytes());
    hasher.update(record.leaf.challenge_until.to_le_bytes());
    hasher.update(record.leaf.use_nonce);
    hasher.update(record.leaf.revocation_policy_id);
    hasher.update(record.leaf.transition_policy_id);
    hasher.update(record.leaf.challenge_policy_id);
    hasher.update(record.leaf.disclosure_policy_id);
    hasher.update(record.leaf.retention_policy_id);
}

fn update_voucher_digest(
    hasher: &mut DomainHasher<GenesisManifestHashDomain>,
    record: &GenesisVoucherRecord,
) {
    hasher.update(record.voucher_index.to_le_bytes());
    hasher.update(record.root_generation.to_le_bytes());
    hasher.update(record.terminal_id);
    hasher.update(record.issuer_commitment);
    hasher.update(record.holder_commitment);
    hasher.update(record.beneficiary_commitment);
    hasher.update(record.refund_target_commitment);
    hasher.update(record.config.id.as_bytes());
    hasher.update(record.config.domain_name.as_bytes());
    hasher.update(record.config.issuer_fixture.as_bytes());
    hasher.update(record.config.holder_fixture.as_bytes());
    hasher.update(record.config.beneficiary_fixture.as_bytes());
    match &record.config.backing {
        VoucherBackingReferenceV1::ReserveCommitment(bytes) => {
            hasher.update(b"reserve_commitment");
            hasher.update(*bytes);
        }
        VoucherBackingReferenceV1::ConsumedAsset {
            definition_id,
            serial_id,
        } => {
            hasher.update(b"consumed_asset");
            hasher.update(*definition_id);
            hasher.update(serial_id.to_le_bytes());
        }
        VoucherBackingReferenceV1::GenesisReserve { reserve_id } => {
            hasher.update(b"genesis_reserve");
            hasher.update(reserve_id.as_bytes());
        }
    }
    hasher.update(record.config.face_value.to_le_bytes());
    hasher.update(record.config.remaining_value.to_le_bytes());
    hasher.update(record.config.policy_id.bytes());
    hasher.update(record.config.action_pool_id.bytes());
    hasher.update([record.config.lifecycle as u8]);
    hasher.update(record.config.validity.valid_from.to_le_bytes());
    hasher.update(record.config.validity.valid_until.to_le_bytes());
    hasher.update([u8::from(record.config.acceptance.receiver_must_accept)]);
    hasher.update([u8::from(record.config.acceptance.allow_reject)]);
    hasher.update(record.config.acceptance.refund_target_fixture.as_bytes());
    hasher.update(record.config.replay_nonce);
    if let Some(bytes) = record.config.disclosure_commitment {
        hasher.update([1]);
        hasher.update(bytes);
    } else {
        hasher.update([0]);
    }
    if let Some(bytes) = record.config.audit_commitment {
        hasher.update([1]);
        hasher.update(bytes);
    } else {
        hasher.update([0]);
    }
}

pub fn compute_genesis_policies_digest(
    records: &[GenesisPolicyRecord],
    label: &'static str,
) -> [u8; 32] {
    let mut hasher = DomainHasher::<GenesisManifestHashDomain>::new_with_label(label);
    for record in records {
        update_policy_digest(&mut hasher, record);
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

pub fn compute_genesis_rights_digest(
    records: &[GenesisRightRecord],
    label: &'static str,
) -> [u8; 32] {
    let mut hasher = DomainHasher::<GenesisManifestHashDomain>::new_with_label(label);
    for record in records {
        update_right_digest(&mut hasher, record);
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

pub fn compute_genesis_vouchers_digest(
    records: &[GenesisVoucherRecord],
    label: &'static str,
) -> [u8; 32] {
    let mut hasher = DomainHasher::<GenesisManifestHashDomain>::new_with_label(label);
    for record in records {
        update_voucher_digest(&mut hasher, record);
    }

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

pub fn compute_genesis_manifest_hash(manifest: &GenesisSettlementManifest) -> [u8; 32] {
    let mut hasher =
        DomainHasher::<GenesisManifestHashDomain>::new_with_label("genesis_manifest_hash");
    hasher.update(manifest.version.to_le_bytes());
    hasher.update(manifest.network.as_bytes());
    hasher.update(manifest.asset_definition_count.to_le_bytes());
    hasher.update(manifest.asset_count.to_le_bytes());
    hasher.update(manifest.policy_count.to_le_bytes());
    hasher.update(manifest.right_template_count.to_le_bytes());
    hasher.update(manifest.right_count.to_le_bytes());
    hasher.update(manifest.voucher_count.to_le_bytes());
    hasher.update(manifest.leaf_count.to_le_bytes());
    hasher.update(manifest.root_generation.to_le_bytes());
    hasher.update(manifest.generation_seed_hash.as_bytes());
    hasher.update(manifest.corpus_digest.as_bytes());
    hasher.update(manifest.state_hash.as_bytes());
    hasher.update(manifest.policies_replay_digest.as_bytes());
    hasher.update(manifest.policies_output_roundtrip_digest.as_bytes());
    hasher.update(manifest.deterministic_replay_digest.as_bytes());
    hasher.update(manifest.rights_output_roundtrip_digest.as_bytes());
    hasher.update(manifest.vouchers_replay_digest.as_bytes());
    hasher.update(manifest.vouchers_output_roundtrip_digest.as_bytes());
    hasher.update(manifest.policies_artifact.as_bytes());
    hasher.update(manifest.rights_artifact.as_bytes());
    hasher.update(manifest.vouchers_artifact.as_bytes());
    hasher.update(
        manifest
            .terminal_collision_checks
            .duplicate_right_terminals
            .to_le_bytes(),
    );
    hasher.update(
        manifest
            .terminal_collision_checks
            .duplicate_voucher_terminals
            .to_le_bytes(),
    );
    hasher.update(
        manifest
            .terminal_collision_checks
            .asset_right_collisions
            .to_le_bytes(),
    );
    hasher.update(
        manifest
            .terminal_collision_checks
            .asset_voucher_collisions
            .to_le_bytes(),
    );
    hasher.update(
        manifest
            .terminal_collision_checks
            .right_voucher_collisions
            .to_le_bytes(),
    );

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

pub fn compute_genesis_seed_hash(genesis_seed: &[u8; 32]) -> [u8; 32] {
    let mut hasher =
        DomainHasher::<GenesisManifestHashDomain>::new_with_label("genesis_generation_seed_hash");
    hasher.update(*genesis_seed);

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash.as_ref()[..32]);
    result
}

pub fn ensure_terminal_collision_free(
    corpus: &GenesisSettlementCorpus,
) -> Result<TerminalCollisionReport, GenesisError> {
    let asset_terminals: BTreeSet<[u8; 32]> = corpus
        .coins
        .iter()
        .chain(corpus.tokens.iter())
        .chain(corpus.nfts.iter())
        .chain(corpus.voids.iter())
        .map(|asset| asset.asset_id())
        .collect();
    let mut seen_right_terminals = BTreeSet::new();
    let mut seen_voucher_terminals = BTreeSet::new();
    let mut report = TerminalCollisionReport::default();
    let mut first_collision = None;

    for record in &corpus.rights {
        if !seen_right_terminals.insert(record.leaf.terminal_id) {
            report.duplicate_right_terminals += 1;
            first_collision.get_or_insert(record.leaf.terminal_id);
        }
        if asset_terminals.contains(&record.leaf.terminal_id) {
            report.asset_right_collisions += 1;
            first_collision.get_or_insert(record.leaf.terminal_id);
        }
    }

    for record in &corpus.vouchers {
        if !seen_voucher_terminals.insert(record.terminal_id) {
            report.duplicate_voucher_terminals += 1;
            first_collision.get_or_insert(record.terminal_id);
        }
        if asset_terminals.contains(&record.terminal_id) {
            report.asset_voucher_collisions += 1;
            first_collision.get_or_insert(record.terminal_id);
        }
        if seen_right_terminals.contains(&record.terminal_id) {
            report.right_voucher_collisions += 1;
            first_collision.get_or_insert(record.terminal_id);
        }
    }

    if report.duplicate_right_terminals > 0
        || report.duplicate_voucher_terminals > 0
        || report.asset_right_collisions > 0
        || report.asset_voucher_collisions > 0
        || report.right_voucher_collisions > 0
    {
        return Err(GenesisError::TerminalCollision {
            terminal_id: first_collision.unwrap_or([0u8; 32]),
            error: format!(
                "duplicate_right_terminals={}, duplicate_voucher_terminals={}, asset_right_collisions={}, asset_voucher_collisions={}, right_voucher_collisions={}",
                report.duplicate_right_terminals,
                report.duplicate_voucher_terminals,
                report.asset_right_collisions,
                report.asset_voucher_collisions,
                report.right_voucher_collisions
            ),
        });
    }

    Ok(report)
}

pub fn export_genesis_settlement_artifacts(
    output_dir: &Path,
    definitions: &[AssetDefinition],
    policies: &[GenesisPolicyRecord],
    corpus: &GenesisSettlementCorpus,
    network_type: ChainType,
    root_generation: u64,
    state_hash: &[u8; 32],
    genesis_seed: &[u8; 32],
) -> Result<(PathBuf, PathBuf), GenesisError> {
    let collision_report = ensure_terminal_collision_free(corpus)?;

    let policies_path = output_dir.join(GENESIS_POLICIES_FILE);
    save_json(&policies_path, &policies.to_vec()).map_err(|e| GenesisError::FileWriteFailed {
        path: policies_path.display().to_string(),
        error: e.to_string(),
    })?;

    let rights_path = output_dir.join(GENESIS_RIGHTS_FILE);
    save_json(&rights_path, &corpus.rights).map_err(|e| GenesisError::FileWriteFailed {
        path: rights_path.display().to_string(),
        error: e.to_string(),
    })?;

    let vouchers_path = output_dir.join(GENESIS_VOUCHERS_FILE);
    save_json(&vouchers_path, &corpus.vouchers).map_err(|e| GenesisError::FileWriteFailed {
        path: vouchers_path.display().to_string(),
        error: e.to_string(),
    })?;

    let roundtrip_policies: Vec<GenesisPolicyRecord> =
        load_json_bounded(&policies_path, MAX_POLICIES_FILE_SIZE).map_err(|e| {
            GenesisError::ConfigLoadFailed(format!(
                "failed to reload policies artifact {}: {}",
                policies_path.display(),
                e
            ))
        })?;

    let roundtrip_rights: Vec<GenesisRightRecord> =
        load_json_bounded(&rights_path, MAX_RIGHTS_FILE_SIZE).map_err(|e| {
            GenesisError::ConfigLoadFailed(format!(
                "failed to reload rights artifact {}: {}",
                rights_path.display(),
                e
            ))
        })?;

    let roundtrip_vouchers: Vec<GenesisVoucherRecord> =
        load_json_bounded(&vouchers_path, MAX_VOUCHERS_FILE_SIZE).map_err(|e| {
            GenesisError::ConfigLoadFailed(format!(
                "failed to reload vouchers artifact {}: {}",
                vouchers_path.display(),
                e
            ))
        })?;

    if roundtrip_policies != policies {
        return Err(GenesisError::SerializationFailed(
            "policies artifact round-trip drifted after export".to_string(),
        ));
    }

    if roundtrip_rights != corpus.rights {
        return Err(GenesisError::SerializationFailed(
            "rights artifact round-trip drifted after export".to_string(),
        ));
    }

    if roundtrip_vouchers != corpus.vouchers {
        return Err(GenesisError::SerializationFailed(
            "vouchers artifact round-trip drifted after export".to_string(),
        ));
    }

    let right_template_count = corpus
        .rights
        .iter()
        .map(|record| record.right_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let corpus_digest = compute_genesis_state_hash(corpus);
    let policies_replay_digest =
        compute_genesis_policies_digest(policies, GENESIS_POLICIES_REPLAY_DIGEST_LABEL);
    let policies_output_roundtrip_digest = compute_genesis_policies_digest(
        &roundtrip_policies,
        GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
    );
    let deterministic_replay_digest =
        compute_genesis_rights_digest(&corpus.rights, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL);
    let rights_output_roundtrip_digest =
        compute_genesis_rights_digest(&roundtrip_rights, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL);
    let vouchers_replay_digest =
        compute_genesis_vouchers_digest(&corpus.vouchers, GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL);
    let vouchers_output_roundtrip_digest = compute_genesis_vouchers_digest(
        &roundtrip_vouchers,
        GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
    );

    let policies_artifact = policies_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(GENESIS_POLICIES_FILE)
        .to_string();
    let rights_artifact = rights_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(GENESIS_RIGHTS_FILE)
        .to_string();
    let vouchers_artifact = vouchers_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(GENESIS_VOUCHERS_FILE)
        .to_string();

    let mut manifest = GenesisSettlementManifest {
        version: 2,
        network: network_type.as_str().to_string(),
        asset_definition_count: definitions.len(),
        asset_count: corpus.total_count(),
        policy_count: policies.len(),
        right_template_count,
        right_count: corpus.total_right_count(),
        voucher_count: corpus.total_voucher_count(),
        leaf_count: corpus.total_leaf_count(),
        root_generation,
        generation_seed_hash: hex::encode(compute_genesis_seed_hash(genesis_seed)),
        corpus_digest: hex::encode(corpus_digest),
        state_hash: hex::encode(state_hash),
        policies_replay_digest: hex::encode(policies_replay_digest),
        policies_output_roundtrip_digest: hex::encode(policies_output_roundtrip_digest),
        deterministic_replay_digest: hex::encode(deterministic_replay_digest),
        rights_output_roundtrip_digest: hex::encode(rights_output_roundtrip_digest),
        vouchers_replay_digest: hex::encode(vouchers_replay_digest),
        vouchers_output_roundtrip_digest: hex::encode(vouchers_output_roundtrip_digest),
        terminal_collision_checks: collision_report,
        policies_artifact,
        rights_artifact,
        vouchers_artifact,
        manifest_hash: String::new(),
    };

    manifest.manifest_hash = hex::encode(compute_genesis_manifest_hash(&manifest));

    let manifest_path = output_dir.join(GENESIS_SETTLEMENT_MANIFEST_FILE);
    save_json(&manifest_path, &manifest).map_err(|e| GenesisError::FileWriteFailed {
        path: manifest_path.display().to_string(),
        error: e.to_string(),
    })?;

    Ok((rights_path, manifest_path))
}
