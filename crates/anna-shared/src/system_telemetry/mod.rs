//! System telemetry tracking (v0.0.291).
//!
//! Tracks system state changes over time for trend analysis.
//! Anna can use this to detect patterns and proactively alert users.
//!
//! v0.0.291: Refactored into modules for maintainability.

mod analysis;
mod types;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

// Re-export types for public API
pub use analysis::{calculate_health_score, generate_insights};
pub use types::{
    AnomalyCategory, AnomalySeverity, InsightCategory, NetworkStatus, ServiceStatus,
    TelemetryAnomaly, TelemetryInsight, TelemetrySample, TrendSummary,
};

/// Maximum telemetry samples to keep in memory
const MAX_SAMPLES: usize = 1000;

/// Default telemetry file path
fn default_telemetry_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/var/lib/anna"))
        .join("anna")
        .join("telemetry.json")
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
    /// v0.0.291: Silently falls back to defaults on parse failure
    pub fn load_from_path(path: &PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            // Silent fallback to defaults on parse failure
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
        self.anomalies.extend(analysis::analyze_cpu(&self.samples));
        self.anomalies
            .extend(analysis::analyze_memory(&self.samples));
        self.anomalies.extend(analysis::analyze_disk(&self.samples));
        self.anomalies.extend(analysis::analyze_load(&self.samples));
        self.anomalies
            .extend(analysis::analyze_services(&self.samples));
        self.trends = analysis::calculate_trends(&self.samples);
        self.last_analysis = Some(Utc::now());
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
        calculate_health_score(&self.anomalies)
    }

    /// Generate insights from telemetry data
    pub fn generate_insights(&self) -> Vec<TelemetryInsight> {
        generate_insights(&self.trends, &self.anomalies)
    }
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
