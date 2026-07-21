//! Process-pinned identity for the validated genesis authority.
//!
//! The only installation path loads strict bounded authority files, reproduces
//! the complete bootstrap corpus, validates the manifest against that corpus,
//! and then pins an opaque identity for process consumers.

use std::{
    collections::BTreeSet,
    path::Path,
    sync::{Arc, OnceLock},
};

use thiserror::Error;
use z00z_crypto::hash::sha256_256;
use z00z_utils::{logger::NoopLogger, metrics::NoopMetrics};

use super::{
    compute_genesis_manifest_hash, compute_genesis_policies_digest, compute_genesis_rights_digest,
    compute_genesis_seed_hash, compute_genesis_vouchers_digest, generate_genesis_lanes,
    load_genesis_context, load_genesis_settlement_manifest, validator::compute_genesis_state_hash,
    GenesisGenerationPlan, GenesisLaneOutputs, GenesisResolvedContext, GenesisSettlementManifest,
    GENESIS_POLICIES_FILE, GENESIS_POLICIES_REPLAY_DIGEST_LABEL,
    GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL, GENESIS_RIGHTS_FILE,
    GENESIS_RIGHTS_REPLAY_DIGEST_LABEL, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_ROOT_GENERATION, GENESIS_VOUCHERS_FILE, GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL,
    GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
};
use crate::domains::GENESIS_ID_SHA256_DOMAIN_V2;

const NETWORK_ID_SHA256_LABEL_V2: &str = "network_id";
const GENESIS_IDENTITY_SHA256_LABEL_V2: &str = "genesis_identity";
const GENESIS_SETTLEMENT_MANIFEST_VERSION_V2: u32 = 2;

static PROCESS_CHAIN_IDENTITY_V2: OnceLock<GenesisChainIdentityV2> = OnceLock::new();

/// Core-owned chain and genesis identity consumed by portable protocol objects.
///
/// The fields are private so no caller can manufacture a trusted raw-digest
/// shadow. Core startup installs this value only after complete validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GenesisChainIdentityV2 {
    chain_id: u32,
    network_id: [u8; 32],
    genesis_digest: [u8; 32],
}

impl GenesisChainIdentityV2 {
    /// Return the validated genesis chain number.
    #[must_use]
    pub const fn chain_id(self) -> u32 {
        self.chain_id
    }

    /// Return the portable network identity digest.
    #[must_use]
    pub const fn network_id(self) -> [u8; 32] {
        self.network_id
    }

    /// Return the portable genesis settlement digest.
    #[must_use]
    pub const fn genesis_digest(self) -> [u8; 32] {
        self.genesis_digest
    }
}

struct ValidatedGenesisToken<'a> {
    context: &'a GenesisResolvedContext,
    manifest: &'a GenesisSettlementManifest,
}

struct CheckedManifest<'a> {
    context: &'a GenesisResolvedContext,
    manifest: &'a GenesisSettlementManifest,
}

/// Load, fully validate, and process-pin the genesis chain identity.
///
/// The settlement manifest is capped at 4 KiB and rejects unknown, duplicate,
/// nested-unknown, or trailing JSON data. Validation regenerates every full
/// bootstrap lane before any identity is derived or installed.
///
/// # Errors
///
/// Returns [`GenesisChainIdentityError`] when either authority file is invalid,
/// bootstrap reproduction disagrees with the manifest, or another identity is
/// already pinned in this process.
pub fn load_validate_install_chain_identity(
    config_path: &str,
    manifest_path: &str,
) -> Result<&'static GenesisChainIdentityV2, GenesisChainIdentityError> {
    let plan = GenesisGenerationPlan::full_bootstrap();
    let context =
        load_genesis_context(config_path, &plan).map_err(|error| genesis_error("config", error))?;
    let manifest = load_genesis_settlement_manifest(Path::new(manifest_path))
        .map_err(|error| genesis_error("manifest", error))?;
    let checked = validate_manifest(&context, &manifest)?;
    let outputs =
        generate_genesis_lanes(&context, &plan, Arc::new(NoopLogger), Arc::new(NoopMetrics))
            .map_err(|error| genesis_error("full_bootstrap", error))?;
    let token = validate_bootstrap(checked, &outputs)?;

    derive_and_install(token)
}

fn genesis_error(
    stage: &'static str,
    error: super::validator::GenesisError,
) -> GenesisChainIdentityError {
    GenesisChainIdentityError::GenesisValidationFailed {
        stage,
        reason: error.to_string(),
    }
}

/// Derive and pin an identity only from the private full-validation token.
///
/// Keeping the token private makes bypassing full bootstrap validation
/// impossible for callers in this crate or downstream crates.
fn derive_and_install(
    token: ValidatedGenesisToken<'_>,
) -> Result<&'static GenesisChainIdentityV2, GenesisChainIdentityError> {
    let config = &token.context.config;
    let canonical_chain_type = token.context.network_type.as_str();

    let identity = GenesisChainIdentityV2 {
        chain_id: config.chain.id,
        network_id: derive_network_id(
            config.chain.id,
            canonical_chain_type,
            &config.chain.name,
            config.chain.magic_bytes,
        ),
        genesis_digest: compute_genesis_identity_digest_v2(token.manifest)?,
    };
    install_into(&PROCESS_CHAIN_IDENTITY_V2, identity)?;
    PROCESS_CHAIN_IDENTITY_V2
        .get()
        .ok_or(GenesisChainIdentityError::ProcessIdentityUnavailable)
}

fn process_chain_identity() -> Option<&'static GenesisChainIdentityV2> {
    PROCESS_CHAIN_IDENTITY_V2.get()
}

/// Require the process-pinned validated identity.
pub fn require_process_chain_identity(
) -> Result<&'static GenesisChainIdentityV2, GenesisChainIdentityError> {
    process_chain_identity().ok_or(GenesisChainIdentityError::ProcessIdentityUnavailable)
}

/// Compute the portable V2 genesis identity with fixed-width, framed SHA-256.
///
/// The legacy manifest self-hash is deliberately not an input. It is validated
/// separately for compatibility, while this digest uses fixed `u64` counts and
/// raw canonical 32-byte digest fields.
pub(crate) fn compute_genesis_identity_digest_v2(
    manifest: &GenesisSettlementManifest,
) -> Result<[u8; 32], GenesisChainIdentityError> {
    let asset_definition_count =
        manifest_count("asset_definition_count", manifest.asset_definition_count)?;
    let asset_count = manifest_count("asset_count", manifest.asset_count)?;
    let policy_count = manifest_count("policy_count", manifest.policy_count)?;
    let right_template_count =
        manifest_count("right_template_count", manifest.right_template_count)?;
    let right_count = manifest_count("right_count", manifest.right_count)?;
    let voucher_count = manifest_count("voucher_count", manifest.voucher_count)?;
    let leaf_count = manifest_count("leaf_count", manifest.leaf_count)?;
    let duplicate_right_terminals = manifest_count(
        "duplicate_right_terminals",
        manifest.terminal_collision_checks.duplicate_right_terminals,
    )?;
    let duplicate_voucher_terminals = manifest_count(
        "duplicate_voucher_terminals",
        manifest
            .terminal_collision_checks
            .duplicate_voucher_terminals,
    )?;
    let asset_right_collisions = manifest_count(
        "asset_right_collisions",
        manifest.terminal_collision_checks.asset_right_collisions,
    )?;
    let asset_voucher_collisions = manifest_count(
        "asset_voucher_collisions",
        manifest.terminal_collision_checks.asset_voucher_collisions,
    )?;
    let right_voucher_collisions = manifest_count(
        "right_voucher_collisions",
        manifest.terminal_collision_checks.right_voucher_collisions,
    )?;

    let generation_seed_hash =
        decode_manifest_digest("generation_seed_hash", &manifest.generation_seed_hash)?;
    let corpus_digest = decode_manifest_digest("corpus_digest", &manifest.corpus_digest)?;
    let state_hash = decode_manifest_digest("state_hash", &manifest.state_hash)?;
    let policies_replay_digest =
        decode_manifest_digest("policies_replay_digest", &manifest.policies_replay_digest)?;
    let policies_output_roundtrip_digest = decode_manifest_digest(
        "policies_output_roundtrip_digest",
        &manifest.policies_output_roundtrip_digest,
    )?;
    let deterministic_replay_digest = decode_manifest_digest(
        "deterministic_replay_digest",
        &manifest.deterministic_replay_digest,
    )?;
    let rights_output_roundtrip_digest = decode_manifest_digest(
        "rights_output_roundtrip_digest",
        &manifest.rights_output_roundtrip_digest,
    )?;
    let vouchers_replay_digest =
        decode_manifest_digest("vouchers_replay_digest", &manifest.vouchers_replay_digest)?;
    let vouchers_output_roundtrip_digest = decode_manifest_digest(
        "vouchers_output_roundtrip_digest",
        &manifest.vouchers_output_roundtrip_digest,
    )?;

    Ok(sha256_256(
        GENESIS_ID_SHA256_DOMAIN_V2,
        GENESIS_IDENTITY_SHA256_LABEL_V2,
        &[
            &manifest.version.to_le_bytes(),
            manifest.network.as_bytes(),
            &asset_definition_count.to_le_bytes(),
            &asset_count.to_le_bytes(),
            &policy_count.to_le_bytes(),
            &right_template_count.to_le_bytes(),
            &right_count.to_le_bytes(),
            &voucher_count.to_le_bytes(),
            &leaf_count.to_le_bytes(),
            &manifest.root_generation.to_le_bytes(),
            &generation_seed_hash,
            &corpus_digest,
            &state_hash,
            &policies_replay_digest,
            &policies_output_roundtrip_digest,
            &deterministic_replay_digest,
            &rights_output_roundtrip_digest,
            &vouchers_replay_digest,
            &vouchers_output_roundtrip_digest,
            &duplicate_right_terminals.to_le_bytes(),
            &duplicate_voucher_terminals.to_le_bytes(),
            &asset_right_collisions.to_le_bytes(),
            &asset_voucher_collisions.to_le_bytes(),
            &right_voucher_collisions.to_le_bytes(),
            manifest.policies_artifact.as_bytes(),
            manifest.rights_artifact.as_bytes(),
            manifest.vouchers_artifact.as_bytes(),
        ],
    ))
}

fn derive_network_id(
    chain_id: u32,
    canonical_chain_type: &str,
    name: &str,
    magic_bytes: [u8; 4],
) -> [u8; 32] {
    sha256_256(
        GENESIS_ID_SHA256_DOMAIN_V2,
        NETWORK_ID_SHA256_LABEL_V2,
        &[
            &chain_id.to_le_bytes(),
            canonical_chain_type.as_bytes(),
            name.as_bytes(),
            &magic_bytes,
        ],
    )
}

fn validate_manifest<'a>(
    context: &'a GenesisResolvedContext,
    manifest: &'a GenesisSettlementManifest,
) -> Result<CheckedManifest<'a>, GenesisChainIdentityError> {
    validate_manifest_header(context, manifest)?;
    validate_config_counts(context, manifest)?;
    validate_manifest_artifacts(manifest)?;
    compute_genesis_identity_digest_v2(manifest)?;
    validate_legacy_hash(manifest)?;

    Ok(CheckedManifest { context, manifest })
}

fn validate_manifest_header(
    context: &GenesisResolvedContext,
    manifest: &GenesisSettlementManifest,
) -> Result<(), GenesisChainIdentityError> {
    if manifest.version != GENESIS_SETTLEMENT_MANIFEST_VERSION_V2 {
        return Err(GenesisChainIdentityError::ManifestVersionMismatch {
            expected: GENESIS_SETTLEMENT_MANIFEST_VERSION_V2,
            actual: manifest.version,
        });
    }
    if manifest.root_generation != GENESIS_ROOT_GENERATION {
        return Err(GenesisChainIdentityError::ManifestRootGenerationMismatch {
            expected: GENESIS_ROOT_GENERATION,
            actual: manifest.root_generation,
        });
    }
    let canonical_chain_type = context.network_type.as_str();
    if manifest.network != canonical_chain_type {
        return Err(GenesisChainIdentityError::ManifestNetworkMismatch {
            config_network: canonical_chain_type.to_string(),
            manifest_network: manifest.network.clone(),
        });
    }
    if manifest.terminal_collision_checks != Default::default() {
        return Err(GenesisChainIdentityError::ManifestTerminalCollision);
    }
    Ok(())
}

fn validate_config_counts(
    context: &GenesisResolvedContext,
    manifest: &GenesisSettlementManifest,
) -> Result<(), GenesisChainIdentityError> {
    let expected_assets = checked_count_sum(
        "config.assets.policy.serials",
        context
            .config
            .assets
            .iter()
            .map(|asset| u64::from(asset.policy.serials)),
    )?;
    let expected_rights = checked_count_sum(
        "config.rights.count",
        context
            .config
            .rights
            .iter()
            .map(|right| u64::from(right.count)),
    )?;
    let expected_vouchers = usize_to_u64("config.vouchers", context.config.vouchers.len())?;
    let expected_leaves = expected_assets
        .checked_add(expected_rights)
        .and_then(|count| count.checked_add(expected_vouchers))
        .ok_or(GenesisChainIdentityError::ManifestCountOverflow {
            field: "expected_leaf_count",
        })?;
    let expected_counts = [
        (
            "asset_definition_count",
            manifest.asset_definition_count,
            usize_to_u64("config.assets", context.config.assets.len())?,
        ),
        ("asset_count", manifest.asset_count, expected_assets),
        (
            "policy_count",
            manifest.policy_count,
            usize_to_u64("resolved.policies", context.policies.len())?,
        ),
        (
            "right_template_count",
            manifest.right_template_count,
            usize_to_u64(
                "config.rights.unique_ids",
                context
                    .config
                    .rights
                    .iter()
                    .map(|entry| entry.id.as_str())
                    .collect::<BTreeSet<_>>()
                    .len(),
            )?,
        ),
        ("right_count", manifest.right_count, expected_rights),
        ("voucher_count", manifest.voucher_count, expected_vouchers),
        ("leaf_count", manifest.leaf_count, expected_leaves),
    ];
    for (field, actual, expected) in expected_counts {
        let actual = manifest_count(field, actual)?;
        if actual != expected {
            return Err(GenesisChainIdentityError::ManifestCountMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn validate_manifest_artifacts(
    manifest: &GenesisSettlementManifest,
) -> Result<(), GenesisChainIdentityError> {
    for (field, actual, expected) in [
        (
            "policies_artifact",
            manifest.policies_artifact.as_str(),
            GENESIS_POLICIES_FILE,
        ),
        (
            "rights_artifact",
            manifest.rights_artifact.as_str(),
            GENESIS_RIGHTS_FILE,
        ),
        (
            "vouchers_artifact",
            manifest.vouchers_artifact.as_str(),
            GENESIS_VOUCHERS_FILE,
        ),
    ] {
        if actual != expected {
            return Err(GenesisChainIdentityError::ManifestArtifactMismatch {
                field,
                expected,
                actual: actual.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_bootstrap<'a>(
    checked: CheckedManifest<'a>,
    outputs: &GenesisLaneOutputs,
) -> Result<ValidatedGenesisToken<'a>, GenesisChainIdentityError> {
    let context = checked.context;
    let manifest = checked.manifest;

    let definitions = require_lane("asset_definitions", outputs.asset_definitions.as_deref())?;
    let policies = require_lane("policies", outputs.policies.as_deref())?;
    require_lane("assets", outputs.assets.as_ref())?;
    let rights = require_lane("rights", outputs.rights.as_deref())?;
    let vouchers = require_lane("vouchers", outputs.vouchers.as_deref())?;
    let corpus = outputs.combined_corpus();
    let right_template_count = rights
        .iter()
        .map(|record| record.right_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();

    for (field, actual, expected) in [
        (
            "asset_definition_count",
            manifest.asset_definition_count,
            definitions.len(),
        ),
        ("asset_count", manifest.asset_count, corpus.total_count()),
        ("policy_count", manifest.policy_count, policies.len()),
        (
            "right_template_count",
            manifest.right_template_count,
            right_template_count,
        ),
        ("right_count", manifest.right_count, rights.len()),
        ("voucher_count", manifest.voucher_count, vouchers.len()),
        ("leaf_count", manifest.leaf_count, corpus.total_leaf_count()),
    ] {
        let actual = manifest_count(field, actual)?;
        let expected = usize_to_u64(field, expected)?;
        if actual != expected {
            return Err(GenesisChainIdentityError::ManifestCountMismatch {
                field,
                expected,
                actual,
            });
        }
    }

    ensure_manifest_digests(context, manifest, policies, rights, vouchers, &corpus)?;

    Ok(ValidatedGenesisToken { context, manifest })
}

fn validate_legacy_hash(
    manifest: &GenesisSettlementManifest,
) -> Result<(), GenesisChainIdentityError> {
    let expected = hex::encode(compute_genesis_manifest_hash(manifest));
    if manifest.manifest_hash != expected {
        return Err(GenesisChainIdentityError::LegacyManifestDigestMismatch {
            expected,
            stored: manifest.manifest_hash.clone(),
        });
    }
    Ok(())
}

fn require_lane<'a, T: ?Sized>(
    lane: &'static str,
    value: Option<&'a T>,
) -> Result<&'a T, GenesisChainIdentityError> {
    value.ok_or(GenesisChainIdentityError::BootstrapLaneMissing { lane })
}

fn ensure_manifest_digests(
    context: &GenesisResolvedContext,
    manifest: &GenesisSettlementManifest,
    policies: &[super::GenesisPolicyRecord],
    rights: &[super::GenesisRightRecord],
    vouchers: &[super::GenesisVoucherRecord],
    corpus: &super::GenesisSettlementCorpus,
) -> Result<(), GenesisChainIdentityError> {
    let state_hash = compute_genesis_state_hash(corpus);
    let expected = [
        (
            "generation_seed_hash",
            compute_genesis_seed_hash(context.seed.as_bytes()),
        ),
        ("corpus_digest", state_hash),
        ("state_hash", state_hash),
        (
            "policies_replay_digest",
            compute_genesis_policies_digest(policies, GENESIS_POLICIES_REPLAY_DIGEST_LABEL),
        ),
        (
            "policies_output_roundtrip_digest",
            compute_genesis_policies_digest(policies, GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL),
        ),
        (
            "deterministic_replay_digest",
            compute_genesis_rights_digest(rights, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL),
        ),
        (
            "rights_output_roundtrip_digest",
            compute_genesis_rights_digest(rights, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL),
        ),
        (
            "vouchers_replay_digest",
            compute_genesis_vouchers_digest(vouchers, GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL),
        ),
        (
            "vouchers_output_roundtrip_digest",
            compute_genesis_vouchers_digest(vouchers, GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL),
        ),
    ];
    let stored = [
        manifest.generation_seed_hash.as_str(),
        manifest.corpus_digest.as_str(),
        manifest.state_hash.as_str(),
        manifest.policies_replay_digest.as_str(),
        manifest.policies_output_roundtrip_digest.as_str(),
        manifest.deterministic_replay_digest.as_str(),
        manifest.rights_output_roundtrip_digest.as_str(),
        manifest.vouchers_replay_digest.as_str(),
        manifest.vouchers_output_roundtrip_digest.as_str(),
    ];

    for ((field, expected), value) in expected.into_iter().zip(stored) {
        let actual = decode_manifest_digest(field, value)?;
        if actual != expected {
            return Err(GenesisChainIdentityError::ManifestDigestMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

fn install_into(
    slot: &OnceLock<GenesisChainIdentityV2>,
    identity: GenesisChainIdentityV2,
) -> Result<(), GenesisChainIdentityError> {
    if let Some(installed) = slot.get() {
        return if *installed == identity {
            Ok(())
        } else {
            Err(GenesisChainIdentityError::ProcessIdentityMismatch {
                installed: *installed,
                proposed: identity,
            })
        };
    }

    match slot.set(identity) {
        Ok(()) => Ok(()),
        Err(proposed) => {
            let installed = slot
                .get()
                .copied()
                .ok_or(GenesisChainIdentityError::ProcessIdentityUnavailable)?;
            if installed == proposed {
                Ok(())
            } else {
                Err(GenesisChainIdentityError::ProcessIdentityMismatch {
                    installed,
                    proposed,
                })
            }
        }
    }
}

fn manifest_count(field: &'static str, value: usize) -> Result<u64, GenesisChainIdentityError> {
    usize_to_u64(field, value)
}

fn usize_to_u64(field: &'static str, value: usize) -> Result<u64, GenesisChainIdentityError> {
    u64::try_from(value).map_err(|_| GenesisChainIdentityError::ManifestCountOverflow { field })
}

fn checked_count_sum(
    field: &'static str,
    values: impl IntoIterator<Item = u64>,
) -> Result<u64, GenesisChainIdentityError> {
    values.into_iter().try_fold(0_u64, |sum, value| {
        sum.checked_add(value)
            .ok_or(GenesisChainIdentityError::ManifestCountOverflow { field })
    })
}

fn decode_manifest_digest(
    field: &'static str,
    value: &str,
) -> Result<[u8; 32], GenesisChainIdentityError> {
    if value.len() != 64
        || value
            .as_bytes()
            .iter()
            .any(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(GenesisChainIdentityError::ManifestDigestEncoding { field });
    }
    let decoded = hex::decode(value)
        .map_err(|_| GenesisChainIdentityError::ManifestDigestEncoding { field })?;
    decoded
        .try_into()
        .map_err(|_| GenesisChainIdentityError::ManifestDigestEncoding { field })
}

/// Fail-closed errors for genesis chain identity validation and process pinning.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum GenesisChainIdentityError {
    #[error("genesis identity {stage} validation failed: {reason}")]
    GenesisValidationFailed { stage: &'static str, reason: String },
    #[error("full genesis bootstrap omitted required {lane} lane")]
    BootstrapLaneMissing { lane: &'static str },
    #[error("genesis manifest version {actual} does not match required version {expected}")]
    ManifestVersionMismatch { expected: u32, actual: u32 },
    #[error(
        "genesis manifest root generation {actual} does not match required generation {expected}"
    )]
    ManifestRootGenerationMismatch { expected: u64, actual: u64 },
    #[error(
        "genesis manifest network {manifest_network:?} does not match validated config network {config_network:?}"
    )]
    ManifestNetworkMismatch {
        config_network: String,
        manifest_network: String,
    },
    #[error("genesis manifest count {field} exceeds u64")]
    ManifestCountOverflow { field: &'static str },
    #[error("genesis manifest count {field} is {actual}, expected {expected}")]
    ManifestCountMismatch {
        field: &'static str,
        expected: u64,
        actual: u64,
    },
    #[error("genesis manifest field {field} is not exactly 32 lowercase hex bytes")]
    ManifestDigestEncoding { field: &'static str },
    #[error("genesis manifest digest {field} is {actual:?}, expected {expected:?}")]
    ManifestDigestMismatch {
        field: &'static str,
        expected: [u8; 32],
        actual: [u8; 32],
    },
    #[error("genesis manifest reports terminal collisions")]
    ManifestTerminalCollision,
    #[error("genesis manifest artifact {field} is {actual:?}, expected {expected:?}")]
    ManifestArtifactMismatch {
        field: &'static str,
        expected: &'static str,
        actual: String,
    },
    #[error(
        "legacy genesis manifest hash {stored:?} does not match compatibility digest {expected}"
    )]
    LegacyManifestDigestMismatch { expected: String, stored: String },
    #[error("validated genesis chain identity is not installed")]
    ProcessIdentityUnavailable,
    #[error("process genesis identity is already pinned to {installed:?}; rejected {proposed:?}")]
    ProcessIdentityMismatch {
        installed: GenesisChainIdentityV2,
        proposed: GenesisChainIdentityV2,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use z00z_crypto::hash::sha256_256_simple;
    use z00z_utils::{
        codec::{Codec, JsonCodec},
        io::write_file,
    };

    fn push_hash_part(preimage: &mut Vec<u8>, part: &[u8]) {
        preimage.extend_from_slice(&(part.len() as u64).to_le_bytes());
        preimage.extend_from_slice(part);
    }

    fn independent_hash(domain: &str, label: &str, parts: &[&[u8]]) -> [u8; 32] {
        let mut destination = b"z00z.hash.v1\0".to_vec();
        destination.extend_from_slice(&(domain.len() as u64).to_le_bytes());
        destination.extend_from_slice(domain.as_bytes());
        destination.extend_from_slice(&(label.len() as u64).to_le_bytes());
        destination.extend_from_slice(label.as_bytes());

        let mut preimage = Vec::new();
        push_hash_part(&mut preimage, &destination);
        for part in parts {
            push_hash_part(&mut preimage, part);
        }
        sha256_256_simple(&preimage)
    }

    fn identity_parts(value: &GenesisSettlementManifest) -> Vec<Vec<u8>> {
        let mut parts = vec![
            value.version.to_le_bytes().to_vec(),
            value.network.as_bytes().to_vec(),
            (value.asset_definition_count as u64).to_le_bytes().to_vec(),
            (value.asset_count as u64).to_le_bytes().to_vec(),
            (value.policy_count as u64).to_le_bytes().to_vec(),
            (value.right_template_count as u64).to_le_bytes().to_vec(),
            (value.right_count as u64).to_le_bytes().to_vec(),
            (value.voucher_count as u64).to_le_bytes().to_vec(),
            (value.leaf_count as u64).to_le_bytes().to_vec(),
            value.root_generation.to_le_bytes().to_vec(),
        ];
        for digest in [
            &value.generation_seed_hash,
            &value.corpus_digest,
            &value.state_hash,
            &value.policies_replay_digest,
            &value.policies_output_roundtrip_digest,
            &value.deterministic_replay_digest,
            &value.rights_output_roundtrip_digest,
            &value.vouchers_replay_digest,
            &value.vouchers_output_roundtrip_digest,
        ] {
            parts.push(hex::decode(digest).expect("fixture digest"));
        }
        parts.extend([
            (value.terminal_collision_checks.duplicate_right_terminals as u64)
                .to_le_bytes()
                .to_vec(),
            (value.terminal_collision_checks.duplicate_voucher_terminals as u64)
                .to_le_bytes()
                .to_vec(),
            (value.terminal_collision_checks.asset_right_collisions as u64)
                .to_le_bytes()
                .to_vec(),
            (value.terminal_collision_checks.asset_voucher_collisions as u64)
                .to_le_bytes()
                .to_vec(),
            (value.terminal_collision_checks.right_voucher_collisions as u64)
                .to_le_bytes()
                .to_vec(),
            value.policies_artifact.as_bytes().to_vec(),
            value.rights_artifact.as_bytes().to_vec(),
            value.vouchers_artifact.as_bytes().to_vec(),
        ]);
        parts
    }

    fn borrow_parts(parts: &[Vec<u8>]) -> Vec<&[u8]> {
        parts.iter().map(Vec::as_slice).collect()
    }

    fn manifest_json() -> String {
        let bytes = JsonCodec.serialize(&manifest()).expect("serialize fixture");
        String::from_utf8(bytes).expect("JSON is UTF-8")
    }

    fn load_test_json(
        json: &str,
    ) -> Result<GenesisSettlementManifest, super::super::validator::GenesisError> {
        let temp = TempDir::new().expect("temp directory");
        let path = temp
            .path()
            .join(super::super::GENESIS_SETTLEMENT_MANIFEST_FILE);
        write_file(&path, json.as_bytes()).expect("write manifest fixture");
        load_genesis_settlement_manifest(&path)
    }

    fn identity(byte: u8) -> GenesisChainIdentityV2 {
        GenesisChainIdentityV2 {
            chain_id: u32::from(byte),
            network_id: [byte; 32],
            genesis_digest: [byte.wrapping_add(1); 32],
        }
    }

    fn manifest() -> GenesisSettlementManifest {
        GenesisSettlementManifest {
            version: 2,
            network: "devnet".to_string(),
            asset_definition_count: 1,
            asset_count: 2,
            policy_count: 3,
            right_template_count: 4,
            right_count: 5,
            voucher_count: 6,
            leaf_count: 13,
            root_generation: 1,
            generation_seed_hash: "01".repeat(32),
            corpus_digest: "02".repeat(32),
            state_hash: "03".repeat(32),
            policies_replay_digest: "04".repeat(32),
            policies_output_roundtrip_digest: "05".repeat(32),
            deterministic_replay_digest: "06".repeat(32),
            rights_output_roundtrip_digest: "07".repeat(32),
            vouchers_replay_digest: "08".repeat(32),
            vouchers_output_roundtrip_digest: "09".repeat(32),
            terminal_collision_checks: Default::default(),
            policies_artifact: GENESIS_POLICIES_FILE.to_string(),
            rights_artifact: GENESIS_RIGHTS_FILE.to_string(),
            vouchers_artifact: GENESIS_VOUCHERS_FILE.to_string(),
            manifest_hash: "0a".repeat(32),
        }
    }

    fn load_test_context() -> GenesisResolvedContext {
        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("configs")
            .join("devnet_genesis_config.yaml");
        let config_path = config_path.to_str().expect("UTF-8 config path");
        load_genesis_context(config_path, &GenesisGenerationPlan::full_bootstrap())
            .expect("validated devnet fixture")
    }

    fn preflight_manifest(context: &GenesisResolvedContext) -> GenesisSettlementManifest {
        let mut value = manifest();
        value.network = context.network_type.as_str().to_string();
        value.asset_definition_count = context.config.assets.len();
        value.asset_count = context
            .config
            .assets
            .iter()
            .map(|asset| asset.policy.serials as usize)
            .sum();
        value.policy_count = context.policies.len();
        value.right_template_count = context
            .config
            .rights
            .iter()
            .map(|right| right.id.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        value.right_count = context
            .config
            .rights
            .iter()
            .map(|right| right.count as usize)
            .sum();
        value.voucher_count = context.config.vouchers.len();
        value.leaf_count = value.asset_count + value.right_count + value.voucher_count;
        value.root_generation = GENESIS_ROOT_GENERATION;
        value.manifest_hash = hex::encode(compute_genesis_manifest_hash(&value));
        value
    }

    #[test]
    fn test_network_identity_golden() {
        let chain_id = 7_u32.to_le_bytes();
        let magic = [0x5a, 0x30, 0x30, 0x5a];
        let independent = independent_hash(
            GENESIS_ID_SHA256_DOMAIN_V2,
            NETWORK_ID_SHA256_LABEL_V2,
            &[&chain_id, b"devnet", b"z00z-dev", &magic],
        );
        assert_eq!(
            derive_network_id(7, "devnet", "z00z-dev", [0x5a, 0x30, 0x30, 0x5a]),
            [
                0xbc, 0xea, 0x51, 0xdd, 0xb3, 0xab, 0x54, 0x9f, 0xee, 0xcb, 0x89, 0x29, 0xf1, 0xa3,
                0xd9, 0xc0, 0xc3, 0xfb, 0x0b, 0x51, 0x24, 0x6a, 0xb8, 0xa0, 0x62, 0x2a, 0x0f, 0xa8,
                0x34, 0xdf, 0xd6, 0x10,
            ]
        );
        assert_eq!(
            derive_network_id(7, "devnet", "z00z-dev", magic),
            independent
        );
    }

    #[test]
    fn test_loader_accepts_strict_manifest() {
        assert_eq!(
            load_test_json(&manifest_json()).expect("strict manifest"),
            manifest()
        );
    }

    #[test]
    fn test_loader_unknown_root_rejects() {
        let changed = manifest_json().replacen('{', "{\"unknown_root\":0,", 1);
        assert!(load_test_json(&changed).is_err());
    }

    #[test]
    fn test_loader_unknown_nested_rejects() {
        let changed = manifest_json().replacen(
            "\"terminal_collision_checks\":{",
            "\"terminal_collision_checks\":{\"unknown_nested\":0,",
            1,
        );
        assert!(load_test_json(&changed).is_err());
    }

    #[test]
    fn test_loader_duplicate_rejects() {
        let root_duplicate =
            manifest_json().replacen("\"version\":2", "\"version\":2,\"version\":2", 1);
        assert!(load_test_json(&root_duplicate).is_err());

        let nested_duplicate = manifest_json().replacen(
            "\"duplicate_right_terminals\":0",
            "\"duplicate_right_terminals\":0,\"duplicate_right_terminals\":0",
            1,
        );
        assert!(load_test_json(&nested_duplicate).is_err());
    }

    #[test]
    fn test_loader_trailing_rejects() {
        let changed = format!("{}{{}}", manifest_json());
        assert!(load_test_json(&changed).is_err());
    }

    #[test]
    fn test_loader_size_rejects() {
        let oversized = " ".repeat(4 * 1024 + 1);
        assert!(load_test_json(&oversized).is_err());
    }

    #[test]
    fn test_manifest_preflight_mutations() {
        let context = load_test_context();
        let valid = preflight_manifest(&context);
        validate_manifest(&context, &valid).expect("valid preflight manifest");

        let mut version = valid.clone();
        version.version += 1;
        assert!(matches!(
            validate_manifest(&context, &version),
            Err(GenesisChainIdentityError::ManifestVersionMismatch { .. })
        ));

        let mut network = valid.clone();
        network.network = "testnet".to_string();
        assert!(matches!(
            validate_manifest(&context, &network),
            Err(GenesisChainIdentityError::ManifestNetworkMismatch { .. })
        ));

        let mut count = valid.clone();
        count.leaf_count += 1;
        assert!(matches!(
            validate_manifest(&context, &count),
            Err(GenesisChainIdentityError::ManifestCountMismatch { .. })
        ));

        let mut collision = valid.clone();
        collision.terminal_collision_checks.right_voucher_collisions = 1;
        assert_eq!(
            validate_manifest(&context, &collision).map(|_| ()),
            Err(GenesisChainIdentityError::ManifestTerminalCollision)
        );

        let mut artifact = valid.clone();
        artifact.rights_artifact = "other.json".to_string();
        assert!(matches!(
            validate_manifest(&context, &artifact),
            Err(GenesisChainIdentityError::ManifestArtifactMismatch { .. })
        ));

        let mut digest = valid.clone();
        digest.state_hash = "AA".repeat(32);
        assert_eq!(
            validate_manifest(&context, &digest).map(|_| ()),
            Err(GenesisChainIdentityError::ManifestDigestEncoding {
                field: "state_hash"
            })
        );

        let mut legacy = valid.clone();
        legacy.manifest_hash = "00".repeat(32);
        assert!(matches!(
            validate_manifest(&context, &legacy),
            Err(GenesisChainIdentityError::LegacyManifestDigestMismatch { .. })
        ));
    }

    #[test]
    fn test_network_identity_mutations() {
        let expected = derive_network_id(7, "devnet", "z00z-dev", [0x5a, 0x30, 0x30, 0x5a]);
        for mutated in [
            derive_network_id(8, "devnet", "z00z-dev", [0x5a, 0x30, 0x30, 0x5a]),
            derive_network_id(7, "testnet", "z00z-dev", [0x5a, 0x30, 0x30, 0x5a]),
            derive_network_id(7, "devnet", "z00z-test", [0x5a, 0x30, 0x30, 0x5a]),
            derive_network_id(7, "devnet", "z00z-dev", [0x5a, 0x30, 0x30, 0x5b]),
        ] {
            assert_ne!(mutated, expected);
        }

        let chain_id = 7_u32.to_le_bytes();
        let magic = [0x5a, 0x30, 0x30, 0x5a];
        assert_ne!(
            sha256_256(
                GENESIS_ID_SHA256_DOMAIN_V2,
                NETWORK_ID_SHA256_LABEL_V2,
                &[b"devnet", &chain_id, b"z00z-dev", &magic],
            ),
            expected
        );
        assert_ne!(
            sha256_256(
                "z00z.core.genesis.chain_identity.v3",
                NETWORK_ID_SHA256_LABEL_V2,
                &[&chain_id, b"devnet", b"z00z-dev", &magic],
            ),
            expected
        );
        assert_ne!(
            sha256_256(
                GENESIS_ID_SHA256_DOMAIN_V2,
                "network_id_v3",
                &[&chain_id, b"devnet", b"z00z-dev", &magic],
            ),
            expected
        );
        let wide_chain_id = 7_u64.to_le_bytes();
        assert_ne!(
            sha256_256(
                GENESIS_ID_SHA256_DOMAIN_V2,
                NETWORK_ID_SHA256_LABEL_V2,
                &[&wide_chain_id, b"devnet", b"z00z-dev", &magic],
            ),
            expected
        );
        let mut concatenated = Vec::new();
        concatenated.extend_from_slice(&chain_id);
        concatenated.extend_from_slice(b"devnet");
        concatenated.extend_from_slice(b"z00z-dev");
        concatenated.extend_from_slice(&magic);
        assert_ne!(
            sha256_256(
                GENESIS_ID_SHA256_DOMAIN_V2,
                NETWORK_ID_SHA256_LABEL_V2,
                &[&concatenated],
            ),
            expected
        );
    }

    #[test]
    fn test_genesis_identity_golden() {
        let fixture = manifest();
        let parts = identity_parts(&fixture);
        let independent = independent_hash(
            GENESIS_ID_SHA256_DOMAIN_V2,
            GENESIS_IDENTITY_SHA256_LABEL_V2,
            &borrow_parts(&parts),
        );
        assert_eq!(
            compute_genesis_identity_digest_v2(&fixture).expect("valid fixture"),
            [
                0x9a, 0x3a, 0x47, 0x11, 0xfc, 0x5d, 0x30, 0xa2, 0xb0, 0xbe, 0x78, 0x04, 0x9c, 0x95,
                0x2b, 0xf4, 0xb1, 0x7d, 0x70, 0xa4, 0x32, 0x84, 0x45, 0x43, 0x29, 0x5b, 0xa8, 0x6b,
                0x5d, 0xfe, 0x95, 0x6d,
            ]
        );
        assert_eq!(
            compute_genesis_identity_digest_v2(&fixture).expect("valid fixture"),
            independent
        );
    }

    #[test]
    fn test_genesis_framing_mutations() {
        let fixture = manifest();
        let mut parts = identity_parts(&fixture);
        let expected = compute_genesis_identity_digest_v2(&fixture).expect("valid fixture");

        assert_ne!(
            independent_hash(
                "z00z.core.genesis.chain_identity.v3",
                GENESIS_IDENTITY_SHA256_LABEL_V2,
                &borrow_parts(&parts),
            ),
            expected
        );
        assert_ne!(
            independent_hash(
                GENESIS_ID_SHA256_DOMAIN_V2,
                "genesis_identity_v3",
                &borrow_parts(&parts),
            ),
            expected
        );

        parts.swap(0, 1);
        assert_ne!(
            independent_hash(
                GENESIS_ID_SHA256_DOMAIN_V2,
                GENESIS_IDENTITY_SHA256_LABEL_V2,
                &borrow_parts(&parts),
            ),
            expected
        );
        parts.swap(0, 1);
        parts[2] = (fixture.asset_definition_count as u32)
            .to_le_bytes()
            .to_vec();
        assert_ne!(
            independent_hash(
                GENESIS_ID_SHA256_DOMAIN_V2,
                GENESIS_IDENTITY_SHA256_LABEL_V2,
                &borrow_parts(&parts),
            ),
            expected
        );

        let concatenated: Vec<u8> = identity_parts(&fixture).concat();
        assert_ne!(
            sha256_256(
                GENESIS_ID_SHA256_DOMAIN_V2,
                GENESIS_IDENTITY_SHA256_LABEL_V2,
                &[&concatenated],
            ),
            expected
        );
    }

    #[test]
    fn test_genesis_identity_mutations() {
        let original = manifest();
        let expected = compute_genesis_identity_digest_v2(&original).expect("valid fixture");
        let mut mutations = Vec::new();
        macro_rules! mutate {
            ($field:ident, $value:expr) => {{
                let mut changed = original.clone();
                changed.$field = $value;
                mutations.push(changed);
            }};
        }

        mutate!(version, 3);
        mutate!(network, "testnet".to_string());
        mutate!(asset_definition_count, 2);
        mutate!(asset_count, 3);
        mutate!(policy_count, 4);
        mutate!(right_template_count, 5);
        mutate!(right_count, 6);
        mutate!(voucher_count, 7);
        mutate!(leaf_count, 14);
        mutate!(root_generation, 2);
        mutate!(generation_seed_hash, "11".repeat(32));
        mutate!(corpus_digest, "12".repeat(32));
        mutate!(state_hash, "13".repeat(32));
        mutate!(policies_replay_digest, "14".repeat(32));
        mutate!(policies_output_roundtrip_digest, "15".repeat(32));
        mutate!(deterministic_replay_digest, "16".repeat(32));
        mutate!(rights_output_roundtrip_digest, "17".repeat(32));
        mutate!(vouchers_replay_digest, "18".repeat(32));
        mutate!(vouchers_output_roundtrip_digest, "19".repeat(32));
        mutate!(policies_artifact, "other-policies.json".to_string());
        mutate!(rights_artifact, "other-rights.json".to_string());
        mutate!(vouchers_artifact, "other-vouchers.json".to_string());

        for field in 0..5 {
            let mut changed = original.clone();
            match field {
                0 => changed.terminal_collision_checks.duplicate_right_terminals = 1,
                1 => {
                    changed
                        .terminal_collision_checks
                        .duplicate_voucher_terminals = 1
                }
                2 => changed.terminal_collision_checks.asset_right_collisions = 1,
                3 => changed.terminal_collision_checks.asset_voucher_collisions = 1,
                4 => changed.terminal_collision_checks.right_voucher_collisions = 1,
                _ => unreachable!(),
            }
            mutations.push(changed);
        }

        for changed in mutations {
            assert_ne!(
                compute_genesis_identity_digest_v2(&changed).expect("valid mutation encoding"),
                expected
            );
        }

        let mut legacy_self_hash_only = original.clone();
        legacy_self_hash_only.manifest_hash = "ff".repeat(32);
        assert_eq!(
            compute_genesis_identity_digest_v2(&legacy_self_hash_only)
                .expect("legacy field is compatibility-only"),
            expected
        );
        assert_ne!(
            legacy_self_hash_only.manifest_hash,
            hex::encode(compute_genesis_manifest_hash(&legacy_self_hash_only))
        );
    }

    #[test]
    fn test_digest_encoding_rejects() {
        let mut uppercase = manifest();
        uppercase.state_hash = "AA".repeat(32);
        assert_eq!(
            compute_genesis_identity_digest_v2(&uppercase),
            Err(GenesisChainIdentityError::ManifestDigestEncoding {
                field: "state_hash"
            })
        );

        let mut short = manifest();
        short.corpus_digest = "01".repeat(31);
        assert_eq!(
            compute_genesis_identity_digest_v2(&short),
            Err(GenesisChainIdentityError::ManifestDigestEncoding {
                field: "corpus_digest"
            })
        );
    }

    #[test]
    fn test_legacy_hash_self_check() {
        let mut fixture = manifest();
        fixture.manifest_hash = hex::encode(compute_genesis_manifest_hash(&fixture));
        validate_legacy_hash(&fixture).expect("legacy self-hash matches");

        fixture.manifest_hash = "g".repeat(64);
        assert!(matches!(
            validate_legacy_hash(&fixture),
            Err(GenesisChainIdentityError::LegacyManifestDigestMismatch { .. })
        ));
    }

    #[test]
    fn test_process_pin_idempotence() {
        let slot = OnceLock::new();
        let expected = identity(7);

        install_into(&slot, expected).expect("first install succeeds");
        install_into(&slot, expected).expect("same identity is idempotent");

        assert_eq!(slot.get(), Some(&expected));
    }

    #[test]
    fn test_process_pin_rotation_rejects() {
        let slot = OnceLock::new();
        let installed = identity(7);
        let proposed = identity(8);
        install_into(&slot, installed).expect("first install succeeds");

        assert_eq!(
            install_into(&slot, proposed),
            Err(GenesisChainIdentityError::ProcessIdentityMismatch {
                installed,
                proposed,
            })
        );
    }
}
