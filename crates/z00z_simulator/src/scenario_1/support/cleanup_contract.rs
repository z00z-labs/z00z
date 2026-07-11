use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
};

use z00z_utils::io::{path_exists, read_dir, read_to_string};

use crate::scenario_1::runner;

use super::{fixture_cache, scenario_support};

const READY_FILE: &str = ".ready";
const FINGERPRINT_FILE: &str = ".fingerprint";
const CONTENT_FINGERPRINT_FILE: &str = ".content-fingerprint";
const SCOPE_FINGERPRINT_FILE: &str = ".scope-fingerprint";
const ROOT_MARK_FILE: &str = ".managed-root-fingerprint";
const SHARED_SCOPE_DIR: &str = "shared";
const SHARED_PRECISE_SCOPE_DIR: &str = "shared_precise";
const RUN_LOCKS_DIR: &str = "run_locks";
const ADHOC_DIR: &str = "adhoc";

pub fn emit_repo_cache_paths(repo_root: &Path, mut out: impl Write) -> Result<(), String> {
    for path in repo_cache_paths(repo_root)? {
        out.write_all(path.as_os_str().as_encoded_bytes())
            .map_err(|err| format!("write cleanup path {} failed: {err}", path.display()))?;
        out.write_all(b"\0")
            .map_err(|err| format!("write cleanup separator {} failed: {err}", path.display()))?;
    }
    Ok(())
}

pub fn repo_cache_paths(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let cache_root = repo_root.join(".cache");
    let runtime_fingerprint = runner::storage_runtime_fingerprint();
    let test_fingerprint = scenario_support::storage_test_fingerprint();
    let mut cleanup = BTreeSet::new();

    if !path_exists(&cache_root)
        .map_err(|err| format!("check {} failed: {err}", cache_root.display()))?
    {
        return Ok(Vec::new());
    }

    for child in sorted_entries(&cache_root)? {
        let Some(name) = child.file_name().and_then(|item| item.to_str()) else {
            cleanup.insert(child);
            continue;
        };

        if name == "scenario_1" && child.is_dir() {
            inspect_scenario_cache(&child, &mut cleanup)?;
            continue;
        }

        if name == "storage" && child.is_dir() {
            inspect_storage_cache(
                &child,
                &runtime_fingerprint,
                &test_fingerprint,
                &mut cleanup,
            )?;
            continue;
        }

        cleanup.insert(child);
    }

    Ok(cleanup.into_iter().collect())
}

fn sorted_entries(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut entries =
        read_dir(root).map_err(|err| format!("read directory {} failed: {err}", root.display()))?;
    entries.sort();
    Ok(entries)
}

fn read_first_line(path: &Path) -> Option<String> {
    read_to_string(path)
        .ok()
        .and_then(|text| text.lines().next().map(str::trim).map(str::to_owned))
        .filter(|text| !text.is_empty())
}

fn pid_after_marker(name: &str, marker: &str) -> Option<u32> {
    let (_, tail) = name.rsplit_once(marker)?;
    let digits: String = tail.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn scope_owner_pid(name: &str) -> Option<u32> {
    pid_after_marker(name, "-pid-")
}

fn tmp_owner_pid(name: &str) -> Option<u32> {
    if !name.starts_with('.') {
        return None;
    }
    pid_after_marker(name, ".tmp.")
}

fn lock_file_stale(path: &Path) -> bool {
    let Some(owner) = read_first_line(path) else {
        return true;
    };
    let Ok(pid) = owner.parse::<u32>() else {
        return true;
    };
    !fixture_cache::process_alive(pid)
}

fn case_missing_markers(case_dir: &Path) -> bool {
    !case_dir.join(READY_FILE).is_file()
        || read_first_line(&case_dir.join(FINGERPRINT_FILE)).is_none()
        || read_first_line(&case_dir.join(CONTENT_FINGERPRINT_FILE)).is_none()
}

fn scope_lock_is_live(scope_dir: &Path) -> bool {
    let Some(scope_name) = scope_dir.file_name().and_then(|item| item.to_str()) else {
        return false;
    };
    let Some(parent) = scope_dir.parent() else {
        return false;
    };
    let lock_path = parent.join(format!(".{scope_name}.scope.lock"));
    lock_path.is_file() && !lock_file_stale(&lock_path)
}

fn inspect_scope_cases(scope_dir: &Path, cleanup: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    for entry in sorted_entries(scope_dir)? {
        let Some(name) = entry.file_name().and_then(|item| item.to_str()) else {
            cleanup.insert(entry);
            continue;
        };

        if entry.is_file() {
            if name.ends_with(".lock") && lock_file_stale(&entry) {
                cleanup.insert(entry);
            }
            continue;
        }

        if !entry.is_dir() {
            continue;
        }

        if let Some(pid) = tmp_owner_pid(name) {
            if !fixture_cache::process_alive(pid) {
                cleanup.insert(entry);
            }
            continue;
        }

        if name.starts_with('.') {
            cleanup.insert(entry);
            continue;
        }

        if case_missing_markers(&entry) {
            cleanup.insert(entry);
            continue;
        }

        let Some(recorded) = read_first_line(&entry.join(CONTENT_FINGERPRINT_FILE)) else {
            cleanup.insert(entry);
            continue;
        };

        if fixture_cache::cache_content_fingerprint(&entry) != recorded {
            cleanup.insert(entry);
        }
    }

    Ok(())
}

fn inspect_scenario_cache(root: &Path, cleanup: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    for entry in sorted_entries(root)? {
        let Some(name) = entry.file_name().and_then(|item| item.to_str()) else {
            cleanup.insert(entry);
            continue;
        };

        if entry.is_file() {
            if name.ends_with(".lock") && lock_file_stale(&entry) {
                cleanup.insert(entry);
            }
            continue;
        }

        if !entry.is_dir() {
            continue;
        }

        if name == ADHOC_DIR {
            for adhoc_entry in sorted_entries(&entry)? {
                let Some(adhoc_name) = adhoc_entry.file_name().and_then(|item| item.to_str())
                else {
                    cleanup.insert(adhoc_entry);
                    continue;
                };
                if !adhoc_entry.is_dir() {
                    cleanup.insert(adhoc_entry);
                    continue;
                }
                let Some(pid) = scope_owner_pid(adhoc_name) else {
                    cleanup.insert(adhoc_entry);
                    continue;
                };
                if !fixture_cache::scope_owner_alive(adhoc_name, pid) {
                    cleanup.insert(adhoc_entry);
                }
            }
            continue;
        }

        if name == RUN_LOCKS_DIR {
            for lock_entry in sorted_entries(&entry)? {
                let Some(lock_name) = lock_entry.file_name().and_then(|item| item.to_str()) else {
                    cleanup.insert(lock_entry);
                    continue;
                };
                if lock_entry.is_file()
                    && lock_name.ends_with(".lock")
                    && lock_file_stale(&lock_entry)
                {
                    cleanup.insert(lock_entry);
                }
            }
            continue;
        }

        if name == SHARED_SCOPE_DIR || name == SHARED_PRECISE_SCOPE_DIR {
            if read_first_line(&entry.join(SCOPE_FINGERPRINT_FILE)).is_none()
                && !scope_lock_is_live(&entry)
            {
                cleanup.insert(entry);
                continue;
            }
            inspect_scope_cases(&entry, cleanup)?;
            continue;
        }

        if let Some(pid) = scope_owner_pid(name) {
            if !fixture_cache::scope_owner_alive(name, pid) {
                cleanup.insert(entry);
                continue;
            }
            inspect_scope_cases(&entry, cleanup)?;
            continue;
        }

        cleanup.insert(entry);
    }

    Ok(())
}

fn inspect_storage_scope_dir(
    scope_dir: &Path,
    expected_fingerprint: &str,
    cleanup: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    for entry in sorted_entries(scope_dir)? {
        let Some(name) = entry.file_name().and_then(|item| item.to_str()) else {
            cleanup.insert(entry);
            continue;
        };

        if !entry.is_dir() {
            cleanup.insert(entry);
            continue;
        }

        let pid = scope_owner_pid(name);
        if let Some(pid) = pid {
            if !fixture_cache::scope_owner_alive(name, pid) {
                cleanup.insert(entry);
                continue;
            }
        }

        let fingerprint_path = entry.join(ROOT_MARK_FILE);
        if !fingerprint_path.is_file() {
            cleanup.insert(entry);
            continue;
        }

        let current = read_first_line(&fingerprint_path);
        if current.as_deref() != Some(expected_fingerprint) && pid.is_none() {
            cleanup.insert(entry);
        }
    }

    Ok(())
}

fn inspect_storage_cache(
    root: &Path,
    runtime_fingerprint: &str,
    test_fingerprint: &str,
    cleanup: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    let scenario_root = root.join("scenario_1");
    if !path_exists(&scenario_root)
        .map_err(|err| format!("check {} failed: {err}", scenario_root.display()))?
    {
        cleanup.insert(root.to_path_buf());
        return Ok(());
    }

    for entry in sorted_entries(&scenario_root)? {
        let Some(name) = entry.file_name().and_then(|item| item.to_str()) else {
            cleanup.insert(entry);
            continue;
        };

        if name == "test_bins" && entry.is_dir() {
            inspect_storage_scope_dir(&entry, test_fingerprint, cleanup)?;
            continue;
        }

        if !entry.is_dir() {
            cleanup.insert(entry);
            continue;
        }

        let pid = scope_owner_pid(name);
        if let Some(pid) = pid {
            if !fixture_cache::scope_owner_alive(name, pid) {
                cleanup.insert(entry);
                continue;
            }
        }

        let fingerprint_path = entry.join(ROOT_MARK_FILE);
        if !fingerprint_path.is_file() {
            cleanup.insert(entry);
            continue;
        }

        let current = read_first_line(&fingerprint_path);
        if current.as_deref() != Some(runtime_fingerprint) && pid.is_none() {
            cleanup.insert(entry);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use z00z_utils::io::stable_current_exe_scope;
    use z00z_utils::io::{create_dir_all, write_file};

    use super::*;

    #[test]
    fn scope_liveness_requires_match() {
        let scope = stable_current_exe_scope("unknown_test_binary");
        let pid = std::process::id();

        assert!(fixture_cache::scope_owner_alive(
            &format!("{scope}-pid-{pid}"),
            pid
        ));
        assert!(!fixture_cache::scope_owner_alive(
            &format!("foreign_scope-pid-{pid}"),
            pid
        ));
    }

    #[test]
    fn storage_cleanup_uses_contracts() {
        let sandbox = TempDir::new().expect("create cleanup contract tempdir");
        let runtime_scope = sandbox
            .path()
            .join(".cache/storage/scenario_1/runtime_scope");
        let test_scope = sandbox
            .path()
            .join(".cache/storage/scenario_1/test_bins/test_scope");

        create_dir_all(&runtime_scope).expect("create runtime storage scope");
        create_dir_all(&test_scope).expect("create test storage scope");

        write_file(
            runtime_scope.join(ROOT_MARK_FILE),
            scenario_support::storage_test_fingerprint().as_bytes(),
        )
        .expect("write runtime scope marker");
        write_file(
            test_scope.join(ROOT_MARK_FILE),
            runner::storage_runtime_fingerprint().as_bytes(),
        )
        .expect("write test scope marker");

        let paths = repo_cache_paths(sandbox.path()).expect("collect cleanup paths");

        assert!(paths.contains(&runtime_scope));
        assert!(paths.contains(&test_scope));
    }
}
