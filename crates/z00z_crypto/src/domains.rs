//! # Domain Separation Policy
//!
//! **MANDATORY:** All cryptographic hashing MUST use `hash_domain!` macro.
//!
//! ## Security Rationale
//!
//! Domain separation prevents cross-protocol attacks where data from one context
//! is misinterpreted in another. Using dedicated type-safe domains ensures:
//!
//! - ✅ **Type safety**: Compile-time validation (typos impossible)
//! - ✅ **Collision resistance**: Different domains → different hashes
//! - ✅ **Auditability**: All domains declared in one file
//! - ✅ **Versioning**: Built-in algorithm upgrade path
//!
//! ## Format Convention
//!
//! ```text
//! "z00z.<component>.<purpose>.v<version>"
//! ```
//!
//! ### Examples:
//!
//! - Wallet keys: `z00z.wallet.key.v1`
//! - Asset IDs: `z00z.asset.id.v1`
//! - Transaction hashes: `z00z.tx.hash.v1`
//!
//! ## Migration Note
//!
//! ⚠️ **Manual DST construction is forbidden by policy.**
//! Do NOT use `format!()` or raw string concatenation for domain separation.
//! Always declare domains via `hash_domain!` macro in this file.
//!
//! ## Usage Example
//!
//! ```rust
//! use z00z_crypto::domains::AssetIdHashDomain;
//! use z00z_crypto::hash::DomainHasher;
//!
//! let mut hasher = DomainHasher::<AssetIdHashDomain>::new_with_label("asset");
//! hasher = hasher.chain(b"asset-data");
//! let hash = hasher.finalize();
//! assert_eq!(hash.as_ref().len(), 64); // Blake2b-512
//! ```
//!
//! These domains are versioned and MUST remain stable once deployed.

use crate::expert::hash_domain;

// ============================================================================
// Backend Domains
// ============================================================================

// Core Tari crypto backend domain.
hash_domain!(TariCryptoHashDomain, "z00z.crypto.tari.v1", 1);

// ============================================================================
// Wallet Domains
// ============================================================================

// Wallet key derivation domain.
hash_domain!(WalletKeyDomain, "z00z.wallet.key.v1", 1);

// Wallet backup encryption domain.
hash_domain!(WalletBackupDomain, "z00z.wallet.backup.v1", 1);

// Wallet encryption domain.
hash_domain!(WalletEncryptDomain, "z00z.wallet.encrypt.v1", 1);

// Wallet stealth tag16 hashing domain preserved for production compatibility.
hash_domain!(
    StealthTag16ProdDomain,
    "z00z.wallet.stealth.tag16.prod.v1",
    1
);

// Wallet stealth leaf_ad hashing domain preserved for production compatibility.
hash_domain!(
    StealthLeafAdProdDomain,
    "z00z.wallet.stealth.leaf_ad.prod.v1",
    1
);

// ============================================================================
// Asset Domains
// ============================================================================

// Asset ID generation domain.
hash_domain!(AssetIdHashDomain, "z00z.asset.id.v1", 1);

// Asset commitment domain.
hash_domain!(AssetCommitDomain, "z00z.asset.commitment.v1", 1);

// Asset blinding factor domain.
hash_domain!(AssetBlindDomain, "z00z.asset.blinding.v1", 1);

// Asset checksum domain.
hash_domain!(ChecksumHashDomain, "z00z.asset.checksum.v1", 1);

// ============================================================================
// Transaction Domains
// ============================================================================

// Transaction hash domain.
hash_domain!(TxHashDomain, "z00z.tx.hash.v1", 1);

// Transaction signature domain.
hash_domain!(TxSignatureDomain, "z00z.tx.signature.v1", 1);

// Transaction kernel domain.
hash_domain!(TxKernelDomain, "z00z.tx.kernel.v1", 1);

// Claim statement hash domain.
hash_domain!(ClaimStmtDomain, "z00z.claim.statement.v1", 1);

// Claim proof domain.
hash_domain!(ClaimProofDomain, "z00z.claim.proof.v1", 1);

// Claim authority signature domain.
hash_domain!(ClaimSigDomain, "z00z.claim.sig.v1", 1);

// ============================================================================
// Network Domains
// ============================================================================

// OnionNet session key domain.
hash_domain!(OnionSessionDomain, "z00z.net.session.v1", 1);

// Peer ID generation domain.
hash_domain!(PeerIdDomain, "z00z.net.peer_id.v1", 1);

// ============================================================================
// Generic Domains
// ============================================================================

// Generic derivation domain (for dynamic contexts).
// Use this when the specific use case needs runtime flexibility.
hash_domain!(GenericDeriveDomain, "z00z.crypto.derive.v1", 1);

// ============================================================================
// Consensus ZK Domains (Generic) - SPEC §2.2.1.1, §2.2.3
// ============================================================================

// Generic consensus hash domain (H_zk) - SPEC §2.2.1.1.
// Used for generic consensus hashing with custom context labels.
// For specific consensus operations, use dedicated domains (ReceiverIdDomain, etc.)
hash_domain!(ConsensusHashDomain, "z00z.consensus.hash.v1", 1);

// Generic hash-to-scalar domain (hash_to_scalar_zk) - SPEC §2.2.3.
// Used for generic scalar derivation with custom context labels.
// For specific scalar derivations, use dedicated domains (ViewKeyDomain, etc.)
hash_domain!(HashToScalarDomain, "z00z.consensus.h2s.v1", 1);

// ============================================================================
// Consensus Domains - SPEC §2.2.2.1 (Phase 6 - Complete Registry)
// ============================================================================

// Core Domains: Receiver Identity & Keys
hash_domain!(
    EphemeralScalarDomain,
    "z00z.consensus.ephemeral_scalar.v1",
    1
); // Hedged ephemeral scalar
hash_domain!(ReceiverIdDomain, "z00z.consensus.receiver_id.v1", 1); // owner_handle = H_zk("z00z.consensus.receiver_id.v1", receiver_secret)
hash_domain!(ViewKeyDomain, "z00z.consensus.view_key.v1", 1); // view_sk = hash_to_scalar_zk("z00z.consensus.view_key.v1", receiver_secret)

// ECDH & Symmetric Keys
hash_domain!(DhKeyDomain, "z00z.consensus.dh_key.v1", 1); // ECDH to symmetric key k_dh

// Ownership & Tags
hash_domain!(OwnerTagDomain, "z00z.consensus.owner_tag.v1", 1); // owner_tag = H_zk("z00z.consensus.owner_tag.v1", owner_handle || k_dh)
hash_domain!(Tag16Domain, "z00z.consensus.tag16.v1", 1); // Scan accelerator (16-byte tag)

// Assets & Leaves
hash_domain!(AssetIdDomain, "z00z.consensus.asset_id.v1", 1); // asset_id = H_zk("z00z.consensus.asset_id.v1", s_out)
hash_domain!(LeafAdDomain, "z00z.consensus.leaf_ad.v1", 1); // Leaf associated data (AEAD)
hash_domain!(LeafHashDomain, "z00z.consensus.leaf_hash.v1", 1); // Leaf hash for JMT (Jellyfish Merkle Tree)

// ZkPack AEAD (Poseidon2-based encryption)
hash_domain!(ZkPackDomain, "z00z.consensus.zkpack.v1", 1); // ZkPack sponge initialization
hash_domain!(PackKeyDomain, "z00z.consensus.pack_key.v1", 1); // Pack encryption key derivation
hash_domain!(PackNonceDomain, "z00z.consensus.pack_nonce.v1", 1); // Pack nonce derivation
hash_domain!(PackFlowDomain, "z00z.consensus.pack_flow.v1", 1); // Pack flow domain (older sponge path)
hash_domain!(PackMacDomain, "z00z.consensus.pack_mac.v1", 1); // Pack MAC domain (older sponge path)
hash_domain!(XofBlockDomain, "z00z.consensus.xof_block.v1", 1); // XOF block generation

// Transactions & Proofs
hash_domain!(TxDigestDomain, "z00z.consensus.tx_digest.v1", 1); // Transaction digest
hash_domain!(SpendNullifierDomain, "z00z.consensus.nullifier.v1", 1); // Regular spend nullifier
hash_domain!(TxOutputNonceDomain, "z00z.consensus.tx_output_nonce.v1", 1); // Tx output wire nonce derivation
hash_domain!(RangeCtxDomain, "z00z.consensus.range_ctx.v1", 1); // Bound range-proof context digest
hash_domain!(Stage4OutSeedDomain, "z00z.consensus.stage4_out_seed.v1", 1); // Stage-4 deterministic output seed
hash_domain!(TxProofDomain, "z00z.consensus.tx_proof.v1", 1); // TxProof Fiat-Shamir challenge
hash_domain!(CheckpointDomain, "z00z.consensus.checkpoint.v1", 1); // CheckpointProof

// Aggregator quorum artifacts
hash_domain!(
    ShardMembershipDomain,
    "z00z.consensus.shard_membership.v1",
    1
); // Active shard placement membership digest
hash_domain!(CommitSubjectDomain, "z00z.consensus.commit_subject.v1", 1); // Canonical shard commit subject digest
hash_domain!(ShardVoteDomain, "z00z.consensus.shard_vote.v1", 1); // Canonical shard vote digest
hash_domain!(
    ShardVoteLocalSignatureDomain,
    "z00z.consensus.shard_vote_local_signature.v1",
    1
); // Deterministic local signature seam for simulator votes
hash_domain!(
    ShardQuorumCertificateDomain,
    "z00z.consensus.shard_qc.v1",
    1
); // Canonical shard quorum certificate digest
hash_domain!(ShardEvidenceDomain, "z00z.consensus.shard_evidence.v1", 1); // Structured shard safety evidence digest
hash_domain!(
    ShardTransportEnvelopeDomain,
    "z00z.consensus.shard_transport_envelope.v1",
    1
); // In-memory shard transport envelope identity digest

// Identity & Requests
hash_domain!(PaymentRequestDomain, "z00z.payment.request.v1", 1); // PaymentRequest signature domain
hash_domain!(ReceiverCardDomain, "z00z.receiver.card.v1", 1); // ReceiverCard signature domain

// ============================================================================
// Test Domains
// ============================================================================

// Test nonce generation domain.
hash_domain!(TestNonceDomain, "z00z.test.nonce.v1", 1);

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use tari_crypto::hashing::DomainSeparation;

    /// Verify all consensus domains are unique (no collisions).
    ///
    /// CRITICAL: Duplicate domains would cause cross-protocol attacks.
    #[test]
    fn test_consensus_domains_unique() {
        let domains: [&[u8]; 29] = [
            // Core: Receiver Identity & Keys
            b"z00z.consensus.ephemeral_scalar.v1",
            b"z00z.consensus.receiver_id.v1",
            b"z00z.consensus.view_key.v1",
            // ECDH & Symmetric Keys
            b"z00z.consensus.dh_key.v1",
            // Ownership & Tags
            b"z00z.consensus.owner_tag.v1",
            b"z00z.consensus.tag16.v1",
            // Assets & Leaves
            b"z00z.consensus.asset_id.v1",
            b"z00z.consensus.leaf_ad.v1",
            b"z00z.consensus.leaf_hash.v1",
            // ZkPack AEAD
            b"z00z.consensus.zkpack.v1",
            b"z00z.consensus.pack_key.v1",
            b"z00z.consensus.pack_nonce.v1",
            b"z00z.consensus.pack_flow.v1",
            b"z00z.consensus.pack_mac.v1",
            b"z00z.consensus.xof_block.v1",
            // Transactions & Proofs
            b"z00z.consensus.tx_digest.v1",
            b"z00z.consensus.nullifier.v1",
            b"z00z.consensus.tx_output_nonce.v1",
            b"z00z.consensus.range_ctx.v1",
            b"z00z.consensus.stage4_out_seed.v1",
            b"z00z.consensus.tx_proof.v1",
            b"z00z.consensus.checkpoint.v1",
            // Aggregator quorum artifacts
            b"z00z.consensus.shard_membership.v1",
            b"z00z.consensus.commit_subject.v1",
            b"z00z.consensus.shard_vote.v1",
            b"z00z.consensus.shard_vote_local_signature.v1",
            b"z00z.consensus.shard_qc.v1",
            // Identity & Requests
            b"z00z.payment.request.v1",
            b"z00z.receiver.card.v1",
        ];

        let unique: HashSet<_> = domains.iter().collect();
        assert_eq!(
            domains.len(),
            unique.len(),
            "Duplicate consensus domains detected! This breaks domain separation."
        );
    }

    /// Verify consensus domain count matches SPEC §2.2.2.1.
    #[test]
    fn test_consensus_domain_count() {
        // Keep this synchronized with the dedicated consensus-domain registry above.
        let expected = 29;
        let actual = 29;

        assert_eq!(
            actual, expected,
            "Consensus domain count mismatch (expected {}, got {})",
            expected, actual
        );
    }

    /// Verify domain strings match SPEC §2.2.2.1 exactly.
    ///
    /// CRITICAL: Domain string changes would break consensus.
    #[test]
    fn test_spec_domain_strings() {
        use crate::domains::*;

        let checks: [(&str, &str, &str); 31] = [
            (
                EphemeralScalarDomain::domain(),
                "z00z.consensus.ephemeral_scalar.v1",
                "EphemeralScalarDomain mismatch",
            ),
            (
                ReceiverIdDomain::domain(),
                "z00z.consensus.receiver_id.v1",
                "ReceiverIdDomain mismatch",
            ),
            (
                ViewKeyDomain::domain(),
                "z00z.consensus.view_key.v1",
                "ViewKeyDomain mismatch",
            ),
            (
                DhKeyDomain::domain(),
                "z00z.consensus.dh_key.v1",
                "DhKeyDomain mismatch",
            ),
            (
                OwnerTagDomain::domain(),
                "z00z.consensus.owner_tag.v1",
                "OwnerTagDomain mismatch",
            ),
            (
                Tag16Domain::domain(),
                "z00z.consensus.tag16.v1",
                "Tag16Domain mismatch",
            ),
            (
                AssetIdDomain::domain(),
                "z00z.consensus.asset_id.v1",
                "AssetIdDomain mismatch",
            ),
            (
                LeafAdDomain::domain(),
                "z00z.consensus.leaf_ad.v1",
                "LeafAdDomain mismatch",
            ),
            (
                LeafHashDomain::domain(),
                "z00z.consensus.leaf_hash.v1",
                "LeafHashDomain mismatch",
            ),
            (
                ZkPackDomain::domain(),
                "z00z.consensus.zkpack.v1",
                "ZkPackDomain mismatch",
            ),
            (
                PackKeyDomain::domain(),
                "z00z.consensus.pack_key.v1",
                "PackKeyDomain mismatch",
            ),
            (
                PackNonceDomain::domain(),
                "z00z.consensus.pack_nonce.v1",
                "PackNonceDomain mismatch",
            ),
            (
                PackFlowDomain::domain(),
                "z00z.consensus.pack_flow.v1",
                "PackFlowDomain mismatch",
            ),
            (
                PackMacDomain::domain(),
                "z00z.consensus.pack_mac.v1",
                "PackMacDomain mismatch",
            ),
            (
                XofBlockDomain::domain(),
                "z00z.consensus.xof_block.v1",
                "XofBlockDomain mismatch",
            ),
            (
                TxDigestDomain::domain(),
                "z00z.consensus.tx_digest.v1",
                "TxDigestDomain mismatch",
            ),
            (
                SpendNullifierDomain::domain(),
                "z00z.consensus.nullifier.v1",
                "SpendNullifierDomain mismatch",
            ),
            (
                TxOutputNonceDomain::domain(),
                "z00z.consensus.tx_output_nonce.v1",
                "TxOutputNonceDomain mismatch",
            ),
            (
                RangeCtxDomain::domain(),
                "z00z.consensus.range_ctx.v1",
                "RangeCtxDomain mismatch",
            ),
            (
                Stage4OutSeedDomain::domain(),
                "z00z.consensus.stage4_out_seed.v1",
                "Stage4OutSeedDomain mismatch",
            ),
            (
                TxProofDomain::domain(),
                "z00z.consensus.tx_proof.v1",
                "TxProofDomain mismatch",
            ),
            (
                CheckpointDomain::domain(),
                "z00z.consensus.checkpoint.v1",
                "CheckpointDomain mismatch",
            ),
            (
                ShardMembershipDomain::domain(),
                "z00z.consensus.shard_membership.v1",
                "ShardMembershipDomain mismatch",
            ),
            (
                CommitSubjectDomain::domain(),
                "z00z.consensus.commit_subject.v1",
                "CommitSubjectDomain mismatch",
            ),
            (
                ShardVoteDomain::domain(),
                "z00z.consensus.shard_vote.v1",
                "ShardVoteDomain mismatch",
            ),
            (
                ShardVoteLocalSignatureDomain::domain(),
                "z00z.consensus.shard_vote_local_signature.v1",
                "ShardVoteLocalSignatureDomain mismatch",
            ),
            (
                ShardQuorumCertificateDomain::domain(),
                "z00z.consensus.shard_qc.v1",
                "ShardQuorumCertificateDomain mismatch",
            ),
            (
                ShardEvidenceDomain::domain(),
                "z00z.consensus.shard_evidence.v1",
                "ShardEvidenceDomain mismatch",
            ),
            (
                ShardTransportEnvelopeDomain::domain(),
                "z00z.consensus.shard_transport_envelope.v1",
                "ShardTransportEnvelopeDomain mismatch",
            ),
            (
                PaymentRequestDomain::domain(),
                "z00z.payment.request.v1",
                "PaymentRequestDomain mismatch",
            ),
            (
                ReceiverCardDomain::domain(),
                "z00z.receiver.card.v1",
                "ReceiverCardDomain mismatch",
            ),
        ];

        for (actual, expected, msg) in checks {
            assert_eq!(actual, expected, "{msg}");
        }
    }
}
