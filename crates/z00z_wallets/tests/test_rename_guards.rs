#![cfg(not(target_arch = "wasm32"))]

use std::{
    fs,
    path::{Path, PathBuf},
};

use z00z_utils::io::read_to_string;

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(crate_dir)
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn workspace_path_exists(relative_path: &str) -> bool {
    workspace_root().join(relative_path).exists()
}

fn collect_test_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read directory {}: {error}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_test_files(&path, files);
            continue;
        }
        let is_rust = path.extension().and_then(|ext| ext.to_str()) == Some("rs");
        if !is_rust {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|item| item.to_str())
            .unwrap_or("");
        let text = path.to_string_lossy();
        let is_test_path = text.contains("/tests/")
            || text.contains("/test/")
            || name.starts_with("test_")
            || name.ends_with("_tests.rs")
            || name.ends_with("tests.rs");
        if is_test_path {
            files.push(path);
        }
    }
}

#[test]
fn test_curated_is_live() {
    let session = read_workspace_file("crates/z00z_wallets/src/redb_store/session.rs");
    let migrations = read_workspace_file("crates/z00z_wallets/src/redb_store/migrations.rs");

    assert!(session.contains("flush_work_file_to_wallet("));
    assert!(migrations.contains("unsupported index format version"));
    assert!(!session.contains("flush_work_file_to_wlt"));
    assert!(!migrations.contains("flush_work_file_to_wlt"));
}

#[test]
fn test_no_original_spellings() {
    let config_defaults = read_workspace_file("crates/z00z_simulator/src/config_defaults.rs");
    let config = read_workspace_file("crates/z00z_simulator/src/config.rs");
    let argon2_kdf = read_workspace_file("crates/z00z_crypto/src/kdf/argon2_kdf.rs");
    let wallet_backup = read_workspace_file("crates/z00z_wallets/src/backup/wallet_backup.rs");

    assert!(config_defaults.contains("stage5_recipient_output_index"));
    assert!(config.contains("stage5_recipient_output_index"));
    assert!(!config_defaults.contains("default_stage5_recipient_output_index"));
    assert!(!config.contains("default_stage5_recipient_output_index"));
    assert!(argon2_kdf.contains("pub fn derive_argon2id32_key"));
    assert!(wallet_backup.contains("derive_argon2id32_key"));
}

#[test]
fn test_wave_terms_gone() {
    let live_paths = [
        "crates/z00z_core/src/assets/test_asset.rs",
        "crates/z00z_core/src/assets/test_definition.rs",
        "crates/z00z_core/src/assets/test_leaf.rs",
        "crates/z00z_core/src/assets/test_nonce.rs",
        "crates/z00z_core/src/assets/test_registry.rs",
        "crates/z00z_core/src/assets/test_wire_compat.rs",
        "crates/z00z_core/src/genesis/test_genesis.rs",
        "crates/z00z_core/src/genesis/test_validator.rs",
        "crates/z00z_crypto/src/aead/test_aead.rs",
        "crates/z00z_simulator/src/config_defaults.rs",
        "crates/z00z_simulator/src/scenario_1/stage_6/tx_lane_runtime.rs",
        "crates/z00z_simulator/src/scenario_1/stage_6/test_tx_lane_runtime_suite.rs",
        "crates/z00z_simulator/src/scenario_1/stage_6/tx_lane_runtime_support.rs",
        "crates/z00z_storage/src/checkpoint/test_checkpoint.rs",
        "crates/z00z_storage/src/snapshot/test_snapshot.rs",
        "crates/z00z_storage/tests/test_snapshot_replay_bound.rs",
        "crates/z00z_utils/src/io/test_fs_io_suite.rs",
        "crates/z00z_utils/src/io/test_fs_suite.rs",
        "crates/z00z_wallets/src/config/common-passwords.txt",
        "crates/z00z_wallets/src/config/mod.rs",
        "crates/z00z_wallets/src/config/password_denylist.bloom",
        "crates/z00z_wallets/src/config/wallet_config.yaml",
        "crates/z00z_wallets/src/config/wallet_config_defaults.rs",
        "crates/z00z_wallets/src/config/redb-schema.yaml",
        "crates/z00z_wallets/src/persistence/receipts.rs",
        "crates/z00z_wallets/src/persistence/scans.rs",
        "crates/z00z_wallets/src/key/bip.rs",
        "crates/z00z_wallets/src/key/bip32.rs",
        "crates/z00z_wallets/src/key/manager_core.rs",
        "crates/z00z_wallets/src/key/manager_redb.rs",
        "crates/z00z_wallets/src/key/seed.rs",
        "crates/z00z_wallets/src/key/receiver_keys.rs",
        "crates/z00z_wallets/src/key/test_bip44_manager_suite.rs",
        "crates/z00z_wallets/src/key/test_bip44_manager_entropy_suite.rs",
        "crates/z00z_wallets/src/key/test_manager_impl_suite.rs",
        "crates/z00z_wallets/src/key/test_receiver_keys_suite.rs",
        "crates/z00z_wallets/docs/KEYS-Bip44-UserGuide.md",
        "crates/z00z_wallets/docs/KEYS-DERIVATION.md",
        "crates/z00z_wallets/docs/KEYS-GUIDE.md",
        "crates/z00z_wallets/docs/KEYS-EXPLANATION.md",
        "crates/z00z_wallets/docs/WLT-BREAKDOWN.md",
        "crates/z00z_wallets/docs/Z00Z-WALLET-PERSISTENCE.md",
        "crates/z00z_wallets/docs/bip44_derivation.md",
        "crates/z00z_wallets/src/rpc/test_tx_impl.rs",
        "crates/z00z_wallets/src/rpc/test_tx_impl_suite.rs",
        "crates/z00z_wallets/src/receiver/manager.rs",
        "crates/z00z_wallets/src/receiver/nfc_ndef.rs",
        "crates/z00z_wallets/src/receiver/asset_scan.rs",
        "crates/z00z_wallets/src/receiver/asset_ownership_check.rs",
        "crates/z00z_wallets/src/security/vault.rs",
    ];
    let removed_paths = [
        concat!("crates/z00z_core/src/assets/test_asset", "_suite.rs"),
        concat!("crates/z00z_core/src/assets/test_definition", "_suite.rs"),
        concat!("crates/z00z_core/src/assets/test_leaf", "_suite.rs"),
        concat!("crates/z00z_core/src/assets/test_nonce", "_suite.rs"),
        concat!("crates/z00z_core/src/assets/test_registry", "_suite.rs"),
        concat!("crates/z00z_core/src/genesis/test_genesis", "_suite.rs"),
        concat!("crates/z00z_core/src/genesis/test_validator", "_suite.rs"),
        "crates/z00z_core/src/assets/asset_tests.rs",
        "crates/z00z_core/src/genesis/genesis_tests.rs",
        "crates/z00z_crypto/src/aead_tests.rs",
        "crates/z00z_crypto/src/aead/test_mod.rs",
        "crates/z00z_crypto/src/aead/test_aead_suite.rs",
        "crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl/tx_lane_runtime/test_tx_lane_runtime_suite/test_tx_lane_runtime_support.rs",
        "crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl/tx_lane_runtime/test_tx_lane_runtime_suite.rs",
        "crates/z00z_storage/src/checkpoint/test_artifact_suite.rs",
        "crates/z00z_storage/src/checkpoint/artifact_tests.rs",
        "crates/z00z_storage/tests/snapshot_suite/test_mod.rs",
        "crates/z00z_utils/src/io/fs/test_mod.rs",
        "crates/z00z_utils/src/io/fs_tests.rs",
        "crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/test_mod.rs",
        "crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/test_tx_impl_suite.rs",
        "crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests.rs",
        "crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_tests_body.rs",
        "crates/z00z_wallets/src/key/bip/bip32.rs",
        "crates/z00z_wallets/src/key/bip/mod.rs",
        "crates/z00z_wallets/src/key/bip/docs/KEYS-Bip44-UserGuide.md",
        "crates/z00z_wallets/src/key/bip/docs/KEYS-GUIDE.md",
        "crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md",
        "crates/z00z_wallets/src/key/bip/docs/bip44_derivation.md",
        "crates/z00z_wallets/src/key/manager/KEYS-DERIVATION.md",
        "crates/z00z_wallets/src/key/test_bip44_manager.inc.rs",
        "crates/z00z_wallets/src/key/test_bip44_manager_entropy.inc.rs",
        "crates/z00z_wallets/src/key/test_bip32_manager.inc.rs",
        "crates/z00z_wallets/src/key/test_bip32_manager_entropy.inc.rs",
        "crates/z00z_wallets/src/key/manager.rs",
        "crates/z00z_wallets/src/key/manager/mod.rs",
        "crates/z00z_wallets/src/key/receiver.rs",
        "crates/z00z_wallets/src/key/receiver/mod.rs",
        "crates/z00z_wallets/src/key/seed/mod.rs",
        "crates/z00z_wallets/src/key/seed/seed_cipher.rs",
        "crates/z00z_wallets/src/persistence/receipts/mod.rs",
        "crates/z00z_wallets/src/persistence/receipts/storage.rs",
        "crates/z00z_wallets/src/persistence/receipts/storage_impl.rs",
        "crates/z00z_wallets/src/persistence/scans/mod.rs",
        "crates/z00z_wallets/src/persistence/scans/storage.rs",
        "crates/z00z_wallets/src/persistence/scans/storage_impl.rs",
        "crates/z00z_wallets/src/receiver/card/nfc_utils.rs",
        "crates/z00z_wallets/src/receiver/manager/mod.rs",
        "crates/z00z_wallets/src/receiver/ownership/claim_own.rs",
        "crates/z00z_wallets/src/receiver/scan/mod.rs",
        "crates/z00z_wallets/src/wallet_config_support.rs",
        "crates/z00z_wallets/config/wallet_config.yaml",
        "crates/z00z_wallets/config/security/common-passwords.txt",
        "crates/z00z_wallets/config/security/password_denylist.bloom",
        "crates/z00z_wallets/schemas/redb-schema.yaml",
        "crates/z00z_wallets/src/security/common-passwords.txt",
        "crates/z00z_wallets/src/security/password_denylist.bloom",
        "crates/z00z_wallets/src/security/vault/mod.rs",
        "crates/z00z_wallets/src/security/vault/secret_store_impl.rs",
        "crates/z00z_wallets/docs/KEYS_EXPALNATION.md",
        "crates/z00z_wallets/docs/🔐-разбор-WLT.md",
        "crates/z00z_wallets/docs/🔐-разбор-кошелька-Z00Z.md",
    ];

    for live_path in live_paths {
        assert!(
            workspace_path_exists(live_path),
            "expected live path: {live_path}"
        );
    }

    for removed_path in removed_paths {
        assert!(
            !workspace_path_exists(removed_path),
            "unexpected removed path: {removed_path}"
        );
    }
}

#[test]
fn test_wave_renamed_files() {
    let assets = read_workspace_file("crates/z00z_core/src/assets/assets.rs");
    let genesis = read_workspace_file("crates/z00z_core/src/genesis/genesis.rs");
    let wire = read_workspace_file("crates/z00z_core/src/assets/wire.rs");
    let aead_parent = read_workspace_file("crates/z00z_crypto/src/aead/mod.rs");
    let aead_suite = read_workspace_file("crates/z00z_crypto/src/aead/test_aead.rs");
    let checkpoint_mod = read_workspace_file("crates/z00z_storage/src/checkpoint/mod.rs");
    let checkpoint_store = read_workspace_file("crates/z00z_storage/src/checkpoint/store.rs");
    let fs_parent = read_workspace_file("crates/z00z_utils/src/io/fs.rs");
    let fs_io_suite = read_workspace_file("crates/z00z_utils/src/io/test_fs_io_suite.rs");
    let fs_suite = read_workspace_file("crates/z00z_utils/src/io/test_fs_suite.rs");
    let simulator_mod = read_workspace_file("crates/z00z_simulator/src/scenario_1/stage_6/mod.rs");
    let simulator_parent =
        read_workspace_file("crates/z00z_simulator/src/scenario_1/stage_6/tx_lane_runtime.rs");
    let simulator_suite = read_workspace_file(
        "crates/z00z_simulator/src/scenario_1/stage_6/test_tx_lane_runtime_suite.rs",
    );
    let tx_impl = read_workspace_file("crates/z00z_wallets/src/rpc/tx_rpc_impl.rs");
    let tx_impl_suite = read_workspace_file("crates/z00z_wallets/src/rpc/test_tx_impl.rs");

    assert!(assets.contains("#[path = \"test_asset.rs\"]"));
    assert!(assets.contains("mod tests;"));
    assert!(genesis.contains("#[cfg(test)]"));
    assert!(genesis.contains("#[path = \"test_genesis.rs\"]"));
    assert!(genesis.contains("mod test_genesis;"));
    assert!(wire.contains("#[path = \"test_wire_compat.rs\"]"));
    assert!(wire.contains("mod test_wire_compat;"));
    assert!(aead_parent.contains("mod test_aead;"));
    assert!(aead_suite.contains("fn test_encrypt_decrypt_roundtrip"));
    assert!(checkpoint_mod.contains("mod test_checkpoint;"));
    assert!(checkpoint_mod.contains("mod test_store;"));
    assert!(fs_parent.contains("#[path = \"test_fs_io_suite.rs\"]"));
    assert!(fs_parent.contains("#[path = \"test_fs_suite.rs\"]"));
    assert!(fs_parent.contains("mod tests;"));
    assert!(fs_io_suite.contains("fn test_write_read_file()"));
    assert!(fs_suite.contains("fn test_io_split_files_exist()"));
    assert!(simulator_mod.contains("mod test_tx_lane_runtime_suite;"));
    assert!(simulator_mod.contains("mod tx_lane_runtime_suite_support;"));
    assert!(simulator_parent.contains("use super::tx_lane_runtime_flow::{"));
    assert!(simulator_parent.contains("use super::tx_lane_runtime_support::{"));
    assert!(simulator_suite.contains("use super::tx_lane_runtime_suite_support::{"));
    assert!(simulator_suite.contains("use crate::scenario_1::stage_6::shared_cases;"));
    assert!(tx_impl.contains("mod tests;"));
    assert!(tx_impl_suite.contains("mod test_tx_impl_suite;"));

    assert!(!assets.contains("asset_tests.rs"));
    assert!(!genesis.contains("genesis_tests.rs"));
    assert!(!assets.contains(concat!("#[path = \"test_asset", "_suite.rs\"]")));
    assert!(!genesis.contains(concat!("mod test_genesis", "_suite;")));
    assert!(!genesis.contains(concat!("include!(\"test_genesis", "_suite.rs\");")));
    assert!(!wire.contains(concat!("mod test_wire_compat", "_suite;")));
    assert!(!aead_parent.contains("aead_tests.rs"));
    assert!(!aead_parent.contains("test_mod.rs"));
    assert!(!aead_suite.contains("include!(\"test_aead_suite.rs\");"));
    assert!(!aead_suite.contains("aead_tests.rs"));
    assert!(!checkpoint_mod.contains("artifact_tests.rs"));
    assert!(!checkpoint_mod.contains("test_artifact_suite.rs"));
    assert!(!checkpoint_store.contains("test_store_suite.rs"));
    assert!(!fs_parent.contains("fs_tests.rs"));
    assert!(!fs_io_suite.contains("fs_tests.rs"));
    assert!(!fs_suite.contains("fs_tests.rs"));
    assert!(!simulator_mod.contains("stage_4_utils"));
    assert!(!simulator_parent.contains("tx_lane_runtime_tests.rs"));
    assert!(!simulator_suite.contains("stage_4_utils"));
    assert!(!tx_impl.contains("tx_impl_tests.rs"));
    assert!(!tx_impl_suite.contains("tx_impl_tests_body.rs"));
}

#[test]
fn test_key_docs_are_canonical() {
    let derivation = read_workspace_file("crates/z00z_wallets/docs/bip44_derivation.md");
    let explanation = read_workspace_file("crates/z00z_wallets/docs/KEYS-EXPLANATION.md");

    assert!(derivation.contains("crates/z00z_wallets/docs/KEYS-Bip44-UserGuide.md"));
    assert!(derivation.contains("[Z00Z BIP-44 User Guide](./KEYS-Bip44-UserGuide.md)"));
    assert!(!derivation.contains("crates/z00z_wallets/src/key/docs/KEYS-Bip44-UserGuide.md"));
    assert!(!derivation.contains("[Z00Z BIP-44 User Guide](../docs/KEYS-Bip44-UserGuide.md)"));

    assert!(explanation.contains("crates/z00z_wallets/src/key/manager_core.rs"));
    assert!(explanation.contains("crates/z00z_wallets/src/key/bip32.rs"));
    assert!(!explanation.contains("crates/z00z_wallets/src/key/bip/bip32.rs"));
}

#[test]
fn test_wallet_docs_ascii() {
    let docs_dir = workspace_root().join("crates/z00z_wallets/docs");
    let entries = fs::read_dir(&docs_dir)
        .unwrap_or_else(|error| panic!("failed to read directory {}: {error}", docs_dir.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read entry: {error}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|item| item.to_str())
            .unwrap_or("<invalid>");
        assert!(file_name.is_ascii(), "non-ASCII doc file name: {file_name}");

        let contents = read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let has_cyrillic = contents
            .chars()
            .any(|ch| matches!(ch, '\u{0400}'..='\u{04FF}' | '\u{0500}'..='\u{052F}'));
        assert!(!has_cyrillic, "non-English wallet doc: {}", path.display());
    }
}

#[test]
fn test_owner_paths_canonical() {
    let stealth_mod = read_workspace_file("crates/z00z_wallets/src/stealth/mod.rs");

    for live_path in [
        "crates/z00z_wallets/src/stealth/ecdh.rs",
        "crates/z00z_wallets/src/stealth/kdf.rs",
        "crates/z00z_wallets/src/stealth/zkpack.rs",
        "crates/z00z_wallets/src/stealth/test_zkpack.rs",
    ] {
        assert!(
            workspace_path_exists(live_path),
            "expected canonical stealth owner path: {live_path}"
        );
    }

    for removed_path in [
        "crates/z00z_wallets/src/stealth/facade_ecdh.rs",
        "crates/z00z_wallets/src/stealth/facade_kdf.rs",
        "crates/z00z_wallets/src/stealth/facade_zkpack/mod.rs",
        "crates/z00z_wallets/src/stealth/facade_zkpack/test_mod.rs",
    ] {
        assert!(
            !workspace_path_exists(removed_path),
            "unexpected stealth facade path: {removed_path}"
        );
    }

    assert!(stealth_mod.contains("pub mod ecdh;"));
    assert!(stealth_mod.contains("pub mod kdf;"));
    assert!(stealth_mod.contains("pub mod zkpack;"));
    assert!(!stealth_mod.contains("#[path = \"facade_"));
}

#[test]
fn test_leaf_paths_canonical() {
    let tx_mod = read_workspace_file("crates/z00z_wallets/src/tx/mod.rs");
    let claim_tx = read_workspace_file("crates/z00z_wallets/src/tx/claim_tx.rs");
    let stealth_crypto = read_workspace_file("crates/z00z_wallets/src/stealth/crypto.rs");
    let stealth_output = read_workspace_file("crates/z00z_wallets/src/stealth/output.rs");
    let wallet_root = read_workspace_file("crates/z00z_wallets/src/wallet/core.rs");
    let wallet_errors = read_workspace_file("crates/z00z_wallets/src/wallet/errors.rs");
    let wallet_persistence = read_workspace_file("crates/z00z_wallets/src/wallet/persistence.rs");
    let wallet_stub_defaults =
        read_workspace_file("crates/z00z_wallets/src/wallet/stub_defaults.rs");
    let wallet_backup = read_workspace_file("crates/z00z_wallets/src/backup/wallet_backup.rs");
    let backup_exporter =
        read_workspace_file("crates/z00z_wallets/src/backup/backup_exporter_impl.rs");
    let wallet_guide = read_workspace_file("crates/z00z_wallets/docs/WALLET-GUIDE.md");

    for live_path in [
        "crates/z00z_wallets/docs/WALLET-GUIDE.md",
        "crates/z00z_wallets/src/backup/backup_importer_impl.rs",
        "crates/z00z_wallets/src/backup/test_backup_importer_impl.rs",
        "crates/z00z_wallets/src/backup/wallet_backup_kdf.rs",
        "crates/z00z_wallets/src/backup/backup_exporter_verify.rs",
        "crates/z00z_wallets/src/backup/test_backup_exporter_suite.rs",
        "crates/z00z_wallets/src/backup/wallet_backup.rs",
        "crates/z00z_wallets/src/backup/test_wallet_backup.rs",
        "crates/z00z_wallets/src/chain/broadcast_impl.rs",
        "crates/z00z_wallets/src/claim/registry.rs",
        "crates/z00z_wallets/src/claim/test_nullifier_store.rs",
        "crates/z00z_wallets/src/stealth/crypto.rs",
        "crates/z00z_wallets/src/stealth/ecdh_core.rs",
        "crates/z00z_wallets/src/stealth/ecdh_validation.rs",
        "crates/z00z_wallets/src/stealth/encoding.rs",
        "crates/z00z_wallets/src/stealth/ephemeral.rs",
        "crates/z00z_wallets/src/stealth/output.rs",
        "crates/z00z_wallets/src/stealth/output_build.rs",
        "crates/z00z_wallets/src/stealth/test_output.rs",
        "crates/z00z_wallets/src/stealth/test_output_edge_cases.rs",
        "crates/z00z_wallets/src/stealth/zkpack.rs",
        "crates/z00z_wallets/src/stealth/test_zkpack.rs",
        "crates/z00z_wallets/src/tx/asset_selector.rs",
        "crates/z00z_wallets/src/tx/asset_selector_multi.rs",
        "crates/z00z_wallets/src/tx/claim_tx.rs",
        "crates/z00z_wallets/src/tx/claim_tx_digest.rs",
        "crates/z00z_wallets/src/tx/claim_tx_statement.rs",
        "crates/z00z_wallets/src/tx/claim_tx_verify.rs",
        "crates/z00z_wallets/src/tx/claim_tx_verify_proof.rs",
        "crates/z00z_wallets/src/tx/fee_estimator.rs",
        "crates/z00z_wallets/src/tx/state_update.rs",
        "crates/z00z_wallets/src/tx/test_asset_selector.rs",
        "crates/z00z_wallets/src/tx/test_asset_selector_multi.rs",
        "crates/z00z_wallets/src/tx/test_claim_tx.rs",
        "crates/z00z_wallets/src/tx/test_fee_estimator.rs",
        "crates/z00z_wallets/src/tx/test_state_update.rs",
        "crates/z00z_wallets/src/tx/test_tx_verifier.rs",
        "crates/z00z_wallets/src/tx/tx_verifier.rs",
        "crates/z00z_wallets/src/tx/tx_verifier_decode.rs",
        "crates/z00z_wallets/src/wallet/entity.rs",
        "crates/z00z_wallets/src/wallet/entity_asset_import.rs",
        "crates/z00z_wallets/src/wallet/entity_constructor.rs",
        "crates/z00z_wallets/src/wallet/entity_core.rs",
        "crates/z00z_wallets/src/wallet/errors_impls.rs",
        "crates/z00z_wallets/src/wallet/errors_types.rs",
        "crates/z00z_wallets/src/wallet/test_errors_suite.rs",
        "crates/z00z_wallets/src/wallet/persistence_types.rs",
        "crates/z00z_wallets/src/wallet/stub_defaults_asset.rs",
        "crates/z00z_wallets/src/wallet/stub_defaults_backup.rs",
        "crates/z00z_wallets/src/wallet/stub_defaults_tx.rs",
        "crates/z00z_wallets/src/wallet/stub_defaults_wallet.rs",
    ] {
        assert!(
            workspace_path_exists(live_path),
            "expected canonical wallet leaf path: {live_path}"
        );
    }

    for removed_path in [
        "crates/z00z_wallets/src/backup/backup_importer_impl/mod.rs",
        "crates/z00z_wallets/src/backup/backup_importer_impl/test_mod.rs",
        "crates/z00z_wallets/src/backup/crypto/wallet_backup_kdf.rs",
        "crates/z00z_wallets/src/backup/export/backup_exporter_verify.rs",
        "crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs",
        "crates/z00z_wallets/src/backup/wallet_backup/mod.rs",
        "crates/z00z_wallets/src/backup/wallet_backup/test_mod.rs",
        "crates/z00z_wallets/src/chain/broadcast/broadcast_impl.rs",
        "crates/z00z_wallets/src/claim/nullifier_store/test_mod.rs",
        "crates/z00z_wallets/src/claim/claim_registry.rs",
        "crates/z00z_wallets/src/stealth/crypto/ecdh.rs",
        "crates/z00z_wallets/src/stealth/crypto/ecdh_validation.rs",
        "crates/z00z_wallets/src/stealth/crypto/encoding.rs",
        "crates/z00z_wallets/src/stealth/crypto/ephemeral.rs",
        "crates/z00z_wallets/src/stealth/crypto/mod.rs",
        "crates/z00z_wallets/src/stealth/output/mod.rs",
        "crates/z00z_wallets/src/stealth/output/output_build.rs",
        "crates/z00z_wallets/src/stealth/output/tests/test_extra.rs",
        "crates/z00z_wallets/src/stealth/output/tests/test_mod.rs",
        "crates/z00z_wallets/src/stealth/test_output_extra.rs",
        "crates/z00z_wallets/src/stealth/zkpack/mod.rs",
        "crates/z00z_wallets/src/stealth/zkpack/test_mod.rs",
        "crates/z00z_wallets/src/tx/asset_selector/mod.rs",
        "crates/z00z_wallets/src/tx/asset_selector/multi.rs",
        "crates/z00z_wallets/src/tx/asset_selector/multi/test_mod.rs",
        "crates/z00z_wallets/src/tx/asset_selector/test_mod.rs",
        "crates/z00z_wallets/src/tx/claim_helpers.rs",
        "crates/z00z_wallets/src/tx/claim/claim_tx_helpers.rs",
        "crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl.rs",
        "crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs",
        "crates/z00z_wallets/src/tx/claim_tx/mod.rs",
        "crates/z00z_wallets/src/tx/claim_tx/test_claim_tx.rs",
        "crates/z00z_wallets/src/tx/fee_estimator/mod.rs",
        "crates/z00z_wallets/src/tx/fee_estimator/test_mod.rs",
        "crates/z00z_wallets/src/tx/state_update/mod.rs",
        "crates/z00z_wallets/src/tx/state_update/test_mod.rs",
        "crates/z00z_wallets/src/tx/tx_verifier/mod.rs",
        "crates/z00z_wallets/src/tx/tx_verifier/test_mod.rs",
        "crates/z00z_wallets/src/tx/verify/tx_verifier_helpers.rs",
        "crates/z00z_wallets/src/wallet/WALLET-GUIDE.md",
        "crates/z00z_wallets/src/wallet/entity/wallet_entity.rs",
        "crates/z00z_wallets/src/wallet/entity/wallet_entity_asset_api.rs",
        "crates/z00z_wallets/src/wallet/entity/wallet_entity_constructor.rs",
        "crates/z00z_wallets/src/wallet/entity/wallet_entity_core.rs",
        "crates/z00z_wallets/src/wallet/entity/wallet_entity_wallet_api.rs",
        "crates/z00z_wallets/src/wallet/errors/errors_impls.rs",
        "crates/z00z_wallets/src/wallet/errors/errors_types.rs",
        "crates/z00z_wallets/src/wallet/errors/test_errors_suite.rs",
        "crates/z00z_wallets/src/wallet/persistence/persistence_types.rs",
    ] {
        assert!(
            !workspace_path_exists(removed_path),
            "unexpected nested wallet leaf path: {removed_path}"
        );
    }

    assert!(tx_mod.contains("mod claim_tx_digest;"));
    assert!(!tx_mod.contains("mod claim_helpers;"));
    assert!(claim_tx.contains("include!(\"claim_tx_statement.rs\");"));
    assert!(claim_tx.contains("include!(\"claim_tx_verify.rs\");"));
    assert!(!claim_tx.contains("../claim/claim_tx_helpers.rs"));
    assert!(!claim_tx.contains("../claim/claim_tx_verify.rs"));
    assert!(stealth_crypto.contains("#[path = \"ecdh_core.rs\"]"));
    assert!(stealth_output.contains("#[path = \"test_output.rs\"]"));
    assert!(wallet_root.contains("include!(\"entity.rs\");"));
    assert!(!wallet_root.contains("include!(\"entity/wallet_entity.rs\");"));
    assert!(wallet_errors.contains("include!(\"errors_types.rs\");"));
    assert!(wallet_errors.contains("include!(\"errors_impls.rs\");"));
    assert!(!wallet_errors.contains("include!(\"errors/errors_types.rs\");"));
    assert!(wallet_persistence.contains("include!(\"persistence_types.rs\");"));
    assert!(!wallet_persistence.contains("include!(\"persistence/persistence_types.rs\");"));
    assert!(wallet_stub_defaults.contains("include!(\"stub_defaults_wallet.rs\");"));
    assert!(!wallet_stub_defaults.contains("include!(\"responses/stub_defaults_wallet.rs\");"));
    assert!(wallet_backup.contains("include!(\"wallet_backup_kdf.rs\");"));
    assert!(!wallet_backup.contains("include!(\"../crypto/wallet_backup_kdf.rs\");"));
    assert!(backup_exporter.contains("include!(\"backup_exporter_verify.rs\");"));
    assert!(!backup_exporter.contains("include!(\"export/backup_exporter_verify.rs\");"));
    assert!(wallet_guide.contains("WalletExportPack"));
}

fn assert_domain_golden_owner_canonical() {
    let domain_tests = read_workspace_file("crates/z00z_wallets/src/domains/test_definitions.rs");

    for removed_path in [
        "crates/z00z_wallets/docs/domains_snapshot.txt",
        "crates/z00z_wallets/src/domains/domains_snapshot.txt",
    ] {
        assert!(
            !workspace_path_exists(removed_path),
            "unexpected duplicate domain golden: {removed_path}"
        );
    }

    assert!(domain_tests.contains("const FROZEN_DOMAIN_SNAPSHOT: &str"));
    assert!(domain_tests.contains("snapshot_lines_from_str(FROZEN_DOMAIN_SNAPSHOT)"));
    assert!(!domain_tests.contains("include_str!(\"../../docs/domains_snapshot.txt\")"));
}

#[test]
fn test_domain_golden_owner_canonical() {
    assert_domain_golden_owner_canonical();
}

// Exact provenance for the deleted archive previously retained only to prove
// the egui tree relocation. Values are derived from the HEAD blob and its
// ordered `tar -tzf` output; no runtime code consumes the archive.
const FROZEN_EGUI_ARCHIVE_METADATA: &str = r#"
git_blob_oid=906761239cb3ec4f8ba80a9ee8b4e96ed88ed81c
blob_len=6528992
blob_sha256=3319d481df7cedd46275eb2d2fe276c563c25e77b32e6602c77c781df8386fd9
member_count=150
member_list_sha256=2b5d20f0fb572bd128cafe94fd910099adea79894af704727f81f36bb448d577
"#;

const REMOVED_EGUI_ARCHIVE_PATHS: [&str; 5] = [
    "crates/z00z_wallets/docs/egui_views.tar.gz",
    "crates/z00z_wallets/src/egui_views/egui_views.tar.gz",
    "crates/z00z_wallets/src/egui_views/ascii-mockups.tar.gz",
    "crates/z00z_wallets/src/egui_views/ref-docs.tar.gz",
    "crates/z00z_wallets/src/egui_views/themes.tar.gz",
];

fn frozen_egui_metadata_value(key: &str) -> &'static str {
    FROZEN_EGUI_ARCHIVE_METADATA
        .lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(candidate, value)| (candidate == key).then_some(value))
        .unwrap_or_else(|| panic!("missing frozen egui archive metadata: {key}"))
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn assert_egui_archive_owner_canonical() {
    for removed_path in REMOVED_EGUI_ARCHIVE_PATHS {
        assert!(
            !workspace_path_exists(removed_path),
            "unexpected legacy egui archive path: {removed_path}"
        );
    }

    let fields = FROZEN_EGUI_ARCHIVE_METADATA
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let unique_keys = fields
        .iter()
        .filter_map(|line| line.split_once('=').map(|(key, _)| key))
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(fields.len(), 5, "unexpected egui archive metadata shape");
    assert_eq!(unique_keys.len(), fields.len(), "duplicate metadata key");
    assert!(is_lower_hex(frozen_egui_metadata_value("git_blob_oid"), 40));
    assert!(is_lower_hex(frozen_egui_metadata_value("blob_sha256"), 64));
    assert!(is_lower_hex(
        frozen_egui_metadata_value("member_list_sha256"),
        64
    ));
    assert!(frozen_egui_metadata_value("blob_len")
        .parse::<u64>()
        .is_ok_and(|value| value > 0));
    assert!(frozen_egui_metadata_value("member_count")
        .parse::<usize>()
        .is_ok_and(|value| value > 0));
}

#[test]
fn test_egui_archive_owner_canonical() {
    assert_egui_archive_owner_canonical();
}

#[test]
fn test_tree_paths_canonical() {
    let definitions = read_workspace_file("crates/z00z_wallets/src/domains/definitions.rs");
    let hashing = read_workspace_file("crates/z00z_wallets/src/domains/hashing.rs");
    let wallet_runtime_config =
        read_workspace_file("crates/z00z_wallets/src/services/wallet_runtime_config.rs");
    let tab_registry = read_workspace_file("crates/z00z_wallets/src/egui_views/tab_registry.rs");

    for live_path in [
        "crates/z00z_wallets/src/domains/test_definitions.rs",
        "crates/z00z_wallets/src/domains/test_hashing.rs",
        "crates/z00z_wallets/src/services/test_wallet_runtime_config_suite.rs",
        "crates/z00z_wallets/src/egui_views/wallet_tab_staking.rs",
    ] {
        assert!(
            workspace_path_exists(live_path),
            "expected final canonical path: {live_path}"
        );
    }

    for removed_path in [
        "crates/z00z_wallets/src/domains/definitions/test_mod.rs",
        "crates/z00z_wallets/src/domains/hashing/test_mod.rs",
        "crates/z00z_wallets/docs/domains_snapshot.txt",
        "crates/z00z_wallets/src/domains/domains_snapshot.txt",
        "crates/z00z_wallets/src/egui_views/app_settings_tab_2.rs",
        "crates/z00z_wallets/src/egui_views/wallet_tab_stacking.rs",
        "crates/z00z_wallets/src/egui_views/egui_config.yaml",
        "crates/z00z_wallets/src/egui_views/themes/_border_patterns.yaml",
        "crates/z00z_wallets/src/egui_views/themes/_border_semantics.yaml",
        "crates/z00z_wallets/src/egui_views/themes/_icon_system.yaml",
        "crates/z00z_wallets/src/egui_views/themes/golden_daylight.yaml",
        "crates/z00z_wallets/src/egui_views/themes/golden_twilight.yaml",
    ] {
        assert!(
            !workspace_path_exists(removed_path),
            "unexpected pre-closeout path: {removed_path}"
        );
    }

    assert!(definitions.contains("#[path = \"test_definitions.rs\"]"));
    assert!(!definitions.contains("#[path = \"definitions/test_mod.rs\"]"));
    assert!(hashing.contains("#[path = \"test_hashing.rs\"]"));
    assert!(!hashing.contains("#[path = \"hashing/test_mod.rs\"]"));
    assert!(wallet_runtime_config.contains("include!(\"test_wallet_runtime_config_suite.rs\");"));
    assert!(tab_registry.contains("module_name: \"wallet_tab_staking\""));
    assert!(!tab_registry.contains("module_name: \"wallet_tab_stacking\""));
    assert_domain_golden_owner_canonical();
    assert_egui_archive_owner_canonical();
}

#[test]
fn test_scanner_path_internal() {
    let scan_mod = read_workspace_file("crates/z00z_wallets/src/receiver/asset_scan.rs");
    let tests_root = workspace_root().join("crates/z00z_wallets/tests");
    let mut files = Vec::new();
    collect_test_files(&tests_root, &mut files);
    let mut violations = Vec::new();

    assert!(scan_mod.contains("mod wallet_asset_scanner;"));
    assert!(!scan_mod.contains("pub mod wallet_asset_scanner;"));

    for path in files {
        let name = path
            .file_name()
            .and_then(|item| item.to_str())
            .unwrap_or("");
        if name == "test_e2e_public_path.rs" || name == "test_rename_guards.rs" {
            continue;
        }

        let text = read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if text.contains("receiver::asset_scan::wallet_asset_scanner::")
            || text.contains("asset_scan::wallet_asset_scanner::")
        {
            violations.push(path.display().to_string());
        }
    }

    assert!(
        violations.is_empty(),
        "unexpected internal wallet_asset_scanner path usage:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_test_file_prefix_guard() {
    let workspace = workspace_root();
    let root = workspace.join("crates");
    let mut files = Vec::new();
    collect_test_files(&root, &mut files);

    let mut violations = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|item| item.to_str())
            .unwrap_or("<invalid>");
        let relative = path
            .strip_prefix(&workspace)
            .map(|item| item.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned());
        if relative.starts_with("crates/z00z_crypto/tari/") {
            continue;
        }
        let has_allowed_prefix = name.starts_with("test_") || name.starts_with("generated_kani_");
        let is_allowed_support_module = matches!(
            relative.as_str(),
            "crates/z00z_simulator/tests/scenario_1/main.rs"
                | "crates/z00z_simulator/tests/scenario_1/claim_pkg_crypto.rs"
                | "crates/z00z_simulator/tests/scenario_1/output_roots.rs"
                | "crates/z00z_simulator/tests/scenario_1/stage4_bob.rs"
                | "crates/z00z_simulator/tests/scenario_1/stage4_paths.rs"
                | "crates/z00z_simulator/tests/scenario_1/stage4_root.rs"
        );
        if !has_allowed_prefix && !is_allowed_support_module {
            violations.push(format!("missing prefix: {}", path.display()));
        }
        if name.to_ascii_lowercase().contains("phase") {
            violations.push(format!("phase in test file name: {}", path.display()));
        }
    }

    assert!(
        violations.is_empty(),
        "unexpected test-file naming violations:\n{}",
        violations.join("\n")
    );
}
