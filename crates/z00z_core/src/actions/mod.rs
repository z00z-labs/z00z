//! Canonical action vocabulary for Phase 059 object semantics.

mod action_descriptor;
mod action_id;
mod action_pool;

pub use action_descriptor::{
    ActionDescriptorV1, LifecycleEffectV1, RequiredSignatureV1, WitnessRequirementV1,
};
pub use action_id::{ActionId, ActionPoolId};
pub use action_pool::{
    fixed_cash_action_pool_descriptor, reject_custom_native_cash_pool, ActionPoolDescriptorV1,
};

#[cfg(test)]
mod test_action_descriptor;
