#![forbid(unsafe_code)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use z00z_crypto::domains::ShardTransportEnvelopeDomain;

use crate::{
    commit_subject::{digest_bytes, push_bytes32, push_len_prefixed, push_u64, push_u8},
    placement::AggregatorId,
    shard_vote::ShardVoteKind,
    CommitSubject,
};

const SHARD_TRANSPORT_TAG: &[u8] = b"z00z.shard_transport";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportPayloadStatus {
    Available,
    Missing { detail: String },
}

impl TransportPayloadStatus {
    fn code(&self) -> u8 {
        match self {
            Self::Available => 1,
            Self::Missing { .. } => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteTransportEnvelope {
    pub message_id: [u8; 32],
    pub from_id: AggregatorId,
    pub to_id: AggregatorId,
    pub subject: CommitSubject,
    pub vote_kind: ShardVoteKind,
    pub payload_status: TransportPayloadStatus,
}

impl VoteTransportEnvelope {
    #[must_use]
    pub fn available(
        from_id: AggregatorId,
        to_id: AggregatorId,
        subject: CommitSubject,
        vote_kind: ShardVoteKind,
    ) -> Self {
        Self::new(
            from_id,
            to_id,
            subject,
            vote_kind,
            TransportPayloadStatus::Available,
        )
    }

    #[must_use]
    pub fn missing_payload(
        from_id: AggregatorId,
        to_id: AggregatorId,
        subject: CommitSubject,
        vote_kind: ShardVoteKind,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            from_id,
            to_id,
            subject,
            vote_kind,
            TransportPayloadStatus::Missing {
                detail: detail.into(),
            },
        )
    }

    #[must_use]
    pub fn new(
        from_id: AggregatorId,
        to_id: AggregatorId,
        subject: CommitSubject,
        vote_kind: ShardVoteKind,
        payload_status: TransportPayloadStatus,
    ) -> Self {
        let message_id = message_id_for(from_id, to_id, &subject, vote_kind, &payload_status);
        Self {
            message_id,
            from_id,
            to_id,
            subject,
            vote_kind,
            payload_status,
        }
    }
}

pub trait VoteTransport {
    fn enqueue(&mut self, envelope: VoteTransportEnvelope);

    fn enqueue_front(&mut self, envelope: VoteTransportEnvelope);

    fn enqueue_delayed(&mut self, envelope: VoteTransportEnvelope, delay: u64);

    fn requeue(&mut self, envelope: VoteTransportEnvelope, delay: u64);

    fn step(&mut self) -> Vec<VoteTransportEnvelope>;

    fn drop_next(&mut self) -> Option<VoteTransportEnvelope>;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InMemoryVoteTransport {
    queue: VecDeque<QueuedEnvelope>,
    tick: u64,
}

impl InMemoryVoteTransport {
    #[must_use]
    pub const fn tick(&self) -> u64 {
        self.tick
    }
}

impl VoteTransport for InMemoryVoteTransport {
    fn enqueue(&mut self, envelope: VoteTransportEnvelope) {
        self.queue
            .push_back(QueuedEnvelope::delayed(envelope, self.tick));
    }

    fn enqueue_front(&mut self, envelope: VoteTransportEnvelope) {
        self.queue
            .push_front(QueuedEnvelope::delayed(envelope, self.tick));
    }

    fn enqueue_delayed(&mut self, envelope: VoteTransportEnvelope, delay: u64) {
        self.queue.push_back(QueuedEnvelope::delayed(
            envelope,
            self.tick.saturating_add(delay),
        ));
    }

    fn requeue(&mut self, envelope: VoteTransportEnvelope, delay: u64) {
        self.enqueue_delayed(envelope, delay);
    }

    fn step(&mut self) -> Vec<VoteTransportEnvelope> {
        self.tick = self.tick.saturating_add(1);
        let mut ready = Vec::new();
        for _ in 0..self.queue.len() {
            let queued = self.queue.pop_front().expect("queued transport envelope");
            if queued.deliver_at <= self.tick {
                ready.push(queued.envelope);
            } else {
                self.queue.push_back(queued);
            }
        }
        ready
    }

    fn drop_next(&mut self) -> Option<VoteTransportEnvelope> {
        self.queue.pop_front().map(|queued| queued.envelope)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedEnvelope {
    envelope: VoteTransportEnvelope,
    deliver_at: u64,
}

impl QueuedEnvelope {
    fn delayed(envelope: VoteTransportEnvelope, deliver_at: u64) -> Self {
        Self {
            envelope,
            deliver_at,
        }
    }
}

fn message_id_for(
    from_id: AggregatorId,
    to_id: AggregatorId,
    subject: &CommitSubject,
    vote_kind: ShardVoteKind,
    payload_status: &TransportPayloadStatus,
) -> [u8; 32] {
    let mut out = Vec::with_capacity(192);
    out.extend_from_slice(SHARD_TRANSPORT_TAG);
    push_u64(&mut out, u64::from(from_id.as_u16()));
    push_u64(&mut out, u64::from(to_id.as_u16()));
    push_bytes32(&mut out, subject.digest());
    push_u8(
        &mut out,
        match vote_kind {
            ShardVoteKind::Prepare => 1,
            ShardVoteKind::Commit => 2,
            ShardVoteKind::LocalCommit => 3,
        },
    );
    push_u8(&mut out, payload_status.code());
    match payload_status {
        TransportPayloadStatus::Available => {}
        TransportPayloadStatus::Missing { detail } => {
            push_bytes32(&mut out, subject.payload_digest);
            push_len_prefixed(&mut out, detail.as_bytes());
        }
    }
    digest_bytes::<ShardTransportEnvelopeDomain>("message_id", &out)
}
