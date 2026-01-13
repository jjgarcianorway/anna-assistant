//! Error and warning summary tracking.

use serde::{Deserialize, Serialize};

/// v0.3.21: Recent error/warning summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// Total error count since daemon start
    pub error_count: u64,
    /// Total warning count since daemon start
    pub warning_count: u64,
    /// Most recent errors (last 5)
    pub recent_errors: Vec<ErrorEntry>,
    /// Most recent warnings (last 5)
    pub recent_warnings: Vec<ErrorEntry>,
}

/// v0.3.21: Single error/warning entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Source component
    pub source: Option<String>,
    /// Timestamp (RFC3339)
    pub timestamp: String,
    /// Whether error is recoverable
    pub recoverable: bool,
}

impl ErrorSummary {
    /// Add an error
    pub fn add_error(
        &mut self,
        code: &str,
        message: &str,
        source: Option<&str>,
        recoverable: bool,
    ) {
        self.error_count += 1;
        let entry = ErrorEntry {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            recoverable,
        };
        self.recent_errors.push(entry);
        // Keep only last 5
        if self.recent_errors.len() > 5 {
            self.recent_errors.remove(0);
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, code: &str, message: &str, source: Option<&str>) {
        self.warning_count += 1;
        let entry = ErrorEntry {
            code: code.to_string(),
            message: message.to_string(),
            source: source.map(|s| s.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            recoverable: true,
        };
        self.recent_warnings.push(entry);
        // Keep only last 5
        if self.recent_warnings.len() > 5 {
            self.recent_warnings.remove(0);
        }
    }
}
