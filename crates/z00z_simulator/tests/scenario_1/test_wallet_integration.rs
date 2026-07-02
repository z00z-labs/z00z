use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process,
    sync::OnceLock,
};

use serde_json::Value;
use z00z_crypto::expert::encoding::{from_hex, to_hex};
use z00z_simulator::{config::ScenarioCfg, scenario_1::runner};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{load_json, read_file, read_to_string, write_file},
};
use z00z_wallets::domains::hashing::compute_wallet_file_id;

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

const ACTORS: [&str; 3] = ["alice", "bob", "charlie"];
const ZSTD_MAGIC: &[u8; 4] = &[0x28, 0xB5, 0x2F, 0xFD];

static S2_OUT: OnceLock<PathBuf> = OnceLock::new();
static PUBLIC_LANE_OUT: OnceLock<PathBuf> = OnceLock::new();

fn test_cfg_paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1");
    (
        base.join("scenario_config.yaml"),
        base.join("scenario_design.yaml"),
    )
}

fn stage2_out() -> &'static PathBuf {
    S2_OUT.get_or_init(|| {
        let root = fixture_cache::ensure_shared_case("wallet_stage2_shared_v1", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |_| {});
            let _ctx = stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1_u32, 2]);
            assert!(
                out.join("stage_2_snapshot.json").exists(),
                "shared stage2 fixture must contain stage_2 snapshot"
            );
            assert!(
                out.join("wallets").exists(),
                "shared stage2 wallets missing"
            );
            assert!(out.join("keys").exists(), "shared stage2 keys missing");
        });

        root.join("outputs/scenario_1")
    })
}

fn public_lane_out() -> &'static PathBuf {
    PUBLIC_LANE_OUT.get_or_init(|| {
        let root = fixture_cache::ensure_shared_case("wallet_public_lane_shared_v1", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |_| {});
            let run = runner::run_with_paths(&cfg_path, &design_path).expect("scenario run");
            assert!(run.is_ok(), "shared public-lane scenario must complete");
            assert!(out.join("pub_flow.json").exists(), "pub_flow.json missing");
            assert!(out.join("val_flow.json").exists(), "val_flow.json missing");
            assert!(
                out.join("watch_flow.json").exists(),
                "watch_flow.json missing"
            );
        });

        root.join("outputs/scenario_1")
    })
}

fn read_owner_handle(out: &Path, name: &str) -> [u8; 32] {
    let path = out.join("keys").join(format!("{name}_keys.json"));
    let value: Value = load_json(&path).expect("load keys json");
    let hex = value["owner_handle"]
        .as_str()
        .expect("owner_handle field missing");
    let bytes = from_hex(hex).expect("invalid hex in owner_handle");
    bytes
        .as_slice()
        .try_into()
        .expect("owner_handle must be 32 bytes")
}

fn read_wallet_id(out: &Path, name: &str) -> String {
    let path = out.join("keys").join(format!("{name}_keys.json"));
    let value: Value = load_json(&path).expect("load keys json");
    let wallet = value["wallet_id"]
        .as_str()
        .expect("wallet_id field missing");
    if wallet.starts_with("wallet_") {
        wallet.to_string()
    } else {
        format!("wallet_{wallet}")
    }
}

fn hex_str(bytes: &[u8]) -> String {
    to_hex(bytes)
}

fn check_no_forbidden_keys(
    val: &Value,
    forbidden_exact: &[&str],
    forbidden_suffix: &[&str],
    forbidden_substr: &[&str],
    ctx: &str,
) {
    match val {
        Value::Object(obj) => {
            for (key, nested) in obj {
                let low = key.to_lowercase();

                if forbidden_exact.iter().any(|f| low == *f) {
                    panic!("{ctx}: forbidden key '{key}' in keys JSON");
                }
                if forbidden_suffix.iter().any(|s| low.ends_with(s)) {
                    panic!("{ctx}: forbidden key '{key}' in keys JSON");
                }
                if forbidden_substr.iter().any(|s| low.contains(s)) {
                    panic!("{ctx}: forbidden key '{key}' in keys JSON");
                }

                check_no_forbidden_keys(
                    nested,
                    forbidden_exact,
                    forbidden_suffix,
                    forbidden_substr,
                    ctx,
                );
            }
        }
        Value::Array(arr) => {
            for nested in arr {
                check_no_forbidden_keys(
                    nested,
                    forbidden_exact,
                    forbidden_suffix,
                    forbidden_substr,
                    ctx,
                );
            }
        }
        _ => {}
    }
}

#[cfg(feature = "wallet_debug_tools")]
fn debug_secret_path(out: &Path) -> PathBuf {
    out.join("wallets")
        .join("private")
        .join("wlt_secrets_debug.md")
}

#[cfg(feature = "wallet_debug_tools")]
fn actor_seed_phrase(out: &Path, name: &str) -> String {
    let path = debug_secret_path(out);
    let data = read_to_string(&path).expect("read wlt_secrets_debug.md");

    for line in data.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(name) {
            continue;
        }

        let cols: Vec<&str> = trimmed.split('|').map(|part| part.trim()).collect();
        if cols.len() >= 8 && cols[0] == name {
            return cols[3].to_string();
        }
    }

    panic!("seed phrase row not found for {name}");
}

#[test]
fn test_stage2_wallet_ids_unique() {
    let out = stage2_out();
    let filenames = ACTORS
        .iter()
        .map(|name| {
            let wallet_id = read_wallet_id(out, name);
            let file_id = compute_wallet_file_id(&wallet_id);
            format!("wallet_{}.wlt", hex_str(&file_id[..8]))
        })
        .collect::<Vec<_>>();

    let unique = filenames.iter().collect::<HashSet<_>>();
    assert_eq!(unique.len(), 3, "wallet filenames must be unique");
    for file in &filenames {
        assert!(
            out.join("wallets").join(file).exists(),
            "missing actor wallet file {file}"
        );
    }
}

#[test]
fn test_stage2_owner_handles_unique() {
    let out = stage2_out();
    let alice = read_owner_handle(out, "alice");
    let bob = read_owner_handle(out, "bob");
    let charlie = read_owner_handle(out, "charlie");

    assert_ne!(alice, bob, "alice and bob share owner_handle");
    assert_ne!(alice, charlie, "alice and charlie share owner_handle");
    assert_ne!(bob, charlie, "bob and charlie share owner_handle");
}

#[test]
fn test_stage2_owner_handles_nonzero() {
    for name in ACTORS {
        let handle = read_owner_handle(stage2_out(), name);
        assert_ne!(handle, [0u8; 32], "{name} owner_handle is all-zero");
    }
}

#[test]
fn test_stage2_keys_secret_free() {
    let forbidden_exact = [
        "receiver_secret",
        "seed",
        "seed_phrase",
        "mnemonic",
        "password",
        "sk",
        "private_key",
        "secret_key",
    ];
    let forbidden_suffix = [
        "_sk",
        "_secret",
        "_seed",
        "_seed_phrase",
        "_mnemonic",
        "_password",
        "_private_key",
    ];
    let forbidden_substr = [
        "receiver_secret",
        "seed_phrase",
        "mnemonic",
        "password",
        "private",
        "secret",
    ];

    for name in ACTORS {
        let path = stage2_out().join("keys").join(format!("{name}_keys.json"));
        assert!(path.exists(), "{name}_keys.json not found");
        let value: Value = load_json(&path).expect("load keys json");
        check_no_forbidden_keys(
            &value,
            &forbidden_exact,
            &forbidden_suffix,
            &forbidden_substr,
            name,
        );
    }
}

#[test]
fn test_stage2_wlt_filename_determinism() {
    for name in ACTORS {
        let wallet_id = read_wallet_id(stage2_out(), name);
        let file_id = compute_wallet_file_id(&wallet_id);
        let expected_name = format!("wallet_{}.wlt", hex_str(&file_id[..8]));
        let path = stage2_out().join("wallets").join(&expected_name);
        assert!(
            path.exists(),
            "expected wlt not found: {expected_name} for actor {name}"
        );
    }
}

#[test]
fn test_stage2_actors_chain_devnet() {
    let path = stage2_out().join("stage_2_snapshot.json");
    assert!(path.exists(), "stage_2_snapshot.json not found");
    let snap: Value = load_json(&path).expect("load stage_2_snapshot.json");
    assert_eq!(
        snap["chain_id"].as_str().unwrap_or_default(),
        "devnet",
        "snapshot chain_id must be devnet"
    );
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_stage2_seed_have_24() {
    for name in ACTORS {
        let phrase = actor_seed_phrase(stage2_out(), name);
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(
            words.len(),
            24,
            "{name} seed phrase: expected 24 words, got {}",
            words.len()
        );
        assert!(
            words.iter().all(|word| word.len() >= 3),
            "{name}: seed phrase has suspiciously short words"
        );
    }
}

#[test]
fn test_stage2_rpc_no_secrets() {
    let path = stage2_out().join("logs/rpc_logger.json");
    assert!(path.exists(), "rpc_logger.json not found");
    let log = read_to_string(&path).expect("failed to read rpc_logger.json");
    assert!(!log.is_empty(), "rpc_logger.json is empty");

    let public_secret_path = stage2_out().join("wallets").join("wlt_secrets_debug.md");
    let private_secret_path = stage2_out()
        .join("wallets")
        .join("private")
        .join("wlt_secrets_debug.md");
    assert!(
        !public_secret_path.exists(),
        "default simulator lane must not emit a plaintext wallet secret artifact"
    );
    if cfg!(feature = "wallet_debug_tools") {
        assert!(
            private_secret_path.exists(),
            "wallet_debug_tools feature must keep the private debug secret artifact on the private lane"
        );
    } else {
        assert!(
            !private_secret_path.exists(),
            "wallet_debug_tools is disabled, so the private debug secret artifact must stay absent"
        );
    }

    let secrets = [
        "Alice_Pass_Z00Z_42!",
        "Bob_Pass_Z00Z_43!",
        "Charlie_Pass_Z00Z_44!",
        "abandon",
    ];
    for secret in secrets {
        assert!(
            !log.contains(secret),
            "rpc_logger.json contains plaintext secret: {secret}"
        );
    }
}

#[test]
fn test_public_lane_secret_free() {
    let out = public_lane_out();
    let public_secret_path = out.join("wallets").join("wlt_secrets_debug.md");
    let private_secret_path = out
        .join("wallets")
        .join("private")
        .join("wlt_secrets_debug.md");
    assert!(
        !public_secret_path.exists(),
        "public lane must never emit plaintext wallet secret artifacts"
    );
    if cfg!(feature = "wallet_debug_tools") {
        assert!(
            private_secret_path.exists(),
            "debug feature must keep the secret artifact on the private lane only"
        );
    } else {
        assert!(
            !private_secret_path.exists(),
            "without wallet_debug_tools the private debug secret artifact must stay absent"
        );
    }

    let val_flow: Value = load_json(out.join("val_flow.json")).expect("load val_flow");
    let watch_flow: Value = load_json(out.join("watch_flow.json")).expect("load watch_flow");
    assert_eq!(
        val_flow["checkpoint_id_hex"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(
        watch_flow["checkpoint_id_hex"].as_str().map(str::len),
        Some(64)
    );
    assert_eq!(val_flow["verdict_kind"].as_str(), Some("accepted"));
    assert_eq!(watch_flow["verdict_kind"].as_str(), Some("accepted"));
    assert_eq!(watch_flow["publication_state"].as_str(), Some("accepted"));
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_stage2_secret_is_private() {
    use std::os::unix::fs::PermissionsExt;

    let path = debug_secret_path(stage2_out());
    assert!(path.exists(), "private debug secret artifact missing");
    assert!(
        !stage2_out()
            .join("wallets")
            .join("wlt_secrets_debug.md")
            .exists(),
        "debug secret artifact must not be published at the old public path"
    );

    let mode = std::fs::metadata(&path)
        .expect("metadata for private debug secret artifact")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o600,
        "private debug secret artifact must be mode 0600"
    );
}

#[test]
fn test_reset_outputs_outside_sandbox() {
    let outside =
        std::env::temp_dir().join(format!("z00z-reset-outside-sandbox-{}", process::id()));
    let err = runner::reset_outputs_dir(outside.to_str().expect("utf-8 temp path"))
        .expect_err("outside sandbox path must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("outside the approved sandbox"),
        "unexpected sandbox rejection error: {msg}"
    );
    assert!(
        !outside.exists(),
        "sandbox rejection must happen before any output directory is created"
    );
}

#[test]
fn test_reset_suffix_outside_sandbox() {
    let outside = std::env::temp_dir()
        .join(format!("z00z-suffix-outside-sandbox-{}", process::id()))
        .join("outputs/scenario_1");
    let err = runner::reset_outputs_dir(outside.to_str().expect("utf-8 temp path"))
        .expect_err("suffix-matched outside path must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("outside the approved sandbox"),
        "unexpected suffix sandbox rejection error: {msg}"
    );
    assert!(
        !outside.exists(),
        "suffix sandbox rejection must happen before any output directory is created"
    );
}

#[test]
fn test_scenario1_invalid_no_panic() {
    let (cfg_path, design_path) = test_cfg_paths();
    let mut cfg = ScenarioCfg::from_file(&cfg_path).expect("load scenario_config.yaml");
    cfg.chain = "invalid-chain-name".to_string();

    let temp = tempfile::tempdir().expect("temp dir");
    let tmp_dir = temp.path();
    let patched = tmp_dir.join("scenario_1_invalid_chain.yaml");
    let cfg_bytes = YamlCodec
        .serialize(&cfg)
        .expect("serialize invalid-chain config");
    write_file(&patched, &cfg_bytes).expect("write invalid-chain config");

    let err = runner::run_with_paths(&patched, &design_path)
        .expect_err("invalid chain config must return an error");
    let msg = err.to_string();
    assert!(
        msg.contains("invalid simulator chain 'invalid-chain-name'"),
        "unexpected invalid chain error: {msg}"
    );
}

#[test]
fn test_stage2_wlt_zstd_magic() {
    for name in ACTORS {
        let wallet_id = read_wallet_id(stage2_out(), name);
        let file_id = compute_wallet_file_id(&wallet_id);
        let path = stage2_out()
            .join("wallets")
            .join(format!("wallet_{}.wlt", hex_str(&file_id[..8])));
        assert!(path.exists(), "wlt not found for {name}");

        let bytes = read_file(&path).expect("failed to read wlt");
        assert!(bytes.len() >= 4, "{name}: wlt file too small");
        assert!(
            bytes.starts_with(ZSTD_MAGIC),
            "{name}: wlt missing zstd magic (first 4 bytes: {:02x?})",
            &bytes[..4]
        );
    }
}

#[test]
fn test_stage2_snapshot_consistent() {
    let path = stage2_out().join("stage_2_snapshot.json");
    assert!(path.exists(), "stage_2_snapshot.json not found");
    let snap: Value = load_json(&path).expect("load stage_2_snapshot.json");

    assert_eq!(
        snap["wallet_count"].as_u64().unwrap_or_default(),
        3,
        "wallet_count must be 3"
    );
    assert!(
        snap["actors_ready_for_stage3"].as_bool().unwrap_or(false),
        "actors_ready_for_stage3 must be true"
    );
    assert_eq!(
        snap["stage"].as_u64().unwrap_or_default(),
        2,
        "snapshot stage field must be 2"
    );
}
