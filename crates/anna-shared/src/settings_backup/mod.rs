// v0.0.575: Settings Backup Manager (Phase 151)
// Automated backup and restore of settings

mod types;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to maintain the same API
pub use types::{BackupType, BackupStatus, BackupMeta, BackupConfig};
pub use manager::BackupManager;
pub use utils::{format_backups, is_backup_query, settings_backup_fun_fact};
