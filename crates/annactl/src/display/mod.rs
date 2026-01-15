//! Display module for annactl TUI output.
//! Split into sub-modules to maintain <400 line files.

pub mod colors;
pub mod formatting;
pub mod status;
pub mod status_detail;
pub mod step;
pub mod stats_cmd;
pub mod alerts;
mod welcome;
#[cfg(test)]
mod snapshot_tests;

// Re-export commonly used items
pub use colors::*;
pub use formatting::{format_duration, format_time_ago, is_debug_mode};
pub use status::print_status;
pub use step::{print_step, print_dialogue, print_timeout_error};
pub use stats_cmd::print_stats;
pub use alerts::{show_proactive_alerts, mark_alerts_shown};
pub use welcome::print_greeting;
