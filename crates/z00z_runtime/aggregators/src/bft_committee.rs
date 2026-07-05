#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::{
    placement::{AggregatorId, ShardPlacement},
    shard_quorum_certificate::membership_digest_for_voters,
    types::{BatchRoute, RejectClass, RejectRecord},
};

/// Deterministic BFT thresholds for one committee size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BftThresholds {
    /// Total number of committee members.
    pub member_count: usize,
    /// Maximum Byzantine members tolerated by the committee.
    pub max_faulty: usize,
    /// Minimum vote count required for one BFT quorum certificate.
    pub quorum_threshold: usize,
}

impl BftThresholds {
    /// Build thresholds for one exact `3f+1` committee.
    pub fn new(member_count: usize) -> Result<Self, RejectRecord> {
        if member_count < 4 {
            return Err(reject(
                RejectClass::PolicyReject,
                "invalid BFT committee: membership must satisfy 3f+1 with at least four members",
            ));
        }
        if (member_count - 1) % 3 != 0 {
            return Err(reject(
                RejectClass::PolicyReject,
                "invalid BFT committee: membership must satisfy 3f+1 exactly",
            ));
        }

        let max_faulty = (member_count - 1) / 3;
        Ok(Self {
            member_count,
            max_faulty,
            quorum_threshold: (2 * max_faulty) + 1,
        })
    }
}

/// Local committee inventory for the BFT-valid simulated backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BftCommittee {
    primary_id: AggregatorId,
    ready_secondary_ids: Vec<AggregatorId>,
    thresholds: BftThresholds,
}

impl BftCommittee {
    /// Build one BFT-valid committee from the primary and ready secondaries.
    pub fn new(
        primary_id: AggregatorId,
        ready_secondary_ids: impl IntoIterator<Item = AggregatorId>,
    ) -> Result<Self, RejectRecord> {
        let ready_secondary_ids = ready_secondary_ids.into_iter().collect::<BTreeSet<_>>();
        if ready_secondary_ids.contains(&primary_id) {
            return Err(reject(
                RejectClass::PolicyReject,
                "invalid BFT committee: primary cannot be listed as a secondary member",
            ));
        }

        let thresholds = BftThresholds::new(ready_secondary_ids.len() + 1)?;
        Ok(Self {
            primary_id,
            ready_secondary_ids: ready_secondary_ids.into_iter().collect(),
            thresholds,
        })
    }

    /// Build a BFT committee from one live shard placement.
    pub fn from_placement(placement: &ShardPlacement) -> Result<Self, RejectRecord> {
        Self::new(
            placement.primary_id,
            placement
                .secondaries
                .iter()
                .filter(|secondary| secondary.is_ready)
                .map(|secondary| secondary.aggregator_id),
        )
    }

    /// Return the primary member id.
    #[must_use]
    pub const fn primary_id(&self) -> AggregatorId {
        self.primary_id
    }

    /// Return the ready secondary member ids in canonical sorted order.
    #[must_use]
    pub fn ready_secondary_ids(&self) -> &[AggregatorId] {
        &self.ready_secondary_ids
    }

    /// Return the full member count.
    #[must_use]
    pub const fn member_count(&self) -> usize {
        self.thresholds.member_count
    }

    /// Return the maximum faulty-member count.
    #[must_use]
    pub const fn max_faulty(&self) -> usize {
        self.thresholds.max_faulty
    }

    /// Return the minimum `2f+1` BFT quorum size.
    #[must_use]
    pub const fn quorum_threshold(&self) -> usize {
        self.thresholds.quorum_threshold
    }

    /// Return the canonical membership digest for one route.
    #[must_use]
    pub fn membership_digest(&self, route: BatchRoute) -> [u8; 32] {
        membership_digest_for_voters(
            route,
            self.primary_id,
            self.ready_secondary_ids.iter().copied(),
        )
    }
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
