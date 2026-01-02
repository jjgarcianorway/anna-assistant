//! Error Recovery Tracker - Phase 97
//!
//! Tracks error recovery attempts and success rates.
//! Helps Anna learn which recovery strategies work best.

mod formatting;
mod tracker;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export types
pub use types::{ErrorCategory, ErrorRecoveryRecord, RecoveryOutcome};

// Re-export tracker
pub use tracker::ErrorRecoveryTracker;

// Re-export formatting functions
pub use formatting::{
    format_error_recovery_tracker, format_error_recovery_tracker_compact,
    format_error_recovery_tracker_oneline,
};

// Re-export utility functions
pub use utils::{error_recovery_fun_fact, is_error_recovery_query};
