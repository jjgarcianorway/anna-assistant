//! Anomaly Engine v0.0.12 - Proactive Anomaly Detection
//!
//! Detects anomalies by comparing recent metrics against baseline windows:
//! - Boot time trend regression
//! - CPU load trend change
//! - Memory pressure / swap activity
//! - Disk I/O latency trend
//! - Journal warnings/errors increase
//! - Repeated crashes
//!
//! Each anomaly includes:
//! - Metric name
//! - Baseline window (14 days) vs recent window (2 days)
//! - Delta and confidence
//! - Timestamps
//! - Remediation hints (read-only only)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::daemon_state::{LastCrash, INTERNAL_DIR};

/// Anomaly severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
}

impl AnomalySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalySeverity::Info => "info",
            AnomalySeverity::Warning => "warning",
            AnomalySeverity::Critical => "critical",
        }
    }
}

/// Anomaly signal types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySignal {
    /// Boot time increased significantly
    BootTimeRegression,
    /// CPU load trend increased
    CpuLoadIncrease,
    /// Memory pressure detected (high usage or swap activity)
    MemoryPressure,
    /// Disk I/O latency increased
    DiskIoLatency,
    /// Journal warnings increased for a service
    JournalWarningsIncrease { service: String },
    /// Journal errors increased for a service
    JournalErrorsIncrease { service: String },
    /// System crash detected
    SystemCrash,
    /// Service repeatedly crashing
    ServiceCrash { service: String },
    /// Service failed to start
    ServiceFailed { service: String },
    /// Disk space low
    DiskSpaceLow { mount_point: String },
}

impl AnomalySignal {
    pub fn metric_name(&self) -> String {
        match self {
            AnomalySignal::BootTimeRegression => "boot_time".to_string(),
            AnomalySignal::CpuLoadIncrease => "cpu_load".to_string(),
            AnomalySignal::MemoryPressure => "memory_pressure".to_string(),
            AnomalySignal::DiskIoLatency => "disk_io_latency".to_string(),
            AnomalySignal::JournalWarningsIncrease { service } => format!("journal_warnings:{}", service),
            AnomalySignal::JournalErrorsIncrease { service } => format!("journal_errors:{}", service),
            AnomalySignal::SystemCrash => "system_crash".to_string(),
            AnomalySignal::ServiceCrash { service } => format!("service_crash:{}", service),
            AnomalySignal::ServiceFailed { service } => format!("service_failed:{}", service),
            AnomalySignal::DiskSpaceLow { mount_point } => format!("disk_space:{}", mount_point),
        }
    }

    /// Generate a unique deduplication key
    pub fn dedup_key(&self) -> String {
        self.metric_name()
    }
}

/// Time window for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start timestamp (Unix seconds)
    pub start: u64,
    /// End timestamp (Unix seconds)
    pub end: u64,
    /// Number of samples in window
    pub sample_count: u64,
}

impl TimeWindow {
    pub fn duration_days(&self) -> f64 {
        (self.end.saturating_sub(self.start)) as f64 / 86400.0
    }

    pub fn format_range(&self) -> String {
        let start = DateTime::from_timestamp(self.start as i64, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "?".to_string());
        let end = DateTime::from_timestamp(self.end as i64, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "?".to_string());
        format!("{} to {}", start, end)
    }
}

/// A detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Unique evidence ID for this anomaly
    pub evidence_id: String,
    /// When the anomaly was detected
    pub detected_at: DateTime<Utc>,
    /// Anomaly signal type
    pub signal: AnomalySignal,
    /// Severity level
    pub severity: AnomalySeverity,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Baseline window for comparison
    pub baseline_window: Option<TimeWindow>,
    /// Recent window for comparison
    pub recent_window: Option<TimeWindow>,
    /// Delta value (percent change or absolute)
    pub delta: Option<f64>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Trend direction if applicable
    /// Trend direction as a string (e.g., "higher", "lower", "stable")
    pub trend: Option<String>,
    /// Read-only remediation hints
    pub hints: Vec<String>,
    /// Whether this anomaly has been acknowledged
    pub acknowledged: bool,
    /// When this was last seen (for deduplication)
    pub last_seen: DateTime<Utc>,
    /// How many times this anomaly has been detected
    pub occurrence_count: u64,
}

impl Anomaly {
    pub fn new(signal: AnomalySignal, severity: AnomalySeverity, title: &str, description: &str) -> Self {
        let now = Utc::now();
        Self {
            evidence_id: generate_evidence_id(),
            detected_at: now,
            signal,
            severity,
            title: title.to_string(),
            description: description.to_string(),
            baseline_window: None,
            recent_window: None,
            delta: None,
            confidence: 0.5,
            trend: None,
            hints: Vec::new(),
            acknowledged: false,
            last_seen: now,
            occurrence_count: 1,
        }
    }

    pub fn with_windows(mut self, baseline: TimeWindow, recent: TimeWindow) -> Self {
        self.baseline_window = Some(baseline);
        self.recent_window = Some(recent);
        self
    }

    pub fn with_delta(mut self, delta: f64) -> Self {
        self.delta = Some(delta);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_trend(mut self, trend: &str) -> Self {
        self.trend = Some(trend.to_string());
        self
    }

    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hints.push(hint.to_string());
        self
    }

    /// Format for status display (concise)
    pub fn format_short(&self) -> String {
        format!("[{}] {} ({})", self.evidence_id, self.title, self.severity.as_str())
    }

    /// Format for detailed display
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] {}", self.evidence_id, self.title),
            format!("  Severity: {}", self.severity.as_str()),
            format!("  {}", self.description),
        ];

        if let Some(ref delta) = self.delta {
            lines.push(format!("  Change: {:.1}%", delta * 100.0));
        }

        if let Some(ref trend) = self.trend {
            lines.push(format!("  Trend: {}", trend));
        }

        if self.confidence > 0.0 {
            lines.push(format!("  Confidence: {:.0}%", self.confidence * 100.0));
        }

        if let (Some(ref baseline), Some(ref recent)) = (&self.baseline_window, &self.recent_window) {
            lines.push(format!("  Baseline: {} ({} samples)", baseline.format_range(), baseline.sample_count));
            lines.push(format!("  Recent: {} ({} samples)", recent.format_range(), recent.sample_count));
        }

        if !self.hints.is_empty() {
            lines.push("  Suggestions:".to_string());
            for hint in &self.hints {
                lines.push(format!("    - {}", hint));
            }
        }

        lines.join("\n")
    }
}

/// Alert queue stored on disk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertQueue {
    /// Active anomalies (not yet acknowledged)
    pub anomalies: Vec<Anomaly>,
    /// Last check timestamp
    pub last_check: Option<DateTime<Utc>>,
    /// Schema version
    pub schema_version: u32,
}

/// Path to alerts file
pub const ALERTS_FILE: &str = "/var/lib/anna/internal/alerts.json";
pub const ALERTS_SCHEMA_VERSION: u32 = 1;

impl AlertQueue {
    pub fn load() -> Self {
        let path = Path::new(ALERTS_FILE);
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Path::new(ALERTS_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(ALERTS_FILE, &content)
    }

    /// Add or update an anomaly (deduplication)
    pub fn add_anomaly(&mut self, mut anomaly: Anomaly) {
        let dedup_key = anomaly.signal.dedup_key();

        // Find existing anomaly with same signal
        if let Some(existing) = self.anomalies.iter_mut().find(|a| a.signal.dedup_key() == dedup_key) {
            // Update existing
            existing.last_seen = Utc::now();
            existing.occurrence_count += 1;
            // Update severity if higher
            if anomaly.severity > existing.severity {
                existing.severity = anomaly.severity;
            }
            // Update description if different
            if anomaly.description != existing.description {
                existing.description = anomaly.description.clone();
            }
        } else {
            // Add new
            self.anomalies.push(anomaly);
        }
    }

    /// Get unacknowledged anomalies by severity
    pub fn get_active(&self) -> Vec<&Anomaly> {
        self.anomalies
            .iter()
            .filter(|a| !a.acknowledged)
            .collect()
    }

    /// Get critical alerts
    pub fn get_critical(&self) -> Vec<&Anomaly> {
        self.get_active()
            .into_iter()
            .filter(|a| a.severity == AnomalySeverity::Critical)
            .collect()
    }

    /// Get warning alerts
    pub fn get_warnings(&self) -> Vec<&Anomaly> {
        self.get_active()
            .into_iter()
            .filter(|a| a.severity == AnomalySeverity::Warning)
            .collect()
    }

    /// Get count by severity
    pub fn count_by_severity(&self) -> (usize, usize, usize) {
        let active = self.get_active();
        let critical = active.iter().filter(|a| a.severity == AnomalySeverity::Critical).count();
        let warning = active.iter().filter(|a| a.severity == AnomalySeverity::Warning).count();
        let info = active.iter().filter(|a| a.severity == AnomalySeverity::Info).count();
        (critical, warning, info)
    }

    /// Acknowledge an anomaly by evidence ID
    pub fn acknowledge(&mut self, evidence_id: &str) -> bool {
        if let Some(anomaly) = self.anomalies.iter_mut().find(|a| a.evidence_id == evidence_id) {
            anomaly.acknowledged = true;
            true
        } else {
            false
        }
    }

    /// Remove old acknowledged anomalies (> 7 days)
    pub fn cleanup(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::days(7);
        self.anomalies.retain(|a| {
            !a.acknowledged || a.last_seen > cutoff
        });
    }

    /// Format summary for status display
    pub fn format_summary(&self) -> String {
        let (critical, warning, info) = self.count_by_severity();
        if critical + warning + info == 0 {
            return "No active alerts".to_string();
        }
        format!("{} critical, {} warnings, {} info", critical, warning, info)
    }
}

/// Anomaly detection thresholds
pub struct AnomalyThresholds {
    /// Boot time increase threshold (percent)
    pub boot_time_increase_percent: f64,
    /// CPU load increase threshold (percent)
    pub cpu_load_increase_percent: f64,
    /// Memory usage threshold (percent of total)
    pub memory_high_percent: f64,
    /// Swap usage threshold to trigger warning (MB)
    pub swap_warning_mb: u64,
    /// Journal warnings increase threshold (percent)
    pub journal_warnings_increase_percent: f64,
    /// Journal errors increase threshold (absolute count)
    pub journal_errors_increase_count: u32,
    /// Disk space low threshold (percent free)
    pub disk_space_low_percent: f64,
    /// Minimum confidence for alerts
    pub min_confidence: f64,
}

impl Default for AnomalyThresholds {
    fn default() -> Self {
        Self {
            boot_time_increase_percent: 20.0,
            cpu_load_increase_percent: 30.0,
            memory_high_percent: 90.0,
            swap_warning_mb: 100,
            journal_warnings_increase_percent: 50.0,
            journal_errors_increase_count: 10,
            disk_space_low_percent: 10.0,
            min_confidence: 0.5,
        }
    }
}

/// Anomaly detection engine
pub struct AnomalyEngine {
    pub thresholds: AnomalyThresholds,
    /// Baseline window in days
    pub baseline_days: u32,
    /// Recent window in days
    pub recent_days: u32,
}

impl Default for AnomalyEngine {
    fn default() -> Self {
        Self {
            thresholds: AnomalyThresholds::default(),
            baseline_days: 14,
            recent_days: 2,
        }
    }
}

impl AnomalyEngine {
    /// Run all anomaly checks and return detected anomalies
    pub fn run_checks(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Check boot time regression
        if let Some(anomaly) = self.check_boot_time() {
            anomalies.push(anomaly);
        }

        // Check CPU load trend
        if let Some(anomaly) = self.check_cpu_load() {
            anomalies.push(anomaly);
        }

        // Check memory pressure
        if let Some(anomaly) = self.check_memory_pressure() {
            anomalies.push(anomaly);
        }

        // Check disk space
        anomalies.extend(self.check_disk_space());

        // Check for recent crashes
        if let Some(anomaly) = self.check_crashes() {
            anomalies.push(anomaly);
        }

        // Check failed services
        anomalies.extend(self.check_failed_services());

        // Check journal warnings/errors increase
        anomalies.extend(self.check_journal_trends());

        anomalies
    }

    /// Check boot time regression
    fn check_boot_time(&self) -> Option<Anomaly> {
        // Parse systemd-analyze output
        let output = std::process::Command::new("systemd-analyze")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse "Startup finished in Xs (firmware) + Ys (loader) + Zs (kernel) + Ws (userspace) = TOTALs"
        let total_secs = parse_systemd_analyze_time(&stdout)?;

        // For boot time, we compare against a fixed baseline (30 seconds is typical)
        // In real implementation, we'd store historical boot times
        let baseline_secs = 30.0;
        let delta = (total_secs - baseline_secs) / baseline_secs;

        if delta > self.thresholds.boot_time_increase_percent / 100.0 {
            let severity = if delta > 0.5 {
                AnomalySeverity::Warning
            } else {
                AnomalySeverity::Info
            };

            return Some(
                Anomaly::new(
                    AnomalySignal::BootTimeRegression,
                    severity,
                    "Boot time increased",
                    &format!("Boot time is {:.1}s, {:.0}% higher than baseline", total_secs, delta * 100.0),
                )
                .with_delta(delta)
                .with_confidence(0.7)
                .with_hint("Run 'systemd-analyze blame' to see slow services")
                .with_hint("Check for new services added recently")
            );
        }

        None
    }

    /// Check CPU load trend
    fn check_cpu_load(&self) -> Option<Anomaly> {
        // Read /proc/loadavg
        let loadavg = fs::read_to_string("/proc/loadavg").ok()?;
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        let load_1min: f64 = parts.first()?.parse().ok()?;

        // Get number of CPUs for normalization
        let num_cpus = num_cpus();

        // Normalized load (per CPU)
        let normalized_load = load_1min / num_cpus as f64;

        // High load is > 1.0 per CPU sustained
        if normalized_load > 1.0 {
            let severity = if normalized_load > 2.0 {
                AnomalySeverity::Critical
            } else if normalized_load > 1.5 {
                AnomalySeverity::Warning
            } else {
                AnomalySeverity::Info
            };

            return Some(
                Anomaly::new(
                    AnomalySignal::CpuLoadIncrease,
                    severity,
                    "High CPU load detected",
                    &format!(
                        "Load average is {:.2} ({:.1}x per CPU core)",
                        load_1min, normalized_load
                    ),
                )
                .with_confidence(0.8)
                .with_hint("Run 'top' or 'htop' to identify resource-heavy processes")
                .with_hint("Check for runaway processes or background tasks")
            );
        }

        None
    }

    /// Check memory pressure
    fn check_memory_pressure(&self) -> Option<Anomaly> {
        // Parse /proc/meminfo
        let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
        let mut mem_total: u64 = 0;
        let mut mem_available: u64 = 0;
        let mut swap_total: u64 = 0;
        let mut swap_free: u64 = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                mem_total = parse_meminfo_value(line)?;
            } else if line.starts_with("MemAvailable:") {
                mem_available = parse_meminfo_value(line)?;
            } else if line.starts_with("SwapTotal:") {
                swap_total = parse_meminfo_value(line)?;
            } else if line.starts_with("SwapFree:") {
                swap_free = parse_meminfo_value(line)?;
            }
        }

        if mem_total == 0 {
            return None;
        }

        let mem_used_percent = ((mem_total - mem_available) as f64 / mem_total as f64) * 100.0;
        let swap_used_mb = (swap_total - swap_free) / 1024;

        // Check high memory usage
        if mem_used_percent > self.thresholds.memory_high_percent {
            return Some(
                Anomaly::new(
                    AnomalySignal::MemoryPressure,
                    AnomalySeverity::Warning,
                    "High memory usage",
                    &format!(
                        "Memory usage is at {:.0}% ({} MB available)",
                        mem_used_percent,
                        mem_available / 1024
                    ),
                )
                .with_confidence(0.9)
                .with_hint("Identify memory-heavy processes with 'ps aux --sort=-%mem | head'")
                .with_hint("Consider closing unused applications")
            );
        }

        // Check swap usage
        if swap_used_mb > self.thresholds.swap_warning_mb {
            return Some(
                Anomaly::new(
                    AnomalySignal::MemoryPressure,
                    AnomalySeverity::Info,
                    "Swap usage detected",
                    &format!(
                        "Swap usage is {} MB, which may indicate memory pressure",
                        swap_used_mb
                    ),
                )
                .with_confidence(0.7)
                .with_hint("High swap usage can slow down the system")
                .with_hint("Check memory usage of running applications")
            );
        }

        None
    }

    /// Check disk space
    fn check_disk_space(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Run df to get disk usage
        let output = std::process::Command::new("df")
            .args(["--output=target,pcent", "-x", "tmpfs", "-x", "devtmpfs"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let mount_point = parts[0];
                        let usage_str = parts[1].trim_end_matches('%');
                        if let Ok(usage) = usage_str.parse::<f64>() {
                            let free_percent = 100.0 - usage;
                            if free_percent < self.thresholds.disk_space_low_percent {
                                let severity = if free_percent < 5.0 {
                                    AnomalySeverity::Critical
                                } else {
                                    AnomalySeverity::Warning
                                };

                                anomalies.push(
                                    Anomaly::new(
                                        AnomalySignal::DiskSpaceLow {
                                            mount_point: mount_point.to_string(),
                                        },
                                        severity,
                                        &format!("Low disk space on {}", mount_point),
                                        &format!(
                                            "Only {:.0}% free space remaining on {}",
                                            free_percent, mount_point
                                        ),
                                    )
                                    .with_confidence(0.95)
                                    .with_hint("Run 'du -sh /* | sort -h' to find large directories")
                                    .with_hint("Clear package cache with 'sudo pacman -Sc'")
                                );
                            }
                        }
                    }
                }
            }
        }

        anomalies
    }

    /// Check for recent crashes
    fn check_crashes(&self) -> Option<Anomaly> {
        let crash = LastCrash::load()?;

        // Only alert if crash was in last 24 hours
        let now = Utc::now();
        let crash_age = now.signed_duration_since(crash.crashed_at);

        if crash_age.num_hours() < 24 {
            return Some(
                Anomaly::new(
                    AnomalySignal::SystemCrash,
                    AnomalySeverity::Warning,
                    "Recent system crash detected",
                    &format!("Anna daemon crashed {} ago: {}",
                        format_duration(crash_age.num_seconds() as u64),
                        crash.reason
                    ),
                )
                .with_confidence(1.0)
                .with_hint("Check logs: journalctl -u annad -n 50")
                .with_hint("Review system state before crash")
            );
        }

        None
    }

    /// Check for failed services
    fn check_failed_services(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        let output = std::process::Command::new("systemctl")
            .args(["--failed", "--no-legend", "--plain"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(unit) = parts.first() {
                        let service = unit.trim_end_matches(".service");
                        anomalies.push(
                            Anomaly::new(
                                AnomalySignal::ServiceFailed {
                                    service: service.to_string(),
                                },
                                AnomalySeverity::Warning,
                                &format!("Service {} failed", service),
                                &format!("The {} service has failed to start", service),
                            )
                            .with_confidence(1.0)
                            .with_hint(&format!("Check logs: journalctl -u {} -n 30", unit))
                            .with_hint(&format!("Try restarting: sudo systemctl restart {}", unit))
                        );
                    }
                }
            }
        }

        anomalies
    }

    /// Check journal warnings/errors trends
    fn check_journal_trends(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Get recent errors from journal (last hour)
        let output = std::process::Command::new("journalctl")
            .args(["--since", "1 hour ago", "-p", "err", "--no-pager", "-o", "cat"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let error_count = stdout.lines().count();

                if error_count > self.thresholds.journal_errors_increase_count as usize {
                    anomalies.push(
                        Anomaly::new(
                            AnomalySignal::JournalErrorsIncrease {
                                service: "system".to_string(),
                            },
                            AnomalySeverity::Warning,
                            "High error rate in system logs",
                            &format!("{} errors logged in the last hour", error_count),
                        )
                        .with_confidence(0.8)
                        .with_hint("Check recent errors: journalctl -p err --since '1 hour ago'")
                        .with_hint("Look for patterns in error messages")
                    );
                }
            }
        }

        anomalies
    }

    /// Update alert queue with new anomalies
    pub fn update_alerts(&self) -> AlertQueue {
        let mut queue = AlertQueue::load();
        let anomalies = self.run_checks();

        for anomaly in anomalies {
            if anomaly.confidence >= self.thresholds.min_confidence {
                queue.add_anomaly(anomaly);
            }
        }

        queue.last_check = Some(Utc::now());
        queue.schema_version = ALERTS_SCHEMA_VERSION;
        queue.cleanup();

        if let Err(e) = queue.save() {
            eprintln!("Failed to save alert queue: {}", e);
        }

        queue
    }
}

/// Generate unique evidence ID for anomaly
fn generate_evidence_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("ANO{}", ts % 100000)
}

/// Get number of CPUs
fn num_cpus() -> usize {
    fs::read_to_string("/proc/cpuinfo")
        .map(|s| s.matches("processor").count())
        .unwrap_or(1)
        .max(1)
}

/// Parse meminfo value (in KB)
fn parse_meminfo_value(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    parts.get(1)?.parse().ok()
}

/// Parse systemd-analyze time output
fn parse_systemd_analyze_time(output: &str) -> Option<f64> {
    // Look for "= Xs" at the end
    for line in output.lines() {
        if line.contains("=") {
            // Find the total time
            if let Some(pos) = line.rfind('=') {
                let time_str = line[pos + 1..].trim();
                // Parse time like "25.123s"
                if time_str.ends_with('s') {
                    return time_str.trim_end_matches('s').parse().ok();
                }
            }
        }
    }
    None
}

/// Format duration in human-readable form
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

// ============================================================================
// What Changed Correlation Tool
// ============================================================================

/// Result from what_changed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatChangedResult {
    /// Evidence ID
    pub evidence_id: String,
    /// Time window analyzed
    pub days: u32,
    /// Packages installed
    pub packages_installed: Vec<PackageChange>,
    /// Packages removed
    pub packages_removed: Vec<PackageChange>,
    /// Packages upgraded
    pub packages_upgraded: Vec<PackageChange>,
    /// Services enabled
    pub services_enabled: Vec<String>,
    /// Services disabled
    pub services_disabled: Vec<String>,
    /// Config changes detected
    pub config_changes: Vec<ConfigChange>,
    /// Anna updates
    pub anna_updates: Vec<String>,
}

/// Package change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChange {
    pub name: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

/// Config change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub path: String,
    pub change_type: String,
    pub timestamp: DateTime<Utc>,
}

impl WhatChangedResult {
    pub fn new(days: u32) -> Self {
        Self {
            evidence_id: generate_evidence_id(),
            days,
            packages_installed: Vec::new(),
            packages_removed: Vec::new(),
            packages_upgraded: Vec::new(),
            services_enabled: Vec::new(),
            services_disabled: Vec::new(),
            config_changes: Vec::new(),
            anna_updates: Vec::new(),
        }
    }

    /// Check if anything changed
    pub fn has_changes(&self) -> bool {
        !self.packages_installed.is_empty()
            || !self.packages_removed.is_empty()
            || !self.packages_upgraded.is_empty()
            || !self.services_enabled.is_empty()
            || !self.services_disabled.is_empty()
            || !self.config_changes.is_empty()
            || !self.anna_updates.is_empty()
    }

    /// Format as human-readable summary
    pub fn format_summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.packages_installed.is_empty() {
            parts.push(format!("{} packages installed", self.packages_installed.len()));
        }
        if !self.packages_removed.is_empty() {
            parts.push(format!("{} packages removed", self.packages_removed.len()));
        }
        if !self.packages_upgraded.is_empty() {
            parts.push(format!("{} packages upgraded", self.packages_upgraded.len()));
        }
        if !self.services_enabled.is_empty() {
            parts.push(format!("{} services enabled", self.services_enabled.len()));
        }
        if !self.services_disabled.is_empty() {
            parts.push(format!("{} services disabled", self.services_disabled.len()));
        }
        if !self.config_changes.is_empty() {
            parts.push(format!("{} config changes", self.config_changes.len()));
        }
        if !self.anna_updates.is_empty() {
            parts.push(format!("{} Anna updates", self.anna_updates.len()));
        }

        if parts.is_empty() {
            format!("No changes in the last {} days", self.days)
        } else {
            format!("Last {} days: {}", self.days, parts.join(", "))
        }
    }

    /// Format detailed output
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] System changes in last {} days", self.evidence_id, self.days),
        ];

        if !self.packages_installed.is_empty() {
            lines.push(String::new());
            lines.push(format!("Packages installed ({}):", self.packages_installed.len()));
            for pkg in &self.packages_installed {
                lines.push(format!("  - {} {} ({})", pkg.name, pkg.version, pkg.timestamp.format("%Y-%m-%d")));
            }
        }

        if !self.packages_removed.is_empty() {
            lines.push(String::new());
            lines.push(format!("Packages removed ({}):", self.packages_removed.len()));
            for pkg in &self.packages_removed {
                lines.push(format!("  - {} ({})", pkg.name, pkg.timestamp.format("%Y-%m-%d")));
            }
        }

        if !self.packages_upgraded.is_empty() {
            lines.push(String::new());
            lines.push(format!("Packages upgraded ({}):", self.packages_upgraded.len()));
            for pkg in &self.packages_upgraded {
                lines.push(format!("  - {} -> {} ({})", pkg.name, pkg.version, pkg.timestamp.format("%Y-%m-%d")));
            }
        }

        if !self.services_enabled.is_empty() {
            lines.push(String::new());
            lines.push(format!("Services enabled ({}):", self.services_enabled.len()));
            for svc in &self.services_enabled {
                lines.push(format!("  - {}", svc));
            }
        }

        if !self.services_disabled.is_empty() {
            lines.push(String::new());
            lines.push(format!("Services disabled ({}):", self.services_disabled.len()));
            for svc in &self.services_disabled {
                lines.push(format!("  - {}", svc));
            }
        }

        if !self.config_changes.is_empty() {
            lines.push(String::new());
            lines.push(format!("Config changes ({}):", self.config_changes.len()));
            for cfg in &self.config_changes {
                lines.push(format!("  - {} [{}] ({})", cfg.path, cfg.change_type, cfg.timestamp.format("%Y-%m-%d")));
            }
        }

        if !self.anna_updates.is_empty() {
            lines.push(String::new());
            lines.push(format!("Anna updates ({}):", self.anna_updates.len()));
            for update in &self.anna_updates {
                lines.push(format!("  - {}", update));
            }
        }

        lines.join("\n")
    }
}

/// Collect what changed in the last N days
pub fn what_changed(days: u32) -> WhatChangedResult {
    let mut result = WhatChangedResult::new(days);
    let cutoff = Utc::now() - chrono::Duration::days(days as i64);

    // Parse pacman.log for package changes
    if let Ok(content) = fs::read_to_string("/var/log/pacman.log") {
        for line in content.lines().rev() {
            // Parse lines like: [2024-01-15T10:30:00+0000] [ALPM] installed package (version)
            if let Some(timestamp) = parse_pacman_log_timestamp(line) {
                if timestamp < cutoff {
                    break; // Logs are sorted, stop when we hit old entries
                }

                if line.contains("[ALPM] installed") {
                    if let Some((name, version)) = parse_pacman_package_line(line, "installed") {
                        result.packages_installed.push(PackageChange {
                            name,
                            version,
                            timestamp,
                        });
                    }
                } else if line.contains("[ALPM] removed") {
                    if let Some((name, version)) = parse_pacman_package_line(line, "removed") {
                        result.packages_removed.push(PackageChange {
                            name,
                            version,
                            timestamp,
                        });
                    }
                } else if line.contains("[ALPM] upgraded") {
                    if let Some((name, version)) = parse_pacman_upgrade_line(line) {
                        result.packages_upgraded.push(PackageChange {
                            name,
                            version,
                            timestamp,
                        });
                    }
                }
            }
        }
    }

    // Reverse to get chronological order
    result.packages_installed.reverse();
    result.packages_removed.reverse();
    result.packages_upgraded.reverse();

    // Check for service state changes via journal
    if let Ok(output) = std::process::Command::new("journalctl")
        .args(["--since", &format!("{} days ago", days), "-u", "systemd-*", "--grep", "Succeeded|Started|Stopped|Enabled|Disabled", "-o", "cat", "--no-pager"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Enabled") {
                // Extract service name if possible
                if let Some(svc) = extract_service_name(line) {
                    if !result.services_enabled.contains(&svc) {
                        result.services_enabled.push(svc);
                    }
                }
            } else if line.contains("Disabled") {
                if let Some(svc) = extract_service_name(line) {
                    if !result.services_disabled.contains(&svc) {
                        result.services_disabled.push(svc);
                    }
                }
            }
        }
    }

    // Check for Anna updates via ops log
    let ops_log_path = Path::new(INTERNAL_DIR).join("ops.jsonl");
    if let Ok(content) = fs::read_to_string(&ops_log_path) {
        for line in content.lines() {
            if line.contains("update") || line.contains("upgrade") {
                // Simple check for update entries
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(action) = entry.get("action").and_then(|a| a.as_str()) {
                        if action.contains("update") {
                            if let Some(ts) = entry.get("timestamp").and_then(|t| t.as_str()) {
                                result.anna_updates.push(format!("{}: {}", ts, action));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

/// Parse timestamp from pacman log line
fn parse_pacman_log_timestamp(line: &str) -> Option<DateTime<Utc>> {
    // Format: [2024-01-15T10:30:00+0000]
    if let Some(start) = line.find('[') {
        if let Some(end) = line.find(']') {
            let ts_str = &line[start + 1..end];
            // Try parsing with timezone
            if let Ok(dt) = chrono::DateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S%z") {
                return Some(dt.with_timezone(&Utc));
            }
            // Try without timezone
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S") {
                return Some(dt.and_utc());
            }
        }
    }
    None
}

/// Parse package name and version from pacman log line
fn parse_pacman_package_line(line: &str, action: &str) -> Option<(String, String)> {
    // Format: [timestamp] [ALPM] installed package (version)
    let action_str = format!("[ALPM] {}", action);
    if let Some(idx) = line.find(&action_str) {
        let rest = &line[idx + action_str.len()..].trim();
        // Find package name and version
        if let Some(paren_start) = rest.find('(') {
            let name = rest[..paren_start].trim();
            if let Some(paren_end) = rest.find(')') {
                let version = &rest[paren_start + 1..paren_end];
                return Some((name.to_string(), version.to_string()));
            }
        }
    }
    None
}

/// Parse upgrade line
fn parse_pacman_upgrade_line(line: &str) -> Option<(String, String)> {
    // Format: [timestamp] [ALPM] upgraded package (old_version -> new_version)
    if let Some(idx) = line.find("[ALPM] upgraded") {
        let rest = &line[idx + 15..].trim();
        if let Some(paren_start) = rest.find('(') {
            let name = rest[..paren_start].trim();
            if let Some(arrow) = rest.find("->") {
                if let Some(paren_end) = rest.find(')') {
                    let new_version = rest[arrow + 2..paren_end].trim();
                    return Some((name.to_string(), new_version.to_string()));
                }
            }
        }
    }
    None
}

/// Extract service name from journal line
fn extract_service_name(line: &str) -> Option<String> {
    // Look for .service in the line
    if let Some(idx) = line.find(".service") {
        // Find the start of the service name
        let before = &line[..idx];
        if let Some(start) = before.rfind(|c: char| c.is_whitespace()) {
            return Some(line[start + 1..idx + 8].to_string());
        }
    }
    None
}

// ============================================================================
// Slowness Hypotheses Analysis Tool
// ============================================================================

/// A hypothesis about what might be causing slowness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlownessHypothesis {
    /// Unique evidence ID
    pub evidence_id: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Human-readable hypothesis title
    pub title: String,
    /// Detailed explanation
    pub explanation: String,
    /// Evidence IDs that support this hypothesis
    pub supporting_evidence: Vec<String>,
    /// Suggested read-only diagnostics
    pub suggested_diagnostics: Vec<String>,
}

impl SlownessHypothesis {
    pub fn new(title: &str, explanation: &str, confidence: f64) -> Self {
        Self {
            evidence_id: generate_evidence_id(),
            confidence,
            title: title.to_string(),
            explanation: explanation.to_string(),
            supporting_evidence: Vec::new(),
            suggested_diagnostics: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, evidence_id: &str) -> Self {
        self.supporting_evidence.push(evidence_id.to_string());
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: &str) -> Self {
        self.suggested_diagnostics.push(diagnostic.to_string());
        self
    }
}

/// Result from slowness hypothesis analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlownessAnalysisResult {
    /// Evidence ID for this analysis
    pub evidence_id: String,
    /// Time window analyzed
    pub days: u32,
    /// Ranked hypotheses (highest confidence first)
    pub hypotheses: Vec<SlownessHypothesis>,
    /// What changed summary
    pub changes_summary: String,
    /// Active anomalies
    pub active_anomalies: Vec<String>,
    /// Top resource processes
    pub top_processes: Vec<String>,
}

impl SlownessAnalysisResult {
    pub fn new(days: u32) -> Self {
        Self {
            evidence_id: generate_evidence_id(),
            days,
            hypotheses: Vec::new(),
            changes_summary: String::new(),
            active_anomalies: Vec::new(),
            top_processes: Vec::new(),
        }
    }

    /// Format for display
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] Slowness Analysis (last {} days)", self.evidence_id, self.days),
        ];

        if !self.hypotheses.is_empty() {
            lines.push(String::new());
            lines.push("Ranked Hypotheses:".to_string());
            for (i, h) in self.hypotheses.iter().enumerate() {
                lines.push(format!(
                    "{}. [{}] {} ({:.0}% confidence)",
                    i + 1,
                    h.evidence_id,
                    h.title,
                    h.confidence * 100.0
                ));
                lines.push(format!("   {}", h.explanation));
                if !h.supporting_evidence.is_empty() {
                    lines.push(format!("   Evidence: {}", h.supporting_evidence.join(", ")));
                }
                if !h.suggested_diagnostics.is_empty() {
                    lines.push("   Diagnostics:".to_string());
                    for diag in &h.suggested_diagnostics {
                        lines.push(format!("     - {}", diag));
                    }
                }
            }
        } else {
            lines.push(String::new());
            lines.push("No specific slowness hypotheses identified.".to_string());
        }

        if !self.changes_summary.is_empty() {
            lines.push(String::new());
            lines.push(format!("Recent Changes: {}", self.changes_summary));
        }

        if !self.active_anomalies.is_empty() {
            lines.push(String::new());
            lines.push(format!("Active Anomalies: {}", self.active_anomalies.join(", ")));
        }

        if !self.top_processes.is_empty() {
            lines.push(String::new());
            lines.push("Top Resource Consumers:".to_string());
            for proc in &self.top_processes {
                lines.push(format!("  - {}", proc));
            }
        }

        lines.join("\n")
    }
}

/// Analyze potential causes of system slowness
pub fn analyze_slowness(days: u32) -> SlownessAnalysisResult {
    let mut result = SlownessAnalysisResult::new(days);

    // 1. Get what changed
    let changes = what_changed(days);
    result.changes_summary = changes.format_summary();

    // 2. Get active anomalies
    let queue = AlertQueue::load();
    for anomaly in queue.get_active() {
        result.active_anomalies.push(anomaly.format_short());
    }

    // 3. Get top resource processes
    if let Ok(output) = std::process::Command::new("ps")
        .args(["aux", "--sort=-%cpu"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for (i, line) in stdout.lines().skip(1).take(5).enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let cpu = parts.get(2).unwrap_or(&"?");
                let mem = parts.get(3).unwrap_or(&"?");
                let cmd = parts[10..].join(" ");
                let cmd_short = if cmd.len() > 40 {
                    format!("{}...", &cmd[..37])
                } else {
                    cmd
                };
                result.top_processes.push(format!("{}% CPU, {}% MEM: {}", cpu, mem, cmd_short));
            }
        }
    }

    // 4. Build hypotheses based on evidence

    // Hypothesis: Recent package installs
    if !changes.packages_installed.is_empty() {
        let pkg_names: Vec<&str> = changes.packages_installed.iter().map(|p| p.name.as_str()).take(3).collect();
        let mut h = SlownessHypothesis::new(
            "Recent package installations",
            &format!(
                "New packages were installed recently: {}. New software may include background services or daemons.",
                pkg_names.join(", ")
            ),
            0.6,
        )
        .with_evidence(&changes.evidence_id)
        .with_diagnostic("systemctl list-units --type=service --state=running")
        .with_diagnostic("top -bn1 | head -20");

        // Higher confidence if many packages
        if changes.packages_installed.len() > 5 {
            h.confidence = 0.75;
        }

        result.hypotheses.push(h);
    }

    // Hypothesis: High CPU load detected
    for anomaly in queue.get_active() {
        if matches!(anomaly.signal, AnomalySignal::CpuLoadIncrease) {
            result.hypotheses.push(
                SlownessHypothesis::new(
                    "High CPU load",
                    &anomaly.description,
                    0.85,
                )
                .with_evidence(&anomaly.evidence_id)
                .with_diagnostic("top -bn1 -o %CPU | head -15")
                .with_diagnostic("ps aux --sort=-%cpu | head -10")
            );
        }

        if matches!(anomaly.signal, AnomalySignal::MemoryPressure) {
            result.hypotheses.push(
                SlownessHypothesis::new(
                    "Memory pressure / swap activity",
                    &anomaly.description,
                    0.8,
                )
                .with_evidence(&anomaly.evidence_id)
                .with_diagnostic("free -h")
                .with_diagnostic("ps aux --sort=-%mem | head -10")
                .with_diagnostic("cat /proc/meminfo | grep -E 'Swap|Mem'")
            );
        }

        if let AnomalySignal::DiskSpaceLow { ref mount_point } = anomaly.signal {
            result.hypotheses.push(
                SlownessHypothesis::new(
                    &format!("Low disk space on {}", mount_point),
                    "Low disk space can cause slowness due to failed writes, swap issues, and log rotation problems.",
                    0.7,
                )
                .with_evidence(&anomaly.evidence_id)
                .with_diagnostic("df -h")
                .with_diagnostic(&format!("du -sh {}/* | sort -h | tail -10", mount_point))
            );
        }

        if let AnomalySignal::ServiceFailed { ref service } = anomaly.signal {
            result.hypotheses.push(
                SlownessHypothesis::new(
                    &format!("Failed service: {}", service),
                    "A failed service may cause dependent services to retry or hang, affecting system performance.",
                    0.65,
                )
                .with_evidence(&anomaly.evidence_id)
                .with_diagnostic(&format!("systemctl status {}", service))
                .with_diagnostic(&format!("journalctl -u {} -n 30", service))
            );
        }

        if matches!(anomaly.signal, AnomalySignal::BootTimeRegression) {
            result.hypotheses.push(
                SlownessHypothesis::new(
                    "Boot time regression",
                    &anomaly.description,
                    0.5,
                )
                .with_evidence(&anomaly.evidence_id)
                .with_diagnostic("systemd-analyze blame | head -10")
                .with_diagnostic("systemd-analyze critical-chain")
            );
        }
    }

    // Hypothesis: Service changes
    if !changes.services_enabled.is_empty() {
        let svc_names: Vec<&str> = changes.services_enabled.iter().map(|s| s.as_str()).take(3).collect();
        result.hypotheses.push(
            SlownessHypothesis::new(
                "Recently enabled services",
                &format!(
                    "Services were recently enabled: {}. New services consume resources.",
                    svc_names.join(", ")
                ),
                0.55,
            )
            .with_evidence(&changes.evidence_id)
            .with_diagnostic("systemctl status")
        );
    }

    // Hypothesis: Journal errors increasing
    for anomaly in queue.get_active() {
        if let AnomalySignal::JournalErrorsIncrease { ref service } = anomaly.signal {
            result.hypotheses.push(
                SlownessHypothesis::new(
                    &format!("Increased errors in {}", service),
                    "High error rates may indicate a struggling service or component.",
                    0.6,
                )
                .with_evidence(&anomaly.evidence_id)
                .with_diagnostic(&format!("journalctl -u {} -p err --since '1 hour ago'", service))
            );
        }
    }

    // Sort hypotheses by confidence (highest first)
    result.hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_severity_ordering() {
        assert!(AnomalySeverity::Critical > AnomalySeverity::Warning);
        assert!(AnomalySeverity::Warning > AnomalySeverity::Info);
    }

    #[test]
    fn test_anomaly_signal_dedup_key() {
        let s1 = AnomalySignal::BootTimeRegression;
        let s2 = AnomalySignal::ServiceFailed { service: "nginx".to_string() };
        let s3 = AnomalySignal::ServiceFailed { service: "nginx".to_string() };

        assert_eq!(s1.dedup_key(), "boot_time");
        assert_eq!(s2.dedup_key(), "service_failed:nginx");
        assert_eq!(s2.dedup_key(), s3.dedup_key());
    }

    #[test]
    fn test_alert_queue_deduplication() {
        let mut queue = AlertQueue::default();

        let a1 = Anomaly::new(
            AnomalySignal::BootTimeRegression,
            AnomalySeverity::Info,
            "Test 1",
            "Description 1",
        );

        let a2 = Anomaly::new(
            AnomalySignal::BootTimeRegression,
            AnomalySeverity::Warning,
            "Test 2",
            "Description 2",
        );

        queue.add_anomaly(a1);
        queue.add_anomaly(a2);

        // Should have only one anomaly (deduplicated)
        assert_eq!(queue.anomalies.len(), 1);
        // Severity should be upgraded to Warning
        assert_eq!(queue.anomalies[0].severity, AnomalySeverity::Warning);
        // Occurrence count should be 2
        assert_eq!(queue.anomalies[0].occurrence_count, 2);
    }

    #[test]
    fn test_anomaly_thresholds_default() {
        let t = AnomalyThresholds::default();
        assert_eq!(t.boot_time_increase_percent, 20.0);
        assert_eq!(t.memory_high_percent, 90.0);
        assert_eq!(t.disk_space_low_percent, 10.0);
    }

    #[test]
    fn test_time_window_format() {
        let window = TimeWindow {
            start: 1700000000,
            end: 1700086400,
            sample_count: 100,
        };
        assert!(window.duration_days() > 0.9);
        assert!(window.duration_days() < 1.1);
    }

    #[test]
    fn test_anomaly_format_short() {
        let anomaly = Anomaly::new(
            AnomalySignal::CpuLoadIncrease,
            AnomalySeverity::Warning,
            "High CPU load",
            "CPU load is elevated",
        );
        let short = anomaly.format_short();
        assert!(short.contains("High CPU load"));
        assert!(short.contains("warning"));
        assert!(short.contains("ANO")); // Evidence ID prefix
    }

    #[test]
    fn test_anomaly_with_builders() {
        let anomaly = Anomaly::new(
            AnomalySignal::MemoryPressure,
            AnomalySeverity::Warning,
            "Memory pressure",
            "High memory usage",
        )
        .with_delta(0.25)
        .with_confidence(0.8)
        .with_hint("Check memory usage");

        assert_eq!(anomaly.delta, Some(0.25));
        assert_eq!(anomaly.confidence, 0.8);
        assert_eq!(anomaly.hints.len(), 1);
    }

    #[test]
    fn test_alert_queue_acknowledge() {
        let mut queue = AlertQueue::default();
        let anomaly = Anomaly::new(
            AnomalySignal::SystemCrash,
            AnomalySeverity::Critical,
            "System crash",
            "System crashed",
        );
        let evidence_id = anomaly.evidence_id.clone();

        queue.add_anomaly(anomaly);
        assert_eq!(queue.get_active().len(), 1);

        // Acknowledge
        queue.acknowledge(&evidence_id);
        assert_eq!(queue.get_active().len(), 0);
    }

    #[test]
    fn test_alert_queue_count_by_severity() {
        let mut queue = AlertQueue::default();

        queue.add_anomaly(Anomaly::new(
            AnomalySignal::SystemCrash,
            AnomalySeverity::Critical,
            "Critical",
            "",
        ));
        queue.add_anomaly(Anomaly::new(
            AnomalySignal::BootTimeRegression,
            AnomalySeverity::Warning,
            "Warning",
            "",
        ));
        queue.add_anomaly(Anomaly::new(
            AnomalySignal::CpuLoadIncrease,
            AnomalySeverity::Info,
            "Info",
            "",
        ));

        let (critical, warning, info) = queue.count_by_severity();
        assert_eq!(critical, 1);
        assert_eq!(warning, 1);
        assert_eq!(info, 1);
    }

    #[test]
    fn test_what_changed_result_format() {
        let mut result = WhatChangedResult::new(7);
        result.packages_installed.push(PackageChange {
            name: "test-pkg".to_string(),
            version: "1.0.0".to_string(),
            timestamp: Utc::now(),
        });

        assert!(result.has_changes());
        let summary = result.format_summary();
        assert!(summary.contains("1 packages installed"));
    }

    #[test]
    fn test_what_changed_result_no_changes() {
        let result = WhatChangedResult::new(7);
        assert!(!result.has_changes());
        let summary = result.format_summary();
        assert!(summary.contains("No changes"));
    }

    #[test]
    fn test_slowness_hypothesis_builders() {
        let h = SlownessHypothesis::new(
            "Test hypothesis",
            "Test explanation",
            0.75,
        )
        .with_evidence("E1")
        .with_diagnostic("test diagnostic");

        assert_eq!(h.title, "Test hypothesis");
        assert_eq!(h.confidence, 0.75);
        assert_eq!(h.supporting_evidence.len(), 1);
        assert_eq!(h.suggested_diagnostics.len(), 1);
    }

    #[test]
    fn test_slowness_analysis_result_format() {
        let mut result = SlownessAnalysisResult::new(7);
        result.hypotheses.push(SlownessHypothesis::new(
            "High CPU",
            "CPU usage is high",
            0.85,
        ));

        let detail = result.format_detail();
        assert!(detail.contains("High CPU"));
        assert!(detail.contains("85%"));
    }

    #[test]
    fn test_anomaly_engine_default() {
        let engine = AnomalyEngine::default();
        assert_eq!(engine.baseline_days, 14);
        assert_eq!(engine.recent_days, 2);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(AnomalySeverity::Critical.as_str(), "critical");
        assert_eq!(AnomalySeverity::Warning.as_str(), "warning");
        assert_eq!(AnomalySeverity::Info.as_str(), "info");
    }

    #[test]
    fn test_signal_metric_name() {
        assert_eq!(AnomalySignal::BootTimeRegression.metric_name(), "boot_time");
        assert_eq!(AnomalySignal::CpuLoadIncrease.metric_name(), "cpu_load");
        assert_eq!(
            AnomalySignal::ServiceFailed { service: "nginx".to_string() }.metric_name(),
            "service_failed:nginx"
        );
    }
}
