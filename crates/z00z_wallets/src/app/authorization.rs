//! Closed application operation ownership and current-authority checks.

use z00z_app_api::{AppOperation, GrantRequirement, APP_OPERATION_COUNT};

/// Final accountable owner behind a public application operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalOwner {
    /// Local backend composition health.
    Runtime,
    /// Source and build provenance.
    Provenance,
    /// Current capability aggregation.
    Capability,
    /// Wallet identity and history state.
    Wallet,
    /// Intent proposal planner.
    Planner,
    /// Policy, fee, publication, validator, and watcher review.
    Policy,
    /// Durable idempotency and operation state.
    Journal,
    /// Installed Extension state.
    ExtensionRegistry,
    /// Bounded Extension upload staging.
    ExtensionImport,
    /// Package trust, signature, digest, and revocation validation.
    ExtensionValidator,
    /// Use-time Extension artifact preparation.
    ExtensionHost,
}

/// One closed public-to-private operation route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnerRoute {
    /// Public closed operation discriminant.
    pub operation: AppOperation,
    /// Canonical facade method name.
    pub facade_method: &'static str,
    /// Private owner mapping.
    pub private_mapping: &'static str,
    /// Final accountable state owner.
    pub owner: FinalOwner,
    /// Required server-owned grant.
    pub grant: GrantRequirement,
}

macro_rules! route {
    ($operation:ident, $mapping:literal, $owner:ident) => {
        OwnerRoute {
            operation: AppOperation::$operation,
            facade_method: AppOperation::$operation.api_method(),
            private_mapping: $mapping,
            owner: FinalOwner::$owner,
            grant: AppOperation::$operation.required_grant(),
        }
    };
}

/// Exhaustive closed operation registry. Array length is tied to the API owner.
pub const OWNER_ROUTES: [OwnerRoute; APP_OPERATION_COUNT] = [
    route!(GetRuntimeStatus, "local.runtime.status", Runtime),
    route!(GetBuildMetadata, "local.provenance.build", Provenance),
    route!(
        GetRuntimeCapabilities,
        "local.capability.aggregate",
        Capability
    ),
    route!(ListWallets, "z00z_walletd.wallet.list", Wallet),
    route!(GetWallet, "z00z_walletd.wallet.get", Wallet),
    route!(CreateWalletIntentProposal, "local.planner.propose", Planner),
    route!(ReviewWalletIntentProposal, "local.policy.review", Policy),
    route!(SubmitWalletIntent, "z00z_walletd.intent.submit", Journal),
    route!(GetOperation, "local.journal.get", Journal),
    route!(CancelOperation, "local.journal.cancel", Journal),
    route!(SubscribeOperationEvents, "local.journal.subscribe", Journal),
    route!(ListExtensions, "local.extension.list", ExtensionRegistry),
    route!(GetExtension, "local.extension.inspect", ExtensionRegistry),
    route!(
        BeginExtensionImport,
        "local.extension.import.begin",
        ExtensionImport
    ),
    route!(
        AppendExtensionImportChunk,
        "local.extension.import.append",
        ExtensionImport
    ),
    route!(
        FinishExtensionImport,
        "local.extension.import.commit",
        ExtensionValidator
    ),
    route!(
        GetExtensionOperation,
        "local.extension.operation.get",
        Journal
    ),
    route!(
        CancelExtensionOperation,
        "local.extension.import.abort",
        ExtensionImport
    ),
    route!(
        UpdateExtension,
        "local.extension.update",
        ExtensionValidator
    ),
    route!(
        SetExtensionEnabled,
        "local.extension.enable",
        ExtensionRegistry
    ),
    route!(
        SetExtensionLocalBlock,
        "local.extension.local_block",
        ExtensionRegistry
    ),
    route!(RemoveExtension, "local.extension.remove", ExtensionRegistry),
    route!(
        PrepareExtensionArtifact,
        "local.extension.artifact.prepare",
        ExtensionHost
    ),
];

/// Resolve exactly one route for an API operation.
#[must_use]
pub fn owner_route(operation: AppOperation) -> Option<&'static OwnerRoute> {
    OWNER_ROUTES
        .iter()
        .find(|route| route.operation == operation)
}
