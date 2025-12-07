//! Supporting types for daemon state (kept lean to satisfy file size limits).

use std::time::{Duration, Instant};

use anna_shared::rpc::ProbeResult;

/// Probe cache TTL (30 seconds)
pub const PROBE_CACHE_TTL: Duration = Duration::from_secs(30);

/// Max number of latency records to keep per stage
pub const MAX_LATENCY_RECORDS: usize = 20;

/// Cached probe result with timestamp
#[derive(Debug, Clone)]
pub struct CachedProbe {
    pub result: ProbeResult,
    pub cached_at: Instant,
}

impl CachedProbe {
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < PROBE_CACHE_TTL
    }
}

/// Latency stats for a pipeline stage
#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    /// Last N latency samples in milliseconds
    pub samples: Vec<u64>,
}

impl LatencyStats {
    /// Add a latency sample
    pub fn add(&mut self, ms: u64) {
        self.samples.push(ms);
        if self.samples.len() > MAX_LATENCY_RECORDS {
            self.samples.remove(0);
        }
    }

    /// Average latency in ms
    pub fn avg_ms(&self) -> Option<u64> {
        if self.samples.is_empty() {
            None
        } else {
            Some(self.samples.iter().sum::<u64>() / self.samples.len() as u64)
        }
    }

    /// P50 (median) latency in ms
    pub fn p50_ms(&self) -> Option<u64> {
        self.percentile_ms(0.50)
    }

    /// P90 latency in ms
    pub fn p90_ms(&self) -> Option<u64> {
        self.percentile_ms(0.90)
    }

    /// P95 latency in ms
    pub fn p95_ms(&self) -> Option<u64> {
        self.percentile_ms(0.95)
    }

    /// Calculate percentile latency
    fn percentile_ms(&self, p: f64) -> Option<u64> {
        if self.samples.is_empty() {
            None
        } else {
            let mut sorted = self.samples.clone();
            sorted.sort_unstable();
            let idx = (sorted.len() as f64 * p).ceil() as usize - 1;
            Some(sorted[idx.min(sorted.len() - 1)])
        }
    }

    /// Min latency in ms
    pub fn min_ms(&self) -> Option<u64> {
        self.samples.iter().min().copied()
    }

    /// Max latency in ms
    pub fn max_ms(&self) -> Option<u64> {
        self.samples.iter().max().copied()
    }

    /// Number of samples collected
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

/// Per-stage latency tracking
#[derive(Debug, Clone, Default)]
pub struct PipelineLatency {
    pub translator: LatencyStats,
    pub probes: LatencyStats,
    pub specialist: LatencyStats,
    pub total: LatencyStats,
}
