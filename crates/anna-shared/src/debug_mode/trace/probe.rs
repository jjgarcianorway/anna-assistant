//! Probe trace information.
//!
//! Captures details about probe execution for debugging.

use crate::debug_mode::redact::Redactor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Probe trace info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeTrace {
    /// Probe ID
    pub id: String,
    /// Command (redacted if level < 3)
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Duration in ms
    pub duration_ms: u64,
    /// Parsed key-value results (level >= 2)
    pub parsed: HashMap<String, String>,
    /// Raw stdout (level 3 only, redacted)
    pub raw_stdout: Option<String>,
    /// Raw stderr (level 3 only, redacted)
    pub raw_stderr: Option<String>,
}

impl ProbeTrace {
    pub fn new(id: &str, command: &str, exit_code: i32, duration_ms: u64) -> Self {
        Self {
            id: id.to_string(),
            command: command.to_string(),
            exit_code,
            duration_ms,
            parsed: HashMap::new(),
            raw_stdout: None,
            raw_stderr: None,
        }
    }

    /// Add parsed key-value.
    pub fn add_parsed(&mut self, key: &str, value: &str) {
        self.parsed.insert(key.to_string(), value.to_string());
    }

    /// Set raw output (will be redacted).
    pub fn with_raw(mut self, stdout: &str, stderr: &str, redactor: &Redactor) -> Self {
        self.raw_stdout = Some(redactor.redact(stdout));
        if !stderr.is_empty() {
            self.raw_stderr = Some(redactor.redact(stderr));
        }
        self
    }
}
