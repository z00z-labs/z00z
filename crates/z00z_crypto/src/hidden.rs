//! Enhanced Hidden type with safer API for sensitive data handling.
//!
//! This module provides an enhanced version of `Hidden<T>` that includes:
//! - `with_revealed`-style accessors to avoid ad-hoc copies
//! - Protection against accidental cloning
//! - Zeroization on drop
//!
//! # Usage
//!
//! ```ignore
//! use z00z_crypto::hidden::Hidden;
//!
//! // Create hidden data
//! let secret = Hidden::hide(vec![1u8, 2u8, 3u8]);
//!
//! // Access safely with with_revealed
//! let result = secret.with_revealed(|data| {
//!     // data is &Vec<u8>
//!     data.len()
//! });
//!
//! // Clone is safe
//! let clone = secret.clone();
//! ```

use zeroize::Zeroize;

/// Enhanced Hidden type with safer API for sensitive data handling.
///
/// This wrapper around `tari_crypto::Hidden<T>` provides:
/// - `with_revealed`-style accessors to avoid ad-hoc copies
/// - Protection against accidental cloning
/// - Zeroization on drop
pub struct Hidden<T: Zeroize> {
    inner: tari_crypto::tari_utilities::Hidden<T>,
}

impl<T: Zeroize> Hidden<T> {
    /// Create new hidden data from the underlying type
    pub fn hide(inner: T) -> Self {
        Self {
            inner: tari_crypto::tari_utilities::Hidden::hide(inner),
        }
    }

    /// Reveal the hidden data as an immutable reference
    pub fn reveal(&self) -> &T {
        self.inner.reveal()
    }

    /// Reveal the hidden data as a mutable reference.
    pub fn reveal_mut(&mut self) -> &mut T {
        self.inner.reveal_mut()
    }

    /// Access the hidden data safely with a closure.
    ///
    /// This method prevents accidental copying by requiring the caller
    /// to provide a closure that operates on a reference to the data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use z00z_crypto::hidden::Hidden;
    ///
    /// let secret = Hidden::hide(vec![1u8, 2u8, 3u8]);
    /// let len = secret.with_revealed(|data| data.len());
    /// assert_eq!(len, 3);
    /// ```
    pub fn with_revealed<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(self.inner.reveal())
    }

    /// Access the hidden value mutably within a narrowly scoped closure.
    ///
    /// This is the mutable counterpart of [`Self::with_revealed`].  It keeps
    /// callers from retaining a raw mutable reference outside the operation
    /// that needs to initialise or scrub a hidden buffer.
    pub fn with_revealed_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self.inner.reveal_mut())
    }

    /// DANGEROUS: Creates a copy of hidden data.
    ///
    /// # Security Warning
    ///
    /// The copy will be independently zeroized on drop.
    /// Only use when absolutely necessary for key derivation or similar operations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use z00z_crypto::hidden::Hidden;
    ///
    /// let secret = Hidden::hide(vec![1u8, 2u8, 3u8]);
    /// // Only clone when needed
    /// let derived = secret.dangerous_clone();
    /// ```
    #[must_use]
    pub fn dangerous_clone(&self) -> Self
    where
        T: Clone,
    {
        Self {
            inner: tari_crypto::tari_utilities::Hidden::hide(self.inner.reveal().clone()),
        }
    }
}

impl<T: Zeroize> From<T> for Hidden<T> {
    fn from(inner: T) -> Self {
        Self::hide(inner)
    }
}

impl<T: Zeroize> core::fmt::Debug for Hidden<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Hidden<{}>:[REDACTED]", core::any::type_name::<T>())
    }
}

impl<T: Zeroize> core::fmt::Display for Hidden<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Hidden<{}>:[REDACTED]", core::any::type_name::<T>())
    }
}

impl<T: Zeroize> Zeroize for Hidden<T> {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

impl<T: Zeroize> Drop for Hidden<T> {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hidden_basic() {
        let secret = Hidden::hide(vec![1u8, 2u8, 3u8]);
        let len = secret.with_revealed(|data| data.len());
        assert_eq!(len, 3);
    }

    #[test]
    fn test_hidden_mutation_stays_scoped() {
        let mut secret = Hidden::hide(vec![1u8, 2u8, 3u8]);
        secret.with_revealed_mut(|data| data[1] = 9);
        assert_eq!(secret.with_revealed(|data| data[1]), 9);
    }

    #[test]
    fn test_hidden_dangerous_clone() {
        let secret1 = Hidden::hide(vec![1u8, 2u8, 3u8]);
        // SAFETY: Clone needed for test verification
        let secret2 = secret1.dangerous_clone();

        let len1 = secret1.with_revealed(|data| data.len());
        let len2 = secret2.with_revealed(|data| data.len());
        assert_eq!(len1, len2);
    }

    #[test]
    fn test_hidden_debug_redacted() {
        let secret = Hidden::hide(vec![42u8, 43u8, 44u8]);
        let debug_output = format!("{:?}", secret);

        // Verify Debug output is redacted
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("42"));
        assert!(!debug_output.contains("43"));
    }

    #[test]
    fn test_hidden_display_redacted() {
        let secret = Hidden::hide(String::from("sensitive_password"));
        let display_output = format!("{}", secret);

        // Verify Display output is redacted
        assert!(display_output.contains("[REDACTED]"));
        assert!(!display_output.contains("password"));
    }
}
