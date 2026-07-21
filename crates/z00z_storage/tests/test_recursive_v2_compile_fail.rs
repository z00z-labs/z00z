use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

fn newest_storage_rlib(deps: &Path) -> PathBuf {
    fs::read_dir(deps)
        .expect("target dependency directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("libz00z_storage-") && name.ends_with(".rlib"))
        })
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        })
        .expect("compiled z00z_storage rlib")
}

#[test]
fn recursive_v2_typestate_and_diagnostic_internals_are_not_downstream_constructible() {
    let deps = std::env::current_exe()
        .expect("current integration test executable")
        .parent()
        .expect("target dependency directory")
        .to_path_buf();
    let rlib = newest_storage_rlib(&deps);
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui/recursive_v2_private_stages.rs");
    let output = tempfile::tempdir().expect("compile-fail output directory");
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let result = Command::new(rustc)
        .arg("--edition=2021")
        .arg("--crate-type=bin")
        .arg("--emit=metadata")
        .arg("--extern")
        .arg(format!("z00z_storage={}", rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("-o")
        .arg(output.path().join("recursive_v2_private_stages.rmeta"))
        .arg(&fixture)
        .output()
        .expect("run rustc compile-fail fixture");

    assert!(
        !result.status.success(),
        "private typestate fixture compiled"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    for forbidden in [
        "DiagnosticSingleStepEnvelopeV2",
        "LiveGateStageV2",
        "PostwriteVerifiedV2",
        "PreparedReceiptV2",
        "RecursiveNovaStepInputV2",
    ] {
        assert!(
            stderr.contains(forbidden),
            "compiler did not reject {forbidden} by name:\n{stderr}"
        );
    }
}
