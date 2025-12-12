//! Retry Strategy (Part D) - v0.0.438.
//!
//! On timeout or parse failure:
//! - Retry once with a smaller/faster model (if available)
//! - Then fall back to probe-only answer
//!
//! This gives us one chance to recover before giving up on specialists.

use serde::{Deserialize, Serialize};

/// Maximum retry attempts.
pub const MAX_RETRIES: usize = 1;

/// Retry strategy for failed calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryStrategy {
    /// No retry, fail immediately.
    NoRetry,
    /// Retry with same model.
    RetrySame,
    /// Retry with smaller/faster model.
    RetrySmaller,
    /// Fall back to probe-only answer.
    FallbackProbeOnly,
}

impl RetryStrategy {
    /// Whether this strategy involves a retry.
    pub fn is_retry(&self) -> bool {
        matches!(self, Self::RetrySame | Self::RetrySmaller)
    }

    /// Whether this is a fallback strategy.
    pub fn is_fallback(&self) -> bool {
        matches!(self, Self::FallbackProbeOnly)
    }

    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::NoRetry => "no_retry",
            Self::RetrySame => "retry_same",
            Self::RetrySmaller => "retry_smaller",
            Self::FallbackProbeOnly => "fallback_probe_only",
        }
    }
}

/// Failure type that triggers retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureType {
    /// Response timed out.
    Timeout,
    /// Failed to parse response.
    ParseError,
    /// Model returned empty response.
    EmptyResponse,
    /// Model returned invalid JSON.
    InvalidJson,
    /// Model returned unexpected format.
    UnexpectedFormat,
    /// Network error.
    NetworkError,
    /// Rate limit hit.
    RateLimit,
}

impl FailureType {
    /// Get default retry strategy for this failure type.
    pub fn default_strategy(&self) -> RetryStrategy {
        match self {
            // Timeouts - retry with smaller model
            Self::Timeout => RetryStrategy::RetrySmaller,
            // Parse errors - retry with same model once
            Self::ParseError => RetryStrategy::RetrySame,
            Self::InvalidJson => RetryStrategy::RetrySame,
            Self::UnexpectedFormat => RetryStrategy::RetrySame,
            // Empty response - retry same
            Self::EmptyResponse => RetryStrategy::RetrySame,
            // Network issues - retry same
            Self::NetworkError => RetryStrategy::RetrySame,
            // Rate limit - no immediate retry
            Self::RateLimit => RetryStrategy::FallbackProbeOnly,
        }
    }

    /// Whether this failure is recoverable.
    pub fn is_recoverable(&self) -> bool {
        !matches!(self, Self::RateLimit)
    }

    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::ParseError => "parse_error",
            Self::EmptyResponse => "empty_response",
            Self::InvalidJson => "invalid_json",
            Self::UnexpectedFormat => "unexpected_format",
            Self::NetworkError => "network_error",
            Self::RateLimit => "rate_limit",
        }
    }
}

/// Result of a retry attempt.
#[derive(Debug, Clone)]
pub enum RetryResult<T> {
    /// Success on first try.
    Success(T),
    /// Success after retry.
    SuccessAfterRetry {
        result: T,
        attempts: usize,
        failure_type: FailureType,
    },
    /// Failed, falling back to probe-only.
    FallbackToProbes {
        attempts: usize,
        last_failure: FailureType,
    },
    /// Exhausted all retries, giving up.
    Exhausted {
        attempts: usize,
        last_failure: FailureType,
    },
}

impl<T> RetryResult<T> {
    /// Check if successful (with or without retry).
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_) | Self::SuccessAfterRetry { .. })
    }

    /// Check if falling back to probes.
    pub fn is_fallback(&self) -> bool {
        matches!(self, Self::FallbackToProbes { .. })
    }

    /// Get the result if successful.
    pub fn get_result(self) -> Option<T> {
        match self {
            Self::Success(r) => Some(r),
            Self::SuccessAfterRetry { result, .. } => Some(result),
            _ => None,
        }
    }

    /// Get attempt count.
    pub fn attempts(&self) -> usize {
        match self {
            Self::Success(_) => 1,
            Self::SuccessAfterRetry { attempts, .. } => *attempts,
            Self::FallbackToProbes { attempts, .. } => *attempts,
            Self::Exhausted { attempts, .. } => *attempts,
        }
    }
}

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts.
    pub max_retries: usize,
    /// Use smaller model on timeout.
    pub use_smaller_on_timeout: bool,
    /// Fall back to probes after exhausting retries.
    pub fallback_to_probes: bool,
    /// Reduced timeout for retry (percentage of original).
    pub retry_timeout_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            use_smaller_on_timeout: true,
            fallback_to_probes: true,
            retry_timeout_factor: 0.7,
        }
    }
}

impl RetryConfig {
    /// Create strict config (no retries).
    pub fn strict() -> Self {
        Self {
            max_retries: 0,
            use_smaller_on_timeout: false,
            fallback_to_probes: true,
            retry_timeout_factor: 1.0,
        }
    }

    /// Create lenient config (more retries).
    pub fn lenient() -> Self {
        Self {
            max_retries: 2,
            use_smaller_on_timeout: true,
            fallback_to_probes: true,
            retry_timeout_factor: 0.5,
        }
    }

    /// Get strategy for a failure.
    pub fn strategy_for(&self, failure: FailureType, attempt: usize) -> RetryStrategy {
        if attempt >= self.max_retries {
            if self.fallback_to_probes {
                return RetryStrategy::FallbackProbeOnly;
            }
            return RetryStrategy::NoRetry;
        }

        match failure {
            FailureType::Timeout if self.use_smaller_on_timeout => RetryStrategy::RetrySmaller,
            FailureType::RateLimit => RetryStrategy::FallbackProbeOnly,
            _ => RetryStrategy::RetrySame,
        }
    }

    /// Calculate timeout for retry attempt.
    pub fn retry_timeout(&self, original_ms: u64, attempt: usize) -> u64 {
        if attempt == 0 {
            original_ms
        } else {
            (original_ms as f64 * self.retry_timeout_factor) as u64
        }
    }
}

/// Retry state tracker.
#[derive(Debug, Clone)]
pub struct RetryTracker {
    /// Configuration.
    pub config: RetryConfig,
    /// Current attempt (0-indexed).
    pub attempt: usize,
    /// Last failure type.
    pub last_failure: Option<FailureType>,
    /// Whether exhausted.
    pub exhausted: bool,
}

impl RetryTracker {
    /// Create new tracker.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            attempt: 0,
            last_failure: None,
            exhausted: false,
        }
    }

    /// Create with default config.
    pub fn with_defaults() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Record a failure and get next strategy.
    pub fn record_failure(&mut self, failure: FailureType) -> RetryStrategy {
        self.last_failure = Some(failure);
        let strategy = self.config.strategy_for(failure, self.attempt);

        if strategy.is_retry() {
            self.attempt += 1;
        } else {
            self.exhausted = true;
        }

        strategy
    }

    /// Check if can retry.
    pub fn can_retry(&self) -> bool {
        !self.exhausted && self.attempt < self.config.max_retries
    }

    /// Get timeout for current attempt.
    pub fn current_timeout(&self, base_ms: u64) -> u64 {
        self.config.retry_timeout(base_ms, self.attempt)
    }

    /// Mark as successful.
    pub fn mark_success(&mut self) {
        self.exhausted = false;
    }
}

impl Default for RetryTracker {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_strategy() {
        assert!(RetryStrategy::RetrySame.is_retry());
        assert!(RetryStrategy::RetrySmaller.is_retry());
        assert!(!RetryStrategy::NoRetry.is_retry());
        assert!(RetryStrategy::FallbackProbeOnly.is_fallback());
    }

    #[test]
    fn test_failure_type_strategy() {
        assert_eq!(FailureType::Timeout.default_strategy(), RetryStrategy::RetrySmaller);
        assert_eq!(FailureType::ParseError.default_strategy(), RetryStrategy::RetrySame);
        assert_eq!(FailureType::RateLimit.default_strategy(), RetryStrategy::FallbackProbeOnly);
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 1);
        assert!(config.use_smaller_on_timeout);

        let strategy = config.strategy_for(FailureType::Timeout, 0);
        assert_eq!(strategy, RetryStrategy::RetrySmaller);

        let strategy = config.strategy_for(FailureType::Timeout, 1);
        assert_eq!(strategy, RetryStrategy::FallbackProbeOnly);
    }

    #[test]
    fn test_retry_tracker() {
        let mut tracker = RetryTracker::with_defaults();

        assert!(tracker.can_retry());

        let strategy = tracker.record_failure(FailureType::Timeout);
        assert_eq!(strategy, RetryStrategy::RetrySmaller);
        assert!(!tracker.can_retry()); // Only 1 retry allowed

        let strategy = tracker.record_failure(FailureType::Timeout);
        assert_eq!(strategy, RetryStrategy::FallbackProbeOnly);
        assert!(tracker.exhausted);
    }

    #[test]
    fn test_retry_result() {
        let success: RetryResult<i32> = RetryResult::Success(42);
        assert!(success.is_success());
        assert_eq!(success.attempts(), 1);

        let after_retry: RetryResult<i32> = RetryResult::SuccessAfterRetry {
            result: 42,
            attempts: 2,
            failure_type: FailureType::Timeout,
        };
        assert!(after_retry.is_success());
        assert_eq!(after_retry.attempts(), 2);

        let fallback: RetryResult<i32> = RetryResult::FallbackToProbes {
            attempts: 2,
            last_failure: FailureType::Timeout,
        };
        assert!(fallback.is_fallback());
    }

    #[test]
    fn test_retry_timeout_factor() {
        let config = RetryConfig::default();
        assert_eq!(config.retry_timeout(1000, 0), 1000);
        assert_eq!(config.retry_timeout(1000, 1), 700);
    }
}
