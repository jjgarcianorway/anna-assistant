//! Types and constants for answer validation.

/// Maximum self-healing attempts before giving up
pub const MAX_HEAL_ATTEMPTS: u8 = 3;

/// Base minimum acceptable reliability score (used when domain unknown)
pub const BASE_ACCEPTABLE_SCORE: u8 = 80;

/// Result of answer validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The (possibly revised) answer
    pub answer: String,
    /// Final reliability score
    pub score: u8,
    /// Whether the answer passed validation
    pub passed: bool,
    /// Number of heal attempts made
    pub heal_attempts: u8,
    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,
    /// Detailed validation path for debugging
    pub validation_path: Vec<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationIssue {
    /// Claims not grounded in evidence
    UngroundedClaims { count: usize },
    /// Invented facts detected
    InventionDetected { claim: String },
    /// Missing required evidence
    MissingEvidence { kind: String },
    /// Answer too vague
    TooVague,
    /// Low confidence from translator
    LowConfidence { confidence: f32 },
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UngroundedClaims { count } => write!(f, "{} ungrounded claims", count),
            Self::InventionDetected { claim } => write!(f, "invented: {}", claim),
            Self::MissingEvidence { kind } => write!(f, "missing {} evidence", kind),
            Self::TooVague => write!(f, "answer too vague"),
            Self::LowConfidence { confidence } => {
                write!(f, "low confidence: {:.0}%", confidence * 100.0)
            }
        }
    }
}
