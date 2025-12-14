// v0.0.608: Settings Monitor (Phase 184)
// Monitor settings for changes and anomalies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Monitor type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorType {
    /// Change detection
    Change,
    /// Threshold monitoring
    Threshold,
    /// Pattern matching
    Pattern,
    /// Anomaly detection
    Anomaly,
    /// Health check
    Health,
}

impl std::fmt::Display for MonitorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Change => write!(f, "change"),
            Self::Threshold => write!(f, "threshold"),
            Self::Pattern => write!(f, "pattern"),
            Self::Anomaly => write!(f, "anomaly"),
            Self::Health => write!(f, "health"),
        }
    }
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Info
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Monitor type
    pub monitor_type: MonitorType,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
    /// Check interval seconds
    pub interval_secs: u64,
    /// Enabled
    pub enabled: bool,
}

impl MonitorConfig {
    /// Create new config
    pub fn new(id: impl Into<String>, monitor_type: MonitorType) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            monitor_type,
            categories: Vec::new(),
            interval_secs: 60,
            enabled: true,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set interval
    pub fn interval(mut self, secs: u64) -> Self {
        self.interval_secs = secs;
        self
    }

    /// Enable/disable
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Monitor ID
    pub monitor_id: String,
    /// Severity
    pub severity: AlertSeverity,
    /// Message
    pub message: String,
    /// Timestamp
    pub timestamp: u64,
    /// Acknowledged
    pub acknowledged: bool,
}

impl Alert {
    /// Create new alert
    pub fn new(
        id: impl Into<String>,
        monitor_id: impl Into<String>,
        severity: AlertSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            monitor_id: monitor_id.into(),
            severity,
            message: message.into(),
            timestamp: 0,
            acknowledged: false,
        }
    }

    /// Acknowledge
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// Is critical
    pub fn is_critical(&self) -> bool {
        self.severity == AlertSeverity::Critical
    }
}

/// Monitor status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorStatus {
    /// Monitor ID
    pub monitor_id: String,
    /// Last check timestamp
    pub last_check: u64,
    /// Healthy
    pub healthy: bool,
    /// Alert count
    pub alert_count: usize,
}

impl MonitorStatus {
    /// Create new status
    pub fn new(monitor_id: impl Into<String>) -> Self {
        Self {
            monitor_id: monitor_id.into(),
            last_check: 0,
            healthy: true,
            alert_count: 0,
        }
    }

    /// Mark unhealthy
    pub fn mark_unhealthy(&mut self) {
        self.healthy = false;
    }

    /// Increment alerts
    pub fn add_alert(&mut self) {
        self.alert_count += 1;
        self.healthy = false;
    }
}

/// Settings monitor
#[derive(Debug, Clone, Default)]
pub struct SettingsMonitor {
    /// Configurations
    configs: HashMap<String, MonitorConfig>,
    /// Status
    status: HashMap<String, MonitorStatus>,
    /// Alerts
    alerts: Vec<Alert>,
    /// Max alerts
    max_alerts: usize,
}

impl SettingsMonitor {
    /// Create new monitor
    pub fn new() -> Self {
        Self {
            max_alerts: 1000,
            ..Default::default()
        }
    }

    /// Add config
    pub fn add_config(&mut self, config: MonitorConfig) {
        let id = config.id.clone();
        self.configs.insert(id.clone(), config);
        self.status.insert(id.clone(), MonitorStatus::new(&id));
    }

    /// Remove config
    pub fn remove_config(&mut self, id: &str) -> Option<MonitorConfig> {
        self.status.remove(id);
        self.configs.remove(id)
    }

    /// Get config
    pub fn get_config(&self, id: &str) -> Option<&MonitorConfig> {
        self.configs.get(id)
    }

    /// Get status
    pub fn get_status(&self, id: &str) -> Option<&MonitorStatus> {
        self.status.get(id)
    }

    /// Add alert
    pub fn add_alert(&mut self, alert: Alert) {
        if let Some(status) = self.status.get_mut(&alert.monitor_id) {
            status.add_alert();
        }
        self.alerts.push(alert);
        while self.alerts.len() > self.max_alerts {
            self.alerts.remove(0);
        }
    }

    /// Get alerts
    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }

    /// Unacknowledged alerts
    pub fn unacknowledged(&self) -> Vec<&Alert> {
        self.alerts.iter().filter(|a| !a.acknowledged).collect()
    }

    /// Config count
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// Alert count
    pub fn alert_count(&self) -> usize {
        self.alerts.len()
    }

    /// Healthy count
    pub fn healthy_count(&self) -> usize {
        self.status.values().filter(|s| s.healthy).count()
    }
}

/// Format monitor
pub fn format_monitor(monitor: &SettingsMonitor) -> String {
    let mut output = String::new();
    output.push_str("Settings Monitor:\n");
    output.push_str(&format!("  Configs: {}\n", monitor.config_count()));
    output.push_str(&format!("  Healthy: {}\n", monitor.healthy_count()));
    output.push_str(&format!("  Alerts: {}\n", monitor.alert_count()));
    output.push_str(&format!("  Unacknowledged: {}\n", monitor.unacknowledged().len()));
    output
}

/// Check if query is about monitor
pub fn is_monitor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("monitor settings")
        || lower.contains("settings alerts")
        || lower.contains("watch settings")
}

/// Fun fact about monitor
pub fn monitor_fun_fact() -> &'static str {
    "Anna monitors your settings for changes and anomalies in real-time!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_type_display() {
        assert_eq!(format!("{}", MonitorType::Change), "change");
        assert_eq!(format!("{}", MonitorType::Anomaly), "anomaly");
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", AlertSeverity::Warning), "warning");
        assert_eq!(format!("{}", AlertSeverity::Critical), "critical");
    }

    #[test]
    fn test_config_new() {
        let c = MonitorConfig::new("m1", MonitorType::Change);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = MonitorConfig::new("m1", MonitorType::Threshold)
            .name("Test")
            .interval(120)
            .category(SettingsCategory::Personality);
        assert_eq!(c.interval_secs, 120);
    }

    #[test]
    fn test_alert_new() {
        let a = Alert::new("a1", "m1", AlertSeverity::Warning, "Test alert");
        assert!(!a.acknowledged);
    }

    #[test]
    fn test_alert_acknowledge() {
        let mut a = Alert::new("a1", "m1", AlertSeverity::Info, "Info");
        a.acknowledge();
        assert!(a.acknowledged);
    }

    #[test]
    fn test_status_new() {
        let s = MonitorStatus::new("m1");
        assert!(s.healthy);
    }

    #[test]
    fn test_monitor_new() {
        let m = SettingsMonitor::new();
        assert_eq!(m.config_count(), 0);
    }

    #[test]
    fn test_monitor_add_config() {
        let mut m = SettingsMonitor::new();
        m.add_config(MonitorConfig::new("m1", MonitorType::Change));
        assert_eq!(m.config_count(), 1);
    }

    #[test]
    fn test_monitor_add_alert() {
        let mut m = SettingsMonitor::new();
        m.add_config(MonitorConfig::new("m1", MonitorType::Change));
        m.add_alert(Alert::new("a1", "m1", AlertSeverity::Warning, "Test"));
        assert_eq!(m.alert_count(), 1);
    }

    #[test]
    fn test_is_monitor_query() {
        assert!(is_monitor_query("monitor settings"));
        assert!(!is_monitor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = monitor_fun_fact();
        assert!(fact.contains("monitor"));
    }
}
