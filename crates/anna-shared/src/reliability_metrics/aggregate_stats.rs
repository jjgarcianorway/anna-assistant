//! Aggregated reliability statistics (v0.0.444).
//!
//! Real reliability metrics that correlate with truth.
//! No fake "staff performance" - only actual outcomes.

// Re-export all public types and functions from sibling modules
pub use super::aggregate_stats_types::{ReliabilityStats, TopicStats};

use super::canonical_outcome::CanonicalOutcome;
use super::request_metrics::RequestMetrics;

impl ReliabilityStats {
    /// Record a request from metrics.
    pub fn record(&mut self, metrics: &RequestMetrics) {
        self.total_requests += 1;

        // Record by outcome
        match metrics.outcome {
            CanonicalOutcome::AnsweredVerified => self.answered_verified += 1,
            CanonicalOutcome::AnsweredPartial => self.answered_partial += 1,
            CanonicalOutcome::ClarificationNeeded => self.clarification_needed += 1,
            CanonicalOutcome::FailedTimeout => self.failed_timeout += 1,
            CanonicalOutcome::FailedParse => self.failed_parse += 1,
            CanonicalOutcome::FailedProbes => self.failed_probes += 1,
            CanonicalOutcome::AbortedByUser => self.aborted_by_user += 1,
            CanonicalOutcome::ErrorInternal => self.error_internal += 1,
        }

        // Record latency
        self.sum_total_ms += metrics.total_ms;
        self.sum_probe_ms += metrics.probe_ms;
        self.sum_llm_ms += metrics.llm_ms;
        self.latencies.push(metrics.total_ms);

        // Keep latencies bounded
        if self.latencies.len() > 10000 {
            self.latencies.drain(0..5000);
        }

        // Record by topic
        let topic = if metrics.routed_topic.is_empty() {
            "unknown"
        } else {
            &metrics.routed_topic
        };
        let topic_stats = self.by_topic.entry(topic.to_string()).or_default();
        topic_stats.total += 1;
        if metrics.outcome.is_resolved() {
            topic_stats.verified += 1;
        } else if metrics.outcome.is_partial() {
            topic_stats.partial += 1;
        } else if metrics.outcome.is_failure() {
            topic_stats.failed += 1;
        }
    }

    /// Record outcome directly (without full metrics).
    pub fn record_outcome(&mut self, outcome: CanonicalOutcome, topic: Option<&str>) {
        self.total_requests += 1;
        match outcome {
            CanonicalOutcome::AnsweredVerified => self.answered_verified += 1,
            CanonicalOutcome::AnsweredPartial => self.answered_partial += 1,
            CanonicalOutcome::ClarificationNeeded => self.clarification_needed += 1,
            CanonicalOutcome::FailedTimeout => self.failed_timeout += 1,
            CanonicalOutcome::FailedParse => self.failed_parse += 1,
            CanonicalOutcome::FailedProbes => self.failed_probes += 1,
            CanonicalOutcome::AbortedByUser => self.aborted_by_user += 1,
            CanonicalOutcome::ErrorInternal => self.error_internal += 1,
        }

        if let Some(t) = topic {
            let ts = self.by_topic.entry(t.to_string()).or_default();
            ts.total += 1;
            if outcome.is_resolved() {
                ts.verified += 1;
            } else if outcome.is_partial() {
                ts.partial += 1;
            } else if outcome.is_failure() {
                ts.failed += 1;
            }
        }
    }
}

/// Compute stats from a slice of metrics.
pub fn compute_stats(metrics: &[RequestMetrics]) -> ReliabilityStats {
    let mut stats = ReliabilityStats::new();
    for m in metrics {
        stats.record(m);
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliability_metrics::request_metrics::RequestMetricsBuilder;

    #[test]
    fn test_stats_recording() {
        let mut stats = ReliabilityStats::new();

        // 6 verified
        for i in 0..6 {
            let m = RequestMetricsBuilder::new(format!("V{}", i), "query")
                .topic("storage")
                .timing(100, 500, 50)
                .finish(CanonicalOutcome::AnsweredVerified, None);
            stats.record(&m);
        }

        // 2 partial
        for i in 0..2 {
            let m = RequestMetricsBuilder::new(format!("P{}", i), "query")
                .topic("network")
                .timing(100, 500, 50)
                .finish(CanonicalOutcome::AnsweredPartial, None);
            stats.record(&m);
        }

        // 2 failed
        let m = RequestMetricsBuilder::new("F1", "query")
            .topic("storage")
            .finish(CanonicalOutcome::FailedTimeout, None);
        stats.record(&m);

        let m = RequestMetricsBuilder::new("F2", "query")
            .topic("network")
            .finish(CanonicalOutcome::FailedParse, None);
        stats.record(&m);

        assert_eq!(stats.total_requests, 10);
        assert_eq!(stats.answered_verified, 6);
        assert_eq!(stats.answered_partial, 2);
        assert_eq!(stats.failed_timeout, 1);
        assert_eq!(stats.failed_parse, 1);

        // 6/10 = 0.6 verified rate
        assert!((stats.verified_rate() - 0.6).abs() < 0.01);
        // 8/10 = 0.8 useful rate
        assert!((stats.useful_rate() - 0.8).abs() < 0.01);
        // 2/10 = 0.2 failure rate
        assert!((stats.failure_rate() - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_stats_display() {
        let mut stats = ReliabilityStats::new();
        stats.record_outcome(CanonicalOutcome::AnsweredVerified, Some("storage"));
        stats.record_outcome(CanonicalOutcome::FailedTimeout, Some("network"));

        let display = stats.display();
        assert!(display.contains("total_requests"));
        assert!(display.contains("verified_rate"));
    }

    #[test]
    fn test_percentile() {
        use crate::reliability_metrics::aggregate_stats_calc::percentile;
        let values = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];
        // p50 with 10 values: index 4.5 rounds to 5 → value 600
        assert_eq!(percentile(&values, 50), 600);
        // p90 with 10 values: index 8.1 rounds to 8 → value 900
        assert_eq!(percentile(&values, 90), 900);
    }

    #[test]
    fn test_topic_stats() {
        let mut stats = ReliabilityStats::new();
        for _ in 0..5 {
            stats.record_outcome(CanonicalOutcome::AnsweredVerified, Some("storage"));
        }
        for _ in 0..3 {
            stats.record_outcome(CanonicalOutcome::FailedParse, Some("storage"));
        }

        let topic_stats = stats.by_topic.get("storage").unwrap();
        assert_eq!(topic_stats.total, 8);
        assert_eq!(topic_stats.verified, 5);
        assert_eq!(topic_stats.failed, 3);

        // 5/8 = 0.625 verified rate for storage
        let low = stats.low_verified_topics(0.7);
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].0, "storage");
    }
}
