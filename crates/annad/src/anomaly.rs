//! Anomaly detection - learns baselines and alerts on deviations.
//!
//! Tracks system metrics over time and detects when current values
//! deviate significantly from learned patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

use crate::telegram::notifier::push_notification;

/// A metric sample with timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub timestamp: i64,
    pub value: f64,
}

/// Baseline statistics for a metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: u32,
}

impl Baseline {
    /// Check if a value is anomalous (>2 std deviations from mean).
    pub fn is_anomaly(&self, value: f64) -> bool {
        if self.sample_count < 10 {
            return false; // Not enough data
        }
        let threshold = 2.0 * self.std_dev;
        (value - self.mean).abs() > threshold
    }

    /// Get severity: how many std deviations from mean.
    pub fn severity(&self, value: f64) -> f64 {
        if self.std_dev < 0.001 {
            return 0.0;
        }
        (value - self.mean).abs() / self.std_dev
    }
}

/// Metric history and baseline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricHistory {
    pub name: String,
    pub unit: String,
    pub samples: Vec<Sample>,
    pub baseline: Option<Baseline>,
    pub last_alert: Option<i64>,
}

impl MetricHistory {
    pub fn new(name: &str, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            unit: unit.to_string(),
            samples: Vec::new(),
            baseline: None,
            last_alert: None,
        }
    }

    /// Add a sample and update baseline.
    pub fn record(&mut self, value: f64) {
        let now = chrono::Utc::now().timestamp();
        self.samples.push(Sample {
            timestamp: now,
            value,
        });

        // Keep last 24 hours of samples (assuming ~1 sample/minute = 1440)
        let cutoff = now - 86400;
        self.samples.retain(|s| s.timestamp > cutoff);

        // Update baseline if we have enough samples
        if self.samples.len() >= 10 {
            self.update_baseline();
        }
    }

    fn update_baseline(&mut self) {
        let values: Vec<f64> = self.samples.iter().map(|s| s.value).collect();
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        self.baseline = Some(Baseline {
            mean,
            std_dev,
            min,
            max,
            sample_count: values.len() as u32,
        });
    }

    /// Check current value for anomaly. Returns alert message if anomalous.
    pub fn check(&mut self, value: f64) -> Option<String> {
        let now = chrono::Utc::now().timestamp();

        // Don't alert more than once per 10 minutes
        if let Some(last) = self.last_alert {
            if now - last < 600 {
                return None;
            }
        }

        if let Some(ref baseline) = self.baseline {
            if baseline.is_anomaly(value) {
                let severity = baseline.severity(value);
                let direction = if value > baseline.mean { "HIGH" } else { "LOW" };
                self.last_alert = Some(now);

                return Some(format!(
                    "[ANOMALY] {} is {}: {:.1}{} (normal: {:.1} +/- {:.1}, {:.1} std devs)",
                    self.name,
                    direction,
                    value,
                    self.unit,
                    baseline.mean,
                    baseline.std_dev,
                    severity
                ));
            }
        }
        None
    }
}

/// Store for all tracked metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalyStore {
    pub metrics: HashMap<String, MetricHistory>,
}

impl AnomalyStore {
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/anomaly_baselines.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Self::path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(self)?;
        std::fs::write(Self::path(), json)
    }

    /// Record a metric value.
    pub fn record(&mut self, name: &str, unit: &str, value: f64) {
        let history = self.metrics
            .entry(name.to_string())
            .or_insert_with(|| MetricHistory::new(name, unit));
        history.record(value);
    }

    /// Check all metrics for anomalies. Returns list of alerts.
    pub fn check_all(&mut self) -> Vec<String> {
        let mut alerts = Vec::new();

        // Collect current values
        let current = collect_metrics();

        for (name, value, unit) in current {
            self.record(&name, &unit, value);

            if let Some(history) = self.metrics.get_mut(&name) {
                if let Some(alert) = history.check(value) {
                    alerts.push(alert);
                }
            }
        }

        alerts
    }
}

/// Collect current system metrics.
fn collect_metrics() -> Vec<(String, f64, String)> {
    let mut metrics = Vec::new();

    // RAM usage percentage
    if let Ok(output) = Command::new("free").arg("-m").output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(total), Ok(used)) = (
                    parts[1].parse::<f64>(),
                    parts[2].parse::<f64>(),
                ) {
                    let pct = (used / total) * 100.0;
                    metrics.push(("RAM".to_string(), pct, "%".to_string()));
                }
            }
        }
    }

    // CPU load (1 min average)
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(first) = load.split_whitespace().next() {
            if let Ok(load1) = first.parse::<f64>() {
                metrics.push(("Load1".to_string(), load1, "".to_string()));
            }
        }
    }

    // Disk usage percentage
    if let Ok(output) = Command::new("df").args(["--output=pcent", "/"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            if let Ok(pct) = line.trim().trim_end_matches('%').parse::<f64>() {
                metrics.push(("Disk".to_string(), pct, "%".to_string()));
            }
        }
    }

    // Network RX bytes (delta would be better but this is a start)
    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let iface = parts[0].trim_end_matches(':');
                if iface != "lo" {
                    if let Ok(rx_bytes) = parts[1].parse::<f64>() {
                        let rx_mb = rx_bytes / 1_000_000.0;
                        metrics.push((format!("Net_{}_RX", iface), rx_mb, "MB".to_string()));
                    }
                }
            }
        }
    }

    // Swap usage
    if let Ok(output) = Command::new("free").arg("-m").output() {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if line.starts_with("Swap:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Ok(used) = parts[2].parse::<f64>() {
                        metrics.push(("Swap".to_string(), used, "MB".to_string()));
                    }
                }
            }
        }
    }

    metrics
}

/// Run anomaly detection and send alerts.
pub fn run_anomaly_check() {
    let mut store = AnomalyStore::load();
    let alerts = store.check_all();

    if !alerts.is_empty() {
        info!("Anomaly detection: {} alerts", alerts.len());
        for alert in &alerts {
            warn!("{}", alert);
            push_notification(alert);
        }
    } else {
        debug!("Anomaly detection: no anomalies");
    }

    if let Err(e) = store.save() {
        warn!("Failed to save anomaly store: {}", e);
    }
}
