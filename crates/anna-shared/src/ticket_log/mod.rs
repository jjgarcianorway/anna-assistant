//! Structured ticket logs for learning (v0.0.406).
//!
//! Stores solved tickets in a consistent format for:
//! - Recipe generation (future)
//! - Pattern analysis
//! - LLM load measurement
//! - v0.0.407: Truthful stats with explicit state tracking
//!
//! Storage: ~/.anna/tickets/{id}.json or /var/lib/anna/tickets/{id}.json

mod types;
mod implementations;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{
    CommandLog,
    DocSnippet,
    ProbeLog,
    SolverOutput,
    TicketLog,
    TicketResult,
};

// Re-export utility functions
pub use utils::{
    chrono_timestamp,
    load_all_tickets,
    load_recent_tickets,
    ticket_log_dir,
};

// Re-export stats from ticket_stats module for backward compatibility
pub use crate::ticket_stats::{calculate_stats, TicketStats};
