//! Logging Configuration v0.8.0
//!
//! Configuration for Anna's logging subsystem.
//! Configurable via natural language through annactl.

use super::LogLevel;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default log directory under user's data directory
pub fn default_log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/var/log"))
        .join("anna")
        .join("logs")
}

/// Maximum log file size in bytes (default 10MB)
pub const DEFAULT_MAX_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of backup files (default 5)
pub const DEFAULT_MAX_BACKUPS: u32 = 5;

/// Truncation limit for single fields (4KB)
pub const TRUNCATION_LIMIT: usize = 4096;

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Global log level
    #[serde(default)]
    pub level: LogLevel,

    /// Enable daemon logging
    #[serde(default = "default_true")]
    pub daemon_enabled: bool,

    /// Enable request logging
    #[serde(default = "default_true")]
    pub requests_enabled: bool,

    /// Enable LLM orchestration logging
    #[serde(default = "default_true")]
    pub llm_enabled: bool,

    /// Log directory path
    #[serde(default = "default_log_dir")]
    pub log_dir: PathBuf,

    /// Maximum log file size in bytes
    #[serde(default = "default_max_size")]
    pub max_size: u64,

    /// Maximum number of backup files
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,

    /// Enable systemd journal logging (for annad)
    #[serde(default = "default_true")]
    pub journal_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_max_size() -> u64 {
    DEFAULT_MAX_SIZE
}

fn default_max_backups() -> u32 {
    DEFAULT_MAX_BACKUPS
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Debug, // v0.8.0: DEBUG by default for development
            daemon_enabled: true,
            requests_enabled: true,
            llm_enabled: true,
            log_dir: default_log_dir(),
            max_size: DEFAULT_MAX_SIZE,
            max_backups: DEFAULT_MAX_BACKUPS,
            journal_enabled: true,
        }
    }
}

impl LogConfig {
    /// Check if a log level should be emitted
    pub fn should_log(&self, level: LogLevel) -> bool {
        level >= self.level
    }

    /// Get path for daemon log file
    pub fn daemon_log_path(&self) -> PathBuf {
        self.log_dir.join("anna-daemon.log")
    }

    /// Get path for requests log file
    pub fn requests_log_path(&self) -> PathBuf {
        self.log_dir.join("anna-requests.log")
    }

    /// Get path for LLM log file
    pub fn llm_log_path(&self) -> PathBuf {
        self.log_dir.join("anna-llm.log")
    }

    /// Ensure log directory exists
    pub fn ensure_log_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.log_dir)
    }

    /// Format config for display
    pub fn format_display(&self) -> String {
        format!(
            r#"[LOG CONFIGURATION]
  level: {}
  daemon_enabled: {}
  requests_enabled: {}
  llm_enabled: {}
  log_dir: {}
  max_size: {} bytes
  max_backups: {}
  journal_enabled: {}"#,
            self.level.as_str(),
            self.daemon_enabled,
            self.requests_enabled,
            self.llm_enabled,
            self.log_dir.display(),
            self.max_size,
            self.max_backups,
            self.journal_enabled
        )
    }
}

/// Log configuration intent for natural language parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogConfigIntent {
    /// Set global log level
    SetLevel(LogLevel),
    /// Enable/disable daemon logging
    SetDaemonEnabled(bool),
    /// Enable/disable request logging
    SetRequestsEnabled(bool),
    /// Enable/disable LLM logging
    SetLlmEnabled(bool),
    /// Show current log configuration
    ShowConfig,
    /// Not a log configuration request
    NotLogConfig,
}

/// Parse natural language log configuration requests
pub fn parse_log_config_intent(input: &str) -> LogConfigIntent {
    let lower = input.to_lowercase();

    // Show config
    if (lower.contains("show") || lower.contains("display") || lower.contains("what"))
        && (lower.contains("log") || lower.contains("logging"))
        && (lower.contains("config") || lower.contains("setting") || lower.contains("level"))
    {
        return LogConfigIntent::ShowConfig;
    }

    // Set level
    if lower.contains("set") && (lower.contains("log") || lower.contains("logging"))
        && lower.contains("level")
    {
        if lower.contains("trace") {
            return LogConfigIntent::SetLevel(LogLevel::Trace);
        }
        if lower.contains("debug") {
            return LogConfigIntent::SetLevel(LogLevel::Debug);
        }
        if lower.contains("info") {
            return LogConfigIntent::SetLevel(LogLevel::Info);
        }
        if lower.contains("warn") {
            return LogConfigIntent::SetLevel(LogLevel::Warn);
        }
        if lower.contains("error") {
            return LogConfigIntent::SetLevel(LogLevel::Error);
        }
    }

    // Enable/disable LLM logging
    if lower.contains("llm") && lower.contains("log") {
        if lower.contains("enable") || lower.contains("detailed") || lower.contains("turn on") {
            return LogConfigIntent::SetLlmEnabled(true);
        }
        if lower.contains("disable") || lower.contains("turn off") {
            return LogConfigIntent::SetLlmEnabled(false);
        }
    }

    // Enable/disable daemon logging
    if lower.contains("daemon") && lower.contains("log") {
        if lower.contains("enable") || lower.contains("turn on") {
            return LogConfigIntent::SetDaemonEnabled(true);
        }
        if lower.contains("disable") || lower.contains("turn off") {
            return LogConfigIntent::SetDaemonEnabled(false);
        }
    }

    // Enable/disable request logging
    if lower.contains("request") && lower.contains("log") {
        if lower.contains("enable") || lower.contains("turn on") {
            return LogConfigIntent::SetRequestsEnabled(true);
        }
        if lower.contains("disable") || lower.contains("turn off") {
            return LogConfigIntent::SetRequestsEnabled(false);
        }
    }

    LogConfigIntent::NotLogConfig
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Debug);
        assert!(config.daemon_enabled);
        assert!(config.requests_enabled);
        assert!(config.llm_enabled);
    }

    #[test]
    fn test_should_log() {
        let mut config = LogConfig::default();
        config.level = LogLevel::Info;

        assert!(!config.should_log(LogLevel::Debug));
        assert!(config.should_log(LogLevel::Info));
        assert!(config.should_log(LogLevel::Warn));
        assert!(config.should_log(LogLevel::Error));
    }

    #[test]
    fn test_log_paths() {
        let config = LogConfig::default();
        assert!(config.daemon_log_path().to_string_lossy().contains("anna-daemon.log"));
        assert!(config.requests_log_path().to_string_lossy().contains("anna-requests.log"));
        assert!(config.llm_log_path().to_string_lossy().contains("anna-llm.log"));
    }

    #[test]
    fn test_parse_set_level() {
        assert_eq!(
            parse_log_config_intent("set logging level to info"),
            LogConfigIntent::SetLevel(LogLevel::Info)
        );
        assert_eq!(
            parse_log_config_intent("set log level to debug"),
            LogConfigIntent::SetLevel(LogLevel::Debug)
        );
    }

    #[test]
    fn test_parse_show_config() {
        assert_eq!(
            parse_log_config_intent("show current logging configuration"),
            LogConfigIntent::ShowConfig
        );
        assert_eq!(
            parse_log_config_intent("what is the log level"),
            LogConfigIntent::ShowConfig
        );
    }

    #[test]
    fn test_parse_enable_llm_logging() {
        assert_eq!(
            parse_log_config_intent("enable detailed llm logging"),
            LogConfigIntent::SetLlmEnabled(true)
        );
        assert_eq!(
            parse_log_config_intent("disable llm logging"),
            LogConfigIntent::SetLlmEnabled(false)
        );
    }

    #[test]
    fn test_parse_not_log_config() {
        assert_eq!(
            parse_log_config_intent("how many CPU cores do I have"),
            LogConfigIntent::NotLogConfig
        );
    }
}
