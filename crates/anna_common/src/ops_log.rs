//! Anna Operations Log v7.12.0 - Internal Tooling Audit Trail
//!
//! Records Anna's own tool installations and system operations.
//! Lives at /var/lib/anna/internal/ops.log
//!
//! Format: ISO8601 timestamp + action + tool + [package]
//! Example: 2025-12-01T17:05:23Z install pacman arch-wiki-docs
//!
//! Actions:
//! - install: Package installation
//! - remove: Package removal
//! - enable: Service enabled
//! - disable: Service disabled
//! - config: Configuration change

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

/// Internal ops directory
pub const INTERNAL_DIR: &str = "/var/lib/anna/internal";

/// Operations log file
pub const OPS_LOG_FILE: &str = "/var/lib/anna/internal/ops.log";

/// Operations log action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpsAction {
    Install,
    Remove,
    Enable,
    Disable,
    Config,
}

impl OpsAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpsAction::Install => "install",
            OpsAction::Remove => "remove",
            OpsAction::Enable => "enable",
            OpsAction::Disable => "disable",
            OpsAction::Config => "config",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "install" => Some(OpsAction::Install),
            "remove" => Some(OpsAction::Remove),
            "enable" => Some(OpsAction::Enable),
            "disable" => Some(OpsAction::Disable),
            "config" => Some(OpsAction::Config),
            _ => None,
        }
    }
}

/// A single ops log entry
#[derive(Debug, Clone)]
pub struct OpsEntry {
    pub timestamp: DateTime<Utc>,
    pub action: OpsAction,
    pub tool: String,
    pub target: String,
}

impl OpsEntry {
    /// Create a new ops entry with current timestamp
    pub fn new(action: OpsAction, tool: &str, target: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            action,
            tool: tool.to_string(),
            target: target.to_string(),
        }
    }

    /// Format as log line
    pub fn to_log_line(&self) -> String {
        format!(
            "{} {} {} {}",
            self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
            self.action.as_str(),
            self.tool,
            self.target
        )
    }

    /// Parse from log line
    pub fn from_log_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 4 {
            return None;
        }

        let timestamp = DateTime::parse_from_rfc3339(parts[0])
            .ok()?
            .with_timezone(&Utc);
        let action = OpsAction::from_str(parts[1])?;
        let tool = parts[2].to_string();
        let target = parts[3].to_string();

        Some(Self {
            timestamp,
            action,
            tool,
            target,
        })
    }
}

/// Operations log writer
pub struct OpsLogWriter {
    log_path: PathBuf,
}

impl OpsLogWriter {
    /// Create new ops log writer
    pub fn new() -> Self {
        Self {
            log_path: PathBuf::from(OPS_LOG_FILE),
        }
    }

    /// Ensure the internal directory exists
    fn ensure_dir(&self) -> std::io::Result<()> {
        let dir = Path::new(INTERNAL_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// Record an operation
    pub fn record(&self, entry: &OpsEntry) -> std::io::Result<()> {
        self.ensure_dir()?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", entry.to_log_line())?;
        Ok(())
    }

    /// Record a package installation
    pub fn record_install(&self, tool: &str, package: &str) -> std::io::Result<()> {
        let entry = OpsEntry::new(OpsAction::Install, tool, package);
        self.record(&entry)
    }

    /// Record a package removal
    pub fn record_remove(&self, tool: &str, package: &str) -> std::io::Result<()> {
        let entry = OpsEntry::new(OpsAction::Remove, tool, package);
        self.record(&entry)
    }

    /// Record a service enable
    pub fn record_enable(&self, tool: &str, service: &str) -> std::io::Result<()> {
        let entry = OpsEntry::new(OpsAction::Enable, tool, service);
        self.record(&entry)
    }

    /// Record a service disable
    pub fn record_disable(&self, tool: &str, service: &str) -> std::io::Result<()> {
        let entry = OpsEntry::new(OpsAction::Disable, tool, service);
        self.record(&entry)
    }

    /// Record a config change
    pub fn record_config(&self, tool: &str, config: &str) -> std::io::Result<()> {
        let entry = OpsEntry::new(OpsAction::Config, tool, config);
        self.record(&entry)
    }
}

impl Default for OpsLogWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Operations log reader
pub struct OpsLogReader {
    log_path: PathBuf,
}

impl OpsLogReader {
    /// Create new ops log reader
    pub fn new() -> Self {
        Self {
            log_path: PathBuf::from(OPS_LOG_FILE),
        }
    }

    /// Check if ops log exists
    pub fn exists(&self) -> bool {
        self.log_path.exists()
    }

    /// Read all entries
    pub fn read_all(&self) -> Vec<OpsEntry> {
        if !self.exists() {
            return Vec::new();
        }

        let file = match File::open(&self.log_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| OpsEntry::from_log_line(&line))
            .collect()
    }

    /// Read recent entries (last N)
    pub fn read_recent(&self, count: usize) -> Vec<OpsEntry> {
        let all = self.read_all();
        let len = all.len();
        if len <= count {
            all
        } else {
            all.into_iter().skip(len - count).collect()
        }
    }

    /// Read entries for a specific action type
    pub fn read_by_action(&self, action: OpsAction) -> Vec<OpsEntry> {
        self.read_all()
            .into_iter()
            .filter(|e| e.action == action)
            .collect()
    }

    /// Read entries for a specific tool
    pub fn read_by_tool(&self, tool: &str) -> Vec<OpsEntry> {
        self.read_all()
            .into_iter()
            .filter(|e| e.tool == tool)
            .collect()
    }

    /// Count entries by action type
    pub fn count_by_action(&self) -> OpsActionCounts {
        let entries = self.read_all();
        OpsActionCounts {
            install: entries.iter().filter(|e| e.action == OpsAction::Install).count(),
            remove: entries.iter().filter(|e| e.action == OpsAction::Remove).count(),
            enable: entries.iter().filter(|e| e.action == OpsAction::Enable).count(),
            disable: entries.iter().filter(|e| e.action == OpsAction::Disable).count(),
            config: entries.iter().filter(|e| e.action == OpsAction::Config).count(),
        }
    }

    /// Get summary for display
    pub fn get_summary(&self) -> OpsLogSummary {
        let entries = self.read_all();
        let counts = OpsActionCounts {
            install: entries.iter().filter(|e| e.action == OpsAction::Install).count(),
            remove: entries.iter().filter(|e| e.action == OpsAction::Remove).count(),
            enable: entries.iter().filter(|e| e.action == OpsAction::Enable).count(),
            disable: entries.iter().filter(|e| e.action == OpsAction::Disable).count(),
            config: entries.iter().filter(|e| e.action == OpsAction::Config).count(),
        };

        let first_entry = entries.first().map(|e| e.timestamp);
        let last_entry = entries.last().map(|e| e.timestamp);

        OpsLogSummary {
            total_entries: entries.len(),
            counts,
            first_entry,
            last_entry,
        }
    }
}

impl Default for OpsLogReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Action counts summary
#[derive(Debug, Clone, Default)]
pub struct OpsActionCounts {
    pub install: usize,
    pub remove: usize,
    pub enable: usize,
    pub disable: usize,
    pub config: usize,
}

impl OpsActionCounts {
    pub fn total(&self) -> usize {
        self.install + self.remove + self.enable + self.disable + self.config
    }
}

/// Ops log summary for display
#[derive(Debug, Clone)]
pub struct OpsLogSummary {
    pub total_entries: usize,
    pub counts: OpsActionCounts,
    pub first_entry: Option<DateTime<Utc>>,
    pub last_entry: Option<DateTime<Utc>>,
}

impl OpsLogSummary {
    /// Format for single-line display
    pub fn format_compact(&self) -> String {
        if self.total_entries == 0 {
            return "no operations recorded".to_string();
        }

        let mut parts = Vec::new();
        if self.counts.install > 0 {
            parts.push(format!("{} installs", self.counts.install));
        }
        if self.counts.remove > 0 {
            parts.push(format!("{} removes", self.counts.remove));
        }
        if self.counts.enable > 0 {
            parts.push(format!("{} enables", self.counts.enable));
        }
        if self.counts.disable > 0 {
            parts.push(format!("{} disables", self.counts.disable));
        }
        if self.counts.config > 0 {
            parts.push(format!("{} configs", self.counts.config));
        }

        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_roundtrip() {
        let entry = OpsEntry::new(OpsAction::Install, "pacman", "arch-wiki-docs");
        let line = entry.to_log_line();
        let parsed = OpsEntry::from_log_line(&line).unwrap();

        assert_eq!(parsed.action, OpsAction::Install);
        assert_eq!(parsed.tool, "pacman");
        assert_eq!(parsed.target, "arch-wiki-docs");
    }

    #[test]
    fn test_action_roundtrip() {
        for action in [
            OpsAction::Install,
            OpsAction::Remove,
            OpsAction::Enable,
            OpsAction::Disable,
            OpsAction::Config,
        ] {
            let s = action.as_str();
            let parsed = OpsAction::from_str(s).unwrap();
            assert_eq!(parsed, action);
        }
    }

    #[test]
    fn test_parse_sample_line() {
        let line = "2025-12-01T17:05:23Z install pacman arch-wiki-docs";
        let entry = OpsEntry::from_log_line(line).unwrap();

        assert_eq!(entry.action, OpsAction::Install);
        assert_eq!(entry.tool, "pacman");
        assert_eq!(entry.target, "arch-wiki-docs");
    }

    #[test]
    fn test_summary_format() {
        let summary = OpsLogSummary {
            total_entries: 5,
            counts: OpsActionCounts {
                install: 3,
                remove: 1,
                enable: 0,
                disable: 0,
                config: 1,
            },
            first_entry: None,
            last_entry: None,
        };

        let compact = summary.format_compact();
        assert!(compact.contains("3 installs"));
        assert!(compact.contains("1 removes"));
        assert!(compact.contains("1 configs"));
        assert!(!compact.contains("enables"));
    }

    #[test]
    fn test_empty_summary() {
        let summary = OpsLogSummary {
            total_entries: 0,
            counts: OpsActionCounts::default(),
            first_entry: None,
            last_entry: None,
        };

        assert_eq!(summary.format_compact(), "no operations recorded");
    }
}
