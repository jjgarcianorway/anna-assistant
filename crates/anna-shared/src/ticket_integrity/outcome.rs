//! Ticket Outcome Integrity (Part 1) - v0.0.442.
//!
//! Fix the lying stats problem:
//! - Stats show 100% success even when "Failed to parse specialist response"
//! - Tickets marked resolved when answer was garbage
//!
//! RULE: Ticket is ONLY `Answered` when:
//! - Specialist output was successfully parsed
//! - Answer was rendered without internal errors

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Explicit ticket outcome states.
/// A ticket can ONLY be in ONE of these states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketOutcome {
    /// Ticket created, not yet processed.
    Pending,
    /// Successfully answered: specialist parsed, answer rendered.
    Answered,
    /// Specialist response could not be parsed.
    ParseError,
    /// Probes failed, preventing meaningful answer.
    ProbeError,
    /// Waiting for user clarification.
    ClarificationPending,
    /// User cancelled the request.
    Cancelled,
    /// Internal system error.
    InternalError,
}

impl TicketOutcome {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Answered => "ANSWERED",
            Self::ParseError => "PARSE_ERROR",
            Self::ProbeError => "PROBE_ERROR",
            Self::ClarificationPending => "CLARIFICATION_PENDING",
            Self::Cancelled => "CANCELLED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    /// Is this a success state for stats?
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Answered)
    }

    /// Is this a failure state for stats?
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::ParseError | Self::ProbeError | Self::InternalError
        )
    }

    /// Is this a terminal state?
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::Pending | Self::ClarificationPending)
    }
}

impl Default for TicketOutcome {
    fn default() -> Self {
        Self::Pending
    }
}

/// Conditions required for Answered state.
#[derive(Debug, Clone, Default)]
pub struct AnsweredConditions {
    /// Specialist output was received.
    pub specialist_responded: bool,
    /// Output was valid JSON.
    pub json_valid: bool,
    /// Required schema fields present.
    pub schema_valid: bool,
    /// Answer was rendered to user.
    pub answer_rendered: bool,
    /// No internal errors during processing.
    pub no_internal_errors: bool,
}

impl AnsweredConditions {
    /// Check if ALL conditions are met for Answered state.
    pub fn is_answered(&self) -> bool {
        self.specialist_responded
            && self.json_valid
            && self.schema_valid
            && self.answer_rendered
            && self.no_internal_errors
    }

    /// Get the reason for failure if not answered.
    pub fn failure_reason(&self) -> Option<TicketOutcome> {
        if !self.specialist_responded {
            return Some(TicketOutcome::InternalError);
        }
        if !self.json_valid || !self.schema_valid {
            return Some(TicketOutcome::ParseError);
        }
        if !self.answer_rendered {
            return Some(TicketOutcome::InternalError);
        }
        if !self.no_internal_errors {
            return Some(TicketOutcome::InternalError);
        }
        None
    }

    /// Determine final outcome.
    pub fn determine_outcome(&self) -> TicketOutcome {
        if self.is_answered() {
            TicketOutcome::Answered
        } else {
            self.failure_reason()
                .unwrap_or(TicketOutcome::InternalError)
        }
    }
}

/// Honest ticket statistics.
/// ONLY counts `Answered` as resolved.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonestTicketStats {
    /// Total tickets processed.
    pub total_tickets: u64,
    /// Successfully answered (parsed + rendered).
    pub answered: u64,
    /// Parse errors (specialist output invalid).
    pub failed_parse: u64,
    /// Probe failures.
    pub probe_failures: u64,
    /// Cancelled by user.
    pub cancelled: u64,
    /// Internal errors.
    pub internal_errors: u64,
    /// Pending clarification.
    pub clarification_pending: u64,
}

impl HonestTicketStats {
    /// Create empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ticket outcome.
    pub fn record(&mut self, outcome: TicketOutcome) {
        self.total_tickets += 1;
        match outcome {
            TicketOutcome::Pending => {}
            TicketOutcome::Answered => self.answered += 1,
            TicketOutcome::ParseError => self.failed_parse += 1,
            TicketOutcome::ProbeError => self.probe_failures += 1,
            TicketOutcome::ClarificationPending => self.clarification_pending += 1,
            TicketOutcome::Cancelled => self.cancelled += 1,
            TicketOutcome::InternalError => self.internal_errors += 1,
        }
    }

    /// Get honest success rate.
    /// ONLY Answered counts as success.
    pub fn success_rate(&self) -> f64 {
        if self.total_tickets == 0 {
            0.0
        } else {
            self.answered as f64 / self.total_tickets as f64
        }
    }

    /// Get failure rate.
    pub fn failure_rate(&self) -> f64 {
        if self.total_tickets == 0 {
            0.0
        } else {
            (self.failed_parse + self.probe_failures + self.internal_errors) as f64
                / self.total_tickets as f64
        }
    }

    /// Format for display (honest format).
    pub fn display(&self) -> String {
        let rate = self.success_rate() * 100.0;
        format!(
            "[service desk]\n  \
             total_tickets         {}\n  \
             answered              {} ({:.0}%)\n  \
             failed_parse          {}\n  \
             probe_failures        {}\n  \
             clarification_pending {}\n  \
             cancelled             {}\n  \
             internal_errors       {}",
            self.total_tickets,
            self.answered,
            rate,
            self.failed_parse,
            self.probe_failures,
            self.clarification_pending,
            self.cancelled,
            self.internal_errors
        )
    }
}

/// Ticket outcome record with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketOutcomeRecord {
    /// Case ID.
    pub case_id: String,
    /// Final outcome.
    pub outcome: TicketOutcome,
    /// Error message if failed.
    pub error_message: Option<String>,
    /// Timestamp (ms since epoch).
    pub timestamp_ms: u64,
}

impl TicketOutcomeRecord {
    /// Create new record.
    pub fn new(case_id: &str, outcome: TicketOutcome) -> Self {
        Self {
            case_id: case_id.to_string(),
            outcome,
            error_message: None,
            timestamp_ms: current_timestamp_ms(),
        }
    }

    /// Add error message.
    pub fn with_error(mut self, msg: &str) -> Self {
        self.error_message = Some(msg.to_string());
        self
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Error patterns that indicate ParseError outcome.
pub const PARSE_ERROR_PATTERNS: &[&str] = &[
    "Failed to parse specialist response",
    "Parse error",
    "Invalid JSON",
    "Schema validation failed",
    "Missing required field",
    "Timeout",
];

/// Check if an error message indicates a parse error.
pub fn is_parse_error(error_msg: &str) -> bool {
    PARSE_ERROR_PATTERNS
        .iter()
        .any(|pattern| error_msg.contains(pattern))
}

/// Determine outcome from error message.
pub fn outcome_from_error(error_msg: &str) -> TicketOutcome {
    if is_parse_error(error_msg) {
        TicketOutcome::ParseError
    } else if error_msg.contains("Probe") || error_msg.contains("probe") {
        TicketOutcome::ProbeError
    } else {
        TicketOutcome::InternalError
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_outcome_states() {
        assert!(TicketOutcome::Answered.is_success());
        assert!(!TicketOutcome::ParseError.is_success());
        assert!(TicketOutcome::ParseError.is_failure());
        assert!(TicketOutcome::Answered.is_terminal());
        assert!(!TicketOutcome::Pending.is_terminal());
    }

    #[test]
    fn test_answered_conditions() {
        let mut cond = AnsweredConditions::default();
        assert!(!cond.is_answered());
        assert!(cond.failure_reason().is_some());

        cond.specialist_responded = true;
        cond.json_valid = true;
        cond.schema_valid = true;
        cond.answer_rendered = true;
        cond.no_internal_errors = true;

        assert!(cond.is_answered());
        assert_eq!(cond.determine_outcome(), TicketOutcome::Answered);
    }

    #[test]
    fn test_parse_error_detection() {
        let mut cond = AnsweredConditions {
            specialist_responded: true,
            json_valid: false,
            ..Default::default()
        };
        assert_eq!(cond.determine_outcome(), TicketOutcome::ParseError);
    }

    #[test]
    fn test_honest_stats() {
        let mut stats = HonestTicketStats::new();
        stats.record(TicketOutcome::Answered);
        stats.record(TicketOutcome::Answered);
        stats.record(TicketOutcome::ParseError);
        stats.record(TicketOutcome::ProbeError);

        assert_eq!(stats.total_tickets, 4);
        assert_eq!(stats.answered, 2);
        assert_eq!(stats.failed_parse, 1);
        assert_eq!(stats.probe_failures, 1);
        assert!((stats.success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_parse_error() {
        assert!(is_parse_error(
            "Failed to parse specialist response. Parse error: Timeout"
        ));
        assert!(is_parse_error("Invalid JSON in response"));
        assert!(!is_parse_error("Everything worked fine"));
    }

    #[test]
    fn test_outcome_from_error() {
        assert_eq!(
            outcome_from_error("Failed to parse specialist response"),
            TicketOutcome::ParseError
        );
        assert_eq!(
            outcome_from_error("Probe execution failed"),
            TicketOutcome::ProbeError
        );
        assert_eq!(
            outcome_from_error("Unknown error"),
            TicketOutcome::InternalError
        );
    }
}
