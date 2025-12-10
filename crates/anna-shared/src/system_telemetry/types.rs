//! Telemetry data types (v0.0.291).
//!
//! Data structures for system telemetry tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
