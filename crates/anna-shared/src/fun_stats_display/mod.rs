//! Fun Statistics Display (v0.0.479).
//!
//! Formats and displays fun/interesting statistics about Anna usage
//! as specified in VISION.md's "Fun Statistics" section.
//!
//! Features:
//! - Most consulted team
//! - Repeated questions
//! - Topic most asked about
//! - Longest/shortest reply
//! - Installation date
//! - And more interesting data

mod category;
mod formatters;
mod fun_facts;
mod query_detection;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use category::{format_fun_stats_category, FunStatsCategory};
pub use formatters::{format_duration, format_fun_stats, format_fun_stats_compact, format_install_date};
pub use fun_facts::generate_fun_fact;
pub use query_detection::is_fun_stats_query;
pub use types::FunStats;
