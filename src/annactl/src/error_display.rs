//! Beautiful error display for Anna CLI
//!
//! Provides color-coded, user-friendly error messages with actionable suggestions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{self, Write};
use std::time::Duration;

/// RPC error code (mirrored from annad)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum RpcErrorCode {
    // Connection errors (1000-1099)
    ConnectionRefused = 1000,
    ConnectionTimeout = 1001,
    SocketNotFound = 1002,
    PermissionDenied = 1003,
    ConnectionReset = 1004,
    ConnectionClosed = 1005,
    IoError = 1006,

    // Request errors (2000-2099)
    InvalidRequest = 2000,
    MalformedJson = 2001,
    MissingParameter = 2002,
    InvalidParameter = 2003,
    UnknownMethod = 2004,
    InvalidAutonomyLevel = 2005,
    InvalidDomain = 2006,
    InvalidTimeRange = 2007,

    // Server errors (3000-3099)
    InternalError = 3000,
    DatabaseError = 3001,
    CollectionFailed = 3002,
    Timeout = 3003,
    ConfigParseError = 3004,
    ConfigReloadError = 3005,
    CommandExecutionError = 3006,
    ParseError = 3007,
    StorageError = 3008,
    EventProcessingError = 3009,
    PolicyError = 3010,

    // Resource errors (4000-4099)
    ResourceNotFound = 4000,
    ResourceBusy = 4001,
    QuotaExceeded = 4002,
    InsufficientPermissions = 4003,
    ConfigNotFound = 4004,
    StorageUnavailable = 4005,
}

/// Error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
}

/// Structured RPC error for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: RpcErrorCode,
    pub context: Option<String>,
    pub timestamp: Option<String>,
}

impl RpcError {
    pub fn new(code: RpcErrorCode) -> Self {
        Self {
            code,
            context: None,
            timestamp: None,
        }
    }

    pub fn with_context<S: Into<String>>(mut self, context: S) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn message(&self) -> &'static str {
        match self.code {
            // Connection errors
            RpcErrorCode::ConnectionRefused => "Connection refused - daemon may not be running",
            RpcErrorCode::ConnectionTimeout => "Connection timed out - daemon not responding",
            RpcErrorCode::SocketNotFound => "Socket file not found at expected path",
            RpcErrorCode::PermissionDenied => "Permission denied - check socket permissions",
            RpcErrorCode::ConnectionReset => "Connection reset by daemon",
            RpcErrorCode::ConnectionClosed => "Connection closed unexpectedly",
            RpcErrorCode::IoError => "Network I/O error occurred",

            // Request errors
            RpcErrorCode::InvalidRequest => "Invalid JSON-RPC request format",
            RpcErrorCode::MalformedJson => "Malformed JSON in request body",
            RpcErrorCode::MissingParameter => "Missing required parameter",
            RpcErrorCode::InvalidParameter => "Invalid parameter value or type",
            RpcErrorCode::UnknownMethod => "Unknown RPC method requested",
            RpcErrorCode::InvalidAutonomyLevel => "Invalid autonomy level (must be 'low' or 'high')",
            RpcErrorCode::InvalidDomain => "Invalid domain specified",
            RpcErrorCode::InvalidTimeRange => "Invalid time range or duration",

            // Server errors
            RpcErrorCode::InternalError => "Internal server error occurred",
            RpcErrorCode::DatabaseError => "Database operation failed",
            RpcErrorCode::CollectionFailed => "Failed to collect system metrics",
            RpcErrorCode::Timeout => "Operation timed out on server",
            RpcErrorCode::ConfigParseError => "Failed to parse configuration file",
            RpcErrorCode::ConfigReloadError => "Failed to reload configuration",
            RpcErrorCode::CommandExecutionError => "Failed to execute system command",
            RpcErrorCode::ParseError => "Failed to parse command output",
            RpcErrorCode::StorageError => "Storage operation failed",
            RpcErrorCode::EventProcessingError => "Event processing failed",
            RpcErrorCode::PolicyError => "Policy evaluation failed",

            // Resource errors
            RpcErrorCode::ResourceNotFound => "Requested resource not found",
            RpcErrorCode::ResourceBusy => "Resource is currently busy or locked",
            RpcErrorCode::QuotaExceeded => "Quota or limit exceeded",
            RpcErrorCode::InsufficientPermissions => "Insufficient permissions for operation",
            RpcErrorCode::ConfigNotFound => "Configuration file not found",
            RpcErrorCode::StorageUnavailable => "Storage not available or unmounted",
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self.code {
            RpcErrorCode::ConnectionRefused | RpcErrorCode::ConnectionTimeout
            | RpcErrorCode::ConnectionReset | RpcErrorCode::ConnectionClosed
            | RpcErrorCode::ResourceBusy | RpcErrorCode::StorageUnavailable
            | RpcErrorCode::Timeout => ErrorSeverity::Warning,

            RpcErrorCode::InternalError | RpcErrorCode::DatabaseError => ErrorSeverity::Critical,

            _ => ErrorSeverity::Error,
        }
    }

    pub fn help_text(&self) -> Vec<&'static str> {
        match self.code {
            RpcErrorCode::ConnectionRefused | RpcErrorCode::ConnectionTimeout => vec![
                "Check daemon status: sudo systemctl status annad",
                "Check daemon logs: sudo journalctl -u annad -n 20",
                "Restart daemon: sudo systemctl restart annad",
                "Verify socket exists: ls -la /run/anna/rpc.sock",
            ],
            RpcErrorCode::SocketNotFound => vec![
                "Verify daemon is running: sudo systemctl status annad",
                "Check socket directory: ls -la /run/anna/",
                "Restart daemon: sudo systemctl restart annad",
            ],
            RpcErrorCode::PermissionDenied => vec![
                "Check socket permissions: ls -la /run/anna/rpc.sock",
                "Verify your user is in the 'anna' group: groups",
                "Add user to group: sudo usermod -aG anna $USER",
                "Log out and back in for group changes to take effect",
            ],
            RpcErrorCode::ConfigParseError | RpcErrorCode::ConfigReloadError => vec![
                "Validate config syntax: annactl config validate",
                "Check config file: cat /etc/anna/config.toml",
                "Review recent changes to configuration",
                "Restore from backup if needed",
            ],
            RpcErrorCode::DatabaseError => vec![
                "Check database integrity: annactl doctor check",
                "Review daemon logs for details: sudo journalctl -u annad -n 50",
                "Database file: /var/lib/anna/telemetry.db",
            ],
            RpcErrorCode::StorageError | RpcErrorCode::StorageUnavailable => vec![
                "Check filesystem status: annactl storage btrfs",
                "Verify filesystem mounted: mount | grep btrfs",
                "Check disk space: df -h",
            ],
            RpcErrorCode::InvalidParameter | RpcErrorCode::MissingParameter => vec![
                "Review command syntax: annactl <command> --help",
                "Check parameter values and types",
                "Refer to documentation for examples",
            ],
            _ => vec![
                "Review daemon logs: sudo journalctl -u annad -n 20",
                "Check system health: annactl doctor check",
                "Report persistent issues to maintainers",
            ],
        }
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code as u32, self.message())?;
        if let Some(ref ctx) = self.context {
            write!(f, ": {}", ctx)?;
        }
        Ok(())
    }
}

/// Retry attempt information
pub struct RetryInfo {
    pub attempt: u32,
    pub max_attempts: u32,
    pub elapsed: Duration,
    pub next_delay: Option<Duration>,
}

/// Display error with beautiful formatting
pub fn display_error(error: &RpcError, retry_info: Option<&RetryInfo>) {
    let mut stdout = io::stdout();
    let severity = error.severity();

    // Color codes
    let (color_code, severity_symbol) = match severity {
        ErrorSeverity::Warning => ("\x1b[33m", "⚠"),  // Yellow
        ErrorSeverity::Error => ("\x1b[31m", "✗"),    // Red
        ErrorSeverity::Critical => ("\x1b[35m", "🔥"), // Magenta
    };
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let dim = "\x1b[2m";

    // Header
    writeln!(stdout, "{}╭─ RPC Error ──────────────────────────────────────────────{}", dim, reset).ok();
    writeln!(stdout, "{}│{}", dim, reset).ok();

    // Error code and severity
    writeln!(
        stdout,
        "{}│{}  {}Error Code:{} {}{}{}  {}({}){}",
        dim, reset, bold, reset, color_code, error.code as u32, reset,
        dim, error.code.code_name(), reset
    ).ok();
    writeln!(
        stdout,
        "{}│{}  {}Severity:{} {}  {}{} {}{}",
        dim, reset, bold, reset, color_code, severity_symbol, format!("{:?}", severity), reset, dim
    ).ok();

    // Retry information if available
    if let Some(retry) = retry_info {
        writeln!(
            stdout,
            "{}│{}  {}Retryable:{}   Yes",
            dim, reset, bold, reset
        ).ok();
        writeln!(
            stdout,
            "{}│{}  {}Attempts:{}    {}/{}",
            dim, reset, bold, reset, retry.attempt + 1, retry.max_attempts
        ).ok();
        writeln!(
            stdout,
            "{}│{}  {}Total Time:{}  {:?}",
            dim, reset, bold, reset, retry.elapsed
        ).ok();
    }

    writeln!(stdout, "{}│{}", dim, reset).ok();

    // Message
    writeln!(stdout, "{}│{}  {}Message:{}", dim, reset, bold, reset).ok();
    writeln!(stdout, "{}│{}  {}", dim, reset, error.message()).ok();

    // Context if available
    if let Some(ref ctx) = error.context {
        writeln!(stdout, "{}│{}", dim, reset).ok();
        writeln!(stdout, "{}│{}  {}Context:{}", dim, reset, bold, reset).ok();
        writeln!(stdout, "{}│{}  {}", dim, reset, ctx).ok();
    }

    // Help text
    let help = error.help_text();
    if !help.is_empty() {
        writeln!(stdout, "{}│{}", dim, reset).ok();
        writeln!(stdout, "{}│{}  {}Suggested Actions:{}", dim, reset, bold, reset).ok();
        for (i, suggestion) in help.iter().enumerate() {
            writeln!(stdout, "{}│{}  {}. {}", dim, reset, i + 1, suggestion).ok();
        }
    }

    writeln!(stdout, "{}│{}", dim, reset).ok();
    writeln!(stdout, "{}╰──────────────────────────────────────────────────────────{}", dim, reset).ok();
    stdout.flush().ok();
}

/// Display retry progress
pub fn display_retry_attempt(attempt: u32, max_attempts: u32, delay: Duration) {
    let mut stdout = io::stdout();
    let dim = "\x1b[2m";
    let yellow = "\x1b[33m";
    let reset = "\x1b[0m";

    write!(
        stdout,
        "{}⏳ Retry {}/{} in {:?}...{}",
        yellow, attempt + 1, max_attempts, delay, reset
    ).ok();
    stdout.flush().ok();
}

/// Display retry success
pub fn display_retry_success(attempts: u32, total_time: Duration) {
    let mut stdout = io::stdout();
    let green = "\x1b[32m";
    let reset = "\x1b[0m";

    writeln!(
        stdout,
        "\r{}✓ Success after {} attempt(s) in {:?}{}",
        green, attempts, total_time, reset
    ).ok();
}

/// Display retry failure
pub fn display_retry_exhausted(attempts: u32, total_time: Duration) {
    let mut stdout = io::stdout();
    let red = "\x1b[31m";
    let reset = "\x1b[0m";

    writeln!(
        stdout,
        "\n{}✗ All retry attempts exhausted ({} attempts, {:?} total){}",
        red, attempts, total_time, reset
    ).ok();
}

impl RpcErrorCode {
    fn code_name(&self) -> &'static str {
        match self {
            Self::ConnectionRefused => "ConnectionRefused",
            Self::ConnectionTimeout => "ConnectionTimeout",
            Self::SocketNotFound => "SocketNotFound",
            Self::PermissionDenied => "PermissionDenied",
            Self::ConnectionReset => "ConnectionReset",
            Self::ConnectionClosed => "ConnectionClosed",
            Self::IoError => "IoError",
            Self::InvalidRequest => "InvalidRequest",
            Self::MalformedJson => "MalformedJson",
            Self::MissingParameter => "MissingParameter",
            Self::InvalidParameter => "InvalidParameter",
            Self::UnknownMethod => "UnknownMethod",
            Self::InvalidAutonomyLevel => "InvalidAutonomyLevel",
            Self::InvalidDomain => "InvalidDomain",
            Self::InvalidTimeRange => "InvalidTimeRange",
            Self::InternalError => "InternalError",
            Self::DatabaseError => "DatabaseError",
            Self::CollectionFailed => "CollectionFailed",
            Self::Timeout => "Timeout",
            Self::ConfigParseError => "ConfigParseError",
            Self::ConfigReloadError => "ConfigReloadError",
            Self::CommandExecutionError => "CommandExecutionError",
            Self::ParseError => "ParseError",
            Self::StorageError => "StorageError",
            Self::EventProcessingError => "EventProcessingError",
            Self::PolicyError => "PolicyError",
            Self::ResourceNotFound => "ResourceNotFound",
            Self::ResourceBusy => "ResourceBusy",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::InsufficientPermissions => "InsufficientPermissions",
            Self::ConfigNotFound => "ConfigNotFound",
            Self::StorageUnavailable => "StorageUnavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_format() {
        let error = RpcError::new(RpcErrorCode::ConnectionTimeout)
            .with_context("Timeout after 2 seconds");

        // Just verify it doesn't panic
        let formatted = format!("{}", error);
        assert!(formatted.contains("1001"));
        assert!(formatted.contains("Connection timed out"));
    }

    #[test]
    fn test_severity_classification() {
        assert_eq!(
            RpcError::new(RpcErrorCode::ConnectionTimeout).severity(),
            ErrorSeverity::Warning
        );
        assert_eq!(
            RpcError::new(RpcErrorCode::InternalError).severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            RpcError::new(RpcErrorCode::InvalidParameter).severity(),
            ErrorSeverity::Error
        );
    }

    #[test]
    fn test_help_text_available() {
        let error = RpcError::new(RpcErrorCode::PermissionDenied);
        let help = error.help_text();
        assert!(!help.is_empty());
        assert!(help.iter().any(|s| s.contains("anna")));
    }
}
