//! Canonical ticket outcomes (v0.0.444).
//!
//! Single source of truth for ticket resolution states.
//! Stats must reflect reality - no lying about failures.
//!
//! Rules:
//! - ANSWERED_VERIFIED = resolved (counts as success)
//! - ANSWERED_PARTIAL = partial (some answer, not complete)
//! - Everything else = NOT resolved

use serde::{Deserialize, Serialize};

/// Canonical ticket outcome - the definitive result of processing a request.
///
/// These are the ONLY valid states a ticket can end in.
/// Stats computation uses ONLY these values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalOutcome {
    /// Answer produced, validator passed, all claims evidence-backed.
    /// This is the ONLY state that counts as "resolved" in stats.
    AnsweredVerified,

    /// Answered some parts; missing evidence for others; explicitly stated gaps.
    /// Counts as "partial" - better than failure but not fully resolved.
    AnsweredPartial,

    /// Asked 1 clarifying question, no claims beyond evidence.
    /// Request is pending user input.
    ClarificationNeeded,

    /// LLM timeout (translator or specialist) prevented verified answer.
    FailedTimeout,

    /// Invalid JSON from translator/specialist or schema validation failure.
    FailedParse,

    /// Required probes failed or returned empty.
    FailedProbes,

    /// User cancelled the request.
    AbortedByUser,

    /// Unexpected internal exception.
    ErrorInternal,
}

impl CanonicalOutcome {
    /// Is this a "resolved" outcome for stats?
    /// ONLY AnsweredVerified counts as resolved.
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::AnsweredVerified)
    }

    /// Is this a partial resolution?
    pub fn is_partial(&self) -> bool {
        matches!(self, Self::AnsweredPartial)
    }

    /// Is this "useful" (resolved or partial)?
    pub fn is_useful(&self) -> bool {
        matches!(self, Self::AnsweredVerified | Self::AnsweredPartial)
    }

    /// Is this a failure state?
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            Self::FailedTimeout | Self::FailedParse | Self::FailedProbes | Self::ErrorInternal
        )
    }

    /// Is this a terminal state?
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::ClarificationNeeded)
    }

    /// Is this pending user action?
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::ClarificationNeeded)
    }

    /// Get display label for UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::AnsweredVerified => "VERIFIED",
            Self::AnsweredPartial => "PARTIAL",
            Self::ClarificationNeeded => "CLARIFYING",
            Self::FailedTimeout => "TIMEOUT",
            Self::FailedParse => "PARSE_ERROR",
            Self::FailedProbes => "PROBE_FAILED",
            Self::AbortedByUser => "CANCELLED",
            Self::ErrorInternal => "INTERNAL_ERROR",
        }
    }

    /// Get short code for compact display.
    pub fn code(&self) -> &'static str {
        match self {
            Self::AnsweredVerified => "OK",
            Self::AnsweredPartial => "PART",
            Self::ClarificationNeeded => "ASK",
            Self::FailedTimeout => "TIME",
            Self::FailedParse => "JSON",
            Self::FailedProbes => "PROBE",
            Self::AbortedByUser => "STOP",
            Self::ErrorInternal => "ERR",
        }
    }
}

impl std::fmt::Display for CanonicalOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Default for CanonicalOutcome {
    fn default() -> Self {
        Self::ErrorInternal // Conservative: assume error until proven otherwise
    }
}

/// Determine canonical outcome from conditions.
/// Use this instead of manually setting outcomes.
#[derive(Debug, Clone, Default)]
pub struct OutcomeConditions {
    /// Did specialist respond at all?
    pub specialist_responded: bool,
    /// Was the response valid JSON?
    pub json_valid: bool,
    /// Did it pass schema validation?
    pub schema_valid: bool,
    /// Was an answer rendered to the user?
    pub answer_rendered: bool,
    /// Was a clarification question asked?
    pub clarification_asked: bool,
    /// Was there an LLM timeout?
    pub timeout_occurred: bool,
    /// Did required probes fail?
    pub probes_failed: bool,
    /// Was the request cancelled by user?
    pub user_cancelled: bool,
    /// Was there an internal error?
    pub internal_error: bool,
    /// Evidence coverage (0.0-1.0) - claims with evidence / total claims
    pub evidence_coverage: f32,
}

impl OutcomeConditions {
    /// Determine the canonical outcome from conditions.
    /// Order matters: check in priority order.
    pub fn determine(&self) -> CanonicalOutcome {
        // User cancelled - takes priority
        if self.user_cancelled {
            return CanonicalOutcome::AbortedByUser;
        }

        // Internal errors take priority over other failures
        if self.internal_error {
            return CanonicalOutcome::ErrorInternal;
        }

        // Timeout
        if self.timeout_occurred {
            return CanonicalOutcome::FailedTimeout;
        }

        // Probe failures
        if self.probes_failed {
            return CanonicalOutcome::FailedProbes;
        }

        // Clarification needed
        if self.clarification_asked && !self.answer_rendered {
            return CanonicalOutcome::ClarificationNeeded;
        }

        // Parse failures (specialist didn't respond properly)
        if !self.specialist_responded || !self.json_valid || !self.schema_valid {
            return CanonicalOutcome::FailedParse;
        }

        // Answer was rendered - check quality
        if self.answer_rendered {
            // High evidence coverage = verified
            if self.evidence_coverage >= 0.8 {
                return CanonicalOutcome::AnsweredVerified;
            }
            // Some coverage = partial
            if self.evidence_coverage >= 0.3 {
                return CanonicalOutcome::AnsweredPartial;
            }
            // Low coverage = still partial (not failure)
            return CanonicalOutcome::AnsweredPartial;
        }

        // Fallback to internal error
        CanonicalOutcome::ErrorInternal
    }
}

/// Convert from old TicketOutcome (ticket_integrity) to canonical.
pub fn from_ticket_integrity_outcome(
    outcome: crate::ticket_integrity::outcome::TicketOutcome,
) -> CanonicalOutcome {
    match outcome {
        crate::ticket_integrity::outcome::TicketOutcome::Pending => CanonicalOutcome::ErrorInternal,
        crate::ticket_integrity::outcome::TicketOutcome::Answered => {
            CanonicalOutcome::AnsweredVerified
        }
        crate::ticket_integrity::outcome::TicketOutcome::ParseError => {
            CanonicalOutcome::FailedParse
        }
        crate::ticket_integrity::outcome::TicketOutcome::ProbeError => {
            CanonicalOutcome::FailedProbes
        }
        crate::ticket_integrity::outcome::TicketOutcome::ClarificationPending => {
            CanonicalOutcome::ClarificationNeeded
        }
        crate::ticket_integrity::outcome::TicketOutcome::Cancelled => {
            CanonicalOutcome::AbortedByUser
        }
        crate::ticket_integrity::outcome::TicketOutcome::InternalError => {
            CanonicalOutcome::ErrorInternal
        }
    }
}

/// Convert from old TicketOutcome (ticket_state) to canonical.
pub fn from_ticket_state_outcome(outcome: crate::ticket_state::TicketOutcome) -> CanonicalOutcome {
    match outcome {
        crate::ticket_state::TicketOutcome::Success => CanonicalOutcome::AnsweredVerified,
        crate::ticket_state::TicketOutcome::Partial => CanonicalOutcome::AnsweredPartial,
        crate::ticket_state::TicketOutcome::CannotAnswerSafely => CanonicalOutcome::AnsweredPartial,
        crate::ticket_state::TicketOutcome::ErrorParse => CanonicalOutcome::FailedParse,
        crate::ticket_state::TicketOutcome::ErrorTimeout => CanonicalOutcome::FailedTimeout,
        crate::ticket_state::TicketOutcome::ErrorTool => CanonicalOutcome::FailedProbes,
        crate::ticket_state::TicketOutcome::ErrorInternal => CanonicalOutcome::ErrorInternal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_classification() {
        assert!(CanonicalOutcome::AnsweredVerified.is_resolved());
        assert!(!CanonicalOutcome::AnsweredPartial.is_resolved());
        assert!(CanonicalOutcome::AnsweredPartial.is_partial());
        assert!(CanonicalOutcome::AnsweredVerified.is_useful());
        assert!(CanonicalOutcome::AnsweredPartial.is_useful());
        assert!(CanonicalOutcome::FailedTimeout.is_failure());
        assert!(CanonicalOutcome::FailedParse.is_failure());
        assert!(CanonicalOutcome::FailedProbes.is_failure());
        assert!(!CanonicalOutcome::AbortedByUser.is_failure());
    }

    #[test]
    fn test_outcome_determination() {
        // Verified answer
        let cond = OutcomeConditions {
            specialist_responded: true,
            json_valid: true,
            schema_valid: true,
            answer_rendered: true,
            evidence_coverage: 0.9,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::AnsweredVerified);

        // Partial answer (low evidence)
        let cond = OutcomeConditions {
            specialist_responded: true,
            json_valid: true,
            schema_valid: true,
            answer_rendered: true,
            evidence_coverage: 0.5,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::AnsweredPartial);

        // Timeout
        let cond = OutcomeConditions {
            timeout_occurred: true,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::FailedTimeout);

        // Parse failure
        let cond = OutcomeConditions {
            specialist_responded: true,
            json_valid: false,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::FailedParse);

        // Probe failure
        let cond = OutcomeConditions {
            probes_failed: true,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::FailedProbes);

        // User cancelled
        let cond = OutcomeConditions {
            user_cancelled: true,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::AbortedByUser);

        // Clarification
        let cond = OutcomeConditions {
            specialist_responded: true,
            json_valid: true,
            schema_valid: true,
            clarification_asked: true,
            ..Default::default()
        };
        assert_eq!(cond.determine(), CanonicalOutcome::ClarificationNeeded);
    }
}
