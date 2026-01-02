// v0.0.584: Metrics Collection
// Main collection and management of metrics

use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;
use super::{Metric, MetricKind, MetricUnit};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
