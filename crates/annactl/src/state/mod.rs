//! State Module - Manage application state
//!
//! Beta.200: Clean state management
//!
//! This module is responsible for:
//! - Managing conversation history for TUI
//! - Tracking user preferences
//! - Persisting state across sessions
//! - Providing clean state access patterns

pub mod manager;

// Re-export key types for convenience
pub use manager::StateManager;
