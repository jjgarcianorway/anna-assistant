//! Windowed Reliability Statistics - v0.0.438.
//!
//! Track stats across time windows for detecting degradation.

use super::stats::ReliabilityStats;
use super::types::ExecutionRecord;

/// Stats for a specific time window.
#[derive(Debug, Clone)]
pub struct WindowedStats {
    /// Stats for current window.
    pub current: ReliabilityStats,
    /// Stats for previous window.
    pub previous: ReliabilityStats,
    /// All-time stats.
    pub all_time: ReliabilityStats,
}

impl WindowedStats {
    /// Create new windowed stats.
    pub fn new() -> Self {
        Self {
            current: ReliabilityStats::new(),
            previous: ReliabilityStats::new(),
            all_time: ReliabilityStats::new(),
        }
    }

    /// Record to current window and all-time.
    pub fn record(&mut self, record: &ExecutionRecord) {
        self.current.record(record);
        self.all_time.record(record);
    }

    /// Rotate window (move current to previous, reset current).
    pub fn rotate_window(&mut self) {
        self.previous = self.current.clone();
        self.current.reset();
    }

    /// Check if degrading (current worse than previous).
    pub fn is_degrading(&self) -> bool {
        self.current.success_rate() < self.previous.success_rate() * 0.9
    }
}

impl Default for WindowedStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fast_pipeline::reliability_stats::ReliabilityOutcome;

    #[test]
    fn test_windowed_stats() {
        let mut stats = WindowedStats::new();

        stats.record(&ExecutionRecord::new(
            ReliabilityOutcome::SpecialistSuccess,
            500,
        ));
        assert_eq!(stats.current.total_executions, 1);
        assert_eq!(stats.all_time.total_executions, 1);

        stats.rotate_window();
        assert_eq!(stats.previous.total_executions, 1);
        assert_eq!(stats.current.total_executions, 0);
    }
}
