//! RNG mode selection for simulator stages.
//!
//! When `mock_rng_seed` is `Some(seed)`, uses deterministic
//! [`z00z_utils::rng::MockRngProvider`]
//! for simulator reproducibility.
//! When `None`, uses cryptographically secure
//! [`z00z_utils::rng::SystemRngProvider`] for the
//! non-mock path.
//! This helper does not claim one unified randomness selector for every
//! simulator stage.
//! It remains a consolidation pass over live abstractions, not a brand-new design.

use rand::RngCore;
use z00z_utils::rng::{MockRngProvider, SystemRngProvider};

/// Selects the local RNG strategy for a caller-owned stage run.
///
/// # Variants
///
/// - `Mock(seed)` — deterministic, reproducible; bounded to CI and simulator
///   fixture flows.
/// - `System` — OS-backed secure randomness; maximum unpredictability for
///   production-like runs.
#[derive(Debug, Clone)]
pub enum RngMode {
    /// Deterministic mock RNG seeded from a fixed `u64`.
    Mock(u64),
    /// OS-backed cryptographically secure RNG.
    System,
}

impl RngMode {
    /// Derive mode from an optional seed at the local call site.
    ///
    /// - `Some(s)` → [`RngMode::Mock`]
    /// - `None`    → [`RngMode::System`]
    pub fn from_seed(seed: Option<u64>) -> Self {
        match seed {
            Some(s) => Self::Mock(s),
            None => Self::System,
        }
    }

    /// Human-readable kind string for logging and event emission.
    ///
    /// Returns `"mock:<seed>"` or `"system"`.
    pub fn kind_str(&self) -> String {
        match self {
            Self::Mock(s) => format!("mock:{s}"),
            Self::System => "system".to_string(),
        }
    }

    /// Create a heap-allocated [`RngCore`] from this mode.
    ///
    /// For `Mock` this returns a freshly seeded [`StdRng`](rand::rngs::StdRng).
    /// For `System` this returns an [`OsRng`](rand::rngs::OsRng).
    pub fn make_rng(&self) -> Box<dyn RngCore> {
        match self {
            Self::Mock(s) => Box::new(MockRngProvider::with_u64_seed(*s).rng()),
            Self::System => Box::new(SystemRngProvider.rng()),
        }
    }
}
