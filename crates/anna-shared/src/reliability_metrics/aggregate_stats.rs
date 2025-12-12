//! Aggregated reliability statistics (v0.0.444).
//!
//! Real reliability metrics that correlate with truth.
//! No fake "staff performance" - only actual outcomes.

use super::canonical_outcome::CanonicalOutcome;
use super::request_metrics::RequestMetrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregated reliability statistics.
///
/// These stats must reflect reality:
/// - verified_rate = answered_verified / total_requests
/// - useful_rate = (answered_verified + answered_partial) / total_requests
/// - failure_rate = (timeout + parse + probes + internal) / total_requests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityStats {
    // === Request Counts ===
    /// Total requests processed.
    pub total_requests: u64,
    /// Answered and verified (evidence-backed).
    pub answered_verified: u64,
    /// Answered partially (some gaps).
    pub answered_partial: u64,
    /// Clarification needed (pending user input).
    pub clarification_needed: u64,
    /// Failed due to LLM timeout.
    pub failed_timeout: u64,
    /// Failed due to parse error.
    pub failed_parse: u64,
    /// Failed due to probe failure.
    pub failed_probes: u64,
    /// Aborted by user.
    pub aborted_by_user: u64,
    /// Internal errors.
    pub error_internal: u64,

    // === Latency Stats ===
    /// Sum of all total_ms (for average calculation).
    pub sum_total_ms: u64,
    /// Sum of all probe_ms.
    pub sum_probe_ms: u64,
    /// Sum of all llm_ms.
    pub sum_llm_ms: u64,
    /// Latency values for percentile calculation.
    #[serde(default)]
    pub latencies: Vec<u64>,

    // === Coverage Stats ===
    /// Topic counts.
    pub by_topic: HashMap<String, TopicStats>,
}

/// Stats for a single topic/domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicStats {
    pub total: u64,
    pub verified: u64,
    pub partial: u64,
    pub failed: u64,
}

impl ReliabilityStats {
    /// Create new stats.
    pub fn new() -> Self {
        Self::default()
    }

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

    // === Rate Calculations ===

    /// Verified rate = answered_verified / total_requests.
    /// Only AnsweredVerified counts as "resolved".
    pub fn verified_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.answered_verified as f64 / self.total_requests as f64
        }
    }

    /// Useful rate = (verified + partial) / total_requests.
    pub fn useful_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.answered_verified + self.answered_partial) as f64 / self.total_requests as f64
        }
    }

    /// Failure rate = (timeout + parse + probes + internal) / total_requests.
    pub fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            let failures =
                self.failed_timeout + self.failed_parse + self.failed_probes + self.error_internal;
            failures as f64 / self.total_requests as f64
        }
    }

    /// Total failures.
    pub fn total_failures(&self) -> u64 {
        self.failed_timeout + self.failed_parse + self.failed_probes + self.error_internal
    }

    // === Latency Calculations ===

    /// Average total latency in ms.
    pub fn avg_total_ms(&self) -> u64 {
        if self.total_requests == 0 {
            0
        } else {
            self.sum_total_ms / self.total_requests
        }
    }

    /// Average probe latency in ms.
    pub fn avg_probe_ms(&self) -> u64 {
        if self.total_requests == 0 {
            0
        } else {
            self.sum_probe_ms / self.total_requests
        }
    }

    /// Average LLM latency in ms.
    pub fn avg_llm_ms(&self) -> u64 {
        if self.total_requests == 0 {
            0
        } else {
            self.sum_llm_ms / self.total_requests
        }
    }

    /// Get p50 latency (median).
    pub fn p50_total_ms(&self) -> u64 {
        percentile(&self.latencies, 50)
    }

    /// Get p90 latency.
    pub fn p90_total_ms(&self) -> u64 {
        percentile(&self.latencies, 90)
    }

    // === Topic Analysis ===

    /// Get topics sorted by count.
    pub fn top_topics(&self, n: usize) -> Vec<(&String, &TopicStats)> {
        let mut topics: Vec<_> = self.by_topic.iter().collect();
        topics.sort_by(|a, b| b.1.total.cmp(&a.1.total));
        topics.truncate(n);
        topics
    }

    /// Get topics with low verified rate.
    pub fn low_verified_topics(&self, threshold: f64) -> Vec<(&String, f64)> {
        self.by_topic
            .iter()
            .filter_map(|(topic, stats)| {
                if stats.total == 0 {
                    return None;
                }
                let rate = stats.verified as f64 / stats.total as f64;
                if rate < threshold {
                    Some((topic, rate))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Format for display.
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str("[requests]\n");
        out.push_str(&format!("  total_requests         {}\n", self.total_requests));
        out.push_str(&format!("  answered_verified      {}\n", self.answered_verified));
        out.push_str(&format!("  answered_partial       {}\n", self.answered_partial));
        out.push_str(&format!("  clarification_needed   {}\n", self.clarification_needed));
        out.push_str(&format!("  failed_timeout         {}\n", self.failed_timeout));
        out.push_str(&format!("  failed_parse           {}\n", self.failed_parse));
        out.push_str(&format!("  failed_probes          {}\n", self.failed_probes));
        out.push_str(&format!("  aborted_by_user        {}\n", self.aborted_by_user));
        out.push_str(&format!("  error_internal         {}\n", self.error_internal));
        out.push('\n');

        out.push_str("[latency]\n");
        out.push_str(&format!("  avg_total_ms           {}\n", self.avg_total_ms()));
        out.push_str(&format!("  avg_probe_ms           {}\n", self.avg_probe_ms()));
        out.push_str(&format!("  avg_llm_ms             {}\n", self.avg_llm_ms()));
        out.push_str(&format!("  p50_total_ms           {}\n", self.p50_total_ms()));
        out.push_str(&format!("  p90_total_ms           {}\n", self.p90_total_ms()));
        out.push('\n');

        out.push_str("[reliability]\n");
        out.push_str(&format!("  verified_rate          {:.1}%\n", self.verified_rate() * 100.0));
        out.push_str(&format!("  useful_rate            {:.1}%\n", self.useful_rate() * 100.0));
        out.push_str(&format!("  failure_rate           {:.1}%\n", self.failure_rate() * 100.0));
        out.push('\n');

        if !self.by_topic.is_empty() {
            out.push_str("[coverage]\n");
            out.push_str("  top_topics_by_count:\n");
            for (topic, stats) in self.top_topics(5) {
                let rate = if stats.total > 0 {
                    stats.verified as f64 / stats.total as f64 * 100.0
                } else {
                    0.0
                };
                out.push_str(&format!(
                    "    {} ({} total, {:.0}% verified)\n",
                    topic, stats.total, rate
                ));
            }

            let low = self.low_verified_topics(0.5);
            if !low.is_empty() {
                out.push_str("  topics_with_low_verified_rate:\n");
                for (topic, rate) in low {
                    out.push_str(&format!("    {} ({:.0}%)\n", topic, rate * 100.0));
                }
            }
        }

        out
    }

    /// Compact summary for status line.
    pub fn summary_line(&self) -> String {
        format!(
            "{}req | {:.0}% verified | {:.0}% useful | {:.0}% failed | {}ms avg",
            self.total_requests,
            self.verified_rate() * 100.0,
            self.useful_rate() * 100.0,
            self.failure_rate() * 100.0,
            self.avg_total_ms(),
        )
    }
}

/// Calculate percentile from values.
fn percentile(values: &[u64], p: u32) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = (p as f64 / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted.get(idx).copied().unwrap_or(0)
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
