//! End-to-end hybrid Nova and Plonky3 checkpoint load scenario.
//!
//! The scenario keeps the production ownership chain intact: wallet packages
//! enter the aggregator, become one storage handoff, advance HJMT, fold through
//! the sole public recursive-checkpoint V2 ingress, and close a full Plonky3
//! epoch every 2,000 blocks.

mod checkpoint;
mod config;
mod da;
mod plonky3;
mod profile;
mod runner;
mod tx_batch;
mod types;
mod wallets;

pub use config::{Scenario2Cfg, DEFAULT_CONFIG_PATH};
pub use runner::{run, run_with_path, Scenario2Err};
pub use types::Scenario2Summary;
