use super::super::{AssetDefinition, ChainType, GenesisAssetAccumulator};
use super::generate_timestamp;
use crate::genesis::validator::GenesisError;
use std::path::{Path, PathBuf};
use z00z_utils::io::create_dir_all;

#[cfg(not(target_arch = "wasm32"))]
use z00z_utils::{
    io::{copy, File, Seek, Write},
    prelude::{Logger, StdoutLogger},
};

const GENESIS_OUTPUT_HASH_SCHEMA: &str = "genesis-output-root-v2";
const GENESIS_OUTPUT_KEEP_ENV: &str = "Z00Z_GENESIS_OUTPUT_KEEP";
const GENESIS_LOG_HASH_SCHEMA: &str = "genesis-log-root-v2";
const GENESIS_LOG_KEEP_ENV: &str = "Z00Z_GENESIS_LOG_KEEP";

pub(crate) fn create_timestamped_output_dir(
    base_path: &str,
    network_type: ChainType,
) -> Result<PathBuf, GenesisError> {
    prepare_managed_genesis_root(
        Path::new(base_path),
        &genesis_output_fingerprint(),
        Some(GENESIS_OUTPUT_KEEP_ENV),
    )?;
    let timestamp = generate_timestamp();
    let dir_name = format!("genesis_{}_{}", network_type.as_str(), timestamp);
    let full_path = Path::new(base_path).join(dir_name);

    create_dir_all(&full_path).map_err(|e| GenesisError::FileWriteFailed {
        path: full_path.display().to_string(),
        error: e.to_string(),
    })?;

    Ok(full_path)
}

pub(crate) fn prepare_genesis_logging_dir(path: &str) -> Result<PathBuf, GenesisError> {
    let dir = PathBuf::from(path);
    prepare_managed_genesis_root(&dir, &genesis_log_fingerprint(), Some(GENESIS_LOG_KEEP_ENV))?;
    Ok(dir)
}

pub(crate) fn prepare_genesis_snapshot_root(path: &str) -> Result<PathBuf, GenesisError> {
    let dir = PathBuf::from(path);
    prepare_managed_genesis_root(
        &dir,
        &genesis_output_fingerprint(),
        Some(GENESIS_OUTPUT_KEEP_ENV),
    )?;
    Ok(dir)
}

fn prepare_managed_genesis_root(
    path: &Path,
    fingerprint: &str,
    preserve_env: Option<&str>,
) -> Result<(), GenesisError> {
    z00z_utils::io::reset_managed_root(path, fingerprint, &[], preserve_env)
        .map(|_| ())
        .map_err(|error| GenesisError::FileWriteFailed {
            path: path.display().to_string(),
            error: error.to_string(),
        })
}

fn genesis_output_fingerprint() -> String {
    static VALUE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VALUE
        .get_or_init(|| genesis_root_fingerprint(GENESIS_OUTPUT_HASH_SCHEMA))
        .clone()
}

fn genesis_log_fingerprint() -> String {
    static VALUE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VALUE
        .get_or_init(|| genesis_root_fingerprint(GENESIS_LOG_HASH_SCHEMA))
        .clone()
}

fn genesis_root_fingerprint(schema: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    z00z_utils::io::hash_root_inputs(
        schema,
        &[
            root.join("Cargo.toml"),
            root.join("Cargo.lock"),
            root.join(".cargo/config.toml"),
            root.join("crates/z00z_core/Cargo.toml"),
            root.join("crates/z00z_crypto/Cargo.toml"),
            root.join("crates/z00z_utils/Cargo.toml"),
        ],
        &[
            root.join("crates/z00z_core/src/assets"),
            root.join("crates/z00z_core/src/genesis"),
            root.join("crates/z00z_crypto/src"),
            root.join("crates/z00z_utils/src"),
        ],
    )
    .expect("hash genesis output root")
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn create_genesis_snapshot_zip(
    snapshot_root: &Path,
    output_dir: &Path,
    network_type: ChainType,
    config_path: &str,
    cli_command: &str,
) -> Result<(), GenesisError> {
    use z00z_utils::io::{Cursor, Read};

    let logger = StdoutLogger;
    let timestamp = output_dir
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|s| s.split('_').next_back())
        .unwrap_or("unknown");
    let zip_name = format!(
        "genesis_snapshot_{}_{}.zip",
        network_type.as_str(),
        timestamp
    );
    let zip_path = snapshot_root.join(&zip_name);

    logger.info("Phase 5: Creating snapshot archive...");
    logger.info(&format!("Snapshot root: {}", snapshot_root.display()));
    logger.info(&format!("Snapshot archive: {}", zip_path.display()));

    let file = File::create(&zip_path).map_err(|e| GenesisError::FileWriteFailed {
        path: zip_path.display().to_string(),
        error: e.to_string(),
    })?;

    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    add_directory_to_zip(
        &mut zip,
        "crates/z00z_core/src/assets",
        "src/assets",
        options,
    )?;
    add_directory_to_zip(
        &mut zip,
        "crates/z00z_core/src/genesis",
        "src/genesis",
        options,
    )?;

    let config_file_name = Path::new(config_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("genesis_config.yaml");
    if let Ok(mut config_file) = File::open(config_path) {
        let mut config_content = Vec::new();
        config_file.read_to_end(&mut config_content).map_err(|e| {
            GenesisError::ConfigLoadFailed(format!("Failed to read {}: {}", config_path, e))
        })?;

        zip.start_file(config_file_name, options)
            .map_err(|e| GenesisError::FileWriteFailed {
                path: zip_name.clone(),
                error: e.to_string(),
            })?;
        copy(&mut Cursor::new(config_content), &mut zip).map_err(|e| {
            GenesisError::FileWriteFailed {
                path: zip_name.clone(),
                error: e.to_string(),
            }
        })?;
    }

    let bash_script = format!(
        "#!/bin/bash\n# Genesis generation script\n# Generated: {}\n# Network: {}\n\n{}\n",
        timestamp,
        network_type.as_str(),
        cli_command
    );
    zip.start_file("run_genesis.sh", options)
        .map_err(|e| GenesisError::FileWriteFailed {
            path: zip_name.clone(),
            error: e.to_string(),
        })?;
    copy(&mut Cursor::new(bash_script.as_bytes()), &mut zip).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: zip_name.clone(),
            error: e.to_string(),
        }
    })?;

    zip.finish().map_err(|e| GenesisError::FileWriteFailed {
        path: zip_path.display().to_string(),
        error: e.to_string(),
    })?;
    logger.info(&format!("Snapshot created: {}", zip_name));
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn add_directory_to_zip<W: Write + Seek>(
    zip: &mut zip::ZipWriter<W>,
    source_dir: &str,
    zip_prefix: &str,
    options: zip::write::FileOptions<'static, ()>,
) -> Result<(), GenesisError> {
    use z00z_utils::io::{Cursor, Read};

    for entry in walkdir::WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(source_dir)
            .map_err(|e| GenesisError::SerializationFailed(format!("Path error: {}", e)))?;
        let zip_path = Path::new(zip_prefix).join(relative_path);

        let mut file = File::open(path).map_err(|e| {
            GenesisError::ConfigLoadFailed(format!("Failed to read {}: {}", path.display(), e))
        })?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            GenesisError::ConfigLoadFailed(format!("Failed to read {}: {}", path.display(), e))
        })?;

        zip.start_file(zip_path.to_string_lossy().to_string(), options)
            .map_err(|e| GenesisError::FileWriteFailed {
                path: "zip_archive".to_string(),
                error: e.to_string(),
            })?;
        copy(&mut Cursor::new(buffer), &mut *zip).map_err(|e| GenesisError::FileWriteFailed {
            path: "zip_archive".to_string(),
            error: e.to_string(),
        })?;
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod output_tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};
    use tempfile::TempDir;
    use z00z_utils::io::{create_dir_all, read_dir, read_file, write_file};

    const SUPPORT_SRC: &str = include_str!("genesis_output_support.rs");

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().expect("genesis output env lock")
    }

    #[test]
    fn test_output_clears_stale() {
        let _guard = env_lock();
        let dir = TempDir::new().expect("temp dir");
        let base = dir.path().join("genesis");
        create_dir_all(base.join("stale")).expect("create stale dir");
        write_file(base.join("stale/marker.txt"), b"stale").expect("write stale marker");

        let out =
            create_timestamped_output_dir(base.to_str().expect("utf-8 base"), ChainType::Devnet)
                .expect("create timestamped dir");

        assert!(out.exists(), "timestamped output dir must exist");
        assert!(
            !base.join("stale").exists(),
            "stale genesis payload must be removed"
        );
    }

    #[test]
    fn test_output_keeps_filtered() {
        let _guard = env_lock();
        let dir = TempDir::new().expect("temp dir");
        let base = dir.path().join("genesis");
        create_dir_all(base.join("fixtures")).expect("create fixture dir");
        create_dir_all(base.join("stale")).expect("create stale dir");
        write_file(base.join("fixtures/keep.bin"), b"keep").expect("write fixture");
        write_file(base.join("stale/drop.txt"), b"drop").expect("write stale marker");
        std::env::set_var(GENESIS_OUTPUT_KEEP_ENV, "fixtures");

        let out =
            create_timestamped_output_dir(base.to_str().expect("utf-8 base"), ChainType::Devnet)
                .expect("create timestamped dir");

        std::env::remove_var(GENESIS_OUTPUT_KEEP_ENV);
        assert!(out.exists(), "timestamped output dir must exist");
        assert_eq!(
            read_file(base.join("fixtures/keep.bin")).expect("read kept fixture"),
            b"keep"
        );
        assert!(
            !base.join("stale").exists(),
            "non-preserved genesis payload must be removed"
        );
        assert!(
            read_dir(&base)
                .expect("read genesis base")
                .iter()
                .any(|path| path.file_name().and_then(|name| name.to_str()) == Some("fixtures")),
            "preserved fixture dir must remain under the genesis root"
        );
    }

    #[test]
    fn test_output_clears_prior() {
        let _guard = env_lock();
        let dir = TempDir::new().expect("temp dir");
        let base = dir.path().join("genesis");

        let first =
            create_timestamped_output_dir(base.to_str().expect("utf-8 base"), ChainType::Devnet)
                .expect("create first timestamped dir");
        write_file(first.join("old.txt"), b"old").expect("write stale output");

        let second =
            create_timestamped_output_dir(base.to_str().expect("utf-8 base"), ChainType::Devnet)
                .expect("create second timestamped dir");

        assert!(second.exists(), "second timestamped output dir must exist");
        assert!(
            !first.join("old.txt").exists(),
            "same-process rerun must clear prior genesis output payload"
        );
        let fresh_dirs = read_dir(&base)
            .expect("read genesis base")
            .into_iter()
            .filter(|path| path.is_dir())
            .count();
        assert_eq!(
            fresh_dirs, 1,
            "genesis base must contain only one fresh timestamped output dir"
        );
    }

    #[test]
    fn test_genesis_root_hash() {
        for needle in [
            "const GENESIS_OUTPUT_HASH_SCHEMA: &str = \"genesis-output-root-v2\";",
            "const GENESIS_LOG_HASH_SCHEMA: &str = \"genesis-log-root-v2\";",
            "z00z_utils::io::hash_root_inputs(",
            "crates/z00z_core/src/assets",
            "crates/z00z_core/src/genesis",
            "crates/z00z_crypto/src",
            "crates/z00z_utils/src",
        ] {
            assert!(
                SUPPORT_SRC.contains(needle),
                "genesis output root contract must include {needle}"
            );
        }
        for legacy in [
            "const GENESIS_OUTPUT_FINGERPRINT: &str = \"genesis-output-root-v1\";",
            "const GENESIS_LOG_FINGERPRINT: &str = \"genesis-log-root-v1\";",
        ] {
            assert!(
                !SUPPORT_SRC.contains(legacy),
                "genesis output root contract must reject legacy constant root {legacy}"
            );
        }
    }

    #[test]
    fn test_snapshot_zip_root() {
        let _guard = env_lock();
        let dir = TempDir::new().expect("temp dir");
        let snapshot_root = dir.path().join("snapshots");
        let output_dir = dir.path().join("outputs/genesis_devnet_20260628_020000");
        let config_path = dir.path().join("genesis_config.yaml");
        create_dir_all(&snapshot_root).expect("create snapshot root");
        create_dir_all(&output_dir).expect("create output dir");
        write_file(&config_path, b"version: 1\n").expect("write config fixture");

        create_genesis_snapshot_zip(
            &snapshot_root,
            &output_dir,
            ChainType::Devnet,
            config_path.to_str().expect("utf-8 config path"),
            "cargo run --release --bin genesis_cli -- --config genesis_config.yaml",
        )
        .expect("create snapshot zip");

        let zip_path = snapshot_root.join("genesis_snapshot_devnet_020000.zip");
        assert!(
            zip_path.exists(),
            "snapshot zip must land under snapshot root"
        );
        assert!(
            !output_dir
                .join("genesis_snapshot_devnet_020000.zip")
                .exists(),
            "snapshot zip must not be emitted under the typed artifact output dir"
        );
    }
}

pub(crate) struct GenesisReportArgs<'a> {
    pub(crate) network_type: ChainType,
    pub(crate) gen_duration_secs: u64,
    pub(crate) verify_duration_secs: u64,
    pub(crate) total_duration_secs: u64,
    pub(crate) state_hash: &'a [u8; 32],
    pub(crate) cli_command: Option<&'a str>,
}

pub(crate) fn write_genesis_report(
    output_dir: &Path,
    definitions: &[AssetDefinition],
    accumulator: &GenesisAssetAccumulator,
    args: GenesisReportArgs<'_>,
) -> Result<(), GenesisError> {
    use z00z_utils::time::{
        format_unix_timestamp_millis_utc, format_unix_timestamp_milliseconds_compact,
        SystemTimeProvider, TimeProvider,
    };

    let logger = StdoutLogger;
    let time_provider = SystemTimeProvider;
    let now_ms = time_provider.compat_unix_timestamp_millis();
    let timestamp = format_unix_timestamp_milliseconds_compact(now_ms);
    let report_path = output_dir.join(format!("genesis_report_{}.txt", timestamp));
    let mut report = File::create(&report_path).map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    let total_assets = accumulator.total_count();
    let total_rights = accumulator.total_right_count();
    let total_leaves = accumulator.total_leaf_count();
    let gen_rate = if args.gen_duration_secs > 0 {
        total_assets as f64 / args.gen_duration_secs as f64
    } else {
        0.0
    };
    let per_asset_ms = if total_assets > 0 {
        (args.total_duration_secs as f64 * 1000.0) / total_assets as f64
    } else {
        0.0
    };

    writeln!(report, "Z00Z Genesis Generation Report").map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(report, "==============================").map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(
        report,
        "Generated: {}",
        format_unix_timestamp_millis_utc(now_ms)
    )
    .map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    if let Some(cmd) = args.cli_command {
        writeln!(report, "Command: {}", cmd).map_err(|e| GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        })?;
    }
    writeln!(report).map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    writeln!(report, "📊 Generation Statistics").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "------------------------").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "Network: {}", args.network_type.as_str()).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(report, "Total assets generated: {}", total_assets).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(report, "Total rights generated: {}", total_rights).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(report, "Total settlement leaves: {}", total_leaves).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(
        report,
        "Generation time: {:.2}s",
        args.total_duration_secs as f64
    )
    .map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "Throughput: {:.2} assets/sec", gen_rate).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(report, "Per-asset time: {:.2}ms", per_asset_ms).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(report).map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    writeln!(report, "📈 Per-Class Breakdown").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "----------------------").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    for definition in definitions {
        writeln!(
            report,
            "  {}: {} assets (divisibility: {})",
            definition.class, definition.serials, definition.decimals
        )
        .map_err(|e| GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        })?;
    }
    writeln!(report).map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    writeln!(report, "🛡️  Verification Statistics").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "---------------------------").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "Total verified: {}", total_assets).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(
        report,
        "Verification time: {:.2}s",
        args.verify_duration_secs as f64
    )
    .map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "All proofs valid: ✓").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report).map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    writeln!(report, "🔐 Genesis State Hash").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "---------------------").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "State hash: {}", hex::encode(args.state_hash)).map_err(|e| {
        GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        }
    })?;
    writeln!(
        report,
        "⚠️  IMPORTANT: Hardcode this hash in consensus parameters!"
    )
    .map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report).map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;

    writeln!(report, "📋 Asset Definitions").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    writeln!(report, "--------------------").map_err(|e| GenesisError::FileWriteFailed {
        path: report_path.display().to_string(),
        error: e.to_string(),
    })?;
    for definition in definitions {
        writeln!(report, "  Symbol: {}", definition.symbol).map_err(|e| {
            GenesisError::FileWriteFailed {
                path: report_path.display().to_string(),
                error: e.to_string(),
            }
        })?;
        writeln!(report, "    Class: {}", definition.class).map_err(|e| {
            GenesisError::FileWriteFailed {
                path: report_path.display().to_string(),
                error: e.to_string(),
            }
        })?;
        writeln!(report, "    Serials: {}", definition.serials).map_err(|e| {
            GenesisError::FileWriteFailed {
                path: report_path.display().to_string(),
                error: e.to_string(),
            }
        })?;
        writeln!(report, "    Divisibility: {}", definition.decimals).map_err(|e| {
            GenesisError::FileWriteFailed {
                path: report_path.display().to_string(),
                error: e.to_string(),
            }
        })?;
        writeln!(report).map_err(|e| GenesisError::FileWriteFailed {
            path: report_path.display().to_string(),
            error: e.to_string(),
        })?;
    }

    logger.info(&format!(
        "   ✓ Report written: {}/genesis_report_{}.txt",
        output_dir.display(),
        timestamp
    ));
    Ok(())
}
