//! Structured RPC error codes and retry logic for Anna Assistant
//!
//! This module provides a comprehensive error taxonomy for RPC failures,
//! enabling intelligent retry logic and user-friendly error messages.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// RPC error code taxonomy
///
/// Errors are organized into ranges:
/// - 1000-1099: Connection errors (retryable)
/// - 2000-2099: Client/request errors (not retryable)
/// - 3000-3099: Server errors (retryable with caution)
/// - 4000-4099: Resource errors (conditionally retryable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum RpcErrorCode {
    // === Connection Errors (1000-1099) ===
    /// Connection refused - daemon not running or socket not available
    ConnectionRefused = 1000,

    /// Connection timeout - daemon not responding within timeout period
    ConnectionTimeout = 1001,

    /// Socket file not found at expected path
    SocketNotFound = 1002,

    /// Permission denied accessing socket
    PermissionDenied = 1003,

    /// Connection reset by peer during communication
    ConnectionReset = 1004,

    /// Connection closed unexpectedly
    ConnectionClosed = 1005,

    /// Network I/O error
    IoError = 1006,

    // === Request Errors (2000-2099) ===
    /// Invalid JSON-RPC request format
    InvalidRequest = 2000,

    /// Malformed JSON in request body
    MalformedJson = 2001,

    /// Missing required parameter
    MissingParameter = 2002,

    /// Invalid parameter value or type
    InvalidParameter = 2003,

    /// Unknown RPC method requested
    UnknownMethod = 2004,

    /// Invalid autonomy level specified
    InvalidAutonomyLevel = 2005,

    /// Invalid domain specified
    InvalidDomain = 2006,

    /// Invalid time range or duration
    InvalidTimeRange = 2007,

    // === Server Errors (3000-3099) ===
    /// Internal server error (unspecified)
    InternalError = 3000,

    /// Database operation failed
    DatabaseError = 3001,

    /// Data collection failed
    CollectionFailed = 3002,

    /// Operation timed out on server side
    Timeout = 3003,

    /// Failed to parse configuration file
    ConfigParseError = 3004,

    /// Failed to reload configuration
    ConfigReloadError = 3005,

    /// Failed to execute system command
    CommandExecutionError = 3006,

    /// Failed to parse command output
    ParseError = 3007,

    /// Storage operation failed
    StorageError = 3008,

    /// Event processing failed
    EventProcessingError = 3009,

    /// Policy evaluation failed
    PolicyError = 3010,

    // === Resource Errors (4000-4099) ===
    /// Requested resource not found
    ResourceNotFound = 4000,

    /// Resource is busy or locked
    ResourceBusy = 4001,

    /// Quota or limit exceeded
    QuotaExceeded = 4002,

    /// Insufficient permissions for operation
    InsufficientPermissions = 4003,

    /// Configuration file not found
    ConfigNotFound = 4004,

    /// Storage not available (unmounted, etc.)
    StorageUnavailable = 4005,
}

impl RpcErrorCode {
    /// Get human-readable error message
    pub fn message(&self) -> &'static str {
        match self {
            // Connection errors
            Self::ConnectionRefused => "Connection refused - daemon may not be running",
            Self::ConnectionTimeout => "Connection timed out - daemon not responding",
            Self::SocketNotFound => "Socket file not found at expected path",
            Self::PermissionDenied => "Permission denied - check socket permissions",
            Self::ConnectionReset => "Connection reset by daemon",
            Self::ConnectionClosed => "Connection closed unexpectedly",
            Self::IoError => "Network I/O error occurred",

            // Request errors
            Self::InvalidRequest => "Invalid JSON-RPC request format",
            Self::MalformedJson => "Malformed JSON in request body",
            Self::MissingParameter => "Missing required parameter",
            Self::InvalidParameter => "Invalid parameter value or type",
            Self::UnknownMethod => "Unknown RPC method requested",
            Self::InvalidAutonomyLevel => "Invalid autonomy level (must be 'low' or 'high')",
            Self::InvalidDomain => "Invalid domain specified",
            Self::InvalidTimeRange => "Invalid time range or duration",

            // Server errors
            Self::InternalError => "Internal server error occurred",
            Self::DatabaseError => "Database operation failed",
            Self::CollectionFailed => "Failed to collect system metrics",
            Self::Timeout => "Operation timed out on server",
            Self::ConfigParseError => "Failed to parse configuration file",
            Self::ConfigReloadError => "Failed to reload configuration",
            Self::CommandExecutionError => "Failed to execute system command",
            Self::ParseError => "Failed to parse command output",
            Self::StorageError => "Storage operation failed",
            Self::EventProcessingError => "Event processing failed",
            Self::PolicyError => "Policy evaluation failed",

            // Resource errors
            Self::ResourceNotFound => "Requested resource not found",
            Self::ResourceBusy => "Resource is currently busy or locked",
            Self::QuotaExceeded => "Quota or limit exceeded",
            Self::InsufficientPermissions => "Insufficient permissions for operation",
            Self::ConfigNotFound => "Configuration file not found",
            Self::StorageUnavailable => "Storage not available or unmounted",
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            // Connection errors - usually retryable
            Self::ConnectionRefused | Self::ConnectionTimeout | Self::ConnectionReset
            | Self::ConnectionClosed | Self::IoError => true,

            // Request errors - not retryable (client bug)
            Self::InvalidRequest | Self::MalformedJson | Self::MissingParameter
            | Self::InvalidParameter | Self::UnknownMethod | Self::InvalidAutonomyLevel
            | Self::InvalidDomain | Self::InvalidTimeRange => false,

            // Server errors - some retryable
            Self::InternalError | Self::Timeout | Self::CommandExecutionError => true,
            Self::DatabaseError | Self::CollectionFailed | Self::StorageError
            | Self::EventProcessingError => true,
            Self::ConfigParseError | Self::ConfigReloadError | Self::ParseError
            | Self::PolicyError => false,

            // Resource errors - conditionally retryable
            Self::ResourceBusy | Self::StorageUnavailable => true,
            Self::ResourceNotFound | Self::QuotaExceeded | Self::InsufficientPermissions
            | Self::ConfigNotFound | Self::SocketNotFound | Self::PermissionDenied => false,
        }
    }

    /// Get suggested retry delay (if retryable)
    pub fn retry_after(&self) -> Option<Duration> {
        if !self.is_retryable() {
            return None;
        }

        Some(match self {
            // Quick retry for connection issues
            Self::ConnectionRefused | Self::ConnectionTimeout => Duration::from_millis(100),
            Self::ConnectionReset | Self::ConnectionClosed => Duration::from_millis(200),

            // Longer delay for server errors
            Self::Timeout | Self::InternalError => Duration::from_millis(500),
            Self::DatabaseError | Self::StorageError => Duration::from_millis(300),

            // Resource contention - wait a bit
            Self::ResourceBusy => Duration::from_millis(250),
            Self::StorageUnavailable => Duration::from_secs(1),

            _ => Duration::from_millis(100),
        })
    }

    /// Get actionable help text for user
    pub fn help_text(&self) -> Vec<&'static str> {
        match self {
            Self::ConnectionRefused | Self::ConnectionTimeout => vec![
                "Check daemon status: sudo systemctl status annad",
                "Check daemon logs: sudo journalctl -u annad -n 20",
                "Restart daemon: sudo systemctl restart annad",
                "Verify socket exists: ls -la /run/anna/rpc.sock",
            ],
            Self::SocketNotFound => vec![
                "Verify daemon is running: sudo systemctl status annad",
                "Check socket directory: ls -la /run/anna/",
                "Restart daemon: sudo systemctl restart annad",
            ],
            Self::PermissionDenied => vec![
                "Check socket permissions: ls -la /run/anna/rpc.sock",
                "Verify your user is in the 'anna' group: groups",
                "Add user to group: sudo usermod -aG anna $USER",
                "Log out and back in for group changes to take effect",
            ],
            Self::ConfigParseError | Self::ConfigReloadError => vec![
                "Validate config syntax: annactl config validate",
                "Check config file: cat /etc/anna/config.toml",
                "Review recent changes to configuration",
                "Restore from backup if needed",
            ],
            Self::DatabaseError => vec![
                "Check database integrity: annactl doctor check",
                "Review daemon logs for details: sudo journalctl -u annad -n 50",
                "Database file: /var/lib/anna/telemetry.db",
            ],
            Self::StorageError | Self::StorageUnavailable => vec![
                "Check filesystem status: annactl storage btrfs",
                "Verify filesystem mounted: mount | grep btrfs",
                "Check disk space: df -h",
            ],
            Self::InvalidParameter | Self::MissingParameter => vec![
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

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Connection issues - warnings (usually transient)
            Self::ConnectionRefused | Self::ConnectionTimeout | Self::ConnectionReset
            | Self::ConnectionClosed => ErrorSeverity::Warning,

            // Permission/configuration - errors (require action)
            Self::PermissionDenied | Self::SocketNotFound | Self::ConfigParseError
            | Self::ConfigNotFound | Self::InvalidAutonomyLevel => ErrorSeverity::Error,

            // Client bugs - errors
            Self::InvalidRequest | Self::MalformedJson | Self::MissingParameter
            | Self::InvalidParameter | Self::UnknownMethod | Self::InvalidDomain
            | Self::InvalidTimeRange => ErrorSeverity::Error,

            // Server issues - critical
            Self::InternalError | Self::DatabaseError => ErrorSeverity::Critical,

            // Resource issues - warnings (may resolve)
            Self::ResourceBusy | Self::StorageUnavailable | Self::Timeout => ErrorSeverity::Warning,

            // Other errors
            _ => ErrorSeverity::Error,
        }
    }
}

/// Error severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Warning - transient issue, may resolve automatically
    Warning,
    /// Error - requires user action to resolve
    Error,
    /// Critical - serious problem affecting daemon operation
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Structured RPC error with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    /// Error code
    pub code: RpcErrorCode,

    /// Additional context information
    pub context: Option<String>,

    /// Timestamp of error occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl RpcError {
    /// Create a new RPC error
    pub fn new(code: RpcErrorCode) -> Self {
        Self {
            code,
            context: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Add context to the error
    pub fn with_context<S: Into<String>>(mut self, context: S) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Get error message
    pub fn message(&self) -> &'static str {
        self.code.message()
    }

    /// Check if retryable
    pub fn is_retryable(&self) -> bool {
        self.code.is_retryable()
    }

    /// Get retry delay
    pub fn retry_after(&self) -> Option<Duration> {
        self.code.retry_after()
    }

    /// Get help text
    pub fn help_text(&self) -> Vec<&'static str> {
        self.code.help_text()
    }

    /// Get severity
    pub fn severity(&self) -> ErrorSeverity {
        self.code.severity()
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

impl std::error::Error for RpcError {}

/// Retry policy for RPC operations
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier (exponential growth)
    pub backoff_multiplier: f32,

    /// Jitter factor (randomness added to delay)
    pub jitter_factor: f32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl RetryPolicy {
    /// Calculate retry delay for a given attempt number (0-indexed)
    ///
    /// Formula: delay = min(initial_delay * (multiplier ^ attempt), max_delay) * (1.0 + random(-jitter, +jitter))
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        // Base exponential backoff
        let base_delay = self.initial_delay_ms as f64 * (self.backoff_multiplier as f64).powi(attempt as i32);

        // Cap at maximum delay
        let capped_delay = base_delay.min(self.max_delay_ms as f64);

        // Add jitter
        let jitter_range = capped_delay * self.jitter_factor as f64;
        let jitter = (rand::random::<f64>() * 2.0 - 1.0) * jitter_range;
        let final_delay = (capped_delay + jitter).max(0.0);

        Duration::from_millis(final_delay as u64)
    }

    /// Check if should retry for this attempt number
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    /// Get total estimated time for all retry attempts
    pub fn total_retry_time(&self) -> Duration {
        let mut total = Duration::ZERO;
        for attempt in 0..self.max_attempts {
            total += self.calculate_delay(attempt);
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_ranges() {
        assert_eq!(RpcErrorCode::ConnectionRefused as u32, 1000);
        assert_eq!(RpcErrorCode::InvalidRequest as u32, 2000);
        assert_eq!(RpcErrorCode::InternalError as u32, 3000);
        assert_eq!(RpcErrorCode::ResourceNotFound as u32, 4000);
    }

    #[test]
    fn test_retryable_classification() {
        assert!(RpcErrorCode::ConnectionRefused.is_retryable());
        assert!(RpcErrorCode::ConnectionTimeout.is_retryable());
        assert!(!RpcErrorCode::InvalidRequest.is_retryable());
        assert!(!RpcErrorCode::MalformedJson.is_retryable());
        assert!(RpcErrorCode::InternalError.is_retryable());
        assert!(!RpcErrorCode::PermissionDenied.is_retryable());
    }

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::default();

        let delay0 = policy.calculate_delay(0);
        let delay1 = policy.calculate_delay(1);
        let delay2 = policy.calculate_delay(2);

        // Each delay should be roughly 2x the previous (with jitter)
        assert!(delay0.as_millis() >= 80 && delay0.as_millis() <= 120); // ~100ms
        assert!(delay1.as_millis() >= 180 && delay1.as_millis() <= 220); // ~200ms
        assert!(delay2.as_millis() >= 360 && delay2.as_millis() <= 440); // ~400ms
    }

    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_delay_ms: 1000,
            ..Default::default()
        };

        // Very high attempt number should be capped
        let delay = policy.calculate_delay(10);
        assert!(delay.as_millis() <= 1100); // Allow some jitter
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy {
            max_attempts: 3,
            ..Default::default()
        };

        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
        assert!(!policy.should_retry(4));
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(RpcErrorCode::ConnectionTimeout.severity(), ErrorSeverity::Warning);
        assert_eq!(RpcErrorCode::PermissionDenied.severity(), ErrorSeverity::Error);
        assert_eq!(RpcErrorCode::InternalError.severity(), ErrorSeverity::Critical);
    }
}
