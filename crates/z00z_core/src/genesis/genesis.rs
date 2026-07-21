//! Genesis Generation Module
//!
//! Core genesis generation logic including asset accumulator and cryptographic derivations.

use crate::actions::ActionPoolId;
#[cfg(feature = "deterministic-rng")]
use crate::assets::nonce::derive_genesis_nonce;
use crate::assets::{Asset, AssetClass, AssetDefinition, ObjectFamily, Z00ZScalar};
use crate::domains::{
    GenesisBlindingDomainDevnet, GenesisBlindingDomainMainnet, GenesisBlindingDomainTestnet,
    GenesisRngSeedDomainDevnet, GenesisRngSeedDomainMainnet, GenesisRngSeedDomainTestnet,
};
use crate::genesis::genesis_config::{load_genesis_config, AssetConfigEntry, GenesisConfig};
use crate::genesis::serde::export_genesis_assets;
use crate::genesis::validator::{
    compute_genesis_state_hash, validate_genesis_seed, verify_genesis_assets,
    verify_genesis_consensus, GenesisError,
};
use crate::policies::PolicyId;
use crate::rights::RightsConfigEntry;
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use z00z_crypto::hash::DomainHasher;
use z00z_utils::prelude::{Logger, MetricsSink, StdoutLogger, SystemTimeProvider, TimeProvider};
#[cfg(feature = "deterministic-rng")]
use z00z_utils::rng::DeterministicRngProvider;
use z00z_utils::time::Instant;

mod chain_type;
mod genesis_accumulator;
mod genesis_derivation;
mod genesis_output;
mod genesis_policies;
mod genesis_rights;
mod genesis_run;
mod genesis_seed;
mod genesis_settlement_manifest;
mod genesis_vouchers;

#[cfg(test)]
use self::genesis_derivation::generate_assets_checked;
use self::genesis_derivation::{
    create_prechecked_asset_definition, generate_genesis_settlement_corpus_prechecked,
};
#[cfg(not(target_arch = "wasm32"))]
use self::genesis_output::create_genesis_snapshot_zip;
use self::genesis_output::{
    create_timestamped_output_dir, generate_timestamp, prepare_genesis_logging_dir,
    prepare_genesis_snapshot_root, write_genesis_report, GenesisReportArgs,
};
use self::genesis_policies::policy_lookup;
#[cfg(test)]
use self::genesis_rights::generate_genesis_rights;
use self::genesis_rights::generate_genesis_rights_with_policies;
#[cfg(test)]
use self::genesis_run::build_genesis_thread_pool;

pub use self::chain_type::ChainType;
pub use self::genesis_accumulator::{GenesisAssetAccumulator, GenesisSettlementCorpus};
pub use self::genesis_derivation::{
    create_asset_definition, derive_deterministic_rng_seed, derive_genesis_blinding,
    generate_all_genesis_assets, generate_genesis_settlement_corpus,
};
pub use self::genesis_policies::{
    generate_genesis_policies, GenesisPolicyRecord, GENESIS_POLICIES_FILE,
};
pub use self::genesis_rights::{GenesisRightLeaf, GenesisRightRecord, GENESIS_ROOT_GENERATION};
pub use self::genesis_run::{
    generate_genesis_lanes, load_genesis_context, resolve_genesis_context, run_genesis,
    run_genesis_with_plan, GenesisExportKind, GenesisGenerationPlan, GenesisGenerationReceipt,
    GenesisLane, GenesisLaneOutputs, GenesisResolvedContext, GenesisSelection,
    GENESIS_GENERATION_RECEIPT_FILE,
};
pub use self::genesis_seed::GenesisSeed;
pub(crate) use self::genesis_settlement_manifest::load_genesis_settlement_manifest;
pub use self::genesis_settlement_manifest::{
    compute_genesis_manifest_hash, compute_genesis_policies_digest, compute_genesis_rights_digest,
    compute_genesis_seed_hash, compute_genesis_vouchers_digest, ensure_terminal_collision_free,
    export_genesis_settlement_artifacts, GenesisSettlementManifest, TerminalCollisionReport,
    GENESIS_POLICIES_REPLAY_DIGEST_LABEL, GENESIS_POLICIES_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_RIGHTS_FILE, GENESIS_RIGHTS_REPLAY_DIGEST_LABEL, GENESIS_RIGHTS_ROUNDTRIP_DIGEST_LABEL,
    GENESIS_SETTLEMENT_MANIFEST_FILE, GENESIS_VOUCHERS_REPLAY_DIGEST_LABEL,
    GENESIS_VOUCHERS_ROUNDTRIP_DIGEST_LABEL,
};
pub use self::genesis_vouchers::{
    generate_genesis_vouchers, GenesisVoucherRecord, GENESIS_VOUCHERS_FILE,
};

#[cfg(test)]
#[path = "test_genesis.rs"]
mod test_genesis;
