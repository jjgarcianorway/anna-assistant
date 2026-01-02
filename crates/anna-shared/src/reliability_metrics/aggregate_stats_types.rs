//! Type definitions for aggregated reliability statistics (v0.0.444).

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

    /// Total failures.
    pub fn total_failures(&self) -> u64 {
        self.failed_timeout + self.failed_parse + self.failed_probes + self.error_internal
    }
}
