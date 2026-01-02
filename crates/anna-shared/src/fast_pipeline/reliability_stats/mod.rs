//! Reliability Stats (Part H) - v0.0.438.
//!
//! Track reality-based reliability metrics:
//! - specialist_timeouts: How often specialists time out
//! - parser_failures: How often we fail to parse responses
//! - fallback_rate: How often we fall back to probe-only
//! - avg_response_time: Average time to answer
//!
//! These stats should be logged and used for monitoring.

mod stats;
mod types;
mod windowed;

pub use stats::ReliabilityStats;
pub use types::{ExecutionRecord, ReliabilityOutcome};
pub use windowed::WindowedStats;
