//! Telemetry Diff Engine - Detect changes between telemetry snapshots
//!
//! v6.47.0: Calculate meaningful differences for greeting context

use serde::{Deserialize, Serialize};

/// Simplified telemetry snapshot for diffing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetrySnapshot {
    /// Package count
    pub package_count: u32,

    /// Failed service count
    pub failed_services: u32,

    /// Memory usage percentage
    pub memory_usage_percent: f64,

    /// CPU load average (1 min)
    pub load_average: f64,

    /// Critical errors in logs
    pub critical_errors: u32,

    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Result of comparing two snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDiff {
    /// Notable changes detected
    pub changes: Vec<TelemetryChange>,

    /// Overall health trend
    pub health_trend: HealthTrend,
}

/// A single detected change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryChange {
    /// What changed
    pub metric: String,

    /// Previous value
    pub old_value: String,

    /// New value
    pub new_value: String,

    /// Severity of the change
    pub severity: ChangeSeverity,

    /// Human-readable description
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthTrend {
    Improving,
    Stable,
    Degrading,
}

/// Calculate diff between snapshots
pub fn calculate_diff(old: &TelemetrySnapshot, new: &TelemetrySnapshot) -> TelemetryDiff {
    let mut changes = Vec::new();

    // Package count changes
    if old.package_count != new.package_count {
        let diff = new.package_count as i32 - old.package_count as i32;
        let description = if diff > 0 {
            format!("{} packages added", diff.abs())
        } else {
            format!("{} packages removed", diff.abs())
        };

        changes.push(TelemetryChange {
            metric: "packages".to_string(),
            old_value: old.package_count.to_string(),
            new_value: new.package_count.to_string(),
            severity: if diff.abs() > 10 {
                ChangeSeverity::Warning
            } else {
                ChangeSeverity::Info
            },
            description,
        });
    }

    // Failed services
    if old.failed_services != new.failed_services {
        let diff = new.failed_services as i32 - old.failed_services as i32;
        let description = if diff > 0 {
            format!("{} services failed", diff.abs())
        } else {
            format!("{} services recovered", diff.abs())
        };

        changes.push(TelemetryChange {
            metric: "services".to_string(),
            old_value: old.failed_services.to_string(),
            new_value: new.failed_services.to_string(),
            severity: if new.failed_services > 0 {
                ChangeSeverity::Critical
            } else if diff < 0 {
                ChangeSeverity::Info
            } else {
                ChangeSeverity::Warning
            },
            description,
        });
    }

    // Memory usage
    let memory_delta = new.memory_usage_percent - old.memory_usage_percent;
    if memory_delta.abs() > 10.0 {
        changes.push(TelemetryChange {
            metric: "memory".to_string(),
            old_value: format!("{:.1}%", old.memory_usage_percent),
            new_value: format!("{:.1}%", new.memory_usage_percent),
            severity: if new.memory_usage_percent > 90.0 {
                ChangeSeverity::Critical
            } else if new.memory_usage_percent > 80.0 {
                ChangeSeverity::Warning
            } else {
                ChangeSeverity::Info
            },
            description: if memory_delta > 0.0 {
                format!("Memory usage increased by {:.1}%", memory_delta)
            } else {
                format!("Memory usage decreased by {:.1}%", memory_delta.abs())
            },
        });
    }

    // Load average
    let load_delta = new.load_average - old.load_average;
    if load_delta.abs() > 1.0 {
        changes.push(TelemetryChange {
            metric: "load".to_string(),
            old_value: format!("{:.2}", old.load_average),
            new_value: format!("{:.2}", new.load_average),
            severity: if new.load_average > 8.0 {
                ChangeSeverity::Critical
            } else if new.load_average > 4.0 {
                ChangeSeverity::Warning
            } else {
                ChangeSeverity::Info
            },
            description: if load_delta > 0.0 {
                format!("Load increased by {:.2}", load_delta)
            } else {
                format!("Load decreased by {:.2}", load_delta.abs())
            },
        });
    }

    // Critical errors
    if old.critical_errors != new.critical_errors {
        let diff = new.critical_errors as i32 - old.critical_errors as i32;
        let description = if diff > 0 {
            format!("{} new critical errors", diff.abs())
        } else {
            format!("{} errors cleared", diff.abs())
        };

        changes.push(TelemetryChange {
            metric: "errors".to_string(),
            old_value: old.critical_errors.to_string(),
            new_value: new.critical_errors.to_string(),
            severity: if new.critical_errors > 0 {
                ChangeSeverity::Critical
            } else {
                ChangeSeverity::Info
            },
            description,
        });
    }

    // Determine health trend
    let health_trend = calculate_health_trend(&changes);

    TelemetryDiff {
        changes,
        health_trend,
    }
}

fn calculate_health_trend(changes: &[TelemetryChange]) -> HealthTrend {
    if changes.is_empty() {
        return HealthTrend::Stable;
    }

    let critical_count = changes
        .iter()
        .filter(|c| matches!(c.severity, ChangeSeverity::Critical))
        .count();

    let warning_count = changes
        .iter()
        .filter(|c| matches!(c.severity, ChangeSeverity::Warning))
        .count();

    // Check for specific improving signals
    let improving_signals = changes.iter().filter(|c| {
        c.description.contains("recovered")
            || c.description.contains("cleared")
            || c.description.contains("decreased")
    }).count();

    if critical_count > 0 || warning_count > 2 {
        HealthTrend::Degrading
    } else if improving_signals > 0 && critical_count == 0 {
        HealthTrend::Improving
    } else {
        HealthTrend::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_snapshot() -> TelemetrySnapshot {
        TelemetrySnapshot {
            package_count: 500,
            failed_services: 0,
            memory_usage_percent: 45.0,
            load_average: 1.5,
            critical_errors: 0,
            uptime_seconds: 86400,
        }
    }

    #[test]
    fn test_no_changes() {
        let old = baseline_snapshot();
        let new = baseline_snapshot();

        let diff = calculate_diff(&old, &new);

        assert!(diff.changes.is_empty());
        assert_eq!(diff.health_trend, HealthTrend::Stable);
    }

    #[test]
    fn test_package_addition() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.package_count = 505;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].metric, "packages");
        assert!(diff.changes[0].description.contains("5 packages added"));
        assert_eq!(diff.changes[0].severity, ChangeSeverity::Info);
    }

    #[test]
    fn test_service_failure() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.failed_services = 2;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].metric, "services");
        assert!(diff.changes[0].description.contains("2 services failed"));
        assert_eq!(diff.changes[0].severity, ChangeSeverity::Critical);
        assert_eq!(diff.health_trend, HealthTrend::Degrading);
    }

    #[test]
    fn test_service_recovery() {
        let mut old = baseline_snapshot();
        old.failed_services = 2;
        let new = baseline_snapshot();

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 1);
        assert!(diff.changes[0].description.contains("2 services recovered"));
        assert_eq!(diff.changes[0].severity, ChangeSeverity::Info);
        assert_eq!(diff.health_trend, HealthTrend::Improving);
    }

    #[test]
    fn test_memory_spike() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.memory_usage_percent = 85.0;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].metric, "memory");
        assert!(diff.changes[0].description.contains("increased"));
        assert_eq!(diff.changes[0].severity, ChangeSeverity::Warning);
    }

    #[test]
    fn test_memory_critical() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.memory_usage_percent = 95.0;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes[0].severity, ChangeSeverity::Critical);
        assert_eq!(diff.health_trend, HealthTrend::Degrading);
    }

    #[test]
    fn test_load_increase() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.load_average = 6.0;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].metric, "load");
        assert!(diff.changes[0].description.contains("increased"));
        assert_eq!(diff.changes[0].severity, ChangeSeverity::Warning);
    }

    #[test]
    fn test_critical_errors_appeared() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.critical_errors = 5;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].metric, "errors");
        assert!(diff.changes[0].description.contains("5 new critical errors"));
        assert_eq!(diff.changes[0].severity, ChangeSeverity::Critical);
    }

    #[test]
    fn test_errors_cleared() {
        let mut old = baseline_snapshot();
        old.critical_errors = 3;
        let new = baseline_snapshot();

        let diff = calculate_diff(&old, &new);

        assert!(diff.changes[0].description.contains("3 errors cleared"));
        assert_eq!(diff.health_trend, HealthTrend::Improving);
    }

    #[test]
    fn test_multiple_changes() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.package_count = 515; // +15 packages
        new.memory_usage_percent = 60.0; // +15%
        new.load_average = 3.0; // +1.5

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.changes.len(), 3);
        assert!(diff.changes.iter().any(|c| c.metric == "packages"));
        assert!(diff.changes.iter().any(|c| c.metric == "memory"));
        assert!(diff.changes.iter().any(|c| c.metric == "load"));
    }

    #[test]
    fn test_health_trend_degrading() {
        let old = baseline_snapshot();
        let mut new = baseline_snapshot();
        new.failed_services = 1;
        new.critical_errors = 2;
        new.memory_usage_percent = 95.0;

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.health_trend, HealthTrend::Degrading);
    }

    #[test]
    fn test_health_trend_improving() {
        let mut old = baseline_snapshot();
        old.failed_services = 2;
        old.critical_errors = 3;
        let new = baseline_snapshot();

        let diff = calculate_diff(&old, &new);

        assert_eq!(diff.health_trend, HealthTrend::Improving);
    }
}
