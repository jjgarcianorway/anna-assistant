//! Request metrics storage (v0.0.444).
//!
//! Maintains a rolling window of recent request metrics with indexing.

use super::canonical_outcome::CanonicalOutcome;
use super::request_metrics_types::RequestMetrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage for request metrics history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequestMetricsStore {
    /// Recent metrics (rolling window).
    pub recent: Vec<RequestMetrics>,

    /// Maximum entries to keep.
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,

    /// Metrics indexed by request_id for quick lookup.
    #[serde(skip)]
    pub by_id: HashMap<String, usize>,
}

fn default_max_entries() -> usize {
    1000
}

impl RequestMetricsStore {
    /// Create a new store.
    pub fn new() -> Self {
        Self {
            recent: Vec::new(),
            max_entries: 1000,
            by_id: HashMap::new(),
        }
    }

    /// Add a request to the store.
    pub fn add(&mut self, metrics: RequestMetrics) {
        let id = metrics.request_id.clone();
        self.recent.push(metrics);
        let idx = self.recent.len() - 1;
        self.by_id.insert(id, idx);

        // Trim if needed
        if self.recent.len() > self.max_entries {
            let removed = self.recent.remove(0);
            self.by_id.remove(&removed.request_id);
            // Rebuild index (indices shifted)
            self.rebuild_index();
        }
    }

    /// Get metrics by request ID.
    pub fn get(&self, request_id: &str) -> Option<&RequestMetrics> {
        self.by_id.get(request_id).and_then(|&i| self.recent.get(i))
    }

    /// Rebuild the by_id index.
    fn rebuild_index(&mut self) {
        self.by_id.clear();
        for (i, m) in self.recent.iter().enumerate() {
            self.by_id.insert(m.request_id.clone(), i);
        }
    }

    /// Get recent N requests.
    pub fn recent(&self, n: usize) -> &[RequestMetrics] {
        let start = self.recent.len().saturating_sub(n);
        &self.recent[start..]
    }

    /// Get requests with a specific outcome.
    pub fn with_outcome(&self, outcome: CanonicalOutcome) -> Vec<&RequestMetrics> {
        self.recent
            .iter()
            .filter(|m| m.outcome == outcome)
            .collect()
    }

    /// Get requests for a topic.
    pub fn for_topic(&self, topic: &str) -> Vec<&RequestMetrics> {
        self.recent
            .iter()
            .filter(|m| m.routed_topic == topic)
            .collect()
    }
}
