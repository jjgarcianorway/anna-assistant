//! Ticket Resolution Stats - Phase 86
//!
//! Tracks tickets closed by Anna vs specialists.
//! VISION.md: "Track amount of tickets closed by Anna vs specialists"
//! "Anna's ticket count should increase over time"

mod formatting;
mod stats;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public items to preserve the API
pub use formatting::{
    format_resolution_stats, format_resolution_stats_compact, format_resolution_stats_oneline,
};
pub use stats::TicketResolutionStats;
pub use types::{ResolutionMethod, ResolutionRecord, Resolver};
pub use utils::{is_resolution_stats_query, resolution_fun_fact};
