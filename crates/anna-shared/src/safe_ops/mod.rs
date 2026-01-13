//! Safe operations with backup and rollback capability.
//! v0.3.21: Reset modes with automatic backups per reliability sprint spec.

pub mod backup_types;
pub mod backup_utils;
mod file_ops;
mod safe_reset;

// Re-exports
pub use backup_types::{BackupInfo, BackupLedger, FileBackup};
pub use backup_utils::{backup_dir, backup_file, create_backup_dir};
pub use file_ops::{
    backup_single_file, cleanup_old_backups, list_recent_backups, rollback_file, safe_append,
    safe_write, verify_file_contains, verify_file_not_contains,
};
pub use safe_reset::SafeReset;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_extended;
