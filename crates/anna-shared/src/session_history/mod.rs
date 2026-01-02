//! Session History Tracker - Phase 94
//!
//! Tracks user session history with Anna.
//! Useful for "what did I do last time" queries.

mod types;
mod tracker;
mod formatting;
mod utils;

// Re-export all public types and functions to preserve the original API
pub use types::{SessionOutcome, SessionRecord, SessionType};
pub use tracker::SessionHistoryTracker;
pub use formatting::{
    format_session_history,
    format_session_history_compact,
    format_session_history_oneline,
};
pub use utils::{is_session_query, session_fun_fact};
