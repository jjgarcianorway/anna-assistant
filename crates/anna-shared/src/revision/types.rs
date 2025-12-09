//! Type definitions for revision module (v0.0.208).

use serde::{Deserialize, Serialize};

/// Issue categories for revision instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionIssue {
    /// Answer lacks required evidence claims
    MissingEvidence,
    /// Answer contradicts collected evidence
    Contradiction,
    /// Answer is too vague or unspecific
    TooVague,
    /// Answer includes unverifiable claims
    UnverifiableClaims,
    /// Answer missing required probe data
    MissingProbes,
    /// Answer exceeds confidence bounds
    OverConfident,
    /// Answer format needs improvement
    FormatIssue,
}

impl std::fmt::Display for RevisionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEvidence => write!(f, "missing evidence"),
            Self::Contradiction => write!(f, "contradiction"),
            Self::TooVague => write!(f, "too vague"),
            Self::UnverifiableClaims => write!(f, "unverifiable claims"),
            Self::MissingProbes => write!(f, "missing probes"),
            Self::OverConfident => write!(f, "over-confident"),
            Self::FormatIssue => write!(f, "format issue"),
        }
    }
}

/// Revision instruction from junior or senior reviewer
///
/// Contains structured feedback for Anna to apply deterministically:
/// - Issues identified in the current answer
/// - Claims that must be included (with evidence refs)
/// - Claims that must be removed (unverifiable)
/// - Additional probes to run if evidence is missing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevisionInstruction {
    /// Issues identified (templated categories)
    pub issues: Vec<RevisionIssue>,
    /// Claims that must be included (e.g., "include / percent_used")
    pub required_claims: Vec<String>,
    /// Claims that must be removed (unverifiable)
    pub forbidden_claims: Vec<String>,
    /// Additional probes to run if evidence_required and probes missing
    pub recommended_probes: Vec<String>,
    /// Free-form explanation for transcript (not used for logic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl RevisionInstruction {
    /// Create an empty instruction (no changes needed)
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if this instruction requires any changes
    pub fn has_changes(&self) -> bool {
        !self.issues.is_empty()
            || !self.required_claims.is_empty()
            || !self.forbidden_claims.is_empty()
            || !self.recommended_probes.is_empty()
    }

    /// Add an issue
    pub fn with_issue(mut self, issue: RevisionIssue) -> Self {
        if !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
        self
    }

    /// Add a required claim
    pub fn with_required_claim(mut self, claim: impl Into<String>) -> Self {
        self.required_claims.push(claim.into());
        self
    }

    /// Add a forbidden claim
    pub fn with_forbidden_claim(mut self, claim: impl Into<String>) -> Self {
        self.forbidden_claims.push(claim.into());
        self
    }

    /// Add a recommended probe
    pub fn with_recommended_probe(mut self, probe: impl Into<String>) -> Self {
        self.recommended_probes.push(probe.into());
        self
    }

    /// Add explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }

    /// Format as a concise summary for transcript
    pub fn summary(&self) -> String {
        if !self.has_changes() {
            return "no changes needed".to_string();
        }

        let mut parts = Vec::new();

        if !self.issues.is_empty() {
            let issues: Vec<_> = self.issues.iter().map(|i| i.to_string()).collect();
            parts.push(format!("issues=[{}]", issues.join(", ")));
        }

        if !self.required_claims.is_empty() {
            parts.push(format!(
                "required_claims=[{}]",
                self.required_claims.join(", ")
            ));
        }

        if !self.forbidden_claims.is_empty() {
            parts.push(format!(
                "forbidden_claims=[{}]",
                self.forbidden_claims.join(", ")
            ));
        }

        if !self.recommended_probes.is_empty() {
            parts.push(format!(
                "recommended_probes=[{}]",
                self.recommended_probes.join(", ")
            ));
        }

        parts.join(" ")
    }
}
