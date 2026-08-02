//! Shared simulator infrastructure reused by all scenarios.

#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(all(
    not(test),
    not(debug_assertions),
    feature = "test-params-fast",
    not(feature = "wallet_debug_tools")
))]
compile_error!(
    "`test-params-fast` MUST NOT be compiled into release-capable z00z_simulator builds"
);

#[cfg(all(
    not(test),
    not(debug_assertions),
    feature = "wallet_debug_tools",
    not(feature = "test-params-fast")
))]
compile_error!(
    "`wallet_debug_tools` MUST NOT be compiled into release-capable z00z_simulator builds"
);

/// Actor model helpers for simulator scenarios.
pub mod actors;
/// Scenario configuration models.
pub mod config;
/// Scenario execution context.
pub mod context;
/// Scenario design metadata.
pub mod design;
/// Scenario events and summaries.
pub mod event;
/// Scenario result types.
pub mod result;
/// Deterministic RNG mode selection.
pub mod rng_mode;
/// Scenario 1 workflow implementation.
pub mod scenario_1;
/// Scenario 11 shard-local quorum harness.
pub mod scenario_11;
/// Recursive checkpoint load and capacity scenario.
pub mod scenario_2;

/// Simulator actor type.
pub use actors::SimActor;
/// Scenario configuration facade.
pub use config::{
    AssetCfg, HjmtCfgRef, OutputCfg, ScenarioCfg, ScenarioCfgErr, ScenarioMeta, SimCfg,
    Stage2ActorCfg, Stage2WalletCfg,
};
/// Simulator context facade.
pub use context::SimContext;
/// Scenario design documents and steps.
pub use design::{DesignDoc, DesignErr, DesignStage, DesignStep};
/// Scenario claim summaries.
pub use event::{ActorClaimSummary, ClaimGenesisEvent};
/// Scenario result status types.
pub use result::{ScenarioResult, StageResult, StageState};
/// RNG mode facade.
pub use rng_mode::RngMode;
