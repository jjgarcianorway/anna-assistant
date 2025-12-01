//! Log Severity Types

use serde::{Deserialize, Serialize};

/// Log entry severity level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSeverity {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

impl LogSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogSeverity::Debug => "debug",
            LogSeverity::Info => "info",
            LogSeverity::Notice => "notice",
            LogSeverity::Warning => "warning",
            LogSeverity::Error => "error",
            LogSeverity::Critical => "critical",
            LogSeverity::Alert => "alert",
            LogSeverity::Emergency => "emergency",
        }
    }

    /// Parse from journalctl priority (0-7)
    pub fn from_priority(priority: u8) -> Self {
        match priority {
            0 => LogSeverity::Emergency,
            1 => LogSeverity::Alert,
            2 => LogSeverity::Critical,
            3 => LogSeverity::Error,
            4 => LogSeverity::Warning,
            5 => LogSeverity::Notice,
            6 => LogSeverity::Info,
            7 => LogSeverity::Debug,
            _ => LogSeverity::Info,
        }
    }

    /// Is this severity an error or worse?
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            LogSeverity::Error
                | LogSeverity::Critical
                | LogSeverity::Alert
                | LogSeverity::Emergency
        )
    }

    /// Is this severity a warning or worse?
    pub fn is_warning_or_worse(&self) -> bool {
        matches!(
            self,
            LogSeverity::Warning
                | LogSeverity::Error
                | LogSeverity::Critical
                | LogSeverity::Alert
                | LogSeverity::Emergency
        )
    }

    /// Color code for terminal display
    pub fn color_code(&self) -> &'static str {
        match self {
            LogSeverity::Debug => "dim",
            LogSeverity::Info => "default",
            LogSeverity::Notice => "cyan",
            LogSeverity::Warning => "yellow",
            LogSeverity::Error => "red",
            LogSeverity::Critical => "red_bold",
            LogSeverity::Alert => "red_bold",
            LogSeverity::Emergency => "red_bold_blink",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(LogSeverity::Error > LogSeverity::Warning);
        assert!(LogSeverity::Critical > LogSeverity::Error);
        assert!(LogSeverity::Warning > LogSeverity::Info);
    }

    #[test]
    fn test_severity_is_error() {
        assert!(LogSeverity::Error.is_error());
        assert!(LogSeverity::Critical.is_error());
        assert!(!LogSeverity::Warning.is_error());
        assert!(!LogSeverity::Info.is_error());
    }
}
