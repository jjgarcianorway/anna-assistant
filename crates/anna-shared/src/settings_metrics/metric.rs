// v0.0.584: Metric Definition
// Individual metric with operations

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{MetricKind, MetricUnit};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
