//! # Gas Calculation and Fee Validation
//!
//! Protocol-wide gas metering for transaction fees, using a schedule-based approach
//! that meters inputs, outputs, and range proof bit lengths.
//!
//! ## Architecture
//!
//! - **GasSchedule**: Per-component costs (base_tx, per_input, per_output, per_proof_bit)
//! - **GasUsage**: Transaction counters (inputs, outputs, range_proof_bits)
//! - **GasAsset**: Native coin wrapper enforcing v1 rule (only coins pay fees)
//!
//! ## Security: Overflow Protection (HIGH-F1)
//!
//! All gas calculations include overflow protection to prevent DoS attacks:
//!
//! ### Protocol Limits
//!
//! ```text
//! MAX_INPUTS       = 10,000 inputs per transaction
//! MAX_OUTPUTS      = 10,000 outputs per transaction
//! MAX_PROOF_BITS   = 640,000 bits (10k outputs × 64 bits)
//! ```
//!
//! ### Validation Strategy
//!
//! 1. **Pre-calculation checks**: Reject txs exceeding limits before arithmetic
//! 2. **Checked arithmetic**: Use `checked_add`/`checked_mul` for all operations
//! 3. **Early returns**: Fail fast with `ArithmeticOverflow` error
//!
//! ### Attack Scenarios Prevented
//!
//! - **Transaction bloat**: Adversary creates tx with MAX_INPUTS + 1 → rejected
//! - **Proof flooding**: Adversary creates 1M proof bits → rejected  
//! - **Integer overflow**: Crafted schedule causes wraparound → detected
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use z00z_core::assets::gas::*;
//!
//! let tx = MyTransaction { inputs: 5, outputs: 10, ... };
//! let schedule = GAS_SCHEDULE_PLACEHOLDER;
//! let price = GasPrice::new(2);
//!
//! // Calculate fee (returns Result for overflow protection)
//! let fee = calculate_fee(&tx, &schedule, &price)?;
//!
//! // Validate fee declaration
//! let gas_asset = GasAsset::new(native_coin_asset)?;
//! validate_fee(&tx, &schedule, &price, &gas_asset)?;
//! ```
//!
//! ## Structured Logging Policy
//!
//! Fee validation uses the injected `z00z_utils::logger::Logger` for audit trails
//! and debugging:
//!
//! ### Log Levels
//!
//! - **debug**: Fee validation inputs (schedule, price, amounts)
//! - **trace**: Detailed comparisons (declared vs expected)
//! - **warn**: Validation failures (mismatches, invalid assets)
//! - **error**: Critical failures (missing fee asset, no covering outputs)
//!
//! ### Structured Fields
//!
//! All log statements use key-value pairs for machine parsing:
//!
//! ```rust,ignore
//! use z00z_utils::logger::{Logger, TracingLogger};
//!
//! let logger = TracingLogger;
//! logger.debug("validating transaction fee");
//! ```
//!
//! ### Field Formatting
//!
//! - `?` (Debug): For enums and byte arrays
//! - `%` (Display): For numbers and formatted types (NOT byte arrays)
//! - No prefix: For primitives (u64, bool)

use std::borrow::Cow;

use crate::assets::{is_native_fee_def, Amount, AssetDefinition, AssetError};

// The canonical native fee definition is owned by assets/mod.rs and reconstructed
// through native_fee_def() so fee validation and tests share one source of truth.

/// Protocol limits to prevent gas calculation overflow and DoS attacks
pub const MAX_INPUTS: usize = 10_000;
pub const MAX_OUTPUTS: usize = 10_000;
pub const MAX_PROOF_BITS: usize = 640_000; // 10,000 outputs * 64 bits

/// Protocol gas units referenced by the fee schedule.
/// The abstract counter allows the core protocol to change pricing without touching the native coin denomination.
pub type GasUnit = u64;

/// Price of a single [`GasUnit`] expressed in native coin units, as described in the validator economics notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GasPrice {
    pub per_unit: Amount,
}

impl GasPrice {
    pub const fn new(per_unit: Amount) -> Self {
        Self { per_unit }
    }
}

/// Protocol-wide schedule that meters each transaction component (base cost, per-input/output, range-proof bits).
/// Mirrors the scheduler outlined in `assets_spec_release.md` §2 when determining the minimum fee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GasSchedule {
    pub base_tx_cost: GasUnit,
    pub per_input_cost: GasUnit,
    pub per_output_cost: GasUnit,
    pub per_range_proof_bit_cost: GasUnit,
}

impl GasSchedule {
    /// Computes gas usage for the provided counters with overflow protection.
    ///
    /// # Security
    ///
    /// - Validates inputs against MAX_INPUTS/MAX_OUTPUTS/MAX_PROOF_BITS before calculation
    /// - Uses checked arithmetic to detect overflow before falling back to saturating ops
    /// - Prevents DoS attacks via adversarial transaction bloat
    ///
    /// # Returns
    ///
    /// - `Ok(gas_units)` if calculation succeeds within protocol limits
    /// - `Err(AssetError::ArithmeticOverflow)` if limits exceeded
    fn calculate_gas_used(&self, usage: GasUsage) -> Result<GasUnit, AssetError> {
        // HIGH-F1: Validate limits before calculation
        if usage.inputs > MAX_INPUTS {
            return Err(AssetError::ArithmeticOverflow(Cow::Owned(format!(
                "inputs ({}) exceeds MAX_INPUTS ({})",
                usage.inputs, MAX_INPUTS
            ))));
        }
        if usage.outputs > MAX_OUTPUTS {
            return Err(AssetError::ArithmeticOverflow(Cow::Owned(format!(
                "outputs ({}) exceeds MAX_OUTPUTS ({})",
                usage.outputs, MAX_OUTPUTS
            ))));
        }
        if usage.range_proof_bits > MAX_PROOF_BITS {
            return Err(AssetError::ArithmeticOverflow(Cow::Owned(format!(
                "range_proof_bits ({}) exceeds MAX_PROOF_BITS ({})",
                usage.range_proof_bits, MAX_PROOF_BITS
            ))));
        }

        // Use checked arithmetic first to detect overflow
        let mut total = self.base_tx_cost;

        total = total
            .checked_add(
                self.per_input_cost
                    .checked_mul(usage.inputs as GasUnit)
                    .ok_or_else(|| {
                        AssetError::ArithmeticOverflow(
                            "per_input_cost multiplication overflow".into(),
                        )
                    })?,
            )
            .ok_or_else(|| AssetError::ArithmeticOverflow("input cost addition overflow".into()))?;

        total = total
            .checked_add(
                self.per_output_cost
                    .checked_mul(usage.outputs as GasUnit)
                    .ok_or_else(|| {
                        AssetError::ArithmeticOverflow(
                            "per_output_cost multiplication overflow".into(),
                        )
                    })?,
            )
            .ok_or_else(|| {
                AssetError::ArithmeticOverflow("output cost addition overflow".into())
            })?;

        total = total
            .checked_add(
                self.per_range_proof_bit_cost
                    .checked_mul(usage.range_proof_bits as GasUnit)
                    .ok_or_else(|| {
                        AssetError::ArithmeticOverflow(
                            "per_range_proof_bit_cost multiplication overflow".into(),
                        )
                    })?,
            )
            .ok_or_else(|| {
                AssetError::ArithmeticOverflow("range proof cost addition overflow".into())
            })?;

        Ok(total)
    }
}

/// Placeholder schedule values from the Phase 6 review (pending economic model updates).
pub const GAS_SCHEDULE_PLACEHOLDER: GasSchedule = GasSchedule {
    base_tx_cost: 100,
    per_input_cost: 50,
    per_output_cost: 50,
    per_range_proof_bit_cost: 1,
};

/// Gas is always paid with the native coin in v1; this wrapper enforces that constraint when wallets declare fees.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GasAsset {
    pub base_asset: AssetDefinition,
}

impl GasAsset {
    /// Instantiates a gas asset, verifying that the definition points to the native coin.
    pub fn new_from_definition(base_asset: AssetDefinition) -> Result<Self, AssetError> {
        Self::check_is_native_coin_asset(&base_asset)?;
        Ok(Self { base_asset })
    }

    fn check_is_native_coin_asset(def: &AssetDefinition) -> Result<(), AssetError> {
        def.validate()?;

        if !is_native_fee_def(def) {
            return Err(AssetError::InvalidFeeAsset(Cow::Borrowed(
                "gas asset must reference the canonical native fee asset",
            )));
        }

        Ok(())
    }
}

/// Minimal counters required to meter a transaction for gas usage (inputs, outputs, and total range-proof bit length).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GasUsage {
    pub inputs: usize,
    pub outputs: usize,
    pub range_proof_bits: usize,
}

/// Trait that exposes the gas-relevant counters for any transaction-like structure.
pub trait GasMetered {
    fn gas_usage(&self) -> GasUsage;
}

/// Trait that extends [`GasMetered`] with fee declaration helpers for validation.
pub trait FeePayingTransaction: GasMetered {
    fn declared_fee(&self) -> Amount;
    fn fee_paying_asset(&self) -> Option<&AssetDefinition>;
}

/// Computes the expected native coin fee by multiplying gas usage with the schedule and price.
/// This function only depends on gas.rs module - no external tx dependencies.
pub fn calculate_fee<T: GasMetered>(
    tx: &T,
    schedule: &GasSchedule,
    price: &GasPrice,
) -> Result<Amount, AssetError> {
    let usage = tx.gas_usage();
    let gas_used = schedule.calculate_gas_used(usage)?;
    gas_used
        .checked_mul(price.per_unit)
        .ok_or_else(|| AssetError::ArithmeticOverflow("fee calculation overflow".into()))
}

#[cfg(test)]
mod tests {
    use super::GasAsset;
    use crate::assets::{
        is_native_fee_def, native_fee_def, AssetClass, AssetDefinition, NATIVE_CASH_POLICY_FLAGS,
    };
    use crate::domains::NativeCoinDomainDevnet;
    use z00z_crypto::expert::traits::DomainSeparation;

    fn mk_native_fee_def() -> AssetDefinition {
        native_fee_def(NativeCoinDomainDevnet::domain()).expect("valid native fee asset")
    }

    fn mk_impostor_fee_def() -> AssetDefinition {
        AssetDefinition::new(
            [0u8; 32],
            AssetClass::Coin,
            "Evil Native Coin".to_string(),
            "Z00Z".to_string(),
            8,
            100,
            20_000,
            NativeCoinDomainDevnet::domain().to_string(),
            1,
            1,
            NATIVE_CASH_POLICY_FLAGS,
            None,
        )
        .expect("valid impostor fee asset")
    }

    fn mk_flag_only_fee_def() -> AssetDefinition {
        AssetDefinition::new(
            [0u8; 32],
            AssetClass::Token,
            "Z00Z Native Coin".to_string(),
            "Z00Z".to_string(),
            8,
            100,
            20_000,
            NativeCoinDomainDevnet::domain().to_string(),
            1,
            1,
            NATIVE_CASH_POLICY_FLAGS,
            None,
        )
        .expect("valid flag-only fee asset")
    }

    fn mk_near_canonical_fee_def() -> AssetDefinition {
        AssetDefinition::new(
            [0u8; 32],
            AssetClass::Coin,
            "Z00Z Native Coin".to_string(),
            "Z00Z".to_string(),
            8,
            101,
            20_000,
            NativeCoinDomainDevnet::domain().to_string(),
            1,
            1,
            NATIVE_CASH_POLICY_FLAGS,
            None,
        )
        .expect("valid near-canonical fee asset")
    }

    #[test]
    fn test_native_fee_canonical_coin() {
        let definition = mk_native_fee_def();

        assert!(is_native_fee_def(&definition));
        assert!(GasAsset::new_from_definition(definition).is_ok());
    }

    #[test]
    fn test_native_fee_only_coin() {
        let definition = mk_impostor_fee_def();

        assert!(!is_native_fee_def(&definition));
        assert!(GasAsset::new_from_definition(definition).is_err());
    }

    #[test]
    fn test_native_fee_only_asset() {
        let definition = mk_flag_only_fee_def();

        assert!(!is_native_fee_def(&definition));
        assert!(GasAsset::new_from_definition(definition).is_err());
    }

    #[test]
    fn test_native_fee_canonical_asset() {
        let definition = mk_near_canonical_fee_def();

        assert!(!is_native_fee_def(&definition));
        assert!(GasAsset::new_from_definition(definition).is_err());
    }
}
