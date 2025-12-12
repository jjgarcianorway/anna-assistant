//! Diagnosis Conclusion Requirements (Part E) - v0.0.437.
//!
//! For category=diagnosis:
//! - A diagnosis is incomplete without a conclusion state
//! - If conclusion=uncertain, Anna must explicitly say uncertainty
//! - No confident language allowed when uncertain

use serde::{Deserialize, Serialize};

/// The conclusion state of a diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionState {
    /// Root cause is likely identified.
    Likely,
    /// Cannot determine with confidence.
    Uncertain,
    /// No issue was detected.
    NoIssueDetected,
}

impl ConclusionState {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Likely => "likely",
            Self::Uncertain => "uncertain",
            Self::NoIssueDetected => "no_issue_detected",
        }
    }

    /// Whether confident language is allowed.
    pub fn allows_confident_language(&self) -> bool {
        matches!(self, Self::Likely | Self::NoIssueDetected)
    }

    /// Get required hedging phrases for uncertain conclusions.
    pub fn required_hedging(&self) -> Option<&'static [&'static str]> {
        match self {
            Self::Uncertain => Some(&[
                "may be",
                "might be",
                "could be",
                "possibly",
                "uncertain",
                "unable to determine",
                "not enough evidence",
            ]),
            _ => None,
        }
    }
}

/// A complete diagnosis conclusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisConclusion {
    /// The conclusion state.
    pub conclusion: ConclusionState,
    /// Primary cause identified (if any).
    pub primary_cause: Option<String>,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f64,
    /// Evidence IDs supporting the conclusion.
    pub supporting_evidence: Vec<String>,
    /// Alternative causes considered.
    pub alternatives: Vec<String>,
}

impl DiagnosisConclusion {
    /// Create a "likely" conclusion.
    pub fn likely(cause: &str, confidence: f64, evidence: Vec<String>) -> Self {
        Self {
            conclusion: ConclusionState::Likely,
            primary_cause: Some(cause.to_string()),
            confidence: confidence.clamp(0.0, 1.0),
            supporting_evidence: evidence,
            alternatives: Vec::new(),
        }
    }

    /// Create an "uncertain" conclusion.
    pub fn uncertain(alternatives: Vec<String>, evidence: Vec<String>) -> Self {
        Self {
            conclusion: ConclusionState::Uncertain,
            primary_cause: None,
            confidence: 0.0,
            supporting_evidence: evidence,
            alternatives,
        }
    }

    /// Create a "no issue" conclusion.
    pub fn no_issue(confidence: f64, evidence: Vec<String>) -> Self {
        Self {
            conclusion: ConclusionState::NoIssueDetected,
            primary_cause: None,
            confidence: confidence.clamp(0.0, 1.0),
            supporting_evidence: evidence,
            alternatives: Vec::new(),
        }
    }

    /// Add alternative causes.
    pub fn with_alternatives(mut self, alternatives: Vec<String>) -> Self {
        self.alternatives = alternatives;
        self
    }

    /// Validate the conclusion is complete.
    pub fn validate(&self) -> ConclusionValidation {
        let mut issues = Vec::new();

        // Likely conclusions must have a cause
        if self.conclusion == ConclusionState::Likely && self.primary_cause.is_none() {
            issues.push("Likely conclusion missing primary cause".to_string());
        }

        // Likely conclusions should have evidence
        if self.conclusion == ConclusionState::Likely && self.supporting_evidence.is_empty() {
            issues.push("Likely conclusion has no supporting evidence".to_string());
        }

        // Uncertain conclusions should have alternatives
        if self.conclusion == ConclusionState::Uncertain && self.alternatives.is_empty() {
            issues.push("Uncertain conclusion should list alternatives".to_string());
        }

        // Confidence should match conclusion state
        if self.conclusion == ConclusionState::Uncertain && self.confidence > 0.5 {
            issues.push("Uncertain conclusion has high confidence".to_string());
        }

        if issues.is_empty() {
            ConclusionValidation::Valid
        } else {
            ConclusionValidation::Invalid { issues }
        }
    }

    /// Render the conclusion as user-facing text.
    pub fn render(&self) -> String {
        match self.conclusion {
            ConclusionState::Likely => {
                let cause = self.primary_cause.as_deref().unwrap_or("unknown cause");
                if self.confidence >= 0.8 {
                    format!("The issue is most likely caused by: {}", cause)
                } else {
                    format!("The probable cause is: {} (confidence: {:.0}%)", cause, self.confidence * 100.0)
                }
            }
            ConclusionState::Uncertain => {
                let mut parts = vec!["Unable to determine the root cause with confidence.".to_string()];
                if !self.alternatives.is_empty() {
                    parts.push(format!("Possible causes: {}", self.alternatives.join(", ")));
                }
                parts.join("\n")
            }
            ConclusionState::NoIssueDetected => {
                "No issue detected based on the available evidence.".to_string()
            }
        }
    }
}

/// Conclusion validation result.
#[derive(Debug, Clone)]
pub enum ConclusionValidation {
    /// Conclusion is valid.
    Valid,
    /// Conclusion has issues.
    Invalid { issues: Vec<String> },
}

impl ConclusionValidation {
    /// Check if valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Validates that diagnosis text matches the conclusion state.
pub struct ConclusionLanguageValidator;

impl ConclusionLanguageValidator {
    /// Check if text uses appropriate language for the conclusion.
    pub fn validate(conclusion: &DiagnosisConclusion, text: &str) -> LanguageValidation {
        let lower = text.to_lowercase();

        match conclusion.conclusion {
            ConclusionState::Uncertain => {
                // Uncertain conclusions must NOT use confident language
                let confident_phrases = [
                    "the cause is",
                    "definitely",
                    "certainly",
                    "without doubt",
                    "clearly",
                    "obviously",
                    "the problem is",
                    "this is caused by",
                ];

                let uses_confident = confident_phrases.iter().any(|p| lower.contains(p));

                // Should use hedging phrases
                let hedging = conclusion.conclusion.required_hedging().unwrap_or(&[]);
                let uses_hedging = hedging.iter().any(|p| lower.contains(p));

                if uses_confident {
                    LanguageValidation::Invalid {
                        reason: "Uncertain conclusion uses confident language".to_string(),
                    }
                } else if !uses_hedging {
                    LanguageValidation::Warning {
                        reason: "Uncertain conclusion should use hedging phrases".to_string(),
                    }
                } else {
                    LanguageValidation::Valid
                }
            }
            ConclusionState::Likely => {
                // Likely conclusions should state the cause
                if conclusion.primary_cause.is_some() {
                    let cause = conclusion.primary_cause.as_ref().unwrap().to_lowercase();
                    if !lower.contains(&cause) {
                        LanguageValidation::Warning {
                            reason: "Likely conclusion doesn't mention the primary cause".to_string(),
                        }
                    } else {
                        LanguageValidation::Valid
                    }
                } else {
                    LanguageValidation::Valid
                }
            }
            ConclusionState::NoIssueDetected => {
                // Should indicate no issue
                let no_issue_phrases = ["no issue", "no problem", "working correctly", "normal"];
                let indicates_no_issue = no_issue_phrases.iter().any(|p| lower.contains(p));

                if !indicates_no_issue {
                    LanguageValidation::Warning {
                        reason: "No-issue conclusion should explicitly state no issue found".to_string(),
                    }
                } else {
                    LanguageValidation::Valid
                }
            }
        }
    }
}

/// Language validation result.
#[derive(Debug, Clone)]
pub enum LanguageValidation {
    /// Language is appropriate.
    Valid,
    /// Language could be better.
    Warning { reason: String },
    /// Language is inappropriate for conclusion.
    Invalid { reason: String },
}

impl LanguageValidation {
    /// Check if valid (not invalid).
    pub fn is_valid(&self) -> bool {
        !matches!(self, Self::Invalid { .. })
    }

    /// Check if has warnings.
    pub fn has_warning(&self) -> bool {
        matches!(self, Self::Warning { .. })
    }
}

/// Builder for diagnosis conclusions.
pub struct DiagnosisBuilder {
    conclusion: DiagnosisConclusion,
}

impl DiagnosisBuilder {
    /// Start with uncertain state.
    pub fn new() -> Self {
        Self {
            conclusion: DiagnosisConclusion::uncertain(Vec::new(), Vec::new()),
        }
    }

    /// Mark as likely with a cause.
    pub fn likely(mut self, cause: &str) -> Self {
        self.conclusion.conclusion = ConclusionState::Likely;
        self.conclusion.primary_cause = Some(cause.to_string());
        self
    }

    /// Mark as no issue detected.
    pub fn no_issue(mut self) -> Self {
        self.conclusion.conclusion = ConclusionState::NoIssueDetected;
        self.conclusion.primary_cause = None;
        self
    }

    /// Set confidence.
    pub fn confidence(mut self, confidence: f64) -> Self {
        self.conclusion.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add supporting evidence.
    pub fn evidence(mut self, evidence_ids: Vec<String>) -> Self {
        self.conclusion.supporting_evidence = evidence_ids;
        self
    }

    /// Add alternative causes.
    pub fn alternatives(mut self, alternatives: Vec<&str>) -> Self {
        self.conclusion.alternatives = alternatives.into_iter().map(String::from).collect();
        self
    }

    /// Build the conclusion.
    pub fn build(self) -> DiagnosisConclusion {
        self.conclusion
    }
}

impl Default for DiagnosisBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_likely_conclusion() {
        let conclusion = DiagnosisConclusion::likely(
            "slow disk I/O",
            0.85,
            vec!["ev_iostat".to_string()],
        );

        assert_eq!(conclusion.conclusion, ConclusionState::Likely);
        assert!(conclusion.validate().is_valid());
        assert!(conclusion.render().contains("slow disk I/O"));
    }

    #[test]
    fn test_uncertain_conclusion() {
        let conclusion = DiagnosisConclusion::uncertain(
            vec!["memory pressure".to_string(), "CPU throttling".to_string()],
            vec!["ev_1".to_string()],
        );

        assert_eq!(conclusion.conclusion, ConclusionState::Uncertain);
        assert!(!conclusion.conclusion.allows_confident_language());

        let rendered = conclusion.render();
        assert!(rendered.contains("Unable to determine"));
    }

    #[test]
    fn test_no_issue_conclusion() {
        let conclusion = DiagnosisConclusion::no_issue(0.9, vec!["ev_check".to_string()]);

        assert_eq!(conclusion.conclusion, ConclusionState::NoIssueDetected);
        assert!(conclusion.render().contains("No issue detected"));
    }

    #[test]
    fn test_validation_likely_needs_cause() {
        let mut conclusion = DiagnosisConclusion::likely("test", 0.8, vec![]);
        conclusion.primary_cause = None;

        let validation = conclusion.validate();
        assert!(!validation.is_valid());
    }

    #[test]
    fn test_language_validation_uncertain() {
        let conclusion = DiagnosisConclusion::uncertain(
            vec!["option A".to_string()],
            vec![],
        );

        // Bad: confident language
        let bad_text = "The cause is definitely memory.";
        let result = ConclusionLanguageValidator::validate(&conclusion, bad_text);
        assert!(!result.is_valid());

        // Good: hedging language
        let good_text = "The cause might be memory, but I'm uncertain.";
        let result = ConclusionLanguageValidator::validate(&conclusion, good_text);
        assert!(result.is_valid());
    }

    #[test]
    fn test_diagnosis_builder() {
        let conclusion = DiagnosisBuilder::new()
            .likely("memory leak")
            .confidence(0.75)
            .evidence(vec!["ev_mem".to_string()])
            .alternatives(vec!["CPU issue"])
            .build();

        assert_eq!(conclusion.conclusion, ConclusionState::Likely);
        assert_eq!(conclusion.confidence, 0.75);
        assert!(!conclusion.supporting_evidence.is_empty());
    }

    #[test]
    fn test_conclusion_state_hedging() {
        assert!(ConclusionState::Uncertain.required_hedging().is_some());
        assert!(ConclusionState::Likely.required_hedging().is_none());
    }
}
