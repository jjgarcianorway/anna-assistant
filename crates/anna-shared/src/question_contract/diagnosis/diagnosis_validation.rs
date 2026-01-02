//! Diagnosis Validation - v0.0.437.
//!
//! Validates diagnosis conclusions and language usage.

use super::diagnosis_types::{ConclusionState, DiagnosisConclusion};

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

impl DiagnosisConclusion {
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
                            reason: "Likely conclusion doesn't mention the primary cause"
                                .to_string(),
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
                        reason: "No-issue conclusion should explicitly state no issue found"
                            .to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_likely_needs_cause() {
        let mut conclusion =
            DiagnosisConclusion::likely("test", 0.8, vec!["ev1".to_string()]);
        conclusion.primary_cause = None;

        let validation = conclusion.validate();
        assert!(!validation.is_valid());
    }

    #[test]
    fn test_language_validation_uncertain() {
        let conclusion = DiagnosisConclusion::uncertain(
            vec!["option A".to_string()],
            vec!["ev1".to_string()],
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
}
