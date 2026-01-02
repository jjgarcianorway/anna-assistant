//! Calculation methods for reliability statistics (v0.0.444).

use super::aggregate_stats_types::{ReliabilityStats, TopicStats};

impl ReliabilityStats {
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
}

/// Calculate percentile from values.
pub(super) fn percentile(values: &[u64], p: u32) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = (p as f64 / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted.get(idx).copied().unwrap_or(0)
}
