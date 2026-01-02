//! Response Length Tracking (v0.0.486).
//!
//! Tracks response lengths for fun statistics.
//! Identifies longest and shortest replies.

mod formatting;
mod tracker;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API to preserve backward compatibility
pub use formatting::{
    format_response_lengths, format_response_lengths_compact, response_length_fun_fact,
};
pub use tracker::ResponseLengthTracker;
pub use types::{RecordedResponse, ResponseLengthSummary};
pub use utils::is_response_length_query;
