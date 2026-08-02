use std::path::{Path, PathBuf};

use thiserror::Error;
use z00z_crypto::{sha256_256_role, CheckpointShaRole};
use z00z_storage::{
    checkpoint::{
        recursive_v2::{
            NovaCadenceRequestV2, NovaCompressionAuthorityV2, NovaCompressionPolicyV2,
            RecursiveAuthoritySnapshotV2, RecursiveCheckpointChainBlockV2,
            RecursiveCheckpointEvidenceStoreV2, RecursiveCircuitProfileV2,
            RecursiveEvidenceCancellationV2, RecursiveEvidenceOutcomeV2,
            RecursiveEvidenceRequestV2, RecursiveFinalizedIvcStateV2,
            SettlementRootGenerationCutoverV2,
        },
        CheckpointFsStore,
    },
    fixture_support::genesis_chain_identity::ensure_test_process_chain_identity,
    settlement::{SettlementStateRoot, SettlementStore, StoreOp},
    snapshot::PrepFsStore,
};
use z00z_utils::{
    io::{create_dir_all, path_exists_no_follow, read_file_bounded, save_json, save_yaml, IoError},
    logger::{Logger, StdoutLogger},
    time::{SystemTimeProvider, TimeProvider},
};

use super::{
    checkpoint::{checkpoint_root, project_block, seal_checkpoint, verify_reopened},
    config::{Scenario2Cfg, DEFAULT_CONFIG_PATH},
    da::{persist_block, prepare_block, verify_block},
    plonky3::{Plonky3EpochOutcome, Plonky3Pipeline},
    profile::{Profiler, StageProbe},
    tx_batch::{build_block, order_block, PreparedBlock},
    types::{BlockOutcome, OwnedCoin, Scenario2Summary},
    wallets::{coin_item, WalletRing},
};

const RECURSIVE_LAYOUT: u32 = 7;
const CUTOVER_HEIGHT: u64 = 1;
const CUTOVER_INSTALL_GEN: u64 = 1;

struct NovaResult {
    successor: RecursiveFinalizedIvcStateV2,
    action: &'static str,
    verifier_attempts: Option<u64>,
    recovery_bytes: Option<u64>,
}

#[derive(Debug, Error)]
pub enum Scenario2Err {
    #[error("I/O: {0}")]
    Io(#[from] IoError),
    #[error("configuration: {0}")]
    Config(String),
    #[error("wallet: {0}")]
    Wallet(String),
    #[error("aggregator: {0}")]
    Aggregator(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("checkpoint: {0}")]
    Checkpoint(String),
    #[error("Nova: {0}")]
    Nova(String),
    #[error("Plonky3: {0}")]
    Plonky3(String),
    #[error("DA: {0}")]
    Da(String),
    #[error("profiling: {0}")]
    Profile(String),
    #[error("invariant: {0}")]
    Invariant(String),
    #[error("runtime: {0}")]
    Runtime(String),
}

pub fn run() -> Result<Scenario2Summary, Scenario2Err> {
    run_with_path(DEFAULT_CONFIG_PATH)
}

pub fn run_with_path(path: impl AsRef<Path>) -> Result<Scenario2Summary, Scenario2Err> {
    let config = Scenario2Cfg::load(path)?;
    execute(config)
}

fn execute(config: Scenario2Cfg) -> Result<Scenario2Summary, Scenario2Err> {
    if cfg!(debug_assertions) {
        return Err(Scenario2Err::Runtime(
            "scenario_2 is a release-only load workload; use --release".to_string(),
        ));
    }
    let worker_threads = config.worker_threads();
    configure_hjmt_workers(worker_threads)?;
    let run_dir = create_run_dir(&config)?;
    save_yaml(run_dir.join("scenario_config.resolved.yaml"), &config)?;

    let live_root = run_dir.join("storage/live");
    let preview_root = run_dir.join("storage/preview");
    let mut live = SettlementStore::load(&live_root)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let mut preview = SettlementStore::load(&preview_root)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(worker_threads)
        .thread_name(|index| format!("scenario-2-{index}"))
        .build()
        .map_err(|error| Scenario2Err::Runtime(error.to_string()))?;
    let mut profiler = Profiler::new(worker_threads)?;

    let probe = StageProbe::start(&live)?;
    let identity = ensure_test_process_chain_identity()
        .map_err(|error| Scenario2Err::Runtime(error.to_string()))?;
    if identity.chain_id() != config.runtime.chain_id {
        return Err(Scenario2Err::Config(format!(
            "configured chain_id {} does not match validated devnet identity {}",
            config.runtime.chain_id,
            identity.chain_id()
        )));
    }
    observe(&mut profiler, probe, 0, 0, "chain_identity", 1, 0, &live)?;

    let probe = StageProbe::start(&live)?;
    let (prover_material, verifier_bundle) = load_nova_materials(&config)?;
    let prover_bytes = bytes_len(&prover_material, "prover material")?;
    let verifier_bytes = bytes_len(&verifier_bundle, "verifier bundle")?;
    profiler.set_material_sizes(prover_bytes, verifier_bytes);
    observe(
        &mut profiler,
        probe,
        0,
        0,
        "nova_material_load",
        2,
        prover_bytes.saturating_add(verifier_bytes),
        &live,
    )?;

    let probe = StageProbe::start(&live)?;
    let wallets = WalletRing::new(&config)?;
    let coins = wallets.seed_coins(&config)?;
    observe(
        &mut profiler,
        probe,
        0,
        0,
        "wallet_seed_build",
        u64::from(config.load.transactions_per_block),
        0,
        &live,
    )?;

    let probe = StageProbe::start(&live)?;
    live.apply_settlement_ops(seed_ops(&coins)?)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    preview
        .apply_settlement_ops(seed_ops(&coins)?)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    require_same_roots(&live, &preview)?;
    observe(
        &mut profiler,
        probe,
        0,
        0,
        "wallet_seed_hjmt",
        u64::from(config.load.transactions_per_block),
        0,
        &live,
    )?;

    let probe = StageProbe::start(&live)?;
    let genesis_root = live
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let mut plonky3 = Plonky3Pipeline::new(&run_dir, &config, genesis_root)?;
    observe(
        &mut profiler,
        probe,
        0,
        0,
        "plonky3_authority_init",
        1,
        0,
        &live,
    )?;

    let probe = StageProbe::start(&live)?;
    let mut prior = install_cutover(&mut live)?;
    observe(&mut profiler, probe, 0, 0, "recursive_cutover", 1, 0, &live)?;

    let checkpoint_root = checkpoint_root(&run_dir);
    let mut checkpoint_store = CheckpointFsStore::new(&checkpoint_root);
    let mut prep_store = PrepFsStore::new(&checkpoint_root);
    let evidence_root = run_dir.join("nova/evidence");
    let mut evidence_store = RecursiveCheckpointEvidenceStoreV2::open(&evidence_root)
        .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
    let cancellation = RecursiveEvidenceCancellationV2::new();
    let recursive_profile = RecursiveCircuitProfileV2::authority_pinned();
    let cadence = NovaCompressionPolicyV2::authority_pinned()
        .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
    profiler.finish_cycle(
        0,
        &run_dir,
        config.profiling.directory_scan_entry_cap,
        config.profiling.save_block_records,
    )?;

    let mut coins = coins;
    let mut final_checkpoint = None;
    let mut final_plonky3: Option<Plonky3EpochOutcome> = None;
    let mut completed_blocks = 0_u64;
    let mut completed_txs = 0_u64;
    for cycle in 1..=config.load.cycles {
        let cycle_start_height = u64::from(cycle - 1)
            .checked_mul(u64::from(config.load.blocks_per_cycle))
            .ok_or_else(|| Scenario2Err::Invariant("cycle start height overflow".to_string()))?;
        let probe = StageProbe::start(&live)?;
        let mut plonky3_cycle = plonky3.begin_cycle(cycle, &live)?;
        observe(
            &mut profiler,
            probe,
            cycle,
            cycle_start_height,
            "plonky3_epoch_frontier_open",
            1,
            0,
            &live,
        )?;
        for offset in 0..config.load.blocks_per_cycle {
            let height = block_height(&config, cycle, offset)?;
            let block_probe = StageProbe::start(&live)?;
            let pre_root = live
                .settlement_root_v2(RECURSIVE_LAYOUT)
                .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
            if preview
                .settlement_root_v2(RECURSIVE_LAYOUT)
                .map_err(|error| Scenario2Err::Storage(error.to_string()))?
                != pre_root
            {
                return Err(Scenario2Err::Invariant(
                    "preview HJMT pre-root drift".to_string(),
                ));
            }
            let input_paths = coins.iter().map(|coin| coin.path).collect::<Vec<_>>();

            let probe = StageProbe::start(&live)?;
            let proofs = live
                .settlement_proof_blobs(&input_paths)
                .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
            let spend_root = live
                .check_root()
                .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "hjmt_membership",
                u64::from(config.load.transactions_per_block),
                0,
                &live,
            )?;

            let probe = StageProbe::start(&live)?;
            let built = build_block(
                &pool, &config, &wallets, height, &coins, &proofs, spend_root,
            )?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "transaction_build_verify",
                u64::from(config.load.transactions_per_block),
                0,
                &live,
            )?;

            let probe = StageProbe::start(&live)?;
            let prepared = order_block(height, built)?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "aggregator_batch",
                u64::from(config.load.transactions_per_block),
                0,
                &live,
            )?;
            let PreparedBlock {
                ordered,
                handoff,
                next_coins,
                spent_paths,
                output_paths,
            } = prepared;

            let probe = StageProbe::start(&preview)?;
            let projection =
                project_block(&mut preview, &handoff, &output_paths, RECURSIVE_LAYOUT)?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "hjmt_projection",
                u64::try_from(output_paths.len()).map_err(|_| {
                    Scenario2Err::Invariant("output count conversion failed".to_string())
                })?,
                0,
                &preview,
            )?;

            let probe = StageProbe::start(&live)?;
            let prepared_da = prepare_block(
                cycle,
                height,
                &ordered,
                pre_root,
                projection.post_root,
                &config.da,
            )?;
            let da_packages = prepared_da.package_count();
            let da_payload_bytes = prepared_da.payload_bytes();
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "celestia_blob_eds_encode",
                u64::from(da_packages),
                da_payload_bytes,
                &live,
            )?;

            let probe = StageProbe::start(&live)?;
            let da_artifact_bytes = prepared_da.artifact_bytes();
            let da = persist_block(&run_dir, prepared_da)?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "celestia_eds_persist",
                u64::from(da.package_count),
                da_artifact_bytes,
                &live,
            )?;

            let probe = StageProbe::start(&live)?;
            verify_block(&da, &config.da)?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "celestia_reload_verify",
                u64::from(da.package_count),
                da.artifact_bytes,
                &live,
            )?;
            drop(ordered);

            let probe = StageProbe::start(&live)?;
            let sealed = seal_checkpoint(
                height,
                pre_root,
                &projection,
                prior.digest(),
                &handoff,
                &spent_paths,
                &da,
                &mut checkpoint_store,
                &mut prep_store,
            )?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "checkpoint_seal_reload",
                1,
                da.payload_bytes,
                &live,
            )?;

            let (request, expected_action) = recursive_request(&cadence, height)?;
            let transition_dir = run_dir
                .join("nova/transitions")
                .join(format!("cycle-{cycle:02}"))
                .join(format!("block-{height:05}"));
            create_dir_all(&transition_dir)?;
            let probe = StageProbe::start(&live)?;
            let (outcome, epoch_capture) = {
                let mut blocks = [RecursiveCheckpointChainBlockV2::new(
                    transition_dir,
                    recursive_profile,
                    &checkpoint_store,
                    &prep_store,
                    sealed.checkpoint_id,
                    handoff,
                )];
                evidence_store
                    .produce_with_epoch_capture(
                        &mut blocks,
                        &mut live,
                        &prover_material,
                        &verifier_bundle,
                        &cancellation,
                        request,
                        plonky3_cycle.stream(),
                    )
                    .map_err(|error| Scenario2Err::Nova(error.to_string()))?
            };
            if epoch_capture.transition_count() != 1 {
                return Err(Scenario2Err::Invariant(
                    "one scenario block did not produce one Plonky3 transition".to_string(),
                ));
            }
            let nova = unpack_outcome(outcome, expected_action)?;
            prior = nova.successor;
            let nova_stage = match nova.action {
                "fold" => "nova_fold_plonky3_transition_prepare",
                "recovery" => "nova_recovery_snapshot_plonky3_transition_prepare",
                "snapshot" => "nova_compression_publication_plonky3_transition_prepare",
                _ => {
                    return Err(Scenario2Err::Invariant(
                        "unknown Nova outcome class".to_string(),
                    ));
                }
            };
            observe(&mut profiler, probe, cycle, height, nova_stage, 1, 0, &live)?;

            let probe = StageProbe::start(&live)?;
            let epoch_work = epoch_capture
                .append_to_epoch(plonky3_cycle.stream_mut())
                .map_err(|error| Scenario2Err::Plonky3(error.to_string()))?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "plonky3_transition_stream_commit",
                1,
                0,
                &live,
            )?;
            if epoch_work.len() > 1 {
                return Err(Scenario2Err::Invariant(
                    "one block emitted more than one Plonky3 trace chunk".to_string(),
                ));
            }
            let mut plonky3_chunk = None;
            let mut plonky3_merge = None;
            for work in epoch_work {
                let probe = StageProbe::start(&live)?;
                let chunk = plonky3_cycle.prove_and_admit(work)?;
                observe(
                    &mut profiler,
                    probe,
                    cycle,
                    height,
                    "plonky3_chunk_prove_verify_admit",
                    chunk.binding_count,
                    chunk.proof_bytes,
                    &live,
                )?;
                let probe = StageProbe::start(&live)?;
                let merge = plonky3_cycle.merge_ready()?;
                observe(
                    &mut profiler,
                    probe,
                    cycle,
                    height,
                    "plonky3_frontier_merge",
                    u64::try_from(merge.merged).map_err(|_| {
                        Scenario2Err::Invariant("Plonky3 merge count conversion failed".to_string())
                    })?,
                    0,
                    &live,
                )?;
                plonky3_chunk = Some(chunk);
                plonky3_merge = Some(merge);
            }

            let probe = StageProbe::start(&live)?;
            verify_post_state(
                &config,
                height,
                &live,
                &preview,
                projection.post_root,
                &prior,
                &next_coins,
                da.package_count,
            )?;
            observe(
                &mut profiler,
                probe,
                cycle,
                height,
                "state_verify",
                u64::from(config.load.transactions_per_block),
                0,
                &live,
            )?;

            if height % u64::from(config.storage.checkpoint_reload_every_blocks) == 0 {
                let probe = StageProbe::start(&live)?;
                verify_reopened(&checkpoint_root, &sealed)?;
                observe(
                    &mut profiler,
                    probe,
                    cycle,
                    height,
                    "checkpoint_cold_reload",
                    1,
                    0,
                    &live,
                )?;
            }
            if height % u64::from(config.storage.hjmt_reload_every_blocks) == 0 {
                let probe = StageProbe::start(&live)?;
                reload_hjmt(
                    &live_root,
                    &preview_root,
                    &projection.post_root,
                    &mut live,
                    &mut preview,
                )?;
                observe(
                    &mut profiler,
                    probe,
                    cycle,
                    height,
                    "hjmt_cold_reload",
                    1,
                    0,
                    &live,
                )?;
            }

            plonky3_cycle.record_block(height, &sealed, &da)?;

            let (sender, recipient) = wallets.edge(height);
            profiler.observe_block(BlockOutcome {
                cycle,
                height,
                tx_count: config.load.transactions_per_block,
                sender: sender.name.clone(),
                recipient: recipient.name.clone(),
                checkpoint_id_hex: hex::encode(sealed.checkpoint_id.as_bytes()),
                recursive_digest_hex: hex::encode(prior.digest()),
                settlement_root_hex: hex::encode(projection.post_root.as_bytes()),
                nova_action: nova.action.to_string(),
                nova_cumulative_steps: prior.cumulative_steps(),
                nova_verifier_attempts: nova.verifier_attempts,
                recovery_snapshot_bytes: nova.recovery_bytes,
                da_bytes: da.payload_bytes,
                plonky3_chunk_ordinal: plonky3_chunk.as_ref().map(|value| value.ordinal),
                plonky3_chunk_proof_bytes: plonky3_chunk.as_ref().map(|value| value.proof_bytes),
                plonky3_trace_rows: plonky3_chunk.as_ref().map(|value| value.trace_rows),
                plonky3_table_count: plonky3_chunk.as_ref().map(|value| value.table_count),
                plonky3_merged_parents: plonky3_merge.as_ref().map(|value| value.merged),
                plonky3_verified_chunks: plonky3_merge.as_ref().map(|value| value.verified_chunks),
                plonky3_active_ranges: plonky3_merge.as_ref().map(|value| value.active_ranges),
            });
            coins = next_coins;
            final_checkpoint = Some(sealed.checkpoint_id);
            completed_blocks = completed_blocks
                .checked_add(1)
                .ok_or_else(|| Scenario2Err::Invariant("completed block overflow".to_string()))?;
            completed_txs = completed_txs
                .checked_add(u64::from(config.load.transactions_per_block))
                .ok_or_else(|| Scenario2Err::Invariant("completed tx overflow".to_string()))?;
            observe(
                &mut profiler,
                block_probe,
                cycle,
                height,
                "block_total",
                u64::from(config.load.transactions_per_block),
                da.payload_bytes,
                &live,
            )?;

            if height.is_multiple_of(100) {
                StdoutLogger.info(&format!(
                    "scenario_2.progress cycle={cycle} height={height} blocks={completed_blocks}"
                ));
            }
        }

        let cycle_height = u64::from(cycle)
            .checked_mul(u64::from(config.load.blocks_per_cycle))
            .ok_or_else(|| Scenario2Err::Invariant("cycle height overflow".to_string()))?;
        let probe = StageProbe::start(&live)?;
        let closed_epoch = plonky3_cycle.close(prior.digest())?;
        let work_manifest_bytes = closed_epoch.logical_bytes();
        observe(
            &mut profiler,
            probe,
            cycle,
            cycle_height,
            "plonky3_epoch_close_reopen",
            u64::from(config.load.blocks_per_cycle),
            work_manifest_bytes,
            &live,
        )?;

        let probe = StageProbe::start(&live)?;
        let sealed_epoch = closed_epoch.seal_and_verify()?;
        let epoch_proof_bytes = sealed_epoch.epoch_proof_bytes();
        observe(
            &mut profiler,
            probe,
            cycle,
            cycle_height,
            "plonky3_epoch_seal_verify",
            1,
            epoch_proof_bytes,
            &live,
        )?;

        let probe = StageProbe::start(&live)?;
        let history = plonky3.prove_history(&sealed_epoch)?;
        let history_proof_bytes = u64::try_from(history.canonical_bytes().len()).map_err(|_| {
            Scenario2Err::Invariant("Plonky3 history proof byte count overflow".to_string())
        })?;
        observe(
            &mut profiler,
            probe,
            cycle,
            cycle_height,
            "plonky3_history_prove_verify",
            u64::from(cycle),
            history_proof_bytes,
            &live,
        )?;

        let probe = StageProbe::start(&live)?;
        let epoch_outcome = plonky3.publish_and_reload(sealed_epoch, history)?;
        let published_bytes = epoch_outcome
            .work_manifest_bytes
            .checked_add(epoch_outcome.epoch_proof_bytes)
            .and_then(|value| value.checked_add(epoch_outcome.history_proof_bytes))
            .and_then(|value| value.checked_add(epoch_outcome.epoch_manifest_bytes))
            .ok_or_else(|| {
                Scenario2Err::Invariant("Plonky3 published byte count overflow".to_string())
            })?;
        save_json(
            run_dir
                .join("plonky3")
                .join(format!("epoch-{:04}", epoch_outcome.epoch_index))
                .join("epoch-summary.json"),
            &epoch_outcome,
        )?;
        observe(
            &mut profiler,
            probe,
            cycle,
            cycle_height,
            "plonky3_manifest_persist_reload",
            1,
            published_bytes,
            &live,
        )?;
        if epoch_outcome.transition_count != config.load.blocks_per_cycle
            || epoch_outcome.end_height != cycle_height
        {
            return Err(Scenario2Err::Invariant(
                "completed Plonky3 epoch has unexpected cadence or height".to_string(),
            ));
        }
        final_plonky3 = Some(epoch_outcome);

        if cycle < config.load.cycles
            && cycle_height.is_multiple_of(cadence.manifest().recovery_snapshot_cadence_blocks())
        {
            let probe = StageProbe::start(&live)?;
            evidence_store = RecursiveCheckpointEvidenceStoreV2::open(&evidence_root)
                .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
            observe(
                &mut profiler,
                probe,
                cycle,
                cycle_height,
                "nova_recovery_reopen",
                1,
                0,
                &live,
            )?;
        }
        profiler.finish_cycle(
            cycle,
            &run_dir,
            config.profiling.directory_scan_entry_cap,
            config.profiling.save_block_records,
        )?;
        StdoutLogger.info(&format!(
            "scenario_2.cycle_complete cycle={cycle} blocks={completed_blocks}"
        ));
    }

    if completed_blocks != config.total_blocks()?
        || completed_txs != config.total_transactions()?
        || plonky3.completed_epochs() != config.load.cycles
    {
        return Err(Scenario2Err::Invariant(
            "completed workload cardinality drift".to_string(),
        ));
    }
    let checkpoint_id = final_checkpoint.ok_or_else(|| {
        Scenario2Err::Invariant("scenario completed without a checkpoint".to_string())
    })?;
    let final_root = live
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let final_plonky3 = final_plonky3.ok_or_else(|| {
        Scenario2Err::Invariant("scenario completed without a Plonky3 epoch".to_string())
    })?;
    let summary = Scenario2Summary {
        run_dir: run_dir.clone(),
        blocks: completed_blocks,
        transactions: completed_txs,
        final_checkpoint_id_hex: hex::encode(checkpoint_id.as_bytes()),
        final_recursive_digest_hex: hex::encode(prior.digest()),
        final_settlement_root_hex: hex::encode(final_root.as_bytes()),
        plonky3_cadence_blocks: config.plonky3.cadence_blocks,
        completed_plonky3_epochs: plonky3.completed_epochs(),
        final_plonky3_epoch_statement_digest_hex: final_plonky3.epoch_statement_digest_hex,
        final_plonky3_history_proof_digest_hex: final_plonky3.history_proof_digest_hex,
        final_plonky3_epoch_manifest_digest_hex: final_plonky3.epoch_manifest_digest_hex,
    };
    save_json(run_dir.join("run_summary.json"), &summary)?;
    profiler.finalize(&config, &run_dir, &summary)?;
    Ok(summary)
}

#[allow(clippy::too_many_arguments)]
fn observe(
    profiler: &mut Profiler,
    probe: StageProbe,
    cycle: u32,
    height: u64,
    stage: &str,
    items: u64,
    logical_bytes: u64,
    store: &SettlementStore,
) -> Result<(), Scenario2Err> {
    profiler.observe_stage(probe.finish(cycle, height, stage, items, logical_bytes, store)?);
    Ok(())
}

fn configure_hjmt_workers(worker_threads: usize) -> Result<(), Scenario2Err> {
    let queue = worker_threads
        .checked_mul(8)
        .ok_or_else(|| Scenario2Err::Config("HJMT queue bound overflow".to_string()))?;
    std::env::set_var("Z00Z_STORAGE_SCHED_CPU", worker_threads.to_string());
    std::env::set_var("Z00Z_STORAGE_SCHED_QUEUE", queue.to_string());
    Ok(())
}

fn create_run_dir(config: &Scenario2Cfg) -> Result<PathBuf, Scenario2Err> {
    let millis = SystemTimeProvider
        .try_unix_timestamp_ms()
        .map_err(|error| Scenario2Err::Runtime(error.to_string()))?;
    let run_dir = config
        .storage
        .output_root
        .join(format!("run-{millis}-{}", std::process::id()));
    if path_exists_no_follow(&run_dir)? {
        return Err(Scenario2Err::Runtime(format!(
            "run directory already exists: {}",
            run_dir.display()
        )));
    }
    create_dir_all(&run_dir)?;
    Ok(run_dir)
}

fn load_nova_materials(config: &Scenario2Cfg) -> Result<(Vec<u8>, Vec<u8>), Scenario2Err> {
    let material = read_file_bounded(
        config
            .nova
            .artifact_dir
            .join(&config.nova.prover_material_file),
        config.nova.max_prover_material_bytes,
    )?;
    let bundle = read_file_bounded(
        config
            .nova
            .artifact_dir
            .join(&config.nova.verifier_bundle_file),
        config.nova.max_verifier_bundle_bytes,
    )?;
    if material.is_empty() || bundle.is_empty() {
        return Err(Scenario2Err::Nova(
            "Nova prover material and verifier bundle must be non-empty".to_string(),
        ));
    }
    Ok((material, bundle))
}

fn seed_ops(coins: &[OwnedCoin]) -> Result<Vec<StoreOp>, Scenario2Err> {
    coins
        .iter()
        .map(coin_item)
        .map(|item| item.map(|item| StoreOp::Put(Box::new(item))))
        .collect()
}

fn install_cutover(
    live: &mut SettlementStore,
) -> Result<RecursiveFinalizedIvcStateV2, Scenario2Err> {
    let authority = RecursiveAuthoritySnapshotV2::resolve_active_authority(live)
        .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
    let settlement_root = live
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let opaque = live
        .settlement_root()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?
        .into_bytes();
    let pinned_opaque = sha256_256_role(
        CheckpointShaRole::Link,
        &[b"z00z.recursive.v2.opaque-last-root-record", &opaque],
    );
    let mut cutover = SettlementRootGenerationCutoverV2::active_authority(
        authority,
        live,
        CUTOVER_HEIGHT,
        opaque,
        pinned_opaque,
        settlement_root,
        CUTOVER_INSTALL_GEN,
    )
    .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
    cutover
        .install_active_authority(live, CUTOVER_INSTALL_GEN)
        .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
    RecursiveFinalizedIvcStateV2::from_installed_cutover(live)
        .map_err(|error| Scenario2Err::Nova(error.to_string()))
}

fn recursive_request(
    cadence: &NovaCompressionPolicyV2,
    height: u64,
) -> Result<(RecursiveEvidenceRequestV2, &'static str), Scenario2Err> {
    let action = cadence
        .action(
            height,
            NovaCompressionAuthorityV2::Scheduled,
            NovaCadenceRequestV2::Scheduled,
        )
        .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
    if action.is_compression_required || action.is_publication_required {
        Ok((
            RecursiveEvidenceRequestV2::Snapshot {
                authority: NovaCompressionAuthorityV2::Scheduled,
                cadence: NovaCadenceRequestV2::Scheduled,
            },
            "snapshot",
        ))
    } else if action.is_recovery_snapshot_required {
        Ok((
            RecursiveEvidenceRequestV2::RecoverySnapshot {
                authority: NovaCompressionAuthorityV2::Scheduled,
            },
            "recovery",
        ))
    } else {
        Ok((RecursiveEvidenceRequestV2::FoldOnly, "fold"))
    }
}

fn unpack_outcome(
    outcome: RecursiveEvidenceOutcomeV2,
    expected: &'static str,
) -> Result<NovaResult, Scenario2Err> {
    let (successor, action, verifier_attempts, recovery_bytes) = match outcome {
        RecursiveEvidenceOutcomeV2::Folded(successor) => (*successor, "fold", None, None),
        RecursiveEvidenceOutcomeV2::Snapshot(evidence) => (
            evidence.successor,
            "snapshot",
            Some(evidence.verifier_attempts),
            None,
        ),
        RecursiveEvidenceOutcomeV2::Recovery(recovery) => (
            recovery.successor,
            "recovery",
            None,
            Some(recovery.snapshot_bytes),
        ),
    };
    if action != expected {
        return Err(Scenario2Err::Invariant(format!(
            "Nova cadence expected {expected}, produced {action}"
        )));
    }
    Ok(NovaResult {
        successor,
        action,
        verifier_attempts,
        recovery_bytes,
    })
}

#[allow(clippy::too_many_arguments)]
fn verify_post_state(
    config: &Scenario2Cfg,
    height: u64,
    live: &SettlementStore,
    preview: &SettlementStore,
    expected_root: SettlementStateRoot,
    prior: &RecursiveFinalizedIvcStateV2,
    next_coins: &[OwnedCoin],
    package_count: u32,
) -> Result<(), Scenario2Err> {
    let live_root = live
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let preview_root = preview
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    if live_root != expected_root
        || preview_root != expected_root
        || prior.height() != height
        || package_count != config.load.transactions_per_block
        || next_coins.len()
            != usize::try_from(config.load.transactions_per_block)
                .map_err(|_| Scenario2Err::Invariant("tx count conversion failed".to_string()))?
    {
        return Err(Scenario2Err::Invariant(
            "post-state, recursion height, or block cardinality drift".to_string(),
        ));
    }
    live.verify_forest_cache()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    preview
        .verify_forest_cache()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    Ok(())
}

fn require_same_roots(
    live: &SettlementStore,
    preview: &SettlementStore,
) -> Result<(), Scenario2Err> {
    let live_root = live
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let preview_root = preview
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    if live_root != preview_root {
        return Err(Scenario2Err::Invariant(
            "live and preview seed roots differ".to_string(),
        ));
    }
    Ok(())
}

fn reload_hjmt(
    live_root: &Path,
    preview_root: &Path,
    expected_root: &SettlementStateRoot,
    live: &mut SettlementStore,
    preview: &mut SettlementStore,
) -> Result<(), Scenario2Err> {
    let recovery = live
        .recovery_state()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let reopened_live = SettlementStore::load(live_root)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let reopened_preview = SettlementStore::load(preview_root)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    let reopened_recovery = reopened_live
        .recovery_state()
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?;
    if reopened_live
        .settlement_root_v2(RECURSIVE_LAYOUT)
        .map_err(|error| Scenario2Err::Storage(error.to_string()))?
        != *expected_root
        || reopened_preview
            .settlement_root_v2(RECURSIVE_LAYOUT)
            .map_err(|error| Scenario2Err::Storage(error.to_string()))?
            != *expected_root
        || reopened_recovery != recovery
    {
        return Err(Scenario2Err::Invariant(
            "cold HJMT reload or recovery-state mismatch".to_string(),
        ));
    }
    *live = reopened_live;
    *preview = reopened_preview;
    Ok(())
}

fn block_height(config: &Scenario2Cfg, cycle: u32, offset: u32) -> Result<u64, Scenario2Err> {
    u64::from(cycle.saturating_sub(1))
        .checked_mul(u64::from(config.load.blocks_per_cycle))
        .and_then(|value| value.checked_add(u64::from(offset)))
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| Scenario2Err::Invariant("block height overflow".to_string()))
}

fn bytes_len(bytes: &[u8], label: &str) -> Result<u64, Scenario2Err> {
    u64::try_from(bytes.len())
        .map_err(|_| Scenario2Err::Runtime(format!("{label} length overflow")))
}
