//! Retry Strategy (Part C) - v0.0.440.
//!
//! For each specialist call:
//! - Timeout: 8s (local models are unreliable workers)
//! - Retries: 2 maximum
//! - Backoff: 250ms then 500ms
//!
//! Retry prompts:
//! 1) "You violated SRC v1. Output ONLY valid JSON matching schema. No prose."
//! 2) "Last chance. Output ONLY JSON. If uncertain, reduce scope and lower confidence."

use std::time::Duration;

use super::validator::{ValidationError, ValidationResult};

/// Specialist call timeout in milliseconds.
pub const SPECIALIST_TIMEOUT_MS: u64 = 8000;

/// Maximum retry attempts.
pub const MAX_RETRIES: usize = 2;

/// First backoff duration in milliseconds.
pub const BACKOFF_1_MS: u64 = 250;

/// Second backoff duration in milliseconds.
pub const BACKOFF_2_MS: u64 = 500;

/// Retry prompt for first retry.
pub const REPAIR_PROMPT_1: &str =
    "You violated SRC v1. Output ONLY valid JSON matching schema. No prose. \
    Required fields: case_id, department, assessment (summary, confidence, risk). \
    No markdown. No extra text.";

/// Retry prompt for second retry.
pub const REPAIR_PROMPT_2: &str =
    "Last chance. Output ONLY JSON. If uncertain, reduce scope and lower confidence. \
    Example: {\"case_id\":\"...\",\"department\":\"Performance\",\
    \"assessment\":{\"summary\":\"Brief answer.\",\"confidence\":0.6,\"risk\":\"read_only\"},\
    \"actions\":[],\"citations\":[]}";

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Timeout per call in milliseconds.
    pub timeout_ms: u64,
    /// Maximum retry attempts.
    pub max_retries: usize,
    /// Backoff durations.
    pub backoffs: Vec<Duration>,
    /// Repair prompts.
    pub repair_prompts: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            timeout_ms: SPECIALIST_TIMEOUT_MS,
            max_retries: MAX_RETRIES,
            backoffs: vec![
                Duration::from_millis(BACKOFF_1_MS),
                Duration::from_millis(BACKOFF_2_MS),
            ],
            repair_prompts: vec![REPAIR_PROMPT_1.to_string(), REPAIR_PROMPT_2.to_string()],
        }
    }
}

impl RetryConfig {
    /// Get timeout as Duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Get backoff for attempt (0-indexed).
    pub fn backoff_for_attempt(&self, attempt: usize) -> Duration {
        self.backoffs
            .get(attempt)
            .cloned()
            .unwrap_or(Duration::from_millis(500))
    }

    /// Get repair prompt for attempt (0-indexed).
    pub fn repair_prompt_for_attempt(&self, attempt: usize) -> &str {
        self.repair_prompts
            .get(attempt)
            .map(|s| s.as_str())
            .unwrap_or(REPAIR_PROMPT_2)
    }

    /// Check if can retry.
    pub fn can_retry(&self, attempt: usize) -> bool {
        attempt < self.max_retries
    }
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

/// Decision after recording a failure.
#[derive(Debug, Clone)]
pub enum RetryDecision {
    /// Should retry with this prompt.
    Retry {
        attempt: usize,
        backoff: Duration,
        repair_prompt: String,
    },
    /// Should give up.
    GiveUp { reason: String },
}

impl RetryDecision {
    /// Check if should retry.
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::Retry { .. })
    }
}

/// Summary of retry attempts.
#[derive(Debug, Clone)]
pub struct RetrySummary {
    /// Total attempts made.
    pub total_attempts: usize,
    /// Whether any attempt succeeded.
    pub successful: bool,
    /// Number of timeouts.
    pub timeouts: usize,
    /// Number of validation failures.
    pub validation_failures: usize,
    /// Total time spent.
    pub total_time_ms: u64,
    /// Whether retries were exhausted.
    pub exhausted: bool,
}

impl RetrySummary {
    /// Format for logging.
    pub fn log_message(&self) -> String {
        let status = if self.successful {
            "SUCCESS"
        } else if self.exhausted {
            "EXHAUSTED"
        } else {
            "FAILED"
        };
        format!(
            "[retry] {} | attempts={} timeouts={} validation_failures={} total_time={}ms",
            status,
            self.total_attempts,
            self.timeouts,
            self.validation_failures,
            self.total_time_ms
        )
    }
}

/// Build a repair prompt with context.
pub fn build_repair_prompt(
    attempt: usize,
    last_error: Option<&ValidationError>,
    case_id: &str,
) -> String {
    let base = if attempt == 0 {
        REPAIR_PROMPT_1
    } else {
        REPAIR_PROMPT_2
    };

    let error_hint = match last_error {
        Some(ValidationError::InvalidJson { message }) => {
            format!(" JSON parse error: {}.", message)
        }
        Some(ValidationError::SchemaInvalid { issues }) => {
            format!(" Schema issues: {}.", issues.join(", "))
        }
        Some(ValidationError::ContainsMarkdown { offending }) => {
            format!(" Remove markdown ({}).", offending)
        }
        Some(ValidationError::CaseIdMismatch { expected, .. }) => {
            format!(" Use case_id=\"{}\".", expected)
        }
        _ => String::new(),
    };

    format!("{}{} Case ID: {}", base, error_hint, case_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.timeout_ms, 8000);
        assert_eq!(config.max_retries, 2);
    }

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
    fn test_build_repair_prompt() {
        let prompt = build_repair_prompt(0, None, "DSK-0101");
        assert!(prompt.contains("SRC v1"));
        assert!(prompt.contains("DSK-0101"));

        let prompt = build_repair_prompt(
            1,
            Some(&ValidationError::ContainsMarkdown {
                offending: "heading".to_string(),
            }),
            "DSK-0102",
        );
        assert!(prompt.contains("markdown"));
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
