//! Parallel Probe Types - v0.0.438.
//!
//! Core types for probe execution and results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Maximum concurrent probes.
pub const MAX_CONCURRENT_PROBES: usize = 4;

/// Default cache TTL in seconds.
pub const CACHE_TTL_SECONDS: u64 = 3;

/// Timeout per probe in milliseconds.
pub const PROBE_TIMEOUT_MS: u64 = 500;

/// Status of a probe execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeStatus {
    /// Pending execution.
    Pending,
    /// Currently running.
    Running,
    /// Completed successfully.
    Completed,
    /// Timed out.
    TimedOut,
    /// Failed with error.
    Failed,
    /// Served from cache.
    Cached,
}

impl ProbeStatus {
    /// Whether this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::TimedOut | Self::Failed | Self::Cached
        )
    }

    /// Whether this was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed | Self::Cached)
    }

    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::TimedOut => "timed_out",
            Self::Failed => "failed",
            Self::Cached => "cached",
        }
    }
}

/// Result of a single probe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe ID.
    pub probe_id: String,
    /// Execution status.
    pub status: ProbeStatus,
    /// Result value if successful.
    pub value: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Execution time in milliseconds.
    pub duration_ms: u64,
}

impl ProbeResult {
    /// Create successful result.
    pub fn success(probe_id: &str, value: &str, duration_ms: u64) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: ProbeStatus::Completed,
            value: Some(value.to_string()),
            error: None,
            duration_ms,
        }
    }

    /// Create cached result.
    pub fn cached(probe_id: &str, value: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: ProbeStatus::Cached,
            value: Some(value.to_string()),
            error: None,
            duration_ms: 0,
        }
    }

    /// Create timed out result.
    pub fn timeout(probe_id: &str, duration_ms: u64) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: ProbeStatus::TimedOut,
            value: None,
            error: Some("Probe timed out".to_string()),
            duration_ms,
        }
    }

    /// Create failed result.
    pub fn failed(probe_id: &str, error: &str, duration_ms: u64) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            status: ProbeStatus::Failed,
            value: None,
            error: Some(error.to_string()),
            duration_ms,
        }
    }
}

/// A batch of probes to execute.
#[derive(Debug, Clone)]
pub struct ProbeBatch {
    /// Probe IDs to execute.
    pub probe_ids: Vec<String>,
    /// Timeout per probe.
    pub timeout: Duration,
    /// Max concurrent.
    pub concurrency: usize,
}

impl ProbeBatch {
    /// Create new batch.
    pub fn new(probe_ids: Vec<String>) -> Self {
        Self {
            probe_ids,
            timeout: Duration::from_millis(PROBE_TIMEOUT_MS),
            concurrency: MAX_CONCURRENT_PROBES,
        }
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set concurrency limit.
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Get number of probes.
    pub fn len(&self) -> usize {
        self.probe_ids.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.probe_ids.is_empty()
    }
}

/// Result of batch execution.
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Results by probe ID.
    pub results: HashMap<String, ProbeResult>,
    /// Total execution time.
    pub total_duration_ms: u64,
    /// How many were cached.
    pub cached_count: usize,
    /// How many succeeded.
    pub success_count: usize,
    /// How many failed.
    pub failed_count: usize,
}

impl BatchResult {
    /// Create from results map.
    pub fn from_results(results: HashMap<String, ProbeResult>, duration_ms: u64) -> Self {
        let cached_count = results
            .values()
            .filter(|r| r.status == ProbeStatus::Cached)
            .count();
        let success_count = results.values().filter(|r| r.status.is_success()).count();
        let failed_count = results.values().filter(|r| !r.status.is_success()).count();

        Self {
            results,
            total_duration_ms: duration_ms,
            cached_count,
            success_count,
            failed_count,
        }
    }

    /// Get successful values as HashMap.
    pub fn values(&self) -> HashMap<String, String> {
        self.results
            .iter()
            .filter_map(|(k, v)| v.value.as_ref().map(|val| (k.clone(), val.clone())))
            .collect()
    }

    /// Check if all succeeded.
    pub fn all_success(&self) -> bool {
        self.failed_count == 0
    }
}

/// Prepared batch ready for execution.
#[derive(Debug, Clone)]
pub struct PreparedBatch {
    /// Already cached results.
    pub cached: Vec<ProbeResult>,
    /// Probe IDs that need execution.
    pub to_execute: Vec<String>,
    /// Concurrency limit.
    pub concurrency: usize,
    /// Timeout per probe.
    pub timeout: Duration,
}

impl PreparedBatch {
    /// Check if any probes need execution.
    pub fn needs_execution(&self) -> bool {
        !self.to_execute.is_empty()
    }

    /// Total probes (cached + to execute).
    pub fn total(&self) -> usize {
        self.cached.len() + self.to_execute.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_status() {
        assert!(ProbeStatus::Completed.is_terminal());
        assert!(ProbeStatus::Completed.is_success());
        assert!(ProbeStatus::Cached.is_success());
        assert!(!ProbeStatus::Running.is_terminal());
        assert!(!ProbeStatus::Failed.is_success());
    }

    #[test]
    fn test_probe_result() {
        let success = ProbeResult::success("sys.mem.free", "4.2 GB", 100);
        assert!(success.status.is_success());
        assert_eq!(success.value, Some("4.2 GB".to_string()));

        let cached = ProbeResult::cached("sys.cpu", "25%");
        assert_eq!(cached.status, ProbeStatus::Cached);
        assert_eq!(cached.duration_ms, 0);
    }

    #[test]
    fn test_probe_batch() {
        let batch = ProbeBatch::new(vec!["a".into(), "b".into(), "c".into()])
            .with_concurrency(2)
            .with_timeout(Duration::from_millis(200));

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.concurrency, 2);
    }

    #[test]
    fn test_batch_result() {
        let mut results = HashMap::new();
        results.insert("a".into(), ProbeResult::success("a", "1", 50));
        results.insert("b".into(), ProbeResult::cached("b", "2"));
        results.insert("c".into(), ProbeResult::timeout("c", 500));

        let batch_result = BatchResult::from_results(results, 500);
        assert_eq!(batch_result.cached_count, 1);
        assert_eq!(batch_result.success_count, 2);
        assert_eq!(batch_result.failed_count, 1);
    }
}
