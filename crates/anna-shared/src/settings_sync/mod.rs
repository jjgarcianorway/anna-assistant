// v0.0.564: Settings Sync (Phase 140)
// Handles synchronizing settings across multiple instances

mod types;
mod manager;
mod utils;

// Re-export all public types and functions to maintain the same API
pub use types::{ConflictResolution, SyncConfig, SyncProvider, SyncStatus};
pub use manager::SyncManager;
pub use utils::{format_sync_status, settings_sync_fun_fact};
