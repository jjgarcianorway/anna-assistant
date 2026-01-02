// v0.0.584: Settings Metrics Types
// Core metric types and enums

use serde::{Deserialize, Serialize};

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
}
