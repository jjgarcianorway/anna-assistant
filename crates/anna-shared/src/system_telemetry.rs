//! System telemetry tracking (v0.0.280).
//!
//! Tracks system state changes over time for trend analysis.
//! Anna can use this to detect patterns and proactively alert users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

/// Maximum telemetry samples to keep in memory
const MAX_SAMPLES: usize = 1000;

/// Default telemetry file path
fn default_telemetry_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/var/lib/anna"))
        .join("anna")
        .join("telemetry.json")
}

/// A single telemetry sample capturing system state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub disk_used_bytes: Option<u64>,
    pub disk_total_bytes: Option<u64>,
    pub load_average_1m: Option<f32>,
    pub load_average_5m: Option<f32>,
    pub load_average_15m: Option<f32>,
    pub process_count: Option<u32>,
    pub uptime_secs: Option<u64>,
    /// Services that were checked and their status
    pub services: Vec<ServiceStatus>,
    /// Network interfaces and their state
    pub network: Vec<NetworkStatus>,
}

/// Status of a monitored service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub enabled: bool,
    pub memory_bytes: Option<u64>,
    pub cpu_percent: Option<f32>,
}

/// Network interface status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub interface: String,
    pub up: bool,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// Telemetry store for historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryStore {
    pub samples: VecDeque<TelemetrySample>,
    /// Last analysis timestamp
    pub last_analysis: Option<DateTime<Utc>>,
    /// Detected anomalies
    pub anomalies: Vec<TelemetryAnomaly>,
    /// Trend summaries
    pub trends: TrendSummary,
}

/// Detected anomaly in telemetry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryAnomaly {
    pub detected_at: DateTime<Utc>,
    pub category: AnomalyCategory,
    pub severity: AnomalySeverity,
    pub description: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
}

/// Category of anomaly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyCategory {
    HighCpu,
    HighMemory,
    LowDisk,
    ServiceDown,
    NetworkError,
    HighLoad,
}

impl std::fmt::Display for AnomalyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighCpu => write!(f, "High CPU"),
            Self::HighMemory => write!(f, "High Memory"),
            Self::LowDisk => write!(f, "Low Disk"),
            Self::ServiceDown => write!(f, "Service Down"),
            Self::NetworkError => write!(f, "Network Error"),
            Self::HighLoad => write!(f, "High Load"),
        }
    }
}

/// Severity of anomaly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
}

/// Summary of trends over time
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrendSummary {
    /// CPU trend: positive = increasing, negative = decreasing
    pub cpu_trend: f32,
    /// Memory trend
    pub memory_trend: f32,
    /// Disk usage trend
    pub disk_trend: f32,
    /// Load average trend
    pub load_trend: f32,
    /// Number of samples used for trend calculation
    pub sample_count: usize,
    /// Time window in hours
    pub window_hours: f32,
}

impl Default for TelemetryStore {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(MAX_SAMPLES),
            last_analysis: None,
            anomalies: Vec::new(),
            trends: TrendSummary::default(),
        }
    }
}

impl TelemetryStore {
    /// Load telemetry from disk
    pub fn load() -> Self {
        Self::load_from_path(&default_telemetry_path())
    }

    /// Load from disk only if file exists, returns None if no data
    pub fn load_if_exists() -> Option<Self> {
        let path = default_telemetry_path();
        if path.exists() {
            let store = Self::load_from_path(&path);
            if !store.samples.is_empty() {
                return Some(store);
            }
        }
        None
    }

    /// Load from specific path
    pub fn load_from_path(path: &PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save telemetry to disk
    pub fn save(&self) -> std::io::Result<()> {
        self.save_to_path(&default_telemetry_path())
    }

    /// Save to specific path
    pub fn save_to_path(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Add a new sample
    pub fn add_sample(&mut self, sample: TelemetrySample) {
        if self.samples.len() >= MAX_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Analyze telemetry and detect anomalies
    pub fn analyze(&mut self) {
        self.anomalies.clear();
        self.analyze_cpu();
        self.analyze_memory();
        self.analyze_disk();
        self.analyze_load();
        self.analyze_services();
        self.calculate_trends();
        self.last_analysis = Some(Utc::now());
    }

    fn analyze_cpu(&mut self) {
        if let Some(sample) = self.samples.back() {
            if let Some(cpu) = sample.cpu_usage_percent {
                if cpu > 90.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::HighCpu,
                        severity: AnomalySeverity::Critical,
                        description: format!("CPU usage at {:.1}%", cpu),
                        metric: "cpu_usage_percent".to_string(),
                        value: cpu as f64,
                        threshold: 90.0,
                    });
                } else if cpu > 75.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::HighCpu,
                        severity: AnomalySeverity::Warning,
                        description: format!("CPU usage elevated at {:.1}%", cpu),
                        metric: "cpu_usage_percent".to_string(),
                        value: cpu as f64,
                        threshold: 75.0,
                    });
                }
            }
        }
    }

    fn analyze_memory(&mut self) {
        if let Some(sample) = self.samples.back() {
            if let (Some(used), Some(total)) = (sample.memory_used_bytes, sample.memory_total_bytes)
            {
                let percent = (used as f64 / total as f64) * 100.0;
                if percent > 90.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::HighMemory,
                        severity: AnomalySeverity::Critical,
                        description: format!("Memory usage at {:.1}%", percent),
                        metric: "memory_percent".to_string(),
                        value: percent,
                        threshold: 90.0,
                    });
                } else if percent > 80.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::HighMemory,
                        severity: AnomalySeverity::Warning,
                        description: format!("Memory usage elevated at {:.1}%", percent),
                        metric: "memory_percent".to_string(),
                        value: percent,
                        threshold: 80.0,
                    });
                }
            }
        }
    }

    fn analyze_disk(&mut self) {
        if let Some(sample) = self.samples.back() {
            if let (Some(used), Some(total)) = (sample.disk_used_bytes, sample.disk_total_bytes) {
                let percent = (used as f64 / total as f64) * 100.0;
                if percent > 95.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::LowDisk,
                        severity: AnomalySeverity::Critical,
                        description: format!("Disk usage at {:.1}% - critical!", percent),
                        metric: "disk_percent".to_string(),
                        value: percent,
                        threshold: 95.0,
                    });
                } else if percent > 85.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::LowDisk,
                        severity: AnomalySeverity::Warning,
                        description: format!("Disk usage elevated at {:.1}%", percent),
                        metric: "disk_percent".to_string(),
                        value: percent,
                        threshold: 85.0,
                    });
                }
            }
        }
    }

    fn analyze_load(&mut self) {
        if let Some(sample) = self.samples.back() {
            if let Some(load) = sample.load_average_1m {
                // High load is typically > number of CPU cores
                if load > 4.0 {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::HighLoad,
                        severity: AnomalySeverity::Warning,
                        description: format!("Load average at {:.2}", load),
                        metric: "load_average_1m".to_string(),
                        value: load as f64,
                        threshold: 4.0,
                    });
                }
            }
        }
    }

    fn analyze_services(&mut self) {
        if let Some(sample) = self.samples.back() {
            for service in &sample.services {
                if !service.running && service.enabled {
                    self.anomalies.push(TelemetryAnomaly {
                        detected_at: Utc::now(),
                        category: AnomalyCategory::ServiceDown,
                        severity: AnomalySeverity::Critical,
                        description: format!("Service '{}' is down but enabled", service.name),
                        metric: format!("service_{}", service.name),
                        value: 0.0,
                        threshold: 1.0,
                    });
                }
            }
        }
    }

    fn calculate_trends(&mut self) {
        let sample_count = self.samples.len();
        if sample_count < 2 {
            return;
        }

        // Calculate time window
        if let (Some(first), Some(last)) = (self.samples.front(), self.samples.back()) {
            let duration = last.timestamp.signed_duration_since(first.timestamp);
            self.trends.window_hours = duration.num_minutes() as f32 / 60.0;
        }

        self.trends.sample_count = sample_count;

        // Simple linear trend calculation
        self.trends.cpu_trend = self.calculate_metric_trend(|s| s.cpu_usage_percent);
        self.trends.memory_trend = self.calculate_metric_trend(|s| {
            match (s.memory_used_bytes, s.memory_total_bytes) {
                (Some(used), Some(total)) if total > 0 => {
                    Some((used as f32 / total as f32) * 100.0)
                }
                _ => None,
            }
        });
        self.trends.disk_trend = self.calculate_metric_trend(|s| {
            match (s.disk_used_bytes, s.disk_total_bytes) {
                (Some(used), Some(total)) if total > 0 => {
                    Some((used as f32 / total as f32) * 100.0)
                }
                _ => None,
            }
        });
        self.trends.load_trend = self.calculate_metric_trend(|s| s.load_average_1m);
    }

    fn calculate_metric_trend<F>(&self, extractor: F) -> f32
    where
        F: Fn(&TelemetrySample) -> Option<f32>,
    {
        let values: Vec<f32> = self.samples.iter().filter_map(|s| extractor(s)).collect();
        if values.len() < 2 {
            return 0.0;
        }

        // Compare first quarter average to last quarter average
        let quarter = values.len() / 4;
        if quarter == 0 {
            return 0.0;
        }

        let first_avg: f32 = values[..quarter].iter().sum::<f32>() / quarter as f32;
        let last_avg: f32 = values[values.len() - quarter..].iter().sum::<f32>() / quarter as f32;

        last_avg - first_avg
    }

    /// Get recent anomalies (last 24 hours)
    pub fn recent_anomalies(&self) -> Vec<&TelemetryAnomaly> {
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        self.anomalies
            .iter()
            .filter(|a| a.detected_at > cutoff)
            .collect()
    }

    /// Get health score (0-100)
    pub fn health_score(&self) -> u8 {
        let mut score: i32 = 100;

        for anomaly in &self.anomalies {
            match anomaly.severity {
                AnomalySeverity::Critical => score -= 25,
                AnomalySeverity::Warning => score -= 10,
                AnomalySeverity::Info => score -= 2,
            }
        }

        score.max(0).min(100) as u8
    }

    /// Generate insights from telemetry data
    pub fn generate_insights(&self) -> Vec<TelemetryInsight> {
        let mut insights = Vec::new();

        // Trend-based insights
        if self.trends.cpu_trend > 10.0 {
            insights.push(TelemetryInsight {
                category: InsightCategory::Trend,
                message: format!(
                    "CPU usage trending up by {:.1}% over the last {:.1} hours",
                    self.trends.cpu_trend, self.trends.window_hours
                ),
                recommendation: "Consider investigating high CPU processes".to_string(),
            });
        }

        if self.trends.memory_trend > 10.0 {
            insights.push(TelemetryInsight {
                category: InsightCategory::Trend,
                message: format!(
                    "Memory usage trending up by {:.1}% over the last {:.1} hours",
                    self.trends.memory_trend, self.trends.window_hours
                ),
                recommendation: "Check for memory leaks or excessive caching".to_string(),
            });
        }

        if self.trends.disk_trend > 5.0 {
            insights.push(TelemetryInsight {
                category: InsightCategory::Trend,
                message: format!(
                    "Disk usage growing by {:.1}% over the last {:.1} hours",
                    self.trends.disk_trend, self.trends.window_hours
                ),
                recommendation: "Review log rotation and cleanup old files".to_string(),
            });
        }

        // Anomaly-based insights
        let critical_count = self
            .anomalies
            .iter()
            .filter(|a| a.severity == AnomalySeverity::Critical)
            .count();
        if critical_count > 0 {
            insights.push(TelemetryInsight {
                category: InsightCategory::Alert,
                message: format!("{} critical issues detected", critical_count),
                recommendation: "Immediate attention required".to_string(),
            });
        }

        insights
    }
}

/// An insight generated from telemetry analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryInsight {
    pub category: InsightCategory,
    pub message: String,
    pub recommendation: String,
}

/// Category of insight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightCategory {
    Trend,
    Alert,
    Optimization,
    Learning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_store_default() {
        let store = TelemetryStore::default();
        assert!(store.samples.is_empty());
        assert!(store.anomalies.is_empty());
    }

    #[test]
    fn test_add_sample() {
        let mut store = TelemetryStore::default();
        let sample = TelemetrySample {
            timestamp: Utc::now(),
            cpu_usage_percent: Some(50.0),
            memory_used_bytes: Some(4_000_000_000),
            memory_total_bytes: Some(8_000_000_000),
            disk_used_bytes: None,
            disk_total_bytes: None,
            load_average_1m: Some(1.5),
            load_average_5m: None,
            load_average_15m: None,
            process_count: Some(150),
            uptime_secs: Some(3600),
            services: vec![],
            network: vec![],
        };
        store.add_sample(sample);
        assert_eq!(store.samples.len(), 1);
    }

    #[test]
    fn test_high_cpu_anomaly() {
        let mut store = TelemetryStore::default();
        let sample = TelemetrySample {
            timestamp: Utc::now(),
            cpu_usage_percent: Some(95.0),
            memory_used_bytes: None,
            memory_total_bytes: None,
            disk_used_bytes: None,
            disk_total_bytes: None,
            load_average_1m: None,
            load_average_5m: None,
            load_average_15m: None,
            process_count: None,
            uptime_secs: None,
            services: vec![],
            network: vec![],
        };
        store.add_sample(sample);
        store.analyze();
        assert!(!store.anomalies.is_empty());
        assert_eq!(store.anomalies[0].category, AnomalyCategory::HighCpu);
    }

    #[test]
    fn test_health_score() {
        let mut store = TelemetryStore::default();
        assert_eq!(store.health_score(), 100);

        store.anomalies.push(TelemetryAnomaly {
            detected_at: Utc::now(),
            category: AnomalyCategory::HighCpu,
            severity: AnomalySeverity::Warning,
            description: "Test".to_string(),
            metric: "test".to_string(),
            value: 80.0,
            threshold: 75.0,
        });
        assert_eq!(store.health_score(), 90);
    }
}
