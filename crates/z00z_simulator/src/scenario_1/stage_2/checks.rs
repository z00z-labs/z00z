use super::{
    build_logged_transport, compute_wallet_file_id, create_dir_all, json, path_exists, push_log,
    save_json, ActorRun, Arc, Bip44Path, Codec, JsonCodec, Path, PersistWalletId, RpcTransport,
    SafePassword, SimContext, WalletService, WalletSource,
};
#[cfg(feature = "wallet_debug_tools")]
use super::{norm_seed, read_seed_md};
use std::thread::sleep;
use std::time::Duration;

const RESTART_WALLET_SOURCE_RETRIES: u32 = 200;
const RESTART_WALLET_SOURCE_WAIT_MS: u64 = 50;

pub(crate) async fn run_export_roundtrip(
    ctx: &SimContext,
    transport: &impl RpcTransport,
    stage_id: u32,
    logs: &mut Vec<String>,
    out: &Path,
    rpc_log_path: &Path,
    wallet_net: &str,
    wallet_chain: &str,
    actor_runs: &[ActorRun],
    debug_secrets_table: Option<&Path>,
) -> Result<(), String> {
    let bob_run = actor_runs
        .iter()
        .find(|actor| actor.name == "bob")
        .ok_or_else(|| "bob runtime not found".to_string())?;
    let export_resp = transport
        .call(
            "app.wallet.export_wallet",
            json!({
                "wallet_id": bob_run.wallet_id,
                "password": bob_run.password,
            }),
        )
        .await
        .map_err(|e| format!("export_wallet(bob) RPC: {e}"))?;
    let payload = export_resp["encrypted_payload"].clone();

    let export_import_dir = out.join("wallets_export_import");
    create_dir_all(&export_import_dir).map_err(|e| e.to_string())?;
    let payload_file = export_import_dir.join("export_wallet_encrypted_payload.json");
    save_json(&payload_file, &payload).map_err(|e| e.to_string())?;
    push_log(
        logs,
        stage_id,
        "S2-17",
        "write_export_payload",
        "ok",
        &payload_file.to_string_lossy(),
    )?;

    let payload_data = String::from_utf8(JsonCodec.serialize(&payload).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let (_svc_import, import_transport) =
        build_logged_transport(ctx, &export_import_dir, rpc_log_path)?;

    let import_resp = import_transport
        .call(
            "app.wallet.import_wallet",
            json!({
                "data": payload_data,
                "password": bob_run.password,
                "name": "Imported Bob",
            }),
        )
        .await
        .map_err(|e| format!("import_wallet(bob) RPC: {e}"))?;
    let imported_id = import_resp["wallet_id"]
        .as_str()
        .ok_or_else(|| "import_wallet response missing wallet_id".to_string())?;
    if imported_id != bob_run.wallet_id {
        return Err(format!(
            "export/import wallet_id mismatch: {} != {}",
            imported_id, bob_run.wallet_id
        ));
    }

    let imported_file_hash = compute_wallet_file_id(imported_id);
    let imported_wlt_name = format!("wallet_{}.wlt", super::hex_str(&imported_file_hash[..8]));
    let imported_wlt_path = export_import_dir.join(&imported_wlt_name);
    if !path_exists(&imported_wlt_path).map_err(|e| e.to_string())? {
        return Err(format!(
            "imported .wlt file is missing: {}",
            imported_wlt_path.display()
        ));
    }

    let verify_wallet_svc = WalletService::with_output_dir(export_import_dir.clone());
    verify_wallet_svc
        .open_wallet_source(WalletSource::Path {
            path: imported_wlt_path.to_string_lossy().to_string(),
        })
        .await
        .map_err(|e| format!("open_wallet_source imported bob wallet: {e}"))?;

    let _ = (wallet_net, wallet_chain);

    #[cfg(feature = "wallet_debug_tools")]
    {
        let debug_secrets_table = debug_secrets_table.ok_or_else(|| {
            "wallet_debug_tools enabled but stage-2 private debug artifact path is missing"
                .to_string()
        })?;
        let md_seed = read_seed_md(debug_secrets_table, "bob")?;
        if norm_seed(&md_seed) != norm_seed(&bob_run.seed_phrase) {
            return Err(
                "S2-17: seed_phrase mismatch between the private stage-2 debug artifact and Bob's runtime seed"
                    .to_string(),
            );
        }

        push_log(
            logs,
            stage_id,
            "S2-17",
            "debug_seed_contract",
            "ok",
            &debug_secrets_table.to_string_lossy(),
        )?;
        push_log(
            logs,
            stage_id,
            "S2-17",
            "seed_phrase_consistency",
            "ok",
            "bob seed phrase matches the private stage-2 debug artifact",
        )?;
    }

    #[cfg(not(feature = "wallet_debug_tools"))]
    let _ = debug_secrets_table;

    push_log(
        logs,
        stage_id,
        "S2-17",
        "import_wlt_verified",
        "ok",
        &imported_wlt_path.to_string_lossy(),
    )?;
    push_log(
        logs,
        stage_id,
        "S2-17",
        "export_import_roundtrip",
        "ok",
        &format!("wallet_id={imported_id}"),
    )?;

    Ok(())
}

pub(crate) async fn run_restart_check(
    wallet_svc: &Arc<WalletService>,
    stage_id: u32,
    logs: &mut Vec<String>,
    alice_run: &ActorRun,
    alice_wlt_path: &Path,
    wallets_dir: &Path,
    restart_source_bytes: &[u8],
) -> Result<(), String> {
    let alice_id = PersistWalletId(alice_run.wallet_id.clone());
    let alice_pw = SafePassword::from(alice_run.password.as_str());
    let canonical_alice_wlt_path =
        super::transport::wallet_source_path(wallets_dir, alice_run.wallet_id.as_str());
    wallet_svc
        .ensure_wallet_session(&alice_id, &alice_pw)
        .await
        .map_err(|e| format!("unlock_wallet(alice before restart service): {e}"))?;
    let before_pk = wallet_svc
        .derive_public_key_for_path(&alice_id, Bip44Path::payment(0).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| format!("derive_public_key_for_path(alice before restart): {e}"))?;
    wallet_svc
        .lock_wallet(&alice_id)
        .await
        .map_err(|e| format!("lock_wallet(alice before restart): {e}"))?;

    let wallet_svc_restart = Arc::new(WalletService::with_output_dir(wallets_dir.to_path_buf()));
    match wait_for_wallet_source_path(alice_wlt_path, &canonical_alice_wlt_path, wallets_dir) {
        Ok(restart_source_path) => {
            open_restart_source(
                &wallet_svc_restart,
                &alice_id,
                WalletSource::Path {
                    path: restart_source_path.to_string_lossy().to_string(),
                },
                "path",
            )
            .await?;
        }
        Err(source_err) => {
            open_restart_source(
                &wallet_svc_restart,
                &alice_id,
                WalletSource::Bytes {
                    bytes: restart_source_bytes.to_vec(),
                },
                "bytes",
            )
            .await
            .map_err(|err| format!("{err}; source_error: {source_err}"))?;
            if !path_exists(&canonical_alice_wlt_path).map_err(|e| e.to_string())? {
                return Err(format!(
                    "alice restart bytes import did not recreate canonical path: {}; source_error: {}",
                    canonical_alice_wlt_path.display(),
                    source_err
                ));
            }
        }
    }
    wallet_svc_restart
        .ensure_wallet_session(&alice_id, &alice_pw)
        .await
        .map_err(|e| format!("unlock_wallet(alice after restart service): {e}"))?;
    let after_pk = wallet_svc_restart
        .derive_public_key_for_path(&alice_id, Bip44Path::payment(0).map_err(|e| e.to_string())?)
        .await
        .map_err(|e| format!("derive_public_key_for_path(alice after restart): {e}"))?;
    if before_pk != after_pk {
        return Err(format!(
            "restart determinism mismatch: before={} after={}",
            super::hex_str(&before_pk),
            super::hex_str(&after_pk)
        ));
    }

    push_log(
        logs,
        stage_id,
        "S2-18",
        "restart_determinism",
        "ok",
        &format!("pubkey={}", super::hex_str(&after_pk)),
    )?;

    Ok(())
}

async fn open_restart_source(
    wallet_svc: &Arc<WalletService>,
    wallet_id: &PersistWalletId,
    source: WalletSource,
    lane: &str,
) -> Result<(), String> {
    let discovery = wallet_svc
        .open_wallet_source(source)
        .await
        .map_err(|e| format!("open_wallet_source(alice restart {lane}): {e}"))?;
    if discovery.wallet_id != *wallet_id {
        return Err(format!(
            "alice restart source wallet_id mismatch on {lane}: {} != {}",
            discovery.wallet_id.0, wallet_id.0
        ));
    }
    Ok(())
}

fn wait_for_wallet_source_path<'a>(
    actor_wlt_path: &'a Path,
    canonical_wlt_path: &'a Path,
    wallets_dir: &Path,
) -> Result<&'a Path, String> {
    for attempt in 0..=RESTART_WALLET_SOURCE_RETRIES {
        if path_exists(actor_wlt_path).map_err(|e| e.to_string())? {
            return Ok(actor_wlt_path);
        }
        if path_exists(canonical_wlt_path).map_err(|e| e.to_string())? {
            return Ok(canonical_wlt_path);
        }
        if attempt < RESTART_WALLET_SOURCE_RETRIES {
            sleep(Duration::from_millis(RESTART_WALLET_SOURCE_WAIT_MS));
        }
    }

    Err(format!(
        "alice restart source missing: actor_path={} canonical_path={} dir_entries=[{}]",
        actor_wlt_path.display(),
        canonical_wlt_path.display(),
        read_wallet_dir_entries(wallets_dir)
    ))
}

fn read_wallet_dir_entries(wallets_dir: &Path) -> String {
    let mut entries = Vec::new();
    if let Ok(dir_entries) = z00z_utils::io::read_dir(wallets_dir) {
        for entry in dir_entries {
            if let Some(name) = entry.file_name().and_then(|value| value.to_str()) {
                entries.push(name.to_string());
            }
        }
        entries.sort();
    }
    entries.join(",")
}

pub(crate) async fn run_lock_check(
    transport: &impl RpcTransport,
    stage_id: u32,
    logs: &mut Vec<String>,
    alice_run: &ActorRun,
) -> Result<(), String> {
    transport
        .call(
            "wallet.lifecycle.on_event",
            json!({"event": "backgrounded"}),
        )
        .await
        .map_err(|e| format!("lifecycle event RPC: {e}"))?;

    let list_result = transport
        .call(
            "wallet.key.list_receivers",
            json!({
                "session": alice_run.session,
                "limit": 1,
                "cursor": z00z_utils::codec::Value::Null,
                "filter": z00z_utils::codec::Value::Null,
            }),
        )
        .await;

    if list_result.is_ok() {
        return Err("S2-16: list_receivers should fail after lifecycle lock (backgrounded), but it succeeded".to_string());
    }

    push_log(
        logs,
        stage_id,
        "S2-16",
        "lifecycle_lock",
        "ok",
        "list_receivers correctly denied after backgrounded event",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario_1::stage_2::actors::actor_password;
    use crate::scenario_1::support::{fixture_cache, scenario_support, stage_runner_support};
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use z00z_utils::codec::Value;
    use z00z_utils::io::{load_json, read_file, remove_file};

    fn stage2_shared_root() -> &'static PathBuf {
        static ROOT: OnceLock<PathBuf> = OnceLock::new();
        ROOT.get_or_init(|| {
            fixture_cache::ensure_shared_case("wallet_stage2_shared_v1", |base| {
                let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |_| {});
                let _ctx = stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1, 2]);
                assert!(
                    out.join("stage_2_snapshot.json").exists(),
                    "shared stage2 fixture must contain stage_2_snapshot.json"
                );
            })
        })
    }

    fn read_wallet_id(out: &Path, actor_name: &str) -> PersistWalletId {
        let path = out.join("keys").join(format!("{actor_name}_keys.json"));
        let value: Value = load_json(&path).expect("load actor keys json");
        let wallet_id = value["wallet_id"]
            .as_str()
            .expect("wallet_id missing in actor keys json");
        if wallet_id.starts_with("wallet_") {
            PersistWalletId(wallet_id.to_string())
        } else {
            PersistWalletId(format!("wallet_{wallet_id}"))
        }
    }

    fn make_actor_run(wallet_id: &PersistWalletId, password: &str) -> ActorRun {
        ActorRun {
            name: "alice".to_string(),
            password: password.to_string(),
            wallet_id: wallet_id.0.clone(),
            session: Value::Null,
            seed_phrase: String::new(),
            receiver_secret_hex: String::new(),
            owner_handle: String::new(),
            view_pk: String::new(),
            identity_pk: String::new(),
            receiver_ids: Vec::new(),
        }
    }

    #[test]
    fn restart_check_restores_wlt_bytes() {
        let local_root = fixture_cache::ensure_case("stage2_restart_bytes_local", |base| {
            fixture_cache::copy_tree(stage2_shared_root(), base);
        });
        let wallets_dir = local_root.join("outputs/scenario_1/wallets");
        let out = local_root.join("outputs/scenario_1");
        let alice_wallet_id = read_wallet_id(&out, "alice");
        let alice_wlt =
            super::super::transport::wallet_source_path(&wallets_dir, &alice_wallet_id.0);
        let restart_source_bytes = read_file(&alice_wlt).expect("read alice restart source bytes");
        let wallet_svc = Arc::new(WalletService::with_output_dir(wallets_dir.clone()));
        let alice_password = actor_password("alice").expect("default alice password");
        let alice_pw = SafePassword::from(alice_password.as_str());

        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async {
            wallet_svc
                .open_wallet_source(WalletSource::Path {
                    path: alice_wlt.to_string_lossy().to_string(),
                })
                .await
                .expect("open alice source before restart");
            wallet_svc
                .unlock_wallet_in_memory(&alice_wallet_id, &alice_pw)
                .await
                .expect("unlock alice before deleting source");

            remove_file(&alice_wlt).expect("delete alice wlt before restart check");
            assert!(
                !path_exists(&alice_wlt).expect("stat deleted alice wlt"),
                "alice wlt must be absent before restart fallback"
            );

            let mut logs = Vec::new();
            run_restart_check(
                &wallet_svc,
                2,
                &mut logs,
                &make_actor_run(&alice_wallet_id, &alice_password),
                &alice_wlt,
                &wallets_dir,
                &restart_source_bytes,
            )
            .await
            .expect("restart check with bytes fallback");

            assert!(
                path_exists(&alice_wlt).expect("stat recreated alice wlt"),
                "bytes fallback must recreate alice canonical wlt"
            );
            assert!(
                logs.iter()
                    .any(|row| row.contains("\"event\":\"restart_determinism\"")),
                "restart check must log restart_determinism"
            );
        });
    }
}
