//! Hard Filesystem Locking for Updater v0.0.73
//!
//! Ensures only one updater can run at a time using:
//! - Filesystem lock file with PID and timestamp
//! - Stale lock detection and recovery
//! - Crash-safe design

use crate::ops_log::OpsLog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::UPDATE_LOCK_FILE;

/// Maximum age of a lock before it's considered stale (5 minutes)
const MAX_LOCK_AGE_SECS: u64 = 300;

/// Lock file contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    /// Process ID holding the lock
    pub pid: u32,
    /// Timestamp when lock was acquired (unix epoch seconds)
    pub acquired_at: u64,
    /// Hostname for debugging
    pub hostname: String,
    /// What step the updater was in when lock was acquired
    pub step: String,
}

impl LockInfo {
    fn new(step: &str) -> Self {
        // Get hostname from /etc/hostname or use "unknown"
        let hostname = fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            pid: process::id(),
            acquired_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            hostname,
            step: step.to_string(),
        }
    }

    fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.acquired_at)
    }

    fn is_stale(&self) -> bool {
        self.age_secs() > MAX_LOCK_AGE_SECS
    }

    fn process_exists(&self) -> bool {
        // Check if process exists by sending signal 0
        Path::new(&format!("/proc/{}", self.pid)).exists()
    }
}

/// Errors from lock operations
#[derive(Debug)]
pub enum UpdateLockError {
    /// Lock is held by another process
    AlreadyLocked { holder: LockInfo },
    /// IO error
    IoError(io::Error),
    /// Lock file is corrupted
    Corrupted(String),
}

impl std::fmt::Display for UpdateLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyLocked { holder } => {
                write!(
                    f,
                    "Update lock held by PID {} since {} seconds ago (step: {})",
                    holder.pid,
                    holder.age_secs(),
                    holder.step
                )
            }
            Self::IoError(e) => write!(f, "Lock IO error: {}", e),
            Self::Corrupted(msg) => write!(f, "Lock file corrupted: {}", msg),
        }
    }
}

impl std::error::Error for UpdateLockError {}

impl From<io::Error> for UpdateLockError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Update lock handle - releases lock on drop
pub struct UpdateLock {
    lock_path: String,
    ops_log: OpsLog,
}

impl UpdateLock {
    /// Attempt to acquire the update lock
    pub fn acquire(step: &str) -> Result<Self, UpdateLockError> {
        let lock_path = UPDATE_LOCK_FILE.to_string();
        let mut ops_log = OpsLog::open();

        // Ensure directory exists
        if let Some(parent) = Path::new(&lock_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if lock file exists
        if Path::new(&lock_path).exists() {
            let content = fs::read_to_string(&lock_path)?;
            match serde_json::from_str::<LockInfo>(&content) {
                Ok(holder) => {
                    // Check if lock is stale
                    if holder.is_stale() {
                        ops_log.log(
                            "updater",
                            "stale_lock_recovered",
                            Some(&format!("pid={} age={}s", holder.pid, holder.age_secs())),
                        );
                        // Remove stale lock
                        fs::remove_file(&lock_path)?;
                    } else if !holder.process_exists() {
                        ops_log.log(
                            "updater",
                            "dead_process_lock_recovered",
                            Some(&format!("pid={}", holder.pid)),
                        );
                        // Process is dead, remove lock
                        fs::remove_file(&lock_path)?;
                    } else {
                        // Lock is valid and held by another process
                        return Err(UpdateLockError::AlreadyLocked { holder });
                    }
                }
                Err(e) => {
                    ops_log.log("updater", "corrupted_lock_recovered", Some(&e.to_string()));
                    // Corrupted, remove it
                    fs::remove_file(&lock_path)?;
                }
            }
        }

        // Create new lock
        let info = LockInfo::new(step);
        let content = serde_json::to_string_pretty(&info)
            .map_err(|e| UpdateLockError::Corrupted(e.to_string()))?;

        let mut file = fs::File::create(&lock_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        ops_log.log(
            "updater",
            "lock_acquired",
            Some(&format!("pid={} step={}", info.pid, step)),
        );

        Ok(Self { lock_path, ops_log })
    }

    /// Update the step in the lock file (for debugging)
    pub fn update_step(&mut self, step: &str) -> io::Result<()> {
        if Path::new(&self.lock_path).exists() {
            let content = fs::read_to_string(&self.lock_path)?;
            if let Ok(mut info) = serde_json::from_str::<LockInfo>(&content) {
                info.step = step.to_string();
                let new_content = serde_json::to_string_pretty(&info)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                fs::write(&self.lock_path, new_content)?;
            }
        }
        Ok(())
    }

    /// Check if we still hold the lock
    pub fn is_valid(&self) -> bool {
        if let Ok(content) = fs::read_to_string(&self.lock_path) {
            if let Ok(info) = serde_json::from_str::<LockInfo>(&content) {
                return info.pid == process::id();
            }
        }
        false
    }
}

impl Drop for UpdateLock {
    fn drop(&mut self) {
        // Release lock on drop
        if self.is_valid() {
            if let Err(e) = fs::remove_file(&self.lock_path) {
                self.ops_log
                    .log("updater", "lock_release_error", Some(&e.to_string()));
            } else {
                self.ops_log.log("updater", "lock_released", None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper to use temp dir for tests
    fn with_temp_lock<F>(f: F)
    where
        F: FnOnce(&str),
    {
        let temp = TempDir::new().unwrap();
        let lock_path = temp.path().join("update.lock");
        f(lock_path.to_str().unwrap());
    }

    #[test]
    fn test_lock_info_creation() {
        let info = LockInfo::new("check_remote");
        assert_eq!(info.pid, process::id());
        assert_eq!(info.step, "check_remote");
        assert!(!info.is_stale());
    }

    #[test]
    fn test_lock_info_stale_detection() {
        let mut info = LockInfo::new("test");
        // Make it old
        info.acquired_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (MAX_LOCK_AGE_SECS + 100);
        assert!(info.is_stale());
    }

    #[test]
    fn test_lock_info_process_check() {
        let info = LockInfo::new("test");
        // Our own process should exist
        assert!(info.process_exists());

        // PID 999999 shouldn't exist (probably)
        let mut fake = info.clone();
        fake.pid = 999999;
        assert!(!fake.process_exists());
    }
}
