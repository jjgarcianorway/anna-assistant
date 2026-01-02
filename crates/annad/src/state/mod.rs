//! Daemon state management.

mod cache;
mod helpers;
mod llm;
mod status;
mod types;

// Re-export all public types and functions to preserve the original API
pub use helpers::load_initial_state;
pub use types::{DaemonStateInner, SharedState, UpdateStateInner, TRUTH_LEDGER_PATH};
