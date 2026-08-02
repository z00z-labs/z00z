use core::{marker::PhantomData, ptr::NonNull};
use zeroize::Zeroize;

#[cfg(test)]
use std::cell::Cell;

#[cfg(all(
    not(miri),
    any(all(unix, not(target_os = "ios")), target_os = "windows")
))]
use super::HardeningError;

#[cfg(all(unix, not(target_os = "ios"), not(miri)))]
use libc::{mlock, munlock};

struct LockState {
    ptr: NonNull<u8>,
    len: usize,
    active: bool,
}

#[cfg(test)]
thread_local! {
    static UNLOCK_PROBE: Cell<bool> = const { Cell::new(false) };
    static UNLOCK_ZERO: Cell<bool> = const { Cell::new(false) };
}

#[cfg(test)]
pub(super) fn reset_unlock_probe() {
    UNLOCK_PROBE.with(|probe| probe.set(false));
    UNLOCK_ZERO.with(|state| state.set(false));
}

#[cfg(test)]
pub(super) fn enable_unlock_probe() {
    UNLOCK_PROBE.with(|probe| probe.set(true));
}

#[cfg(test)]
pub(super) fn saw_zero_before_unlock() -> bool {
    UNLOCK_ZERO.with(Cell::get)
}

#[cfg(test)]
fn probe_unlock(ptr: NonNull<u8>, len: usize) {
    UNLOCK_PROBE.with(|probe| {
        if !probe.get() || len == 0 {
            return;
        }

        let is_zero = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }
            .iter()
            .all(|byte| *byte == 0);

        UNLOCK_ZERO.with(|state| state.set(is_zero));
    });
}

impl LockState {
    fn new(bytes: &mut [u8], active: bool) -> Self {
        let ptr = NonNull::new(bytes.as_mut_ptr()).unwrap_or_else(NonNull::dangling);
        Self {
            ptr,
            len: bytes.len(),
            active,
        }
    }

    fn try_lock(bytes: &mut [u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        #[cfg(miri)]
        {
            Some(Self::new(bytes, true))
        }

        #[cfg(not(miri))]
        {
            #[cfg(all(unix, not(target_os = "ios")))]
            {
                let ptr = bytes.as_mut_ptr() as *mut core::ffi::c_void;
                if unsafe { mlock(ptr, bytes.len()) } == 0 {
                    return Some(Self::new(bytes, true));
                }

                let _ = HardeningError::Mlock(std::io::Error::last_os_error());
            }

            #[cfg(target_os = "windows")]
            {
                use windows_sys::Win32::System::Memory::VirtualLock;

                let ptr = bytes.as_mut_ptr() as *mut core::ffi::c_void;
                if unsafe { VirtualLock(ptr, bytes.len()) } != 0 {
                    return Some(Self::new(bytes, true));
                }

                let _ = HardeningError::VirtualLock(std::io::Error::last_os_error());
            }

            #[cfg(all(not(unix), not(target_os = "windows")))]
            {
                let _ = bytes;
            }

            None
        }
    }

    fn zeroize_bytes(&mut self) {
        if self.len == 0 {
            return;
        }

        unsafe {
            let bytes = std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len);
            bytes.zeroize();
        }
    }

    fn unlock_only(&mut self) {
        if !self.active || self.len == 0 {
            self.active = false;
            return;
        }

        #[cfg(test)]
        probe_unlock(self.ptr, self.len);

        #[cfg(not(miri))]
        #[cfg(all(unix, not(target_os = "ios")))]
        {
            let ptr = self.ptr.as_ptr() as *mut core::ffi::c_void;
            if unsafe { munlock(ptr, self.len) } != 0 {
                let _ = HardeningError::Munlock(std::io::Error::last_os_error());
            }
        }

        #[cfg(not(miri))]
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::System::Memory::VirtualUnlock;

            let ptr = self.ptr.as_ptr() as *mut core::ffi::c_void;
            if unsafe { VirtualUnlock(ptr, self.len) } == 0 {
                let _ = HardeningError::VirtualUnlock(std::io::Error::last_os_error());
            }
        }

        self.active = false;
    }

    fn zeroize_and_unlock(&mut self) {
        self.zeroize_bytes();
        self.unlock_only();
    }
}

#[must_use = "keep the guard alive for as long as the buffer must stay locked"]
pub struct LockedBytes<'a> {
    lock: LockState,
    _borrow: PhantomData<&'a mut [u8]>,
}

unsafe impl Send for LockedBytes<'_> {}
unsafe impl Sync for LockedBytes<'_> {}

#[must_use = "keep the wrapper alive for as long as the secret must stay locked"]
pub struct OwnedLockedBytes<const N: usize> {
    bytes: Box<[u8; N]>,
    lock: LockState,
}

unsafe impl<const N: usize> Send for OwnedLockedBytes<N> {}
unsafe impl<const N: usize> Sync for OwnedLockedBytes<N> {}

impl std::fmt::Debug for LockedBytes<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LockedBytes")
            .field("len", &self.lock.len)
            .field("active", &self.lock.active)
            .finish()
    }
}

impl<const N: usize> std::fmt::Debug for OwnedLockedBytes<N> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OwnedLockedBytes")
            .field("len", &self.bytes.len())
            .field("active", &self.lock.active)
            .finish()
    }
}

impl<const N: usize> OwnedLockedBytes<N> {
    pub fn new_best_effort_with(fill: impl FnOnce(&mut [u8; N])) -> Self {
        let mut bytes = Box::new([0u8; N]);
        let lock = match LockState::try_lock(bytes.as_mut()) {
            Some(lock) => lock,
            None => LockState::new(bytes.as_mut(), false),
        };

        let mut secret = Self { bytes, lock };
        fill(secret.reveal_mut());
        secret
    }

    pub fn new_best_effort(bytes: [u8; N]) -> Self {
        Self::new_best_effort_with(|slot| *slot = bytes)
    }

    pub fn reveal(&self) -> &[u8; N] {
        self.bytes.as_ref()
    }

    pub fn reveal_mut(&mut self) -> &mut [u8; N] {
        self.bytes.as_mut()
    }
}

impl<const N: usize> Drop for OwnedLockedBytes<N> {
    fn drop(&mut self) {
        self.bytes.as_mut().zeroize();
        self.lock.unlock_only();
    }
}

#[must_use = "dropping the guard immediately unlocks and zeroizes the buffer"]
pub fn lock_bytes_best_effort<'a>(bytes: &'a mut [u8]) -> Option<LockedBytes<'a>> {
    LockState::try_lock(bytes).map(|lock| LockedBytes {
        lock,
        _borrow: PhantomData,
    })
}

impl Drop for LockedBytes<'_> {
    fn drop(&mut self) {
        self.lock.zeroize_and_unlock();
    }
}
