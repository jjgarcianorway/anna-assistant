//! Validation types and error definitions (v0.0.428).

use super::ResponseStatus;

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the response is valid
    pub valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings (response still usable)
    pub warnings: Vec<String>,
    /// Adjusted status (may be downgraded)
    pub adjusted_status: ResponseStatus,
    /// Adjusted confidence (may be lowered)
    pub adjusted_confidence: f32,
}

/// Types of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Response contains invented/nonsense data
    InventedData(String),
    /// Generic how-to when user asked for state
    GenericHowTo,
    /// Missing required evidence for success status
    MissingEvidence,
    /// Summary doesn't match evidence
    SummaryEvidenceMismatch(String),
    /// Forbidden pattern detected
    ForbiddenPattern(String),
    /// Confidence out of range
    InvalidConfidence(f32),
    /// Summary too long
    SummaryTooLong(usize),
    /// Too many key facts
    TooManyKeyFacts(usize),
    /// Empty summary for success status
    EmptySummary,
    /// Vague language in success response
    VagueLanguage(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InventedData(s) => write!(f, "Invented data detected: {}", s),
            Self::GenericHowTo => write!(f, "Generic how-to when user asked for current state"),
            Self::MissingEvidence => write!(f, "Success status but no evidence provided"),
            Self::SummaryEvidenceMismatch(s) => write!(f, "Summary doesn't match evidence: {}", s),
            Self::ForbiddenPattern(s) => write!(f, "Forbidden pattern: {}", s),
            Self::InvalidConfidence(c) => write!(f, "Invalid confidence: {}", c),
            Self::SummaryTooLong(len) => write!(f, "Summary too long: {} chars", len),
            Self::TooManyKeyFacts(n) => write!(f, "Too many key facts: {}", n),
            Self::EmptySummary => write!(f, "Empty summary for success status"),
            Self::VagueLanguage(s) => write!(f, "Vague language in success response: {}", s),
        }
    }
}
