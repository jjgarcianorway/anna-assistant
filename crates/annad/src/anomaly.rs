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

/// An optimization suggestion.
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub description: String,
    pub potential_savings: Option<String>,
    pub action: String,
}

/// Check for system optimization opportunities.
pub fn check_optimizations() -> Vec<OptimizationSuggestion> {
    let mut suggestions = Vec::new();

    // 1. Package cache size
    if let Ok(output) = Command::new("du").args(["-sm", "/var/cache/pacman/pkg"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(size) = out.split_whitespace().next() {
            if let Ok(mb) = size.parse::<u64>() {
                if mb > 1000 {
                    suggestions.push(OptimizationSuggestion {
                        category: "Disk".to_string(),
                        description: format!("Package cache is {}MB", mb),
                        potential_savings: Some(format!("~{}MB", mb / 2)),
                        action: "clean up".to_string(),
                    });
                }
            }
        }
    }

    // 2. Journal size
    if let Ok(output) = Command::new("journalctl").args(["--disk-usage"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        // Parse "Archived and active journals take up 512.0M"
        if let Some(pos) = out.find("take up") {
            let rest = &out[pos + 8..];
            if let Some(end) = rest.find(|c: char| !c.is_alphanumeric() && c != '.') {
                let size_str = &rest[..end];
                if size_str.ends_with('M') || size_str.ends_with('G') {
                    let is_gb = size_str.ends_with('G');
                    if let Ok(size) = size_str[..size_str.len()-1].parse::<f64>() {
                        let mb = if is_gb { size * 1024.0 } else { size };
                        if mb > 500.0 {
                            suggestions.push(OptimizationSuggestion {
                                category: "Disk".to_string(),
                                description: format!("Journal logs are {}MB", mb as u64),
                                potential_savings: Some(format!("~{}MB", (mb * 0.7) as u64)),
                                action: "clean up".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // 3. Orphan packages
    if let Ok(output) = Command::new("pacman").args(["-Qtdq"]).output() {
        let orphans = String::from_utf8_lossy(&output.stdout);
        let count = orphans.lines().filter(|l| !l.is_empty()).count();
        if count > 0 {
            suggestions.push(OptimizationSuggestion {
                category: "Packages".to_string(),
                description: format!("{} orphan packages found", count),
                potential_savings: None,
                action: "review and remove".to_string(),
            });
        }
    }

    // 4. Failed systemd services
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--no-pager"])
        .output()
    {
        let out = String::from_utf8_lossy(&output.stdout);
        let count = out.lines().filter(|l| !l.is_empty()).count();
        if count > 0 {
            suggestions.push(OptimizationSuggestion {
                category: "Services".to_string(),
                description: format!("{} failed services", count),
                potential_savings: None,
                action: "investigate".to_string(),
            });
        }
    }

    // 5. Disk space low
    if let Ok(output) = Command::new("df").args(["--output=avail", "-BG", "/"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            if let Ok(gb) = line.trim().trim_end_matches('G').parse::<u64>() {
                if gb < 10 {
                    suggestions.push(OptimizationSuggestion {
                        category: "Disk".to_string(),
                        description: format!("Only {}GB free on root", gb),
                        potential_savings: None,
                        action: "free up space".to_string(),
                    });
                }
            }
        }
    }

    // 6. High swap usage (might indicate memory pressure)
    if let Ok(output) = Command::new("free").arg("-m").output() {
        let out = String::from_utf8_lossy(&output.stdout);
        for line in out.lines() {
            if line.starts_with("Swap:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (
                        parts[1].parse::<u64>(),
                        parts[2].parse::<u64>(),
                    ) {
                        if total > 0 && used > total / 2 {
                            suggestions.push(OptimizationSuggestion {
                                category: "Memory".to_string(),
                                description: format!("Swap usage: {}MB / {}MB", used, total),
                                potential_savings: None,
                                action: "check memory-hungry processes".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    suggestions
}

/// Run optimization check and push a summary if suggestions exist.
pub fn run_optimization_check() {
    let suggestions = check_optimizations();

    if !suggestions.is_empty() {
        info!("Optimization check: {} suggestions", suggestions.len());

        let mut lines = vec!["Optimization suggestions:".to_string()];
        for s in &suggestions {
            let savings = s.potential_savings.as_deref().unwrap_or("");
            if savings.is_empty() {
                lines.push(format!("- {}: {} ({})", s.category, s.description, s.action));
            } else {
                lines.push(format!("- {}: {} - save {} ({})", s.category, s.description, savings, s.action));
            }
        }

        let msg = lines.join("\n");
        push_notification(&msg);
    } else {
        debug!("Optimization check: no suggestions");
    }
}
