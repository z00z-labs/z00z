# Refactor Recommendations

Этот документ фиксирует безопасный migration map для HJMT-aligned refactor в `z00z_rollup_node`, `z00z_runtime/*` и `z00z_storage`.

Документ намеренно правится **точечно**, а не переписывается с нуля. Причина простая: ниже уже собран полный path-level inventory, и полный rewrite такой таблицы повышает риск drift по путям и целевым модулям. Поэтому безопаснее сохранить существующий инвентарь и поправить только нормативные выводы, спорные строки и execution order.

## 🎯 Architectural Verdict

- `z00z_rollup_node` должен остаться только orchestration/composition root.
- `z00z_storage` должен стать backend-agnostic на уровне durable seams, но не должен терять ownership над settlement semantics, proof surfaces и deterministic replay rules.
- Planner authority действительно должна быть runtime-owned, как требует `Z00Z-HJMT-Upgrade.md`, но это **не** означает, что весь `tx_plan` надо слепо выносить из storage.
- `snapshot/`, `serialization/`, `checkpoint/` должны остаться отдельными surface area; их нельзя схлопывать в один generic backup layer.
- Rename wave должна идти **после** semantic move wave. Смешивать их в одной фазе нельзя.

## ⚠️ Migration Safety Rules

- `z00z_runtime` здесь не один crate, а namespace-папка с тремя crate: `aggregators`, `validators`, `watchers`.
- Paper target names from `Z00Z-HJMT-Upgrade.md` such as `BatchPlanned`, `AggregatorId`, `ShardPlacementTable`, `ShardExecutor`, `StorageBackend`, and `JournalBackend` must be treated as **target-state vocabulary**, not as a claim that these are already live repo exports.
- Live public/runtime contracts should stay explicitly anchored to current symbols during the migration text: `AggregatorService`, `ValidatorService`, `WatcherService`, `WatcherBoundary`, `ValidatorBoundary`, `IngressBoundary`, `OrderingBoundary`, `RecoveryBoundary`, `SchedulerBoundary`, `SettlementTreeBackend`, and `StoreBackendError`.
- В `z00z_storage` первой фазой нужен backend seam, но без big-bang rewrite публичного surface `SettlementStore`.
- В первой фазе достаточно `redb` + `memory`. `rocksdb` лучше оставить как phase-2 optional adapter, а не создавать stub ради будущего.
- `settlement_proofs.rs` и существующие storage test suites надо использовать как compatibility gate для proof/public API, а не ломать их вместе с первой структурной миграцией.

## 🔒 ZINV Anchors

### ZINV-CHECKPOINT-001

Invariant reference: `ZINV: CHECKPOINT-001`

Любой HJMT refactor или module move из этого migration map обязан сохранить
одну checkpoint/root lineage chain. Ни backend split, ни sharding vocabulary,
ни planner extraction не могут вводить второй canonical root path или ломать
double-consume rejection.

### ZINV-CHECKPOINT-002

Invariant reference: `ZINV: CHECKPOINT-002`

Aggregator publication, watcher export, and DA-facing adapters в этой migration
scheme остаются подчинены уже committed checkpoint state. Любой performance или
module split work должен сохранять правило publish-after-commit.

### ZINV-ASSET-002

Invariant reference: `ZINV: ASSET-002`

Rights, vouchers, wrappers, and any migration-time compatibility facades не
должны открывать transfer path для non-transferable control surfaces. Если
semantic move временно вводит adapter/shim, он обязан fail-closed сохранять
non-transferability.

## ✅ Наилучший способ сделать это

Наилучший и более безопасный способ миграции — **не** переписывать storage и runtime одновременно по всем осям, а двигаться четырьмя разделёнными волнами:

1. **Backend seam wave**: убрать прямое знание о `RedbBackend` из semantic facade и ввести backend contracts.
2. **Adapter move wave**: перенести redb-specific код в `backend/redb/*` и оставить старый semantic surface прежним или почти прежним.
3. **Planner authority split wave**: вынести в runtime только planner-authority logic, связанный с route/shard admission и `BatchPlanned`, но оставить в storage store-side precheck/dry-run helpers.
4. **Rename wave**: только после стабилизации поведения и тестов переименовывать `agg_*`, `val_*`, `types_*`, `README.MD` и подобные элементы.

Это снижает риск drift в трёх местах:

- semantic drift: storage не теряет ownership над settlement semantics;
- API drift: benches и tests продолжают держать публичную proof surface;
- review drift: structural moves и naming cleanup не маскируют друг друга.

## 1) `z00z_rollup_node`

| Current path                                             | Target module               | Move / what to do                             | Better rename                            |
| -------------------------------------------------------- | --------------------------- | --------------------------------------------- | ---------------------------------------- |
| crates/z00z_rollup_node/Cargo.toml                       | crate root                  | keep                                          | no                                       |
| crates/z00z_rollup_node/README.md                        | crate docs                  | keep, update wording after refactor           | no                                       |
| crates/z00z_rollup_node/src/lib.rs                       | `z00z_rollup_node`          | keep as composition root                      | no                                       |
| crates/z00z_rollup_node/src/config.rs                    | `z00z_rollup_node::config`  | keep                                          | no                                       |
| crates/z00z_rollup_node/src/da_adapter.rs                | `z00z_rollup_node::da`      | move role into shorter DA facade              | `da.rs`                                  |
| crates/z00z_rollup_node/src/lifecycle.rs                 | `z00z_rollup_node::runtime` | keep or split if lifecycle gets big           | `runtime.rs` if you want stricter naming |
| crates/z00z_rollup_node/src/mode.rs                      | `z00z_rollup_node::mode`    | keep                                          | `node_mode.rs` if you want explicitness  |
| crates/z00z_rollup_node/src/rpc.rs                       | `z00z_rollup_node::rpc`     | keep                                          | no                                       |
| crates/z00z_rollup_node/src/status.rs                    | `z00z_rollup_node::status`  | keep                                          | `service_status.rs` if you want clarity  |
| crates/z00z_rollup_node/src/empty_file                   | none                        | delete placeholder or replace with `.gitkeep` | `.gitkeep`                               |
| crates/z00z_rollup_node/bin/empty_file                   | none                        | delete placeholder or replace with `.gitkeep` | `.gitkeep`                               |
| crates/z00z_rollup_node/examples/empty_file              | none                        | delete placeholder or replace with `.gitkeep` | `.gitkeep`                               |
| crates/z00z_rollup_node/benches/empty_file               | none                        | delete placeholder or replace with `.gitkeep` | `.gitkeep`                               |
| crates/z00z_rollup_node/tests/empty_file                 | none                        | delete placeholder or replace with `.gitkeep` | `.gitkeep`                               |
| tests/test_rollup_theorem_guard.rs                       | `z00z_rollup_node::tests`   | keep here                                     | no                                       |

## 2) `z00z_runtime` `aggregators`

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_runtime/aggregators/Cargo.toml | crate root | keep | no |
| crates/z00z_runtime/aggregators/README.md | crate docs | keep, update after adding planner/placement/shard executor | no |
| crates/z00z_runtime/aggregators/src/lib.rs | `z00z_runtime::aggregators` | keep as service facade; preserve current public re-exports (`AggregatorService`, `AggregatorIngress`, `AggregatorOrdering`, `AggregatorRecovery`, `IngressBoundary`, `OrderingBoundary`, `RecoveryBoundary`, `SchedulerBoundary`) during file moves | no |
| crates/z00z_runtime/aggregators/src/agg_iface.rs | `z00z_runtime::aggregators::service` | file/module rename is okay, but keep current public trait names stable until a separate API rename wave | `service.rs` (file rename only) |
| crates/z00z_runtime/aggregators/src/agg_ingress.rs | `z00z_runtime::aggregators::ingress` | keep ingress logic here | `ingress.rs` |
| crates/z00z_runtime/aggregators/src/agg_ordering.rs | `z00z_runtime::aggregators::ordering` | move batch admission/ordering pieces out of storage here | `ordering.rs` |
| crates/z00z_runtime/aggregators/src/agg_recovery.rs | `z00z_runtime::aggregators::recovery` | keep failover/recovery here | `recovery.rs` |
| crates/z00z_runtime/aggregators/src/agg_scheduler.rs | `z00z_runtime::aggregators::scheduler` | keep scheduling here | `scheduler.rs` |
| crates/z00z_runtime/aggregators/src/agg_types.rs | `z00z_runtime::aggregators::types` | keep domain types here | types.rs |
| new: crates/z00z_runtime/aggregators/src/batch_planner.rs | `z00z_runtime::aggregators::batch_planner` | move planner-authority logic here: canonicalization, route-table targeting, single-shard admission, `BatchPlanned` digest; do **not** blindly move all store-side `tx_plan` helpers | add new file |
| new: crates/z00z_runtime/aggregators/src/placement.rs | `z00z_runtime::aggregators::placement` | move `AggregatorId`, `ShardPlacementTable`, standby metadata here | add new file |
| new: crates/z00z_runtime/aggregators/src/shard_exec.rs | `z00z_runtime::aggregators::shard_exec` | move `ShardExecutor` runtime loop here | add new file |
| crates/z00z_runtime/aggregators/bin/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/aggregators/examples/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/aggregators/benches/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/aggregators/tests/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |

## 3) `z00z_runtime` `validators`

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_runtime/validators/Cargo.toml | crate root | keep | no |
| crates/z00z_runtime/validators/README.md | crate docs | keep, update to checkpoint/claim/reconcile roles | no |
| crates/z00z_runtime/validators/src/lib.rs | `z00z_runtime::validators` | keep as facade; keep current public re-exports stable in this wave | no |
| crates/z00z_runtime/validators/src/artifact_decode.rs | `z00z_runtime::validators::artifact` | artifact decode/check helpers live here | `artifact.rs` |
| crates/z00z_runtime/validators/src/checkpoint_flow.rs | `z00z_runtime::validators::checkpoint` | checkpoint pipeline logic here | `checkpoint.rs` |
| crates/z00z_runtime/validators/src/claim_nulls.rs | `z00z_runtime::validators::nullifier` | claim-null checks here | `nullifier.rs` |
| crates/z00z_runtime/validators/src/claim_pkg_verify.rs | `z00z_runtime::validators::claim_verify` | claim package verification here | `claim_verify.rs` |
| crates/z00z_runtime/validators/src/reconcile_rules.rs | `z00z_runtime::validators::reconcile` | reconcile checks here | `reconcile.rs` |
| crates/z00z_runtime/validators/src/spend_rules.rs | `z00z_runtime::validators::spend` | spend rules here | `spend.rs` |
| crates/z00z_runtime/validators/src/tx_pkg_verify.rs | `z00z_runtime::validators::tx_verify` | tx package verification here | `tx_verify.rs` |
| crates/z00z_runtime/validators/src/val_engine.rs | `z00z_runtime::validators::engine` | rename file only; keep `ValidatorService` / `ValidatorBoundary` | `engine.rs` |
| crates/z00z_runtime/validators/src/verdicts.rs | `z00z_runtime::validators::verdict` | verdict/reject-class types here | `verdict.rs` |
| crates/z00z_runtime/validators/bin/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/validators/examples/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/validators/benches/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/validators/tests/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |

## 4) `z00z_runtime` `watchers`

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_runtime/watchers/Cargo.toml | crate root | keep | no |
| crates/z00z_runtime/watchers/README.md | crate docs | keep, update for evidence/export role | no |
| crates/z00z_runtime/watchers/src/lib.rs | `z00z_runtime::watchers` | keep as facade; keep current public re-exports stable in this wave | no |
| crates/z00z_runtime/watchers/src/alerts.rs | `z00z_runtime::watchers::alerts` | keep | no |
| crates/z00z_runtime/watchers/src/censorship_watch.rs | `z00z_runtime::watchers::censorship` | keep / maybe shorten | `censorship.rs` |
| crates/z00z_runtime/watchers/src/da_health.rs | `z00z_runtime::watchers::da_health` | keep | `da.rs` only if you want shorter naming |
| crates/z00z_runtime/watchers/src/evidence_export.rs | `z00z_runtime::watchers::evidence_export` | keep | no |
| crates/z00z_runtime/watchers/src/provider_compare.rs | `z00z_runtime::watchers::provider` | provider compare logic here | `provider.rs` or `compare.rs` |
| crates/z00z_runtime/watchers/src/publication_watch.rs | `z00z_runtime::watchers::publication` | keep | no |
| crates/z00z_runtime/watchers/src/status_view.rs | `z00z_runtime::watchers::status` | status projection here | `status.rs` |
| crates/z00z_runtime/watchers/src/watcher_engine.rs | `z00z_runtime::watchers::engine` | rename file only; keep `WatcherService` / `WatcherBoundary` / `WatcherInput` | `engine.rs` |
| crates/z00z_runtime/watchers/bin/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/watchers/examples/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/watchers/benches/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |
| crates/z00z_runtime/watchers/tests/empty_file | none | delete placeholder or replace with `.gitkeep` | `.gitkeep` |

## 5) `z00z_storage` root and universal backend

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/Cargo.toml | crate root | keep, add feature flags for `redb` and `memory`; `rocksdb` add only when a real adapter exists | no |
| crates/z00z_storage/README.md | crate docs | update after backend split | no |
| crates/z00z_storage/src/lib.rs | `z00z_storage` | add `backend` module re-export; keep facade exports | no |
| crates/z00z_storage/src/error.rs | `z00z_storage::error` / `backend::error` | move backend-specific error out of root error file, but keep `StoreBackendError` as the live backend-local symbol in phase 1; decide any rename only after seam stabilization and downstream/test/doc migration | no immediate rename |
| crates/z00z_storage/src/backend/mod.rs | **new** | create low-level backend seam here: `StorageBackend`, `JournalBackend`, `ReadTxn`, `WriteTxn`, and backend-local error types; keep this **below** the existing semantic facade `SettlementTreeBackend`, and keep `snapshot` above this layer | add new file |
| crates/z00z_storage/src/backend/error.rs | **new** | move backend error definitions here | add new file |
| crates/z00z_storage/src/backend/{codec,query,roots,rows,types}.rs | `z00z_storage::backend::*` | shared backend-agnostic store helpers as flat sibling modules | keep flat |
| crates/z00z_storage/src/backend/redb/* | **new** | move redb adapter here | add new folder |
| crates/z00z_storage/src/backend/rocksdb/* | **new, optional later** | add only in phase 2+ when there is a real backend implementation and tests | add new folder |
| crates/z00z_storage/src/backend/memory/* | **new** | in-memory/test adapter here; safe to add in phase 1 | add new folder |

## 6) `z00z_storage/src/checkpoint`

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/checkpoint/mod.rs | `z00z_storage::checkpoint` | keep facade | no |
| crates/z00z_storage/src/checkpoint/artifact.rs | `z00z_storage::checkpoint::artifact` | keep | `artifact_api.rs` if you want clearer naming |
| crates/z00z_storage/src/checkpoint/audit.rs | `z00z_storage::checkpoint::audit` | keep | no |
| crates/z00z_storage/src/checkpoint/build.rs | `z00z_storage::checkpoint::build` | keep, but later split builder vs state if needed | `builder.rs` |
| crates/z00z_storage/src/checkpoint/codec.rs | `z00z_storage::checkpoint::codec` | keep | no |
| crates/z00z_storage/src/checkpoint/exec_input.rs | `z00z_storage::checkpoint::exec_input` | keep | no |
| crates/z00z_storage/src/checkpoint/ids.rs | `z00z_storage::checkpoint::ids` | keep | no |
| crates/z00z_storage/src/checkpoint/link.rs | `z00z_storage::checkpoint::link` | keep | no |
| crates/z00z_storage/src/checkpoint/store.rs | `z00z_storage::checkpoint::store` | this is public checkpoint store facade | `checkpoint_store.rs` |
| crates/z00z_storage/src/checkpoint/{artifact.rs,artifact_*.rs} | `z00z_storage::checkpoint::artifact` | flat checkpoint artifact seam with prefixed siblings | keep current names |
| crates/z00z_storage/src/checkpoint/{build.rs,build_prepare.rs,build_state.rs} | `z00z_storage::checkpoint::build` | flat checkpoint build seam with explicit helper siblings | keep current names |
| crates/z00z_storage/src/checkpoint/store/* | `z00z_storage::checkpoint::store` | keep nested split | `store_fs.rs -> fs.rs`, `tests.rs -> tests.rs` |
| crates/z00z_storage/src/checkpoint/test_artifact_suite.rs | `z00z_storage::checkpoint` tests | keep | no |
| crates/z00z_storage/src/checkpoint/test_store_suite.rs | `z00z_storage::checkpoint` tests | keep | no |
| crates/z00z_storage/src/checkpoint/store/store_fs.rs | `z00z_storage::checkpoint::store` | keep as fs backend for checkpoint store | `fs_store.rs` if you want explicitness |

## 7) `z00z_storage/src/snapshot`  — это и есть backup/restore API surface

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/snapshot/mod.rs | `z00z_storage::snapshot` | keep as backup/restore facade above backend seam; do not force a separate `SnapshotBackend` in phase 1 | no |
| crates/z00z_storage/src/snapshot/codec.rs | `z00z_storage::snapshot::codec` | keep | no |
| crates/z00z_storage/src/snapshot/error.rs | `z00z_storage::snapshot::error` | keep | no |
| crates/z00z_storage/src/snapshot/store.rs | `z00z_storage::snapshot::store` | keep as snapshot persistence facade | `snapshot_store.rs` if you want explicitness |
| crates/z00z_storage/src/snapshot/types.rs | `z00z_storage::snapshot::types` | keep | no |
| crates/z00z_storage/src/snapshot/store/tests.rs | `z00z_storage::snapshot::store` tests | keep | no |

## 8) `z00z_storage/src/serialization`

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/serialization/mod.rs | `z00z_storage::serialization` | keep facade | no |
| crates/z00z_storage/src/serialization/artifact.rs | `z00z_storage::serialization::artifact` | keep | no |
| crates/z00z_storage/src/serialization/build.rs | `z00z_storage::serialization::build` | keep | no |
| crates/z00z_storage/src/serialization/build/temp_tree.rs | `z00z_storage::serialization::build` | canonical temp-tree helper | temp_tree.rs or keep |
| crates/z00z_storage/src/serialization/build_temp_tree.rs | `z00z_storage::serialization::build` | duplicate/alias; remove one side and keep only one canonical location | delete this one or rename to `build_tree.rs` |
| crates/z00z_storage/src/serialization/codec.rs | `z00z_storage::serialization::codec` | keep | no |
| crates/z00z_storage/src/serialization/restore.rs | `z00z_storage::serialization::restore` | keep | no |
| crates/z00z_storage/src/serialization/store.rs | `z00z_storage::serialization::store` | keep | `jmt_store.rs` if you want domain clarity |
| crates/z00z_storage/src/serialization/view.rs | `z00z_storage::serialization::view` | keep rendering helpers here | `render.rs` if you want better intent |

## 9) `z00z_storage/src/settlement` root

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/settlement/mod.rs | `z00z_storage::settlement` | keep facade; add backend-agnostic exports only | no |
| crates/z00z_storage/src/settlement/README.MD | docs | keep, but normalize filename casing | README.md |
| crates/z00z_storage/src/settlement/root-types.md | docs | keep or move to docs if it becomes long-lived design text | `root_types.md` |
| crates/z00z_storage/src/settlement/fee_envelope.rs | `z00z_storage::settlement::fee_envelope` | keep | no |
| crates/z00z_storage/src/settlement/keys.rs | `z00z_storage::settlement::keys` | keep | no |
| crates/z00z_storage/src/settlement/leaf.rs | `z00z_storage::settlement::leaf` | keep | no |
| crates/z00z_storage/src/settlement/model.rs | `z00z_storage::settlement::model` | keep | no |
| crates/z00z_storage/src/settlement/proof.rs | `z00z_storage::settlement::proof` | keep | no |
| crates/z00z_storage/src/settlement/timing.rs | `z00z_storage::settlement::timing` | keep | no |
| crates/z00z_storage/src/settlement/tree_id.rs | `z00z_storage::settlement::tree_id` | keep | no |
| crates/z00z_storage/src/settlement/test_live_recovery.rs | tests-only | move to `tests/` if you want cleaner src; otherwise keep | `test_live_recovery.rs` |
| crates/z00z_storage/src/settlement/test_model.rs | tests-only | move to `tests/` if you want cleaner src; otherwise keep | `test_model.rs` is okay |
| crates/z00z_storage/src/settlement/types_identity.rs | `z00z_storage::settlement::identity` | move identity types here | `identity.rs` |
| crates/z00z_storage/src/settlement/types_query.rs | `z00z_storage::settlement::query` | move query types here | `query.rs` |
| crates/z00z_storage/src/settlement/types_record.rs | `z00z_storage::settlement::record` | move record types here | `record.rs` |
| crates/z00z_storage/src/settlement/store.rs | `z00z_storage::settlement::settlement_store` | split into backend-agnostic semantic facade; remove direct redb dependency, but keep settlement semantics and proof ownership here, and keep `SettlementTreeBackend` as the live semantic contract during the backend extraction wave | `settlement_store.rs` (file rename only) |
| crates/z00z_storage/src/settlement/tx_plan/* | split between `z00z_runtime::aggregators::batch_planner` and `z00z_storage::settlement::*` helpers | move only planner-authority logic to runtime; keep `StoreSnap`, `NextState`, duplicate-path precheck, and store-local dry-run helpers in storage if they remain semantic/store-scoped | `tx_plan` should be split, not blindly exported wholesale |

## 10) `z00z_storage/src/settlement/hjmt`

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/settlement/hjmt/hjmt_cache.rs | `z00z_storage::settlement::hjmt::cache` | keep | no |
| crates/z00z_storage/src/settlement/hjmt/hjmt_commit.rs | `z00z_storage::settlement::hjmt::commit` | keep | no |
| crates/z00z_storage/src/settlement/hjmt/hjmt_config.rs | `z00z_storage::settlement::hjmt::config` | keep | config.rs (drop prefix) |
| crates/z00z_storage/src/settlement/hjmt/hjmt_journal.rs | `z00z_storage::settlement::hjmt::journal` | keep | no |
| crates/z00z_storage/src/settlement/hjmt/hjmt_plan.rs | `z00z_storage::settlement::hjmt::plan` | keep as HJMT internal plan helpers, not runtime planner authority | `plan.rs` |
| crates/z00z_storage/src/settlement/hjmt/hjmt_policy.rs | `z00z_storage::settlement::hjmt::policy` | keep | no |
| crates/z00z_storage/src/settlement/hjmt/hjmt_proof.rs | `z00z_storage::settlement::hjmt::proof` | keep | no |
| crates/z00z_storage/src/settlement/hjmt/hjmt_scheduler.rs | `z00z_storage::settlement::hjmt::scheduler` | keep | `scheduler.rs` |
| crates/z00z_storage/src/settlement/hjmt/hjmt_store.rs | `z00z_storage::settlement::hjmt::store` | keep | store.rs |
| crates/z00z_storage/src/settlement/whitebox/ | tests/internal future space | empty directory, keep only if you need it | remove or keep empty |

## 11) `z00z_storage/src/settlement/redb_backend`  → move into universal backend

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/settlement/redb_backend/mod.rs | `z00z_storage::backend::redb` | move all redb-specific adapter wiring here | `backend/redb/mod.rs` |
| crates/z00z_storage/src/settlement/redb_backend/redb_backend_helpers.rs | `z00z_storage::backend::redb::helpers` | move helper functions here | `helpers.rs` |
| crates/z00z_storage/src/settlement/redb_backend/redb_backend_hjmt.rs | `z00z_storage::backend::redb::hjmt` | move HJMT persistence bridge here | `hjmt.rs` |
| crates/z00z_storage/src/settlement/redb_backend/redb_backend_state.rs | `z00z_storage::backend::redb::state` | move durable state model here | `state.rs` |
| crates/z00z_storage/src/settlement/redb_backend/redb_backend_validate.rs | `z00z_storage::backend::redb::validate` | move backend validation here | `validate.rs` |

## 12) `z00z_storage/src/settlement/store/*` helpers

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/settlement/store/store_codec.rs | `z00z_storage::backend::codec` | move backend-neutral key/value encoding helpers here | codec.rs |
| crates/z00z_storage/src/settlement/store/store_mem.rs | `z00z_storage::backend::memory` | move in-memory tree backend here | `memory.rs` |
| crates/z00z_storage/src/settlement/store/store_query.rs | `z00z_storage::backend::query` | move query helpers here | `query.rs` |
| crates/z00z_storage/src/settlement/store/store_roots.rs | `z00z_storage::backend::roots` | move root bookkeeping helpers here | `roots.rs` |
| crates/z00z_storage/src/settlement/store/store_rows.rs | `z00z_storage::backend::rows` | move row-materialization helpers here | `rows.rs` |
| crates/z00z_storage/src/settlement/store/store_types.rs | `z00z_storage::backend::types` | move shared backend/store types here | types.rs |

## 13) `z00z_storage/src/checkpoint` tests and support files

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/checkpoint/test_checkpoint.rs | `z00z_storage::checkpoint::artifact` tests | keep | no |
| crates/z00z_storage/src/checkpoint/build_prepare.rs | `z00z_storage::checkpoint::build` | keep | `prepare.rs` |
| crates/z00z_storage/src/checkpoint/build_state.rs | `z00z_storage::checkpoint::build` | keep | `state.rs` |
| crates/z00z_storage/src/checkpoint/store/store_fs.rs | `z00z_storage::checkpoint::store` | keep | `fs.rs` |
| crates/z00z_storage/src/checkpoint/store/tests.rs | `z00z_storage::checkpoint::store` tests | keep | no |

## 14) `z00z_storage/src/snapshot` tests

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/snapshot/store/tests.rs | `z00z_storage::snapshot::store` tests | keep | no |

## 15) `z00z_storage/src/serialization` nested build helper

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/src/serialization/build/temp_tree.rs | `z00z_storage::serialization::build` | keep one canonical temp-tree helper here | temp_tree.rs |
| crates/z00z_storage/src/serialization/build_temp_tree.rs | `z00z_storage::serialization::build` | duplicate alias, remove or fold into nested module | delete or rename away |

## 16) `z00z_storage` benches, fuzz, scripts, tests

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/benches/adaptive_policy_bench.rs | benches | keep | no |
| crates/z00z_storage/benches/settlement_hjmt.rs | benches | keep | no |
| crates/z00z_storage/benches/settlement_nested.rs | benches | keep | no |
| crates/z00z_storage/benches/settlement_proofs.rs | benches | keep | no |
| crates/z00z_storage/benches/settlement_shard.rs | benches | keep | no |
| crates/z00z_storage/benches/assets/settlement_benches.md | benches docs | keep | no |
| crates/z00z_storage/benches/common/fixture.rs | benches common | keep | no |
| crates/z00z_storage/benches/common/output.rs | benches common | keep | no |
| crates/z00z_storage/benches/common/phase053.rs | benches common | keep | `phase_053.rs` if you want snake_case consistency |
| crates/z00z_storage/benches/common/timing.rs | benches common | keep | no |
| crates/z00z_storage/fuzz/Cargo.toml | fuzz | keep | no |
| crates/z00z_storage/fuzz/Cargo.lock | fuzz | keep | no |
| crates/z00z_storage/fuzz/fuzz_targets/phase053_settlement_proofs.rs | fuzz target | keep | no |
| crates/z00z_storage/fuzz/seeds/phase053_settlement/00_settlement_leaf.seed, 01_settlement_path.seed, 02_fee_envelope.seed, 03_proof_envelope.seed, 04_occupancy.seed, 05_policy_transition.seed, 06_split.seed, 07_merge.seed, README.md | fuzz seeds | keep | no |
| crates/z00z_storage/scripts/run_storage_settlement_bench.py | scripts | keep | no |
| crates/z00z_storage/scripts/run_storage_settlement_nested_bench.sh | scripts | keep | no |
| crates/z00z_storage/scripts/run_storage_settlement_shard_bench.sh | scripts | keep | no |

## 17) `z00z_storage/tests`

### Root tests

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/tests/test_async_scheduler.rs, test_bench_lanes.rs, test_cache_recompute.rs, test_checkpoint_codec.rs, test_checkpoint_draft_build.rs, test_checkpoint_draft_final.rs, test_checkpoint_finalization.rs, test_checkpoint_ids.rs, test_checkpoint_leaf_hash.rs, test_checkpoint_link_injective.rs, test_checkpoint_replay_inputs.rs, test_checkpoint_root_binding.rs, test_checkpoint_store.rs, test_claim_source_proof.rs, test_default_gate.rs, test_downstream_guardrails.rs, test_fee_envelope.rs, test_fee_replay.rs, test_forest_cache.rs, test_fuzz_seeds.rs, test_genesis_ingestion.rs, test_golden_corpus.rs, test_hjmt_adaptive_policy_proofs.rs, test_hjmt_live_proof_families.rs, test_hjmt_proofs.rs, test_layout_guardrails.rs, test_live_guardrails.rs, test_metrics.rs, test_object_reject_codes.rs, test_occupancy_evidence.rs, test_occupancy_privacy.rs, test_prep_snapshot.rs, test_property_corpus.rs, test_readme_examples.rs, test_redb_reload.rs, test_right_leaf.rs, test_serialization_determinism.rs, test_serialization_restore.rs, test_serialization_roundtrip.rs, test_serialization_visualization.rs, test_settlement_leaf.rs, test_settlement_proof_boundaries.rs, test_settlement_root.rs, test_snapshot_ids.rs, test_snapshot_leaf_hash.rs, test_snapshot_ordering.rs, test_snapshot_path_bind.rs, test_snapshot_persist.rs, test_snapshot_replay_bound.rs, test_snapshot_root_bind.rs, test_snapshot_versions.rs, test_snapshot_wit_decode.rs, test_store_api.rs | `z00z_storage` integration tests | keep, split later by surface if you want smaller suites | `test_*` names are already fine |
| crates/z00z_storage/tests/test_checkpoint_finalization.rs, test_checkpoint_leaf_hash.rs, test_checkpoint_ids.rs, test_checkpoint_root_binding.rs, test_checkpoint_store.rs, test_checkpoint_codec.rs, test_checkpoint_draft_build.rs, test_checkpoint_draft_final.rs, test_checkpoint_link_injective.rs, test_checkpoint_replay_inputs.rs, test_checkpoint_store.rs | `z00z_storage::checkpoint` integration tests | keep | no |
| crates/z00z_storage/tests/test_snapshot_ids.rs, test_snapshot_leaf_hash.rs, test_snapshot_ordering.rs, test_snapshot_path_bind.rs, test_snapshot_persist.rs, test_snapshot_replay_bound.rs, test_snapshot_root_bind.rs, test_snapshot_versions.rs, test_snapshot_wit_decode.rs, test_serialization_restore.rs, test_serialization_roundtrip.rs, test_serialization_visualization.rs, test_serialization_determinism.rs | `z00z_storage::snapshot` / `serialization` integration tests | keep flat | no |
| crates/z00z_storage/tests/test_layout_guardrails.rs, test_downstream_guardrails.rs, test_live_guardrails.rs, test_default_gate.rs | `z00z_storage` policy/guardrail tests | keep | no |

### Flat Test Helpers

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/tests/test_checkpoint_fixtures.inc | `z00z_storage::checkpoint` shared fixtures | keep at tests root | no |
| crates/z00z_storage/tests/test_snapshot_mod.inc, test_snapshot_fix.inc, test_snapshot_persist.inc, test_snapshot_replay_bound.inc, test_snapshot_path_bind.inc, test_snapshot_versions.inc, test_snapshot_leaf_hash.inc, test_snapshot_wit_decode.inc, test_snapshot_ids.inc, test_snapshot_ordering.inc, test_snapshot_root_bind.inc | `z00z_storage::snapshot` shared fixtures | keep at tests root | no |
| crates/z00z_storage/tests/test_settlement_corpus_support.inc | shared settlement corpus support | keep at tests root | no |
| crates/z00z_storage/tests/fixtures/test_settlement_corpus_fixture.json | fixtures | keep | no |

### Proptest regressions

| Current path | Target module | Move / what to do | Better rename |
| --- | --- | --- | --- |
| crates/z00z_storage/proptest-regressions/settlement/whitebox/whitebox_state.txt | whitebox regression data | keep | no |

## 🔢 Жёсткий execution order на 14 шагов

1. Зафиксировать текущий compatibility gate: storage tests, proof benches, и особенно `crates/z00z_storage/benches/settlement_proofs.rs` как guard на публичный proof surface.
2. Ввести `crates/z00z_storage/src/backend/mod.rs` и `crates/z00z_storage/src/backend/error.rs` с low-level backend contracts под существующим semantic facade `SettlementTreeBackend`, без изменения внешнего API `SettlementStore`.
3. Вынести `StoreBackendError` из `crates/z00z_storage/src/error.rs` в `backend/error.rs`, не меняя пока его live symbol name и не трогая high-level settlement error surface больше необходимого.
4. Перенести redb-specific модули из `crates/z00z_storage/src/settlement/redb_backend/*` в `crates/z00z_storage/src/backend/redb/*`.
5. Перенести shared backend helpers из `crates/z00z_storage/src/settlement/store/*` в flat sibling modules under `crates/z00z_storage/src/backend/` и `backend/memory/*`, не меняя пока semantics `SettlementStore`.
6. Переключить `SettlementStore` на новый low-level backend seam, убрав прямые поля/конструкторы `RedbBackend`, но сохранив semantic ownership store-level операций, proof methods и live trait contract `SettlementTreeBackend`.
7. После стабилизации storage seam прогнать tests и benches ещё раз; только после этого двигаться к planner split.
8. Создать `crates/z00z_runtime/aggregators/src/batch_planner.rs` и вынести туда planner-authority logic: canonicalization, route mapping, single-shard admission, canonical op digest, `BatchPlanned` inputs.
9. Оставить в storage те части `tx_plan`, которые являются store-side precheck/dry-run/rollback machinery, если они опираются на `SettlementModel` и внутренний semantic state.
10. Создать `crates/z00z_runtime/aggregators/src/placement.rs` и `shard_exec.rs` для runtime placement objects: `AggregatorId`, `ShardPlacementTable`, `ShardExecutor`.
11. Убедиться, что watchers и validators читают новые runtime surfaces без превращения в planner authority и без протаскивания placement metadata в verifier-visible truth.
12. Локально почистить только явные structural drifts: убрать дубликат `serialization/build_temp_tree.rs` против `serialization/build/temp_tree.rs`, если оба реально дублируют один helper.
13. Только после semantic stabilization запустить rename wave: `agg_*`, `val_*`, `types_identity/types_query/types_record`, `README.MD`, `root-types.md`.
14. Последним шагом обновить README, architecture docs и migration tables так, чтобы итоговая документация уже описывала landed topology, а не смешивала target-state с in-flight state.

## 🚫 Что нельзя смешивать в одной волне

- Нельзя смешивать backend seam extraction и rename wave.
- Нельзя одновременно делать file/module rename и public symbol rename для runtime facades (`AggregatorService`, `ValidatorService`, `WatcherService`) и storage semantic facade (`SettlementTreeBackend`).
- Нельзя одновременно менять planner authority location и watcher/validator naming cleanup.
- Нельзя одновременно перепривязывать `SettlementStore` к новому backend и ломать его публичный proof surface.
- Нельзя вводить `rocksdb` stub в ту же фазу, где только появляется backend seam: это создаёт пустую поверхность без проверяемого поведения.
- Нельзя выносить весь `tx_plan` из storage одним move-only PR: сначала надо отделить runtime planner authority от store-local semantic helpers.

## ✅ Рекомендуемый стиль правки этого документа

Для этого файла предпочтительны именно **точечные правки**, а не полный rewrite с нуля.

Причины:

- path inventory уже собран и полезен как migration map;
- полный rewrite повышает риск потерять реальные пути или ослабить привязку к текущему репозиторию;
- точечная правка позволяет скорректировать только нормативные ошибки: `SnapshotBackend`, слишком грубый move `tx_plan/*`, преждевременный `rocksdb`, и отсутствие жёсткого execution order.

Если позднее структура реально приземлится и таблицы устареют по live paths, тогда уже будет оправдан второй проход с полной переписью документа по фактически landed topology.
