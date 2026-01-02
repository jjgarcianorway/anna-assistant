//! Backup History Tracking - Phase 79
//!
//! Tracks backups created by Anna when making changes to files/configs.
//! Critical for the undo/rollback functionality mentioned in VISION.md.

mod formatting;
mod storage;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types
pub use formatting::{
    backup_fun_fact, format_backup_history, format_backup_history_compact,
    format_backup_history_oneline, format_size,
};
pub use storage::BackupHistory;
pub use types::{BackupRecord, BackupStatus, BackupType};
pub use utils::is_backup_query;
