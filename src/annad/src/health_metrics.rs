// Health Metrics Module - v0.12.7
// Tracks daemon health: RPC latency, memory usage, queue depth

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum number of latency samples to keep in memory
const MAX_LATENCY_SAMPLES: usize = 100;

/// Health status thresholds
pub struct HealthThresholds {
    pub latency_warn_ms: u64,      // Warn if p95 > this (default: 200ms)
    pub latency_critical_ms: u64,  // Critical if p99 > this (default: 500ms)
    pub memory_warn_mb: u64,       // Warn if RSS > this (default: 60MB)
    pub memory_critical_mb: u64,   // Critical if RSS > this (default: 70MB)
    pub queue_warn_depth: usize,   // Warn if queue > this (default: 50)
    pub queue_critical_depth: usize, // Critical if queue > this (default: 100)
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            latency_warn_ms: 200,
            latency_critical_ms: 500,
            memory_warn_mb: 60,
            memory_critical_mb: 70,
            queue_warn_depth: 50,
            queue_critical_depth: 100,
        }
    }
}

/// RPC latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcLatencyMetrics {
    pub avg_ms: f64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub sample_count: usize,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub current_mb: f64,
    pub peak_mb: f64,
    pub limit_mb: u64,
    pub vmsize_mb: f64,
    pub threads: usize,
}

/// Queue health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub depth: usize,
    pub rate_per_sec: f64,
    pub oldest_event_sec: u64,
    pub total_processed: u64,
}

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,   // All metrics within normal range
    Warning,   // Some metrics elevated but not critical
    Critical,  // Metrics at critical levels
    Unknown,   // Unable to determine status
}

/// Complete health snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub status: HealthStatus,
    pub uptime_sec: u64,
    pub rpc_latency: Option<RpcLatencyMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub queue: Option<QueueMetrics>,
    pub capabilities_active: usize,
    pub capabilities_degraded: usize,
    pub timestamp: u64, // Unix timestamp
}

/// Latency tracker - thread-safe RPC latency recording
pub struct LatencyTracker {
    samples: Arc<Mutex<VecDeque<Duration>>>,
    start_time: Instant,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LATENCY_SAMPLES))),
            start_time: Instant::now(),
        }
    }

    /// Record a new latency sample
    pub fn record(&self, latency: Duration) {
        let mut samples = self.samples.lock().unwrap();
        if samples.len() >= MAX_LATENCY_SAMPLES {
            samples.pop_front();
        }
        samples.push_back(latency);
    }

    /// Get current latency metrics
    pub fn metrics(&self) -> Option<RpcLatencyMetrics> {
        let samples = self.samples.lock().unwrap();
        if samples.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = samples.iter().map(|d| d.as_millis() as u64).collect();
        sorted.sort_unstable();

        let sum: u64 = sorted.iter().sum();
        let count = sorted.len();
        let avg_ms = sum as f64 / count as f64;

        let p50 = percentile(&sorted, 50);
        let p95 = percentile(&sorted, 95);
        let p99 = percentile(&sorted, 99);

        Some(RpcLatencyMetrics {
            avg_ms,
            p50_ms: p50,
            p95_ms: p95,
            p99_ms: p99,
            min_ms: sorted[0],
            max_ms: sorted[sorted.len() - 1],
            sample_count: count,
        })
    }

    /// Get uptime in seconds
    pub fn uptime_sec(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

/// Calculate percentile from sorted array
fn percentile(sorted: &[u64], p: u8) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * p as f64 / 100.0).round() as usize;
    sorted[idx]
}

/// Memory metrics reader
pub struct MemoryMonitor {
    peak_rss_kb: Arc<Mutex<u64>>,
    limit_mb: u64,
}

impl MemoryMonitor {
    pub fn new(limit_mb: u64) -> Self {
        Self {
            peak_rss_kb: Arc::new(Mutex::new(0)),
            limit_mb,
        }
    }

    /// Read current memory metrics from /proc/self/status
    pub fn metrics(&self) -> Result<MemoryMetrics> {
        let status = std::fs::read_to_string("/proc/self/status")?;

        let mut vmrss_kb = 0u64;
        let mut vmsize_kb = 0u64;
        let mut threads = 0usize;

        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                vmrss_kb = parse_kb_line(line);
            } else if line.starts_with("VmSize:") {
                vmsize_kb = parse_kb_line(line);
            } else if line.starts_with("Threads:") {
                threads = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }

        // Update peak
        let mut peak = self.peak_rss_kb.lock().unwrap();
        if vmrss_kb > *peak {
            *peak = vmrss_kb;
        }

        Ok(MemoryMetrics {
            current_mb: vmrss_kb as f64 / 1024.0,
            peak_mb: *peak as f64 / 1024.0,
            limit_mb: self.limit_mb,
            vmsize_mb: vmsize_kb as f64 / 1024.0,
            threads,
        })
    }

    /// Check if memory usage is concerning
    pub fn check_status(&self, metrics: &MemoryMetrics, thresholds: &HealthThresholds) -> HealthStatus {
        let current_mb = metrics.current_mb as u64;
        if current_mb >= thresholds.memory_critical_mb {
            HealthStatus::Critical
        } else if current_mb >= thresholds.memory_warn_mb {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Parse line like "VmRSS:    12345 kB" -> 12345
fn parse_kb_line(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Health evaluator - determines overall health status
pub struct HealthEvaluator {
    thresholds: HealthThresholds,
}

impl HealthEvaluator {
    pub fn new(thresholds: HealthThresholds) -> Self {
        Self { thresholds }
    }

    pub fn with_defaults() -> Self {
        Self::new(HealthThresholds::default())
    }

    /// Evaluate overall health from individual metrics
    pub fn evaluate(&self, snapshot: &HealthSnapshot) -> HealthStatus {
        let mut worst = HealthStatus::Healthy;

        // Check RPC latency
        if let Some(ref latency) = snapshot.rpc_latency {
            let status = self.check_latency(latency);
            worst = worst.max(status);
        }

        // Check memory
        if let Some(ref memory) = snapshot.memory {
            let status = self.check_memory(memory);
            worst = worst.max(status);
        }

        // Check queue
        if let Some(ref queue) = snapshot.queue {
            let status = self.check_queue(queue);
            worst = worst.max(status);
        }

        worst
    }

    fn check_latency(&self, latency: &RpcLatencyMetrics) -> HealthStatus {
        if latency.p99_ms >= self.thresholds.latency_critical_ms {
            HealthStatus::Critical
        } else if latency.p95_ms >= self.thresholds.latency_warn_ms {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    fn check_memory(&self, memory: &MemoryMetrics) -> HealthStatus {
        let current_mb = memory.current_mb as u64;
        if current_mb >= self.thresholds.memory_critical_mb {
            HealthStatus::Critical
        } else if current_mb >= self.thresholds.memory_warn_mb {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    fn check_queue(&self, queue: &QueueMetrics) -> HealthStatus {
        if queue.depth >= self.thresholds.queue_critical_depth {
            HealthStatus::Critical
        } else if queue.depth >= self.thresholds.queue_warn_depth {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

impl PartialOrd for HealthStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HealthStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Warning => 1,
            HealthStatus::Critical => 2,
            HealthStatus::Unknown => 3,
        };
        let other_val = match other {
            HealthStatus::Healthy => 0,
            HealthStatus::Warning => 1,
            HealthStatus::Critical => 2,
            HealthStatus::Unknown => 3,
        };
        self_val.cmp(&other_val)
    }
}

impl HealthStatus {
    pub fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculation() {
        let samples = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        assert_eq!(percentile(&samples, 50), 50);
        assert_eq!(percentile(&samples, 95), 100);
        assert_eq!(percentile(&samples, 99), 100);
    }

    #[test]
    fn test_latency_tracker() {
        let tracker = LatencyTracker::new();

        // Record some samples
        tracker.record(Duration::from_millis(10));
        tracker.record(Duration::from_millis(20));
        tracker.record(Duration::from_millis(30));

        let metrics = tracker.metrics().unwrap();
        assert_eq!(metrics.sample_count, 3);
        assert_eq!(metrics.min_ms, 10);
        assert_eq!(metrics.max_ms, 30);
        assert!((metrics.avg_ms - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_health_evaluator() {
        let evaluator = HealthEvaluator::with_defaults();

        let snapshot = HealthSnapshot {
            status: HealthStatus::Healthy,
            uptime_sec: 100,
            rpc_latency: Some(RpcLatencyMetrics {
                avg_ms: 15.0,
                p50_ms: 12,
                p95_ms: 45,
                p99_ms: 78,
                min_ms: 5,
                max_ms: 120,
                sample_count: 50,
            }),
            memory: Some(MemoryMetrics {
                current_mb: 25.0,
                peak_mb: 30.0,
                limit_mb: 80,
                vmsize_mb: 145.0,
                threads: 3,
            }),
            queue: Some(QueueMetrics {
                depth: 5,
                rate_per_sec: 12.3,
                oldest_event_sec: 2,
                total_processed: 1234,
            }),
            capabilities_active: 4,
            capabilities_degraded: 0,
            timestamp: 0,
        };

        let status = evaluator.evaluate(&snapshot);
        assert_eq!(status, HealthStatus::Healthy);
    }
}
