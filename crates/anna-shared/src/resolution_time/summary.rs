//! Resolution time summary types and generation.

use serde::{Deserialize, Serialize};

use super::stats::ResolutionTimeTracker;

/// Resolution time summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionTimeSummary {
    /// Total resolutions
    pub total: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average time (ms)
    pub avg_ms: f64,
    /// Fastest (ms)
    pub fastest_ms: u64,
    /// Slowest (ms)
    pub slowest_ms: u64,
    /// Escalation rate percentage
    pub escalation_rate: f64,
}

impl ResolutionTimeTracker {
    /// Generate summary
    pub fn summary(&self) -> ResolutionTimeSummary {
        ResolutionTimeSummary {
            total: self.total_resolutions,
            success_rate: self.success_rate(),
            avg_ms: self.average_ms(),
            fastest_ms: self.fastest.as_ref().map(|f| f.duration_ms).unwrap_or(0),
            slowest_ms: self.slowest.as_ref().map(|s| s.duration_ms).unwrap_or(0),
            escalation_rate: self.escalation_rate(),
        }
    }
}
