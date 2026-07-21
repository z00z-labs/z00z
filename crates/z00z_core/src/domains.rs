//! Domain separation tags for Z00Z Core.
//!
//! These are versioned and MUST remain stable once deployed.

use z00z_crypto::expert::hash_domain;

/// Frozen portable SHA-256 domain for the core-owned genesis chain identity.
pub(crate) const GENESIS_ID_SHA256_DOMAIN_V2: &str = "z00z.core.genesis.chain_identity.v2";

// Assets
hash_domain!(MetadataHashDomain, "z00z.core.assets.metadata", 1);
hash_domain!(OwnerSignatureDomain, "z00z.core.assets.owner_signature", 1);
hash_domain!(RegistryHashDomain, "z00z.core.assets.registry.version", 1);
hash_domain!(
    ActionDescriptorHashDomain,
    "z00z.core.actions.descriptor",
    1
);
hash_domain!(ActionPoolDescriptorHashDomain, "z00z.core.actions.pool", 1);
hash_domain!(
    PolicyDescriptorHashDomain,
    "z00z.core.policies.descriptor",
    1
);
#[rustfmt::skip]
hash_domain!(NativeCoinDomainDevnet, "z00z.core.assets.native_coin.devnet", 1);
#[rustfmt::skip]
hash_domain!(NativeCoinDomainTestnet, "z00z.core.assets.native_coin.testnet", 1);
#[rustfmt::skip]
hash_domain!(NativeCoinDomainMainnet, "z00z.core.assets.native_coin.mainnet", 1);
hash_domain!(
    NonceDerivationDomain,
    "z00z.core.assets.nonce_derivation",
    1
);
hash_domain!(GenesisNonceDomain, "z00z.core.assets.nonce.genesis", 1);

// Test-only
hash_domain!(TestAssetIdDomain, "z00z.test.asset_id", 1);

// Genesis
hash_domain!(GenesisStateHashDomain, "z00z.core.genesis.state_hash", 1);
#[rustfmt::skip]
hash_domain!(GenesisBlindingDomainDevnet, "z00z.core.genesis.blinding.devnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisBlindingDomainTestnet, "z00z.core.genesis.blinding.testnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisBlindingDomainMainnet, "z00z.core.genesis.blinding.mainnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisRngSeedDomainDevnet, "z00z.core.genesis.rng_seed.devnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisRngSeedDomainTestnet, "z00z.core.genesis.rng_seed.testnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisRngSeedDomainMainnet, "z00z.core.genesis.rng_seed.mainnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisAssetIdDomainDevnet, "z00z.core.genesis.asset_id.devnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisAssetIdDomainTestnet, "z00z.core.genesis.asset_id.testnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisAssetIdDomainMainnet, "z00z.core.genesis.asset_id.mainnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisRightDerivationDomainDevnet, "z00z.core.genesis.right.devnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisRightDerivationDomainTestnet, "z00z.core.genesis.right.testnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisRightDerivationDomainMainnet, "z00z.core.genesis.right.mainnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisVoucherDerivationDomainDevnet, "z00z.core.genesis.voucher.devnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisVoucherDerivationDomainTestnet, "z00z.core.genesis.voucher.testnet", 1);
#[rustfmt::skip]
hash_domain!(GenesisVoucherDerivationDomainMainnet, "z00z.core.genesis.voucher.mainnet", 1);
hash_domain!(GenesisManifestHashDomain, "z00z.core.genesis.manifest", 1);
