//! Updater v0.0.73 - Reliable Auto-Update System
//!
//! Non-negotiable guarantees:
//! 1. Single source of truth state in /var/lib/anna/internal/update_state.json
//! 2. Hard filesystem locking - only one updater can run
//! 3. Atomic installs - never half-written binaries
//! 4. Correct restart semantics with verification
//! 5. Automatic rollback on failure
//! 6. No silent failures - everything logged
//!
//! State machine steps:
//! 1. acquire_lock
//! 2. check_remote_release
//! 3. compare_versions
//! 4. download_assets
//! 5. verify_assets
//! 6. install_cli (optional)
//! 7. install_daemon (optional)
//! 8. restart_daemon (optional)
//! 9. healthcheck
//! 10. release_lock
//! 11. rollback (on failure)

pub mod lock;
pub mod state;
pub mod steps;

// Re-exports
pub use lock::{UpdateLock, UpdateLockError};
pub use state::{UpdateStateV73, UpdateStep, UpdateStepResult};
pub use steps::{UpdateExecutor, UpdateResult};

/// Lock file path
pub const UPDATE_LOCK_FILE: &str = "/var/lib/anna/internal/update.lock";

/// State file path
pub const UPDATE_STATE_FILE: &str = "/var/lib/anna/internal/update_state.json";

/// Backup directory for rollback
pub const UPDATE_BACKUP_DIR: &str = "/var/lib/anna/internal/backups";

/// Ops log path
pub const OPS_LOG_FILE: &str = "/var/lib/anna/internal/ops.log";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_constants() {
        assert!(UPDATE_LOCK_FILE.starts_with("/var/lib/anna"));
        assert!(UPDATE_STATE_FILE.ends_with(".json"));
    }
}
