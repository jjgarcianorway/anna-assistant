//! Gate outcome types and messages.

use serde::{Deserialize, Serialize};

/// Outcome of the reliability gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateOutcome {
    /// All checks passed - answer can be shown
    Pass,
    /// Failed - no evidence for claims
    FailedNoEvidence,
    /// Failed - timeout in required stage
    FailedTimeout,
    /// Failed - parsing error occurred
    FailedParse,
    /// Failed - low confidence answer
    FailedLowConfidence,
    /// Failed - query was ambiguous
    FailedAmbiguousQuery,
    /// Failed - answer shape doesn't match question
    FailedContractViolation,
    /// Failed - no claims in answer
    FailedNoClaims,
    /// Failed - generic/irrelevant answer detected
    FailedGenericAnswer,
    /// Failed - answer doesn't match question type
    FailedQuestionMismatch,
    /// Failed - domain doesn't match probes
    FailedDomainMismatch,
    /// Failed - hallucinated entity detected
    FailedHallucination,
    /// Failed - probe failed or returned empty
    FailedProbeCoverage,
}

impl GateOutcome {
    /// Check if this is a success outcome.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Get user-facing failure message.
    pub fn failure_message(&self) -> &'static str {
        match self {
            Self::Pass => "",
            Self::FailedNoEvidence => "I don't have enough verified data to answer this yet.",
            Self::FailedTimeout => "I couldn't complete the analysis in time. Please try again.",
            Self::FailedParse => "I encountered an internal error processing this request.",
            Self::FailedLowConfidence => "I'm not confident enough in my answer to show it.",
            Self::FailedAmbiguousQuery => "I need more details to answer this accurately.",
            Self::FailedContractViolation => "I couldn't produce an answer in the expected format.",
            Self::FailedNoClaims => "I don't have any verified information to share.",
            Self::FailedGenericAnswer => {
                "I don't have specific information to answer this question."
            }
            Self::FailedQuestionMismatch => {
                "My answer doesn't match what you asked. Let me try again."
            }
            Self::FailedDomainMismatch => {
                "I gathered information from the wrong area. Let me refocus."
            }
            Self::FailedHallucination => "I couldn't verify some details in my answer.",
            Self::FailedProbeCoverage => "Some system checks failed or returned incomplete data.",
        }
    }

    /// Get code for metrics/logging.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::FailedNoEvidence => "FAILED_NO_EVIDENCE",
            Self::FailedTimeout => "FAILED_TIMEOUT",
            Self::FailedParse => "FAILED_PARSE",
            Self::FailedLowConfidence => "FAILED_LOW_CONFIDENCE",
            Self::FailedAmbiguousQuery => "FAILED_AMBIGUOUS_QUERY",
            Self::FailedContractViolation => "FAILED_CONTRACT_VIOLATION",
            Self::FailedNoClaims => "FAILED_NO_CLAIMS",
            Self::FailedGenericAnswer => "FAILED_GENERIC_ANSWER",
            Self::FailedQuestionMismatch => "FAILED_QUESTION_MISMATCH",
            Self::FailedDomainMismatch => "FAILED_DOMAIN_MISMATCH",
            Self::FailedHallucination => "FAILED_HALLUCINATION",
            Self::FailedProbeCoverage => "FAILED_PROBE_COVERAGE",
        }
    }

    /// Check if this is a timeout outcome.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::FailedTimeout)
    }

    /// Check if this is any failure outcome.
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}
