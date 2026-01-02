//! Reliability Stats Types - v0.0.438.
//!
//! Core types for reliability tracking:
//! - ReliabilityOutcome: Outcome of a pipeline execution
//! - ExecutionRecord: A single execution record

use serde::{Deserialize, Serialize};

/// Outcome of a pipeline execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReliabilityOutcome {
    /// Specialist answered successfully.
    SpecialistSuccess,
    /// Specialist timed out.
    SpecialistTimeout,
    /// Failed to parse specialist response.
    ParserFailure,
    /// Fell back to probe-only answer.
    FallbackToProbes,
    /// Complete failure (no answer).
    TotalFailure,
    /// Success after retry.
    SuccessAfterRetry,
}

impl ReliabilityOutcome {
    /// Whether this is a success outcome.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::SpecialistSuccess | Self::SuccessAfterRetry)
    }

    /// Whether this is a timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::SpecialistTimeout)
    }

    /// Whether this is a parser failure.
    pub fn is_parser_failure(&self) -> bool {
        matches!(self, Self::ParserFailure)
    }

    /// Whether this is a fallback.
    pub fn is_fallback(&self) -> bool {
        matches!(self, Self::FallbackToProbes)
    }

    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::SpecialistSuccess => "specialist_success",
            Self::SpecialistTimeout => "specialist_timeout",
            Self::ParserFailure => "parser_failure",
            Self::FallbackToProbes => "fallback_to_probes",
            Self::TotalFailure => "total_failure",
            Self::SuccessAfterRetry => "success_after_retry",
        }
    }
}

/// A single execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Outcome of execution.
    pub outcome: ReliabilityOutcome,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Number of retry attempts.
    pub retry_attempts: usize,
    /// Whether probes were used.
    pub probes_used: bool,
    /// Specialist type used (junior/senior).
    pub specialist_tier: Option<String>,
}

impl ExecutionRecord {
    /// Create new record.
    pub fn new(outcome: ReliabilityOutcome, duration_ms: u64) -> Self {
        Self {
            outcome,
            duration_ms,
            retry_attempts: 0,
            probes_used: false,
            specialist_tier: None,
        }
    }

    /// Set retry attempts.
    pub fn with_retries(mut self, attempts: usize) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Mark probes as used.
    pub fn with_probes(mut self) -> Self {
        self.probes_used = true;
        self
    }

    /// Set specialist tier.
    pub fn with_tier(mut self, tier: &str) -> Self {
        self.specialist_tier = Some(tier.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_outcome() {
        assert!(ReliabilityOutcome::SpecialistSuccess.is_success());
        assert!(ReliabilityOutcome::SpecialistTimeout.is_timeout());
        assert!(ReliabilityOutcome::ParserFailure.is_parser_failure());
        assert!(ReliabilityOutcome::FallbackToProbes.is_fallback());
    }

    #[test]
    fn test_execution_record() {
        let record = ExecutionRecord::new(ReliabilityOutcome::SpecialistSuccess, 500)
            .with_retries(1)
            .with_probes()
            .with_tier("junior");

        assert_eq!(record.outcome, ReliabilityOutcome::SpecialistSuccess);
        assert_eq!(record.duration_ms, 500);
        assert_eq!(record.retry_attempts, 1);
        assert!(record.probes_used);
    }
}
