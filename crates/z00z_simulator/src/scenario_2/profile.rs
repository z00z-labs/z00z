use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Serialize;
use z00z_storage::settlement::{
    CacheLayerMetrics, ForestCacheMetrics, ForestSchedulerMetrics, SettlementStore,
};
use z00z_utils::{
    io::{
        path_exists_no_follow, read_dir_bounded, read_file_bounded, save_json, symlink_metadata,
        write_file,
    },
    time::Instant,
};

use super::{
    config::Scenario2Cfg,
    runner::Scenario2Err,
    types::{BlockOutcome, Scenario2Summary},
};

const PROC_FILE_CAP: u64 = 64 * 1024;
const CPUINFO_CAP: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
struct SystemFacts {
    os: String,
    architecture: String,
    kernel_release: Option<String>,
    cpu_model: Option<String>,
    logical_cpus: usize,
    host_memory_bytes: Option<u64>,
    release_build: bool,
}

impl SystemFacts {
    fn capture() -> Result<Self, Scenario2Err> {
        let logical_cpus = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        #[cfg(target_os = "linux")]
        {
            let cpuinfo = read_proc_cap("/proc/cpuinfo", CPUINFO_CAP)?;
            let meminfo = read_proc("/proc/meminfo")?;
            let kernel = read_proc("/proc/sys/kernel/osrelease")?;
            let cpu_model = cpuinfo.lines().find_map(|line| {
                line.strip_prefix("model name")
                    .and_then(|value| value.split_once(':'))
                    .map(|(_, value)| value.trim().to_string())
            });
            let host_memory_bytes = status_kib(&meminfo, "MemTotal:").ok();
            Ok(Self {
                os: std::env::consts::OS.to_string(),
                architecture: std::env::consts::ARCH.to_string(),
                kernel_release: Some(kernel.trim().to_string()),
                cpu_model,
                logical_cpus,
                host_memory_bytes,
                release_build: !cfg!(debug_assertions),
            })
        }
        #[cfg(not(target_os = "linux"))]
        Ok(Self {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            kernel_release: None,
            cpu_model: None,
            logical_cpus,
            host_memory_bytes: None,
            release_build: !cfg!(debug_assertions),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
struct ProcSample {
    user_ticks: u64,
    system_ticks: u64,
    rss_bytes: u64,
    peak_rss_bytes: u64,
    threads: u64,
    voluntary_switches: u64,
    involuntary_switches: u64,
    read_chars: u64,
    write_chars: u64,
    read_syscalls: u64,
    write_syscalls: u64,
    read_bytes: u64,
    write_bytes: u64,
}

impl ProcSample {
    #[cfg(target_os = "linux")]
    fn capture() -> Result<Self, Scenario2Err> {
        let stat = read_proc("/proc/self/stat")?;
        let status = read_proc("/proc/self/status")?;
        let io = read_proc("/proc/self/io")?;
        let close = stat.rfind(')').ok_or_else(|| {
            Scenario2Err::Profile("/proc/self/stat has no process-name terminator".to_string())
        })?;
        let fields = stat
            .get(close + 1..)
            .ok_or_else(|| Scenario2Err::Profile("invalid /proc/self/stat".to_string()))?
            .split_whitespace()
            .collect::<Vec<_>>();
        if fields.len() <= 12 {
            return Err(Scenario2Err::Profile(
                "/proc/self/stat is missing CPU fields".to_string(),
            ));
        }
        Ok(Self {
            user_ticks: parse_value(fields[11], "stat.utime")?,
            system_ticks: parse_value(fields[12], "stat.stime")?,
            rss_bytes: status_kib(&status, "VmRSS:")?,
            peak_rss_bytes: status_kib(&status, "VmHWM:")?,
            threads: status_value(&status, "Threads:")?,
            voluntary_switches: status_value(&status, "voluntary_ctxt_switches:")?,
            involuntary_switches: status_value(&status, "nonvoluntary_ctxt_switches:")?,
            read_chars: status_value(&io, "rchar:")?,
            write_chars: status_value(&io, "wchar:")?,
            read_syscalls: status_value(&io, "syscr:")?,
            write_syscalls: status_value(&io, "syscw:")?,
            read_bytes: status_value(&io, "read_bytes:")?,
            write_bytes: status_value(&io, "write_bytes:")?,
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn capture() -> Result<Self, Scenario2Err> {
        Ok(Self::default())
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
struct CacheTotals {
    hits: u64,
    misses: u64,
    inserts: u64,
    evictions: u64,
    invalidations: u64,
    entries: u64,
}

impl CacheTotals {
    fn from_metrics(metrics: &ForestCacheMetrics) -> Self {
        let layers = [
            &metrics.subtree_root,
            &metrics.parent_leaf,
            &metrics.terminal_leaf,
            &metrics.bucket_derivation,
            &metrics.proof_segment,
            &metrics.nonexistence,
            &metrics.policy_proof,
            &metrics.journal_digest,
            &metrics.path_index,
        ];
        layers.into_iter().fold(Self::default(), |mut sum, layer| {
            sum.add_layer(layer);
            sum
        })
    }

    fn add_layer(&mut self, layer: &CacheLayerMetrics) {
        self.hits = self.hits.saturating_add(layer.hits);
        self.misses = self.misses.saturating_add(layer.misses);
        self.inserts = self.inserts.saturating_add(layer.inserts);
        self.evictions = self.evictions.saturating_add(layer.evictions);
        self.invalidations = self.invalidations.saturating_add(layer.invalidations);
        self.entries = self
            .entries
            .saturating_add(u64::try_from(layer.entries).unwrap_or(u64::MAX));
    }

    fn delta(self, earlier: Self) -> Self {
        Self {
            hits: self.hits.saturating_sub(earlier.hits),
            misses: self.misses.saturating_sub(earlier.misses),
            inserts: self.inserts.saturating_sub(earlier.inserts),
            evictions: self.evictions.saturating_sub(earlier.evictions),
            invalidations: self.invalidations.saturating_sub(earlier.invalidations),
            entries: self.entries,
        }
    }
}

pub(super) struct StageProbe {
    started: Instant,
    process: ProcSample,
    cache: CacheTotals,
    scheduler: ForestSchedulerMetrics,
}

impl StageProbe {
    pub fn start(store: &SettlementStore) -> Result<Self, Scenario2Err> {
        Ok(Self {
            started: Instant::now(),
            process: ProcSample::capture()?,
            cache: CacheTotals::from_metrics(&store.forest_cache_metrics()),
            scheduler: store.forest_scheduler_metrics(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finish(
        self,
        cycle: u32,
        height: u64,
        stage: &str,
        items: u64,
        logical_bytes: u64,
        store: &SettlementStore,
    ) -> Result<StageRecord, Scenario2Err> {
        let elapsed_ns = u64::try_from(self.started.elapsed().as_nanos())
            .map_err(|_| Scenario2Err::Profile("stage duration overflow".to_string()))?;
        let process = ProcSample::capture()?;
        let cache = CacheTotals::from_metrics(&store.forest_cache_metrics()).delta(self.cache);
        let scheduler = store.forest_scheduler_metrics();
        Ok(StageRecord {
            cycle,
            height,
            stage: stage.to_string(),
            items,
            logical_bytes,
            elapsed_ns,
            user_cpu_ticks: process.user_ticks.saturating_sub(self.process.user_ticks),
            system_cpu_ticks: process
                .system_ticks
                .saturating_sub(self.process.system_ticks),
            rss_bytes: process.rss_bytes,
            peak_rss_bytes: process.peak_rss_bytes,
            threads: process.threads,
            voluntary_context_switches: process
                .voluntary_switches
                .saturating_sub(self.process.voluntary_switches),
            involuntary_context_switches: process
                .involuntary_switches
                .saturating_sub(self.process.involuntary_switches),
            read_chars: process.read_chars.saturating_sub(self.process.read_chars),
            write_chars: process.write_chars.saturating_sub(self.process.write_chars),
            read_syscalls: process
                .read_syscalls
                .saturating_sub(self.process.read_syscalls),
            write_syscalls: process
                .write_syscalls
                .saturating_sub(self.process.write_syscalls),
            read_bytes: process.read_bytes.saturating_sub(self.process.read_bytes),
            write_bytes: process.write_bytes.saturating_sub(self.process.write_bytes),
            cache_hits: cache.hits,
            cache_misses: cache.misses,
            cache_inserts: cache.inserts,
            cache_evictions: cache.evictions,
            cache_invalidations: cache.invalidations,
            cache_entries: cache.entries,
            scheduler_max_active: scheduler.max_active,
            scheduler_rejects: scheduler
                .reject_count
                .saturating_sub(self.scheduler.reject_count),
            scheduler_cancels: scheduler
                .cancel_count
                .saturating_sub(self.scheduler.cancel_count),
            scheduler_max_queued: scheduler.max_queued,
            scheduler_last_batch: scheduler.last_batch,
            scheduler_last_wait_us: scheduler.last_blocking_wait_us,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct StageRecord {
    cycle: u32,
    height: u64,
    stage: String,
    items: u64,
    logical_bytes: u64,
    elapsed_ns: u64,
    user_cpu_ticks: u64,
    system_cpu_ticks: u64,
    rss_bytes: u64,
    peak_rss_bytes: u64,
    threads: u64,
    voluntary_context_switches: u64,
    involuntary_context_switches: u64,
    read_chars: u64,
    write_chars: u64,
    read_syscalls: u64,
    write_syscalls: u64,
    read_bytes: u64,
    write_bytes: u64,
    cache_hits: u64,
    cache_misses: u64,
    cache_inserts: u64,
    cache_evictions: u64,
    cache_invalidations: u64,
    cache_entries: u64,
    scheduler_max_active: usize,
    scheduler_rejects: u64,
    scheduler_cancels: u64,
    scheduler_max_queued: usize,
    scheduler_last_batch: usize,
    scheduler_last_wait_us: u64,
}

#[derive(Debug, Serialize)]
struct CycleProfile<'a> {
    cycle: u32,
    disk_bytes: u64,
    disk_entries: usize,
    stages: &'a [StageRecord],
    blocks: &'a [BlockOutcome],
}

#[derive(Clone, Debug, Serialize)]
struct StageStats {
    samples: usize,
    items: u64,
    logical_bytes: u64,
    total_seconds: f64,
    mean_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    max_ms: f64,
    items_per_second: f64,
    user_cpu_ticks: u64,
    system_cpu_ticks: u64,
}

#[derive(Debug, Serialize)]
struct ProfileSummary {
    system: SystemFacts,
    elapsed_seconds: f64,
    blocks: u64,
    transactions: u64,
    transactions_per_second: f64,
    peak_rss_bytes: u64,
    max_threads: u64,
    process_read_chars: u64,
    process_write_chars: u64,
    process_read_syscalls: u64,
    process_write_syscalls: u64,
    process_read_bytes: u64,
    process_write_bytes: u64,
    final_disk_bytes: u64,
    final_disk_entries: usize,
    disk_breakdown: DiskBreakdown,
    hjmt_cache_hit_ratio: f64,
    hjmt_scheduler_max_active: usize,
    hjmt_scheduler_max_queued: usize,
    stages: BTreeMap<String, StageStats>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct DiskSlice {
    bytes: u64,
    entries: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct DiskBreakdown {
    live_storage: DiskSlice,
    projection_storage: DiskSlice,
    da_archive: DiskSlice,
    checkpoints: DiskSlice,
    nova: DiskSlice,
    plonky3: DiskSlice,
    profiles: DiskSlice,
    other_bytes: u64,
}

#[derive(Debug, Serialize)]
struct AggregatorRequirements {
    basis: &'static str,
    headroom_percent: u32,
    measurement_cpu_model: Option<String>,
    measurement_logical_cpus: usize,
    measurement_host_memory_bytes: Option<u64>,
    measured_peak_rss_bytes: u64,
    recommended_ram_bytes: u64,
    measured_run_disk_bytes: u64,
    measured_aggregator_disk_bytes: u64,
    simulation_projection_disk_bytes: u64,
    recommended_disk_bytes: u64,
    configured_worker_threads: usize,
    observed_process_threads: u64,
    observed_hjmt_parallelism: usize,
    authority_native_evaluator_bytes: u64,
    authority_hot_recovery_cap_bytes: u64,
    prover_material_bytes: u64,
    verifier_bundle_bytes: u64,
    plonky3_cadence_blocks: u64,
    completed_plonky3_epochs: u32,
    max_inflight_plonky3_chunk_proofs: usize,
    block_p95_ms: f64,
    block_p99_ms: f64,
    sustained_transactions_per_second: f64,
    note: &'static str,
}

pub(super) struct Profiler {
    started: Instant,
    process_start: ProcSample,
    system: SystemFacts,
    worker_threads: usize,
    prover_material_bytes: u64,
    verifier_bundle_bytes: u64,
    stage_ns: BTreeMap<String, Vec<u64>>,
    stage_items: BTreeMap<String, u64>,
    stage_bytes: BTreeMap<String, u64>,
    stage_user_ticks: BTreeMap<String, u64>,
    stage_system_ticks: BTreeMap<String, u64>,
    cache_hits: u64,
    cache_misses: u64,
    cycle_records: Vec<StageRecord>,
    cycle_blocks: Vec<BlockOutcome>,
    peak_rss_bytes: u64,
    max_threads: u64,
    scheduler_max_active: usize,
    scheduler_max_queued: usize,
}

impl Profiler {
    pub fn new(worker_threads: usize) -> Result<Self, Scenario2Err> {
        let process_start = ProcSample::capture()?;
        Ok(Self {
            started: Instant::now(),
            process_start,
            system: SystemFacts::capture()?,
            worker_threads,
            prover_material_bytes: 0,
            verifier_bundle_bytes: 0,
            stage_ns: BTreeMap::new(),
            stage_items: BTreeMap::new(),
            stage_bytes: BTreeMap::new(),
            stage_user_ticks: BTreeMap::new(),
            stage_system_ticks: BTreeMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            cycle_records: Vec::new(),
            cycle_blocks: Vec::new(),
            peak_rss_bytes: process_start.peak_rss_bytes,
            max_threads: process_start.threads,
            scheduler_max_active: 0,
            scheduler_max_queued: 0,
        })
    }

    pub fn set_material_sizes(&mut self, prover_material_bytes: u64, verifier_bundle_bytes: u64) {
        self.prover_material_bytes = prover_material_bytes;
        self.verifier_bundle_bytes = verifier_bundle_bytes;
    }

    pub fn observe_stage(&mut self, record: StageRecord) {
        if record.stage != "block_total" {
            self.cache_hits = self.cache_hits.saturating_add(record.cache_hits);
            self.cache_misses = self.cache_misses.saturating_add(record.cache_misses);
        }
        self.stage_ns
            .entry(record.stage.clone())
            .or_default()
            .push(record.elapsed_ns);
        add_metric(&mut self.stage_items, &record.stage, record.items);
        add_metric(&mut self.stage_bytes, &record.stage, record.logical_bytes);
        add_metric(
            &mut self.stage_user_ticks,
            &record.stage,
            record.user_cpu_ticks,
        );
        add_metric(
            &mut self.stage_system_ticks,
            &record.stage,
            record.system_cpu_ticks,
        );
        self.peak_rss_bytes = self.peak_rss_bytes.max(record.peak_rss_bytes);
        self.max_threads = self.max_threads.max(record.threads);
        self.scheduler_max_active = self.scheduler_max_active.max(record.scheduler_max_active);
        self.scheduler_max_queued = self.scheduler_max_queued.max(record.scheduler_max_queued);
        self.cycle_records.push(record);
    }

    pub fn observe_block(&mut self, outcome: BlockOutcome) {
        self.cycle_blocks.push(outcome);
    }

    pub fn finish_cycle(
        &mut self,
        cycle: u32,
        run_dir: &Path,
        entry_cap: usize,
        save_blocks: bool,
    ) -> Result<(), Scenario2Err> {
        let (disk_bytes, disk_entries) = directory_usage(run_dir, entry_cap)?;
        let empty = Vec::new();
        let blocks = if save_blocks {
            self.cycle_blocks.as_slice()
        } else {
            empty.as_slice()
        };
        let profile = CycleProfile {
            cycle,
            disk_bytes,
            disk_entries,
            stages: &self.cycle_records,
            blocks,
        };
        save_json(
            run_dir
                .join("profile")
                .join(format!("cycle-{cycle:02}.json")),
            &profile,
        )?;
        self.cycle_records.clear();
        self.cycle_blocks.clear();
        Ok(())
    }

    pub fn finalize(
        &self,
        config: &Scenario2Cfg,
        run_dir: &Path,
        run_summary: &Scenario2Summary,
    ) -> Result<(), Scenario2Err> {
        let elapsed_seconds = self.started.elapsed().as_secs_f64();
        let process = ProcSample::capture()?;
        let (disk_breakdown, disk_bytes, disk_entries) =
            disk_breakdown(run_dir, config.profiling.directory_scan_entry_cap)?;
        let stages = self.stage_stats();
        let transactions_per_second = rate(run_summary.transactions, elapsed_seconds);
        let hit_ratio = rate(
            self.cache_hits,
            self.cache_hits.saturating_add(self.cache_misses) as f64,
        );
        let summary = ProfileSummary {
            system: self.system.clone(),
            elapsed_seconds,
            blocks: run_summary.blocks,
            transactions: run_summary.transactions,
            transactions_per_second,
            peak_rss_bytes: self.peak_rss_bytes.max(process.peak_rss_bytes),
            max_threads: self.max_threads.max(process.threads),
            process_read_chars: process
                .read_chars
                .saturating_sub(self.process_start.read_chars),
            process_write_chars: process
                .write_chars
                .saturating_sub(self.process_start.write_chars),
            process_read_syscalls: process
                .read_syscalls
                .saturating_sub(self.process_start.read_syscalls),
            process_write_syscalls: process
                .write_syscalls
                .saturating_sub(self.process_start.write_syscalls),
            process_read_bytes: process
                .read_bytes
                .saturating_sub(self.process_start.read_bytes),
            process_write_bytes: process
                .write_bytes
                .saturating_sub(self.process_start.write_bytes),
            final_disk_bytes: disk_bytes,
            final_disk_entries: disk_entries,
            disk_breakdown: disk_breakdown.clone(),
            hjmt_cache_hit_ratio: hit_ratio,
            hjmt_scheduler_max_active: self.scheduler_max_active,
            hjmt_scheduler_max_queued: self.scheduler_max_queued,
            stages: stages.clone(),
        };
        save_json(run_dir.join("profile/profile_summary.json"), &summary)?;

        let recursive_profile =
            z00z_storage::checkpoint::recursive_v2::RecursiveCircuitProfileV2::authority_pinned();
        let cadence =
            z00z_storage::checkpoint::recursive_v2::NovaCompressionPolicyV2::authority_pinned()
                .map_err(|error| Scenario2Err::Nova(error.to_string()))?;
        let block = stages.get("block_total");
        let aggregator_disk_bytes = disk_breakdown
            .live_storage
            .bytes
            .saturating_add(disk_breakdown.da_archive.bytes)
            .saturating_add(disk_breakdown.checkpoints.bytes)
            .saturating_add(disk_breakdown.nova.bytes)
            .saturating_add(disk_breakdown.plonky3.bytes);
        let requirements = AggregatorRequirements {
            basis: "observed end-to-end scenario_2 high-water marks plus configured headroom",
            headroom_percent: config.profiling.requirement_headroom_percent,
            measurement_cpu_model: self.system.cpu_model.clone(),
            measurement_logical_cpus: self.system.logical_cpus,
            measurement_host_memory_bytes: self.system.host_memory_bytes,
            measured_peak_rss_bytes: summary.peak_rss_bytes,
            recommended_ram_bytes: with_headroom(
                summary.peak_rss_bytes,
                config.profiling.requirement_headroom_percent,
            )?,
            measured_run_disk_bytes: disk_bytes,
            measured_aggregator_disk_bytes: aggregator_disk_bytes,
            simulation_projection_disk_bytes: disk_breakdown.projection_storage.bytes,
            recommended_disk_bytes: with_headroom(
                aggregator_disk_bytes,
                config.profiling.requirement_headroom_percent,
            )?,
            configured_worker_threads: self.worker_threads,
            observed_process_threads: summary.max_threads,
            observed_hjmt_parallelism: summary.hjmt_scheduler_max_active,
            authority_native_evaluator_bytes: recursive_profile
                .native_evaluator_resident_bytes()
                .map_err(|error| Scenario2Err::Nova(error.to_string()))?,
            authority_hot_recovery_cap_bytes: cadence.manifest().max_hot_recovery_bytes(),
            prover_material_bytes: self.prover_material_bytes,
            verifier_bundle_bytes: self.verifier_bundle_bytes,
            plonky3_cadence_blocks: config.plonky3.cadence_blocks,
            completed_plonky3_epochs: run_summary.completed_plonky3_epochs,
            max_inflight_plonky3_chunk_proofs: config.plonky3.max_inflight_chunk_proofs,
            block_p95_ms: block.map_or(0.0, |value| value.p95_ms),
            block_p99_ms: block.map_or(0.0, |value| value.p99_ms),
            sustained_transactions_per_second: transactions_per_second,
            note: "RAM includes sequential full Plonky3 chunk, epoch-seal, and history proving high-water marks and is conservative because the correctness projection shares the process; disk excludes the projection and profile evidence. Results are valid only for the recorded hardware, config, build, and completed workload.",
        };
        save_json(
            run_dir.join("minimum_aggregator_requirements.json"),
            &requirements,
        )?;
        write_file(
            run_dir.join("optimization_candidates.md"),
            optimization_report(&stages, hit_ratio).as_bytes(),
        )?;
        Ok(())
    }

    fn stage_stats(&self) -> BTreeMap<String, StageStats> {
        self.stage_ns
            .iter()
            .map(|(stage, samples)| {
                let mut sorted = samples.clone();
                sorted.sort_unstable();
                let total_ns = sorted.iter().copied().fold(0_u64, u64::saturating_add);
                let total_seconds = total_ns as f64 / 1_000_000_000.0;
                let items = self.stage_items.get(stage).copied().unwrap_or(0);
                let stats = StageStats {
                    samples: sorted.len(),
                    items,
                    logical_bytes: self.stage_bytes.get(stage).copied().unwrap_or(0),
                    total_seconds,
                    mean_ms: ns_to_ms(total_ns / u64::try_from(sorted.len()).unwrap_or(1)),
                    p50_ms: percentile_ms(&sorted, 50),
                    p95_ms: percentile_ms(&sorted, 95),
                    p99_ms: percentile_ms(&sorted, 99),
                    max_ms: sorted.last().copied().map_or(0.0, ns_to_ms),
                    items_per_second: rate(items, total_seconds),
                    user_cpu_ticks: self.stage_user_ticks.get(stage).copied().unwrap_or(0),
                    system_cpu_ticks: self.stage_system_ticks.get(stage).copied().unwrap_or(0),
                };
                (stage.clone(), stats)
            })
            .collect()
    }
}

fn add_metric(metrics: &mut BTreeMap<String, u64>, stage: &str, value: u64) {
    let entry = metrics.entry(stage.to_string()).or_default();
    *entry = entry.saturating_add(value);
}

fn percentile_ms(values: &[u64], percentile: usize) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let rank = values.len().saturating_mul(percentile).saturating_add(99) / 100;
    let index = rank.saturating_sub(1);
    ns_to_ms(values[index.min(values.len() - 1)])
}

fn ns_to_ms(value: u64) -> f64 {
    value as f64 / 1_000_000.0
}

fn rate(items: u64, seconds: f64) -> f64 {
    if seconds > 0.0 {
        items as f64 / seconds
    } else {
        0.0
    }
}

fn with_headroom(value: u64, percent: u32) -> Result<u64, Scenario2Err> {
    value
        .checked_mul(u64::from(100_u32.saturating_add(percent)))
        .and_then(|scaled| scaled.checked_add(99))
        .map(|scaled| scaled / 100)
        .ok_or_else(|| Scenario2Err::Profile("capacity headroom overflow".to_string()))
}

fn optimization_report(stages: &BTreeMap<String, StageStats>, cache_ratio: f64) -> String {
    let core_total = stages
        .iter()
        .filter(|(name, _)| name.as_str() != "block_total")
        .map(|(_, value)| value.total_seconds)
        .sum::<f64>();
    let mut ranked = stages
        .iter()
        .filter(|(name, _)| name.as_str() != "block_total")
        .map(|(name, value)| {
            let share = if core_total > 0.0 {
                value.total_seconds * 100.0 / core_total
            } else {
                0.0
            };
            (share, name, value)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut report = String::from(
        "# Scenario 2 optimization candidates\n\n\
         These are measurement-led candidates, not protocol changes. Re-run the full correctness gates after every optimization.\n\n\
         ## Measured stage ranking\n\n",
    );
    for (share, name, value) in ranked.iter().take(10) {
        report.push_str(&format!(
            "- `{name}`: {share:.2}% of non-overlapping measured stage time; p95 {:.3} ms.\n",
            value.p95_ms
        ));
    }
    report.push_str(&format!(
        "\nObserved aggregate HJMT cache hit ratio: {:.2}%.\n\n",
        cache_ratio * 100.0
    ));
    report.push_str(
        "## Safe investigation order\n\n\
         1. Transaction construction: reuse immutable wallet/card material, tune the bounded Rayon pool, and benchmark batched proof verification without weakening per-package gates.\n\
         2. HJMT proof and projection: tune existing bounded proof batching and cache sizes; preserve deterministic ordering, backpressure, and cache verification.\n\
         3. Checkpoint persistence: replace repeated predecessor-directory scans with a bounded durable index and profile hashing/serialization copies.\n\
         4. DA persistence: benchmark larger buffered frames and an explicit durability policy; never acknowledge a block before the configured atomic persistence boundary.\n\
         5. Nova: split trace, native evaluation, fold, and compression timers; retain the existing verifier cache and keep the single accumulator sequential. Optimize only measured serialization or circuit hotspots.\n\
         6. Plonky3: compare direct-AIR chunk proving, frontier merge, epoch seal, history recursion, and artifact reload separately. Keep one heavyweight chunk proof in flight until measured RAM headroom proves that bounded parallelism is safe; reuse only authority-bound verifier/common-data caches.\n",
    );
    report
}

fn disk_breakdown(
    root: &Path,
    entry_cap: usize,
) -> Result<(DiskBreakdown, u64, usize), Scenario2Err> {
    let live_storage = optional_usage(&root.join("storage/live"), entry_cap)?;
    let projection_storage = optional_usage(&root.join("storage/preview"), entry_cap)?;
    let da_archive = optional_usage(&root.join("da"), entry_cap)?;
    let checkpoints = optional_usage(&root.join("checkpoint"), entry_cap)?;
    let nova = optional_usage(&root.join("nova"), entry_cap)?;
    let plonky3 = optional_usage(&root.join("plonky3"), entry_cap)?;
    let profiles = optional_usage(&root.join("profile"), entry_cap)?;
    let component_bytes = [
        live_storage.bytes,
        projection_storage.bytes,
        da_archive.bytes,
        checkpoints.bytes,
        nova.bytes,
        plonky3.bytes,
        profiles.bytes,
    ]
    .into_iter()
    .fold(0_u64, u64::saturating_add);
    let (total_bytes, total_entries) = directory_usage(root, entry_cap)?;
    Ok((
        DiskBreakdown {
            live_storage,
            projection_storage,
            da_archive,
            checkpoints,
            nova,
            plonky3,
            profiles,
            other_bytes: total_bytes.saturating_sub(component_bytes),
        },
        total_bytes,
        total_entries,
    ))
}

fn optional_usage(path: &Path, entry_cap: usize) -> Result<DiskSlice, Scenario2Err> {
    if !path_exists_no_follow(path)? {
        return Ok(DiskSlice::default());
    }
    let (bytes, entries) = directory_usage(path, entry_cap)?;
    Ok(DiskSlice { bytes, entries })
}

pub(super) fn directory_usage(root: &Path, entry_cap: usize) -> Result<(u64, usize), Scenario2Err> {
    let mut stack = vec![PathBuf::from(root)];
    let mut bytes = 0_u64;
    let mut entries = 0_usize;
    while let Some(directory) = stack.pop() {
        let remaining = entry_cap.checked_sub(entries).ok_or_else(|| {
            Scenario2Err::Profile("directory scan entry cap exceeded".to_string())
        })?;
        if remaining == 0 {
            return Err(Scenario2Err::Profile(
                "directory scan entry cap exceeded".to_string(),
            ));
        }
        for entry in read_dir_bounded(&directory, remaining)? {
            entries = entries.checked_add(1).ok_or_else(|| {
                Scenario2Err::Profile("directory entry count overflow".to_string())
            })?;
            let metadata = symlink_metadata(&entry)?;
            if metadata.file_type().is_symlink() {
                bytes = bytes.saturating_add(metadata.len());
            } else if metadata.is_dir() {
                stack.push(entry);
            } else if metadata.is_file() {
                bytes = bytes.checked_add(metadata.len()).ok_or_else(|| {
                    Scenario2Err::Profile("directory byte count overflow".to_string())
                })?;
            }
        }
    }
    Ok((bytes, entries))
}

#[cfg(target_os = "linux")]
fn read_proc(path: &str) -> Result<String, Scenario2Err> {
    read_proc_cap(path, PROC_FILE_CAP)
}

#[cfg(target_os = "linux")]
fn read_proc_cap(path: &str, cap: u64) -> Result<String, Scenario2Err> {
    let bytes = read_file_bounded(path, cap)?;
    String::from_utf8(bytes)
        .map_err(|_| Scenario2Err::Profile(format!("{path} is not valid UTF-8")))
}

#[cfg(target_os = "linux")]
fn status_value(input: &str, key: &str) -> Result<u64, Scenario2Err> {
    let value = input
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .and_then(|value| value.split_whitespace().next())
        .ok_or_else(|| Scenario2Err::Profile(format!("missing {key} in procfs")))?;
    parse_value(value, key)
}

#[cfg(target_os = "linux")]
fn status_kib(input: &str, key: &str) -> Result<u64, Scenario2Err> {
    status_value(input, key)?
        .checked_mul(1024)
        .ok_or_else(|| Scenario2Err::Profile(format!("{key} byte conversion overflow")))
}

#[cfg(target_os = "linux")]
fn parse_value(value: &str, field: &str) -> Result<u64, Scenario2Err> {
    value
        .parse::<u64>()
        .map_err(|_| Scenario2Err::Profile(format!("invalid procfs value for {field}")))
}
