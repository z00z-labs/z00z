pub const CONS_DOMAINS: &[&[u8]] = &[
    b"z00z.consensus.receiver_id.v1",
    b"z00z.consensus.view_key.v1",
    b"Z00Z/IDENTITY",
    b"z00z.consensus.dh_key.v1",
    b"z00z.consensus.owner_tag.v1",
    b"z00z.consensus.tag16.v1",
    b"z00z.consensus.asset_id.v1",
    b"z00z.consensus.leaf_ad.v1",
    b"z00z.consensus.leaf_hash.v1",
    b"z00z.consensus.pack_key.v1",
    b"z00z.consensus.pack_nonce.v1",
    b"z00z.consensus.pack_flow.v1",
    b"z00z.consensus.pack_mac.v1",
    b"z00z.consensus.xof_block.v1",
    b"z00z.consensus.tx_digest.v1",
    b"z00z.consensus.range_ctx.v1",
    b"z00z.consensus.ephemeral_scalar.v1",
];

pub const WALLET_DOMAINS: &[&[u8]] = &[b"WALLET/SEED", b"WALLET/DB_ID", b"WALLET/CACHE"];

/// Typed V2 checkpoint SHA-256 commitment roles.
///
/// Each role owns one frozen domain/label pair.  Checkpoint code must use this
/// registry instead of spelling SHA-256 domains locally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointShaRole {
    SettlementRoot,
    TransactionRow,
    TransactionRoot,
    WitnessRoot,
    Journal,
    Statement,
    Link,
    Trace,
    Content,
    UniquenessCounts,
    UniquenessContext,
    SpentOriginalIds,
    SpentSortedIds,
    OutputOriginalIds,
    OutputSortedIds,
    IdPrecommit,
    IdChallenge,
}

/// Complete frozen checkpoint SHA-role registry.
pub const ALL_CHECKPOINT_SHA_ROLES_V2: &[CheckpointShaRole] = &[
    CheckpointShaRole::SettlementRoot,
    CheckpointShaRole::TransactionRow,
    CheckpointShaRole::TransactionRoot,
    CheckpointShaRole::WitnessRoot,
    CheckpointShaRole::Journal,
    CheckpointShaRole::Statement,
    CheckpointShaRole::Link,
    CheckpointShaRole::Trace,
    CheckpointShaRole::Content,
    CheckpointShaRole::UniquenessCounts,
    CheckpointShaRole::UniquenessContext,
    CheckpointShaRole::SpentOriginalIds,
    CheckpointShaRole::SpentSortedIds,
    CheckpointShaRole::OutputOriginalIds,
    CheckpointShaRole::OutputSortedIds,
    CheckpointShaRole::IdPrecommit,
    CheckpointShaRole::IdChallenge,
];

impl CheckpointShaRole {
    /// Returns the frozen domain for this commitment role.
    #[must_use]
    pub const fn domain(self) -> &'static str {
        match self {
            Self::SettlementRoot => "z00z.storage.settlement.root",
            Self::TransactionRow | Self::TransactionRoot => "z00z.storage.checkpoint.exec.v2",
            Self::WitnessRoot | Self::Trace => "z00z.storage.checkpoint.witness.v2",
            Self::Journal => "z00z.storage.checkpoint.journal.v2",
            Self::Statement => "z00z.storage.checkpoint.statement.v2",
            Self::Link => "z00z.storage.checkpoint.link.v2",
            Self::Content => "z00z.storage.checkpoint.content.v2",
            Self::UniquenessCounts
            | Self::UniquenessContext
            | Self::SpentOriginalIds
            | Self::SpentSortedIds
            | Self::OutputOriginalIds
            | Self::OutputSortedIds
            | Self::IdPrecommit
            | Self::IdChallenge => "z00z.storage.checkpoint.uniqueness.v2",
        }
    }

    /// Returns the frozen label for this commitment role.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SettlementRoot => "settlement_hjmt_root_v2",
            Self::TransactionRow => "tx_row_v2",
            Self::TransactionRoot => "tx_root_v2",
            Self::WitnessRoot => "witness_root_v2",
            Self::Journal => "journal_v2",
            Self::Statement => "statement_v2",
            Self::Link => "checkpoint_link_v2",
            Self::Trace => "trace_v2",
            Self::Content => "content_v2",
            Self::UniquenessCounts => "pre_uniqueness_counts_v2",
            Self::UniquenessContext => "pre_uniqueness_context_v2",
            Self::SpentOriginalIds => "spent_original_ids_v2",
            Self::SpentSortedIds => "spent_sorted_ids_v2",
            Self::OutputOriginalIds => "output_original_ids_v2",
            Self::OutputSortedIds => "output_sorted_ids_v2",
            Self::IdPrecommit => "id_lists_precommit_v2",
            Self::IdChallenge => "id_permutation_challenge_v2",
        }
    }
}

pub const ALL_DOMAINS: &[&[u8]] = &[
    b"z00z.consensus.receiver_id.v1",
    b"z00z.consensus.view_key.v1",
    b"Z00Z/IDENTITY",
    b"z00z.consensus.dh_key.v1",
    b"z00z.consensus.owner_tag.v1",
    b"z00z.consensus.tag16.v1",
    b"z00z.consensus.asset_id.v1",
    b"z00z.consensus.leaf_ad.v1",
    b"z00z.consensus.leaf_hash.v1",
    b"z00z.consensus.pack_key.v1",
    b"z00z.consensus.pack_nonce.v1",
    b"z00z.consensus.pack_flow.v1",
    b"z00z.consensus.pack_mac.v1",
    b"z00z.consensus.xof_block.v1",
    b"z00z.consensus.tx_digest.v1",
    b"z00z.consensus.range_ctx.v1",
    b"z00z.consensus.ephemeral_scalar.v1",
    b"WALLET/SEED",
    b"WALLET/DB_ID",
    b"WALLET/CACHE",
];
