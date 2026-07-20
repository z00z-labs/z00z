use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use jmt::RootHash;

use crate::backend::{
    codec::{def_payload, ser_payload},
    memory::KeyValueOp,
    redb::{hjmt::HjmtPersistWork, state::WriteArts},
    roots::{HjmtBucketKey, HjmtRoots, HjmtSerialKey},
    types::terminal_value_hash,
};
use crate::settlement::{
    keys::{definition_key, serial_key},
    tx_plan_types::ObjectDeltaSetV1,
    BucketRootLeaf, ClaimNullTx, FeeActorCtx, FeeEnvelope, SettlementRouteCtx, SettlementScope,
};

use super::{
    hjmt_journal::{
        hjmt_child_digest, hjmt_journal_digest, hjmt_parent_digest, HjmtCommitJournalEntry,
        HjmtCommitStatus,
    },
    hjmt_plan::{bucket_key_for_path, HjmtBatch, HjmtPlan},
    hjmt_store::HjmtTreeSnap,
    model::SettlementModel,
    proof_batch::{
        JmtTraceSegmentContextV2, JmtTreeRoleV2, JmtUpdateTraceV2, SettlementUpdateTraceBuilderV2,
        SettlementUpdateTraceEnvelopeV2,
    },
    store::{next_ver, scope_tx_id},
    timing,
    tree_id::{HjmtTreeId, TreeRootRef},
    CheckRoot, DefinitionId, DefinitionRootLeaf, FeeReplayRec, RootGeneration, ScopeFlow,
    ScopeFlowItem, ScopeLeafKind, ScopeOpKind, ScopeSeen, SerialId, SerialRootLeaf,
    SettlementExecHandoff, SettlementListReq, SettlementLookup, SettlementPage, SettlementPageTok,
    SettlementPath, SettlementStateRoot, SettlementStore, SettlementStoreError, StoreItem, StoreOp,
};

struct TreeJob {
    tree_id: HjmtTreeId,
    ops: Vec<KeyValueOp>,
    snap: HjmtTreeSnap,
    version: u64,
}

struct TreeOut {
    tree_id: HjmtTreeId,
    root: RootHash,
    trace: JmtUpdateTraceV2,
    snap: HjmtTreeSnap,
}

struct EnsureJob {
    tree_id: HjmtTreeId,
    snap: HjmtTreeSnap,
    version: u64,
}

struct EnsureOut {
    tree_id: HjmtTreeId,
    snap: HjmtTreeSnap,
}

fn verify_tree_out_trace(out: &TreeOut) -> Result<(), SettlementStoreError> {
    out.trace
        .verify_native()
        .map_err(|err| SettlementStoreError::Backend(format!("HJMT update trace rejected: {err}")))
}

fn record_streamed_trace(
    sink: Option<&mut SettlementUpdateTraceBuilderV2>,
    trace: JmtUpdateTraceV2,
) -> Result<(), SettlementStoreError> {
    if let Some(sink) = sink {
        sink.push_update(trace).map_err(|err| {
            SettlementStoreError::Backend(format!("HJMT segment stream rejected: {err}"))
        })?;
    }
    Ok(())
}

type ScopeInputKey = ([u8; 32], u32);
type ScopeOutputKey = ([u8; 32], [u8; 32], u32, [u8; 32]);

// Storage-created scopes, canonical SettlementPath lookup, and durable HJMT
// commits stay behind this facade. Runtime may supply route context, but
// wallet callers must stay on public proofs/API only.
impl SettlementStore {
    pub(crate) fn hjmt_root(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        Ok(self.hjmt_roots.sem_root())
    }

    pub(crate) fn hjmt_check_root(&self) -> Result<CheckRoot, SettlementStoreError> {
        Ok(self.hjmt_root()?.into())
    }

    pub(crate) fn hjmt_get_settlement_item(
        &self,
        path: &SettlementPath,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        if self.cached_path_for_terminal(path.terminal_id) != Some(*path) {
            return Ok(None);
        }

        Ok(self.model.item_opt(path)?)
    }

    pub(crate) fn hjmt_lookup_settlement(
        &self,
        lookup: SettlementLookup,
    ) -> Result<Option<StoreItem>, SettlementStoreError> {
        match lookup {
            SettlementLookup::Path(path) => self.hjmt_get_settlement_item(&path),
            SettlementLookup::Terminal(terminal_id) => self
                .cached_path_for_terminal(terminal_id)
                .map(|path| self.hjmt_get_settlement_item(&path))
                .transpose()
                .map(Option::flatten),
        }
    }

    pub(crate) fn hjmt_list_settlement(
        &self,
        req: SettlementListReq,
    ) -> Result<SettlementPage, SettlementStoreError> {
        let limit = req.limit().max(1);
        let mut items = Vec::new();
        let mut next = None;

        for path in self.sorted_paths() {
            if !settlement_path_in_bounds(&path, &req) {
                continue;
            }

            if let Some(item) = self.hjmt_get_settlement_item(&path)? {
                if !settlement_item_matches(&item, req.scope()) {
                    continue;
                }

                if items.len() == limit {
                    next = items
                        .last()
                        .map(|page_item: &StoreItem| SettlementPageTok::new(page_item.path()));
                    break;
                }

                items.push(item);
            }
        }

        Ok(SettlementPage::new(items, next))
    }

    pub(crate) fn hjmt_put_settlement_item(
        &mut self,
        item: StoreItem,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_ops(vec![StoreOp::Put(Box::new(item))])
    }

    pub(crate) fn hjmt_del_settlement_item(
        &mut self,
        path: &SettlementPath,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_ops(vec![StoreOp::Delete(*path)])
    }

    pub(crate) fn hjmt_apply_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_ops_route(ops, None, None)
    }

    pub(crate) fn hjmt_apply_ops_with_delta(
        &mut self,
        ops: Vec<StoreOp>,
        delta_set: Option<ObjectDeltaSetV1>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_ops_route(ops, delta_set, None)
    }

    fn hjmt_apply_ops_route(
        &mut self,
        ops: Vec<StoreOp>,
        delta_set: Option<ObjectDeltaSetV1>,
        route: Option<SettlementRouteCtx>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if ops.is_empty() {
            return self.hjmt_root();
        }

        timing::clear();
        let result = timing::run("hjmt_apply_ops", || {
            let plan = self.sched_plan_ops_with_delta(&ops, delta_set, None)?;
            if !plan.has_ops() {
                return self.hjmt_root();
            }

            self.commit_hjmt_plan(plan, &[], None, route, None)
        });
        timing::flush();
        result
    }

    pub(crate) fn hjmt_apply_settlement_ops(
        &mut self,
        ops: Vec<StoreOp>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_ops(ops)
    }

    pub(crate) fn hjmt_apply_exec_handoff(
        &mut self,
        handoff: SettlementExecHandoff,
    ) -> Result<ScopeFlow, SettlementStoreError> {
        let (route, ops, txs) = handoff.into_parts();
        let prev_root = self.hjmt_root()?;
        if ops.is_empty() {
            if !txs.is_empty() {
                return Err(SettlementStoreError::Backend(
                    "checkpoint exec rows require terminal settlement ops".to_string(),
                ));
            }
            return Ok(ScopeFlow::new(route, Vec::new(), prev_root, prev_root));
        }

        let exec_ops = self.exec_terminal_ops(&ops)?;
        if exec_ops.is_empty() && !txs.is_empty() {
            return Err(SettlementStoreError::Backend(
                "checkpoint exec rows require terminal settlement ops".to_string(),
            ));
        }
        if !exec_ops.is_empty() && txs.is_empty() {
            return Err(SettlementStoreError::Backend(
                "terminal settlement ops require checkpoint exec rows".to_string(),
            ));
        }
        if !exec_ops.is_empty() {
            self.check_exec_ops(&exec_ops, &txs)?;
            if exec_ops.len() != ops.len() {
                return Err(SettlementStoreError::Backend(
                    "checkpoint exec rows cannot mix terminal and non-terminal settlement ops"
                        .to_string(),
                ));
            }
        }

        let flow_items = self.scope_flow_items(&ops, &txs)?;
        let post_root = if exec_ops.is_empty() {
            self.hjmt_apply_ops_route(ops, None, Some(route))?
        } else {
            self.hjmt_apply_attest_route(ops, txs, None, Some(route))?
        };

        Ok(ScopeFlow::new(route, flow_items, prev_root, post_root))
    }

    /// Apply the canonical checkpoint execution handoff and retain every trace
    /// emitted by that one HJMT commit in a frozen V2 envelope.
    pub(crate) fn hjmt_apply_exec_handoff_with_update_trace(
        &mut self,
        handoff: SettlementExecHandoff,
        segment_dir: &Path,
        segment_context: JmtTraceSegmentContextV2,
    ) -> Result<(ScopeFlow, SettlementUpdateTraceEnvelopeV2), SettlementStoreError> {
        if handoff.is_recursive_v2_noop() {
            let (route, ops, txs) = handoff.into_parts();
            debug_assert!(ops.is_empty() && txs.is_empty());
            let root = self.hjmt_root()?;
            let envelope = SettlementUpdateTraceEnvelopeV2::new_noop(RootGeneration::SettlementV2)
                .map_err(|err| {
                    SettlementStoreError::Backend(format!(
                        "recursive V2 no-op envelope rejected: {err}"
                    ))
                })?;
            return Ok((ScopeFlow::new(route, Vec::new(), root, root), envelope));
        }
        let (route, ops, txs) = handoff.into_parts();
        let prev_root = self.hjmt_root()?;
        if ops.is_empty() || txs.is_empty() {
            return Err(SettlementStoreError::Backend(
                "recursive V2 execution requires nonempty terminal ops and checkpoint rows"
                    .to_string(),
            ));
        }

        let exec_ops = self.exec_terminal_ops(&ops)?;
        if exec_ops.is_empty() || exec_ops.len() != ops.len() {
            return Err(SettlementStoreError::Backend(
                "recursive V2 execution accepts only canonical terminal settlement ops".to_string(),
            ));
        }
        self.check_exec_ops(&exec_ops, &txs)?;
        let flow_items = self.scope_flow_items(&ops, &txs)?;
        let (post_root, trace) = self.hjmt_apply_attest_route_with_update_trace(
            ops,
            txs,
            None,
            Some(route),
            segment_dir,
            segment_context,
        )?;
        Ok((
            ScopeFlow::new(route, flow_items, prev_root, post_root),
            trace,
        ))
    }

    pub(crate) fn hjmt_apply_attest_exec(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_attest_route(ops, txs, None, None)
    }

    pub(crate) fn hjmt_apply_attest_delta(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
        delta_set: Option<ObjectDeltaSetV1>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_attest_route(ops, txs, delta_set, None)
    }

    fn hjmt_apply_attest_route(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
        delta_set: Option<ObjectDeltaSetV1>,
        route: Option<SettlementRouteCtx>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if ops.is_empty() {
            return self.hjmt_root();
        }

        timing::clear();
        let result = timing::run("hjmt_apply_attest_exec", || {
            self.check_exec_ops(&ops, &txs)?;
            let plan = self.sched_plan_ops_with_delta(&ops, delta_set, None)?;
            if !plan.has_ops() {
                return Err(SettlementStoreError::Backend(
                    "attested HJMT execution cannot encode a no-op plan".to_string(),
                ));
            }
            self.commit_hjmt_plan(plan, &[], None, route, Some(txs))
        });
        timing::flush();
        result
    }

    fn hjmt_apply_attest_route_with_update_trace(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
        delta_set: Option<ObjectDeltaSetV1>,
        route: Option<SettlementRouteCtx>,
        segment_dir: &Path,
        segment_context: JmtTraceSegmentContextV2,
    ) -> Result<(SettlementStateRoot, SettlementUpdateTraceEnvelopeV2), SettlementStoreError> {
        if ops.is_empty() {
            return Err(SettlementStoreError::Backend(
                "recursive V2 update trace cannot encode an empty operation set".to_string(),
            ));
        }

        timing::clear();
        let result = timing::run("hjmt_apply_attest_exec", || {
            self.check_exec_ops(&ops, &txs)?;
            let plan = self.sched_plan_ops_with_delta(&ops, delta_set, None)?;
            if !plan.has_ops() {
                return Err(SettlementStoreError::Backend(
                    "recursive V2 update trace cannot encode a no-op plan".to_string(),
                ));
            }

            let version = next_ver(self.hjmt_roots.version);
            let mut trace_builder =
                SettlementUpdateTraceBuilderV2::create_in(segment_dir, segment_context).map_err(
                    |err| {
                        SettlementStoreError::Backend(format!("HJMT segment spool rejected: {err}"))
                    },
                )?;
            let root = self.commit_hjmt_plan_at_with_trace_sink(
                plan,
                version,
                &[],
                None,
                self.backend.is_on(),
                route,
                Some(txs),
                Some(&mut trace_builder),
            )?;
            let envelope = trace_builder.finish().map_err(|err| {
                SettlementStoreError::Backend(format!(
                    "recursive V2 trace envelope rejected: {err}"
                ))
            })?;
            Ok((root, envelope))
        });
        timing::flush();
        result
    }

    pub(crate) fn hjmt_apply_fee_ops(
        &mut self,
        ops: Vec<StoreOp>,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_fee_delta(ops, envelope, actor, None)
    }

    pub(crate) fn hjmt_apply_fee_delta(
        &mut self,
        ops: Vec<StoreOp>,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
        delta_set: Option<ObjectDeltaSetV1>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if ops.is_empty() {
            return Err(SettlementStoreError::Backend(
                "fee support commit requires settlement ops".to_string(),
            ));
        }

        timing::clear();
        let result = timing::run("hjmt_apply_fee_ops", || {
            let support = self.fee_support_ctx(&ops)?;
            let fee_row = self.check_fee_support(&envelope, support, actor)?;
            let plan = self.sched_plan_ops_with_delta(&ops, delta_set, Some(envelope))?;
            if !plan.has_ops() {
                return Err(SettlementStoreError::Backend(
                    "fee support commit requires materialized settlement ops".to_string(),
                ));
            }

            self.commit_hjmt_plan(plan, &[], Some(fee_row), None, None)
        });
        timing::flush();
        result
    }

    pub(crate) fn hjmt_apply_fee_attest_exec(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.hjmt_apply_fee_attest_delta(ops, txs, envelope, actor, None)
    }

    pub(crate) fn hjmt_apply_fee_attest_delta(
        &mut self,
        ops: Vec<StoreOp>,
        txs: Vec<crate::checkpoint::CheckpointExecTx>,
        envelope: FeeEnvelope,
        actor: FeeActorCtx,
        delta_set: Option<ObjectDeltaSetV1>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if ops.is_empty() {
            return Err(SettlementStoreError::Backend(
                "fee support commit requires settlement ops".to_string(),
            ));
        }

        timing::clear();
        let result = timing::run("hjmt_apply_fee_attest_exec", || {
            self.check_exec_ops(&ops, &txs)?;
            let support = self.fee_support_exec_ctx(&ops, &txs)?;
            let fee_row = self.check_fee_support(&envelope, support, actor)?;
            let plan = self.sched_plan_ops_with_delta(&ops, delta_set, Some(envelope))?;
            if !plan.has_ops() {
                return Err(SettlementStoreError::Backend(
                    "fee support commit requires materialized settlement ops".to_string(),
                ));
            }

            self.commit_hjmt_plan(plan, &[], Some(fee_row), None, Some(txs))
        });
        timing::flush();
        result
    }

    pub(crate) fn hjmt_apply_claim_ops(
        &mut self,
        ops: Vec<StoreOp>,
        claims: &[ClaimNullTx],
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        if claims.is_empty() {
            return self.hjmt_apply_ops(ops);
        }
        if ops.is_empty() {
            return Err(SettlementStoreError::Backend(
                "claim nullifier commit requires settlement ops".to_string(),
            ));
        }

        timing::clear();
        let result = timing::run("hjmt_apply_claim_ops", || {
            let plan = self.sched_plan_ops(&ops)?;
            if !plan.has_ops() {
                return Err(SettlementStoreError::Backend(
                    "claim nullifier commit requires materialized settlement ops".to_string(),
                ));
            }

            self.commit_hjmt_plan(plan, claims, None, None, None)
        });
        timing::flush();
        result
    }

    pub(crate) fn sched_plan_ops(&self, ops: &[StoreOp]) -> Result<HjmtPlan, SettlementStoreError> {
        timing::run("hjmt_plan_ops", || {
            self.sched_run_local_queued("hjmt_plan_ops", ops.len(), || self.hjmt_plan_ops(ops))
        })
    }

    pub(crate) fn sched_plan_ops_with_delta(
        &self,
        ops: &[StoreOp],
        delta_set: Option<ObjectDeltaSetV1>,
        fee_envelope: Option<FeeEnvelope>,
    ) -> Result<HjmtPlan, SettlementStoreError> {
        timing::run("hjmt_plan_ops", || {
            self.sched_run_local_queued("hjmt_plan_ops", ops.len(), || {
                self.hjmt_plan_ops_with_delta(ops, delta_set, fee_envelope)
            })
        })
    }

    pub(crate) fn hjmt_validate_reload(&self) -> Result<SettlementStateRoot, SettlementStoreError> {
        let model_root = self.model.root()?;
        if model_root != self.hjmt_roots.sem_root {
            return Err(SettlementStoreError::Backend(
                "hjmt reload validation root mismatch".to_string(),
            ));
        }

        let mut expected = BTreeMap::new();
        for path in self.model.paths() {
            expected.insert(path.terminal_id(), path);
        }
        let got: BTreeMap<_, _> = self
            .path_by_terminal_id
            .iter()
            .map(|(terminal_id, path)| (*terminal_id, *path))
            .collect();
        if expected != got {
            return Err(SettlementStoreError::Backend(
                "hjmt reload validation path index mismatch".to_string(),
            ));
        }

        Ok(model_root)
    }

    fn exec_terminal_ops(&self, ops: &[StoreOp]) -> Result<Vec<StoreOp>, SettlementStoreError> {
        let mut out = Vec::new();
        for op in ops {
            match op {
                StoreOp::Put(item) => {
                    if item.terminal_leaf().is_ok() {
                        out.push(op.clone());
                    }
                }
                StoreOp::Delete(path) => {
                    let item = self.load_item(path)?;
                    if item.terminal_leaf().is_ok() {
                        out.push(StoreOp::Delete(*path));
                    }
                }
            }
        }

        Ok(out)
    }

    fn scope_flow_items(
        &self,
        ops: &[StoreOp],
        txs: &[crate::checkpoint::CheckpointExecTx],
    ) -> Result<Vec<ScopeFlowItem>, SettlementStoreError> {
        let input_ids = bind_input_ids(txs)?;
        let output_ids = bind_output_ids(txs)?;
        let mut items = Vec::with_capacity(ops.len());
        let mut seen_defs = BTreeSet::new();
        let mut seen_serials = BTreeSet::new();

        for (index, op) in ops.iter().enumerate() {
            match op {
                StoreOp::Put(item) => {
                    let path = item.path();
                    let first_def = !self.model.has_def(path.definition_id)
                        && seen_defs.insert(path.definition_id);
                    let first_serial = !self.model.has_serial(path.definition_id, path.serial_id)
                        && seen_serials.insert((path.definition_id, path.serial_id));
                    let tx_id = if item.terminal_leaf().is_ok() {
                        let leaf_hash = terminal_value_hash(item.leaf())?.0;
                        let key = (
                            path.terminal_id().into_bytes(),
                            path.definition_id.into_bytes(),
                            path.serial_id.get(),
                            leaf_hash,
                        );
                        output_ids.get(&key).cloned().ok_or_else(|| {
                            SettlementStoreError::Backend(
                                "terminal settlement op is missing tx output binding".to_string(),
                            )
                        })?
                    } else {
                        semantic_tx_id(index)
                    };
                    items.push(ScopeFlowItem {
                        tx_id,
                        op_kind: ScopeOpKind::Put,
                        definition_id: hex_id(path.definition_id.into_bytes()),
                        serial_id: path.serial_id.get(),
                        terminal_id: hex_id(path.terminal_id().into_bytes()),
                        leaf_value_hash: terminal_value_hash(item.leaf())?.0,
                        leaf_family: scope_leaf_kind(item),
                        first_seen: ScopeSeen {
                            definition: first_def,
                            serial: first_serial,
                            object: self.hjmt_get_settlement_item(&path)?.is_none(),
                        },
                    });
                }
                StoreOp::Delete(path) => {
                    let item = self.load_item(path)?;
                    let tx_id = if item.terminal_leaf().is_ok() {
                        let key = (path.terminal_id().into_bytes(), path.serial_id.get());
                        input_ids.get(&key).cloned().ok_or_else(|| {
                            SettlementStoreError::Backend(
                                "terminal settlement delete is missing tx input binding"
                                    .to_string(),
                            )
                        })?
                    } else {
                        semantic_tx_id(index)
                    };
                    items.push(ScopeFlowItem {
                        tx_id,
                        op_kind: ScopeOpKind::Delete,
                        definition_id: hex_id(path.definition_id.into_bytes()),
                        serial_id: path.serial_id.get(),
                        terminal_id: hex_id(path.terminal_id().into_bytes()),
                        leaf_value_hash: terminal_value_hash(item.leaf())?.0,
                        leaf_family: scope_leaf_kind(&item),
                        first_seen: ScopeSeen {
                            definition: false,
                            serial: false,
                            object: false,
                        },
                    });
                }
            }
        }

        Ok(items)
    }

    fn commit_hjmt_plan(
        &mut self,
        plan: HjmtPlan,
        claims: &[ClaimNullTx],
        fee_row: Option<FeeReplayRec>,
        route: Option<SettlementRouteCtx>,
        txs: Option<Vec<crate::checkpoint::CheckpointExecTx>>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let version = next_ver(self.hjmt_roots.version);
        self.commit_hjmt_plan_at(
            plan,
            version,
            claims,
            fee_row,
            self.backend.is_on(),
            route,
            txs,
        )
    }

    pub(crate) fn commit_hjmt_plan_at(
        &mut self,
        plan: HjmtPlan,
        version: u64,
        rollback_claims: &[ClaimNullTx],
        fee_row: Option<FeeReplayRec>,
        persist: bool,
        route: Option<SettlementRouteCtx>,
        txs: Option<Vec<crate::checkpoint::CheckpointExecTx>>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        self.commit_hjmt_plan_at_with_trace_sink(
            plan,
            version,
            rollback_claims,
            fee_row,
            persist,
            route,
            txs,
            None,
        )
    }

    fn commit_hjmt_plan_at_with_trace_sink(
        &mut self,
        plan: HjmtPlan,
        version: u64,
        rollback_claims: &[ClaimNullTx],
        fee_row: Option<FeeReplayRec>,
        persist: bool,
        route: Option<SettlementRouteCtx>,
        txs: Option<Vec<crate::checkpoint::CheckpointExecTx>>,
        mut trace_sink: Option<&mut SettlementUpdateTraceBuilderV2>,
    ) -> Result<SettlementStateRoot, SettlementStoreError> {
        let prev_root = self.hjmt_root()?;
        let object_delta = plan.delta_set.clone();
        let write_arts = if persist {
            Some(self.hjmt_plan_arts(version, &plan.ops, txs.as_deref())?)
        } else {
            None
        };
        let claim_rows = self.build_claim_rows(rollback_claims)?;
        let store_snap = self.hjmt_store.snap();
        let cache_snap = self.forest_cache.snapshot();
        let roots_snap = self.hjmt_roots.clone();
        let state_snap = self.snap_store();
        let mut next_roots = self.hjmt_roots.clone();
        let plan_root = plan.root;
        let touched_buckets = plan.touched_buckets.clone();
        let result = (|| {
            timing::run("hjmt_child_commit", || {
                self.commit_terminal_batches(
                    version,
                    &plan,
                    &mut next_roots,
                    trace_sink.as_deref_mut(),
                )
            })?;
            timing::run("hjmt_parent_commit", || {
                self.commit_bucket_batches(
                    version,
                    &plan,
                    &mut next_roots,
                    trace_sink.as_deref_mut(),
                )?;
                self.commit_serial_batches(
                    version,
                    &plan,
                    &mut next_roots,
                    trace_sink.as_deref_mut(),
                )?;
                self.commit_definition_batch(
                    version,
                    &plan,
                    &mut next_roots,
                    trace_sink.as_deref_mut(),
                )?;
                self.ensure_live_hjmt_tree_versions(version, &plan.live_model)
            })?;
            if !plan.path_batch.is_empty() {
                timing::run("hjmt_path_index_commit", || {
                    let out = self.commit_tree_job(TreeJob {
                        tree_id: plan.path_batch.tree_id,
                        ops: plan.path_batch.ops.clone(),
                        snap: self.hjmt_store.tree_snap(plan.path_batch.tree_id),
                        version,
                    })?;
                    verify_tree_out_trace(&out)?;
                    let TreeOut {
                        tree_id,
                        root,
                        trace,
                        snap,
                    } = out;
                    record_streamed_trace(trace_sink, trace)?;
                    self.hjmt_store.restore_tree(tree_id, snap);
                    Ok::<RootHash, SettlementStoreError>(root)
                })?;
            }

            next_roots.version = version;
            next_roots.sem_root = plan_root;
            next_roots.settlement_root = plan_root;
            self.hjmt_roots = next_roots;
            plan.next.merge_into(self);
            if let Some(fee_row) = fee_row {
                self.commit_fee_replay_row(fee_row);
            }
            self.commit_claim_rows(&claim_rows);
            self.hjmt_roots.journal_digest = Some(self.hjmt_live_journal_digest(
                version,
                prev_root,
                plan_root,
                &touched_buckets,
                route,
            )?);
            self.invalidate_forest_cache_for_ops(&plan.ops, &touched_buckets);
            if let Some(write_arts) = write_arts {
                let persist_work = self.hjmt_persist_work(
                    write_arts,
                    version,
                    prev_root,
                    plan_root,
                    &touched_buckets,
                    route,
                    Some(object_delta.clone()),
                )?;
                timing::run("hjmt_journal_sync", || {
                    self.sched_block("hjmt_journal_sync", || {
                        self.backend
                            .sync_hjmt_work(persist_work)
                            .map_err(|err| SettlementStoreError::Backend(err.to_string()))
                    })
                })?;
            }
            self.settlement_root_by_ver.insert(version, plan_root);
            timing::run("hjmt_model_hist", || {
                self.model_by_ver.insert(version, self.model.clone());
            });
            self.hjmt_roots_by_ver
                .insert(version, self.hjmt_roots.clone());
            self.last_object_delta = Some(object_delta.clone());
            self.object_deltas_by_ver.insert(version, object_delta);
            self.refresh_cache_post_commit();
            Ok(plan_root)
        })();

        if let Err(err) = result {
            self.hjmt_store.restore(store_snap);
            self.forest_cache.restore(cache_snap);
            self.hjmt_roots = roots_snap;
            self.restore_store(state_snap);
            return Err(err);
        }

        result
    }

    fn refresh_cache_post_commit(&self) {
        // Cache warming is a service lane, not commit truth. If it drifts, drop
        // the cache and let later reads rebuild it from durable state.
        let warmed = self.sched_run_local("hjmt_cache_warm", || self.warm_forest_cache_current());
        if warmed.is_err() {
            self.forest_cache.clear_all();
            return;
        }
        if self
            .sched_run_local("hjmt_cache_verify", || self.verify_forest_cache_sample())
            .is_err()
        {
            self.forest_cache.clear_all();
        }
    }

    fn hjmt_live_journal_digest(
        &self,
        version: u64,
        prev_root: SettlementStateRoot,
        next_root: SettlementStateRoot,
        touched_buckets: &[HjmtBucketKey],
        route: Option<SettlementRouteCtx>,
    ) -> Result<[u8; 32], SettlementStoreError> {
        let terminal_rows = self.ser_hjmt_terminal_rows()?;
        let settlement_path_rows = self.ser_hjmt_settlement_path_rows()?;
        let claim_rows = self.ser_claim_null_rows();
        let fee_rows = self.ser_fee_replay_rows();
        let root_rows = self.ser_hjmt_root_rows(version);
        let (child_root_rows, parent_root_rows) = split_hjmt_root_rows(&root_rows);
        let child_digest = hjmt_child_digest(
            &terminal_rows,
            &settlement_path_rows,
            &claim_rows,
            &fee_rows,
            &child_root_rows,
        )?;
        let parent_digest = hjmt_parent_digest(&parent_root_rows)?;
        let mut entry = HjmtCommitJournalEntry::new(
            version,
            version,
            self.bucket_policy().bucket_policy_id(),
            prev_root,
            next_root,
            touched_buckets,
            child_digest,
            parent_digest,
        )
        .with_status(HjmtCommitStatus::RootPublished);
        if let Some(route) = route {
            entry = entry.with_route(route);
        }
        entry.seal_fee_replay_state(&fee_rows);
        Ok(hjmt_journal_digest(&entry))
    }

    fn hjmt_persist_work(
        &self,
        write_arts: WriteArts,
        version: u64,
        prev_root: SettlementStateRoot,
        next_root: SettlementStateRoot,
        touched_buckets: &[HjmtBucketKey],
        route: Option<SettlementRouteCtx>,
        object_delta: Option<ObjectDeltaSetV1>,
    ) -> Result<HjmtPersistWork, SettlementStoreError> {
        Ok(HjmtPersistWork {
            write_arts,
            version,
            prev_root,
            next_root,
            route,
            bucket_policy_id: self.bucket_policy().bucket_policy_id(),
            touched_buckets: touched_buckets.to_vec(),
            terminal_rows: self.ser_hjmt_terminal_rows()?,
            settlement_path_rows: self.ser_hjmt_settlement_path_rows()?,
            claim_rows: self.ser_claim_null_rows(),
            fee_rows: self.ser_fee_replay_rows(),
            root_rows: self.ser_hjmt_root_rows(version),
            def_root: self.ser_hjmt_def_root(),
            object_delta,
        })
    }

    fn commit_terminal_batches(
        &mut self,
        version: u64,
        plan: &HjmtPlan,
        next_roots: &mut HjmtRoots,
        mut trace_sink: Option<&mut SettlementUpdateTraceBuilderV2>,
    ) -> Result<(), SettlementStoreError> {
        let jobs = plan
            .terminal_batches
            .iter()
            .map(|batch| TreeJob {
                tree_id: batch.tree_id,
                ops: batch.ops.clone(),
                snap: self.hjmt_store.tree_snap(batch.tree_id),
                version,
            })
            .collect::<Vec<_>>();

        for job in jobs {
            let out = self.commit_tree_job(job)?;
            verify_tree_out_trace(&out)?;
            let TreeOut {
                tree_id,
                root,
                trace,
                snap,
            } = out;
            record_streamed_trace(trace_sink.as_deref_mut(), trace)?;
            let HjmtTreeId::BucketTerminal(definition_id, serial_id, bucket_id) = tree_id else {
                return Err(SettlementStoreError::Backend(
                    "hjmt terminal batch used non-terminal tree".to_string(),
                ));
            };
            self.hjmt_store.restore_tree(tree_id, snap);
            let key = (definition_id, serial_id, bucket_id);
            if model_has_bucket(&plan.next.model, self.bucket_policy(), key) {
                next_roots
                    .terminal_roots
                    .insert(key, TreeRootRef::new(root.0));
            } else {
                next_roots.terminal_roots.remove(&key);
            }
        }

        Ok(())
    }

    fn ensure_live_hjmt_tree_versions(
        &mut self,
        version: u64,
        model: &SettlementModel,
    ) -> Result<(), SettlementStoreError> {
        let mut tree_ids = BTreeSet::new();
        for path in model.paths() {
            let bucket_id = self.bucket_policy().derive_bucket_id(path);
            tree_ids.insert(HjmtTreeId::Definition);
            tree_ids.insert(HjmtTreeId::Serial(path.definition_id));
            tree_ids.insert(HjmtTreeId::Bucket(path.definition_id, path.serial_id));
            tree_ids.insert(HjmtTreeId::BucketTerminal(
                path.definition_id,
                path.serial_id,
                bucket_id,
            ));
            tree_ids.insert(HjmtTreeId::PathIndex);
        }

        let jobs = tree_ids
            .into_iter()
            .map(|tree_id| EnsureJob {
                tree_id,
                snap: self.hjmt_store.tree_snap(tree_id),
                version,
            })
            .collect::<Vec<_>>();
        for out in self.ensure_tree_jobs("hjmt_ensure_version", jobs)? {
            self.hjmt_store.restore_tree(out.tree_id, out.snap);
        }
        Ok(())
    }

    fn commit_bucket_batches(
        &mut self,
        version: u64,
        plan: &HjmtPlan,
        next_roots: &mut HjmtRoots,
        mut trace_sink: Option<&mut SettlementUpdateTraceBuilderV2>,
    ) -> Result<(), SettlementStoreError> {
        let mut by_serial = BTreeMap::<HjmtSerialKey, Vec<HjmtBucketKey>>::new();
        for key in &plan.touched_buckets {
            by_serial.entry((key.0, key.1)).or_default().push(*key);
        }

        let mut jobs = Vec::new();
        for ((definition_id, serial_id), bucket_keys) in by_serial {
            let mut batch = HjmtBatch {
                tree_id: HjmtTreeId::Bucket(definition_id, serial_id),
                ops: Vec::new(),
            };

            for key in bucket_keys {
                let old_leaf = self.bucket_leaf_from_roots(&self.hjmt_roots, &self.model, key)?;
                let new_leaf = self.bucket_leaf_from_roots(next_roots, &plan.next.model, key)?;
                if old_leaf != new_leaf {
                    batch.push(
                        bucket_key_for_path(key.2),
                        new_leaf.map(|leaf| leaf.encode()),
                    );
                }
            }

            if batch.is_empty() {
                continue;
            }
            jobs.push(TreeJob {
                tree_id: batch.tree_id,
                ops: batch.ops,
                snap: self.hjmt_store.tree_snap(batch.tree_id),
                version,
            });
        }

        for job in jobs {
            let out = self.commit_tree_job(job)?;
            verify_tree_out_trace(&out)?;
            let TreeOut {
                tree_id,
                root,
                trace,
                snap,
            } = out;
            record_streamed_trace(trace_sink.as_deref_mut(), trace)?;
            let HjmtTreeId::Bucket(definition_id, serial_id) = tree_id else {
                return Err(SettlementStoreError::Backend(
                    "hjmt bucket batch used non-bucket tree".to_string(),
                ));
            };
            self.hjmt_store.restore_tree(tree_id, snap);
            let serial_key = (definition_id, serial_id);
            if model_has_serial(&plan.next.model, definition_id, serial_id) {
                next_roots
                    .bucket_roots
                    .insert(serial_key, TreeRootRef::new(root.0));
            } else {
                next_roots.bucket_roots.remove(&serial_key);
            }
        }

        Ok(())
    }

    fn commit_serial_batches(
        &mut self,
        version: u64,
        plan: &HjmtPlan,
        next_roots: &mut HjmtRoots,
        mut trace_sink: Option<&mut SettlementUpdateTraceBuilderV2>,
    ) -> Result<(), SettlementStoreError> {
        let mut by_def = BTreeMap::<DefinitionId, BTreeSet<SerialId>>::new();
        for key in &plan.touched_buckets {
            by_def.entry(key.0).or_default().insert(key.1);
        }

        let mut jobs = Vec::new();
        for (definition_id, serial_ids) in by_def {
            let mut batch = HjmtBatch {
                tree_id: HjmtTreeId::Serial(definition_id),
                ops: Vec::new(),
            };

            for serial_id in serial_ids {
                let key = (definition_id, serial_id);
                let old_leaf = self.serial_leaf_from_roots(&self.hjmt_roots, &self.model, key)?;
                let new_leaf = self.serial_leaf_from_roots(next_roots, &plan.next.model, key)?;
                if old_leaf != new_leaf {
                    batch.push(
                        serial_key(definition_id, serial_id),
                        new_leaf.map(ser_payload),
                    );
                }
            }

            if batch.is_empty() {
                continue;
            }
            jobs.push(TreeJob {
                tree_id: batch.tree_id,
                ops: batch.ops,
                snap: self.hjmt_store.tree_snap(batch.tree_id),
                version,
            });
        }

        for job in jobs {
            let out = self.commit_tree_job(job)?;
            verify_tree_out_trace(&out)?;
            let TreeOut {
                tree_id,
                root,
                trace,
                snap,
            } = out;
            record_streamed_trace(trace_sink.as_deref_mut(), trace)?;
            let HjmtTreeId::Serial(definition_id) = tree_id else {
                return Err(SettlementStoreError::Backend(
                    "hjmt serial batch used non-serial tree".to_string(),
                ));
            };
            self.hjmt_store.restore_tree(tree_id, snap);
            if model_has_definition(&plan.next.model, definition_id) {
                next_roots
                    .serial_roots
                    .insert(definition_id, TreeRootRef::new(root.0));
            } else {
                next_roots.serial_roots.remove(&definition_id);
            }
        }

        Ok(())
    }

    fn commit_definition_batch(
        &mut self,
        version: u64,
        plan: &HjmtPlan,
        next_roots: &mut HjmtRoots,
        trace_sink: Option<&mut SettlementUpdateTraceBuilderV2>,
    ) -> Result<(), SettlementStoreError> {
        let mut def_ids = BTreeSet::new();
        for key in &plan.touched_buckets {
            def_ids.insert(key.0);
        }

        let mut batch = HjmtBatch {
            tree_id: HjmtTreeId::Definition,
            ops: Vec::new(),
        };

        for definition_id in def_ids {
            let old_leaf =
                self.definition_leaf_from_roots(&self.hjmt_roots, &self.model, definition_id)?;
            let new_leaf =
                self.definition_leaf_from_roots(next_roots, &plan.next.model, definition_id)?;
            if old_leaf != new_leaf {
                batch.push(definition_key(definition_id), new_leaf.map(def_payload));
            }
        }

        if batch.is_empty() {
            return Ok(());
        }

        let out = self.commit_tree_job(TreeJob {
            tree_id: batch.tree_id,
            ops: batch.ops,
            snap: self.hjmt_store.tree_snap(batch.tree_id),
            version,
        })?;
        verify_tree_out_trace(&out)?;
        let TreeOut {
            tree_id,
            root,
            trace,
            snap,
        } = out;
        record_streamed_trace(trace_sink, trace)?;
        self.hjmt_store.restore_tree(tree_id, snap);
        if plan.has_live_definitions {
            next_roots.def_root = Some(TreeRootRef::new(root.0));
        } else {
            next_roots.def_root = None;
        }
        Ok(())
    }

    fn commit_tree_job(&self, job: TreeJob) -> Result<TreeOut, SettlementStoreError> {
        self.sched_one("hjmt_tree_commit", job, |job| {
            let (root, trace, snap) = super::hjmt_store::HjmtStore::commit_snap_with_update_trace(
                JmtTreeRoleV2::from(job.tree_id),
                job.snap,
                job.ops,
                job.version,
            )?;
            Ok(TreeOut {
                tree_id: job.tree_id,
                root,
                trace,
                snap,
            })
        })
    }

    fn ensure_tree_jobs(
        &self,
        stage: &'static str,
        jobs: Vec<EnsureJob>,
    ) -> Result<Vec<EnsureOut>, SettlementStoreError> {
        self.sched_map(stage, jobs, |job| {
            let snap = super::hjmt_store::HjmtStore::ensure_snap(job.snap, job.version)?;
            Ok(EnsureOut {
                tree_id: job.tree_id,
                snap,
            })
        })
    }

    pub(crate) fn bucket_leaf_from_roots(
        &self,
        roots: &HjmtRoots,
        model: &SettlementModel,
        key: HjmtBucketKey,
    ) -> Result<Option<BucketRootLeaf>, SettlementStoreError> {
        if !model_has_bucket(model, self.bucket_policy(), key) {
            return Ok(None);
        }
        let root = roots.terminal_roots.get(&key).ok_or_else(|| {
            SettlementStoreError::Backend("missing hjmt terminal root for live bucket".to_string())
        })?;
        Ok(Some(BucketRootLeaf {
            definition_id: key.0,
            serial_id: key.1,
            bucket_id: key.2,
            terminal_jmt_root: root.into_bytes(),
            bucket_policy_id: self.bucket_policy().bucket_policy_id(),
        }))
    }

    pub(crate) fn serial_leaf_from_roots(
        &self,
        roots: &HjmtRoots,
        model: &SettlementModel,
        key: HjmtSerialKey,
    ) -> Result<Option<SerialRootLeaf>, SettlementStoreError> {
        if !model_has_serial(model, key.0, key.1) {
            return Ok(None);
        }
        let root = roots.bucket_roots.get(&key).ok_or_else(|| {
            SettlementStoreError::Backend("missing hjmt bucket root for live serial".to_string())
        })?;
        Ok(Some(SerialRootLeaf {
            definition_id: key.0,
            serial_id: key.1,
            serial_root: root.into_bytes(),
        }))
    }

    pub(crate) fn definition_leaf_from_roots(
        &self,
        roots: &HjmtRoots,
        model: &SettlementModel,
        definition_id: DefinitionId,
    ) -> Result<Option<DefinitionRootLeaf>, SettlementStoreError> {
        if !model_has_definition(model, definition_id) {
            return Ok(None);
        }
        let root = roots.serial_roots.get(&definition_id).ok_or_else(|| {
            SettlementStoreError::Backend(
                "missing hjmt serial root for live definition".to_string(),
            )
        })?;
        Ok(Some(DefinitionRootLeaf {
            definition_id,
            definition_root: root.into_bytes(),
        }))
    }
}

fn bind_input_ids(
    txs: &[crate::checkpoint::CheckpointExecTx],
) -> Result<BTreeMap<ScopeInputKey, String>, SettlementStoreError> {
    let mut out = BTreeMap::new();
    for (index, tx) in txs.iter().enumerate() {
        let tx_id = scope_tx_id(index, tx.tx_proof());
        for input in tx.input_refs() {
            let key = (input.terminal_id().into_bytes(), input.serial_id().get());
            if out.insert(key, tx_id.clone()).is_some() {
                return Err(SettlementStoreError::Backend(
                    "checkpoint exec duplicates one terminal input binding".to_string(),
                ));
            }
        }
    }

    Ok(out)
}

fn bind_output_ids(
    txs: &[crate::checkpoint::CheckpointExecTx],
) -> Result<BTreeMap<ScopeOutputKey, String>, SettlementStoreError> {
    let mut out = BTreeMap::new();
    for (index, tx) in txs.iter().enumerate() {
        let tx_id = scope_tx_id(index, tx.tx_proof());
        for output in tx.outputs() {
            let key = (
                output.leaf().asset_id,
                output.definition_id().into_bytes(),
                output.leaf().serial_id,
                terminal_value_hash(output.leaf())?.0,
            );
            if out.insert(key, tx_id.clone()).is_some() {
                return Err(SettlementStoreError::Backend(
                    "checkpoint exec duplicates one terminal output binding".to_string(),
                ));
            }
        }
    }

    Ok(out)
}

fn hex_id(bytes: [u8; 32]) -> String {
    z00z_crypto::expert::encoding::to_hex(&bytes)
}

fn semantic_tx_id(index: usize) -> String {
    format!("semantic-{index:04}")
}

fn scope_leaf_kind(item: &StoreItem) -> ScopeLeafKind {
    if item.terminal_leaf().is_ok() {
        return ScopeLeafKind::Terminal;
    }
    ScopeLeafKind::Right
}

fn settlement_path_in_bounds(path: &SettlementPath, req: &SettlementListReq) -> bool {
    req.start().is_none_or(|start_path| *path >= start_path)
        && req.end().is_none_or(|end_path| *path <= end_path)
        && req.after().is_none_or(|page_tok| *path > page_tok.path())
}

fn settlement_item_matches(item: &StoreItem, scope: SettlementScope) -> bool {
    let path = item.path();
    match scope {
        SettlementScope::All => true,
        SettlementScope::Def(definition_id) => path.definition_id == definition_id,
        SettlementScope::Ser(definition_id, serial_id) => {
            path.definition_id == definition_id && path.serial_id == serial_id
        }
        SettlementScope::RightClass(right_class) => item
            .right_leaf()
            .is_ok_and(|leaf| leaf.right_class == right_class),
    }
}

type HjmtRootRow = (Vec<u8>, [u8; 32]);

fn split_hjmt_root_rows(root_rows: &[HjmtRootRow]) -> (Vec<HjmtRootRow>, Vec<HjmtRootRow>) {
    root_rows
        .iter()
        .cloned()
        .partition(|(key, _)| key.get(8).copied() == Some(4))
}

fn model_has_bucket(
    model: &SettlementModel,
    policy: crate::settlement::BucketPolicy,
    key: HjmtBucketKey,
) -> bool {
    model.paths().into_iter().any(|path| {
        path.definition_id == key.0
            && path.serial_id == key.1
            && policy.derive_bucket_id(path) == key.2
    })
}

fn model_has_serial(
    model: &SettlementModel,
    definition_id: DefinitionId,
    serial_id: SerialId,
) -> bool {
    model
        .paths()
        .into_iter()
        .any(|path| path.definition_id == definition_id && path.serial_id == serial_id)
}

fn model_has_definition(model: &SettlementModel, definition_id: DefinitionId) -> bool {
    model
        .paths()
        .into_iter()
        .any(|path| path.definition_id == definition_id)
}
