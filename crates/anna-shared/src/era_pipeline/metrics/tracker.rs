//! Honest metrics tracking and reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::ResolutionStatus;

/// Honest metrics tracker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonestMetrics {
    /// Counts by status.
    status_counts: HashMap<ResolutionStatus, u64>,
    /// Total tickets.
    total: u64,
    /// Average confidence of resolved tickets.
    avg_confidence: f64,
    /// Sum of confidences (for calculating average).
    confidence_sum: f64,
    /// Confidence count.
    confidence_count: u64,
    /// Fast-path usage.
    fast_path_count: u64,
    /// Fast-path successes.
    fast_path_successes: u64,
}

impl HonestMetrics {
    /// Create empty metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ticket resolution.
    pub fn record(&mut self, status: ResolutionStatus, confidence: f64) {
        *self.status_counts.entry(status).or_insert(0) += 1;
        self.total += 1;

        if status == ResolutionStatus::Resolved {
            self.confidence_sum += confidence;
            self.confidence_count += 1;
            self.avg_confidence = self.confidence_sum / self.confidence_count as f64;
        }
    }

    /// Record fast-path attempt.
    pub fn record_fast_path(&mut self, success: bool) {
        self.fast_path_count += 1;
        if success {
            self.fast_path_successes += 1;
        }
    }

    /// Get count for status.
    pub fn count(&self, status: ResolutionStatus) -> u64 {
        self.status_counts.get(&status).copied().unwrap_or(0)
    }

    /// Get resolved count (ONLY fully resolved).
    pub fn resolved(&self) -> u64 {
        self.count(ResolutionStatus::Resolved)
    }

    /// Get failed count.
    pub fn failed(&self) -> u64 {
        self.count(ResolutionStatus::CannotAnswer) + self.count(ResolutionStatus::Failed)
    }

    /// Get total.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Get SUCCESS RATE (honest).
    /// Only counts fully resolved tickets as success.
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.resolved() as f64 / self.total as f64
        }
    }

    /// Get failure rate.
    pub fn failure_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.failed() as f64 / self.total as f64
        }
    }

    /// Get fast-path success rate.
    pub fn fast_path_rate(&self) -> f64 {
        if self.fast_path_count == 0 {
            0.0
        } else {
            self.fast_path_successes as f64 / self.fast_path_count as f64
        }
    }

    /// Get average confidence.
    pub fn average_confidence(&self) -> f64 {
        self.avg_confidence
    }

    /// Get summary.
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total: self.total,
            resolved: self.resolved(),
            partial: self.count(ResolutionStatus::Partial),
            cannot_answer: self.count(ResolutionStatus::CannotAnswer),
            failed: self.count(ResolutionStatus::Failed),
            success_rate: self.success_rate(),
            failure_rate: self.failure_rate(),
            avg_confidence: self.avg_confidence,
            fast_path_rate: self.fast_path_rate(),
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Metrics summary for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total tickets.
    pub total: u64,
    /// Fully resolved.
    pub resolved: u64,
    /// Partially resolved.
    pub partial: u64,
    /// Cannot answer.
    pub cannot_answer: u64,
    /// Failed.
    pub failed: u64,
    /// Success rate (resolved / total).
    pub success_rate: f64,
    /// Failure rate.
    pub failure_rate: f64,
    /// Average confidence of resolved.
    pub avg_confidence: f64,
    /// Fast-path success rate.
    pub fast_path_rate: f64,
}

impl MetricsSummary {
    /// Format for logging.
    pub fn log_message(&self) -> String {
        format!(
            "[metrics] total={} resolved={} partial={} cannot_answer={} failed={} \
             success_rate={:.1}% avg_confidence={:.2}",
            self.total,
            self.resolved,
            self.partial,
            self.cannot_answer,
            self.failed,
            self.success_rate * 100.0,
            self.avg_confidence
        )
    }

    /// Format for display.
    pub fn display(&self) -> String {
        format!(
            "Tickets: {} total, {} resolved ({:.0}%), {} failed",
            self.total,
            self.resolved,
            self.success_rate * 100.0,
            self.failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honest_metrics() {
        let mut metrics = HonestMetrics::new();

        metrics.record(ResolutionStatus::Resolved, 0.9);
        metrics.record(ResolutionStatus::Resolved, 0.85);
        metrics.record(ResolutionStatus::CannotAnswer, 0.0);
        metrics.record(ResolutionStatus::Failed, 0.0);

        assert_eq!(metrics.total(), 4);
        assert_eq!(metrics.resolved(), 2);
        assert_eq!(metrics.failed(), 2);
        assert!((metrics.success_rate() - 0.5).abs() < 0.01);
        assert!((metrics.average_confidence() - 0.875).abs() < 0.01);
    }

    #[test]
    fn test_metrics_summary() {
        let mut metrics = HonestMetrics::new();
        metrics.record(ResolutionStatus::Resolved, 0.9);
        metrics.record(ResolutionStatus::Partial, 0.5);
        metrics.record(ResolutionStatus::CannotAnswer, 0.0);

        let summary = metrics.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.resolved, 1);
        assert_eq!(summary.partial, 1);
        assert_eq!(summary.cannot_answer, 1);
    }

    #[test]
    fn test_fast_path_tracking() {
        let mut metrics = HonestMetrics::new();
        metrics.record_fast_path(true);
        metrics.record_fast_path(true);
        metrics.record_fast_path(false);

        assert_eq!(metrics.fast_path_count, 3);
        assert_eq!(metrics.fast_path_successes, 2);
        assert!((metrics.fast_path_rate() - 0.666).abs() < 0.01);
    }
}
