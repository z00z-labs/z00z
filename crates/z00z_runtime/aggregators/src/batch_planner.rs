#![forbid(unsafe_code)]

use std::{collections::BTreeSet, fmt};

use sha2::{Digest, Sha256};

use crate::types::{
    BatchId, BatchPlanned, BatchRoute, OrderedBatch, PlanDigest, PlannerMode, RejectClass,
    RejectRecord, ShardId, WorkItem,
};

const PLAN_DIGEST_LABEL: &[u8] = b"z00z.runtime.batch_planned.v1";
const PLANNER_AUTHORITY_DIGEST_LABEL: &[u8] = b"z00z.runtime.planner_authority.v1";
const ROUTE_TABLE_DIGEST_LABEL: &[u8] = b"z00z.hjmt.route-table.v1";
const HASH_MIN: [u8; 32] = [0u8; 32];
const HASH_MAX: [u8; 32] = [0xffu8; 32];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteErr {
    NoRoute,
    EmptyShardSet,
    UnsortedShardSet,
    DupShardId,
    EmptyRuleSet,
    BackwardRule,
    UnsortedRules,
    Overlap,
    Gap,
    ForeignShard,
    UnusedShard,
    BadPrevGen,
    Truncated,
    TrailingBytes,
}

impl fmt::Display for RouteErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let detail = match self {
            Self::NoRoute => "route table does not own the requested route key",
            Self::EmptyShardSet => "route table shard_set must not be empty",
            Self::UnsortedShardSet => "route table shard_set must stay sorted ascending",
            Self::DupShardId => "route table shard_set must stay unique",
            Self::EmptyRuleSet => "route table range_rules must not be empty",
            Self::BackwardRule => "route table range_rules must keep start_hash <= end_hash",
            Self::UnsortedRules => "route table range_rules must stay sorted by start_hash",
            Self::Overlap => "route table range_rules must not overlap",
            Self::Gap => "route table range_rules must stay gap-free over the full hash domain",
            Self::ForeignShard => "route table range_rules must reference declared shard_set ids",
            Self::UnusedShard => "route table shard_set ids must be referenced by range_rules",
            Self::BadPrevGen => "route table previous_generation_digest linkage is invalid",
            Self::Truncated => "route table canonical bytes are truncated",
            Self::TrailingBytes => "route table canonical bytes contain trailing data",
        };
        f.write_str(detail)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteRangeRule {
    pub start: [u8; 32],
    pub end: [u8; 32],
    pub shard_id: ShardId,
}

impl RouteRangeRule {
    #[must_use]
    pub const fn new(start: [u8; 32], end: [u8; 32], shard_id: ShardId) -> Self {
        Self {
            start,
            end,
            shard_id,
        }
    }

    fn contains(&self, route_key: [u8; 32]) -> bool {
        self.start <= route_key && route_key <= self.end
    }
}

// Runtime owns batch admission and route-table canonicalization. This table
// binds shard-local planning only; it must not become a second public
// settlement authority or a raw-backend surface for wallet-facing code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardRouteTable {
    pub routing_generation: u64,
    pub shard_set: Vec<ShardId>,
    pub rules: Vec<RouteRangeRule>,
    pub previous_generation_digest: Option<PlanDigest>,
    pub activation_checkpoint: u64,
}

impl Default for ShardRouteTable {
    fn default() -> Self {
        Self {
            routing_generation: 0,
            shard_set: vec![ShardId::new(0)],
            rules: vec![RouteRangeRule::new(HASH_MIN, HASH_MAX, ShardId::new(0))],
            previous_generation_digest: None,
            activation_checkpoint: 0,
        }
    }
}

impl ShardRouteTable {
    pub fn validate(&self) -> Result<(), RouteErr> {
        if self.shard_set.is_empty() {
            return Err(RouteErr::EmptyShardSet);
        }
        if self.rules.is_empty() {
            return Err(RouteErr::EmptyRuleSet);
        }

        let mut seen_shards = BTreeSet::new();
        let mut prev_shard = None;
        for shard_id in &self.shard_set {
            if prev_shard.is_some_and(|prev| prev > *shard_id) {
                return Err(RouteErr::UnsortedShardSet);
            }
            if !seen_shards.insert(*shard_id) {
                return Err(RouteErr::DupShardId);
            }
            prev_shard = Some(*shard_id);
        }

        if self.rules[0].start != HASH_MIN {
            return Err(RouteErr::Gap);
        }

        let mut used_shards = BTreeSet::new();
        let mut prev_rule: Option<RouteRangeRule> = None;
        for rule in &self.rules {
            if rule.end < rule.start {
                return Err(RouteErr::BackwardRule);
            }
            if !seen_shards.contains(&rule.shard_id) {
                return Err(RouteErr::ForeignShard);
            }
            used_shards.insert(rule.shard_id);

            if let Some(prev) = prev_rule {
                if rule.start <= prev.start {
                    return Err(RouteErr::UnsortedRules);
                }
                match next_hash(prev.end) {
                    Some(expected) if rule.start == expected => {}
                    Some(_) if rule.start <= prev.end => return Err(RouteErr::Overlap),
                    Some(_) => return Err(RouteErr::Gap),
                    None => return Err(RouteErr::Gap),
                }
            }
            prev_rule = Some(*rule);
        }

        if self.rules.last().expect("route rules").end != HASH_MAX {
            return Err(RouteErr::Gap);
        }
        if used_shards != seen_shards {
            return Err(RouteErr::UnusedShard);
        }
        if self.routing_generation == 0 {
            if self.previous_generation_digest.is_some() {
                return Err(RouteErr::BadPrevGen);
            }
        } else if self.previous_generation_digest.is_none() {
            return Err(RouteErr::BadPrevGen);
        }
        Ok(())
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            8 + 4 + (self.shard_set.len() * 4) + 4 + (self.rules.len() * 68) + 1 + 32 + 8,
        );
        out.extend_from_slice(&self.routing_generation.to_be_bytes());
        out.extend_from_slice(&(self.shard_set.len() as u32).to_be_bytes());
        for shard_id in &self.shard_set {
            out.extend_from_slice(&shard_id.as_u32().to_be_bytes());
        }
        out.extend_from_slice(&(self.rules.len() as u32).to_be_bytes());
        for rule in &self.rules {
            out.extend_from_slice(&rule.start);
            out.extend_from_slice(&rule.end);
            out.extend_from_slice(&rule.shard_id.as_u32().to_be_bytes());
        }
        match self.previous_generation_digest {
            Some(digest) => {
                out.push(1);
                out.extend_from_slice(digest.as_bytes());
            }
            None => out.push(0),
        }
        out.extend_from_slice(&self.activation_checkpoint.to_be_bytes());
        out
    }

    pub fn from_canon(bytes: &[u8]) -> Result<Self, RouteErr> {
        let mut cursor = 0usize;
        let routing_generation = take_u64(bytes, &mut cursor)?;
        let shard_len = take_u32(bytes, &mut cursor)? as usize;
        let mut shard_set = Vec::with_capacity(shard_len);
        for _ in 0..shard_len {
            shard_set.push(take_shard(bytes, &mut cursor)?);
        }

        let rule_len = take_u32(bytes, &mut cursor)? as usize;
        let mut rules = Vec::with_capacity(rule_len);
        for _ in 0..rule_len {
            let start = take_hash(bytes, &mut cursor)?;
            let end = take_hash(bytes, &mut cursor)?;
            let shard_id = take_shard(bytes, &mut cursor)?;
            rules.push(RouteRangeRule::new(start, end, shard_id));
        }

        let prev_tag = take_u8(bytes, &mut cursor)?;
        let previous_generation_digest = match prev_tag {
            0 => None,
            1 => Some(PlanDigest::new(take_hash(bytes, &mut cursor)?)),
            _ => return Err(RouteErr::BadPrevGen),
        };
        let activation_checkpoint = take_u64(bytes, &mut cursor)?;
        if cursor != bytes.len() {
            return Err(RouteErr::TrailingBytes);
        }

        let table = Self {
            routing_generation,
            shard_set,
            rules,
            previous_generation_digest,
            activation_checkpoint,
        };
        table.validate()?;
        Ok(table)
    }

    pub fn validate_migration(&self, prev: &Self) -> Result<(), RouteErr> {
        prev.validate()?;
        self.validate()?;
        if self.routing_generation <= prev.routing_generation {
            return Err(RouteErr::BadPrevGen);
        }
        if self.previous_generation_digest != Some(prev.digest()) {
            return Err(RouteErr::BadPrevGen);
        }
        if self.activation_checkpoint < prev.activation_checkpoint {
            return Err(RouteErr::BadPrevGen);
        }
        Ok(())
    }

    #[must_use]
    pub fn digest(&self) -> PlanDigest {
        let mut hasher = Sha256::new();
        hasher.update(ROUTE_TABLE_DIGEST_LABEL);
        hasher.update(self.canonical_bytes());
        PlanDigest::new(hasher.finalize().into())
    }

    pub fn lookup(&self, route_key: [u8; 32]) -> Result<ShardId, RouteErr> {
        self.validate()?;
        self.lookup_live(route_key)
    }

    fn lookup_live(&self, route_key: [u8; 32]) -> Result<ShardId, RouteErr> {
        self.rules
            .iter()
            .find(|rule| rule.contains(route_key))
            .map(|rule| rule.shard_id)
            .ok_or(RouteErr::NoRoute)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BatchPlanner {
    route_table: ShardRouteTable,
}

impl BatchPlanner {
    #[must_use]
    pub fn new(route_table: ShardRouteTable) -> Self {
        Self { route_table }
    }

    #[must_use]
    pub const fn route_table(&self) -> &ShardRouteTable {
        &self.route_table
    }

    pub fn plan_batch(
        &self,
        batch_id: BatchId,
        items: &[WorkItem],
    ) -> Result<BatchPlanned, RejectRecord> {
        let entries = self.canonical_entries(items)?;
        Ok(self.build_planned(batch_id, &entries))
    }

    pub fn make_batch(
        &self,
        batch_id: BatchId,
        items: &[WorkItem],
    ) -> Result<OrderedBatch, RejectRecord> {
        let entries = self.canonical_entries(items)?;
        let planned = self.build_planned(batch_id, &entries);
        let items = entries.into_iter().map(|entry| entry.item).collect();

        Ok(OrderedBatch {
            batch_id,
            items,
            created_leaves: Vec::new(),
            planned,
        })
    }

    fn canonical_entries(&self, items: &[WorkItem]) -> Result<Vec<PlanEntry>, RejectRecord> {
        if items.is_empty() {
            return Err(RejectRecord {
                intake_id: None,
                class: RejectClass::ShapeInvalid,
                detail: "empty batch is not admissible".to_string(),
            });
        }
        self.route_table.validate().map_err(route_invalid)?;

        let mut entries = Vec::with_capacity(items.len());
        for item in items {
            let route_key = item.route_key();
            let shard_id = self
                .route_table
                .lookup_live(route_key)
                .map_err(|err| route_lookup_reject(item, err))?;
            entries.push(PlanEntry {
                item: item.clone(),
                route_key,
                shard_id,
            });
        }

        entries.sort_by(|left, right| {
            left.route_key
                .cmp(&right.route_key)
                .then(left.item.kind_tag().cmp(&right.item.kind_tag()))
                .then(left.item.digest_hex().cmp(right.item.digest_hex()))
        });

        let route = entries[0].shard_id;
        if entries.iter().any(|entry| entry.shard_id != route) {
            return Err(RejectRecord {
                intake_id: None,
                class: RejectClass::PolicyReject,
                detail: "multi-shard batch admission stays closed in the current runtime wave"
                    .to_string(),
            });
        }

        Ok(entries)
    }

    fn build_planned(&self, batch_id: BatchId, entries: &[PlanEntry]) -> BatchPlanned {
        let route_table_digest = self.route_table.digest();
        let route = BatchRoute {
            shard_id: entries[0].shard_id,
            routing_generation: self.route_table.routing_generation,
        };
        let intake_ids = entries
            .iter()
            .map(|entry| entry.item.intake_id().clone())
            .collect();
        let plan_digest = plan_digest(batch_id, route, route_table_digest, entries);

        BatchPlanned {
            batch_id,
            route,
            route_table_digest,
            intake_ids,
            op_count: entries.len(),
            plan_digest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlannerAuthority {
    pub mode: PlannerMode,
    pub routing_generation: u64,
    pub route_table_digest: PlanDigest,
    pub planner_cfg_digest: PlanDigest,
}

impl PlannerAuthority {
    #[must_use]
    pub fn bind(
        mode: PlannerMode,
        route_table: &ShardRouteTable,
        planner_cfg_digest: PlanDigest,
    ) -> Self {
        Self {
            mode,
            routing_generation: route_table.routing_generation,
            route_table_digest: route_table.digest(),
            planner_cfg_digest,
        }
    }

    #[must_use]
    pub fn digest(&self) -> PlanDigest {
        let mut hasher = Sha256::new();
        hasher.update(PLANNER_AUTHORITY_DIGEST_LABEL);
        hasher.update(self.mode.as_str().as_bytes());
        hasher.update(self.routing_generation.to_be_bytes());
        hasher.update(self.route_table_digest.as_bytes());
        hasher.update(self.planner_cfg_digest.as_bytes());
        PlanDigest::new(hasher.finalize().into())
    }

    pub fn verify_inputs(
        &self,
        mode: PlannerMode,
        route_table: &ShardRouteTable,
        planner_cfg_digest: PlanDigest,
    ) -> Result<(), RejectRecord> {
        route_table.validate().map_err(route_invalid)?;
        if self.mode != mode {
            return Err(planner_reject(
                "planner config drift: planner mode drifted from the canonical authority",
            ));
        }
        if self.planner_cfg_digest != planner_cfg_digest {
            return Err(planner_reject(
                "planner config drift: planner config digest drifted from the canonical authority",
            ));
        }
        if self.routing_generation != route_table.routing_generation {
            return Err(planner_reject(
                "mixed planner generation: live route table generation drifted from the canonical authority",
            ));
        }
        if self.route_table_digest != route_table.digest() {
            return Err(planner_reject(
                "stale route-table digest: live route table digest drifted from the canonical authority",
            ));
        }
        Ok(())
    }

    pub fn verify_batch(
        &self,
        mode: PlannerMode,
        route_table: &ShardRouteTable,
        planner_cfg_digest: PlanDigest,
        batch_id: BatchId,
        items: &[WorkItem],
        claimed: &BatchPlanned,
    ) -> Result<BatchPlanned, RejectRecord> {
        self.verify_inputs(mode, route_table, planner_cfg_digest)?;

        let planned = BatchPlanner::new(route_table.clone()).plan_batch(batch_id, items)?;
        if planned != *claimed {
            return Err(planner_reject(
                "planner digest drift: local recomputation drifted from the claimed planned batch",
            ));
        }
        Ok(planned)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlanEntry {
    item: WorkItem,
    route_key: [u8; 32],
    shard_id: ShardId,
}

fn plan_digest(
    batch_id: BatchId,
    route: BatchRoute,
    route_table_digest: PlanDigest,
    entries: &[PlanEntry],
) -> PlanDigest {
    let mut hasher = Sha256::new();
    hasher.update(PLAN_DIGEST_LABEL);
    hasher.update(batch_id.draft_id.as_bytes());
    hasher.update(route.shard_id.as_u16().to_be_bytes());
    hasher.update(route.routing_generation.to_be_bytes());
    hasher.update(route_table_digest.as_bytes());
    hasher.update((entries.len() as u32).to_be_bytes());
    for entry in entries {
        hasher.update([entry.item.kind_tag()]);
        hasher.update(entry.route_key);
        hasher.update(entry.item.digest_hex().as_bytes());
    }
    PlanDigest::new(hasher.finalize().into())
}

fn route_invalid(err: RouteErr) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class: RejectClass::PolicyReject,
        detail: format!("route table contract violation: {err}"),
    }
}

fn route_lookup_reject(item: &WorkItem, err: RouteErr) -> RejectRecord {
    RejectRecord {
        intake_id: Some(item.intake_id().clone()),
        class: RejectClass::PolicyReject,
        detail: err.to_string(),
    }
}

fn planner_reject(detail: impl Into<String>) -> RejectRecord {
    RejectRecord {
        intake_id: None,
        class: RejectClass::PolicyReject,
        detail: detail.into(),
    }
}

fn next_hash(mut hash: [u8; 32]) -> Option<[u8; 32]> {
    for index in (0..hash.len()).rev() {
        if hash[index] != u8::MAX {
            hash[index] += 1;
            for tail in &mut hash[index + 1..] {
                *tail = 0;
            }
            return Some(hash);
        }
    }
    None
}

fn take_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8, RouteErr> {
    if *cursor >= bytes.len() {
        return Err(RouteErr::Truncated);
    }
    let value = bytes[*cursor];
    *cursor += 1;
    Ok(value)
}

fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, RouteErr> {
    let chunk = take_exact(bytes, cursor, 4)?;
    let mut raw = [0u8; 4];
    raw.copy_from_slice(chunk);
    Ok(u32::from_be_bytes(raw))
}

fn take_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, RouteErr> {
    let chunk = take_exact(bytes, cursor, 8)?;
    let mut raw = [0u8; 8];
    raw.copy_from_slice(chunk);
    Ok(u64::from_be_bytes(raw))
}

fn take_shard(bytes: &[u8], cursor: &mut usize) -> Result<ShardId, RouteErr> {
    let raw = take_u32(bytes, cursor)?;
    if raw > u16::MAX as u32 {
        return Err(RouteErr::ForeignShard);
    }
    Ok(ShardId::new(raw as u16))
}

fn take_hash(bytes: &[u8], cursor: &mut usize) -> Result<[u8; 32], RouteErr> {
    let chunk = take_exact(bytes, cursor, 32)?;
    let mut raw = [0u8; 32];
    raw.copy_from_slice(chunk);
    Ok(raw)
}

fn take_exact<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], RouteErr> {
    if bytes.len().saturating_sub(*cursor) < len {
        return Err(RouteErr::Truncated);
    }
    let end = *cursor + len;
    let chunk = &bytes[*cursor..end];
    *cursor = end;
    Ok(chunk)
}

#[cfg(test)]
mod tests {
    use z00z_storage::checkpoint::CheckpointDraftId;
    use z00z_wallets::tx::{
        build_claim_tx_digest, build_tx_package_digest, ClaimAuthWire, ClaimContextWire,
        ClaimProofWire, ClaimTxPackage, ClaimTxWire, TxAuthWire, TxContextWire, TxPackage,
        TxProofWire, TxWire,
    };

    use super::*;
    use crate::{IngressBoundary, WorkPayload};

    #[test]
    fn test_plan_rejects_empty_input() {
        let planner = BatchPlanner::default();
        let batch_id = BatchId::new(CheckpointDraftId::new([7u8; 32]));
        let err = planner
            .plan_batch(batch_id, &[])
            .expect_err("empty batch must reject");

        assert_eq!(err.class, RejectClass::ShapeInvalid);
        assert!(err.detail.contains("empty batch"));
    }

    #[test]
    fn test_single_shard_inputs_canonically() {
        let planner = BatchPlanner::default();
        let batch_id = BatchId::new(CheckpointDraftId::new([4u8; 32]));
        let late = tx_item("late");
        let early = claim_item("early");

        let batch = planner
            .make_batch(batch_id, &[late.clone(), early.clone()])
            .expect("single shard batch");
        let mut expected = [
            late.digest_hex().to_string(),
            early.digest_hex().to_string(),
        ];
        expected.sort();

        assert_eq!(batch.items[0].digest_hex(), expected[0]);
        assert_eq!(batch.items[1].digest_hex(), expected[1]);
        assert_eq!(batch.planned.op_count, 2);
        assert_eq!(batch.planned.intake_ids[0].digest_hex(), expected[0]);
        assert_eq!(batch.planned.intake_ids[1].digest_hex(), expected[1]);
        assert_eq!(batch.planned.route.shard_id, ShardId::new(0));
        assert_eq!(batch.planned.route.routing_generation, 0);
    }

    #[test]
    fn test_stable_across_input_order() {
        let planner = BatchPlanner::default();
        let batch_id = BatchId::new(CheckpointDraftId::new([6u8; 32]));
        let tx = tx_item("stable-tx");
        let claim = claim_item("stable-claim");

        let first = planner
            .plan_batch(batch_id, &[tx.clone(), claim.clone()])
            .expect("stable plan");
        let second = planner
            .plan_batch(batch_id, &[claim, tx])
            .expect("stable plan");

        assert_eq!(first.intake_ids, second.intake_ids);
        assert_eq!(first.plan_digest, second.plan_digest);
        assert_eq!(first.route, second.route);
    }

    #[test]
    fn test_rejects_multi_shard_inputs() {
        let left = tx_item("left-shard");
        let right = claim_item("right-shard");
        let left_key = left.route_key();
        let right_key = right.route_key();
        assert_ne!(left_key, right_key, "fixture digests must differ");

        let (low_key, _high_key, low_item, high_item) = if left_key < right_key {
            (left_key, right_key, left, right)
        } else {
            (right_key, left_key, right, left)
        };
        let route_table = ShardRouteTable {
            routing_generation: 9,
            shard_set: vec![ShardId::new(1), ShardId::new(2)],
            rules: vec![
                RouteRangeRule::new([0x00; 32], low_key, ShardId::new(1)),
                RouteRangeRule::new(
                    next_hash(low_key).expect("split + 1"),
                    [0xff; 32],
                    ShardId::new(2),
                ),
            ],
            previous_generation_digest: Some(PlanDigest::new([8u8; 32])),
            activation_checkpoint: 9,
        };
        let planner = BatchPlanner::new(route_table);
        let batch_id = BatchId::new(CheckpointDraftId::new([5u8; 32]));

        let err = planner
            .plan_batch(batch_id, &[low_item, high_item])
            .expect_err("multi shard batch must reject");

        assert_eq!(err.class, RejectClass::PolicyReject);
        assert!(err.detail.contains("multi-shard"));
    }

    #[test]
    fn test_route_table_canon_roundtrip() {
        let table = ShardRouteTable {
            routing_generation: 1,
            shard_set: vec![ShardId::new(1), ShardId::new(2)],
            rules: vec![
                RouteRangeRule::new(HASH_MIN, split_hash(), ShardId::new(1)),
                RouteRangeRule::new(
                    next_hash(split_hash()).expect("split + 1"),
                    HASH_MAX,
                    ShardId::new(2),
                ),
            ],
            previous_generation_digest: Some(PlanDigest::new([9u8; 32])),
            activation_checkpoint: 44,
        };

        let bytes = table.canonical_bytes();
        let decoded = ShardRouteTable::from_canon(&bytes).expect("canonical decode");

        assert_eq!(decoded, table);
        assert_eq!(decoded.canonical_bytes(), bytes);
        assert_eq!(decoded.digest(), table.digest());
    }

    #[test]
    fn test_route_table_rejects_gap() {
        let err = ShardRouteTable {
            routing_generation: 0,
            shard_set: vec![ShardId::new(0), ShardId::new(1)],
            rules: vec![
                RouteRangeRule::new(HASH_MIN, split_hash(), ShardId::new(0)),
                RouteRangeRule::new(bump_hash(split_hash(), 2), HASH_MAX, ShardId::new(1)),
            ],
            previous_generation_digest: None,
            activation_checkpoint: 0,
        }
        .validate()
        .expect_err("gap must reject");

        assert_eq!(err, RouteErr::Gap);
    }

    #[test]
    fn route_table_rejects_truncated_bytes() {
        let mut bytes = ShardRouteTable::default().canonical_bytes();
        bytes.pop();

        let err = ShardRouteTable::from_canon(&bytes).expect_err("truncated table must reject");
        assert_eq!(err, RouteErr::Truncated);
    }

    #[test]
    fn route_table_rejects_trailing_bytes() {
        let mut bytes = ShardRouteTable::default().canonical_bytes();
        bytes.push(0);

        let err = ShardRouteTable::from_canon(&bytes).expect_err("trailing bytes must reject");
        assert_eq!(err, RouteErr::TrailingBytes);
    }

    #[test]
    fn test_digest_binds_package() {
        let planner = BatchPlanner::default();
        let batch_id = BatchId::from_bytes([0x61; 32]);
        let left_item = tx_item("object-bound").with_object_package(dummy_object_package(7));
        let right_item = tx_item("object-bound").with_object_package(dummy_object_package(8));

        let left = planner
            .plan_batch(batch_id, &[left_item])
            .expect("left batch plan");
        let right = planner
            .plan_batch(batch_id, &[right_item])
            .expect("right batch plan");

        assert_ne!(left.plan_digest, right.plan_digest);
        assert_eq!(left.route, right.route);
    }

    fn tx_item(seed: &str) -> WorkItem {
        let mut pkg = TxPackage {
            kind: "TxPackage".to_string(),
            package_type: "regular_tx".to_string(),
            version: 1,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: format!("z00z-{seed}"),
            tx: TxWire {
                tx_type: "regular_tx".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                fee: 0,
                nonce: 0,
                context: TxContextWire::default(),
                proof: TxProofWire::default(),
                auth: TxAuthWire::default(),
            },
            tx_digest_hex: String::new(),
            status: "received".to_string(),
        };
        pkg.tx_digest_hex = build_tx_package_digest(
            &pkg.kind,
            &pkg.package_type,
            pkg.version,
            pkg.chain_id,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        )
        .expect("tx digest");
        IngressBoundary
            .normalize(WorkPayload::Tx(Box::new(pkg)))
            .expect("normalized tx")
    }

    fn claim_item(seed: &str) -> WorkItem {
        let mut pkg = ClaimTxPackage {
            kind: "ClaimTxPackage".to_string(),
            package_type: "claim_tx".to_string(),
            version: 1,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: format!("z00z-{seed}"),
            tx: ClaimTxWire {
                tx_type: "claim_tx".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                fee: 0,
                nonce: 0,
                context: ClaimContextWire {
                    recipient_wallet_id: "wallet".to_string(),
                    recipient_owner_hex: "00".repeat(32),
                    claim_scope_hash_hex: "11".repeat(32),
                    recipient_card_hex: None,
                    nullifier_hex: "22".repeat(32),
                },
                proof: ClaimProofWire {
                    proof_type: "genesis_claim".to_string(),
                    proof_hex: "33".repeat(32),
                },
                auth: ClaimAuthWire {
                    claim_authority_sig_hex: "44".repeat(64),
                },
            },
            tx_digest_hex: String::new(),
            status: "received".to_string(),
        };
        pkg.tx_digest_hex = build_claim_tx_digest(
            &pkg.kind,
            &pkg.package_type,
            pkg.version,
            pkg.chain_id,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        )
        .expect("claim digest");
        IngressBoundary
            .normalize(WorkPayload::Claim(Box::new(pkg)))
            .expect("normalized claim")
    }

    fn split_hash() -> [u8; 32] {
        let mut hash = [0x7f; 32];
        hash[31] = 0x7e;
        hash
    }

    fn bump_hash(mut hash: [u8; 32], step: u8) -> [u8; 32] {
        hash[31] = hash[31].saturating_add(step);
        hash
    }

    fn dummy_object_package(seed: u8) -> crate::types::RuntimeObjectPackageV1 {
        use std::collections::BTreeSet;

        use z00z_core::ObjectFamily;
        use z00z_storage::settlement::{
            ObjectDeltaSetV1, SettlementActionV1, SettlementStateRoot, VoucherAction,
        };

        crate::types::RuntimeObjectPackageV1 {
            primary_family: ObjectFamily::Voucher,
            selected_action: SettlementActionV1::Voucher(VoucherAction::RedeemFull),
            selected_action_id: [seed; 32],
            policy_descriptor_hash: [seed.wrapping_add(1); 32],
            action_pool_id: [seed.wrapping_add(2); 32],
            required_rights: vec![crate::types::RightWitnessRefV1 {
                right_policy: format!("kyc_{seed}"),
                witness_state: crate::types::RightWitnessStateV1::Present,
            }],
            object_witnesses: crate::types::ObjectWitnessBundleV1 {
                signatures: BTreeSet::new(),
                attestation_labels: BTreeSet::new(),
                has_acceptance_proof: false,
                has_replay_nonce: true,
                has_prior_root_binding: true,
                has_disclosure_commitment: false,
            },
            delta_set: ObjectDeltaSetV1::new(
                SettlementActionV1::Voucher(VoucherAction::RedeemFull),
                [seed.wrapping_add(1); 32],
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
                SettlementStateRoot::settlement_v1([seed; 32]),
                SettlementStateRoot::settlement_v1([seed.wrapping_add(1); 32]),
            ),
            fee_support_ref: None,
            prior_root: SettlementStateRoot::settlement_v1([seed; 32]),
            expected_new_root: SettlementStateRoot::settlement_v1([seed.wrapping_add(1); 32]),
        }
    }
}
