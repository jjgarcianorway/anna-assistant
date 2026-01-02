//! Monitor types and implementations (v0.0.430).
//!
//! User-defined monitors check conditions and trigger alerts.

use super::checks::*;
use crate::background_worker::job::{BackgroundJob, JobPriority};
use crate::background_worker::notifications::AlertPriority;
use crate::background_worker::ALERT_COOLDOWN_HOURS;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A user-defined monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    /// Unique monitor ID
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Check command or probe type
    pub check: MonitorCheck,
    /// Threshold condition
    pub threshold: ThresholdCondition,
    /// Alert message template
    pub alert_message: String,
    /// Check interval in seconds
    pub interval_secs: u64,
    /// Alert priority
    pub priority: AlertPriority,
    /// Whether monitor is enabled
    pub enabled: bool,
    /// When created
    pub created_at: u64,
    /// Last check time
    pub last_check: Option<u64>,
    /// Last alert time (for cooldown)
    pub last_alert: Option<u64>,
    /// Current status
    pub status: MonitorStatus,
}

/// What the monitor checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MonitorCheck {
    /// Check disk space on a path
    DiskSpace { path: String },
    /// Check if a process is running
    ProcessRunning { name: String },
    /// Check file age (modified time)
    FileAge { path: String },
    /// Run a custom command
    Command { cmd: String, args: Vec<String> },
    /// Check system load
    SystemLoad,
    /// Check memory usage
    MemoryUsage,
}

/// Threshold condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ThresholdCondition {
    /// Value greater than threshold
    GreaterThan { value: f64 },
    /// Value less than threshold
    LessThan { value: f64 },
    /// Value equals threshold
    Equals { value: f64 },
    /// Boolean check (0 = false, non-0 = true)
    IsTrue,
    /// Boolean check (0 = true, non-0 = false)
    IsFalse,
}

impl ThresholdCondition {
    /// Check if value triggers the condition
    pub fn check(&self, value: f64) -> bool {
        match self {
            Self::GreaterThan { value: threshold } => value > *threshold,
            Self::LessThan { value: threshold } => value < *threshold,
            Self::Equals { value: threshold } => (value - threshold).abs() < 0.001,
            Self::IsTrue => value != 0.0,
            Self::IsFalse => value == 0.0,
        }
    }
}

/// Monitor status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MonitorStatus {
    /// Never checked
    Unknown,
    /// Last check passed
    Ok { last_value: f64 },
    /// Last check triggered alert
    Alert { last_value: f64, message: String },
    /// Check failed to run
    Error { error: String },
}

impl Monitor {
    /// Create a new monitor
    pub fn new(
        id: &str,
        description: &str,
        check: MonitorCheck,
        threshold: ThresholdCondition,
    ) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            check,
            threshold,
            alert_message: "Monitor {id} triggered: {value}".to_string(),
            interval_secs: 3600, // 1 hour default
            priority: AlertPriority::Normal,
            enabled: true,
            created_at: now_timestamp(),
            last_check: None,
            last_alert: None,
            status: MonitorStatus::Unknown,
        }
    }

    /// Set alert message template
    pub fn with_message(mut self, msg: &str) -> Self {
        self.alert_message = msg.to_string();
        self
    }

    /// Set check interval
    pub fn with_interval(mut self, secs: u64) -> Self {
        self.interval_secs = secs;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: AlertPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Check if due for a check
    pub fn is_due(&self, now: u64) -> bool {
        if !self.enabled {
            return false;
        }
        match self.last_check {
            Some(last) => now.saturating_sub(last) >= self.interval_secs,
            None => true,
        }
    }

    /// Check if alert is in cooldown
    pub fn in_cooldown(&self, now: u64) -> bool {
        match self.last_alert {
            Some(last) => now.saturating_sub(last) < ALERT_COOLDOWN_HOURS * 3600,
            None => false,
        }
    }

    /// Run the check and return result
    pub fn run_check(&mut self) -> MonitorCheckResult {
        let now = now_timestamp();
        self.last_check = Some(now);

        let check_result = match &self.check {
            MonitorCheck::DiskSpace { path } => check_disk_space(path),
            MonitorCheck::ProcessRunning { name } => check_process(name),
            MonitorCheck::FileAge { path } => check_file_age(path),
            MonitorCheck::Command { cmd, args } => run_command(cmd, args),
            MonitorCheck::SystemLoad => check_system_load(),
            MonitorCheck::MemoryUsage => check_memory(),
        };

        match check_result {
            Ok(value) => {
                let triggered = self.threshold.check(value);
                if triggered {
                    let message = self
                        .alert_message
                        .replace("{id}", &self.id)
                        .replace("{value}", &format!("{:.2}", value));
                    self.status = MonitorStatus::Alert {
                        last_value: value,
                        message: message.clone(),
                    };

                    if self.in_cooldown(now) {
                        MonitorCheckResult::AlertCooldown { value, message }
                    } else {
                        self.last_alert = Some(now);
                        MonitorCheckResult::Alert { value, message }
                    }
                } else {
                    self.status = MonitorStatus::Ok { last_value: value };
                    MonitorCheckResult::Ok { value }
                }
            }
            Err(error) => {
                self.status = MonitorStatus::Error {
                    error: error.clone(),
                };
                MonitorCheckResult::Error { error }
            }
        }
    }

    /// Create a background job for this monitor
    pub fn to_job(&self) -> BackgroundJob {
        BackgroundJob::monitor_check(&self.id, JobPriority::Normal)
    }
}

/// Result of a monitor check
#[derive(Debug, Clone)]
pub enum MonitorCheckResult {
    /// Check passed, no alert
    Ok { value: f64 },
    /// Check triggered, should alert
    Alert { value: f64, message: String },
    /// Check triggered but in cooldown
    AlertCooldown { value: f64, message: String },
    /// Check failed to run
    Error { error: String },
}

fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_conditions() {
        assert!(ThresholdCondition::GreaterThan { value: 80.0 }.check(90.0));
        assert!(!ThresholdCondition::GreaterThan { value: 80.0 }.check(70.0));
        assert!(ThresholdCondition::LessThan { value: 10.0 }.check(5.0));
        assert!(ThresholdCondition::IsTrue.check(1.0));
        assert!(!ThresholdCondition::IsTrue.check(0.0));
    }

    #[test]
    fn test_monitor_creation() {
        let monitor = Monitor::new(
            "disk-root",
            "Root disk space",
            MonitorCheck::DiskSpace {
                path: "/".to_string(),
            },
            ThresholdCondition::GreaterThan { value: 90.0 },
        );
        assert_eq!(monitor.id, "disk-root");
        assert!(monitor.enabled);
    }
}
