//! Telemetry Snapshot Aggregator for Anna Assistant
//!
//! Provides real-time snapshot of daemon metrics including queue stats,
//! event counts, and system activity for watch mode.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Live telemetry snapshot for watch mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    /// Timestamp of snapshot (RFC3339)
    pub timestamp: String,

    /// Unix timestamp (seconds since epoch)
    pub timestamp_unix: i64,

    /// Queue metrics
    pub queue: QueueMetrics,

    /// Event statistics
    pub events: EventMetrics,

    /// System resource usage
    pub resources: ResourceMetrics,

    /// Module activity status
    pub modules: ModuleActivity,
}

/// Queue-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    /// Current queue depth (pending events)
    pub depth: usize,

    /// Events per second (calculated over last 60s)
    pub rate_per_sec: f64,

    /// Oldest pending event age (seconds)
    pub oldest_event_sec: u64,

    /// Total events processed since startup
    pub total_processed: u64,

    /// Total events dropped
    pub total_dropped: u64,
}

/// Event-related statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    /// Events in last 1 minute
    pub last_1min: u64,

    /// Events in last 5 minutes
    pub last_5min: u64,

    /// Events in last 15 minutes
    pub last_15min: u64,

    /// Events in last hour
    pub last_1hour: u64,

    /// Events by domain (last 5 minutes)
    pub by_domain: std::collections::HashMap<String, u64>,
}

/// System resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Memory usage (bytes)
    pub memory_bytes: u64,

    /// Memory usage (human-readable)
    pub memory_human: String,

    /// CPU utilization percentage (daemon process)
    pub cpu_percent: f64,

    /// Thread count
    pub thread_count: usize,

    /// File descriptor count
    pub fd_count: usize,
}

/// Module activity indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleActivity {
    /// Telemetry collector active
    pub telemetry_active: bool,

    /// Event engine active
    pub event_engine_active: bool,

    /// Policy engine active
    pub policy_engine_active: bool,

    /// Storage manager active
    pub storage_active: bool,

    /// RPC server active
    pub rpc_active: bool,

    /// Last activity timestamps (module name -> unix timestamp)
    pub last_activity: std::collections::HashMap<String, i64>,
}

impl TelemetrySnapshot {
    /// Create a new snapshot with current timestamp
    pub fn new() -> Self {
        let now = SystemTime::now();
        let unix_time = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let timestamp = chrono::Utc::now().to_rfc3339();

        Self {
            timestamp,
            timestamp_unix: unix_time,
            queue: QueueMetrics::default(),
            events: EventMetrics::default(),
            resources: ResourceMetrics::default(),
            modules: ModuleActivity::default(),
        }
    }

    /// Calculate delta between two snapshots
    pub fn delta(&self, other: &TelemetrySnapshot) -> SnapshotDelta {
        let time_delta = self.timestamp_unix - other.timestamp_unix;

        SnapshotDelta {
            time_delta_sec: time_delta,
            queue_depth_delta: self.queue.depth as i64 - other.queue.depth as i64,
            event_rate_delta: self.queue.rate_per_sec - other.queue.rate_per_sec,
            memory_delta_bytes: self.resources.memory_bytes as i64 - other.resources.memory_bytes as i64,
            cpu_delta_percent: self.resources.cpu_percent - other.resources.cpu_percent,
        }
    }
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self {
            depth: 0,
            rate_per_sec: 0.0,
            oldest_event_sec: 0,
            total_processed: 0,
            total_dropped: 0,
        }
    }
}

impl Default for EventMetrics {
    fn default() -> Self {
        Self {
            last_1min: 0,
            last_5min: 0,
            last_15min: 0,
            last_1hour: 0,
            by_domain: std::collections::HashMap::new(),
        }
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            memory_bytes: 0,
            memory_human: "0 B".to_string(),
            cpu_percent: 0.0,
            thread_count: 0,
            fd_count: 0,
        }
    }
}

impl Default for ModuleActivity {
    fn default() -> Self {
        Self {
            telemetry_active: false,
            event_engine_active: false,
            policy_engine_active: false,
            storage_active: false,
            rpc_active: false,
            last_activity: std::collections::HashMap::new(),
        }
    }
}

/// Delta between two snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDelta {
    /// Time difference (seconds)
    pub time_delta_sec: i64,

    /// Queue depth change
    pub queue_depth_delta: i64,

    /// Event rate change (events/sec)
    pub event_rate_delta: f64,

    /// Memory change (bytes)
    pub memory_delta_bytes: i64,

    /// CPU change (percentage points)
    pub cpu_delta_percent: f64,
}

/// Snapshot aggregator for collecting live metrics
pub struct SnapshotAggregator {
    /// Last snapshot (for delta calculation)
    last_snapshot: Arc<Mutex<Option<TelemetrySnapshot>>>,

    /// Queue event counter
    event_counter: Arc<Mutex<EventCounter>>,
}

/// Event counter for rate calculation
struct EventCounter {
    /// Events with timestamps (last 60 seconds)
    events: Vec<(i64, String)>, // (unix_timestamp, domain)

    /// Total events since startup
    total_events: u64,
}

impl EventCounter {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            total_events: 0,
        }
    }

    fn add_event(&mut self, domain: String) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        self.events.push((now, domain));
        self.total_events += 1;

        // Clean up events older than 60 seconds
        let cutoff = now - 60;
        self.events.retain(|(ts, _)| *ts > cutoff);
    }

    fn get_rate(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }

        // Calculate events per second over last 60s
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let cutoff = now - 60;
        let recent_events = self.events.iter().filter(|(ts, _)| *ts > cutoff).count();

        recent_events as f64 / 60.0
    }

    fn get_by_domain(&self) -> std::collections::HashMap<String, u64> {
        let mut counts = std::collections::HashMap::new();
        for (_, domain) in &self.events {
            *counts.entry(domain.clone()).or_insert(0) += 1;
        }
        counts
    }
}

impl SnapshotAggregator {
    /// Create a new snapshot aggregator
    pub fn new() -> Self {
        Self {
            last_snapshot: Arc::new(Mutex::new(None)),
            event_counter: Arc::new(Mutex::new(EventCounter::new())),
        }
    }

    /// Record an event for rate calculation
    pub async fn record_event(&self, domain: String) {
        let mut counter = self.event_counter.lock().await;
        counter.add_event(domain);
    }

    /// Capture current telemetry snapshot
    pub async fn capture_snapshot(&self, queue_depth: usize) -> TelemetrySnapshot {
        let mut snapshot = TelemetrySnapshot::new();

        // Queue metrics
        let counter = self.event_counter.lock().await;
        snapshot.queue.depth = queue_depth;
        snapshot.queue.rate_per_sec = counter.get_rate();
        snapshot.queue.total_processed = counter.total_events;
        snapshot.queue.oldest_event_sec = 0; // TODO: track oldest event

        // Event metrics
        snapshot.events.last_5min = counter.events.len() as u64;
        snapshot.events.by_domain = counter.get_by_domain();

        // Resource metrics
        snapshot.resources = Self::collect_resource_metrics();

        // Module activity
        snapshot.modules = Self::collect_module_activity();

        // Store for delta calculation
        let mut last = self.last_snapshot.lock().await;
        *last = Some(snapshot.clone());

        snapshot
    }

    /// Get last snapshot (for delta calculation)
    pub async fn get_last_snapshot(&self) -> Option<TelemetrySnapshot> {
        let last = self.last_snapshot.lock().await;
        last.clone()
    }

    fn collect_resource_metrics() -> ResourceMetrics {
        // Read /proc/self/status for memory info
        let memory_bytes = Self::read_memory_usage().unwrap_or(0);
        let memory_human = format_bytes(memory_bytes);

        ResourceMetrics {
            memory_bytes,
            memory_human,
            cpu_percent: 0.0, // TODO: calculate CPU usage
            thread_count: Self::read_thread_count().unwrap_or(0),
            fd_count: Self::read_fd_count().unwrap_or(0),
        }
    }

    fn read_memory_usage() -> Option<u64> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().ok()?;
                    return Some(kb * 1024);
                }
            }
        }
        None
    }

    fn read_thread_count() -> Option<usize> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if line.starts_with("Threads:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse().ok();
                }
            }
        }
        None
    }

    fn read_fd_count() -> Option<usize> {
        std::fs::read_dir("/proc/self/fd").ok()?.count().into()
    }

    fn collect_module_activity() -> ModuleActivity {
        ModuleActivity {
            telemetry_active: true,
            event_engine_active: true,
            policy_engine_active: true,
            storage_active: true,
            rpc_active: true,
            last_activity: std::collections::HashMap::new(),
        }
    }
}

impl Default for SnapshotAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = TelemetrySnapshot::new();
        assert!(snapshot.timestamp_unix > 0);
        assert!(!snapshot.timestamp.is_empty());
    }

    #[test]
    fn test_snapshot_delta() {
        let mut snap1 = TelemetrySnapshot::new();
        snap1.timestamp_unix = 1000;
        snap1.queue.depth = 10;
        snap1.resources.memory_bytes = 1024;

        let mut snap2 = TelemetrySnapshot::new();
        snap2.timestamp_unix = 1005;
        snap2.queue.depth = 15;
        snap2.resources.memory_bytes = 2048;

        let delta = snap2.delta(&snap1);
        assert_eq!(delta.time_delta_sec, 5);
        assert_eq!(delta.queue_depth_delta, 5);
        assert_eq!(delta.memory_delta_bytes, 1024);
    }

    #[test]
    fn test_event_counter_rate() {
        let mut counter = EventCounter::new();

        // Add 10 events
        for _ in 0..10 {
            counter.add_event("test".to_string());
        }

        let rate = counter.get_rate();
        assert!(rate > 0.0);
        assert!(rate <= 10.0 / 60.0 + 0.01); // Allow small margin
    }

    #[test]
    fn test_event_counter_by_domain() {
        let mut counter = EventCounter::new();

        counter.add_event("cpu".to_string());
        counter.add_event("cpu".to_string());
        counter.add_event("memory".to_string());

        let by_domain = counter.get_by_domain();
        assert_eq!(by_domain.get("cpu"), Some(&2));
        assert_eq!(by_domain.get("memory"), Some(&1));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.0 B");
        assert_eq!(format_bytes(500), "500.0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[tokio::test]
    async fn test_aggregator_capture() {
        let aggregator = SnapshotAggregator::new();

        // Record some events
        aggregator.record_event("test".to_string()).await;
        aggregator.record_event("test".to_string()).await;

        // Capture snapshot
        let snapshot = aggregator.capture_snapshot(5).await;

        assert_eq!(snapshot.queue.depth, 5);
        assert!(snapshot.queue.total_processed >= 2);
    }

    #[tokio::test]
    async fn test_aggregator_last_snapshot() {
        let aggregator = SnapshotAggregator::new();

        // Initially no snapshot
        assert!(aggregator.get_last_snapshot().await.is_none());

        // Capture snapshot
        aggregator.capture_snapshot(0).await;

        // Now should have snapshot
        assert!(aggregator.get_last_snapshot().await.is_some());
    }

    #[test]
    fn test_queue_metrics_default() {
        let metrics = QueueMetrics::default();
        assert_eq!(metrics.depth, 0);
        assert_eq!(metrics.rate_per_sec, 0.0);
        assert_eq!(metrics.total_processed, 0);
    }

    #[test]
    fn test_resource_metrics_default() {
        let metrics = ResourceMetrics::default();
        assert_eq!(metrics.memory_bytes, 0);
        assert_eq!(metrics.cpu_percent, 0.0);
        assert_eq!(metrics.thread_count, 0);
    }
}
