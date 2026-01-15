//! LLM Core types and structures

/// Investigation state tracks what we've learned
#[derive(Debug, Default)]
pub struct InvestigationState {
    /// Commands we've run and their outputs
    pub findings: Vec<Finding>,
    /// What we still need to find out
    pub open_questions: Vec<String>,
    /// Current iteration
    pub iteration: u8,
}

/// A single finding from command execution
#[derive(Debug, Clone)]
pub struct Finding {
    pub command: String,
    pub output: String,
    pub success: bool,
}

/// Result of the LLM deciding next steps
#[derive(Debug)]
pub enum NextStep {
    /// Run these commands to gather more info
    Investigate(Vec<String>),
    /// Have enough info, generate answer
    Answer,
    /// Found a problem, suggest a fix
    SuggestFix { problem: String, fix_command: String, explanation: String },
    /// Can't help with this
    OutOfScope(String),
}

/// v0.3.25: Result of ClaimGate verification
pub struct VerificationResult {
    /// The answer text (may have [unverified] markers)
    pub answer: String,
    /// Evidence line to append (formatted)
    pub evidence_line: String,
    /// Whether investigation is needed (unverified FACT claims with low confidence)
    pub needs_investigation: bool,
    /// Suggested probes if investigation is needed
    pub suggested_probes: Vec<String>,
    /// Number of verified claims
    pub verified_count: usize,
    /// Number of unverified claims
    pub unverified_count: usize,
}

/// Understanding result from LLM
#[derive(Debug)]
pub struct Understanding {
    /// What type of question is this
    #[allow(dead_code)]
    pub intent: String,
    /// What information do we need to answer
    #[allow(dead_code)]
    pub info_needed: Vec<String>,
    /// Is this out of scope?
    pub out_of_scope_reason: Option<String>,
}
