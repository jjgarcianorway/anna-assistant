//! Anna Telemetry v5.3.0 - Process Monitoring & Usage Tracking
//!
//! Pure system telemetry: process activity, resource usage, service health.
//! No LLM, no Q&A - just system intelligence.
//!
//! ## Event Log Format
//!
//! Each log file is append-only with timestamped entries:
//! `timestamp|type|key|value1|value2|...`
//!
//! ## Log Files
//!
//! - process_activity.log: Process CPU/memory samples
//! - command_usage.log: Command execution events
//! - service_changes.log: Service state transitions
//! - package_changes.log: Package install/upgrade/remove
//! - error_events.log: Errors from system logs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Data directory for telemetry
pub const TELEMETRY_DIR: &str = "/var/lib/anna/telemetry";

/// Process activity log (CPU/memory samples)
pub const PROCESS_ACTIVITY_LOG: &str = "process_activity.log";
/// Command usage log
pub const COMMAND_USAGE_LOG: &str = "command_usage.log";
/// Service changes log
pub const SERVICE_CHANGES_LOG: &str = "service_changes.log";
/// Package changes log
pub const PACKAGE_CHANGES_LOG: &str = "package_changes.log";
/// Error events log
pub const ERROR_EVENTS_LOG: &str = "error_events.log";

/// Telemetry aggregates stored in JSON for quick status display
pub const TELEMETRY_STATE_FILE: &str = "/var/lib/anna/telemetry_state.json";

// ============================================================================
// Process Activity Events
// ============================================================================

/// A single process activity sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSample {
    /// Unix timestamp
    pub timestamp: u64,
    /// Process name (command)
    pub name: String,
    /// Process ID
    pub pid: u32,
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Memory usage in bytes
    pub mem_bytes: u64,
}

impl ProcessSample {
    /// Format for log file: timestamp|process|name|pid|cpu|mem
    pub fn to_log_line(&self) -> String {
        format!(
            "{}|process|{}|{}|{:.1}|{}",
            self.timestamp, self.name, self.pid, self.cpu_percent, self.mem_bytes
        )
    }

    /// Parse from log line
    pub fn from_log_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 6 || parts[1] != "process" {
            return None;
        }
        Some(Self {
            timestamp: parts[0].parse().ok()?,
            name: parts[2].to_string(),
            pid: parts[3].parse().ok()?,
            cpu_percent: parts[4].parse().ok()?,
            mem_bytes: parts[5].parse().ok()?,
        })
    }
}

// ============================================================================
// Command Usage Events
// ============================================================================

/// A command execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    /// Unix timestamp
    pub timestamp: u64,
    /// Command name
    pub command: String,
    /// Arguments (optional, truncated)
    pub args: Option<String>,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl CommandEvent {
    /// Format for log file: timestamp|command|name|args|exit|duration
    pub fn to_log_line(&self) -> String {
        format!(
            "{}|command|{}|{}|{}|{}",
            self.timestamp,
            self.command,
            self.args.as_deref().unwrap_or("-"),
            self.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string()),
            self.duration_ms.map(|d| d.to_string()).unwrap_or_else(|| "-".to_string())
        )
    }

    /// Parse from log line
    pub fn from_log_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 6 || parts[1] != "command" {
            return None;
        }
        Some(Self {
            timestamp: parts[0].parse().ok()?,
            command: parts[2].to_string(),
            args: if parts[3] == "-" { None } else { Some(parts[3].to_string()) },
            exit_code: parts[4].parse().ok(),
            duration_ms: parts[5].parse().ok(),
        })
    }
}

// ============================================================================
// Service Change Events
// ============================================================================

/// A service state change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceChangeEvent {
    /// Unix timestamp
    pub timestamp: u64,
    /// Service unit name
    pub unit: String,
    /// Previous state
    pub from_state: String,
    /// New state
    pub to_state: String,
}

impl ServiceChangeEvent {
    /// Format for log file: timestamp|service|unit|from|to
    pub fn to_log_line(&self) -> String {
        format!(
            "{}|service|{}|{}|{}",
            self.timestamp, self.unit, self.from_state, self.to_state
        )
    }

    /// Parse from log line
    pub fn from_log_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 || parts[1] != "service" {
            return None;
        }
        Some(Self {
            timestamp: parts[0].parse().ok()?,
            unit: parts[2].to_string(),
            from_state: parts[3].to_string(),
            to_state: parts[4].to_string(),
        })
    }
}

// ============================================================================
// Package Change Events
// ============================================================================

/// Package change type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageChangeType {
    Installed,
    Upgraded,
    Removed,
}

impl PackageChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Upgraded => "upgraded",
            Self::Removed => "removed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "installed" => Some(Self::Installed),
            "upgraded" => Some(Self::Upgraded),
            "removed" => Some(Self::Removed),
            _ => None,
        }
    }
}

/// A package change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageChangeEvent {
    /// Unix timestamp
    pub timestamp: u64,
    /// Package name
    pub package: String,
    /// Change type
    pub change_type: PackageChangeType,
    /// Previous version (for upgrades)
    pub from_version: Option<String>,
    /// New version (for installs/upgrades)
    pub to_version: Option<String>,
}

impl PackageChangeEvent {
    /// Format for log file: timestamp|package|name|type|from_ver|to_ver
    pub fn to_log_line(&self) -> String {
        format!(
            "{}|package|{}|{}|{}|{}",
            self.timestamp,
            self.package,
            self.change_type.as_str(),
            self.from_version.as_deref().unwrap_or("-"),
            self.to_version.as_deref().unwrap_or("-")
        )
    }

    /// Parse from log line
    pub fn from_log_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 6 || parts[1] != "package" {
            return None;
        }
        Some(Self {
            timestamp: parts[0].parse().ok()?,
            package: parts[2].to_string(),
            change_type: PackageChangeType::from_str(parts[3])?,
            from_version: if parts[4] == "-" { None } else { Some(parts[4].to_string()) },
            to_version: if parts[5] == "-" { None } else { Some(parts[5].to_string()) },
        })
    }
}

// ============================================================================
// Telemetry State (aggregated stats for quick display)
// ============================================================================

/// Aggregated telemetry state for quick status display
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryState {
    /// Daemon start timestamp
    pub daemon_start_at: u64,
    /// Total process samples collected
    pub total_samples: u64,
    /// Last sample timestamp
    pub last_sample_at: u64,
    /// Total commands observed
    pub total_commands: u64,
    /// Unique commands seen (lifetime)
    pub unique_commands: usize,
    /// Total service changes
    pub total_service_changes: u64,
    /// Total package changes
    pub total_package_changes: u64,
}

impl TelemetryState {
    /// Load from file
    pub fn load() -> Self {
        let path = PathBuf::from(TELEMETRY_STATE_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    /// Save to file
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(TELEMETRY_STATE_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Update daemon start time
    pub fn mark_daemon_start(&mut self) {
        self.daemon_start_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

// ============================================================================
// Telemetry Writer
// ============================================================================

/// Writer for telemetry log files
pub struct TelemetryWriter {
    dir: PathBuf,
}

impl Default for TelemetryWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryWriter {
    /// Create with default directory
    pub fn new() -> Self {
        let dir = PathBuf::from(TELEMETRY_DIR);
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Create with custom directory
    pub fn with_dir(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Append a line to a log file
    fn append(&self, filename: &str, line: &str) -> std::io::Result<()> {
        let path = self.dir.join(filename);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", line)
    }

    /// Record a process sample
    pub fn record_process(&self, sample: &ProcessSample) -> std::io::Result<()> {
        self.append(PROCESS_ACTIVITY_LOG, &sample.to_log_line())
    }

    /// Record a command event
    pub fn record_command(&self, event: &CommandEvent) -> std::io::Result<()> {
        self.append(COMMAND_USAGE_LOG, &event.to_log_line())
    }

    /// Record a service change
    pub fn record_service_change(&self, event: &ServiceChangeEvent) -> std::io::Result<()> {
        self.append(SERVICE_CHANGES_LOG, &event.to_log_line())
    }

    /// Record a package change
    pub fn record_package_change(&self, event: &PackageChangeEvent) -> std::io::Result<()> {
        self.append(PACKAGE_CHANGES_LOG, &event.to_log_line())
    }

    /// Get the log directory
    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }
}

// ============================================================================
// Telemetry Reader
// ============================================================================

/// Reader for telemetry log files
pub struct TelemetryReader {
    dir: PathBuf,
}

impl Default for TelemetryReader {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryReader {
    /// Create with default directory
    pub fn new() -> Self {
        Self {
            dir: PathBuf::from(TELEMETRY_DIR),
        }
    }

    /// Create with custom directory
    pub fn with_dir(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Read lines from a log file within a time window
    fn read_lines_in_window(&self, filename: &str, since: u64) -> Vec<String> {
        let path = self.dir.join(filename);
        if !path.exists() {
            return Vec::new();
        }

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        BufReader::new(file)
            .lines()
            .filter_map(|l| l.ok())
            .filter(|line| {
                // Quick timestamp check (first field before |)
                if let Some(ts_str) = line.split('|').next() {
                    if let Ok(ts) = ts_str.parse::<u64>() {
                        return ts >= since;
                    }
                }
                false
            })
            .collect()
    }

    /// Get process samples in time window
    pub fn process_samples_since(&self, since: u64) -> Vec<ProcessSample> {
        self.read_lines_in_window(PROCESS_ACTIVITY_LOG, since)
            .iter()
            .filter_map(|line| ProcessSample::from_log_line(line))
            .collect()
    }

    /// Get command events in time window
    pub fn command_events_since(&self, since: u64) -> Vec<CommandEvent> {
        self.read_lines_in_window(COMMAND_USAGE_LOG, since)
            .iter()
            .filter_map(|line| CommandEvent::from_log_line(line))
            .collect()
    }

    /// Get service changes in time window
    pub fn service_changes_since(&self, since: u64) -> Vec<ServiceChangeEvent> {
        self.read_lines_in_window(SERVICE_CHANGES_LOG, since)
            .iter()
            .filter_map(|line| ServiceChangeEvent::from_log_line(line))
            .collect()
    }

    /// Get package changes in time window
    pub fn package_changes_since(&self, since: u64) -> Vec<PackageChangeEvent> {
        self.read_lines_in_window(PACKAGE_CHANGES_LOG, since)
            .iter()
            .filter_map(|line| PackageChangeEvent::from_log_line(line))
            .collect()
    }

    /// Count total events in a log file
    pub fn count_events(&self, filename: &str) -> u64 {
        let path = self.dir.join(filename);
        if !path.exists() {
            return 0;
        }

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return 0,
        };

        BufReader::new(file).lines().count() as u64
    }

    /// Check if telemetry directory exists and has data
    pub fn has_data(&self) -> bool {
        if !self.dir.exists() {
            return false;
        }
        // Check if any log file has content
        self.count_events(PROCESS_ACTIVITY_LOG) > 0
            || self.count_events(COMMAND_USAGE_LOG) > 0
            || self.count_events(SERVICE_CHANGES_LOG) > 0
            || self.count_events(PACKAGE_CHANGES_LOG) > 0
    }
}

// ============================================================================
// Time Window Helpers
// ============================================================================

/// Get timestamp for N hours ago
pub fn hours_ago(hours: u64) -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(hours * 3600)
}

/// Get timestamp for N days ago
pub fn days_ago(days: u64) -> u64 {
    hours_ago(days * 24)
}

/// Get current timestamp
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================================
// Usage Statistics
// ============================================================================

/// Per-command usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandStats {
    /// Total executions
    pub count: u64,
    /// First seen timestamp
    pub first_seen: u64,
    /// Last seen timestamp
    pub last_seen: u64,
}

/// Get command usage stats for a time window
pub fn command_stats(since: u64) -> HashMap<String, CommandStats> {
    let reader = TelemetryReader::new();
    let events = reader.command_events_since(since);

    let mut stats: HashMap<String, CommandStats> = HashMap::new();

    for event in events {
        let entry = stats.entry(event.command.clone()).or_default();
        entry.count += 1;
        if entry.first_seen == 0 || event.timestamp < entry.first_seen {
            entry.first_seen = event.timestamp;
        }
        if event.timestamp > entry.last_seen {
            entry.last_seen = event.timestamp;
        }
    }

    stats
}

/// Get top N most used commands in time window
pub fn top_commands(since: u64, limit: usize) -> Vec<(String, u64)> {
    let stats = command_stats(since);
    let mut sorted: Vec<_> = stats.into_iter().map(|(k, v)| (k, v.count)).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(limit);
    sorted
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_process_sample_log_format() {
        let sample = ProcessSample {
            timestamp: 1700000000,
            name: "firefox".to_string(),
            pid: 1234,
            cpu_percent: 15.5,
            mem_bytes: 1024 * 1024 * 512,
        };

        let line = sample.to_log_line();
        assert!(line.starts_with("1700000000|process|firefox|"));

        let parsed = ProcessSample::from_log_line(&line).unwrap();
        assert_eq!(parsed.name, "firefox");
        assert_eq!(parsed.pid, 1234);
    }

    #[test]
    fn test_command_event_log_format() {
        let event = CommandEvent {
            timestamp: 1700000000,
            command: "git".to_string(),
            args: Some("status".to_string()),
            exit_code: Some(0),
            duration_ms: Some(100),
        };

        let line = event.to_log_line();
        let parsed = CommandEvent::from_log_line(&line).unwrap();
        assert_eq!(parsed.command, "git");
        assert_eq!(parsed.args, Some("status".to_string()));
    }

    #[test]
    fn test_package_change_event() {
        let event = PackageChangeEvent {
            timestamp: 1700000000,
            package: "linux".to_string(),
            change_type: PackageChangeType::Upgraded,
            from_version: Some("6.6.1".to_string()),
            to_version: Some("6.6.2".to_string()),
        };

        let line = event.to_log_line();
        let parsed = PackageChangeEvent::from_log_line(&line).unwrap();
        assert_eq!(parsed.package, "linux");
        assert_eq!(parsed.change_type, PackageChangeType::Upgraded);
    }

    #[test]
    fn test_telemetry_writer_reader() {
        let temp_dir = TempDir::new().unwrap();
        let writer = TelemetryWriter::with_dir(temp_dir.path().to_path_buf());
        let reader = TelemetryReader::with_dir(temp_dir.path().to_path_buf());

        // Write some samples
        let sample = ProcessSample {
            timestamp: now(),
            name: "test".to_string(),
            pid: 1,
            cpu_percent: 10.0,
            mem_bytes: 1000,
        };
        writer.record_process(&sample).unwrap();

        // Read back
        let samples = reader.process_samples_since(0);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "test");
    }

    #[test]
    fn test_time_windows() {
        let now_ts = now();
        let one_hour_ago = hours_ago(1);
        let one_day_ago = days_ago(1);

        assert!(one_hour_ago < now_ts);
        assert!(one_day_ago < one_hour_ago);
        assert_eq!(now_ts - one_hour_ago, 3600);
        assert_eq!(now_ts - one_day_ago, 86400);
    }
}
