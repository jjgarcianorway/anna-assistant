//! Log Writer v0.8.0
//!
//! JSONL file writer with rotation support.
//! Handles graceful degradation when logging fails.

use super::{LlmTrace, LogConfig, LogEntry, LogLevel, RequestTrace};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Log writer status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterStatus {
    Healthy,
    Degraded(String),
    Failed(String),
}

/// Thread-safe log writer
pub struct LogWriter {
    config: LogConfig,
    daemon_writer: Arc<Mutex<Option<BufWriter<File>>>>,
    requests_writer: Arc<Mutex<Option<BufWriter<File>>>>,
    llm_writer: Arc<Mutex<Option<BufWriter<File>>>>,
    status: Arc<Mutex<WriterStatus>>,
}

impl LogWriter {
    /// Create a new log writer with the given configuration
    pub fn new(config: LogConfig) -> Self {
        let writer = Self {
            config: config.clone(),
            daemon_writer: Arc::new(Mutex::new(None)),
            requests_writer: Arc::new(Mutex::new(None)),
            llm_writer: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(WriterStatus::Healthy)),
        };

        // Initialize writers
        writer.init_writers();
        writer
    }

    /// Initialize all log file writers
    fn init_writers(&self) {
        // Ensure log directory exists
        if let Err(e) = self.config.ensure_log_dir() {
            self.set_status(WriterStatus::Degraded(format!(
                "Failed to create log directory: {}",
                e
            )));
            return;
        }

        // Initialize daemon writer
        if self.config.daemon_enabled {
            match self.open_log_file(&self.config.daemon_log_path()) {
                Ok(file) => {
                    *self.daemon_writer.lock().unwrap() = Some(BufWriter::new(file));
                }
                Err(e) => {
                    self.set_status(WriterStatus::Degraded(format!(
                        "Failed to open daemon log: {}",
                        e
                    )));
                }
            }
        }

        // Initialize requests writer
        if self.config.requests_enabled {
            match self.open_log_file(&self.config.requests_log_path()) {
                Ok(file) => {
                    *self.requests_writer.lock().unwrap() = Some(BufWriter::new(file));
                }
                Err(e) => {
                    self.set_status(WriterStatus::Degraded(format!(
                        "Failed to open requests log: {}",
                        e
                    )));
                }
            }
        }

        // Initialize LLM writer
        if self.config.llm_enabled {
            match self.open_log_file(&self.config.llm_log_path()) {
                Ok(file) => {
                    *self.llm_writer.lock().unwrap() = Some(BufWriter::new(file));
                }
                Err(e) => {
                    self.set_status(WriterStatus::Degraded(format!(
                        "Failed to open LLM log: {}",
                        e
                    )));
                }
            }
        }
    }

    /// Open a log file (with rotation check)
    fn open_log_file(&self, path: &Path) -> io::Result<File> {
        // Check if rotation is needed
        if path.exists() {
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() >= self.config.max_size {
                    self.rotate_log(path)?;
                }
            }
        }

        OpenOptions::new().create(true).append(true).open(path)
    }

    /// Rotate a log file
    fn rotate_log(&self, path: &Path) -> io::Result<()> {
        let base_name = path.to_string_lossy();

        // Shift existing backups
        for i in (1..self.config.max_backups).rev() {
            let old_path = PathBuf::from(format!("{}.{}", base_name, i));
            let new_path = PathBuf::from(format!("{}.{}", base_name, i + 1));
            if old_path.exists() {
                let _ = fs::rename(&old_path, &new_path);
            }
        }

        // Move current to .1
        let backup_path = PathBuf::from(format!("{}.1", base_name));
        fs::rename(path, backup_path)?;

        Ok(())
    }

    /// Set writer status
    fn set_status(&self, status: WriterStatus) {
        *self.status.lock().unwrap() = status;
    }

    /// Get current status
    pub fn status(&self) -> WriterStatus {
        self.status.lock().unwrap().clone()
    }

    /// Check if logging is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status(), WriterStatus::Healthy)
    }

    /// Write a log entry to the daemon log
    pub fn write_daemon(&self, entry: &LogEntry) {
        if !self.config.daemon_enabled || !self.config.should_log(entry.level) {
            return;
        }

        if let Ok(mut writer) = self.daemon_writer.lock() {
            if let Some(w) = writer.as_mut() {
                let jsonl = entry.to_jsonl();
                if let Err(e) = writeln!(w, "{}", jsonl) {
                    self.set_status(WriterStatus::Degraded(format!(
                        "Daemon log write error: {}",
                        e
                    )));
                }
                let _ = w.flush();
            }
        }
    }

    /// Write a request trace to the requests log
    pub fn write_request(&self, trace: &RequestTrace) {
        if !self.config.requests_enabled {
            return;
        }

        if let Ok(mut writer) = self.requests_writer.lock() {
            if let Some(w) = writer.as_mut() {
                if let Ok(jsonl) = serde_json::to_string(trace) {
                    if let Err(e) = writeln!(w, "{}", jsonl) {
                        self.set_status(WriterStatus::Degraded(format!(
                            "Requests log write error: {}",
                            e
                        )));
                    }
                    let _ = w.flush();
                }
            }
        }
    }

    /// Write an LLM trace to the LLM log
    pub fn write_llm(&self, trace: &LlmTrace) {
        if !self.config.llm_enabled {
            return;
        }

        if let Ok(mut writer) = self.llm_writer.lock() {
            if let Some(w) = writer.as_mut() {
                if let Ok(jsonl) = serde_json::to_string(trace) {
                    if let Err(e) = writeln!(w, "{}", jsonl) {
                        self.set_status(WriterStatus::Degraded(format!(
                            "LLM log write error: {}",
                            e
                        )));
                    }
                    let _ = w.flush();
                }
            }
        }
    }

    /// Convenience method to log at DEBUG level
    pub fn debug(&self, component: super::LogComponent, message: impl Into<String>) {
        self.write_daemon(&LogEntry::new(LogLevel::Debug, component, message));
    }

    /// Convenience method to log at INFO level
    pub fn info(&self, component: super::LogComponent, message: impl Into<String>) {
        self.write_daemon(&LogEntry::new(LogLevel::Info, component, message));
    }

    /// Convenience method to log at WARN level
    pub fn warn(&self, component: super::LogComponent, message: impl Into<String>) {
        self.write_daemon(&LogEntry::new(LogLevel::Warn, component, message));
    }

    /// Convenience method to log at ERROR level
    pub fn error(&self, component: super::LogComponent, message: impl Into<String>) {
        self.write_daemon(&LogEntry::new(LogLevel::Error, component, message));
    }

    /// Flush all writers
    pub fn flush(&self) {
        if let Ok(mut writer) = self.daemon_writer.lock() {
            if let Some(w) = writer.as_mut() {
                let _ = w.flush();
            }
        }
        if let Ok(mut writer) = self.requests_writer.lock() {
            if let Some(w) = writer.as_mut() {
                let _ = w.flush();
            }
        }
        if let Ok(mut writer) = self.llm_writer.lock() {
            if let Some(w) = writer.as_mut() {
                let _ = w.flush();
            }
        }
    }
}

impl Default for LogWriter {
    fn default() -> Self {
        Self::new(LogConfig::default())
    }
}

/// Global logger instance
static GLOBAL_LOGGER: std::sync::OnceLock<LogWriter> = std::sync::OnceLock::new();

/// Initialize the global logger
pub fn init_logger(config: LogConfig) {
    let _ = GLOBAL_LOGGER.set(LogWriter::new(config));
}

/// Get the global logger (initializes with defaults if not set)
pub fn logger() -> &'static LogWriter {
    GLOBAL_LOGGER.get_or_init(LogWriter::default)
}

/// Log a daemon entry using the global logger
pub fn log_daemon(entry: &LogEntry) {
    logger().write_daemon(entry);
}

/// Log a request trace using the global logger
pub fn log_request(trace: &RequestTrace) {
    logger().write_request(trace);
}

/// Log an LLM trace using the global logger
pub fn log_llm(trace: &LlmTrace) {
    logger().write_llm(trace);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_writer_status_default() {
        // Using a temp dir to avoid permission issues
        let temp = tempdir().unwrap();
        let config = LogConfig {
            log_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let writer = LogWriter::new(config);
        assert!(writer.is_healthy());
    }

    #[test]
    fn test_write_daemon_log() {
        let temp = tempdir().unwrap();
        let config = LogConfig {
            log_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let writer = LogWriter::new(config.clone());
        writer.info(super::super::LogComponent::Daemon, "Test message");
        writer.flush();

        // Verify file was created and contains data
        let log_path = config.daemon_log_path();
        assert!(log_path.exists());
        let content = fs::read_to_string(log_path).unwrap();
        assert!(content.contains("Test message"));
    }

    #[test]
    fn test_write_request_trace() {
        let temp = tempdir().unwrap();
        let config = LogConfig {
            log_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        let writer = LogWriter::new(config.clone());

        let trace = RequestTrace {
            request_id: "req-test-0001".to_string(),
            timestamp_start: chrono::Utc::now(),
            timestamp_end: chrono::Utc::now(),
            duration_ms: 100,
            user_query: "test query".to_string(),
            probe_summary: vec!["cpu.info".to_string()],
            self_health_actions: vec![],
            reliability_score: 0.95,
            result_status: super::super::RequestStatus::Ok,
        };

        writer.write_request(&trace);
        writer.flush();

        let log_path = config.requests_log_path();
        assert!(log_path.exists());
        let content = fs::read_to_string(log_path).unwrap();
        assert!(content.contains("req-test-0001"));
    }

    #[test]
    fn test_log_level_filtering() {
        let temp = tempdir().unwrap();
        let config = LogConfig {
            log_dir: temp.path().to_path_buf(),
            level: LogLevel::Warn, // Only WARN and above
            ..Default::default()
        };

        let writer = LogWriter::new(config.clone());

        // DEBUG should not be logged
        writer.debug(super::super::LogComponent::Daemon, "Debug message");
        // WARN should be logged
        writer.warn(super::super::LogComponent::Daemon, "Warning message");
        writer.flush();

        let log_path = config.daemon_log_path();
        let content = fs::read_to_string(log_path).unwrap();
        assert!(!content.contains("Debug message"));
        assert!(content.contains("Warning message"));
    }
}
