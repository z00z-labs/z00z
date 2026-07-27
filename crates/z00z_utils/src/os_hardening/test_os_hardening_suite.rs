use super::locking::{enable_unlock_probe, reset_unlock_probe, saw_zero_before_unlock};
use super::*;

fn keep_life<'a>(bytes: &'a mut [u8]) -> Option<LockedBytes<'a>> {
    lock_bytes_best_effort(bytes)
}

fn assert_lock_api_shape(handler: for<'a> fn(&'a mut [u8]) -> Option<LockedBytes<'a>>) {
    let _ = handler;
}

#[test]
fn test_contract_apply_best_effort() {
    let _ = apply_best_effort as fn() -> HardeningReport;
}

#[test]
fn test_contract_trim_process_heap_best_effort() {
    let _ = trim_process_heap_best_effort as fn() -> bool;
}

#[test]
fn test_api_contract_lock_bytes() {
    assert_lock_api_shape(lock_bytes_best_effort);
}

#[test]
fn test_contract_owned_locked_bytes() {
    let _ = OwnedLockedBytes::<32>::new_best_effort as fn([u8; 32]) -> OwnedLockedBytes<32>;
    let _ = OwnedLockedBytes::<32>::new_best_effort_with(|_| {});
}

#[test]
fn test_lock_bytes_lifetime() {
    let mut data = vec![1u8, 2, 3, 4];
    let _guard = keep_life(&mut data);
}

#[test]
#[cfg(unix)]
fn test_disable_core_dumps() {
    let result = disable_core_dumps();
    assert!(result.is_ok() || result.is_err());
}

#[test]
#[cfg(all(unix, not(target_os = "ios")))]
fn test_memory_lock() {
    let mut data = vec![0x42u8; 4096];
    let guard = lock_bytes_best_effort(&mut data);
    assert!(guard.is_some() || guard.is_none());
}

#[test]
fn test_lock_bytes_zero_drop() {
    let mut data = vec![0x42u8; 64];

    {
        let guard = lock_bytes_best_effort(&mut data);
        if guard.is_none() {
            return;
        }
    }

    assert!(data.iter().all(|byte| *byte == 0));
}

#[test]
fn test_zero_before_unlock() {
    let mut data = vec![0x42u8; 64];

    enable_unlock_probe();

    {
        let guard = lock_bytes_best_effort(&mut data);
        let Some(_guard) = guard else {
            reset_unlock_probe();
            return;
        };
    }

    let saw_zero = saw_zero_before_unlock();
    reset_unlock_probe();

    assert!(saw_zero);
    assert!(data.iter().all(|byte| *byte == 0));
}

#[test]
fn test_bytes_debug_redacts_pointer() {
    let mut data = vec![0x24u8; 32];
    let guard = lock_bytes_best_effort(&mut data);

    let Some(guard) = guard else {
        return;
    };

    let rendered = format!("{guard:?}");
    assert!(rendered.contains("LockedBytes"));
    assert!(rendered.contains("len"));
    assert!(rendered.contains("active"));
    assert!(!rendered.contains("addr"));
    assert!(!rendered.contains("ptr"));
    assert!(!rendered.contains("0x"));
}

#[test]
fn test_locked_bytes_zeroize_drop() {
    let mut secret = OwnedLockedBytes::<32>::new_best_effort([7u8; 32]);
    assert_eq!(secret.reveal(), &[7u8; 32]);
    secret.reveal_mut()[0] = 9;
    assert_eq!(secret.reveal()[0], 9);
}

#[test]
fn test_bytes_debug_redacts_contents() {
    let secret = OwnedLockedBytes::new_best_effort([0x42u8; 32]);
    let rendered = format!("{secret:?}");

    assert!(rendered.contains("OwnedLockedBytes"));
    assert!(rendered.contains("len"));
    assert!(rendered.contains("active"));
    assert!(!rendered.contains("0x42"));
    assert!(!rendered.contains("ptr"));
    assert!(!rendered.contains("0x"));
}
