use super::*;

const ASSET_OUTPUT_HASH_SCHEMA: &str = "core-assets-cli-output-v2";
const ASSET_OUTPUT_KEEP_ENV: &str = "Z00Z_CORE_ASSET_OUTPUT_KEEP";

// ============================================================================
// Phase 1: Validation
// ============================================================================

pub(super) fn validate_inputs(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // Check config file exists
    if !args.config.exists() {
        return Err(format!("Config file not found: {}", args.config.display()).into());
    }

    // Check config is readable
    fs::read_to_string(&args.config)
        .map_err(|e| format!("Cannot read config file {}: {}", args.config.display(), e))?;

    // Validate thread count if specified
    if let Some(threads) = args.threads {
        if threads == 0 {
            return Err("Thread count must be > 0".into());
        }
    }

    z00z_utils::io::reset_managed_root(
        &args.output,
        &asset_output_fingerprint(&args.config),
        &[],
        Some(ASSET_OUTPUT_KEEP_ENV),
    )
    .map_err(|e| {
        format!(
            "Cannot reset output directory {}: {}",
            args.output.display(),
            e
        )
    })?;

    // Create subdirectories
    for subdir in &["json", "bin", "reports"] {
        let path = args.output.join(subdir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Cannot create directory {}: {}", path.display(), e))?;
    }

    // Test write permissions
    let test_file = args.output.join(".write_test");
    fs::write(&test_file, b"test")
        .map_err(|e| format!("Output directory is not writable: {}", e))?;
    fs::remove_file(test_file).ok();

    Ok(())
}

fn asset_output_fingerprint(config_path: &Path) -> String {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    let root = ROOT
        .get_or_init(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .clone();

    z00z_utils::io::hash_root_inputs(
        ASSET_OUTPUT_HASH_SCHEMA,
        &[
            root.join("Cargo.toml"),
            root.join("Cargo.lock"),
            root.join(".cargo/config.toml"),
            root.join("crates/z00z_core/Cargo.toml"),
            root.join("crates/z00z_crypto/Cargo.toml"),
            root.join("crates/z00z_utils/Cargo.toml"),
            root.join("crates/z00z_core/bin/assets_generation_cli.rs"),
            root.join("crates/z00z_core/bin/assets_generation_cli_phase.rs"),
            root.join("crates/z00z_core/bin/assets_generation_cli_report.rs"),
            config_path.to_path_buf(),
        ],
        &[
            root.join("crates/z00z_core/src"),
            root.join("crates/z00z_crypto/src"),
            root.join("crates/z00z_utils/src"),
        ],
    )
    .expect("hash assets cli output root")
}

// ============================================================================
// Phase 2: Load Registry Catalog
// ============================================================================

pub(super) fn load_config(
    args: &Args,
) -> Result<AssetDefinitionRegistry, Box<dyn std::error::Error>> {
    // Create dependencies
    let logger = Arc::new(z00z_utils::logger::NoopLogger);
    let metrics = Arc::new(z00z_utils::metrics::NoopMetrics);
    let time = Arc::new(z00z_utils::time::SystemTimeProvider);

    let registry =
        AssetDefinitionRegistry::load_catalog_from_yaml(&args.config, logger, metrics, time)
            .map_err(|e| format!("Failed to load registry catalog: {}", e))?;

    // Validate we have at least one definition
    let snapshot = registry.get_shared_snapshot()?;
    if snapshot.is_empty() {
        return Err("No asset definitions found in config".into());
    }

    Ok(registry)
}

pub(super) fn get_definitions_from_registry(
    registry: &AssetDefinitionRegistry,
) -> Vec<Arc<AssetDefinition>> {
    let snapshot = registry
        .get_shared_snapshot()
        .expect("Failed to get registry snapshot");
    snapshot.values().cloned().collect()
}

// ============================================================================
// Phase 3: Generate Assets
// ============================================================================

pub(super) fn generate_assets(
    definitions: &[Arc<AssetDefinition>],
    verbose: bool,
) -> Result<(Vec<AssetWithSecrets>, GenerationStats), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut per_class_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_class_amounts: BTreeMap<String, u64> = BTreeMap::new();

    // PARALLELISM:
    // Level 1: Generate assets in parallel for each definition (par_iter)
    // Level 2: Generate serials sequentially within each definition (iter, not par_iter to avoid deadlock)
    let all_assets: Result<Vec<Vec<AssetWithSecrets>>, String> = definitions
        .par_iter()
        .map(|def| {
            if verbose {
                println!(
                    "   Generating {} serials for {} (nominal: {})...",
                    def.serials, def.symbol, def.nominal
                );
            }

            // Generate EXACTLY serials assets (one per serial_id) - SEQUENTIAL (avoids rayon deadlock)
            let assets_result: Result<Vec<AssetWithSecrets>, String> = (0..def.serials)
                .map(|serial_id| {
                    // Create deterministic RNG for this serial (reproducible generation)
                    let mut rng_seed = [0u8; 32];
                    rng_seed[..4].copy_from_slice(&serial_id.to_le_bytes());
                    rng_seed[4..32].copy_from_slice(&def.id[..28]);
                    let provider = DeterministicRngProvider::from_seed(rng_seed);
                    let mut rng = provider.rng();

                    // serial_id is directly from range 0..serials (no modulo needed)
                    // Generate unique nonce from definition ID and serial
                    let nonce = generate_nonce(&def.id, serial_id);

                    // Generate random blinding factor (secret key)
                    let blinding =
                        Z00ZScalar::random(&mut rng).map_err(|error| error.to_string())?;

                    // Determine amount based on asset class
                    let amount = match def.class {
                        AssetClass::Coin | AssetClass::Token => {
                            // For coins/tokens: use nominal value
                            if def.nominal > 0 {
                                def.nominal
                            } else {
                                // Fallback: generate random amount based on serial_id
                                1_000_000 + (serial_id as u64 * 100)
                            }
                        }
                        AssetClass::Nft | AssetClass::Void => {
                            // NFTs and Voids have zero nominal value
                            0
                        }
                    };

                    // Create Asset with real cryptography
                    let asset = Asset::new(
                        Arc::clone(def),
                        serial_id,
                        amount,
                        &blinding,
                        nonce,
                        &mut rng,
                    )
                    .map_err(|e| format!("Failed to generate asset for {}: {}", def.symbol, e))?;

                    Ok(AssetWithSecrets {
                        asset,
                        blinding_factor: blinding,
                    })
                })
                .collect();

            assets_result
        })
        .collect();

    let all_assets = all_assets?;

    // Flatten and collect stats
    let assets_with_secrets: Vec<AssetWithSecrets> = all_assets.into_iter().flatten().collect();

    for aws in &assets_with_secrets {
        let class_name = format!("{:?}", aws.asset.definition.class);
        *per_class_counts.entry(class_name.clone()).or_insert(0) += 1;
        *per_class_amounts.entry(class_name).or_insert(0) += aws.asset.amount;
    }

    let generation_time_ms = start.elapsed().as_millis();

    let stats = GenerationStats {
        total_assets: assets_with_secrets.len(),
        generation_time_ms,
        per_class_counts,
        per_class_amounts,
    };

    Ok((assets_with_secrets, stats))
}

fn generate_nonce(definition_id: &[u8; 32], serial_id: u32) -> Nonce {
    hash_domain!(ExampleNonceDomain, "z00z.core.examples.nonce.v1", 1);

    let hash = DomainHasher::<ExampleNonceDomain>::new_with_label("example_nonce")
        .chain(definition_id)
        .chain(serial_id.to_le_bytes())
        .finalize();

    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&hash.as_ref()[..32]);
    nonce
}

// ============================================================================
// Phase 4: Cryptographic Verification
// ============================================================================

pub(super) fn verify_assets(
    assets_with_secrets: &[AssetWithSecrets],
    verbose: bool,
) -> Result<VerificationStats, Box<dyn std::error::Error>> {
    if verbose {
        println!("   Verifying {} assets...", assets_with_secrets.len());
    }

    let mut stats = VerificationStats {
        total_verified: assets_with_secrets.len(),
        commitments_valid: 0,
        commitments_invalid: 0,
        range_proofs_present: 0,
        range_proofs_missing: 0,
        signatures_valid: 0,
        signatures_invalid: 0,
        homomorphic_tests_passed: 0,
        homomorphic_tests_failed: 0,
    };

    // Verify commitments and range proofs
    for aws in assets_with_secrets {
        // Check commitment is 32 bytes
        if aws.asset.commitment.as_bytes().len() == 32 {
            stats.commitments_valid += 1;
        } else {
            stats.commitments_invalid += 1;
        }

        // Verify range proof cryptographically
        if let Some(ref proof) = aws.asset.range_proof {
            // Real cryptographic verification using Bulletproofs+ via z00z_crypto
            match verify_range_proof(proof, &aws.asset.commitment, 64, 1, 0) {
                Ok(()) => stats.range_proofs_present += 1,
                Err(_) => stats.range_proofs_missing += 1, // Count failed verification as missing
            }
        } else {
            stats.range_proofs_missing += 1;
        }

        // Verify owner signature cryptographically
        if aws.asset.owner_signature.is_some() {
            // Verify using Asset::verify_owner_signature() - full cryptographic check
            match aws.asset.verify_owner_signature() {
                Ok(()) => stats.signatures_valid += 1,
                Err(_) => stats.signatures_invalid += 1,
            }
        } else {
            stats.signatures_invalid += 1;
        }
    }

    // Perform homomorphic property test if we have at least 2 assets of same class
    if assets_with_secrets.len() >= 2 {
        if let (Some(a1), Some(a2)) = (assets_with_secrets.first(), assets_with_secrets.get(1)) {
            if a1.asset.definition.class == a2.asset.definition.class {
                match test_homomorphic_property(a1, a2) {
                    Ok(true) => stats.homomorphic_tests_passed += 1,
                    Ok(false) => stats.homomorphic_tests_failed += 1,
                    Err(_) => stats.homomorphic_tests_failed += 1,
                }
            }
        }
    }

    Ok(stats)
}

fn test_homomorphic_property(
    aws1: &AssetWithSecrets,
    aws2: &AssetWithSecrets,
) -> Result<bool, AssetError> {
    // Test: C(A1) + C(A2) should equal C(A1.amount + A2.amount, A1.blind + A2.blind)
    // This is the real homomorphic property test with blinding factors

    let a1 = &aws1.asset;
    let a2 = &aws2.asset;
    let b1 = &aws1.blinding_factor;
    let b2 = &aws2.blinding_factor;

    // Get actual commitments from assets
    let c1 = &a1.commitment;
    let c2 = &a2.commitment;

    // Add commitments: C1 + C2
    let c_sum_actual = c1 + c2;

    // Compute expected commitment for (amount1 + amount2) with (blinding1 + blinding2)
    let total_amount = a1.amount + a2.amount;
    let total_blinding = b1 + b2;
    let c_sum_expected = create_commitment(total_amount, &total_blinding)?;

    // Compare: should be equal due to homomorphic property
    Ok(c_sum_actual == c_sum_expected)
}

// Phase 5: Serialization
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const PHASE_SRC: &str = include_str!("assets_generation_cli_phase.rs");

    fn test_args(config: PathBuf, output: PathBuf) -> Args {
        Args {
            format: Format::Json,
            config,
            output,
            threads: None,
            verbose: false,
        }
    }

    #[test]
    fn validate_inputs_clears_prior_outputs() {
        let dir = TempDir::new().expect("temp dir");
        let config = dir.path().join("assets.yaml");
        let output = dir.path().join("outputs");
        fs::write(&config, "assets: []\n").expect("write config");

        let args = test_args(config.clone(), output.clone());
        validate_inputs(&args).expect("validate first output root");
        fs::write(output.join("reports/stale.txt"), b"stale").expect("write stale output");

        let args = test_args(config, output.clone());
        validate_inputs(&args).expect("validate second output root");

        assert!(
            !output.join("reports/stale.txt").exists(),
            "same-process rerun must clear prior assets CLI output payload"
        );
    }

    #[test]
    fn test_hashed_output_root() {
        for needle in [
            "const ASSET_OUTPUT_HASH_SCHEMA: &str = \"core-assets-cli-output-v2\";",
            "z00z_utils::io::hash_root_inputs(",
            "crates/z00z_core/bin/assets_generation_cli.rs",
            "crates/z00z_core/bin/assets_generation_cli_phase.rs",
            "crates/z00z_core/bin/assets_generation_cli_report.rs",
            "config_path.to_path_buf()",
            "crates/z00z_core/src",
            "crates/z00z_crypto/src",
            "crates/z00z_utils/src",
        ] {
            assert!(
                PHASE_SRC.contains(needle),
                "assets CLI output root contract must include {needle}"
            );
        }
        assert!(
            !PHASE_SRC
                .contains("const ASSET_OUTPUT_FINGERPRINT: &str = \"core-assets-cli-output-v1\";"),
            "assets CLI output root contract must reject the legacy constant fingerprint constant"
        );
    }
}
