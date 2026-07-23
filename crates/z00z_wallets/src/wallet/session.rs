//! Session management for secret access.
//!
//! Provides time-limited access tokens for unlocked wallets.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Secret session handle.
///
/// Represents an active unlocked session with time-based expiration.
///
/// # Security
///
/// - Sessions expire after inactivity timeout
/// - Session tokens are unique per unlock operation
/// - Tokens are NOT cryptographic keys (used for session tracking only)
///
/// # Examples
///
/// ```
/// use z00z_wallets::wallet::SessionHandle;
///
/// let session = SessionHandle::new(1234567890);
/// let timeout_ms = 60_000;
/// assert!(!session.is_expired(1234567890, timeout_ms));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionHandle {
    /// Unique session identifier
    pub token: Uuid,

    /// Creation timestamp (Unix milliseconds)
    pub created_at: u64,

    /// Last activity timestamp (Unix milliseconds)
    pub last_activity_ms: u64,
}

impl SessionHandle {
    /// Create new session handle.
    ///
    /// # Arguments
    ///
    /// * `now_ms` - Current timestamp in Unix milliseconds
    ///
    /// # Examples
    ///
    /// ```
    /// use z00z_wallets::wallet::SessionHandle;
    ///
    /// let now = 1234567890;
    /// let session = SessionHandle::new(now);
    /// assert_eq!(session.created_at, now);
    /// ```
    pub fn new(now_ms: u64) -> Self {
        Self {
            token: Uuid::new_v4(),
            created_at: now_ms,
            last_activity_ms: now_ms,
        }
    }

    /// Update activity timestamp.
    ///
    /// # Arguments
    ///
    /// * `now_ms` - Current timestamp in Unix milliseconds
    pub fn update_activity(&mut self, now_ms: u64) {
        self.last_activity_ms = now_ms;
    }

    /// Check if session is expired.
    ///
    /// # Arguments
    ///
    /// * `now_ms` - Current timestamp in Unix milliseconds
    /// * `timeout_ms` - Inactivity timeout in milliseconds
    ///
    /// # Returns
    ///
    /// Returns `true` if session has exceeded inactivity timeout.
    ///
    /// # Examples
    ///
    /// ```
    /// use z00z_wallets::wallet::SessionHandle;
    ///
    /// let mut session = SessionHandle::new(1000);
    /// let timeout = 5000; // 5 seconds
    ///
    /// // Not expired (within timeout)
    /// assert!(!session.is_expired(2000, timeout));
    ///
    /// // Expired (exceeded timeout)
    /// assert!(session.is_expired(7000, timeout));
    /// ```
    pub fn is_expired(&self, now_ms: u64, timeout_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_activity_ms) > timeout_ms
    }
}

/// Secret session data.
///
/// Contains decrypted secrets for active session.
#[derive(Debug)]
pub struct SecretSession {
    /// Session handle
    pub handle: SessionHandle,

    /// Encrypted master key (not exposed, kept in memory)
    master_key: Vec<u8>,
}

impl SecretSession {
    /// Create new secret session.
    ///
    /// # Arguments
    ///
    /// * `handle` - Session handle
    /// * `master_key` - Encrypted master key bytes
    pub fn new(handle: SessionHandle, master_key: Vec<u8>) -> Self {
        Self { handle, master_key }
    }
}

impl Drop for SecretSession {
    fn drop(&mut self) {
        use zeroize::Zeroize;

        self.master_key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_handle_creation() {
        let now = 1234567890;
        let session = SessionHandle::new(now);

        assert_eq!(session.created_at, now);
        assert_eq!(session.last_activity_ms, now);
    }

    #[test]
    fn test_session_not_expired() {
        let session = SessionHandle::new(1000);
        let timeout = 5000; // 5 seconds

        // Within timeout
        assert!(!session.is_expired(2000, timeout));
        assert!(!session.is_expired(5999, timeout));
    }

    #[test]
    fn test_session_expired() {
        let session = SessionHandle::new(1000);
        let timeout = 5000; // 5 seconds

        // Exceeded timeout
        assert!(session.is_expired(7000, timeout));
        assert!(session.is_expired(10000, timeout));
    }

    #[test]
    fn test_update_activity() {
        let mut session = SessionHandle::new(1000);
        let timeout = 5000;

        // Update activity
        session.update_activity(3000);
        assert_eq!(session.last_activity_ms, 3000);

        // Not expired after update
        assert!(!session.is_expired(4000, timeout));
    }
}
