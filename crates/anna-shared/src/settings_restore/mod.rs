// v0.0.576: Settings Restore (Phase 152)
// Restore settings from backups with validation
//
// This module is organized into:
// - types: Core types and enums for restore operations
// - manager: RestoreManager for managing restore operations
// - helpers: Utility functions for formatting and queries

mod types;
mod manager;
mod helpers;

// Re-export all public types to preserve the original API
pub use types::{
    RestoreMode,
    RestoreStatus,
    RestoreValidation,
    RestorePoint,
    RestoreRecord,
};

pub use manager::RestoreManager;

pub use helpers::{
    format_restore_history,
    is_restore_query,
    settings_restore_fun_fact,
};
