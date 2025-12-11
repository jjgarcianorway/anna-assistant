//! Monitor and reminder system (v0.0.430).
//!
//! User-defined monitors check conditions and trigger alerts.
//! Reminders fire at scheduled times.

use super::job::{BackgroundJob, JobPriority};
use super::notifications::AlertPriority;
use super::ALERT_COOLDOWN_HOURS;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
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
    pub fn new(id: &str, description: &str, check: MonitorCheck, threshold: ThresholdCondition) -> Self {
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

/// A user reminder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// Unique reminder ID
    pub id: String,
    /// What to remind about
    pub message: String,
    /// Schedule (cron-like or specific time)
    pub schedule: ReminderSchedule,
    /// Whether enabled
    pub enabled: bool,
    /// When created
    pub created_at: u64,
    /// Next trigger time
    pub next_trigger: Option<u64>,
    /// Last triggered
    pub last_triggered: Option<u64>,
}

/// Reminder schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReminderSchedule {
    /// One-time reminder at specific timestamp
    Once { at: u64 },
    /// Daily at specific hour:minute
    Daily { hour: u8, minute: u8 },
    /// Weekly on specific day and time (0=Sun, 1=Mon, etc.)
    Weekly { day: u8, hour: u8, minute: u8 },
    /// Monthly on specific day and time
    Monthly { day: u8, hour: u8, minute: u8 },
}

impl Reminder {
    /// Create a new reminder
    pub fn new(id: &str, message: &str, schedule: ReminderSchedule) -> Self {
        let mut reminder = Self {
            id: id.to_string(),
            message: message.to_string(),
            schedule,
            enabled: true,
            created_at: now_timestamp(),
            next_trigger: None,
            last_triggered: None,
        };
        reminder.calculate_next_trigger();
        reminder
    }

    /// Calculate next trigger time
    pub fn calculate_next_trigger(&mut self) {
        let now = now_timestamp();
        self.next_trigger = match &self.schedule {
            ReminderSchedule::Once { at } => {
                if *at > now {
                    Some(*at)
                } else {
                    None
                }
            }
            ReminderSchedule::Daily { hour, minute } => {
                Some(next_daily_time(now, *hour as u32, *minute as u32))
            }
            ReminderSchedule::Weekly { day, hour, minute } => {
                Some(next_weekly_time(now, *day, *hour as u32, *minute as u32))
            }
            ReminderSchedule::Monthly { day, hour, minute } => {
                Some(next_monthly_time(now, *day, *hour as u32, *minute as u32))
            }
        };
    }

    /// Check if reminder is due
    pub fn is_due(&self, now: u64) -> bool {
        self.enabled && self.next_trigger.map(|t| t <= now).unwrap_or(false)
    }

    /// Mark as triggered and calculate next
    pub fn trigger(&mut self) {
        self.last_triggered = Some(now_timestamp());
        self.calculate_next_trigger();
    }

    /// Create a background job for this reminder
    pub fn to_job(&self) -> BackgroundJob {
        BackgroundJob::reminder(&self.id)
    }
}

/// Monitor storage
pub struct MonitorStorage {
    path: PathBuf,
}

impl MonitorStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            path: PathBuf::from(base_path),
        }
    }

    fn monitors_file(&self) -> PathBuf {
        self.path.join(super::MONITORS_FILE)
    }

    pub fn load_monitors(&self) -> Result<Vec<Monitor>, String> {
        let file = self.monitors_file();
        if !file.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save_monitors(&self, monitors: &[Monitor]) -> Result<(), String> {
        fs::create_dir_all(&self.path).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(monitors).map_err(|e| e.to_string())?;
        fs::write(self.monitors_file(), content).map_err(|e| e.to_string())
    }
}

// Check implementations

fn check_disk_space(path: &str) -> Result<f64, String> {
    // Read from df command
    let output = Command::new("df")
        .arg("--output=pcent")
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        if let Some(pct) = line.trim().strip_suffix('%') {
            return pct.parse().map_err(|e: std::num::ParseFloatError| e.to_string());
        }
    }
    Err("Could not parse df output".to_string())
}

fn check_process(name: &str) -> Result<f64, String> {
    let output = Command::new("pgrep")
        .arg("-c")
        .arg(name)
        .output()
        .map_err(|e| e.to_string())?;

    let count: f64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0.0);
    Ok(count)
}

fn check_file_age(path: &str) -> Result<f64, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let modified = metadata.modified().map_err(|e| e.to_string())?;
    let age = SystemTime::now()
        .duration_since(modified)
        .map(|d| d.as_secs() as f64)
        .unwrap_or(0.0);
    Ok(age)
}

fn run_command(cmd: &str, args: &[String]) -> Result<f64, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())
}

fn check_system_load() -> Result<f64, String> {
    let content = fs::read_to_string("/proc/loadavg").map_err(|e| e.to_string())?;
    content
        .split_whitespace()
        .next()
        .ok_or_else(|| "Empty loadavg".to_string())?
        .parse()
        .map_err(|e: std::num::ParseFloatError| e.to_string())
}

fn check_memory() -> Result<f64, String> {
    let content = fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut available = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_meminfo_value(line);
        } else if line.starts_with("MemAvailable:") {
            available = parse_meminfo_value(line);
        }
    }

    if total > 0 {
        Ok(((total - available) as f64 / total as f64) * 100.0)
    } else {
        Err("Could not read memory info".to_string())
    }
}

fn parse_meminfo_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

// Time calculation helpers

fn next_daily_time(now: u64, hour: u32, minute: u32) -> u64 {
    let secs_today = now % 86400;
    let target_secs = hour * 3600 + minute * 60;
    let day_start = now - secs_today;

    if secs_today < target_secs as u64 {
        day_start + target_secs as u64
    } else {
        day_start + 86400 + target_secs as u64
    }
}

fn next_weekly_time(now: u64, day: u8, hour: u32, minute: u32) -> u64 {
    // Simplified: just add 7 days from next daily
    next_daily_time(now, hour, minute) + (day as u64 * 86400)
}

fn next_monthly_time(now: u64, day: u8, hour: u32, minute: u32) -> u64 {
    // Simplified: assume ~30 days
    let daily = next_daily_time(now, hour, minute);
    daily + ((day.saturating_sub(1)) as u64 * 86400)
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

    #[test]
    fn test_reminder_creation() {
        let reminder = Reminder::new(
            "weekly-report",
            "Generate weekly report",
            ReminderSchedule::Weekly {
                day: 1,
                hour: 9,
                minute: 0,
            },
        );
        assert!(reminder.next_trigger.is_some());
    }
}
