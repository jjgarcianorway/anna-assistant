//! Resolution types and criteria.

use serde::{Deserialize, Serialize};

use crate::era_pipeline::pipeline::EraPipeline;

/// Resolution status (honest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolutionStatus {
    /// Fully resolved: can_answer=true, missing=[], answer delivered.
    Resolved,
    /// Partially resolved: answer given but with caveats.
    Partial,
    /// Cannot answer: missing facts, specialist said no.
    CannotAnswer,
    /// Failed: pipeline error, timeout, etc.
    Failed,
    /// In progress.
    InProgress,
}

impl ResolutionStatus {
    /// Get label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Resolved => "RESOLVED",
            Self::Partial => "PARTIAL",
            Self::CannotAnswer => "CANNOT_ANSWER",
            Self::Failed => "FAILED",
            Self::InProgress => "IN_PROGRESS",
        }
    }

    /// Is this a success for metrics?
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Resolved)
    }

    /// Is this a failure for metrics?
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::CannotAnswer | Self::Failed)
    }
}

/// Resolution criteria (strict).
#[derive(Debug, Clone)]
pub struct ResolutionCriteria {
    /// Reasoning says can_answer.
    pub can_answer: bool,
    /// No missing facts.
    pub no_missing: bool,
    /// Answer was delivered.
    pub answer_delivered: bool,
    /// Confidence threshold met.
    pub confidence_ok: bool,
}

impl ResolutionCriteria {
    /// Check from pipeline state.
    pub fn from_pipeline(pipeline: &EraPipeline, confidence_threshold: f64) -> Self {
        let can_answer = pipeline
            .reasoning
            .as_ref()
            .map(|r| r.can_answer)
            .unwrap_or(false);

        let no_missing = pipeline
            .evidence
            .as_ref()
            .map(|e| e.missing.is_empty())
            .unwrap_or(false);

        let answer_delivered = pipeline.answer.is_some();

        let confidence_ok = pipeline
            .reasoning
            .as_ref()
            .map(|r| r.confidence >= confidence_threshold)
            .unwrap_or(false);

        Self {
            can_answer,
            no_missing,
            answer_delivered,
            confidence_ok,
        }
    }

    /// Is fully resolved?
    pub fn is_resolved(&self) -> bool {
        self.can_answer && self.no_missing && self.answer_delivered
    }

    /// Is partially resolved?
    pub fn is_partial(&self) -> bool {
        self.answer_delivered && (!self.can_answer || !self.no_missing || !self.confidence_ok)
    }

    /// Get resolution status.
    pub fn status(&self) -> ResolutionStatus {
        if self.is_resolved() && self.confidence_ok {
            ResolutionStatus::Resolved
        } else if self.is_partial() {
            ResolutionStatus::Partial
        } else if !self.can_answer {
            ResolutionStatus::CannotAnswer
        } else {
            ResolutionStatus::Failed
        }
    }

    /// Get failure reasons.
    pub fn failure_reasons(&self) -> Vec<&'static str> {
        let mut reasons = Vec::new();
        if !self.can_answer {
            reasons.push("Specialist cannot answer from evidence");
        }
        if !self.no_missing {
            reasons.push("Missing required facts");
        }
        if !self.answer_delivered {
            reasons.push("Answer not delivered");
        }
        if !self.confidence_ok {
            reasons.push("Confidence below threshold");
        }
        reasons
    }
}

/// Resolution reason for user feedback.
#[derive(Debug, Clone)]
pub struct ResolutionReason {
    /// Status.
    pub status: ResolutionStatus,
    /// Human-readable reason.
    pub reason: String,
    /// Missing facts (if any).
    pub missing: Vec<String>,
}

impl ResolutionReason {
    /// Build from pipeline.
    pub fn from_pipeline(pipeline: &EraPipeline, threshold: f64) -> Self {
        use super::validator::validate_resolution;

        let status = validate_resolution(pipeline, threshold);
        let criteria = ResolutionCriteria::from_pipeline(pipeline, threshold);

        let reason = match status {
            ResolutionStatus::Resolved => "Question answered successfully.".to_string(),
            ResolutionStatus::Partial => {
                let reasons = criteria.failure_reasons();
                format!("Partial answer. Issues: {}", reasons.join(", "))
            }
            ResolutionStatus::CannotAnswer => {
                if let Some(reasoning) = &pipeline.reasoning {
                    if !reasoning.requires.is_empty() {
                        format!("Cannot answer. Need: {}", reasoning.requires.join(", "))
                    } else {
                        "Cannot answer from available evidence.".to_string()
                    }
                } else {
                    "Cannot answer. Reasoning stage failed.".to_string()
                }
            }
            ResolutionStatus::Failed => {
                if !pipeline.errors.is_empty() {
                    format!("Failed: {}", pipeline.errors[0].message)
                } else {
                    "Pipeline failed.".to_string()
                }
            }
            ResolutionStatus::InProgress => {
                format!("In progress at stage: {}", pipeline.stage.label())
            }
        };

        let missing = pipeline
            .evidence
            .as_ref()
            .map(|e| e.missing.clone())
            .unwrap_or_default();

        Self {
            status,
            reason,
            missing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::era_pipeline::evidence::EvidenceBundle;
    use crate::era_pipeline::ReasoningOutput;

    #[test]
    fn test_resolution_criteria_resolved() {
        let mut pipeline = EraPipeline::new("DSK-0127");
        pipeline.evidence = Some(EvidenceBundle::new("DSK-0127"));
        pipeline.reasoning = Some(ReasoningOutput::answerable("DSK-0127", "Test", 0.9));
        pipeline.answer = Some("17.0 GiB".to_string());

        let criteria = ResolutionCriteria::from_pipeline(&pipeline, 0.6);
        assert!(criteria.is_resolved());
        assert_eq!(criteria.status(), ResolutionStatus::Resolved);
    }

    #[test]
    fn test_resolution_criteria_cannot_answer() {
        let mut pipeline = EraPipeline::new("DSK-0127");
        pipeline.evidence = Some(EvidenceBundle::new("DSK-0127"));
        pipeline.reasoning = Some(ReasoningOutput::unanswerable(
            "DSK-0127",
            "Missing data",
            vec!["boot.blame"],
        ));

        let criteria = ResolutionCriteria::from_pipeline(&pipeline, 0.6);
        assert!(!criteria.can_answer);
        assert_eq!(criteria.status(), ResolutionStatus::CannotAnswer);
    }
}
