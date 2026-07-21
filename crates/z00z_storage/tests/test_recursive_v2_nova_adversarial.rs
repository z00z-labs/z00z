//! Integration guard for Nova artifact and mutation evidence.

use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};

use z00z_utils::{io::atomic_write_file_private, os_hardening::apply_best_effort};
use zeroize::{Zeroize, Zeroizing};

const NOVA_OWNER: &str = include_str!("../src/checkpoint/nova.rs");
const CIRCUIT_SPEC_OWNER: &str = include_str!("../src/checkpoint/recursive_circuit.rs");
const TRACE_OWNER: &str = include_str!("../src/checkpoint/recursive_trace.rs");
const JMT_PROOF_BATCH_OWNER: &str = include_str!("../src/settlement/proof_batch.rs");
const CUTOVER_OWNER: &str = include_str!("../src/checkpoint/canonical_transition.rs");
const SECRET_CHILD_ENV: &str = "Z00Z_RECURSIVE_V2_SECRET_CHILD";
const SECRET_MODE_ENV: &str = "Z00Z_RECURSIVE_V2_SECRET_MODE";
const SECRET_CANARY_ENV: &str = "Z00Z_RECURSIVE_V2_SECRET_CANARY";
const SECRET_ARTIFACT_ENV: &str = "Z00Z_RECURSIVE_V2_SECRET_ARTIFACT";
const SECRET_CANCEL_ENV: &str = "Z00Z_RECURSIVE_V2_SECRET_CANCEL";
const SECRET_CANARY: &str = "z00z-recursive-process-canary-f24-9d3e71";

#[test]
fn test_artifacts_bind_full_identity() {
    for required in [
        "struct ProverMaterialHeaderV2",
        "struct VerifierBundleHeaderV2",
        "struct VerifierBundleSelectionV2",
        "expected_bundle_digest",
        "selected_sha_batch_width",
        "authority_generation",
        "activation_start_height",
        "activation_end_height",
        "source_revision_digest",
        "settlement/proof_batch.rs",
        "settlement/keys.rs",
        "settlement/identity.rs",
        "z00z_crypto/src/hash/policy.rs",
        "workspace-and-storage-manifests",
        "Nova curve/transcript/entropy dependency identity drifted",
        "test_nova_keccak_transcript_pinned",
        "pallas-ipa",
        "vesta-ipa",
        "decode_bincode",
        "canonical_payload != vk_payload",
    ] {
        assert!(
            NOVA_OWNER.contains(required),
            "missing strict Nova artifact gate: {required}"
        );
    }
}

#[test]
fn test_secret_buffers_stay_private() {
    for (owner_name, owner) in [
        ("recursive trace", TRACE_OWNER),
        ("JMT proof batch", JMT_PROOF_BATCH_OWNER),
    ] {
        for forbidden in [
            "eprintln!",
            "println!",
            "tracing::",
            "log_event(",
            ".debug(",
            ".trace(",
            ".info(",
            ".warn(",
            ".error(",
        ] {
            assert!(
                !owner.contains(forbidden),
                "{owner_name} secret owner gained a log/telemetry sink: {forbidden}",
            );
        }
    }

    assert!(TRACE_OWNER.contains("impl Drop for RecursiveTraceEventV2"));
    assert!(TRACE_OWNER.contains("self.payload.zeroize()"));
    assert!(JMT_PROOF_BATCH_OWNER.contains("impl Drop for JmtUpdateOpV2"));
    assert!(JMT_PROOF_BATCH_OWNER.contains("self.prior_value.zeroize()"));
    assert!(JMT_PROOF_BATCH_OWNER.contains("self.value.zeroize()"));

    // The sole cutover diagnostic is compiled only into tests and receives the
    // fixed backend error, never the manifest or opaque record fields.
    assert!(CUTOVER_OWNER.contains(
        "#[cfg(test)]\n                eprintln!(\"recursive V2 durable cutover rejected: {_error}\");"
    ));
}

fn secret_child() {
    let report = apply_best_effort();
    #[cfg(all(unix, not(target_os = "ios")))]
    assert!(
        report.core_dumps_disabled,
        "secret worker must fail closed when core dumps cannot be disabled: {:?}",
        report.notes
    );
    #[cfg(any(target_os = "linux", target_os = "android"))]
    assert!(
        report.non_dumpable,
        "secret worker must fail closed when non-dumpable mode cannot be set: {:?}",
        report.notes
    );

    let mode = std::env::var(SECRET_MODE_ENV).expect("secret worker mode");
    let artifact =
        PathBuf::from(std::env::var_os(SECRET_ARTIFACT_ENV).expect("secret worker artifact path"));
    let cancel = PathBuf::from(
        std::env::var_os(SECRET_CANCEL_ENV).expect("secret worker cancellation path"),
    );
    let mut canary = Zeroizing::new(
        std::env::var(SECRET_CANARY_ENV)
            .expect("secret worker canary")
            .into_bytes(),
    );
    assert_eq!(canary.as_slice(), SECRET_CANARY.as_bytes());

    match mode.as_str() {
        "success" => {
            canary.zeroize();
            assert!(canary.is_empty());
            atomic_write_file_private(artifact, b"status=success\n")
                .expect("write sanitized success artifact");
        }
        "failure" => {
            canary.zeroize();
            assert!(canary.is_empty());
            atomic_write_file_private(artifact, b"status=failure\n")
                .expect("write sanitized failure artifact");
            std::process::exit(23);
        }
        "panic" => {
            atomic_write_file_private(artifact, b"status=panic\n")
                .expect("write sanitized panic artifact");
            canary.zeroize();
            assert!(canary.is_empty());
            panic!("sanitized recursive V2 worker panic");
        }
        "cancel" => {
            atomic_write_file_private(&artifact, b"status=ready\n")
                .expect("write sanitized cancellation-ready artifact");
            let deadline = Instant::now() + Duration::from_secs(5);
            while !cancel.exists() {
                assert!(
                    Instant::now() < deadline,
                    "sanitized cancellation marker deadline exceeded"
                );
                std::thread::sleep(Duration::from_millis(5));
            }
            canary.zeroize();
            assert!(canary.is_empty());
            atomic_write_file_private(artifact, b"status=cancelled\n")
                .expect("write sanitized cancellation artifact");
        }
        "timeout" | "hard-kill" => {
            atomic_write_file_private(artifact, b"status=ready\n")
                .expect("write sanitized kill-ready artifact");
            loop {
                std::thread::sleep(Duration::from_millis(50));
            }
        }
        _ => panic!("unknown sanitized secret-worker mode"),
    }
}

fn spawn_secret_child(test_name: &str, mode: &str, root: &Path) -> Child {
    let artifact = root.join(format!("{mode}.status"));
    let cancel = root.join(format!("{mode}.cancel"));
    let mut command = Command::new(std::env::current_exe().expect("secret-worker executable"));
    command
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .env_clear()
        .env(SECRET_CHILD_ENV, "1")
        .env(SECRET_MODE_ENV, mode)
        .env(SECRET_CANARY_ENV, SECRET_CANARY)
        .env(SECRET_ARTIFACT_ENV, artifact)
        .env(SECRET_CANCEL_ENV, cancel)
        .env("RUST_BACKTRACE", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().expect("spawn bounded secret worker")
}

fn wait_for_artifact_or_exit(child: &mut Child, artifact: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if artifact.exists() {
            return;
        }
        assert!(
            child.try_wait().expect("poll secret worker").is_none(),
            "secret worker exited before publishing its sanitized ready artifact"
        );
        assert!(Instant::now() < deadline, "secret worker readiness timeout");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn assert_sanitized_output(mode: &str, output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains(SECRET_CANARY) && !stderr.contains(SECRET_CANARY),
        "{mode} leaked the secret canary: stdout={stdout} stderr={stderr}"
    );
}

fn assert_private_canary_free_artifacts(root: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let root_mode = root
            .metadata()
            .expect("secret-worker directory metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(root_mode, 0o700, "secret-worker directory must be private");
    }

    for entry in std::fs::read_dir(root).expect("scan secret-worker artifacts") {
        let entry = entry.expect("secret-worker artifact entry");
        let path = entry.path();
        assert!(path.is_file(), "unexpected nested secret-worker artifact");
        let name = entry.file_name().to_string_lossy().into_owned();
        assert!(
            !name.to_ascii_lowercase().starts_with("core"),
            "secret worker left a core artifact: {name}"
        );
        let bytes = std::fs::read(&path).expect("read secret-worker artifact");
        assert!(
            !bytes
                .windows(SECRET_CANARY.len())
                .any(|window| window == SECRET_CANARY.as_bytes()),
            "secret canary escaped into artifact {name}"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = path
                .metadata()
                .expect("secret-worker artifact metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                mode, 0o600,
                "secret-worker artifact must be private: {name}"
            );
        }
    }
}

#[test]
fn test_secret_process_outcomes() {
    if std::env::var_os(SECRET_CHILD_ENV).is_some() {
        secret_child();
        return;
    }

    let temp = tempfile::tempdir().expect("secret-worker private directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700))
            .expect("set private secret-worker directory permissions");
    }
    let current_thread = std::thread::current();
    let test_name = current_thread.name().expect("test harness name");

    for mode in ["success", "failure", "panic"] {
        let child = spawn_secret_child(test_name, mode, temp.path());
        let output = child.wait_with_output().expect("wait for secret worker");
        match mode {
            "success" => assert!(output.status.success()),
            "failure" => assert_eq!(output.status.code(), Some(23)),
            "panic" => assert!(!output.status.success()),
            _ => unreachable!(),
        }
        assert_sanitized_output(mode, &output);
    }

    let cancel_artifact = temp.path().join("cancel.status");
    let mut cancel_child = spawn_secret_child(test_name, "cancel", temp.path());
    wait_for_artifact_or_exit(&mut cancel_child, &cancel_artifact);
    atomic_write_file_private(temp.path().join("cancel.cancel"), b"cancel\n")
        .expect("publish private cancellation marker");
    let cancel_output = cancel_child
        .wait_with_output()
        .expect("wait for cancelled secret worker");
    assert!(cancel_output.status.success());
    assert_sanitized_output("cancel", &cancel_output);

    for mode in ["timeout", "hard-kill"] {
        let artifact = temp.path().join(format!("{mode}.status"));
        let mut child = spawn_secret_child(test_name, mode, temp.path());
        wait_for_artifact_or_exit(&mut child, &artifact);
        if mode == "timeout" {
            std::thread::sleep(Duration::from_millis(100));
        }
        child.kill().expect("terminate bounded secret worker");
        let output = child.wait_with_output().expect("reap killed secret worker");
        assert!(!output.status.success());
        assert_sanitized_output(mode, &output);
    }

    assert_private_canary_free_artifacts(temp.path());
}

#[test]
fn test_adversarial_evidence_remains_live() {
    for required in [
        "assert_bundle_rejected_early",
        "assert_prover_material_rejected",
        "test_hash_controls_reject_mutations",
        "test_jmt_machine_rejects_mutations",
        "test_successor_rejects_opcode_change",
        "a changed compressed body must fail before Nova verification",
        "require_expected_public_endpoint",
        "recompute_compressed_mixed_candidate",
        "complete mixed Model C",
        "selected invalid VK rejected at strict key decode",
        "validate_pinned_verifier_key_wire",
        "forbidden primary commitment identity",
        "zero/default primary blinding key",
        "swapped primary/secondary commitment generators",
        "attempted-invalid-subgroup-or-identity-bundle",
        "test_nova_clean_verifier_process",
        "a verifier-only process must never enter PublicParams::setup",
        "setup_path=forbidden",
        ".arg(\"--core=0\")",
    ] {
        assert!(
            NOVA_OWNER.contains(required),
            "missing adversarial Nova evidence owner: {required}"
        );
    }
    assert!(NOVA_OWNER.contains("spartan-snark-ipa"));
    assert!(!NOVA_OWNER.contains("spartan-ppsnark"));
    assert!(TRACE_OWNER.contains("impl Drop for RecursiveTraceEventV2"));
    assert!(TRACE_OWNER.contains("Zeroizing<Vec<u8>>"));
    assert!(TRACE_OWNER.contains("test_secret_canary_stays_redacted"));
    assert!(!TRACE_OWNER
        .contains("#[derive(Clone, Debug, PartialEq, Eq)]\npub struct RecursiveTraceEventV2"));
}

#[test]
fn test_a17_claim_stays_conditional() {
    // The exact EAGM/GZT/DL/compression premises are evidence-layer policy,
    // not a compiled source dependency.  Production binds the selected
    // circuit/security geometry and must never promote it to an unconditional
    // cumulative-security claim.
    for required in [
        "RecursiveCircuitSpecV2",
        "UNIQUENESS_CHALLENGE_BITS_V2",
        "UNIQUENESS_RO_QUERY_LOG2_V2",
        "profile_digest",
        "shape_digest",
    ] {
        assert!(
            CIRCUIT_SPEC_OWNER.contains(required),
            "missing compiled conditional-security binding: {required}"
        );
    }
    assert!(!NOVA_OWNER.contains("unconditional 128-bit cumulative"));
    assert!(!CIRCUIT_SPEC_OWNER.contains("unconditional 128-bit cumulative"));
}
