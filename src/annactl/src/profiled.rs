//! Continuous Profiling Daemon for Anna v0.14.0 "Orion III" Phase 2.2
//!
//! Self-monitoring system that tracks Anna's own runtime performance.
//! Detects performance drift and environment anomalies in real time.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSnapshot {
    pub timestamp: u64,
    pub rpc_latency_ms: f32,     // RPC call latency
    pub memory_mb: f32,           // Memory usage in MB
    pub io_latency_ms: f32,       // I/O operation latency
    pub cpu_percent: f32,         // CPU usage percentage
    pub queue_depth: u32,         // Event queue depth
}

impl PerfSnapshot {
    /// Create new performance snapshot
    pub fn capture() -> Result<Self> {
        let start = Instant::now();

        // Measure RPC latency (simulated with a fast check)
        let rpc_start = Instant::now();
        let _ = std::path::Path::new("/run/anna/annad.sock").exists();
        let rpc_latency_ms = rpc_start.elapsed().as_micros() as f32 / 1000.0;

        // Get memory usage
        let memory_mb = Self::get_memory_usage()?;

        // Measure I/O latency (read small state file)
        let io_latency_ms = Self::measure_io_latency()?;

        // Get CPU usage (from /proc/self/stat)
        let cpu_percent = Self::get_cpu_usage()?;

        // Queue depth (placeholder - would query daemon)
        let queue_depth = 0;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Verify we're under 50ms target
        let capture_time = start.elapsed();
        if capture_time > Duration::from_millis(50) {
            eprintln!("Warning: Performance snapshot took {}ms (target: <50ms)", capture_time.as_millis());
        }

        Ok(Self {
            timestamp,
            rpc_latency_ms,
            memory_mb,
            io_latency_ms,
            cpu_percent,
            queue_depth,
        })
    }

    /// Get current memory usage in MB
    fn get_memory_usage() -> Result<f32> {
        // Read /proc/self/status for VmRSS
        let status = fs::read_to_string("/proc/self/status")
            .context("Failed to read /proc/self/status")?;

        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f32>() {
                        return Ok(kb / 1024.0);  // Convert KB to MB
                    }
                }
            }
        }

        Ok(0.0)
    }

    /// Measure I/O latency by reading a small file
    fn measure_io_latency() -> Result<f32> {
        let start = Instant::now();
        let state_dir = Self::get_state_dir()?;
        let _ = state_dir.exists();  // Quick filesystem check
        Ok(start.elapsed().as_micros() as f32 / 1000.0)
    }

    /// Get CPU usage percentage
    fn get_cpu_usage() -> Result<f32> {
        // Read /proc/self/stat for CPU time
        let stat = fs::read_to_string("/proc/self/stat")
            .context("Failed to read /proc/self/stat")?;

        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() < 15 {
            return Ok(0.0);
        }

        // Extract utime (14) and stime (15) - simplified
        let utime = parts.get(13).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let stime = parts.get(14).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        // Simplified CPU calculation (would need delta over time for accuracy)
        let total_time = (utime + stime) as f32;
        let cpu_percent = (total_time / 100.0).min(100.0);

        Ok(cpu_percent)
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }
}

/// Performance baseline (7-day average)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBaseline {
    pub avg_rpc_latency_ms: f32,
    pub avg_memory_mb: f32,
    pub avg_io_latency_ms: f32,
    pub avg_cpu_percent: f32,
    pub sample_count: u32,
    pub created_at: u64,
}

impl PerfBaseline {
    /// Create baseline from recent snapshots
    pub fn from_snapshots(snapshots: &[PerfSnapshot]) -> Self {
        if snapshots.is_empty() {
            return Self::default();
        }

        let count = snapshots.len() as f32;
        let sum_rpc: f32 = snapshots.iter().map(|s| s.rpc_latency_ms).sum();
        let sum_mem: f32 = snapshots.iter().map(|s| s.memory_mb).sum();
        let sum_io: f32 = snapshots.iter().map(|s| s.io_latency_ms).sum();
        let sum_cpu: f32 = snapshots.iter().map(|s| s.cpu_percent).sum();

        Self {
            avg_rpc_latency_ms: sum_rpc / count,
            avg_memory_mb: sum_mem / count,
            avg_io_latency_ms: sum_io / count,
            avg_cpu_percent: sum_cpu / count,
            sample_count: snapshots.len() as u32,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Default baseline
    pub fn default() -> Self {
        Self {
            avg_rpc_latency_ms: 1.0,
            avg_memory_mb: 50.0,
            avg_io_latency_ms: 0.5,
            avg_cpu_percent: 5.0,
            sample_count: 0,
            created_at: 0,
        }
    }
}

/// Performance degradation classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegradationLevel {
    Normal,
    Minor,      // >15% above baseline
    Moderate,   // >30% above baseline
    Critical,   // >50% above baseline
}

impl DegradationLevel {
    /// Classify degradation from percentage above baseline
    pub fn from_percent(pct: f32) -> Self {
        if pct > 50.0 {
            Self::Critical
        } else if pct > 30.0 {
            Self::Moderate
        } else if pct > 15.0 {
            Self::Minor
        } else {
            Self::Normal
        }
    }

    /// Get emoji for level
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Normal => "✅",
            Self::Minor => "⚠️",
            Self::Moderate => "🔶",
            Self::Critical => "🔴",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Minor => "Minor Degradation",
            Self::Moderate => "Moderate Degradation",
            Self::Critical => "Critical Degradation",
        }
    }
}

/// Performance watch entry (logged to perfwatch.jsonl)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfWatchEntry {
    pub timestamp: u64,
    pub snapshot: PerfSnapshot,
    pub baseline: PerfBaseline,
    pub degradation: DegradationLevel,
    pub rpc_delta_pct: f32,
    pub memory_delta_pct: f32,
    pub io_delta_pct: f32,
    pub cpu_delta_pct: f32,
}

impl PerfWatchEntry {
    /// Create watch entry by comparing snapshot to baseline
    pub fn new(snapshot: PerfSnapshot, baseline: PerfBaseline) -> Self {
        let rpc_delta_pct = Self::calc_delta_pct(snapshot.rpc_latency_ms, baseline.avg_rpc_latency_ms);
        let memory_delta_pct = Self::calc_delta_pct(snapshot.memory_mb, baseline.avg_memory_mb);
        let io_delta_pct = Self::calc_delta_pct(snapshot.io_latency_ms, baseline.avg_io_latency_ms);
        let cpu_delta_pct = Self::calc_delta_pct(snapshot.cpu_percent, baseline.avg_cpu_percent);

        // Overall degradation is worst of all metrics
        let max_delta = rpc_delta_pct.max(memory_delta_pct).max(io_delta_pct).max(cpu_delta_pct);
        let degradation = DegradationLevel::from_percent(max_delta);

        Self {
            timestamp: snapshot.timestamp,
            snapshot,
            baseline,
            degradation,
            rpc_delta_pct,
            memory_delta_pct,
            io_delta_pct,
            cpu_delta_pct,
        }
    }

    /// Calculate delta percentage
    fn calc_delta_pct(current: f32, baseline: f32) -> f32 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }
}

/// Performance profiler
pub struct Profiler {
    perfwatch_path: PathBuf,
    baseline_path: PathBuf,
}

impl Profiler {
    /// Create new profiler
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let perfwatch_path = state_dir.join("perfwatch.jsonl");
        let baseline_path = state_dir.join("perfbaseline.json");

        Ok(Self {
            perfwatch_path,
            baseline_path,
        })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Capture and log performance snapshot
    pub fn tick(&self) -> Result<PerfWatchEntry> {
        let snapshot = PerfSnapshot::capture()?;
        let baseline = self.load_baseline()?;
        let entry = PerfWatchEntry::new(snapshot, baseline);

        // Log to perfwatch.jsonl
        self.log_entry(&entry)?;

        Ok(entry)
    }

    /// Load baseline from disk
    pub fn load_baseline(&self) -> Result<PerfBaseline> {
        if !self.baseline_path.exists() {
            return Ok(PerfBaseline::default());
        }

        let content = fs::read_to_string(&self.baseline_path)?;
        let baseline: PerfBaseline = serde_json::from_str(&content)?;
        Ok(baseline)
    }

    /// Save baseline to disk
    pub fn save_baseline(&self, baseline: &PerfBaseline) -> Result<()> {
        let json = serde_json::to_string_pretty(baseline)?;
        fs::write(&self.baseline_path, json)?;
        Ok(())
    }

    /// Log entry to perfwatch.jsonl
    fn log_entry(&self, entry: &PerfWatchEntry) -> Result<()> {
        let json = serde_json::to_string(entry)?;
        let mut content = String::new();

        if self.perfwatch_path.exists() {
            content = fs::read_to_string(&self.perfwatch_path)?;
        }

        content.push_str(&json);
        content.push('\n');

        fs::write(&self.perfwatch_path, content)?;
        Ok(())
    }

    /// Load all watch entries
    pub fn load_all(&self) -> Result<Vec<PerfWatchEntry>> {
        if !self.perfwatch_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.perfwatch_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<PerfWatchEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse perfwatch entry: {}", e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    /// Get recent entries (last N)
    pub fn get_recent(&self, n: usize) -> Result<Vec<PerfWatchEntry>> {
        let mut entries = self.load_all()?;

        if entries.len() > n {
            entries.drain(0..(entries.len() - n));
        }

        Ok(entries)
    }

    /// Rebuild baseline from last 7 days
    pub fn rebuild_baseline(&self) -> Result<PerfBaseline> {
        let entries = self.load_all()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let seven_days_ago = now.saturating_sub(7 * 24 * 3600);

        // Filter to last 7 days
        let recent: Vec<PerfSnapshot> = entries
            .into_iter()
            .filter(|e| e.timestamp >= seven_days_ago)
            .map(|e| e.snapshot)
            .collect();

        let baseline = PerfBaseline::from_snapshots(&recent);
        self.save_baseline(&baseline)?;

        Ok(baseline)
    }

    /// Detect persistent degradation (3+ consecutive cycles)
    pub fn detect_persistent_degradation(&self) -> Result<Option<DegradationLevel>> {
        let recent = self.get_recent(3)?;

        if recent.len() < 3 {
            return Ok(None);
        }

        // Check if last 3 are degraded
        let all_degraded = recent.iter().all(|e| e.degradation != DegradationLevel::Normal);

        if all_degraded {
            // Return worst level
            let worst = recent
                .iter()
                .map(|e| e.degradation)
                .max_by_key(|d| *d as u8)
                .unwrap();
            Ok(Some(worst))
        } else {
            Ok(None)
        }
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> Result<ProfilerSummary> {
        let entries = self.load_all()?;
        let baseline = self.load_baseline()?;

        let total_entries = entries.len();
        let degraded_count = entries.iter().filter(|e| e.degradation != DegradationLevel::Normal).count();

        let avg_rpc = if !entries.is_empty() {
            entries.iter().map(|e| e.snapshot.rpc_latency_ms).sum::<f32>() / entries.len() as f32
        } else {
            0.0
        };

        let avg_memory = if !entries.is_empty() {
            entries.iter().map(|e| e.snapshot.memory_mb).sum::<f32>() / entries.len() as f32
        } else {
            0.0
        };

        Ok(ProfilerSummary {
            total_entries,
            degraded_count,
            baseline,
            current_avg_rpc_ms: avg_rpc,
            current_avg_memory_mb: avg_memory,
        })
    }
}

/// Profiler summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerSummary {
    pub total_entries: usize,
    pub degraded_count: usize,
    pub baseline: PerfBaseline,
    pub current_avg_rpc_ms: f32,
    pub current_avg_memory_mb: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_snapshot_creation() {
        let snapshot = PerfSnapshot::capture();
        assert!(snapshot.is_ok());

        let s = snapshot.unwrap();
        assert!(s.timestamp > 0);
        assert!(s.rpc_latency_ms >= 0.0);
        assert!(s.memory_mb >= 0.0);
    }

    #[test]
    fn test_baseline_from_snapshots() {
        let snapshots = vec![
            PerfSnapshot {
                timestamp: 12345,
                rpc_latency_ms: 1.0,
                memory_mb: 50.0,
                io_latency_ms: 0.5,
                cpu_percent: 5.0,
                queue_depth: 0,
            },
            PerfSnapshot {
                timestamp: 12346,
                rpc_latency_ms: 2.0,
                memory_mb: 60.0,
                io_latency_ms: 0.7,
                cpu_percent: 7.0,
                queue_depth: 0,
            },
        ];

        let baseline = PerfBaseline::from_snapshots(&snapshots);
        assert_eq!(baseline.avg_rpc_latency_ms, 1.5);
        assert_eq!(baseline.avg_memory_mb, 55.0);
        assert_eq!(baseline.sample_count, 2);
    }

    #[test]
    fn test_degradation_classification() {
        assert_eq!(DegradationLevel::from_percent(10.0), DegradationLevel::Normal);
        assert_eq!(DegradationLevel::from_percent(20.0), DegradationLevel::Minor);
        assert_eq!(DegradationLevel::from_percent(35.0), DegradationLevel::Moderate);
        assert_eq!(DegradationLevel::from_percent(60.0), DegradationLevel::Critical);
    }

    #[test]
    fn test_perfwatch_entry_creation() {
        let snapshot = PerfSnapshot {
            timestamp: 12345,
            rpc_latency_ms: 2.0,
            memory_mb: 60.0,
            io_latency_ms: 1.0,
            cpu_percent: 10.0,
            queue_depth: 0,
        };

        let baseline = PerfBaseline {
            avg_rpc_latency_ms: 1.0,
            avg_memory_mb: 50.0,
            avg_io_latency_ms: 0.5,
            avg_cpu_percent: 5.0,
            sample_count: 10,
            created_at: 12000,
        };

        let entry = PerfWatchEntry::new(snapshot, baseline);
        assert_eq!(entry.rpc_delta_pct, 100.0);  // 2x baseline
        assert_eq!(entry.memory_delta_pct, 20.0);  // 20% above
    }

    #[test]
    fn test_delta_calculation() {
        assert_eq!(PerfWatchEntry::calc_delta_pct(20.0, 10.0), 100.0);
        assert_eq!(PerfWatchEntry::calc_delta_pct(15.0, 10.0), 50.0);
        assert_eq!(PerfWatchEntry::calc_delta_pct(10.0, 10.0), 0.0);
        assert_eq!(PerfWatchEntry::calc_delta_pct(5.0, 10.0), -50.0);
    }

    #[test]
    fn test_degradation_emoji() {
        assert_eq!(DegradationLevel::Normal.emoji(), "✅");
        assert_eq!(DegradationLevel::Minor.emoji(), "⚠️");
        assert_eq!(DegradationLevel::Moderate.emoji(), "🔶");
        assert_eq!(DegradationLevel::Critical.emoji(), "🔴");
    }

    #[test]
    fn test_degradation_name() {
        assert_eq!(DegradationLevel::Normal.name(), "Normal");
        assert_eq!(DegradationLevel::Minor.name(), "Minor Degradation");
        assert_eq!(DegradationLevel::Moderate.name(), "Moderate Degradation");
        assert_eq!(DegradationLevel::Critical.name(), "Critical Degradation");
    }

    #[test]
    fn test_state_dir_path() {
        if let Ok(dir) = Profiler::get_state_dir() {
            assert!(dir.to_string_lossy().contains(".local/state/anna"));
        }
    }
}
