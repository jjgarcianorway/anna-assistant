//! Proactive health alerts based on telemetry (v0.0.281).
//!
//! Generates alerts that Anna can proactively mention to users.

use crate::system_telemetry::{AnomalyCategory, AnomalySeverity, TelemetryStore};
use serde::{Deserialize, Serialize};

/// A proactive health alert for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub severity: AlertSeverity,
    pub category: String,
    pub message: String,
    pub recommendation: String,
    pub dismissed: bool,
}

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl From<AnomalySeverity> for AlertSeverity {
    fn from(s: AnomalySeverity) -> Self {
        match s {
            AnomalySeverity::Info => Self::Info,
            AnomalySeverity::Warning => Self::Warning,
            AnomalySeverity::Critical => Self::Critical,
        }
    }
}

/// Generate health alerts from telemetry data
pub fn generate_alerts(store: &TelemetryStore) -> Vec<HealthAlert> {
    let mut alerts = Vec::new();

    // Convert anomalies to alerts
    for anomaly in store.recent_anomalies() {
        let recommendation = match anomaly.category {
            AnomalyCategory::HighCpu => {
                "Check running processes with 'top' or 'htop'. Look for runaway processes."
            }
            AnomalyCategory::HighMemory => {
                "Check memory usage with 'free -h'. Consider closing unused applications."
            }
            AnomalyCategory::LowDisk => {
                "Clean up disk space. Check large files with 'du -sh /*' or clean package cache."
            }
            AnomalyCategory::ServiceDown => {
                "Check service status with 'systemctl status'. Restart if needed."
            }
            AnomalyCategory::NetworkError => {
                "Check network interfaces with 'ip addr' and 'networkctl status'."
            }
            AnomalyCategory::HighLoad => "System under heavy load. Check CPU-intensive processes.",
        };

        alerts.push(HealthAlert {
            severity: anomaly.severity.into(),
            category: anomaly.category.to_string(),
            message: anomaly.description.clone(),
            recommendation: recommendation.to_string(),
            dismissed: false,
        });
    }

    // Add trend-based alerts
    let trends = &store.trends;
    if trends.sample_count >= 10 {
        // Disk filling up warning
        if trends.disk_trend > 3.0 {
            alerts.push(HealthAlert {
                severity: AlertSeverity::Warning,
                category: "Disk Trend".to_string(),
                message: format!(
                    "Disk usage increasing by {:.1}% over the tracking window",
                    trends.disk_trend
                ),
                recommendation: "Monitor disk usage. Consider setting up log rotation.".to_string(),
                dismissed: false,
            });
        }

        // Memory pressure trend
        if trends.memory_trend > 10.0 {
            alerts.push(HealthAlert {
                severity: AlertSeverity::Info,
                category: "Memory Trend".to_string(),
                message: format!("Memory usage trending up by {:.1}%", trends.memory_trend),
                recommendation: "Check for memory leaks in long-running processes.".to_string(),
                dismissed: false,
            });
        }
    }

    alerts
}

/// Format alerts for display in greeting
pub fn format_alerts_for_greeting(alerts: &[HealthAlert]) -> Option<String> {
    let critical: Vec<_> = alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::Critical && !a.dismissed)
        .collect();

    let warnings: Vec<_> = alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::Warning && !a.dismissed)
        .collect();

    if critical.is_empty() && warnings.is_empty() {
        return None;
    }

    let mut parts = Vec::new();

    if !critical.is_empty() {
        parts.push(format!(
            "{} critical issue{}",
            critical.len(),
            if critical.len() > 1 { "s" } else { "" }
        ));
    }

    if !warnings.is_empty() {
        parts.push(format!(
            "{} warning{}",
            warnings.len(),
            if warnings.len() > 1 { "s" } else { "" }
        ));
    }

    Some(format!(
        "I noticed {} that may need attention.",
        parts.join(" and ")
    ))
}

/// Get the most urgent alert for proactive mention
pub fn get_urgent_alert(alerts: &[HealthAlert]) -> Option<&HealthAlert> {
    // First look for undismissed critical alerts
    alerts
        .iter()
        .find(|a| a.severity == AlertSeverity::Critical && !a.dismissed)
        .or_else(|| {
            // Then look for warnings
            alerts
                .iter()
                .find(|a| a.severity == AlertSeverity::Warning && !a.dismissed)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_alerts_for_greeting() {
        let alerts = vec![
            HealthAlert {
                severity: AlertSeverity::Critical,
                category: "High CPU".to_string(),
                message: "CPU at 95%".to_string(),
                recommendation: "Check processes".to_string(),
                dismissed: false,
            },
            HealthAlert {
                severity: AlertSeverity::Warning,
                category: "Low Disk".to_string(),
                message: "Disk at 88%".to_string(),
                recommendation: "Clean up".to_string(),
                dismissed: false,
            },
        ];

        let formatted = format_alerts_for_greeting(&alerts);
        assert!(formatted.is_some());
        let text = formatted.unwrap();
        assert!(text.contains("1 critical issue"));
        assert!(text.contains("1 warning"));
    }

    #[test]
    fn test_get_urgent_alert() {
        let alerts = vec![
            HealthAlert {
                severity: AlertSeverity::Warning,
                category: "Warning".to_string(),
                message: "Warning msg".to_string(),
                recommendation: "".to_string(),
                dismissed: false,
            },
            HealthAlert {
                severity: AlertSeverity::Critical,
                category: "Critical".to_string(),
                message: "Critical msg".to_string(),
                recommendation: "".to_string(),
                dismissed: false,
            },
        ];

        let urgent = get_urgent_alert(&alerts);
        assert!(urgent.is_some());
        assert_eq!(urgent.unwrap().severity, AlertSeverity::Critical);
    }
}
