// v0.0.584: Settings Metrics (Phase 160)
// Metrics collection and reporting for settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricKind {
    /// Counter (monotonically increasing)
    Counter,
    /// Gauge (can go up or down)
    Gauge,
    /// Histogram (distribution of values)
    Histogram,
    /// Timer (duration measurements)
    Timer,
}

impl std::fmt::Display for MetricKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Counter => write!(f, "Counter"),
            Self::Gauge => write!(f, "Gauge"),
            Self::Histogram => write!(f, "Histogram"),
            Self::Timer => write!(f, "Timer"),
        }
    }
}

/// Metric unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricUnit {
    /// Count
    Count,
    /// Bytes
    Bytes,
    /// Milliseconds
    Milliseconds,
    /// Percent
    Percent,
    /// Custom/none
    None,
}

impl std::fmt::Display for MetricUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count => write!(f, "count"),
            Self::Bytes => write!(f, "bytes"),
            Self::Milliseconds => write!(f, "ms"),
            Self::Percent => write!(f, "%"),
            Self::None => write!(f, ""),
        }
    }
}

/// Single metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Value
    pub value: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MetricValue {
    /// Create new value
    pub fn new(value: f64) -> Self {
        Self {
            value,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Name
    pub name: String,
    /// Kind
    pub kind: MetricKind,
    /// Unit
    pub unit: MetricUnit,
    /// Description
    pub description: String,
    /// Category (optional)
    pub category: Option<SettingsCategory>,
    /// Current value
    pub current: f64,
    /// Min value (for gauges/histograms)
    pub min: Option<f64>,
    /// Max value (for gauges/histograms)
    pub max: Option<f64>,
    /// Sample count (for histograms)
    pub samples: u64,
    /// Last updated
    pub updated: chrono::DateTime<chrono::Utc>,
}

impl Metric {
    /// Create new counter
    pub fn counter(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: MetricKind::Counter,
            unit: MetricUnit::Count,
            description: description.into(),
            category: None,
            current: 0.0,
            min: None,
            max: None,
            samples: 0,
            updated: chrono::Utc::now(),
        }
    }

    /// Create new gauge
    pub fn gauge(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: MetricKind::Gauge,
            unit: MetricUnit::None,
            description: description.into(),
            category: None,
            current: 0.0,
            min: None,
            max: None,
            samples: 0,
            updated: chrono::Utc::now(),
        }
    }

    /// Create new timer
    pub fn timer(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: MetricKind::Timer,
            unit: MetricUnit::Milliseconds,
            description: description.into(),
            category: None,
            current: 0.0,
            min: None,
            max: None,
            samples: 0,
            updated: chrono::Utc::now(),
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set unit
    pub fn unit(mut self, unit: MetricUnit) -> Self {
        self.unit = unit;
        self
    }

    /// Increment counter
    pub fn increment(&mut self) {
        self.increment_by(1.0);
    }

    /// Increment by value
    pub fn increment_by(&mut self, value: f64) {
        self.current += value;
        self.samples += 1;
        self.updated = chrono::Utc::now();
    }

    /// Set gauge value
    pub fn set(&mut self, value: f64) {
        self.current = value;
        self.min = Some(self.min.map_or(value, |m| m.min(value)));
        self.max = Some(self.max.map_or(value, |m| m.max(value)));
        self.samples += 1;
        self.updated = chrono::Utc::now();
    }

    /// Record timer value
    pub fn record(&mut self, duration_ms: f64) {
        self.set(duration_ms);
    }

    /// Get average (for timers/histograms)
    pub fn average(&self) -> Option<f64> {
        if self.samples > 0 {
            Some(self.current / self.samples as f64)
        } else {
            None
        }
    }
}

/// Metrics collection
#[derive(Debug, Clone, Default)]
pub struct SettingsMetrics {
    /// All metrics
    metrics: HashMap<String, Metric>,
    /// Collection start time
    started: Option<chrono::DateTime<chrono::Utc>>,
}

impl SettingsMetrics {
    /// Create new metrics collection
    pub fn new() -> Self {
        let mut sm = Self {
            started: Some(chrono::Utc::now()),
            ..Default::default()
        };
        sm.init_default_metrics();
        sm
    }

    fn init_default_metrics(&mut self) {
        // Core counters
        self.register(Metric::counter("settings.changes", "Total setting changes"));
        self.register(Metric::counter("settings.reads", "Total setting reads"));
        self.register(Metric::counter("settings.exports", "Total exports"));
        self.register(Metric::counter("settings.imports", "Total imports"));
        self.register(Metric::counter("settings.resets", "Total resets"));

        // Gauges
        self.register(Metric::gauge("settings.health_score", "Current health score")
            .unit(MetricUnit::Percent));
        self.register(Metric::gauge("settings.categories_healthy", "Healthy categories count"));

        // Timers
        self.register(Metric::timer("settings.save_duration", "Settings save duration"));
        self.register(Metric::timer("settings.load_duration", "Settings load duration"));
    }

    /// Register a metric
    pub fn register(&mut self, metric: Metric) {
        self.metrics.insert(metric.name.clone(), metric);
    }

    /// Get metric by name
    pub fn get(&self, name: &str) -> Option<&Metric> {
        self.metrics.get(name)
    }

    /// Get mutable metric
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Metric> {
        self.metrics.get_mut(name)
    }

    /// Increment counter
    pub fn increment(&mut self, name: &str) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.increment();
        }
    }

    /// Set gauge value
    pub fn set(&mut self, name: &str, value: f64) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.set(value);
        }
    }

    /// Record timer
    pub fn record(&mut self, name: &str, duration_ms: f64) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.record(duration_ms);
        }
    }

    /// Get all metrics
    pub fn all(&self) -> impl Iterator<Item = &Metric> {
        self.metrics.values()
    }

    /// Get metrics by kind
    pub fn by_kind(&self, kind: MetricKind) -> Vec<&Metric> {
        self.metrics.values().filter(|m| m.kind == kind).collect()
    }

    /// Get metrics by category
    pub fn by_category(&self, category: SettingsCategory) -> Vec<&Metric> {
        self.metrics.values()
            .filter(|m| m.category == Some(category))
            .collect()
    }

    /// Get uptime
    pub fn uptime(&self) -> Option<chrono::Duration> {
        self.started.map(|s| chrono::Utc::now() - s)
    }

    /// Metric count
    pub fn count(&self) -> usize {
        self.metrics.len()
    }
}

/// Format metrics for display
pub fn format_metrics(metrics: &SettingsMetrics) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Metrics ===\n\n");
    output.push_str(&format!("Total Metrics: {}\n", metrics.count()));

    if let Some(uptime) = metrics.uptime() {
        output.push_str(&format!("Uptime: {}s\n\n", uptime.num_seconds()));
    }

    output.push_str("--- Counters ---\n");
    for metric in metrics.by_kind(MetricKind::Counter) {
        output.push_str(&format!("• {}: {:.0} {}\n", metric.name, metric.current, metric.unit));
    }

    output.push_str("\n--- Gauges ---\n");
    for metric in metrics.by_kind(MetricKind::Gauge) {
        output.push_str(&format!("• {}: {:.1} {}\n", metric.name, metric.current, metric.unit));
    }

    output
}

/// Check if query is about metrics
pub fn is_metrics_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("metrics")
        || lower.contains("telemetry")
        || lower.contains("statistics")
}

/// Fun fact about metrics
pub fn settings_metrics_fun_fact() -> &'static str {
    "Anna collects metrics to help you understand your settings usage patterns!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_kind_display() {
        assert_eq!(format!("{}", MetricKind::Counter), "Counter");
        assert_eq!(format!("{}", MetricKind::Gauge), "Gauge");
    }

    #[test]
    fn test_metric_unit_display() {
        assert_eq!(format!("{}", MetricUnit::Milliseconds), "ms");
        assert_eq!(format!("{}", MetricUnit::Percent), "%");
    }

    #[test]
    fn test_metric_value_new() {
        let val = MetricValue::new(42.0);
        assert_eq!(val.value, 42.0);
    }

    #[test]
    fn test_metric_counter() {
        let metric = Metric::counter("test", "Test counter");
        assert_eq!(metric.kind, MetricKind::Counter);
        assert_eq!(metric.current, 0.0);
    }

    #[test]
    fn test_metric_increment() {
        let mut metric = Metric::counter("test", "Test");
        metric.increment();
        assert_eq!(metric.current, 1.0);
        metric.increment_by(5.0);
        assert_eq!(metric.current, 6.0);
    }

    #[test]
    fn test_metric_gauge_set() {
        let mut metric = Metric::gauge("test", "Test gauge");
        metric.set(50.0);
        assert_eq!(metric.current, 50.0);
        assert_eq!(metric.min, Some(50.0));
        metric.set(25.0);
        assert_eq!(metric.min, Some(25.0));
        assert_eq!(metric.max, Some(50.0));
    }

    #[test]
    fn test_metric_timer_record() {
        let mut metric = Metric::timer("test", "Test timer");
        metric.record(100.0);
        assert_eq!(metric.current, 100.0);
    }

    #[test]
    fn test_settings_metrics_new() {
        let metrics = SettingsMetrics::new();
        assert!(metrics.count() > 0);
    }

    #[test]
    fn test_settings_metrics_increment() {
        let mut metrics = SettingsMetrics::new();
        metrics.increment("settings.changes");
        let metric = metrics.get("settings.changes").unwrap();
        assert_eq!(metric.current, 1.0);
    }

    #[test]
    fn test_settings_metrics_by_kind() {
        let metrics = SettingsMetrics::new();
        let counters = metrics.by_kind(MetricKind::Counter);
        assert!(!counters.is_empty());
    }

    #[test]
    fn test_format_metrics() {
        let metrics = SettingsMetrics::new();
        let output = format_metrics(&metrics);
        assert!(output.contains("Metrics"));
    }

    #[test]
    fn test_is_metrics_query() {
        assert!(is_metrics_query("show metrics"));
        assert!(is_metrics_query("telemetry data"));
        assert!(!is_metrics_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_metrics_fun_fact();
        assert!(fact.contains("metrics"));
    }
}
