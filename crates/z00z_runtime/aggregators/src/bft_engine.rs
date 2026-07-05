#![forbid(unsafe_code)]

use z00z_storage::settlement::SettlementStateRoot;

use crate::{
    bft_committee::BftCommittee,
    commit_subject::CommitSubject,
    placement::ShardPlacement,
    shard_quorum_certificate::ShardQuorumCertificate,
    shard_vote::ShardVote,
    types::{BatchId, BatchRoute, RejectClass, RejectRecord},
};

/// One local BFT commit produced behind the commit-subject seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BftCommit {
    pub term: u64,
    pub batch_id: BatchId,
    pub route: BatchRoute,
    pub state_root: SettlementStateRoot,
    pub journal_lineage: [u8; 32],
    pub subject: CommitSubject,
    pub certificate: ShardQuorumCertificate,
}

/// Minimal local BFT backend for simulated `3f+1` and `2f+1` proofs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BftEngine {
    route: BatchRoute,
    term: u64,
    committee: BftCommittee,
}

impl BftEngine {
    /// Build one BFT engine from the live shard placement.
    pub fn from_placement(placement: &ShardPlacement) -> Result<Self, RejectRecord> {
        Ok(Self {
            route: placement.route,
            term: 0,
            committee: BftCommittee::from_placement(placement)?,
        })
    }

    /// Return the active committee.
    #[must_use]
    pub const fn committee(&self) -> &BftCommittee {
        &self.committee
    }

    /// Return the canonical membership digest for the active route.
    #[must_use]
    pub fn membership_digest(&self) -> [u8; 32] {
        self.committee.membership_digest(self.route)
    }

    /// Build one BFT-bound commit certificate for the current subject.
    pub fn commit(
        &mut self,
        subject: &CommitSubject,
        votes: &[ShardVote],
    ) -> Result<BftCommit, RejectRecord> {
        if subject.route() != self.route {
            return Err(reject(
                RejectClass::PolicyReject,
                "wrong generation: BFT subject route drifted from the active route",
            ));
        }
        if subject.term < self.term {
            return Err(reject(
                RejectClass::PolicyReject,
                "stale term: BFT term regressed",
            ));
        }
        if subject.membership_digest != self.membership_digest() {
            return Err(reject(
                RejectClass::PolicyReject,
                "membership drift: BFT subject membership digest drifted from the active committee",
            ));
        }

        let certificate = ShardQuorumCertificate::new_bft(
            subject,
            self.committee.primary_id(),
            self.committee.ready_secondary_ids().iter().copied(),
            votes,
        )?;
        self.term = subject.term;

        Ok(BftCommit {
            term: subject.term,
            batch_id: subject.batch_id,
            route: subject.route(),
            state_root: subject.new_state_root,
            journal_lineage: subject.journal_lineage,
            subject: subject.clone(),
            certificate,
        })
    }
}

fn reject(class: RejectClass, detail: &str) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class,
        detail: detail.to_string(),
    }
}
