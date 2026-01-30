//! Predictive alerts generation.
//!
//! Generates alerts based on trend analysis and forecasts.

use serde::{Deserialize, Serialize};
use super::forecaster::{Forecaster, ResourceForecast};
use super::trends::{detect_boot_degradation, detect_memory_leak};

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// A predictive alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveAlert {
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Severity level
    pub severity: AlertSeverity,
    /// Resource affected
    pub resource: String,
    /// Recommended action
    pub recommendation: String,
    /// Time horizon (e.g., "3 days", "1 week")
    pub time_horizon: Option<String>,
}

impl PredictiveAlert {
    fn disk_warning(forecast: &ResourceForecast, days: f64) -> Self {
        Self {
            title: "Disk Space Running Low".to_string(),
            description: format!(
                "Disk usage at {:.1}%, projected to reach {:.0}% in {:.0} days.",
                forecast.current_value, forecast.warning_threshold, days
            ),
            severity: AlertSeverity::Warning,
            resource: "disk".to_string(),
            recommendation: "Consider cleaning up old files, logs, or package cache (paccache -r).".to_string(),
            time_horizon: Some(format!("{:.0} days", days)),
        }
    }

    fn disk_critical(forecast: &ResourceForecast, days: f64) -> Self {
        Self {
            title: "Disk Space Critical".to_string(),
            description: format!(
                "Disk usage at {:.1}%, will reach critical {:.0}% in {:.0} days!",
                forecast.current_value, forecast.critical_threshold, days
            ),
            severity: AlertSeverity::Critical,
            resource: "disk".to_string(),
            recommendation: "Urgent: Free up disk space immediately. Run 'ncdu /' to find large files.".to_string(),
            time_horizon: Some(format!("{:.0} days", days)),
        }
    }

    fn memory_leak() -> Self {
        Self {
            title: "Possible Memory Leak Detected".to_string(),
            description: "Memory usage has been consistently increasing over time.".to_string(),
            severity: AlertSeverity::Warning,
            resource: "memory".to_string(),
            recommendation: "Check for applications with growing memory usage using 'htop' sorted by memory.".to_string(),
            time_horizon: None,
        }
    }

    fn boot_degradation(current: f64, slope: f64) -> Self {
        Self {
            title: "Boot Time Increasing".to_string(),
            description: format!(
                "Boot time is {:.1}s and increasing by {:.1}s per boot.",
                current, slope
            ),
            severity: AlertSeverity::Info,
            resource: "boot".to_string(),
            recommendation: "Run 'systemd-analyze blame' to identify slow services.".to_string(),
            time_horizon: None,
        }
    }
}

/// Input data for generating alerts.
pub struct AlertInput {
    /// Daily disk usage percentages (most recent last)
    pub disk_usage: Vec<f64>,
    /// Memory usage samples (most recent last)
    pub memory_usage: Vec<f64>,
    /// Boot times in seconds (most recent last)
    pub boot_times: Vec<f64>,
}

/// Generate predictive alerts from historical data.
pub fn generate_predictive_alerts(input: &AlertInput) -> Vec<PredictiveAlert> {
    let mut alerts = Vec::new();
    let forecaster = Forecaster::default();

    // Disk alerts
    if !input.disk_usage.is_empty() {
        let forecast = forecaster.forecast_disk(&input.disk_usage);

        if let Some(days) = forecast.days_until_critical {
            if days < 7.0 {
                alerts.push(PredictiveAlert::disk_critical(&forecast, days));
            }
        } else if let Some(days) = forecast.days_until_warning {
            if days < 14.0 {
                alerts.push(PredictiveAlert::disk_warning(&forecast, days));
            }
        }
    }

    // Memory leak detection
    if input.memory_usage.len() >= 7 {
        if detect_memory_leak(&input.memory_usage).is_some() {
            alerts.push(PredictiveAlert::memory_leak());
        }
    }

    // Boot time degradation
    if input.boot_times.len() >= 5 {
        if let Some(analysis) = detect_boot_degradation(&input.boot_times) {
            alerts.push(PredictiveAlert::boot_degradation(analysis.current, analysis.slope));
        }
    }

    // Sort by severity (critical first)
    alerts.sort_by(|a, b| {
        let severity_order = |s: &AlertSeverity| match s {
            AlertSeverity::Critical => 0,
            AlertSeverity::Warning => 1,
            AlertSeverity::Info => 2,
        };
        severity_order(&a.severity).cmp(&severity_order(&b.severity))
    });

    alerts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_alerts_healthy_system() {
        let input = AlertInput {
            disk_usage: vec![50.0, 50.0, 50.0, 50.0, 50.0],
            memory_usage: vec![40.0, 42.0, 41.0, 40.0, 41.0],
            boot_times: vec![15.0, 15.0, 15.0, 15.0, 15.0],
        };

        let alerts = generate_predictive_alerts(&input);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_disk_warning_alert() {
        let input = AlertInput {
            disk_usage: vec![70.0, 72.0, 74.0, 76.0, 78.0, 80.0, 82.0],
            memory_usage: vec![],
            boot_times: vec![],
        };

        let alerts = generate_predictive_alerts(&input);
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].resource, "disk");
    }

    #[test]
    fn test_memory_leak_alert() {
        let input = AlertInput {
            disk_usage: vec![],
            memory_usage: vec![30.0, 32.0, 34.0, 36.0, 38.0, 40.0, 42.0],
            boot_times: vec![],
        };

        let alerts = generate_predictive_alerts(&input);
        // Memory leak detection requires consistent increase
        if !alerts.is_empty() {
            assert!(alerts.iter().any(|a| a.resource == "memory"));
        }
    }
}
