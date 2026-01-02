//! Reliability Statistics Tracker - v0.0.438.
//!
//! Track reality-based reliability metrics:
//! - specialist_timeouts: How often specialists time out
//! - parser_failures: How often we fail to parse responses
//! - fallback_rate: How often we fall back to probe-only
//! - avg_response_time: Average time to answer

use super::types::{ExecutionRecord, ReliabilityOutcome};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Reliability statistics tracker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityStats {
    /// Total executions.
    pub total_executions: usize,
    /// Successful executions.
    pub successful: usize,
    /// Specialist timeouts.
    pub specialist_timeouts: usize,
    /// Parser failures.
    pub parser_failures: usize,
    /// Fallback to probes.
    pub fallback_count: usize,
    /// Total failures.
    pub total_failures: usize,
    /// Success after retry.
    pub success_after_retry: usize,
    /// Total retry attempts.
    pub total_retries: usize,
    /// Sum of durations for averaging.
    pub duration_sum_ms: u64,
    /// Min duration.
    pub min_duration_ms: Option<u64>,
    /// Max duration.
    pub max_duration_ms: Option<u64>,
}

impl ReliabilityStats {
    /// Create new stats tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an execution.
    pub fn record(&mut self, record: &ExecutionRecord) {
        self.total_executions += 1;
        self.duration_sum_ms += record.duration_ms;
        self.total_retries += record.retry_attempts;

        // Update min/max
        match self.min_duration_ms {
            Some(min) if record.duration_ms < min => {
                self.min_duration_ms = Some(record.duration_ms);
            }
            None => {
                self.min_duration_ms = Some(record.duration_ms);
            }
            _ => {}
        }

        match self.max_duration_ms {
            Some(max) if record.duration_ms > max => {
                self.max_duration_ms = Some(record.duration_ms);
            }
            None => {
                self.max_duration_ms = Some(record.duration_ms);
            }
            _ => {}
        }

        // Count by outcome
        match record.outcome {
            ReliabilityOutcome::SpecialistSuccess => {
                self.successful += 1;
            }
            ReliabilityOutcome::SpecialistTimeout => {
                self.specialist_timeouts += 1;
            }
            ReliabilityOutcome::ParserFailure => {
                self.parser_failures += 1;
            }
            ReliabilityOutcome::FallbackToProbes => {
                self.fallback_count += 1;
            }
            ReliabilityOutcome::TotalFailure => {
                self.total_failures += 1;
            }
            ReliabilityOutcome::SuccessAfterRetry => {
                self.successful += 1;
                self.success_after_retry += 1;
            }
        }
    }

    /// Get success rate (0.0-1.0).
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_executions as f64
        }
    }

    /// Get timeout rate (0.0-1.0).
    pub fn timeout_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.specialist_timeouts as f64 / self.total_executions as f64
        }
    }

    /// Get parser failure rate (0.0-1.0).
    pub fn parser_failure_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.parser_failures as f64 / self.total_executions as f64
        }
    }

    /// Get fallback rate (0.0-1.0).
    pub fn fallback_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.fallback_count as f64 / self.total_executions as f64
        }
    }

    /// Get average response time.
    pub fn avg_response_time_ms(&self) -> u64 {
        if self.total_executions == 0 {
            0
        } else {
            self.duration_sum_ms / self.total_executions as u64
        }
    }

    /// Get average response time as Duration.
    pub fn avg_response_time(&self) -> Duration {
        Duration::from_millis(self.avg_response_time_ms())
    }

    /// Get retry rate (avg retries per execution).
    pub fn retry_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.total_retries as f64 / self.total_executions as f64
        }
    }

    /// Format as summary string.
    pub fn summary(&self) -> String {
        format!(
            "Reliability: {:.1}% success | {:.1}% timeout | {:.1}% parse_fail | {:.1}% fallback | avg {}ms",
            self.success_rate() * 100.0,
            self.timeout_rate() * 100.0,
            self.parser_failure_rate() * 100.0,
            self.fallback_rate() * 100.0,
            self.avg_response_time_ms()
        )
    }

    /// Check if stats indicate a problem.
    pub fn has_issues(&self) -> bool {
        // More than 10% timeouts or parse failures is concerning
        self.timeout_rate() > 0.1 || self.parser_failure_rate() > 0.1
    }

    /// Get issue description if any.
    pub fn issue_description(&self) -> Option<String> {
        let mut issues = Vec::new();

        if self.timeout_rate() > 0.1 {
            issues.push(format!("{:.0}% timeouts", self.timeout_rate() * 100.0));
        }
        if self.parser_failure_rate() > 0.1 {
            issues.push(format!(
                "{:.0}% parse failures",
                self.parser_failure_rate() * 100.0
            ));
        }
        if self.fallback_rate() > 0.2 {
            issues.push(format!("{:.0}% fallbacks", self.fallback_rate() * 100.0));
        }

        if issues.is_empty() {
            None
        } else {
            Some(issues.join(", "))
        }
    }

    /// Merge with another stats instance.
    pub fn merge(&mut self, other: &ReliabilityStats) {
        self.total_executions += other.total_executions;
        self.successful += other.successful;
        self.specialist_timeouts += other.specialist_timeouts;
        self.parser_failures += other.parser_failures;
        self.fallback_count += other.fallback_count;
        self.total_failures += other.total_failures;
        self.success_after_retry += other.success_after_retry;
        self.total_retries += other.total_retries;
        self.duration_sum_ms += other.duration_sum_ms;

        // Update min
        if let Some(other_min) = other.min_duration_ms {
            match self.min_duration_ms {
                Some(min) if other_min < min => self.min_duration_ms = Some(other_min),
                None => self.min_duration_ms = Some(other_min),
                _ => {}
            }
        }

        // Update max
        if let Some(other_max) = other.max_duration_ms {
            match self.max_duration_ms {
                Some(max) if other_max > max => self.max_duration_ms = Some(other_max),
                None => self.max_duration_ms = Some(other_max),
                _ => {}
            }
        }
    }

    /// Reset stats.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_stats() {
        let mut stats = ReliabilityStats::new();

        // Record some successes
        for _ in 0..8 {
            stats.record(&ExecutionRecord::new(
                ReliabilityOutcome::SpecialistSuccess,
                500,
            ));
        }

        // Record some failures
        stats.record(&ExecutionRecord::new(
            ReliabilityOutcome::SpecialistTimeout,
            1500,
        ));
        stats.record(&ExecutionRecord::new(
            ReliabilityOutcome::ParserFailure,
            1000,
        ));

        assert_eq!(stats.total_executions, 10);
        assert_eq!(stats.successful, 8);
        assert_eq!(stats.specialist_timeouts, 1);
        assert_eq!(stats.parser_failures, 1);
        assert!((stats.success_rate() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_stats_summary() {
        let mut stats = ReliabilityStats::new();
        stats.record(&ExecutionRecord::new(
            ReliabilityOutcome::SpecialistSuccess,
            500,
        ));

        let summary = stats.summary();
        assert!(summary.contains("success"));
        assert!(summary.contains("timeout"));
    }

    #[test]
    fn test_has_issues() {
        let mut stats = ReliabilityStats::new();

        // Good stats
        for _ in 0..10 {
            stats.record(&ExecutionRecord::new(
                ReliabilityOutcome::SpecialistSuccess,
                500,
            ));
        }
        assert!(!stats.has_issues());

        // Add many timeouts
        for _ in 0..5 {
            stats.record(&ExecutionRecord::new(
                ReliabilityOutcome::SpecialistTimeout,
                1500,
            ));
        }
        assert!(stats.has_issues());
    }

    #[test]
    fn test_merge_stats() {
        let mut stats1 = ReliabilityStats::new();
        stats1.record(&ExecutionRecord::new(
            ReliabilityOutcome::SpecialistSuccess,
            500,
        ));

        let mut stats2 = ReliabilityStats::new();
        stats2.record(&ExecutionRecord::new(
            ReliabilityOutcome::SpecialistTimeout,
            1500,
        ));

        stats1.merge(&stats2);
        assert_eq!(stats1.total_executions, 2);
        assert_eq!(stats1.successful, 1);
        assert_eq!(stats1.specialist_timeouts, 1);
    }
}
