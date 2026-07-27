#![allow(missing_docs)]
#![allow(unsafe_code)]

//! OS-level best-effort hardening.
//!
//! This module is the ONE SOURCE OF TRUTH entrypoint for OS-hardening.
//!
//! Design constraints:
//! - `z00z_utils` warns on `unsafe`; this module contains a small audited amount
//!   of `unsafe` for `mlock`/`munlock`, Linux `prctl`, and Windows `VirtualLock`.
//! - All operations are best-effort and must fail closed (no panics).

use thiserror::Error;

mod locking;
#[cfg(test)]
#[path = "os_hardening/test_mod.rs"]
mod tests;
#[cfg(all(any(target_os = "linux", target_os = "android"), not(miri)))]
use libc::{prctl, PR_SET_DUMPABLE};
#[cfg(all(unix, not(target_os = "ios"), not(miri)))]
use libc::{rlimit, setrlimit, RLIMIT_CORE};

pub use self::locking::{lock_bytes_best_effort, LockedBytes, OwnedLockedBytes};

#[derive(Debug, Error)]
enum HardeningError {
    #[cfg(all(unix, not(target_os = "ios"), not(miri)))]
    #[error("setrlimit failed: {0}")]
    Setrlimit(#[source] std::io::Error),
    #[cfg(all(unix, not(target_os = "ios"), not(miri)))]
    #[error("mlock failed: {0}")]
    Mlock(#[source] std::io::Error),
    #[cfg(all(unix, not(target_os = "ios"), not(miri)))]
    #[error("munlock failed: {0}")]
    Munlock(#[source] std::io::Error),
    #[cfg(all(any(target_os = "linux", target_os = "android"), not(miri)))]
    #[error("prctl failed: {0}")]
    Prctl(#[source] std::io::Error),
    #[cfg(target_os = "windows")]
    #[error("VirtualLock failed: {0}")]
    VirtualLock(#[source] std::io::Error),
    #[cfg(target_os = "windows")]
    #[error("VirtualUnlock failed: {0}")]
    VirtualUnlock(#[source] std::io::Error),
}

/// Summary of best-effort hardening operations.
#[derive(Debug, Default)]
pub struct HardeningReport {
    pub core_dumps_disabled: bool,
    pub non_dumpable: bool,
    pub notes: Vec<String>,
}

/// Apply process-level best-effort hardening.
///
/// Currently attempts:
/// - Disable core dumps (Unix only).
/// - Mark process non-dumpable (Linux/Android only; best-effort).
pub fn apply_best_effort() -> HardeningReport {
    let mut report = HardeningReport::default();

    match disable_core_dumps() {
        Ok(true) => report.core_dumps_disabled = true,
        Ok(false) => report.notes.push("core dumps: not supported".to_string()),
        Err(err) => report.notes.push(format!("core dumps: {err}")),
    }

    match set_non_dumpable() {
        Ok(true) => report.non_dumpable = true,
        Ok(false) => report.notes.push("non-dumpable: not supported".to_string()),
        Err(err) => report.notes.push(format!("non-dumpable: {err}")),
    }

    report
}

/// Return allocator-owned free heap pages to the OS when glibc supports it.
///
/// This is a process-local best-effort operation. It does not change sysctl,
/// swap, cgroup, systemd, or global OOM policy. A `false` result means either
/// that no pages were released or that the platform has no compatible API.
#[must_use]
pub fn trim_process_heap_best_effort() -> bool {
    #[cfg(all(target_os = "linux", target_env = "gnu", not(miri)))]
    {
        // SAFETY: glibc documents `malloc_trim` as MT-safe. Passing zero asks
        // the allocator to keep no additional top-of-heap padding.
        return unsafe { libc::malloc_trim(0) != 0 };
    }

    #[cfg(any(miri, not(target_os = "linux"), not(target_env = "gnu")))]
    {
        false
    }
}

fn disable_core_dumps() -> Result<bool, HardeningError> {
    // Conservative: iOS behavior/availability differs and is harder to validate in CI.
    #[cfg(all(unix, not(target_os = "ios"), not(miri)))]
    {
        // SAFETY: `setrlimit` is a C API that only reads the provided pointer.
        // We pass a valid pointer to a stack-allocated `rlimit`.
        let limit = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        let rc = unsafe { setrlimit(RLIMIT_CORE, &limit) };
        if rc == 0 {
            Ok(true)
        } else {
            Err(HardeningError::Setrlimit(std::io::Error::last_os_error()))
        }
    }

    #[cfg(any(miri, not(unix), target_os = "ios"))]
    {
        Ok(false)
    }
}

fn set_non_dumpable() -> Result<bool, HardeningError> {
    #[cfg(all(any(target_os = "linux", target_os = "android"), not(miri)))]
    {
        // SAFETY: `prctl(PR_SET_DUMPABLE, 0, ..)` is a process-level hardening knob.
        // The call doesn't dereference pointers; arguments are plain integers.
        let rc = unsafe { prctl(PR_SET_DUMPABLE, 0, 0, 0, 0) };
        if rc == 0 {
            return Ok(true);
        }

        Err(HardeningError::Prctl(std::io::Error::last_os_error()))
    }

    #[cfg(any(miri, not(any(target_os = "linux", target_os = "android"))))]
    {
        Ok(false)
    }
}
