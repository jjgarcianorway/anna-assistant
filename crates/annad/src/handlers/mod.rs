//! Utility RPC handlers for status, probes, reset, uninstall, autofix, and stats.
//! v0.0.404: Reset now clears event log for consistent stats.
//!
//! This module is organized into submodules:
//! - types: Common types and imports
//! - status: Status, progress, stats, and daemon info handlers
//! - probe: System probe handlers
//! - state_management: Reset, uninstall, and autofix handlers
//! - change_engine: Change plan, apply, and rollback handlers
//! - greeting: LLM-generated greeting handler
//! - command: Command execution handler

mod types;
mod status;
mod probe;
mod state_management;
mod change_engine;
mod greeting;
mod command;

// Re-export all public handlers to maintain the existing API
pub use status::{
    handle_status,
    handle_progress,
    handle_stats,
    handle_status_snapshot,
    handle_get_daemon_info,
};

pub use probe::handle_probe;

pub use state_management::{
    handle_reset,
    handle_uninstall,
    handle_autofix,
};

pub use change_engine::{
    handle_plan_change,
    handle_apply_change,
    handle_rollback_change,
};

pub use greeting::handle_generate_greeting;

pub use command::handle_execute_command;
