//! Diagnosis Builder - v0.0.437.
//!
//! Builder pattern for constructing diagnosis conclusions.

use super::diagnosis_types::{ConclusionState, DiagnosisConclusion};

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
}
