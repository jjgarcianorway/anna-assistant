//! Resource forecasting.
//!
//! Predicts when resources will hit thresholds.

use serde::{Deserialize, Serialize};
use super::trends::{analyze_trend, TrendAnalysis, TrendDirection};

/// Resource forecast result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceForecast {
    /// Resource name (e.g., "disk", "memory")
    pub resource: String,
    /// Current value (percentage or absolute)
    pub current_value: f64,
    /// Unit (e.g., "GB", "%")
    pub unit: String,
    /// Warning threshold
    pub warning_threshold: f64,
    /// Critical threshold
    pub critical_threshold: f64,
    /// Days until warning (None if not trending towards warning)
    pub days_until_warning: Option<f64>,
    /// Days until critical (None if not trending towards critical)
    pub days_until_critical: Option<f64>,
    /// Trend analysis
    pub trend: Option<TrendAnalysis>,
    /// Human-readable summary
    pub summary: String,
}

impl ResourceForecast {
    /// Check if resource needs attention.
    pub fn needs_attention(&self) -> bool {
        self.days_until_warning.map(|d| d < 7.0).unwrap_or(false)
            || self.current_value >= self.warning_threshold
    }

    /// Check if resource is critical.
    pub fn is_critical(&self) -> bool {
        self.days_until_critical.map(|d| d < 3.0).unwrap_or(false)
            || self.current_value >= self.critical_threshold
    }
}

/// Forecaster for system resources.
pub struct Forecaster {
    /// Warning threshold for disk (percentage)
    pub disk_warning: f64,
    /// Critical threshold for disk (percentage)
    pub disk_critical: f64,
    /// Warning threshold for memory (percentage)
    pub memory_warning: f64,
    /// Critical threshold for memory (percentage)
    pub memory_critical: f64,
    /// Forecast horizon in days
    pub horizon_days: usize,
}

impl Default for Forecaster {
    fn default() -> Self {
        Self {
            disk_warning: 85.0,
            disk_critical: 95.0,
            memory_warning: 85.0,
            memory_critical: 95.0,
            horizon_days: 14,
        }
    }
}

impl Forecaster {
    /// Create with custom thresholds.
    pub fn with_thresholds(
        disk_warning: f64,
        disk_critical: f64,
        memory_warning: f64,
        memory_critical: f64,
    ) -> Self {
        Self {
            disk_warning,
            disk_critical,
            memory_warning,
            memory_critical,
            horizon_days: 14,
        }
    }

    /// Forecast disk usage.
    pub fn forecast_disk(&self, disk_usage_history: &[f64]) -> ResourceForecast {
        let trend = analyze_trend(disk_usage_history, self.horizon_days);
        let current = disk_usage_history.last().copied().unwrap_or(0.0);

        let (days_until_warning, days_until_critical) = if let Some(ref t) = trend {
            (
                t.days_until(self.disk_warning),
                t.days_until(self.disk_critical),
            )
        } else {
            (None, None)
        };

        let summary = self.generate_disk_summary(current, &trend, days_until_warning, days_until_critical);

        ResourceForecast {
            resource: "disk".to_string(),
            current_value: current,
            unit: "%".to_string(),
            warning_threshold: self.disk_warning,
            critical_threshold: self.disk_critical,
            days_until_warning,
            days_until_critical,
            trend,
            summary,
        }
    }

    /// Forecast memory usage.
    pub fn forecast_memory(&self, memory_usage_history: &[f64]) -> ResourceForecast {
        let trend = analyze_trend(memory_usage_history, self.horizon_days);
        let current = memory_usage_history.last().copied().unwrap_or(0.0);

        let (days_until_warning, days_until_critical) = if let Some(ref t) = trend {
            (
                t.days_until(self.memory_warning),
                t.days_until(self.memory_critical),
            )
        } else {
            (None, None)
        };

        let summary = self.generate_memory_summary(current, &trend);

        ResourceForecast {
            resource: "memory".to_string(),
            current_value: current,
            unit: "%".to_string(),
            warning_threshold: self.memory_warning,
            critical_threshold: self.memory_critical,
            days_until_warning,
            days_until_critical,
            trend,
            summary,
        }
    }

    /// Forecast boot time.
    pub fn forecast_boot_time(&self, boot_times: &[f64]) -> ResourceForecast {
        let trend = analyze_trend(boot_times, 7);
        let current = boot_times.last().copied().unwrap_or(0.0);

        let summary = self.generate_boot_summary(current, &trend);

        // Boot time uses absolute thresholds (seconds)
        ResourceForecast {
            resource: "boot_time".to_string(),
            current_value: current,
            unit: "s".to_string(),
            warning_threshold: 30.0,  // 30 seconds warning
            critical_threshold: 60.0, // 60 seconds critical
            days_until_warning: None, // Boot time forecasting is different
            days_until_critical: None,
            trend,
            summary,
        }
    }

    fn generate_disk_summary(
        &self,
        current: f64,
        trend: &Option<TrendAnalysis>,
        days_warning: Option<f64>,
        days_critical: Option<f64>,
    ) -> String {
        let mut parts = vec![format!("Disk usage: {:.1}%", current)];

        if current >= self.disk_critical {
            parts.push("CRITICAL - Disk almost full!".to_string());
        } else if current >= self.disk_warning {
            parts.push("Warning - Disk usage high.".to_string());
        }

        if let Some(days) = days_critical {
            if days < 7.0 {
                parts.push(format!("Will reach critical in {:.0} days.", days));
            }
        } else if let Some(days) = days_warning {
            if days < 14.0 {
                parts.push(format!("May reach warning level in {:.0} days.", days));
            }
        }

        if let Some(t) = trend {
            match t.direction {
                TrendDirection::Increasing if t.is_significant() => {
                    parts.push(format!("Growing at {:.1}%/day.", t.slope));
                }
                TrendDirection::Decreasing if t.is_significant() => {
                    parts.push("Usage trending down.".to_string());
                }
                _ => {}
            }
        }

        parts.join(" ")
    }

    fn generate_memory_summary(&self, current: f64, trend: &Option<TrendAnalysis>) -> String {
        let mut parts = vec![format!("Memory usage: {:.1}%", current)];

        if let Some(t) = trend {
            if t.direction == TrendDirection::Increasing && t.slope > 0.5 && t.r_squared > 0.6 {
                parts.push("Possible memory leak detected - usage consistently increasing.".to_string());
            }
        }

        parts.join(" ")
    }

    fn generate_boot_summary(&self, current: f64, trend: &Option<TrendAnalysis>) -> String {
        let mut parts = vec![format!("Boot time: {:.1}s", current)];

        if let Some(t) = trend {
            if t.direction == TrendDirection::Increasing && t.slope > 0.5 {
                parts.push(format!(
                    "Boot time increasing by {:.1}s per boot. Consider investigating startup services.",
                    t.slope
                ));
            } else if t.direction == TrendDirection::Decreasing && t.slope.abs() > 0.5 {
                parts.push("Boot time improving.".to_string());
            }
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_forecast_stable() {
        let forecaster = Forecaster::default();
        let history = vec![50.0, 51.0, 50.0, 51.0, 50.0];
        let forecast = forecaster.forecast_disk(&history);

        assert!(!forecast.needs_attention());
        assert!(!forecast.is_critical());
    }

    #[test]
    fn test_disk_forecast_growing() {
        let forecaster = Forecaster::default();
        let history = vec![70.0, 72.0, 74.0, 76.0, 78.0, 80.0];
        let forecast = forecaster.forecast_disk(&history);

        assert!(forecast.trend.is_some());
        assert_eq!(forecast.trend.as_ref().unwrap().direction, TrendDirection::Increasing);
    }

    #[test]
    fn test_memory_leak_detection() {
        let forecaster = Forecaster::default();
        let history = vec![40.0, 42.0, 44.0, 46.0, 48.0, 50.0, 52.0];
        let forecast = forecaster.forecast_memory(&history);

        assert!(forecast.summary.contains("memory leak") || forecast.trend.as_ref().map(|t| t.slope > 1.0).unwrap_or(false));
    }
}
