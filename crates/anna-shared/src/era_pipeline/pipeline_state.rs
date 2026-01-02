//! ERA Pipeline state machine.

use super::evidence::EvidenceBundle;
use super::reasoning_types::ReasoningOutput;
use super::pipeline_types::{ExtractedIntent, PipelineStage};

/// ERA Pipeline state machine.
#[derive(Debug, Clone)]
pub struct EraPipeline {
    /// Case ID.
    pub case_id: String,
    /// Current stage.
    pub stage: PipelineStage,
    /// Extracted intent (set after Evidence stage).
    pub intent: Option<ExtractedIntent>,
    /// Evidence bundle (set after Evidence stage).
    pub evidence: Option<EvidenceBundle>,
    /// Reasoning output (set after Reasoning stage).
    pub reasoning: Option<ReasoningOutput>,
    /// Final answer (set after Answer stage).
    pub answer: Option<String>,
    /// Pipeline errors.
    pub errors: Vec<PipelineError>,
}

/// Pipeline error.
#[derive(Debug, Clone)]
pub struct PipelineError {
    /// Stage where error occurred.
    pub stage: PipelineStage,
    /// Error message.
    pub message: String,
    /// Whether pipeline can continue.
    pub recoverable: bool,
}

impl EraPipeline {
    /// Create new pipeline.
    pub fn new(case_id: &str) -> Self {
        Self {
            case_id: case_id.to_string(),
            stage: PipelineStage::Evidence,
            intent: None,
            evidence: None,
            reasoning: None,
            answer: None,
            errors: Vec::new(),
        }
    }

    /// Check if pipeline can proceed to next stage.
    pub fn can_proceed(&self) -> bool {
        match self.stage {
            PipelineStage::Evidence => self.intent.is_some() && self.evidence.is_some(),
            PipelineStage::Reasoning => self.reasoning.is_some(),
            PipelineStage::Answer => false, // Terminal stage
        }
    }

    /// Advance to next stage.
    pub fn advance(&mut self) -> Result<(), PipelineError> {
        if !self.can_proceed() {
            return Err(PipelineError {
                stage: self.stage,
                message: format!(
                    "Cannot proceed from {} - prerequisites not met",
                    self.stage.label()
                ),
                recoverable: false,
            });
        }

        if let Some(next) = self.stage.next() {
            self.stage = next;
            Ok(())
        } else {
            Err(PipelineError {
                stage: self.stage,
                message: "Pipeline already complete".to_string(),
                recoverable: false,
            })
        }
    }

    /// Set evidence stage results.
    pub fn set_evidence(&mut self, intent: ExtractedIntent, evidence: EvidenceBundle) {
        self.intent = Some(intent);
        self.evidence = Some(evidence);
    }

    /// Set reasoning stage results.
    pub fn set_reasoning(&mut self, reasoning: ReasoningOutput) {
        self.reasoning = Some(reasoning);
    }

    /// Set answer stage results.
    pub fn set_answer(&mut self, answer: String) {
        self.answer = Some(answer);
    }

    /// Record an error.
    pub fn record_error(&mut self, error: PipelineError) {
        self.errors.push(error);
    }

    /// Check if pipeline completed successfully.
    pub fn is_complete(&self) -> bool {
        self.stage == PipelineStage::Answer && self.answer.is_some()
    }

    /// Check if pipeline failed.
    pub fn is_failed(&self) -> bool {
        self.errors.iter().any(|e| !e.recoverable)
    }

    /// Get completion status.
    pub fn status(&self) -> PipelineStatus {
        if self.is_complete() {
            PipelineStatus::Complete
        } else if self.is_failed() {
            PipelineStatus::Failed
        } else {
            PipelineStatus::InProgress(self.stage)
        }
    }
}

/// Pipeline completion status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStatus {
    /// Pipeline in progress at stage.
    InProgress(PipelineStage),
    /// Pipeline completed successfully.
    Complete,
    /// Pipeline failed.
    Failed,
}
