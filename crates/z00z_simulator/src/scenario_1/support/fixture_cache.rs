use std::{
    cell::RefCell,
    collections::BTreeSet,
    ffi::OsStr,
    io::ErrorKind,
    io::Read,
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

use super::path_roots::{normalize_path, resolve_workspace_path, workspace_root};
use z00z_utils::io::{
    create_dir_all, current_exe_run_root, prune_scope_alias_dirs, read_dir, read_link,
    read_to_string, remove_dir_all, remove_file, rename_file, stable_current_exe_scope, write_file,
    File,
};
use z00z_utils::logger::{Logger, StdoutLogger};

const FINGERPRINT_FILE: &str = ".fingerprint";
const READY_FILE: &str = ".ready";
const CONTENT_FINGERPRINT_FILE: &str = ".content-fingerprint";
const FINGERPRINT_SCHEMA: &str = "fixture-cache-fingerprint-v2";
const CONTENT_FINGERPRINT_SCHEMA: &str = "fixture-cache-content-v1";
const SCOPE_FINGERPRINT_FILE: &str = ".scope-fingerprint";
const SHARED_SCOPE_DIR: &str = "shared";
const SHARED_PRECISE_SCOPE_DIR: &str = "shared_precise";
const RUN_LOCKS_DIR: &str = "run_locks";
const CARGO_TARGET_DIR_ENV: &str = "CARGO_TARGET_DIR";
const RUNTIME_CWD_ROOT_ENV: &str = "Z00Z_RUNTIME_CWD_ROOT";
const SCENARIO_CACHE_ROOT_ENV: &str = "Z00Z_SIMULATOR_CACHE_ROOT";
const VERIFICATION_RUN_ROOT_ENV: &str = "Z00Z_VERIFICATION_RUN_ROOT";
const CACHE_CLEANUP_TRACE_ENV: &str = "Z00Z_FIXTURE_CACHE_CLEANUP_TRACE";

const RUNTIME_INPUTS: &[&str] = &[
    "crates/z00z_simulator/src/scenario_1/scenario_config.yaml",
    "crates/z00z_simulator/src/scenario_1/scenario_design.yaml",
];

const SHARED_FINGERPRINT_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    ".cargo/config.toml",
    "crates/z00z_core/Cargo.toml",
    "crates/z00z_crypto/Cargo.toml",
    "crates/z00z_networks/rpc/Cargo.toml",
    "crates/z00z_simulator/Cargo.toml",
    "crates/z00z_storage/Cargo.toml",
    "crates/z00z_utils/Cargo.toml",
    "crates/z00z_wallets/Cargo.toml",
];

const SHARED_FINGERPRINT_CORE_TREES: &[&str] = &[
    "crates/z00z_core/src",
    "crates/z00z_crypto/src",
    "crates/z00z_networks/rpc/src",
    "crates/z00z_simulator/src",
    "crates/z00z_storage/src",
    "crates/z00z_utils/src",
    "crates/z00z_wallets/src",
];

const SHARED_FINGERPRINT_TEST_TREES: &[&str] = &["crates/z00z_simulator/tests"];

struct BuildGuard {
    lock_path: PathBuf,
    tmp_dir: PathBuf,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CacheRootCleanupStats {
    removed_scope_dirs: usize,
    removed_tmp_dirs: usize,
    removed_lock_files: usize,
}

impl CacheRootCleanupStats {
    fn total_removed(self) -> usize {
        self.removed_scope_dirs + self.removed_tmp_dirs + self.removed_lock_files
    }
}

thread_local! {
    static CACHE_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[doc(hidden)]
pub struct CacheRootOverrideGuard {
    prev: Option<PathBuf>,
}

impl Drop for CacheRootOverrideGuard {
    fn drop(&mut self) {
        let prev = std::mem::take(&mut self.prev);
        CACHE_ROOT_OVERRIDE.with(|slot| {
            slot.replace(prev);
        });
    }
}

impl BuildGuard {
    fn new(lock_path: PathBuf, tmp_dir: PathBuf) -> Self {
        Self { lock_path, tmp_dir }
    }

    fn clear_tmp(&mut self) {
        self.tmp_dir = PathBuf::new();
    }
}

impl Drop for BuildGuard {
    fn drop(&mut self) {
        if !self.tmp_dir.as_os_str().is_empty() && self.tmp_dir.exists() {
            let _ = remove_dir_all(&self.tmp_dir);
        }
        if self.lock_path.exists() {
            let _ = remove_file(&self.lock_path);
        }
    }
}

pub(crate) struct CaseLockGuard {
    lock_path: PathBuf,
}

impl CaseLockGuard {
    fn new(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }
}

impl Drop for CaseLockGuard {
    fn drop(&mut self) {
        if self.lock_path.exists() {
            let _ = remove_file(&self.lock_path);
        }
    }
}

pub fn repo_root() -> PathBuf {
    workspace_root()
}

fn runtime_cache_root_base() -> Option<PathBuf> {
    std::env::var_os(RUNTIME_CWD_ROOT_ENV)
        .map(PathBuf::from)
        .map(|root| resolve_workspace_path(&root).join("cache"))
        .or_else(|| {
            std::env::var_os(VERIFICATION_RUN_ROOT_ENV)
                .map(PathBuf::from)
                .map(|root| resolve_workspace_path(&root).join("cache"))
        })
        .or_else(|| current_exe_run_root().map(|root| resolve_workspace_path(&root).join("cache")))
        .or_else(|| {
            std::env::var_os(CARGO_TARGET_DIR_ENV)
                .map(PathBuf::from)
                .map(|target| resolve_workspace_path(&target).join("z00z-simulator-cache"))
        })
}

fn cache_root_override() -> Option<PathBuf> {
    CACHE_ROOT_OVERRIDE.with(|slot| slot.borrow().clone())
}

#[doc(hidden)]
pub fn override_cache_root_for_thread(path: &Path) -> CacheRootOverrideGuard {
    let prev = CACHE_ROOT_OVERRIDE.with(|slot| slot.replace(Some(resolve_workspace_path(path))));
    CacheRootOverrideGuard { prev }
}

pub fn scenario_cache_root() -> PathBuf {
    cache_root_override()
        .map(resolve_workspace_path)
        .or_else(|| std::env::var_os(SCENARIO_CACHE_ROOT_ENV).map(PathBuf::from))
        .map(resolve_workspace_path)
        .or_else(|| runtime_cache_root_base().map(|root| root.join("scenario_1")))
        .unwrap_or_else(|| repo_root().join(".cache").join("scenario_1"))
}

fn exe_scope() -> String {
    stable_current_exe_scope("unknown_test_binary")
}

fn process_scope_name() -> String {
    format!("{}-pid-{}", exe_scope(), std::process::id())
}

fn scoped_cache_root(scope: &Path, case_name: &str) -> PathBuf {
    scenario_cache_root().join(scope).join(case_name)
}

pub fn cache_root(case_name: &str) -> PathBuf {
    scoped_cache_root(Path::new(&process_scope_name()), case_name)
}

pub fn shared_cache_root(case_name: &str) -> PathBuf {
    scoped_cache_root(Path::new(SHARED_SCOPE_DIR), case_name)
}

pub fn shared_precise_cache_root(case_name: &str) -> PathBuf {
    scoped_cache_root(Path::new(SHARED_PRECISE_SCOPE_DIR), case_name)
}

#[derive(Clone, Copy)]
enum CacheScope {
    Local,
    Shared,
    SharedPrecise,
}

fn ensure_case_at(
    cache_dir: PathBuf,
    case_name: &str,
    scope: CacheScope,
    build: impl FnOnce(&Path),
) -> PathBuf {
    let _process_guard = crate::scenario_1::acquire_scenario_process_guard();
    prepare_cache_root_once(&scenario_cache_root());
    let ready = cache_dir.join(READY_FILE);
    let fingerprint_path = cache_dir.join(FINGERPRINT_FILE);
    let content_fingerprint_path = cache_dir.join(CONTENT_FINGERPRINT_FILE);
    let want_fingerprint = case_fingerprint(scope);
    let scope_dir = cache_dir
        .parent()
        .unwrap_or_else(|| panic!("cache scope missing for {}", cache_dir.display()));
    prepare_scope_root(scope_dir, &want_fingerprint);
    if cache_ready(
        &cache_dir,
        &ready,
        &fingerprint_path,
        &content_fingerprint_path,
        &want_fingerprint,
    ) {
        return cache_dir;
    }

    create_dir_all(scope_dir).expect("create fixture cache parent");

    if cache_ready(
        &cache_dir,
        &ready,
        &fingerprint_path,
        &content_fingerprint_path,
        &want_fingerprint,
    ) {
        return cache_dir;
    }
    let _case_lock = acquire_case_lock(&cache_dir)
        .unwrap_or_else(|err| panic!("acquire fixture cache lock failed: {err}"));
    if cache_ready(
        &cache_dir,
        &ready,
        &fingerprint_path,
        &content_fingerprint_path,
        &want_fingerprint,
    ) {
        return cache_dir;
    }

    clear_case_tmp_dirs(scope_dir, case_name);
    let pid = std::process::id();
    let tmp_dir = scope_dir.join(format!(".{case_name}.tmp.{pid}"));
    create_dir_all(&tmp_dir).expect("create fixture temp dir");
    let mut guard = BuildGuard::new(PathBuf::new(), tmp_dir.clone());

    build(&tmp_dir);

    if cache_dir.exists() {
        remove_dir_all(&cache_dir).expect("remove stale fixture cache dir");
    }
    rename_file(&tmp_dir, &cache_dir).expect("promote fixture cache dir");
    guard.clear_tmp();
    write_file(&fingerprint_path, want_fingerprint.as_bytes())
        .expect("write fixture cache fingerprint");
    let content_fingerprint = cache_content_fingerprint(&cache_dir);
    write_file(&content_fingerprint_path, content_fingerprint.as_bytes())
        .expect("write fixture cache content fingerprint");
    File::create(&ready).expect("write fixture cache marker");
    cache_dir
}

fn prepare_cache_root_once(root: &Path) {
    let key = normalize_path(root);
    let first_use = {
        let mut prepared = cache_root_cleanup_registry()
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        prepared.insert(key.clone())
    };
    if !first_use {
        return;
    }

    let start = Instant::now();
    let stats = cleanup_cache_root(&key);
    let elapsed = start.elapsed();
    if stats.total_removed() > 0 || std::env::var_os(CACHE_CLEANUP_TRACE_ENV).is_some() {
        StdoutLogger.info(&format!(
            "scenario_1.fixture_cache_cleanup: root={}, removed_scope_dirs={}, removed_tmp_dirs={}, removed_lock_files={}, elapsed_ms={}",
            key.display(),
            stats.removed_scope_dirs,
            stats.removed_tmp_dirs,
            stats.removed_lock_files,
            elapsed.as_millis(),
        ));
    }
}

fn cleanup_cache_root(root: &Path) -> CacheRootCleanupStats {
    if !root.exists() {
        return CacheRootCleanupStats::default();
    }

    let mut stats = CacheRootCleanupStats::default();
    prune_scope_aliases(root, &exe_scope(), &mut stats);
    prune_scope_aliases(root, SHARED_SCOPE_DIR, &mut stats);
    prune_scope_aliases(root, SHARED_PRECISE_SCOPE_DIR, &mut stats);
    cleanup_run_lock_dir(&root.join(RUN_LOCKS_DIR), &mut stats);
    let current_scope = process_scope_name();
    for entry in read_dir(root)
        .unwrap_or_else(|err| panic!("read fixture cache root {} failed: {err}", root.display()))
    {
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if entry.is_dir() {
            if let Some(pid) = scope_owner_pid(name) {
                if !scope_owner_alive(name, pid) {
                    remove_dir_all(&entry).unwrap_or_else(|err| {
                        panic!(
                            "remove stale fixture cache scope {} failed: {err}",
                            entry.display()
                        )
                    });
                    stats.removed_scope_dirs += 1;
                    continue;
                }
            }
            if name == current_scope || name == SHARED_SCOPE_DIR || name == SHARED_PRECISE_SCOPE_DIR
            {
                cleanup_scope_contents(&entry, &mut stats);
            }
            continue;
        }
        if entry.is_file() && is_lock_file_name(name) && prune_dead_lock_file(&entry) {
            stats.removed_lock_files += 1;
        }
    }

    stats
}

fn cleanup_run_lock_dir(run_lock_dir: &Path, stats: &mut CacheRootCleanupStats) {
    if !run_lock_dir.exists() {
        return;
    }

    for entry in read_dir(run_lock_dir).unwrap_or_else(|err| {
        panic!(
            "read scenario run lock dir {} failed: {err}",
            run_lock_dir.display()
        )
    }) {
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if entry.is_file() && is_lock_file_name(name) && prune_dead_lock_file(&entry) {
            stats.removed_lock_files += 1;
        }
    }
}

fn prune_scope_aliases(root: &Path, scope_name: &str, stats: &mut CacheRootCleanupStats) {
    let removed = prune_scope_alias_dirs(root, scope_name).unwrap_or_else(|err| {
        panic!(
            "prune stale fixture cache scope aliases in {} for {} failed: {err}",
            root.display(),
            scope_name
        )
    });
    stats.removed_scope_dirs += removed;
}

fn cleanup_scope_contents(scope_dir: &Path, stats: &mut CacheRootCleanupStats) {
    for entry in read_dir(scope_dir).unwrap_or_else(|err| {
        panic!(
            "read fixture scope dir {} failed: {err}",
            scope_dir.display()
        )
    }) {
        let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if entry.is_dir() {
            if let Some(pid) = tmp_owner_pid(name) {
                if !process_alive(pid) {
                    remove_dir_all(&entry).unwrap_or_else(|err| {
                        panic!(
                            "remove stale fixture temp dir {} failed: {err}",
                            entry.display()
                        )
                    });
                    stats.removed_tmp_dirs += 1;
                }
            }
            continue;
        }
        if entry.is_file() && is_lock_file_name(name) && prune_dead_lock_file(&entry) {
            stats.removed_lock_files += 1;
        }
    }
}

fn cache_root_cleanup_registry() -> &'static Mutex<BTreeSet<PathBuf>> {
    static CLEANED: OnceLock<Mutex<BTreeSet<PathBuf>>> = OnceLock::new();
    CLEANED.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn scope_owner_pid(name: &str) -> Option<u32> {
    pid_after_marker(name, "-pid-")
}

pub(crate) fn scope_owner_alive(name: &str, pid: u32) -> bool {
    let Some((scope_name, _)) = name.rsplit_once("-pid-") else {
        return false;
    };
    cache_process_scope(pid).is_some_and(|live_scope| live_scope == scope_name)
}

fn tmp_owner_pid(name: &str) -> Option<u32> {
    if !name.starts_with('.') {
        return None;
    }
    pid_after_marker(name, ".tmp.")
}

fn pid_after_marker(name: &str, marker: &str) -> Option<u32> {
    let (_, tail) = name.rsplit_once(marker)?;
    let digits: String = tail.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn is_lock_file_name(name: &str) -> bool {
    name.ends_with(".lock")
}

fn prune_dead_lock_file(lock_path: &Path) -> bool {
    let Some(pid) = read_lock_owner_pid(lock_path) else {
        return false;
    };
    if process_alive(pid) {
        return false;
    }
    remove_lock_file(lock_path).unwrap_or_else(|err| panic!("{err}"));
    true
}

fn clear_case_tmp_dirs(scope_dir: &Path, case_name: &str) {
    if !scope_dir.exists() {
        return;
    }

    let prefix = format!(".{case_name}.tmp.");
    for path in read_dir(scope_dir).expect("read fixture scope dir") {
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().expect("fixture scope entry");
        if name.to_string_lossy().starts_with(&prefix) {
            remove_dir_all(&path).expect("remove stale fixture temp dir");
        }
    }
}

pub fn ensure_case(case_name: &str, build: impl FnOnce(&Path)) -> PathBuf {
    let _process_guard = crate::scenario_1::acquire_scenario_process_guard();
    ensure_case_at(cache_root(case_name), case_name, CacheScope::Local, build)
}

pub fn ensure_shared_case(case_name: &str, build: impl FnOnce(&Path)) -> PathBuf {
    let _process_guard = crate::scenario_1::acquire_scenario_process_guard();
    ensure_case_at(
        shared_cache_root(case_name),
        case_name,
        CacheScope::Shared,
        build,
    )
}

pub fn ensure_shared_case_precise(case_name: &str, build: impl FnOnce(&Path)) -> PathBuf {
    let _process_guard = crate::scenario_1::acquire_scenario_process_guard();
    ensure_case_at(
        shared_precise_cache_root(case_name),
        case_name,
        CacheScope::SharedPrecise,
        build,
    )
}

pub(crate) fn acquire_case_lock(cache_dir: &Path) -> Result<CaseLockGuard, String> {
    let lock_path = case_lock_path(cache_dir)?;

    loop {
        match File::options()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                write_lock_owner_pid(&mut file, &lock_path)?;
                return Ok(CaseLockGuard::new(lock_path));
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                if clear_stale_case_lock(cache_dir, &lock_path)? {
                    continue;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(err) => {
                return Err(format!(
                    "open fixture cache lock {} failed: {err}",
                    lock_path.display()
                ));
            }
        }
    }
}

fn write_lock_owner_pid(file: &mut File, lock_path: &Path) -> Result<(), String> {
    file.write_all(std::process::id().to_string().as_bytes())
        .map_err(|err| {
            format!(
                "write fixture cache lock owner {} failed: {err}",
                lock_path.display()
            )
        })
}

fn clear_stale_case_lock(cache_dir: &Path, lock_path: &Path) -> Result<bool, String> {
    if let Some(pid) = read_lock_owner_pid(lock_path) {
        return clear_dead_owner_lock(lock_path, pid);
    }

    let scope_dir = cache_dir
        .parent()
        .ok_or_else(|| format!("fixture cache scope missing for {}", cache_dir.display()))?;
    let case_name = cache_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "fixture cache case name missing for {}",
                cache_dir.display()
            )
        })?;
    let prefix = format!(".{case_name}.tmp.");
    let mut saw_legacy_tmp = false;

    for path in read_dir(scope_dir).map_err(|err| {
        format!(
            "read fixture scope dir {} failed while clearing stale lock: {err}",
            scope_dir.display()
        )
    })? {
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(pid_text) = name.strip_prefix(&prefix) else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u32>() else {
            continue;
        };
        saw_legacy_tmp = true;
        if process_alive(pid) {
            return Ok(false);
        }
    }

    if saw_legacy_tmp {
        remove_lock_file(lock_path)?;
        return Ok(true);
    }

    Ok(false)
}

fn read_lock_owner_pid(lock_path: &Path) -> Option<u32> {
    read_to_string(lock_path)
        .ok()
        .and_then(|text| text.lines().next()?.trim().parse::<u32>().ok())
}

fn clear_dead_owner_lock(lock_path: &Path, pid: u32) -> Result<bool, String> {
    if process_alive(pid) {
        return Ok(false);
    }
    remove_lock_file(lock_path)?;
    Ok(true)
}

fn remove_lock_file(lock_path: &Path) -> Result<(), String> {
    match remove_file(lock_path) {
        Ok(()) => Ok(()),
        Err(_err) if !lock_path.exists() => Ok(()),
        Err(err) => Err(format!(
            "remove stale fixture cache lock {} failed: {err}",
            lock_path.display()
        )),
    }
}

pub(crate) fn process_alive(pid: u32) -> bool {
    cache_process_scope(pid).is_some()
}

fn cache_process_scope(pid: u32) -> Option<String> {
    let exe_path = read_link(PathBuf::from("/proc").join(pid.to_string()).join("exe"))
        .ok()
        .map(|path| normalize_path(&path))?;
    if !is_cache_process_executable(&exe_path) {
        return None;
    }
    let stem = exe_path.file_stem()?.to_str()?;
    Some(normalize_scope_name(stem))
}

fn is_cache_process_executable(exe_path: &Path) -> bool {
    if exe_path.starts_with(normalize_path(&repo_root())) {
        return true;
    }

    // Verification runs may set CARGO_TARGET_DIR outside the repository. Trust
    // only binaries under this test process's own target root, never arbitrary
    // live PIDs elsewhere on the host.
    current_target_root().is_some_and(|target_root| exe_path.starts_with(target_root))
}

fn current_target_root() -> Option<PathBuf> {
    let current_exe = std::env::current_exe()
        .ok()
        .map(|path| normalize_path(&path))?;
    current_exe
        .ancestors()
        .find(|ancestor| ancestor.file_name() == Some(OsStr::new("deps")))
        .and_then(|deps_dir| deps_dir.parent())
        .and_then(|profile_dir| profile_dir.parent())
        .map(Path::to_path_buf)
}

fn normalize_scope_name(raw: &str) -> String {
    let Some((prefix, suffix)) = raw.rsplit_once('-') else {
        return raw.to_string();
    };
    if suffix.len() >= 8 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        prefix.to_string()
    } else {
        raw.to_string()
    }
}

fn case_lock_path(cache_dir: &Path) -> Result<PathBuf, String> {
    let scope_dir = cache_dir
        .parent()
        .ok_or_else(|| format!("fixture cache scope missing for {}", cache_dir.display()))?;
    let case_name = cache_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "fixture cache case name missing for {}",
                cache_dir.display()
            )
        })?;
    Ok(scope_dir.join(format!("{case_name}.lock")))
}

pub fn refresh_case_content_fingerprint(cache_dir: &Path) -> Result<(), String> {
    if !cache_dir.exists() {
        return Err(format!(
            "fixture cache dir missing for fingerprint refresh: {}",
            cache_dir.display()
        ));
    }

    let content_fingerprint = cache_content_fingerprint(cache_dir);
    write_file(
        cache_dir.join(CONTENT_FINGERPRINT_FILE),
        content_fingerprint.as_bytes(),
    )
    .map_err(|err| {
        format!(
            "write fixture cache content fingerprint {} failed: {err}",
            cache_dir.display()
        )
    })
}

pub fn copy_tree(src: &Path, dst: &Path) {
    create_dir_all(dst).expect("create destination dir");
    for src_path in read_dir(src).expect("read source dir") {
        let dst_path = dst.join(src_path.file_name().expect("dir entry file name"));
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else if src_path.is_file() {
            copy_file(&src_path, &dst_path);
        }
    }
}

pub fn copy_selected(src_root: &Path, dst_root: &Path, rel_paths: &[&str]) {
    for rel_path in rel_paths {
        let rel = Path::new(rel_path);
        let src_path = src_root.join(rel);
        let dst_path = dst_root.join(rel);
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else if src_path.is_file() {
            copy_file(&src_path, &dst_path);
        } else {
            panic!("copy_selected source missing: {}", src_path.display());
        }
    }
}

fn copy_file(src_path: &Path, dst_path: &Path) {
    if let Some(parent) = dst_path.parent() {
        create_dir_all(parent).expect("create destination parent");
    }
    let mut src_file = File::open(src_path).expect("open source file");
    let mut dst_file = File::create(dst_path).expect("create destination file");
    std::io::copy(&mut src_file, &mut dst_file).expect("copy file");
}

fn cache_ready(
    cache_dir: &Path,
    ready: &Path,
    fingerprint_path: &Path,
    content_fingerprint_path: &Path,
    want_fingerprint: &str,
) -> bool {
    ready.exists()
        && read_to_string(fingerprint_path)
            .ok()
            .is_some_and(|current| current == want_fingerprint)
        && read_to_string(content_fingerprint_path)
            .ok()
            .is_some_and(|current| current == cache_content_fingerprint(cache_dir))
}

pub(crate) fn cache_content_fingerprint(cache_dir: &Path) -> String {
    let mut parts = vec![CONTENT_FINGERPRINT_SCHEMA.as_bytes().to_vec()];
    let mut entries = Vec::new();
    collect_case_files(cache_dir, &mut entries);
    entries.sort();

    for path in entries {
        let rel = path
            .strip_prefix(cache_dir)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .to_string();
        parts.push(file_fingerprint(&rel, &path).to_vec());
    }

    let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
    hex::encode(z00z_crypto::blake2b_hash(
        b"z00z.fixture_cache.content.v1",
        &refs,
    ))
}

fn prepare_scope_root(scope_dir: &Path, want_fingerprint: &str) {
    if scope_ready(scope_dir, want_fingerprint) {
        return;
    }

    let scope_name = scope_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("fixture_scope");
    let lock_dir = scope_dir
        .parent()
        .unwrap_or_else(|| panic!("fixture scope parent missing for {}", scope_dir.display()));
    create_dir_all(lock_dir).expect("create fixture scope parent");
    let lock_path = lock_dir.join(format!(".{scope_name}.scope.lock"));

    loop {
        if scope_ready(scope_dir, want_fingerprint) {
            return;
        }

        match File::options()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                write_lock_owner_pid(&mut file, &lock_path).unwrap_or_else(|err| panic!("{err}"));
                break;
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                if let Some(pid) = read_lock_owner_pid(&lock_path) {
                    if clear_dead_owner_lock(&lock_path, pid).unwrap_or_else(|err| panic!("{err}"))
                    {
                        continue;
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(err) => panic!(
                "open fixture scope lock {} failed: {err}",
                lock_path.display()
            ),
        }
    }

    let mut guard = BuildGuard::new(lock_path.clone(), PathBuf::new());
    if !scope_ready(scope_dir, want_fingerprint) {
        prune_scope_alias_dirs(lock_dir, scope_name).expect("prune stale fixture scope aliases");
        if scope_dir.exists() {
            remove_dir_all(scope_dir).expect("remove stale fixture scope");
        }
        create_dir_all(scope_dir).expect("create fixture scope dir");
        write_file(
            scope_dir.join(SCOPE_FINGERPRINT_FILE),
            want_fingerprint.as_bytes(),
        )
        .expect("write fixture scope fingerprint");
    }
    guard.clear_tmp();
}

fn scope_ready(scope_dir: &Path, want_fingerprint: &str) -> bool {
    scope_dir.exists()
        && read_to_string(scope_dir.join(SCOPE_FINGERPRINT_FILE))
            .ok()
            .is_some_and(|current| current == want_fingerprint)
}

fn case_fingerprint(scope: CacheScope) -> String {
    match scope {
        CacheScope::Local => local_case_fingerprint(),
        CacheScope::Shared => shared_case_fingerprint(),
        CacheScope::SharedPrecise => shared_precise_case_fingerprint(),
    }
}

fn local_case_fingerprint() -> String {
    static LOCAL: OnceLock<String> = OnceLock::new();
    LOCAL
        .get_or_init(|| {
            let mut parts = vec![
                FINGERPRINT_SCHEMA.as_bytes().to_vec(),
                b"scope=local".to_vec(),
            ];
            hash_feature_flags(&mut parts);
            if let Ok(exe_path) = std::env::current_exe() {
                parts.push(file_fingerprint("current_exe", &exe_path).to_vec());
            }
            hash_runtime_inputs(&mut parts);
            let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
            hex::encode(z00z_crypto::blake2b_hash(
                b"z00z.fixture_cache.local_scope.v1",
                &refs,
            ))
        })
        .clone()
}

fn shared_case_fingerprint() -> String {
    static SHARED: OnceLock<String> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let mut parts = vec![
                FINGERPRINT_SCHEMA.as_bytes().to_vec(),
                b"scope=shared".to_vec(),
            ];
            hash_feature_flags(&mut parts);
            for rel in SHARED_FINGERPRINT_FILES {
                let abs = repo_root().join(rel);
                parts.push(file_fingerprint(rel, &abs).to_vec());
            }
            for rel in SHARED_FINGERPRINT_CORE_TREES {
                let abs = repo_root().join(rel);
                hash_tree(&mut parts, rel, &abs);
            }
            for rel in SHARED_FINGERPRINT_TEST_TREES {
                let abs = repo_root().join(rel);
                hash_tree(&mut parts, rel, &abs);
            }
            hash_runtime_inputs(&mut parts);
            let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
            hex::encode(z00z_crypto::blake2b_hash(
                b"z00z.fixture_cache.shared_scope.v1",
                &refs,
            ))
        })
        .clone()
}

fn shared_precise_case_fingerprint() -> String {
    static SHARED_PRECISE: OnceLock<String> = OnceLock::new();
    SHARED_PRECISE
        .get_or_init(|| {
            let mut parts = vec![
                FINGERPRINT_SCHEMA.as_bytes().to_vec(),
                b"scope=shared-precise".to_vec(),
            ];
            hash_feature_flags(&mut parts);
            for rel in SHARED_FINGERPRINT_FILES {
                let abs = repo_root().join(rel);
                parts.push(file_fingerprint(rel, &abs).to_vec());
            }
            for rel in SHARED_FINGERPRINT_CORE_TREES {
                let abs = repo_root().join(rel);
                hash_tree(&mut parts, rel, &abs);
            }
            for rel in SHARED_FINGERPRINT_TEST_TREES {
                let abs = repo_root().join(rel);
                hash_tree(&mut parts, rel, &abs);
            }
            hash_runtime_inputs(&mut parts);
            let refs = parts.iter().map(Vec::as_slice).collect::<Vec<_>>();
            hex::encode(z00z_crypto::blake2b_hash(
                b"z00z.fixture_cache.shared_precise_scope.v1",
                &refs,
            ))
        })
        .clone()
}

fn hash_feature_flags(parts: &mut Vec<Vec<u8>>) {
    parts.push(
        format!(
            "feature:test-params-fast={}",
            cfg!(feature = "test-params-fast")
        )
        .into_bytes(),
    );
    parts.push(
        format!(
            "feature:wallet_debug_tools={}",
            cfg!(feature = "wallet_debug_tools")
        )
        .into_bytes(),
    );
}

fn hash_runtime_inputs(parts: &mut Vec<Vec<u8>>) {
    for rel in RUNTIME_INPUTS {
        let abs = repo_root().join(rel);
        parts.push(file_fingerprint(rel, &abs).to_vec());
    }
}

fn hash_tree(parts: &mut Vec<Vec<u8>>, rel_root: &str, abs_root: &Path) {
    if !abs_root.exists() {
        parts.push(
            z00z_crypto::blake2b_hash(
                b"z00z.fixture_cache.missing_tree.v1",
                &[rel_root.as_bytes()],
            )
            .to_vec(),
        );
        return;
    }

    let mut entries = Vec::new();
    collect_files(abs_root, &mut entries);
    entries.sort();
    for path in entries {
        let rel = path
            .strip_prefix(repo_root())
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .to_string();
        parts.push(file_fingerprint(&rel, &path).to_vec());
    }
}

fn collect_files(root: &Path, files: &mut Vec<PathBuf>) {
    let mut entries =
        read_dir(root).unwrap_or_else(|err| panic!("read_dir {} failed: {err}", root.display()));
    entries.sort();

    for path in entries {
        if path
            .file_name()
            .is_some_and(|name| name == OsStr::new(".git") || name == OsStr::new("target"))
        {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn collect_case_files(root: &Path, files: &mut Vec<PathBuf>) {
    let mut entries =
        read_dir(root).unwrap_or_else(|err| panic!("read_dir {} failed: {err}", root.display()));
    entries.sort();

    for path in entries {
        let Some(name) = path.file_name() else {
            continue;
        };
        if name == OsStr::new(FINGERPRINT_FILE)
            || name == OsStr::new(READY_FILE)
            || name == OsStr::new(CONTENT_FINGERPRINT_FILE)
        {
            continue;
        }
        if path.is_dir() {
            collect_case_files(&path, files);
        } else if path.is_file() {
            files.push(path);
        }
    }
}

fn file_fingerprint(label: &str, path: &Path) -> [u8; 32] {
    match File::open(path) {
        Ok(mut file) => {
            let size_bytes = file
                .metadata()
                .map(|meta| meta.len())
                .unwrap_or_default()
                .to_le_bytes();
            let mut content = Vec::new();
            file.read_to_end(&mut content).unwrap_or_else(|err| {
                panic!(
                    "read fixture fingerprint file {} failed: {err}",
                    path.display()
                )
            });
            z00z_crypto::blake2b_hash(
                b"z00z.fixture_cache.file.v1",
                &[label.as_bytes(), &size_bytes, content.as_slice()],
            )
        }
        Err(err) => {
            let err_text = err.to_string();
            z00z_crypto::blake2b_hash(
                b"z00z.fixture_cache.missing_file.v1",
                &[label.as_bytes(), err_text.as_bytes()],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::time::{SystemTimeProvider, TimeProvider};

    fn unique_scope_dir(label: &str) -> PathBuf {
        let time = SystemTimeProvider;
        let nonce = time
            .try_unix_timestamp_micros()
            .expect("system clock before unix epoch");
        std::env::temp_dir().join(format!(
            "z00z_fixture_cache_{label}_{}_{}",
            std::process::id(),
            nonce
        ))
    }

    fn fixture_cache_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    #[test]
    fn builder_keeps_tmp_dir() {
        let _guard = fixture_cache_test_lock();
        let scope_dir = unique_scope_dir("waiter_lock");
        let case_name = "waiter_lock_case";
        let cache_dir = scope_dir.join(case_name);
        let want_fingerprint = case_fingerprint(CacheScope::Local);

        create_dir_all(&scope_dir).expect("create test fixture scope");
        write_file(
            scope_dir.join(SCOPE_FINGERPRINT_FILE),
            want_fingerprint.as_bytes(),
        )
        .expect("write scope fingerprint");

        let active_tmp_dir = scope_dir.join(format!(".{case_name}.tmp.active"));
        create_dir_all(&active_tmp_dir).expect("create active tmp dir");
        write_file(active_tmp_dir.join("sentinel.txt"), b"live").expect("write active sentinel");

        let lock_path = scope_dir.join(format!("{case_name}.lock"));
        File::create(&lock_path).expect("create synthetic case lock");

        let join = thread::spawn({
            let cache_dir = cache_dir.clone();
            move || {
                ensure_case_at(cache_dir, case_name, CacheScope::Local, |tmp_dir| {
                    write_file(tmp_dir.join("built.txt"), b"ok").expect("write built fixture");
                })
            }
        });

        thread::sleep(Duration::from_millis(250));
        assert!(
            active_tmp_dir.exists(),
            "waiting builder deleted active tmp dir before lock release"
        );

        remove_file(&lock_path).expect("remove synthetic case lock");

        let built_dir = join.join().expect("fixture builder thread");
        assert_eq!(built_dir, cache_dir);
        assert!(cache_dir.join("built.txt").exists());

        if scope_dir.exists() {
            remove_dir_all(&scope_dir).expect("remove test fixture scope");
        }
    }

    #[test]
    fn stale_case_lock_pid_cleanup() {
        let _guard = fixture_cache_test_lock();
        let scope_dir = unique_scope_dir("stale_legacy_lock");
        let case_name = "stale_legacy_lock_case";
        let cache_dir = scope_dir.join(case_name);
        let want_fingerprint = case_fingerprint(CacheScope::Local);

        create_dir_all(&scope_dir).expect("create test fixture scope");
        write_file(
            scope_dir.join(SCOPE_FINGERPRINT_FILE),
            want_fingerprint.as_bytes(),
        )
        .expect("write scope fingerprint");

        let stale_pid = 999_999_u32;
        let stale_tmp_dir = scope_dir.join(format!(".{case_name}.tmp.{stale_pid}"));
        create_dir_all(&stale_tmp_dir).expect("create stale tmp dir");
        write_file(stale_tmp_dir.join("sentinel.txt"), b"stale").expect("write stale sentinel");

        let lock_path = scope_dir.join(format!("{case_name}.lock"));
        File::create(&lock_path).expect("create synthetic legacy case lock");

        let built_dir =
            ensure_case_at(cache_dir.clone(), case_name, CacheScope::Local, |tmp_dir| {
                write_file(tmp_dir.join("built.txt"), b"ok").expect("write built fixture");
            });

        assert_eq!(built_dir, cache_dir);
        assert!(cache_dir.join("built.txt").exists());
        assert!(
            !lock_path.exists(),
            "stale legacy lock should be cleared after rebuild"
        );
        assert!(
            !stale_tmp_dir.exists(),
            "stale legacy tmp dir should be cleared after rebuild"
        );

        if scope_dir.exists() {
            remove_dir_all(&scope_dir).expect("remove test fixture scope");
        }
    }

    #[test]
    fn cleanup_prunes_dead_pid() {
        let _guard = fixture_cache_test_lock();
        let root = unique_scope_dir("root_cleanup_dead_scope");
        let dead_scope = root.join(format!("{}-pid-999999", exe_scope()));
        let live_scope = root.join(format!("{}-pid-{}", exe_scope(), std::process::id()));

        create_dir_all(dead_scope.join("case")).expect("create dead scope");
        write_file(dead_scope.join("case").join("marker.txt"), b"stale")
            .expect("write dead scope marker");
        create_dir_all(live_scope.join("case")).expect("create live scope");

        let stats = cleanup_cache_root(&root);

        assert_eq!(stats.removed_scope_dirs, 1);
        assert!(
            !dead_scope.exists(),
            "dead pid scope must be removed under {}",
            root.display()
        );
        assert!(live_scope.exists(), "live pid scope must survive");

        if root.exists() {
            remove_dir_all(&root).expect("remove cleanup dead scope root");
        }
    }

    #[test]
    fn cleanup_prunes_foreign_pid() {
        let _guard = fixture_cache_test_lock();
        let root = unique_scope_dir("root_cleanup_foreign_live_scope");
        let foreign_scope = root.join(format!("foreign_scope-pid-{}", std::process::id()));

        create_dir_all(foreign_scope.join("case")).expect("create foreign live scope");
        write_file(foreign_scope.join("case").join("marker.txt"), b"stale")
            .expect("write foreign live scope marker");

        let stats = cleanup_cache_root(&root);

        assert_eq!(stats.removed_scope_dirs, 1);
        assert!(
            !foreign_scope.exists(),
            "foreign live pid scope must be removed under {}",
            root.display()
        );

        if root.exists() {
            remove_dir_all(&root).expect("remove cleanup foreign live scope root");
        }
    }

    #[test]
    fn cleanup_prunes_dead_tmp() {
        let _guard = fixture_cache_test_lock();
        let root = unique_scope_dir("root_cleanup_dead_tmp");
        let shared_scope = root.join(SHARED_SCOPE_DIR);
        let dead_tmp = shared_scope.join(".case.tmp.999999");
        let live_tmp = shared_scope.join(format!(".case.tmp.{}", std::process::id()));
        let case_lock = shared_scope.join("case.lock");
        let scope_lock = root.join(".shared.scope.lock");

        create_dir_all(dead_tmp.join("nested")).expect("create dead tmp dir");
        create_dir_all(&live_tmp).expect("create live tmp dir");
        write_file(dead_tmp.join("nested").join("marker.txt"), b"stale")
            .expect("write dead tmp marker");
        write_file(&case_lock, b"999999").expect("write dead case lock");
        write_file(&scope_lock, b"999999").expect("write dead scope lock");

        let stats = cleanup_cache_root(&root);

        assert_eq!(stats.removed_tmp_dirs, 1);
        assert_eq!(stats.removed_lock_files, 2);
        assert!(!dead_tmp.exists(), "dead tmp dir must be removed");
        assert!(live_tmp.exists(), "live tmp dir must survive");
        assert!(!case_lock.exists(), "dead case lock must be removed");
        assert!(!scope_lock.exists(), "dead scope lock must be removed");

        if root.exists() {
            remove_dir_all(&root).expect("remove cleanup dead tmp root");
        }
    }

    #[test]
    fn cleanup_prunes_foreign_run_lock() {
        let _guard = fixture_cache_test_lock();
        let root = unique_scope_dir("root_cleanup_foreign_run_lock");
        let run_locks = root.join(RUN_LOCKS_DIR);
        let lock_path = run_locks.join("foreign.lock");
        create_dir_all(&run_locks).expect("create run lock dir");

        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn foreign live process");
        // `Command::spawn` returns after fork and can race the child's `exec`.
        // Until `sleep` is exec'd, `/proc/<pid>/exe` may still name this test
        // binary, which is intentionally recognised as a cache process. Wait
        // for the foreign executable before asserting foreign-lock cleanup.
        #[cfg(target_os = "linux")]
        {
            let child_exe = PathBuf::from("/proc")
                .join(child.id().to_string())
                .join("exe");
            let deadline = Instant::now() + Duration::from_secs(1);
            let mut child_execed_sleep = false;
            while Instant::now() < deadline {
                child_execed_sleep = read_link(&child_exe)
                    .ok()
                    .and_then(|path| path.file_name().map(OsStr::to_owned))
                    .is_some_and(|name| name == "sleep");
                if child_execed_sleep {
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            }
            if !child_execed_sleep {
                let _ = child.kill();
                let _ = child.wait();
                panic!("foreign child did not exec sleep before cleanup test");
            }
        }
        write_file(
            &lock_path,
            format!("{}\n/tmp/foreign/outputs/scenario_1\n", child.id()).as_bytes(),
        )
        .expect("write foreign run lock");

        let stats = cleanup_cache_root(&root);

        assert_eq!(stats.removed_lock_files, 1);
        assert!(
            !lock_path.exists(),
            "foreign live pid run lock must be removed"
        );

        let _ = child.kill();
        let _ = child.wait();
        if root.exists() {
            remove_dir_all(&root).expect("remove cleanup foreign run lock root");
        }
    }

    #[test]
    fn cleanup_prunes_scope_aliases() {
        let _guard = fixture_cache_test_lock();
        let root = unique_scope_dir("root_cleanup_scope_aliases");
        let local_alias = root.join(format!("{}-9d4bd19d594c355f", exe_scope()));
        let shared_alias = root.join("shared-9d4bd19d594c355f");
        let precise_alias = root.join("shared_precise-9d4bd19d594c355f");
        let keep_scope = root.join("shared-v1");

        create_dir_all(&local_alias).expect("create local alias scope");
        create_dir_all(&shared_alias).expect("create shared alias scope");
        create_dir_all(&precise_alias).expect("create shared precise alias scope");
        create_dir_all(&keep_scope).expect("create non-alias scope");

        let stats = cleanup_cache_root(&root);

        assert_eq!(stats.removed_scope_dirs, 3);
        assert!(
            !local_alias.exists(),
            "hash local alias scope must be removed"
        );
        assert!(
            !shared_alias.exists(),
            "hash shared alias scope must be removed"
        );
        assert!(
            !precise_alias.exists(),
            "hash shared precise alias scope must be removed"
        );
        assert!(keep_scope.exists(), "non-alias scope must survive");

        if root.exists() {
            remove_dir_all(&root).expect("remove cleanup scope alias root");
        }
    }
}
