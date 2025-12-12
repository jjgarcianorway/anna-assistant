//! Reason codes for debug diagnostics (v0.0.444).
//!
//! Machine-readable codes explaining why something happened.

use serde::{Deserialize, Serialize};

/// Standardized reason codes for debug output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonCode {
    // === Routing Issues ===
    /// Translator confidence below threshold
    RouteLowConfidence,
    /// Intent could not be determined
    RouteUnknownIntent,
    /// Domain could not be determined
    RouteUnknownDomain,
    /// No probes selected for query
    RouteNoProbes,

    // === Probe Issues ===
    /// Required probe was not run
    ProbeMissingRequired,
    /// Probe command failed (non-zero exit)
    ProbeFailedExit,
    /// Probe returned empty output
    ProbeEmptyOutput,
    /// Probe exceeded timeout
    ProbeTimeout,
    /// Probe command not found
    ProbeNotFound,

    // === LLM Timeouts ===
    /// Translator LLM timed out
    LlmTimeoutTranslator,
    /// Specialist LLM timed out
    LlmTimeoutSpecialist,
    /// Verifier/Supervisor LLM timed out
    LlmTimeoutVerifier,

    // === Parse/Validation Issues ===
    /// LLM output was not valid JSON
    LlmInvalidJson,
    /// JSON did not match expected schema
    ValidatorFailSchema,
    /// Answer lacked required evidence
    ValidatorFailEvidence,
    /// Validator flagged unsafe content
    ValidatorFailSafety,

    // === Fallback/Recovery ===
    /// Fallback answer was used
    FallbackUsed,
    /// Retry was attempted
    RetryAttempted,
    /// All retries exhausted
    RetriesExhausted,

    // === User Interaction ===
    /// Clarification question triggered
    ClarificationTriggered,
    /// User cancelled request
    UserCancelled,

    // === Success Indicators ===
    /// Request completed successfully
    Success,
    /// Fast path was used (no LLM)
    FastPathUsed,
    /// Recipe was used
    RecipeUsed,

    // === Internal Issues ===
    /// Internal error occurred
    InternalError,
    /// Budget exceeded
    BudgetExceeded,
}

impl ReasonCode {
    /// Get human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::RouteLowConfidence => "Translator confidence below threshold",
            Self::RouteUnknownIntent => "Could not determine query intent",
            Self::RouteUnknownDomain => "Could not determine query domain",
            Self::RouteNoProbes => "No probes selected for this query",
            Self::ProbeMissingRequired => "Required probe was not run",
            Self::ProbeFailedExit => "Probe command failed (non-zero exit)",
            Self::ProbeEmptyOutput => "Probe returned empty output",
            Self::ProbeTimeout => "Probe exceeded timeout",
            Self::ProbeNotFound => "Probe command not found",
            Self::LlmTimeoutTranslator => "Translator LLM timed out",
            Self::LlmTimeoutSpecialist => "Specialist LLM timed out",
            Self::LlmTimeoutVerifier => "Verifier LLM timed out",
            Self::LlmInvalidJson => "LLM output was not valid JSON",
            Self::ValidatorFailSchema => "Response did not match expected schema",
            Self::ValidatorFailEvidence => "Answer lacked required evidence",
            Self::ValidatorFailSafety => "Content flagged as unsafe",
            Self::FallbackUsed => "Fallback answer was used",
            Self::RetryAttempted => "Retry was attempted",
            Self::RetriesExhausted => "All retries exhausted",
            Self::ClarificationTriggered => "Asked clarification question",
            Self::UserCancelled => "User cancelled request",
            Self::Success => "Request completed successfully",
            Self::FastPathUsed => "Fast path used (no LLM)",
            Self::RecipeUsed => "Recipe was used",
            Self::InternalError => "Internal error occurred",
            Self::BudgetExceeded => "Time budget exceeded",
        }
    }

    /// Check if this is an error code.
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ProbeMissingRequired
                | Self::ProbeFailedExit
                | Self::ProbeTimeout
                | Self::ProbeNotFound
                | Self::LlmTimeoutTranslator
                | Self::LlmTimeoutSpecialist
                | Self::LlmTimeoutVerifier
                | Self::LlmInvalidJson
                | Self::ValidatorFailSchema
                | Self::ValidatorFailEvidence
                | Self::ValidatorFailSafety
                | Self::RetriesExhausted
                | Self::InternalError
                | Self::BudgetExceeded
        )
    }

    /// Check if this is a warning code.
    pub fn is_warning(&self) -> bool {
        matches!(
            self,
            Self::RouteLowConfidence
                | Self::RouteUnknownIntent
                | Self::RouteUnknownDomain
                | Self::RouteNoProbes
                | Self::ProbeEmptyOutput
                | Self::FallbackUsed
                | Self::RetryAttempted
        )
    }

    /// Check if this is a success/info code.
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::FastPathUsed | Self::RecipeUsed
        )
    }

    /// Get code as string for display.
    pub fn code(&self) -> &'static str {
        match self {
            Self::RouteLowConfidence => "ROUTE_LOW_CONFIDENCE",
            Self::RouteUnknownIntent => "ROUTE_UNKNOWN_INTENT",
            Self::RouteUnknownDomain => "ROUTE_UNKNOWN_DOMAIN",
            Self::RouteNoProbes => "ROUTE_NO_PROBES",
            Self::ProbeMissingRequired => "PROBE_MISSING_REQUIRED",
            Self::ProbeFailedExit => "PROBE_FAILED_EXIT",
            Self::ProbeEmptyOutput => "PROBE_EMPTY_OUTPUT",
            Self::ProbeTimeout => "PROBE_TIMEOUT",
            Self::ProbeNotFound => "PROBE_NOT_FOUND",
            Self::LlmTimeoutTranslator => "LLM_TIMEOUT_TRANSLATOR",
            Self::LlmTimeoutSpecialist => "LLM_TIMEOUT_SPECIALIST",
            Self::LlmTimeoutVerifier => "LLM_TIMEOUT_VERIFIER",
            Self::LlmInvalidJson => "LLM_INVALID_JSON",
            Self::ValidatorFailSchema => "VALIDATOR_FAIL_SCHEMA",
            Self::ValidatorFailEvidence => "VALIDATOR_FAIL_EVIDENCE",
            Self::ValidatorFailSafety => "VALIDATOR_FAIL_SAFETY",
            Self::FallbackUsed => "FALLBACK_USED",
            Self::RetryAttempted => "RETRY_ATTEMPTED",
            Self::RetriesExhausted => "RETRIES_EXHAUSTED",
            Self::ClarificationTriggered => "CLARIFICATION_TRIGGERED",
            Self::UserCancelled => "USER_CANCELLED",
            Self::Success => "SUCCESS",
            Self::FastPathUsed => "FAST_PATH_USED",
            Self::RecipeUsed => "RECIPE_USED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::BudgetExceeded => "BUDGET_EXCEEDED",
        }
    }
}

impl std::fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Collection of reason codes with helper methods.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReasonCodes {
    codes: Vec<ReasonCode>,
}

impl ReasonCodes {
    /// Create empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a reason code.
    pub fn add(&mut self, code: ReasonCode) {
        if !self.codes.contains(&code) {
            self.codes.push(code);
        }
    }

    /// Add multiple codes.
    pub fn add_all(&mut self, codes: &[ReasonCode]) {
        for code in codes {
            self.add(*code);
        }
    }

    /// Check if any errors.
    pub fn has_errors(&self) -> bool {
        self.codes.iter().any(|c| c.is_error())
    }

    /// Check if any warnings.
    pub fn has_warnings(&self) -> bool {
        self.codes.iter().any(|c| c.is_warning())
    }

    /// Get all codes.
    pub fn all(&self) -> &[ReasonCode] {
        &self.codes
    }

    /// Get error codes.
    pub fn errors(&self) -> Vec<ReasonCode> {
        self.codes.iter().filter(|c| c.is_error()).copied().collect()
    }

    /// Get warning codes.
    pub fn warnings(&self) -> Vec<ReasonCode> {
        self.codes.iter().filter(|c| c.is_warning()).copied().collect()
    }

    /// Format for display.
    pub fn display(&self) -> String {
        if self.codes.is_empty() {
            return "[]".to_string();
        }
        let codes: Vec<&str> = self.codes.iter().map(|c| c.code()).collect();
        format!("[{}]", codes.join(", "))
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.codes.is_empty()
    }

    /// Get count.
    pub fn len(&self) -> usize {
        self.codes.len()
    }
}

impl From<Vec<ReasonCode>> for ReasonCodes {
    fn from(codes: Vec<ReasonCode>) -> Self {
        Self { codes }
    }
}

impl IntoIterator for ReasonCodes {
    type Item = ReasonCode;
    type IntoIter = std::vec::IntoIter<ReasonCode>;

    fn into_iter(self) -> Self::IntoIter {
        self.codes.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reason_code_display() {
        assert_eq!(ReasonCode::LlmTimeoutTranslator.code(), "LLM_TIMEOUT_TRANSLATOR");
        assert_eq!(ReasonCode::ProbeFailedExit.code(), "PROBE_FAILED_EXIT");
    }

    #[test]
    fn test_reason_code_classification() {
        assert!(ReasonCode::LlmTimeoutTranslator.is_error());
        assert!(ReasonCode::RouteLowConfidence.is_warning());
        assert!(ReasonCode::Success.is_success());
        assert!(!ReasonCode::Success.is_error());
    }

    #[test]
    fn test_reason_codes_collection() {
        let mut codes = ReasonCodes::new();
        codes.add(ReasonCode::RouteLowConfidence);
        codes.add(ReasonCode::LlmTimeoutSpecialist);
        codes.add(ReasonCode::RouteLowConfidence); // Duplicate

        assert_eq!(codes.len(), 2);
        assert!(codes.has_errors());
        assert!(codes.has_warnings());

        let display = codes.display();
        assert!(display.contains("ROUTE_LOW_CONFIDENCE"));
        assert!(display.contains("LLM_TIMEOUT_SPECIALIST"));
    }
}
