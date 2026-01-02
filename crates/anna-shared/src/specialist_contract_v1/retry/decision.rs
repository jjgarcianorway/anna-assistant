//! Retry decision logic and summary reporting.

use std::time::Duration;

use super::config::{REPAIR_PROMPT_1, REPAIR_PROMPT_2};
use crate::specialist_contract_v1::validator::ValidationError;

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
}
