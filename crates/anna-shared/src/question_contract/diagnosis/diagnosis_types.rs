//! Diagnosis Conclusion Types - v0.0.437.
//!
//! Core types for diagnosis conclusions.

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

    /// Render the conclusion as user-facing text.
    pub fn render(&self) -> String {
        match self.conclusion {
            ConclusionState::Likely => {
                let cause = self.primary_cause.as_deref().unwrap_or("unknown cause");
                if self.confidence >= 0.8 {
                    format!("The issue is most likely caused by: {}", cause)
                } else {
                    format!(
                        "The probable cause is: {} (confidence: {:.0}%)",
                        cause,
                        self.confidence * 100.0
                    )
                }
            }
            ConclusionState::Uncertain => {
                let mut parts =
                    vec!["Unable to determine the root cause with confidence.".to_string()];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_likely_conclusion() {
        let conclusion =
            DiagnosisConclusion::likely("slow disk I/O", 0.85, vec!["ev_iostat".to_string()]);

        assert_eq!(conclusion.conclusion, ConclusionState::Likely);
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
    fn test_conclusion_state_hedging() {
        assert!(ConclusionState::Uncertain.required_hedging().is_some());
        assert!(ConclusionState::Likely.required_hedging().is_none());
    }
}
