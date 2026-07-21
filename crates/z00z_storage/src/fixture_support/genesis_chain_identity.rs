//! Deterministic validated genesis identity bootstrap for tests and benches.

use std::sync::Arc;

use anyhow::{Context, Result};
use z00z_core::{
    config_paths::devnet_genesis_path,
    genesis::{
        export_genesis_settlement_artifacts, generate_genesis_lanes, load_genesis_context,
        load_validate_install_chain_identity, require_process_chain_identity,
        validator::compute_genesis_state_hash, GenesisChainIdentityV2, GenesisGenerationPlan,
        GENESIS_ROOT_GENERATION,
    },
};
use z00z_utils::prelude::{NoopLogger, NoopMetrics};

/// Install the canonical devnet genesis identity through the sole validated path.
///
/// The helper generates a complete deterministic bootstrap manifest in an
/// ephemeral directory. It never accepts caller-selected identity digests and
/// never bypasses the core manifest reproduction checks.
///
/// # Errors
///
/// Returns an error when canonical config loading, full bootstrap generation,
/// artifact export, or validated process installation fails.
pub fn ensure_test_process_chain_identity() -> Result<&'static GenesisChainIdentityV2> {
    if let Ok(identity) = require_process_chain_identity() {
        return Ok(identity);
    }

    let config_path = devnet_genesis_path();
    let config_path = config_path
        .to_str()
        .context("canonical devnet genesis path is not UTF-8")?;
    let plan = GenesisGenerationPlan::full_bootstrap();
    let context = load_genesis_context(config_path, &plan)
        .context("load canonical devnet genesis context")?;
    let outputs =
        generate_genesis_lanes(&context, &plan, Arc::new(NoopLogger), Arc::new(NoopMetrics))
            .context("generate canonical devnet genesis lanes")?;
    let definitions = outputs
        .asset_definitions
        .as_deref()
        .context("full bootstrap omitted asset definitions")?;
    let policies = outputs
        .policies
        .as_deref()
        .context("full bootstrap omitted policies")?;
    let corpus = outputs.combined_corpus();
    let state_hash = compute_genesis_state_hash(&corpus);
    let temp = tempfile::tempdir().context("create genesis fixture directory")?;
    let (_, manifest_path) = export_genesis_settlement_artifacts(
        temp.path(),
        definitions,
        policies,
        &corpus,
        context.network_type,
        GENESIS_ROOT_GENERATION,
        &state_hash,
        context.seed.as_bytes(),
    )
    .context("export canonical devnet genesis artifacts")?;
    let manifest_path = manifest_path
        .to_str()
        .context("generated genesis manifest path is not UTF-8")?;

    load_validate_install_chain_identity(config_path, manifest_path)
        .context("validate and install canonical devnet genesis identity")
}
