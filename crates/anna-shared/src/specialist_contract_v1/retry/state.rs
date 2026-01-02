//! Retry state tracking.

use std::time::Duration;

use super::config::RetryConfig;
use super::decision::{RetryDecision, RetrySummary};
use crate::specialist_contract_v1::validator::ValidationError;

/// Result of an attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptResult {
    /// Succeeded.
    Success,
    /// Failed validation.
    ValidationFailed,
    /// Timed out.
    Timeout,
    /// Network/other error.
    Error,
}

/// Record of a retry attempt.
#[derive(Debug, Clone)]
pub struct RetryAttempt {
    /// Attempt number (0-indexed).
    pub attempt: usize,
    /// Result of this attempt.
    pub result: AttemptResult,
    /// Duration of this attempt.
    pub duration_ms: u64,
    /// Error if failed.
    pub error: Option<ValidationError>,
}

/// Retry state tracker.
#[derive(Debug, Clone)]
pub struct RetryState {
    /// Configuration.
    pub config: RetryConfig,
    /// Current attempt (0-indexed).
    pub attempt: usize,
    /// Last validation error.
    pub last_error: Option<ValidationError>,
    /// Whether exhausted.
    pub exhausted: bool,
    /// Total time spent.
    pub total_time_ms: u64,
    /// Attempt history.
    pub history: Vec<RetryAttempt>,
}

impl RetryState {
    /// Create new state.
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(config: RetryConfig) -> Self {
        Self {
            config,
            attempt: 0,
            last_error: None,
            exhausted: false,
            total_time_ms: 0,
            history: Vec::new(),
        }
    }

    /// Record a successful attempt.
    pub fn record_success(&mut self, duration_ms: u64) {
        self.history.push(RetryAttempt {
            attempt: self.attempt,
            result: AttemptResult::Success,
            duration_ms,
            error: None,
        });
        self.total_time_ms += duration_ms;
    }

    /// Record a failed attempt and return whether to retry.
    pub fn record_failure(&mut self, error: ValidationError, duration_ms: u64) -> RetryDecision {
        self.history.push(RetryAttempt {
            attempt: self.attempt,
            result: AttemptResult::ValidationFailed,
            duration_ms,
            error: Some(error.clone()),
        });
        self.total_time_ms += duration_ms;
        self.last_error = Some(error.clone());

        // Check if error is retriable
        if !error.is_retriable() {
            self.exhausted = true;
            return RetryDecision::GiveUp {
                reason: "Non-retriable error".to_string(),
            };
        }

        // Check if can retry
        if self.config.can_retry(self.attempt) {
            let backoff = self.config.backoff_for_attempt(self.attempt);
            let prompt = self
                .config
                .repair_prompt_for_attempt(self.attempt)
                .to_string();
            self.attempt += 1;
            RetryDecision::Retry {
                attempt: self.attempt,
                backoff,
                repair_prompt: prompt,
            }
        } else {
            self.exhausted = true;
            RetryDecision::GiveUp {
                reason: "Max retries exhausted".to_string(),
            }
        }
    }

    /// Record a timeout.
    pub fn record_timeout(&mut self, duration_ms: u64) -> RetryDecision {
        self.history.push(RetryAttempt {
            attempt: self.attempt,
            result: AttemptResult::Timeout,
            duration_ms,
            error: None,
        });
        self.total_time_ms += duration_ms;

        if self.config.can_retry(self.attempt) {
            let backoff = self.config.backoff_for_attempt(self.attempt);
            let prompt = self
                .config
                .repair_prompt_for_attempt(self.attempt)
                .to_string();
            self.attempt += 1;
            RetryDecision::Retry {
                attempt: self.attempt,
                backoff,
                repair_prompt: prompt,
            }
        } else {
            self.exhausted = true;
            RetryDecision::GiveUp {
                reason: "Max retries exhausted after timeout".to_string(),
            }
        }
    }

    /// Get summary of attempts.
    pub fn summary(&self) -> RetrySummary {
        RetrySummary {
            total_attempts: self.history.len(),
            successful: self
                .history
                .iter()
                .any(|a| a.result == AttemptResult::Success),
            timeouts: self
                .history
                .iter()
                .filter(|a| a.result == AttemptResult::Timeout)
                .count(),
            validation_failures: self
                .history
                .iter()
                .filter(|a| a.result == AttemptResult::ValidationFailed)
                .count(),
            total_time_ms: self.total_time_ms,
            exhausted: self.exhausted,
        }
    }
}

impl Default for RetryState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_state_success() {
        let mut state = RetryState::new();
        state.record_success(500);

        let summary = state.summary();
        assert!(summary.successful);
        assert_eq!(summary.total_attempts, 1);
    }

    #[test]
    fn test_retry_state_failures() {
        let mut state = RetryState::new();

        // First failure - should retry
        let error = ValidationError::SchemaInvalid {
            issues: vec!["test".to_string()],
        };
        let decision = state.record_failure(error.clone(), 500);
        assert!(decision.should_retry());

        // Second failure - should retry
        let decision = state.record_failure(error.clone(), 500);
        assert!(decision.should_retry());

        // Third failure - exhausted
        let decision = state.record_failure(error, 500);
        assert!(!decision.should_retry());

        let summary = state.summary();
        assert!(!summary.successful);
        assert!(summary.exhausted);
        assert_eq!(summary.validation_failures, 3);
    }

    #[test]
    fn test_non_retriable_error() {
        let mut state = RetryState::new();

        // Empty error is not retriable
        let error = ValidationError::Empty;
        let decision = state.record_failure(error, 100);
        assert!(!decision.should_retry());
    }

    #[test]
    fn test_retry_summary() {
        let mut state = RetryState::new();
        state.record_timeout(8000);
        state.record_failure(ValidationError::SchemaInvalid { issues: vec![] }, 500);
        state.record_success(300);

        let summary = state.summary();
        assert!(summary.successful);
        assert_eq!(summary.timeouts, 1);
        assert_eq!(summary.validation_failures, 1);
        assert_eq!(summary.total_time_ms, 8800);
    }
}
